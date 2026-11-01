//! Tarefas: comandos do projeto que rodam no terminal (`Ctrl+Shift+B`).
//!
//! A lista junta as do config (`[tasks]`, primeiro) com as que dá para achar na raiz do projeto: cargo
//! (`Cargo.toml`), scripts do `package.json` e alvos do `Makefile`. Quem roda é a extensão do terminal
//! (`CoreEvent::RunTask`).
//!
//! "Rodar o teste no cursor" (Rust): acha o `fn` mais perto acima do cursor com um atributo de teste
//! (`#[test]`, `#[tokio::test]`...) e o pacote dono do arquivo (o `Cargo.toml` com `[package]` mais perto),
//! e roda `cargo test -p <pacote> [--test <arquivo>] <nome>`. O nome filtra por pedaço do caminho do teste.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDef {
    pub name: String,
    pub command: String,
    pub cwd: PathBuf,
}

impl TaskDef {
    fn new(name: impl Into<String>, command: impl Into<String>, cwd: &Path) -> Self {
        Self { name: name.into(), command: command.into(), cwd: cwd.to_path_buf() }
    }
}

/// Alvos do Makefile que aparecem na lista (os primeiros).
const MAX_MAKE_TARGETS: usize = 15;

/// Tarefas do projeto: as do config e as achadas na raiz.
pub fn list(root: &Path, configured: &BTreeMap<String, String>) -> Vec<TaskDef> {
    let mut out: Vec<TaskDef> = configured.iter().map(|(n, c)| TaskDef::new(n, c, root)).collect();
    if root.join("Cargo.toml").is_file() {
        for (name, cmd) in [
            ("cargo build", "cargo build"),
            ("cargo run", "cargo run"),
            ("cargo test", "cargo test"),
            ("cargo clippy", "cargo clippy --all-targets"),
        ] {
            out.push(TaskDef::new(name, cmd, root));
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("package.json")) {
        let runner = if root.join("pnpm-lock.yaml").exists() {
            "pnpm"
        } else if root.join("yarn.lock").exists() {
            "yarn"
        } else {
            "npm"
        };
        for script in package_scripts(&text) {
            let cmd = if runner == "npm" { format!("npm run {script}") } else { format!("{runner} {script}") };
            out.push(TaskDef::new(cmd.clone(), cmd, root));
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("Makefile")) {
        for target in make_targets(&text).into_iter().take(MAX_MAKE_TARGETS) {
            let cmd = format!("make {target}");
            out.push(TaskDef::new(cmd.clone(), cmd, root));
        }
    }
    // Uma tarefa do config com o mesmo nome de uma achada fica só a do config.
    let mut seen = std::collections::HashSet::new();
    out.retain(|t| seen.insert(t.name.clone()));
    out
}

fn package_scripts(text: &str) -> Vec<String> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else { return Vec::new() };
    json.get("scripts").and_then(|s| s.as_object()).map(|o| o.keys().cloned().collect()).unwrap_or_default()
}

/// Alvos do Makefile: `nome:` no começo da linha (sem `.PHONY`, padrões com `%` e atribuições `:=`).
fn make_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with(['\t', ' ', '#', '.']) {
            continue;
        }
        let Some((name, rest)) = line.split_once(':') else { continue };
        let name = name.trim();
        let ok = !name.is_empty()
            && !rest.starts_with('=')
            && name.chars().all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '/' | '.'));
        if ok && !out.iter().any(|t| t == name) {
            out.push(name.to_string());
        }
    }
    out
}

/// O teste Rust em volta da linha `line` (começa em 1): nome do `fn`. `None`: o cursor não está num teste.
pub fn rust_test_at(text: &str, line: usize) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let start = line.checked_sub(1)?.min(lines.len().checked_sub(1)?);
    for i in (0..=start).rev() {
        let Some(name) = fn_name(lines[i]) else { continue };
        // Atributos logo acima (pulando linhas vazias, comentários e outros atributos).
        let is_test = lines[..i]
            .iter()
            .rev()
            .map(|l| l.trim())
            .take_while(|l| l.is_empty() || l.starts_with("#[") || l.starts_with("//"))
            .any(|l| l.starts_with("#[") && is_test_attr(l));
        return is_test.then_some(name);
    }
    None
}

fn is_test_attr(attr: &str) -> bool {
    let inner = attr.trim_start_matches("#[").split(['(', ']']).next().unwrap_or("");
    inner == "test" || inner.ends_with("::test") || inner == "rstest" || inner == "test_case"
}

/// `fn nome(` numa linha (com `pub`, `async`... antes).
fn fn_name(line: &str) -> Option<String> {
    let code = line.split("//").next()?;
    let at = code.find("fn ").filter(|&i| i == 0 || !code.as_bytes()[i - 1].is_ascii_alphanumeric())?;
    let rest = code[at + 3..].trim_start();
    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    (!name.is_empty() && rest[name.len()..].trim_start().starts_with(['(', '<'])).then_some(name)
}

/// Comando para rodar o teste `name` do arquivo `file`, a partir da raiz do workspace `root`.
pub fn rust_test_task(root: &Path, file: &Path, name: &str) -> Option<TaskDef> {
    let (pkg_dir, pkg) = package_of(file)?;
    let rel = file.strip_prefix(&pkg_dir).ok()?;
    let mut cmd = format!("cargo test -p {pkg}");
    // Teste de integração (`tests/x.rs`): só esse alvo.
    if rel.components().count() == 2
        && rel.starts_with("tests")
        && let Some(stem) = rel.file_stem()
    {
        cmd.push_str(&format!(" --test {}", stem.to_string_lossy()));
    }
    cmd.push_str(&format!(" {name}"));
    Some(TaskDef::new(format!("{name} ({pkg})"), cmd, root))
}

/// Pasta e nome do pacote dono do arquivo: o `Cargo.toml` com `[package]` mais perto, subindo.
fn package_of(file: &Path) -> Option<(PathBuf, String)> {
    let mut dir = file.parent();
    while let Some(d) = dir {
        if let Ok(text) = std::fs::read_to_string(d.join("Cargo.toml"))
            && let Ok(table) = text.parse::<toml::Table>()
            && let Some(name) = table.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str())
        {
            return Some((d.to_path_buf(), name.to_string()));
        }
        dir = d.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_test_around_the_cursor() {
        let src = "fn helper() {}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    // comentário\n    fn soma_funciona() {\n        assert_eq!(1, 1);\n    }\n\n    #[tokio::test]\n    async fn rede() {}\n\n    fn nao_e_teste() {\n        x();\n    }\n}\n";
        assert_eq!(rust_test_at(src, 10).as_deref(), Some("soma_funciona"), "dentro do corpo");
        assert_eq!(rust_test_at(src, 9).as_deref(), Some("soma_funciona"), "na linha do fn");
        assert_eq!(rust_test_at(src, 14).as_deref(), Some("rede"));
        assert_eq!(rust_test_at(src, 17), None, "fn sem #[test]");
        assert_eq!(rust_test_at(src, 1), None);
        assert_eq!(rust_test_at(src, 5), None, "antes de qualquer teste, o fn de cima não é teste");
    }

    #[test]
    fn detected_tasks_and_test_command() {
        let dir = std::env::temp_dir().join(format!("idetea-tasks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates/app/tests")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/app\"]\n").unwrap();
        std::fs::write(dir.join("crates/app/Cargo.toml"), "[package]\nname = \"app\"\n").unwrap();
        std::fs::write(dir.join("package.json"), r#"{"scripts": {"dev": "vite", "build": "vite build"}}"#).unwrap();
        std::fs::write(dir.join("Makefile"), ".PHONY: all\nCC := gcc\nall: app\n\tcc x\n%.o: %.c\ninstall:\n").unwrap();
        let configured: BTreeMap<String, String> =
            [("cargo test".to_string(), "cargo nextest run".to_string()), ("servidor".to_string(), "cargo run".to_string())].into();
        let names: Vec<_> = list(&dir, &configured).into_iter().map(|t| (t.name, t.command)).collect();
        let n = |a: &str, b: &str| (a.to_string(), b.to_string());
        assert_eq!(
            names,
            [
                n("cargo test", "cargo nextest run"),
                n("servidor", "cargo run"),
                n("cargo build", "cargo build"),
                n("cargo run", "cargo run"),
                n("cargo clippy", "cargo clippy --all-targets"),
                n("npm run build", "npm run build"),
                n("npm run dev", "npm run dev"),
                n("make all", "make all"),
                n("make install", "make install"),
            ],
            "a do config vence a achada com o mesmo nome"
        );

        let unit = rust_test_task(&dir, &dir.join("crates/app/src/lib.rs"), "soma").unwrap();
        assert_eq!(unit.command, "cargo test -p app soma");
        assert_eq!(unit.cwd, dir);
        let integ = rust_test_task(&dir, &dir.join("crates/app/tests/api.rs"), "rota").unwrap();
        assert_eq!(integ.command, "cargo test -p app --test api rota");
        assert!(rust_test_task(&dir, Path::new("/solto.rs"), "x").is_none(), "fora de um pacote");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
