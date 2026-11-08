//! O que a IDE aberta está mostrando, para os agentes (`idetea status`), e os pedidos de `idetea open` (RFC 0006).
//!
//! A IDE grava `.idetea/state.json` quando as abas ou o cursor mudam (pelo temporário, no máximo a cada 200 ms) e
//! apaga ao sair. `idetea open` grava `.idetea/requests/open-<n>.json`; a IDE lê, apaga e abre a aba no lugar.
//! Tudo fica fora do git (`.idetea/.gitignore` e `.idetea/requests/.gitignore`).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;


pub const FORMAT: &str = "idetea-state";
pub const OPEN_FORMAT: &str = "idetea-open";
/// Intervalo mínimo entre duas gravações do `state.json`.
pub const MIN_GAP: Duration = Duration::from_millis(200);
/// Com que frequência a IDE olha a pasta dos pedidos de `open`.
const OPEN_POLL: Duration = Duration::from_millis(300);
/// Quanto o `idetea open` espera a IDE pegar o pedido.
const OPEN_WAIT: Duration = Duration::from_secs(3);

const EXIT_OK: i32 = 0;
const EXIT_INVALID: i32 = 2;
const EXIT_NOT_OPEN: i32 = 3;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub format: String,
    pub version: u32,
    /// Processo da IDE (o `idetea status` confere se ainda está vivo).
    pub pid: u32,
    /// Arquivo da aba ativa (relativo à raiz quando está dentro dela).
    pub active: Option<String>,
    /// Cursor da aba ativa, começando em 1.
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub files: Vec<FileState>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileState {
    pub path: String,
    /// Tem mudança não salva.
    pub modified: bool,
    pub line: usize,
    pub column: usize,
}

pub fn state_path(root: &Path) -> PathBuf {
    root.join(".idetea").join("state.json")
}

pub fn requests_dir(root: &Path) -> PathBuf {
    root.join(".idetea").join("requests")
}

/// Caminho para mostrar: relativo à raiz quando está dentro dela.
pub fn shown_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned()
}

/// Põe `name` no `.gitignore` de `.idetea/` (cria se faltar; não mexe no resto do que já existe).
fn ignore_in_idetea(root: &Path, name: &str) {
    let path = root.join(".idetea").join(".gitignore");
    // Criado aqui: ignora a si mesmo também (não aparece no `git status` do projeto).
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| "/.gitignore\n".into());
    if text.lines().any(|l| l.trim() == name || l.trim() == format!("/{name}")) {
        return;
    }
    let sep = if text.is_empty() || text.ends_with('\n') { "" } else { "\n" };
    let _ = std::fs::write(&path, format!("{text}{sep}/{name}\n"));
}

/// Grava o estado (só se o projeto já tem `.idetea/`: a IDE não cria a pasta só para isso).
pub fn write(root: &Path, state: &State) {
    let dir = root.join(".idetea");
    if !dir.is_dir() {
        return;
    }
    ignore_in_idetea(root, "state.json");
    let Ok(text) = serde_json::to_string_pretty(state) else { return };
    let path = state_path(root);
    let tmp = dir.join(".state.json.tmp");
    if std::fs::write(&tmp, text + "\n").is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

/// Ao sair: apaga o estado se for desta IDE (outra janela no mesmo projeto pode ter gravado por cima).
pub fn remove(root: &Path, pid: u32) {
    let path = state_path(root);
    if read(root).is_some_and(|s| s.pid == pid) {
        let _ = std::fs::remove_file(path);
    }
}

pub fn read(root: &Path) -> Option<State> {
    let text = std::fs::read_to_string(state_path(root)).ok()?;
    serde_json::from_str::<State>(&text).ok().filter(|s| s.format == FORMAT)
}

/// O processo ainda existe.
fn alive(pid: u32) -> bool {
    tea_core::os::process_alive(pid)
}

/// O estado de uma IDE aberta agora (um arquivo deixado por uma IDE que travou não conta).
fn live(root: &Path) -> Option<State> {
    read(root).filter(|s| alive(s.pid))
}

/// Um pedido de `open`, como a IDE recebe.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenRequest {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}

/// Lê um pedido de `open`; o caminho precisa ficar dentro da raiz.
fn parse_open(root: &Path, text: &str) -> Result<OpenRequest, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if v["format"] != OPEN_FORMAT {
        return Err(format!("format: {}", v["format"]));
    }
    let path = v["path"].as_str().filter(|p| !p.is_empty()).ok_or("path")?;
    let path = inside(root, &root.join(path)).ok_or_else(|| path.to_string())?;
    let num = |k: &str| v[k].as_u64().and_then(|n| usize::try_from(n).ok()).filter(|n| *n > 0).unwrap_or(1);
    Ok(OpenRequest { path, line: num("line"), column: num("column") })
}

/// O caminho, se fica dentro da raiz (resolvendo `..` e links do que já existe).
fn inside(root: &Path, path: &Path) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let full = path.canonicalize().ok().or_else(|| {
        // Arquivo novo: confere a pasta.
        let parent = path.parent()?.canonicalize().ok()?;
        Some(parent.join(path.file_name()?))
    })?;
    full.starts_with(&root).then_some(full)
}

/// Pega (lê e apaga) os pedidos de `open` que estão na pasta, na ordem dos nomes. Os inválidos são apagados também.
pub fn take_open_requests(root: &Path) -> Vec<OpenRequest> {
    let dir = requests_dir(root);
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            name.starts_with("open-") && name.ends_with(".json")
        })
        .collect();
    files.sort();
    let mut out = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let _ = std::fs::remove_file(&file);
        match parse_open(root, &text) {
            Ok(request) => out.push(request),
            Err(e) => eprintln!("idetea: pedido de open ignorado ({}): {e}", file.display()),
        }
    }
    out
}

/// Olha a pasta dos pedidos de `open` e manda cada um. Termina quando a subscription some.
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
pub fn watch_open(root: &PathBuf) -> iced::futures::channel::mpsc::UnboundedReceiver<OpenRequest> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    let root = root.clone();
    std::thread::spawn(move || {
        loop {
            if tx.is_closed() {
                return;
            }
            for request in take_open_requests(&root) {
                if tx.unbounded_send(request).is_err() {
                    return;
                }
            }
            std::thread::sleep(OPEN_POLL);
        }
    });
    rx
}

// ---- Linha de comando -----------------------------------------------------

/// `idetea status [--json]`: o que a IDE aberta neste projeto mostra. Sem IDE aberta: código 3.
pub fn run_status(args: &[String], cwd: &Path) -> i32 {
    let mut as_json = false;
    for a in args {
        match a.as_str() {
            "--json" => as_json = true,
            "-h" | "--help" => {
                println!("{}", fl!("status-usage"));
                return EXIT_OK;
            }
            other => {
                eprintln!("{}", fl!("status-bad-arg", arg = other.to_string()));
                eprintln!("{}", fl!("status-usage"));
                return EXIT_INVALID;
            }
        }
    }
    let root = project_root(cwd);
    let Some(state) = live(&root) else {
        if as_json {
            println!("{}", json!({"open": false}));
        } else {
            eprintln!("{}", fl!("status-not-open", root = root.display().to_string()));
        }
        return EXIT_NOT_OPEN;
    };
    if as_json {
        let mut v = serde_json::to_value(&state).unwrap_or_default();
        v["open"] = true.into();
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
        return EXIT_OK;
    }
    match (&state.active, state.line, state.column) {
        (Some(path), Some(line), Some(column)) => println!("{}", fl!("status-active", path = format!("{path}:{line}:{column}"))),
        _ => println!("{}", fl!("status-no-active")),
    }
    for f in &state.files {
        let mark = if f.modified { fl!("status-modified") } else { String::new() };
        println!("  {}:{}:{}{mark}", f.path, f.line, f.column);
    }
    EXIT_OK
}

/// `arquivo[:linha[:coluna]]` → caminho, linha e coluna (começando em 1).
fn split_location(arg: &str) -> (String, usize, usize) {
    let mut parts: Vec<&str> = arg.rsplitn(3, ':').collect();
    parts.reverse();
    let num = |s: &str| s.parse::<usize>().ok().filter(|n| *n > 0);
    match parts.as_slice() {
        [path, line, column] if num(line).is_some() && num(column).is_some() => {
            (path.to_string(), num(line).unwrap_or(1), num(column).unwrap_or(1))
        }
        [rest @ .., last] if num(last).is_some() && !rest.is_empty() => (rest.join(":"), num(last).unwrap_or(1), 1),
        _ => (arg.to_string(), 1, 1),
    }
}

/// `idetea open arquivo[:linha[:coluna]]`: a IDE aberta mostra o arquivo com o cursor no lugar.
pub fn run_open(args: &[String], cwd: &Path) -> i32 {
    let location = match args {
        [one] if !one.starts_with('-') => one,
        [h] if h == "-h" || h == "--help" => {
            println!("{}", fl!("open-usage"));
            return EXIT_OK;
        }
        _ => {
            eprintln!("{}", fl!("open-usage"));
            return EXIT_INVALID;
        }
    };
    let (file, line, column) = split_location(location);
    let root = project_root(cwd);
    let Some(path) = inside(&root, &cwd.join(&file)) else {
        eprintln!("{}", fl!("open-outside", path = file, root = root.display().to_string()));
        return EXIT_INVALID;
    };
    if live(&root).is_none() {
        eprintln!("{}", fl!("status-not-open", root = root.display().to_string()));
        return EXIT_NOT_OPEN;
    }
    let canonical_root = root.canonicalize().unwrap_or(root.clone());
    let relative = shown_path(&canonical_root, &path);
    let dir = requests_dir(&root);
    if std::fs::create_dir_all(&dir).is_err() {
        eprintln!("{}", fl!("open-write-error", path = dir.display().to_string()));
        return EXIT_INVALID;
    }
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        let _ = std::fs::write(&ignore, "*\n");
    }
    let millis = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let name = format!("open-{millis:015}-{}.json", std::process::id());
    let doc = json!({"format": OPEN_FORMAT, "version": 1, "path": relative, "line": line, "column": column});
    let tmp = dir.join(format!(".{name}.tmp"));
    let target = dir.join(&name);
    if std::fs::write(&tmp, doc.to_string()).and_then(|()| std::fs::rename(&tmp, &target)).is_err() {
        eprintln!("{}", fl!("open-write-error", path = target.display().to_string()));
        return EXIT_INVALID;
    }
    let deadline = Instant::now() + OPEN_WAIT;
    while target.exists() {
        if Instant::now() >= deadline {
            let _ = std::fs::remove_file(&target);
            eprintln!("{}", fl!("open-not-taken"));
            return EXIT_NOT_OPEN;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    println!("{relative}:{line}:{column}");
    EXIT_OK
}

/// A raiz do projeto: a pasta (desta ou de cima) com `.idetea/`, senão a de cima com `.git`, senão esta.
fn project_root(cwd: &Path) -> PathBuf {
    cwd.ancestors()
        .find(|d| d.join(".idetea").is_dir())
        .or_else(|| cwd.ancestors().find(|d| d.join(".git").exists()))
        .unwrap_or(cwd)
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("idetea-state-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".idetea")).unwrap();
        root.canonicalize().unwrap()
    }

    #[test]
    fn locations() {
        assert_eq!(split_location("src/main.rs"), ("src/main.rs".into(), 1, 1));
        assert_eq!(split_location("src/main.rs:42"), ("src/main.rs".into(), 42, 1));
        assert_eq!(split_location("src/main.rs:42:7"), ("src/main.rs".into(), 42, 7));
        assert_eq!(split_location("a:b.rs:3"), ("a:b.rs".into(), 3, 1));
        assert_eq!(split_location("x.rs:0"), ("x.rs:0".into(), 1, 1));
    }

    /// Grava, lê e apaga só o estado desta IDE; o `.gitignore` de `.idetea/` ganha a linha uma vez.
    #[test]
    fn state_round_trip() {
        let root = temp("round");
        std::fs::write(root.join(".idetea/.gitignore"), "worktrees").unwrap();
        let state = State {
            format: FORMAT.into(),
            version: 1,
            pid: std::process::id(),
            active: Some("src/a.rs".into()),
            line: Some(3),
            column: Some(1),
            files: vec![FileState { path: "src/a.rs".into(), modified: true, line: 3, column: 1 }],
        };
        write(&root, &state);
        write(&root, &state);
        assert_eq!(read(&root), Some(state.clone()));
        assert_eq!(live(&root), Some(state.clone()));
        assert_eq!(std::fs::read_to_string(root.join(".idetea/.gitignore")).unwrap(), "worktrees\n/state.json\n");
        remove(&root, state.pid + 1);
        assert!(state_path(&root).exists(), "de outra IDE: fica");
        remove(&root, state.pid);
        assert!(!state_path(&root).exists());
        // Sem `.idetea/`, não cria nada.
        let bare = std::env::temp_dir().join(format!("idetea-state-bare-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&bare);
        std::fs::create_dir_all(&bare).unwrap();
        write(&bare, &state);
        assert!(!bare.join(".idetea").exists());
        // `.gitignore` novo: ignora a si mesmo e o estado.
        let fresh = temp("fresh");
        write(&fresh, &state);
        assert_eq!(std::fs::read_to_string(fresh.join(".idetea/.gitignore")).unwrap(), "/.gitignore\n/state.json\n");
    }

    /// A IDE pega os pedidos (e apaga), na ordem; fora da raiz ou estragado é apagado sem abrir.
    #[test]
    fn open_requests() {
        let root = temp("open");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/a.rs"), "fn main() {}\n").unwrap();
        let dir = requests_dir(&root);
        std::fs::create_dir_all(&dir).unwrap();
        let put = |name: &str, v: serde_json::Value| std::fs::write(dir.join(name), v.to_string()).unwrap();
        put("open-2.json", json!({"format": OPEN_FORMAT, "version": 1, "path": "src/a.rs", "line": 4, "column": 2}));
        put("open-1.json", json!({"format": OPEN_FORMAT, "version": 1, "path": "src/a.rs"}));
        put("open-3.json", json!({"format": OPEN_FORMAT, "version": 1, "path": "../fora.rs"}));
        put("open-4.json", json!({"format": "outro", "path": "src/a.rs"}));
        std::fs::write(dir.join("open-5.json"), "{").unwrap();
        let got = take_open_requests(&root);
        let a = root.join("src/a.rs");
        assert_eq!(
            got,
            vec![
                OpenRequest { path: a.clone(), line: 1, column: 1 },
                OpenRequest { path: a, line: 4, column: 2 },
            ]
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    }

    /// `idetea status` e `idetea open` sem IDE aberta: código 3; com o estado de um processo vivo, `open` grava o
    /// pedido e espera alguém pegar.
    #[test]
    fn commands_need_an_open_ide() {
        let root = temp("cmd");
        std::fs::write(root.join("a.txt"), "oi\n").unwrap();
        assert_eq!(run_status(&[], &root), EXIT_NOT_OPEN);
        assert_eq!(run_open(&["a.txt:2".into()], &root), EXIT_NOT_OPEN);
        assert_eq!(run_open(&["../x".into()], &root), EXIT_INVALID);
        // Um estado de processo morto não conta.
        write(&root, &State { format: FORMAT.into(), version: 1, pid: u32::MAX - 1, ..State::default() });
        assert_eq!(run_status(&[], &root), EXIT_NOT_OPEN);
        write(&root, &State { format: FORMAT.into(), version: 1, pid: std::process::id(), ..State::default() });
        assert_eq!(run_status(&["--json".into()], &root), EXIT_OK);
        let ide = {
            let root = root.clone();
            std::thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    let got = take_open_requests(&root);
                    if !got.is_empty() || Instant::now() > deadline {
                        return got;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            })
        };
        assert_eq!(run_open(&["a.txt:2:3".into()], &root), EXIT_OK);
        assert_eq!(ide.join().unwrap(), vec![OpenRequest { path: root.join("a.txt"), line: 2, column: 3 }]);
        assert_eq!(std::fs::read_to_string(requests_dir(&root).join(".gitignore")).unwrap(), "*\n");
    }
}
