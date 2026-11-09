//! `host.run`: roda um programa para um plugin, numa thread, com prazo e saída limitada.
//!
//! O processo roda num grupo próprio: ao estourar o prazo, o grupo inteiro morre (um `sh -c` leva os filhos junto).

use std::io::{Read, Write};
use std::path::{Component, Path};
use std::process::{Command as Process, Stdio};
use std::time::{Duration, Instant};

use crate::runtime::{Command, RunOutput};

/// Prazo quando o plugin pede 0.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_TIMEOUT: Duration = Duration::from_secs(60);
/// Saída e erro, cada um.
pub const MAX_OUTPUT: usize = 1024 * 1024;
/// Processos de um plugin ao mesmo tempo.
pub const MAX_RUNNING: usize = 4;

/// Resultado de um pedido que nem começou.
pub fn refused(id: u32, error: String) -> RunOutput {
    RunOutput { id, status: None, stdout: String::new(), stderr: String::new(), error: Some(error) }
}

/// `src/x` sim; `/x`, `../x` e `C:x`, não.
fn inside(path: &str) -> bool {
    path.as_bytes().get(1) != Some(&b':')
        && Path::new(path).components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

/// Roda e espera (chame numa thread). `root`: a raiz do projeto.
pub fn run(root: &Path, id: u32, command: &Command) -> RunOutput {
    let cwd = match &command.cwd {
        Some(dir) if !inside(dir) => return refused(id, fl!("plugin-run-cwd", cwd = dir.clone())),
        Some(dir) => root.join(dir),
        None => root.to_path_buf(),
    };
    let timeout = match command.timeout_ms {
        0 => DEFAULT_TIMEOUT,
        ms => Duration::from_millis(ms.into()).min(MAX_TIMEOUT),
    };
    let mut process = Process::new(&command.program);
    tea_core::os::hide_console(&mut process)
        .args(&command.args)
        .current_dir(&cwd)
        .stdin(if command.stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut process, 0);
    let mut child = match process.spawn() {
        Ok(c) => c,
        Err(e) => {
            return refused(id, fl!("plugin-run-start", program = command.program.clone(), error = e.to_string()));
        }
    };
    // Entrada, saída e erro em threads: um pipe cheio não trava o outro.
    if let (Some(mut pipe), Some(text)) = (child.stdin.take(), command.stdin.clone()) {
        std::thread::spawn(move || {
            let _ = pipe.write_all(text.as_bytes());
        });
    }
    let stdout = child.stdout.take().map(|p| std::thread::spawn(move || read_capped(p)));
    let stderr = child.stderr.take().map(|p| std::thread::spawn(move || read_capped(p)));

    let start = Instant::now();
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) if start.elapsed() >= timeout => {
                kill(&mut child);
                let _ = child.wait();
                break (None, true);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => break (None, false),
        }
    };
    let join = |h: Option<std::thread::JoinHandle<String>>| {
        if timed_out && cfg!(windows) {
            String::new()
        } else {
            h.and_then(|h| h.join().ok()).unwrap_or_default()
        }
    };
    RunOutput {
        id,
        status,
        stdout: join(stdout),
        stderr: join(stderr),
        error: timed_out.then(|| fl!("plugin-run-timeout", ms = (timeout.as_millis() as u64))),
    }
}

/// Mata o grupo inteiro (o programa e o que ele começou).
fn kill(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        // SAFETY: `kill` só manda um sinal; o grupo é o do filho (process_group(0) acima).
        unsafe { libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL) };
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

/// Lê até [`MAX_OUTPUT`] e descarta o resto (sem parar de ler: o programa não fica preso no pipe cheio).
fn read_capped(mut pipe: impl Read) -> String {
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    while let Ok(n) = pipe.read(&mut buf) {
        if n == 0 {
            break;
        }
        let room = MAX_OUTPUT.saturating_sub(out.len());
        out.extend_from_slice(&buf[..n.min(room)]);
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(program: &str, args: &[&str]) -> Command {
        Command {
            program: program.into(),
            args: args.iter().map(|a| a.to_string()).collect(),
            cwd: None,
            stdin: None,
            timeout_ms: 0,
        }
    }

    #[test]
    fn runs_with_stdin_cwd_and_status() {
        let root = std::env::temp_dir().join(format!("idetea-run-{}", std::process::id()));
        std::fs::create_dir_all(root.join("sub")).unwrap();
        let out = run(&root, 7, &cmd("sh", &["-c", "pwd; echo erro >&2; exit 3"]));
        assert_eq!((out.id, out.status, out.error), (7, Some(3), None));
        assert!(out.stdout.trim().ends_with(root.file_name().unwrap().to_str().unwrap()), "{}", out.stdout);
        assert_eq!(out.stderr, "erro\n");

        let mut c = cmd("sh", &["-c", "pwd; cat"]);
        c.cwd = Some("sub".into());
        c.stdin = Some("entrada".into());
        let out = run(&root, 1, &c);
        assert!(out.stdout.contains("/sub\n") && out.stdout.ends_with("entrada"), "{}", out.stdout);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn refuses_and_times_out() {
        let root = std::env::temp_dir();
        let mut c = cmd("ls", &[]);
        for bad in ["../", "/etc", "a/../.."] {
            c.cwd = Some(bad.into());
            assert!(run(&root, 1, &c).error.is_some(), "{bad}");
        }
        let out = run(&root, 1, &cmd("programa-que-nao-existe-idetea", &[]));
        assert!(out.error.is_some() && out.status.is_none());

        // Prazo: o `sh` e o `sleep` que ele começou morrem juntos.
        let mut c = cmd("sh", &["-c", "sleep 30; echo nao"]);
        c.timeout_ms = 200;
        let t = Instant::now();
        let out = run(&root, 1, &c);
        assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
        assert!(out.error.is_some() && out.status.is_none() && out.stdout.is_empty());
    }

    #[test]
    fn output_is_capped() {
        let out = run(&std::env::temp_dir(), 1, &cmd("sh", &["-c", "head -c 3000000 /dev/zero | tr '\\0' x"]));
        assert_eq!((out.status, out.stdout.len()), (Some(0), MAX_OUTPUT));
    }
}
