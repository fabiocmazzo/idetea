//! Soltar do terminal: `idetea ./` devolve o prompt na hora, como `code .`.
//!
//! Aberta de um terminal, a IDE roda de novo a si mesma numa sessão própria (`setsid`) e sai. A cópia não recebe o
//! Ctrl+C nem o fim do terminal; o que ela escreveria no terminal vai para `~/.local/state/idetea/idetea.log`.
//! Não solta com `--foreground`, em build de debug (`cargo run` mostra os logs), nem fora de um terminal (menu de
//! aplicativos, outra janela aberta pela IDE).

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

/// O log passa disso: começa de novo (é para o último problema, não um histórico).
const LOG_MAX: u64 = 1024 * 1024;

/// Soltar ou não. `foreground`: o `--foreground` da linha de comando.
pub fn wanted(foreground: bool) -> bool {
    !foreground
        && !cfg!(debug_assertions)
        && cfg!(unix)
        && (std::io::stdout().is_terminal() || std::io::stderr().is_terminal())
}

/// `~/.local/state/idetea/idetea.log` (ou em `$XDG_STATE_HOME`).
pub fn log_path() -> Option<PathBuf> {
    crate::ui_state::state_path().map(|p| p.with_file_name("idetea.log"))
}

/// Roda a cópia solta com os mesmos argumentos. `Ok`: este processo deve sair; `Err`: segue no terminal.
pub fn spawn() -> std::io::Result<()> {
    // AppImage: o executável está numa montagem que some quando este processo sai; roda o `.AppImage` de novo.
    let exe = match std::env::var_os("APPIMAGE").map(PathBuf::from).filter(|p| p.is_file()) {
        Some(appimage) => appimage,
        None => std::env::current_exe()?,
    };
    spawn_exe(&exe)
}

/// Roda `exe` com os argumentos deste processo, numa sessão própria, com a saída no log.
#[cfg(unix)]
pub fn spawn_exe(exe: &Path) -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    let log = log_path().and_then(|p| open_log(&p).ok());
    let (out, err) = match &log {
        Some(f) => (Stdio::from(f.try_clone()?), Stdio::from(f.try_clone()?)),
        None => (Stdio::null(), Stdio::null()),
    };
    let mut cmd = Command::new(exe);
    cmd.args(std::env::args_os().skip(1)).stdin(Stdio::null()).stdout(out).stderr(err);
    // SAFETY: só `setsid` entre o fork e o exec (seguro depois de um fork).
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    // Sem esperar: a cópia é adotada pelo init quando este processo sai.
    cmd.spawn().map(drop)
}

#[cfg(not(unix))]
pub fn spawn_exe(exe: &Path) -> std::io::Result<()> {
    use std::process::{Command, Stdio};

    let log = log_path().and_then(|p| open_log(&p).ok());
    let (out, err) = match &log {
        Some(f) => (Stdio::from(f.try_clone()?), Stdio::from(f.try_clone()?)),
        None => (Stdio::null(), Stdio::null()),
    };
    Command::new(exe)
        .args(std::env::args_os().skip(1))
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .spawn()
        .map(drop)
}

/// Abre o log para acrescentar, começando de novo se passou de [`LOG_MAX`].
fn open_log(path: &Path) -> std::io::Result<std::fs::File> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let big = std::fs::metadata(path).is_ok_and(|m| m.len() > LOG_MAX);
    let mut file = std::fs::OpenOptions::new().create(true).append(!big).write(true).truncate(big).open(path)?;
    let when = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    writeln!(file, "--- idetea {} (unix {when})", env!("CARGO_PKG_VERSION"))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_never_detaches() {
        assert!(!wanted(true));
    }

    #[test]
    fn log_restarts_when_too_big() {
        let dir = std::env::temp_dir().join(format!("tea-detach-{}", std::process::id()));
        let path = dir.join("idetea.log");
        drop(open_log(&path).unwrap());
        drop(open_log(&path).unwrap());
        let two = std::fs::read_to_string(&path).unwrap();
        assert_eq!(two.lines().count(), 2, "acrescenta");
        std::fs::write(&path, vec![b'x'; LOG_MAX as usize + 1]).unwrap();
        drop(open_log(&path).unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 1, "começa de novo");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
