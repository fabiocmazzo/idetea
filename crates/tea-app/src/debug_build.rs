//! Compilar antes de depurar e achar o executável.
//!
//! - `[debug] build = "..."` no config: roda esse comando (`sh -c`). `build = ""` não compila.
//! - Sem `build` e com `Cargo.toml` na raiz: `cargo build`, lendo a saída JSON do cargo para
//!   saber quais executáveis ele gerou (não precisa adivinhar `target/debug/<nome>`).
//! - Sem nada disso: não compila.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;
use tea_core::config::DebugConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Build {
    Cargo,
    Shell(String),
}

impl Build {
    pub fn label(&self) -> &str {
        match self {
            Build::Cargo => "cargo build",
            Build::Shell(cmd) => cmd,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub ok: bool,
    /// Executáveis que o cargo gerou.
    pub executables: Vec<PathBuf>,
    /// Saída da compilação (vai para a saída do debug).
    pub log: String,
    /// Resumo do erro, quando falha.
    pub error: Option<String>,
}

/// O que rodar antes de depurar, se algo.
pub fn plan(config: &DebugConfig, root: &Path) -> Option<Build> {
    match config.build.as_deref().map(str::trim) {
        Some("") => None,
        Some(cmd) => Some(Build::Shell(cmd.to_string())),
        None if root.join("Cargo.toml").is_file() => Some(Build::Cargo),
        None => None,
    }
}

/// Compila (bloqueia: chame numa thread).
pub fn run(build: &Build, root: &Path) -> Outcome {
    let mut cmd = match build {
        Build::Cargo => {
            let mut c = Command::new("cargo");
            c.args(["build", "--message-format=json-render-diagnostics"]);
            c
        }
        Build::Shell(line) => {
            let mut c = Command::new("sh");
            c.args(["-c", line]);
            c
        }
    };
    let out = cmd
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .stdin(Stdio::null())
        .output();
    let out = match out {
        Ok(o) => o,
        Err(e) => {
            let error = format!("não foi possível rodar \"{}\": {e}", build.label());
            return Outcome { error: Some(error), ..Outcome::default() };
        }
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let cargo = *build == Build::Cargo;
    // No cargo, o stdout é JSON (para nós) e o stderr é o texto para gente.
    let log = if cargo { stderr } else { format!("{stdout}{stderr}") };
    let ok = out.status.success();
    Outcome {
        ok,
        executables: if cargo { executables(&stdout) } else { Vec::new() },
        error: (!ok).then(|| first_error(&log)),
        log,
    }
}

/// Executáveis (`bin`) na saída `--message-format=json` do cargo.
pub fn executables(json_lines: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for line in json_lines.lines() {
        let Ok(msg) = serde_json::from_str::<Value>(line) else { continue };
        let is_bin = msg["target"]["kind"].as_array().is_some_and(|k| k.iter().any(|k| k == "bin"));
        if msg["reason"] == "compiler-artifact"
            && is_bin
            && let Some(exe) = msg["executable"].as_str()
        {
            let exe = PathBuf::from(exe);
            if !found.contains(&exe) {
                found.push(exe);
            }
        }
    }
    found
}

/// A primeira linha de erro, ou a última linha com texto.
fn first_error(log: &str) -> String {
    let line = log
        .lines()
        .find(|l| l.trim_start().starts_with("error"))
        .or_else(|| log.lines().rfind(|l| !l.trim().is_empty()))
        .unwrap_or("falhou sem mensagem");
    line.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(build: Option<&str>) -> DebugConfig {
        DebugConfig { build: build.map(str::to_string), ..DebugConfig::default() }
    }

    #[test]
    fn plan_rules() {
        let dir = std::env::temp_dir().join(format!("idetea-plan-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(plan(&cfg(None), &dir), None, "sem Cargo.toml e sem config: não compila");
        assert_eq!(plan(&cfg(Some("make")), &dir), Some(Build::Shell("make".into())));
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();
        assert_eq!(plan(&cfg(None), &dir), Some(Build::Cargo));
        assert_eq!(plan(&cfg(Some("  ")), &dir), None, "build vazio desliga até no cargo");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn executables_from_cargo_json() {
        let out = r#"{"reason":"compiler-artifact","target":{"kind":["lib"],"name":"x"},"executable":null}
{"reason":"compiler-artifact","target":{"kind":["custom-build"],"name":"build-script-build"},"executable":null}
{"reason":"compiler-artifact","target":{"kind":["bin"],"name":"app"},"executable":"/p/target/debug/app"}
{"reason":"compiler-artifact","target":{"kind":["bin"],"name":"cli"},"executable":"/p/target/debug/cli"}
{"reason":"build-finished","success":true}
não é json"#;
        assert_eq!(executables(out), [PathBuf::from("/p/target/debug/app"), PathBuf::from("/p/target/debug/cli")]);
    }

    #[test]
    fn error_summary() {
        let log = "   Compiling x v0.1.0\nwarning: unused\nerror[E0425]: cannot find value `y`\n --> src/main.rs:2:5\n";
        assert_eq!(first_error(log), "error[E0425]: cannot find value `y`");
        assert_eq!(first_error("make: *** [all] Error 2\n"), "make: *** [all] Error 2");
    }

    /// Compila um projeto cargo de verdade (pula sem cargo).
    #[test]
    fn real_cargo_build() {
        let dir = std::env::temp_dir().join(format!("idetea-cargo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
            .unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() { println!(\"oi\"); }\n").unwrap();
        if Command::new("cargo").arg("--version").output().is_err() {
            return;
        }
        let out = run(&Build::Cargo, &dir);
        assert!(out.ok, "{}", out.log);
        assert_eq!(out.executables.len(), 1);
        assert!(out.executables[0].is_file() && out.executables[0].ends_with("demo"));

        std::fs::write(dir.join("src/main.rs"), "fn main() { y }\n").unwrap();
        let out = run(&Build::Cargo, &dir);
        assert!(!out.ok);
        assert!(out.error.as_deref().is_some_and(|e| e.starts_with("error")), "{:?}", out.error);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
