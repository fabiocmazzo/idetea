//! `idetea run` (RFC 0006, etapa 1): um pedido de terminal numa chamada só.
//!
//! Escreve `.idetea/agents/<nome>.json` (pelo temporário), espera o status chegar a um fim, imprime o estado, o
//! código e a resposta (`IDETEA_ANSWER`) e limpa: o pedido primeiro, depois o status e a resposta (apagar o status
//! com o pedido ainda lá faria a IDE perguntar de novo). A confirmação do usuário continua igual.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::agents;

const POLL: Duration = Duration::from_millis(250);

/// Saídas próprias (fora delas, o código do comando).
pub const EXIT_INVALID: i32 = 2;
pub const EXIT_TIMEOUT: i32 = 4;
pub const EXIT_REFUSED: i32 = 5;
pub const EXIT_CLOSED: i32 = 6;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Args {
    command: Option<String>,
    name: Option<String>,
    title: Option<String>,
    source: Option<String>,
    cwd: Option<String>,
    worktree: Option<String>,
    keep: bool,
    no_wait: bool,
    timeout: Option<Duration>,
    /// `--wait a b c`: os nomes, na ordem (RFC 0029).
    wait: Vec<String>,
    stop: Option<String>,
}

/// Um argumento para o shell, entre aspas simples quando precisa.
fn quote(arg: &str) -> String {
    if !arg.is_empty() && arg.chars().all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c)) {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut next = |flag: &str| it.next().cloned().ok_or_else(|| flag.to_string());
        match arg.as_str() {
            "--" => {
                let rest: Vec<&String> = it.by_ref().collect();
                // Um argumento só é a linha de shell inteira; vários são palavras, cada uma entre aspas.
                a.command = match rest.as_slice() {
                    [] => return Err("--".into()),
                    [one] => Some((*one).clone()),
                    many => Some(many.iter().map(|w| quote(w)).collect::<Vec<_>>().join(" ")),
                };
            }
            "--name" => a.name = Some(next(arg)?),
            "--title" => a.title = Some(next(arg)?),
            "--source" => a.source = Some(next(arg)?),
            "--cwd" => a.cwd = Some(next(arg)?),
            "--worktree" => a.worktree = Some(next(arg)?),
            "--keep" => a.keep = true,
            "--no-wait" => a.no_wait = true,
            "--timeout" => a.timeout = Some(Duration::from_secs(next(arg)?.parse().map_err(|_| arg.clone())?)),
            "--wait" => {
                // Os nomes até a próxima opção: `--wait a b c` espera um lote (também `--wait a --wait b`).
                a.wait.push(next(arg)?);
                while let Some(more) = it.as_slice().first().filter(|n| !n.starts_with("--")) {
                    a.wait.push(more.clone());
                    it.next();
                }
            }
            "--stop" => a.stop = Some(next(arg)?),
            other => return Err(other.to_string()),
        }
    }
    let modes = [a.command.is_some(), !a.wait.is_empty(), a.stop.is_some()].iter().filter(|m| **m).count();
    if modes != 1 {
        return Err(String::new());
    }
    for name in [&a.name, &a.stop].into_iter().flatten().chain(&a.wait) {
        if !agents::valid_name(name) {
            return Err(name.clone());
        }
    }
    Ok(a)
}

/// A raiz do projeto: a pasta (desta ou de cima) que já tem `.idetea/`, senão a de cima com `.git`, senão esta.
fn project_root(cwd: &Path) -> PathBuf {
    cwd.ancestors()
        .find(|d| d.join(".idetea").is_dir())
        .or_else(|| cwd.ancestors().find(|d| d.join(".git").exists()))
        .unwrap_or(cwd)
        .to_path_buf()
}

fn fresh_name() -> String {
    let mut n = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let mut s = String::new();
    while n > 0 {
        s.insert(0, char::from_digit((n % 36) as u32, 36).unwrap_or('0'));
        n /= 36;
    }
    format!("run-{s}")
}

fn request_path(root: &Path, name: &str) -> PathBuf {
    agents::dir(root).join(format!("{name}.json"))
}

/// Escreve o pedido pelo temporário. Um pedido com o mesmo nome ainda na pasta é recusado (não troca o de
/// outro agente no meio).
fn write_request(root: &Path, name: &str, a: &Args, command: &str, ide: Option<u32>) -> Result<(), String> {
    let path = request_path(root, name);
    if path.exists() || agents::status_path(root, name).exists() {
        return Err(fl!("run-busy", name = name.to_string()));
    }
    let dir = agents::dir(root);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut req = json!({"format": agents::FORMAT, "version": 1, "title": a.title.clone().unwrap_or_else(|| name.to_string()),
                         "source": a.source.clone().unwrap_or_else(|| "agent".into()), "command": command});
    if let Some(c) = &a.cwd {
        req["cwd"] = c.clone().into();
    }
    if let Some(w) = &a.worktree {
        req["worktree"] = w.clone().into();
    }
    // Saiu de uma aba da IDE que tem este projeto aberto: o pedido é dela (RFC 0024, 2b).
    if let Some(pid) = ide {
        req["ide"] = pid.into();
    }
    let tmp = dir.join(format!(".{name}.json.tmp"));
    std::fs::write(&tmp, serde_json::to_string_pretty(&req).unwrap_or_default()).map_err(|e| e.to_string())?;
    std::fs::rename(tmp, path).map_err(|e| e.to_string())
}

fn read_status(root: &Path, name: &str) -> Option<Value> {
    std::fs::read_to_string(agents::status_path(root, name)).ok().and_then(|t| serde_json::from_str(&t).ok())
}

/// Espera um estado final (`exited`, `refused`, `closed`, `invalid`, `lost`). `None`: prazo esgotado.
fn wait(root: &Path, name: &str, timeout: Option<Duration>) -> Option<Value> {
    let end = timeout.map(|t| Instant::now() + t);
    loop {
        if let Some(v) = read_status(root, name)
            && matches!(v["state"].as_str(), Some("exited" | "refused" | "closed" | "invalid" | "lost"))
        {
            return Some(v);
        }
        if end.is_some_and(|e| Instant::now() >= e) {
            return None;
        }
        std::thread::sleep(POLL);
    }
}

/// Tira o pedido e, um instante depois (a IDE vê o pedido sumir e encerra o terminal), o status e a resposta.
fn clean(root: &Path, name: &str) {
    let _ = std::fs::remove_file(request_path(root, name));
    std::thread::sleep(Duration::from_millis(1500));
    let _ = std::fs::remove_file(agents::status_path(root, name));
    let _ = std::fs::remove_file(agents::answer_path(root, name));
}

/// Mostra o fim e devolve o código de saída.
fn finish(root: &Path, name: &str, status: &Value, keep: bool) -> i32 {
    let state = status["state"].as_str().unwrap_or_default();
    let answer = std::fs::read_to_string(agents::answer_path(root, name)).ok();
    let code = match state {
        "exited" => {
            let code = status["code"].as_i64().unwrap_or(1);
            eprintln!("{}", fl!("run-exited", name = name.to_string(), code = code.to_string()));
            i32::try_from(code).unwrap_or(1)
        }
        "refused" => {
            eprintln!("{}", fl!("run-refused", name = name.to_string()));
            EXIT_REFUSED
        }
        "invalid" => {
            eprintln!("{}", fl!("run-invalid", name = name.to_string(), error = status["error"].as_str().unwrap_or_default().to_string()));
            EXIT_INVALID
        }
        other => {
            eprintln!("{}", fl!("run-ended", name = name.to_string(), state = other.to_string()));
            EXIT_CLOSED
        }
    };
    if let Some(branch) = status["branch"].as_str() {
        eprintln!("{}", fl!("run-branch", branch = branch.to_string()));
    }
    if let Some(answer) = answer {
        print!("{answer}");
    }
    if !keep {
        clean(root, name);
    }
    code
}

pub fn run(args: &[String], cwd: &Path) -> i32 {
    let a = match parse(args) {
        Ok(a) => a,
        Err(arg) => {
            if !arg.is_empty() {
                eprintln!("{}", fl!("run-bad-arg", arg = arg));
            }
            eprintln!("{}", fl!("run-usage"));
            return EXIT_INVALID;
        }
    };
    let root = project_root(cwd);
    if let Some(name) = &a.stop {
        clean(&root, name);
        return 0;
    }
    let names = match &a.command {
        None => a.wait.clone(),
        Some(command) => {
            let name = a.name.clone().unwrap_or_else(fresh_name);
            if let Err(e) = write_request(&root, &name, &a, command, crate::tab_hooks::origin_ide(&root)) {
                eprintln!("{e}");
                return EXIT_INVALID;
            }
            eprintln!("{}", fl!("run-asked", name = name.clone()));
            if a.no_wait {
                println!("{name}");
                return 0;
            }
            vec![name]
        }
    };
    wait_all(&root, &names, a.timeout, a.keep)
}

/// Espera cada nome na ordem, com um prazo só para todos, e mostra o fim de cada um (o código no stderr). Com vários
/// nomes, a resposta de cada um sai no stdout depois de `==> nome <==`. Sai com o primeiro código diferente de 0
/// (o prazo vencido conta como 4), ou 0 se todos deram 0.
fn wait_all(root: &Path, names: &[String], timeout: Option<Duration>, keep: bool) -> i32 {
    let end = timeout.map(|t| Instant::now() + t);
    let mut worst = 0;
    for name in names {
        let left = end.map(|e| e.saturating_duration_since(Instant::now()));
        if names.len() > 1 {
            println!("==> {name} <==");
        }
        let code = match wait(root, name, left) {
            Some(status) => finish(root, name, &status, keep),
            None => {
                eprintln!("{}", fl!("run-timeout", name = name.clone()));
                EXIT_TIMEOUT
            }
        };
        if worst == 0 {
            worst = code;
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn reads_the_arguments_and_quotes_words() {
        let a = parse(&s(&["--title", "Servidor", "--", "npm run dev"])).unwrap();
        assert_eq!(a.command.as_deref(), Some("npm run dev"), "um argumento: a linha de shell inteira");
        let a = parse(&s(&["--", "echo", "it's", "a b", "$HOME"])).unwrap();
        assert_eq!(a.command.as_deref(), Some(r"echo 'it'\''s' 'a b' '$HOME'"), "vários: cada palavra entre aspas");
        assert!(parse(&s(&["--title", "x"])).is_err(), "sem comando");
        assert!(parse(&s(&["--", "x", "--wait", "y"])).is_ok(), "depois de -- é comando");
        assert!(parse(&s(&["--wait", "A B"])).is_err());
        assert!(parse(&s(&["--stop", "x"])).is_ok());
        let a = parse(&s(&["--wait", "a", "b", "--timeout", "5", "--wait", "c"])).unwrap();
        assert_eq!(a.wait, ["a", "b", "c"], "um lote: os nomes até a próxima opção, e --wait de novo");
        assert!(parse(&s(&["--wait", "a", "B C"])).is_err(), "todo nome do lote é conferido");
        assert!(parse(&s(&["--wat"])).is_err());
    }

    #[test]
    fn asks_waits_reports_and_cleans_up() {
        let root = std::env::temp_dir().join(format!("idetea-run-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".idetea")).unwrap();
        let a = parse(&s(&["--title", "T", "--worktree", "agente/x", "--cwd", "src", "--", "true"])).unwrap();
        write_request(&root, "um", &a, "true", Some(4242)).unwrap();
        let req: Value = serde_json::from_str(&std::fs::read_to_string(request_path(&root, "um")).unwrap()).unwrap();
        assert_eq!((req["format"].as_str(), req["worktree"].as_str(), req["cwd"].as_str()), (Some(agents::FORMAT), Some("agente/x"), Some("src")));
        assert_eq!(req["ide"].as_u64(), Some(4242));
        assert!(write_request(&root, "um", &a, "true", None).is_err(), "não troca um pedido que já está lá");

        // Ainda `running`: o prazo vence.
        agents::write_status(&root, "um", &agents::State::Running).unwrap();
        assert!(wait(&root, "um", Some(Duration::from_millis(300))).is_none());
        // A IDE (aqui, o teste) grava o fim e a resposta.
        agents::write_status(&root, "um", &agents::State::Exited(3)).unwrap();
        std::fs::write(agents::answer_path(&root, "um"), "resposta\n").unwrap();
        let status = wait(&root, "um", Some(Duration::from_secs(2))).unwrap();
        assert_eq!(finish(&root, "um", &status, false), 3, "o código do comando");
        for p in [request_path(&root, "um"), agents::status_path(&root, "um"), agents::answer_path(&root, "um")] {
            assert!(!p.exists(), "{}", p.display());
        }

        write_request(&root, "dois", &a, "true", None).unwrap();
        agents::write_status(&root, "dois", &agents::State::Refused).unwrap();
        let status = wait(&root, "dois", None).unwrap();
        assert_eq!(finish(&root, "dois", &status, true), EXIT_REFUSED);
        assert!(request_path(&root, "dois").exists(), "--keep deixa tudo");

        // Um lote: o primeiro código diferente de 0, depois de esperar todos.
        for (name, state) in [("l1", agents::State::Exited(0)), ("l2", agents::State::Exited(7)), ("l3", agents::State::Refused)] {
            write_request(&root, name, &a, "true", None).unwrap();
            agents::write_status(&root, name, &state).unwrap();
        }
        assert_eq!(wait_all(&root, &s(&["l1", "l2", "l3"]), Some(Duration::from_secs(2)), true), 7);
        agents::write_status(&root, "l2", &agents::State::Exited(0)).unwrap();
        assert_eq!(wait_all(&root, &s(&["l1", "l2", "l3"]), Some(Duration::from_secs(2)), true), EXIT_REFUSED);
        assert_eq!(wait_all(&root, &s(&["l1", "nada"]), Some(Duration::from_millis(300)), true), EXIT_TIMEOUT);
        let _ = std::fs::remove_dir_all(&root);
    }
}
