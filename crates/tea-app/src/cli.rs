//! Linha de comando: `idetea [--no-splash] [caminho...]`
//!
//! - Raiz do projeto = pasta atual, a menos que um argumento seja uma pasta.
//! - Todo arquivo passado é aberto (mesmo fora da raiz ou ainda inexistente).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
Uso: idetea [opções] [caminho...]

  caminho        arquivo para abrir (pode não existir: é criado ao salvar)
                 ou pasta para usar como raiz do projeto
  --no-splash    não mostra a splash screen (também: IDETEA_NO_SPLASH=1)
  -h, --help     mostra esta ajuda
  -V, --version  mostra a versão";

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Run(Launch),
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Launch {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
    pub no_splash: bool,
    pub warnings: Vec<String>,
}

pub fn parse(args: impl IntoIterator<Item = OsString>, cwd: &Path, env_no_splash: bool) -> Cli {
    let mut launch = Launch { root: cwd.to_path_buf(), files: Vec::new(), no_splash: env_no_splash, warnings: Vec::new() };
    let mut root_set = false;
    let mut flags_done = false;

    for arg in args {
        if !flags_done && let Some(flag) = arg.to_str().filter(|a| a.starts_with('-') && a.len() > 1) {
            match flag {
                "--" => flags_done = true,
                "--no-splash" => launch.no_splash = true,
                "-h" | "--help" => return Cli::Help,
                "-V" | "--version" => return Cli::Version,
                other => launch.warnings.push(format!("opção desconhecida ignorada: {other}")),
            }
            continue;
        }
        let path = absolute(cwd, Path::new(&arg));
        if path.is_dir() {
            if root_set {
                launch.warnings.push(format!("só uma pasta raiz por vez, ignorando {}", path.display()));
            } else {
                launch.root = path;
                root_set = true;
            }
        } else if !launch.files.contains(&path) {
            launch.files.push(path);
        }
    }
    Cli::Run(launch)
}

/// `IDETEA_NO_SPLASH` ligado = qualquer valor que não seja vazio, `0` ou `false`.
pub fn env_flag(value: Option<OsString>) -> bool {
    value.is_some_and(|v| !matches!(v.to_str(), Some("" | "0" | "false")))
}

/// Caminho absoluto e limpo. Usa `canonicalize` quando o arquivo (ou a pasta dele) existe.
fn absolute(cwd: &Path, p: &Path) -> PathBuf {
    let joined = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
    if let Ok(c) = joined.canonicalize() {
        return c;
    }
    // Arquivo novo: canoniza a pasta mais próxima que existe e junta o resto.
    let mut existing = joined.as_path();
    let mut rest = Vec::new();
    while let (Some(parent), Some(name)) = (existing.parent(), existing.file_name()) {
        rest.push(name.to_os_string());
        existing = parent;
        if let Ok(mut c) = existing.canonicalize() {
            for part in rest.iter().rev() {
                c.push(part);
            }
            return c;
        }
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tea-cli-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("outro")).unwrap();
        std::fs::write(dir.join("src/main.rs"), "").unwrap();
        std::fs::write(dir.join("a.rs"), "").unwrap();
        dir.canonicalize().unwrap()
    }

    fn run(cwd: &Path, args: &[&str]) -> Launch {
        match parse(args.iter().map(OsString::from), cwd, false) {
            Cli::Run(l) => l,
            other => panic!("esperava Run, veio {other:?}"),
        }
    }

    #[test]
    fn no_args_uses_cwd() {
        let cwd = setup("none");
        let l = run(&cwd, &[]);
        assert_eq!(l.root, cwd);
        assert!(l.files.is_empty());
        assert!(!l.no_splash);
    }

    #[test]
    fn file_arg_keeps_cwd_as_root() {
        let cwd = setup("file");
        let l = run(&cwd, &["src/main.rs"]);
        assert_eq!(l.root, cwd, "raiz é a pasta atual, não a pasta do arquivo");
        assert_eq!(l.files, [cwd.join("src/main.rs")]);
    }

    #[test]
    fn dir_arg_becomes_root() {
        let cwd = setup("dir");
        let l = run(&cwd, &["outro"]);
        assert_eq!(l.root, cwd.join("outro"));
    }

    #[test]
    fn many_files_new_files_and_dedup() {
        let cwd = setup("many");
        let l = run(&cwd, &["a.rs", "./src/../a.rs", "src/novo/x.rs", "--no-splash"]);
        assert_eq!(l.files, [cwd.join("a.rs"), cwd.join("src/novo/x.rs")]);
        assert!(l.no_splash);
    }

    #[test]
    fn flags() {
        let cwd = setup("flags");
        assert_eq!(parse([OsString::from("--help")], &cwd, false), Cli::Help);
        assert_eq!(parse([OsString::from("-V")], &cwd, false), Cli::Version);
        let l = run(&cwd, &["--wat", "--", "--no-splash"]);
        assert_eq!(l.warnings.len(), 1);
        assert!(!l.no_splash, "depois de -- é caminho, não opção");
        assert_eq!(l.files, [cwd.join("--no-splash")]);
    }

    #[test]
    fn env_flag_values() {
        assert!(!env_flag(None));
        assert!(!env_flag(Some("0".into())));
        assert!(!env_flag(Some("".into())));
        assert!(env_flag(Some("1".into())));
        assert!(env_flag(Some("yes".into())));
    }
}
