//! Compilar antes de depurar e achar o executável.
//!
//! - `[debug] build = "..."` no config: roda esse comando (`sh -c`). `build = ""` não compila.
//! - Sem `build` e com `Cargo.toml` na raiz: `cargo build`, lendo a saída JSON do cargo para
//!   saber quais executáveis ele gerou (não precisa adivinhar `target/debug/<nome>`).
//! - Sem `build` e com `build.zig` na raiz: `zig build`, e os executáveis são os de `zig-out/bin/`.
//! - Sem `build` e com `moon.mod` na raiz (MoonBit): `moon build --target native -g --output-json`; os executáveis
//!   vêm das mensagens `artifact` e os erros das `diagnostic`. O backend nativo gera DWARF apontando para os `.mbt`,
//!   então o gdb para nos breakpoints e mostra pilha e variáveis (RFC 0014, spike de debug).
//! - Sem nada disso: não compila.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;
use tea_core::config::DebugConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Build {
    Cargo,
    Zig,
    Moon,
    Shell(String),
}

impl Build {
    pub fn label(&self) -> &str {
        match self {
            Build::Cargo => "cargo build",
            Build::Zig => "zig build",
            Build::Moon => "moon build --target native -g",
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
        None if root.join("build.zig").is_file() => Some(Build::Zig),
        None if root.join("moon.mod").is_file() => Some(Build::Moon),
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
        Build::Zig => {
            let mut c = Command::new("zig");
            c.arg("build");
            c
        }
        Build::Moon => {
            let mut c = Command::new("moon");
            c.args(["build", "--target", "native", "-g", "--output-json"]);
            c
        }
        Build::Shell(line) => {
            #[cfg(windows)]
            {
                let mut c = Command::new("cmd");
                c.args(["/C", line]);
                c
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("sh");
                c.args(["-c", line]);
                c
            }
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
            let error = fl!("build-run-error", command = build.label(), error = e.to_string());
            return Outcome { error: Some(error), ..Outcome::default() };
        }
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    // No cargo, o stdout é JSON (para nós) e o stderr é o texto para gente. No moon, o JSON também traz os erros,
    // que viram texto (`arquivo:linha:coluna: error: ...`) para o log.
    let log = match build {
        Build::Cargo => stderr,
        Build::Moon => format!("{}{stderr}", moon_errors(&stdout)),
        _ => format!("{stdout}{stderr}"),
    };
    let ok = out.status.success();
    Outcome {
        ok,
        executables: match build {
            Build::Cargo => executables(&stdout),
            Build::Zig if ok => zig_executables(&root.join("zig-out/bin")),
            Build::Moon if ok => moon_executables(&stdout),
            _ => Vec::new(),
        },
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

/// Executáveis nativos nas mensagens `artifact` de `moon build --output-json` (a lista também traz bibliotecas
/// externas: só valem os `.exe` ou sem extensão).
fn moon_executables(json_lines: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for line in json_lines.lines() {
        let Ok(msg) = serde_json::from_str::<Value>(line) else { continue };
        if msg["$message_type"] == "artifact"
            && msg["target_backend"] == "native"
            && let Some(path) = msg["path"].as_str()
        {
            let path = PathBuf::from(path);
            let program = path.extension().is_none_or(|e| e.eq_ignore_ascii_case("exe"));
            if program && !found.contains(&path) {
                found.push(path);
            }
        }
    }
    found
}

/// Os erros de `moon build --output-json`, um por linha: `arquivo:linha:coluna: error: mensagem` (a primeira linha
/// da mensagem; o resto, recuado, embaixo).
fn moon_errors(json_lines: &str) -> String {
    let mut out = String::new();
    for line in json_lines.lines() {
        let Ok(msg) = serde_json::from_str::<Value>(line) else { continue };
        if msg["$message_type"] != "diagnostic" || msg["level"] != "error" {
            continue;
        }
        let path = msg["path"].as_str().unwrap_or_default();
        let at = msg["loc"].as_str().and_then(|l| l.split('-').next()).unwrap_or_default();
        let text = msg["message"].as_str().unwrap_or_default();
        let mut lines = text.lines();
        out.push_str(&format!("{path}:{at}: error: {}\n", lines.next().unwrap_or_default()));
        for rest in lines {
            out.push_str(&format!("    {}\n", rest.trim()));
        }
    }
    out
}

/// Executáveis em `zig-out/bin/` (o `zig build` instala lá), em ordem de nome.
fn zig_executables(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| {
            #[cfg(unix)]
            {
                tea_core::os::is_executable(&e.path())
            }
            #[cfg(not(unix))]
            {
                e.metadata().is_ok_and(|m| m.is_file())
                    && e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("exe"))
            }
        })
        .map(|e| e.path())
        .collect();
    found.sort();
    found
}

/// A primeira linha de erro (`error...`, ou `arquivo:linha:coluna: error: ...` do Zig e do gcc), ou a última
/// linha com texto.
fn first_error(log: &str) -> String {
    let line = log
        .lines()
        .find(|l| l.trim_start().starts_with("error") || l.contains(": error:"))
        .or_else(|| log.lines().rfind(|l| !l.trim().is_empty()))
        .map(str::to_string)
        .unwrap_or_else(|| fl!("build-no-message"));
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
        assert!(out.executables[0].is_file() && out.executables[0].file_stem() == Some(std::ffi::OsStr::new("demo")));

        std::fs::write(dir.join("src/main.rs"), "fn main() { y }\n").unwrap();
        let out = run(&Build::Cargo, &dir);
        assert!(!out.ok);
        assert!(out.error.as_deref().is_some_and(|e| e.starts_with("error")), "{:?}", out.error);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zig_projects_build_with_zig_build() {
        let dir = std::env::temp_dir().join(format!("idetea-zig-debug-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("zig-out/bin")).unwrap();
        assert_eq!(plan(&cfg(None), &dir), None);
        std::fs::write(dir.join("build.zig"), "").unwrap();
        assert_eq!(plan(&cfg(None), &dir), Some(Build::Zig));
        assert_eq!(plan(&cfg(Some("make")), &dir), Some(Build::Shell("make".into())), "o config vence");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let bin = dir.join("zig-out/bin");
            for (name, mode) in [("app", 0o755), ("tool", 0o755), ("leia-me.txt", 0o644)] {
                std::fs::write(bin.join(name), "").unwrap();
                std::fs::set_permissions(bin.join(name), std::fs::Permissions::from_mode(mode)).unwrap();
            }
            assert_eq!(zig_executables(&bin), [bin.join("app"), bin.join("tool")]);
            assert!(zig_executables(&dir.join("nao-existe")).is_empty());
        }

        let log = "install\n+- compile exe app Debug native 1 errors\nsrc/main.zig:3:5: error: use of undeclared identifier 'x'\n";
        assert_eq!(first_error(log), "src/main.zig:3:5: error: use of undeclared identifier 'x'");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn moonbit_modules_build_native_with_debug_info() {
        let dir = std::env::temp_dir().join(format!("idetea-moon-debug-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("moon.mod"), "name = \"x/y\"\n").unwrap();
        assert_eq!(plan(&cfg(None), &dir), Some(Build::Moon));
        assert_eq!(plan(&cfg(Some("make")), &dir), Some(Build::Shell("make".into())), "o config vence");
        let _ = std::fs::remove_dir_all(&dir);

        let ok = r#"{"$message_type":"artifact","package":"x/y/cmd/main","target_backend":"native","path":"/p/_build/native/debug/build/cmd/main/main.exe"}
{"$message_type":"artifact","package":"x/y/lib","target_backend":"native","path":"/p/_build/native/debug/build/lib/liblib.a","kind":"foreign"}
{"$message_type":"artifact","package":"x/y/web","target_backend":"wasm","path":"/p/_build/wasm/web.wasm"}"#;
        assert_eq!(moon_executables(ok), [PathBuf::from("/p/_build/native/debug/build/cmd/main/main.exe")]);

        let failed = r#"{"$message_type":"diagnostic","level":"error","error_code":4014,"path":"/p/sum.mbt","loc":"6:21-6:24","message":"Expr Type Mismatch\n        has type : String\n        wanted   : Int"}
{"$message_type":"diagnostic","level":"warning","path":"/p/a.mbt","loc":"1:1-1:2","message":"unused"}"#;
        let log = moon_errors(failed);
        assert_eq!(first_error(&log), "/p/sum.mbt:6:21: error: Expr Type Mismatch");
        assert!(log.contains("    has type : String") && !log.contains("unused"));
    }

    /// Compila um módulo MoonBit de verdade no backend nativo (pula sem `moon`).
    #[test]
    fn real_moon_build() {
        if tea_lsp::find_in_path("moon").is_none() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("idetea-moon-build-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("cmd/main")).unwrap();
        std::fs::write(dir.join("moon.mod"), "name = \"idetea/demo\"\n").unwrap();
        std::fs::write(dir.join("moon.pkg"), "").unwrap();
        std::fs::write(dir.join("cmd/main/moon.pkg"), "pkgtype(kind: \"executable\")\n").unwrap();
        std::fs::write(dir.join("cmd/main/main.mbt"), "fn main {\n  println(\"oi\")\n}\n").unwrap();
        let out = run(&Build::Moon, &dir);
        assert!(out.ok, "{}", out.log);
        assert_eq!(out.executables.len(), 1, "{:?}", out.executables);
        assert!(out.executables[0].is_file());

        std::fs::write(dir.join("cmd/main/main.mbt"), "fn main {\n  let x : Int = \"oi\"\n}\n").unwrap();
        let out = run(&Build::Moon, &dir);
        assert!(!out.ok);
        assert!(out.error.as_deref().is_some_and(|e| e.contains("main.mbt:2:") && e.contains("error")), "{:?}", out.error);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
