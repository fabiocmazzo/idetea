//! As abas de terminal de um projeto, para reabrir como estavam (RFC 0024, M2).
//!
//! Fica em `$XDG_STATE_HOME/idetea/terminals/<pasta>-<hash da raiz>.toml`, fora do projeto. Guarda os shells (com o
//! nome que o usuário deu) e os agentes abertos pelo lançador (o comando, a worktree e o id da conversa, quando a IDE
//! sabe). A tarefa e os terminais pedidos por agentes não voltam: o pedido já virou `lost`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Shell,
    Agent,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tab {
    pub kind: Kind,
    /// O nome que o usuário deu à aba.
    pub name: Option<String>,
    /// Agente: o título da aba ("Claude Code").
    pub title: String,
    /// Agente: o comando com que abriu.
    pub command: String,
    /// Agente: o branch da worktree onde rodava.
    pub worktree: Option<String>,
    /// Agente: o id da conversa, quando a IDE o escolheu (`claude --session-id`).
    pub session: Option<String>,
    /// O texto da aba na sessão anterior (`keep_scrollback`), já pronto para mostrar. Fica em arquivo à parte.
    #[serde(skip)]
    pub text: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Saved {
    /// Raiz do projeto (confere que o arquivo é deste projeto).
    pub root: PathBuf,
    pub tabs: Vec<Tab>,
}

impl Saved {
    // Nos testes a extensão não grava em lugar nenhum.
    #[cfg_attr(test, allow(dead_code))]
    pub fn path(root: &Path) -> Option<PathBuf> {
        tea_core::os::project_state_file("terminals", root)
    }

    /// A pasta dos textos das abas (`keep_scrollback`): ao lado do arquivo das abas, com o mesmo nome.
    pub fn text_dir(path: &Path) -> PathBuf {
        path.with_extension("")
    }

    /// Lê as abas guardadas. Arquivo faltando, estragado ou de outra raiz: `None`.
    pub fn load(path: &Path, root: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let saved: Saved = toml::from_str(&text).ok()?;
        (saved.root == root).then_some(saved)
    }

    /// Grava pelo temporário. Sem nenhuma aba, apaga o arquivo.
    pub fn save(&self, path: &Path) {
        if self.tabs.is_empty() {
            let _ = std::fs::remove_file(path);
            return;
        }
        let Some(dir) = path.parent() else { return };
        let _ = std::fs::create_dir_all(dir);
        let Ok(text) = toml::to_string(self) else { return };
        let tmp = path.with_extension("toml.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}

/// Grava o texto de cada aba (`<posição>.txt`, só o usuário lê) e apaga os da vez anterior. Sem nenhum texto, a
/// pasta some.
pub fn write_texts(dir: &Path, texts: &[Option<String>]) {
    let _ = std::fs::remove_dir_all(dir);
    if texts.iter().all(|t| t.as_deref().is_none_or(str::is_empty)) {
        return;
    }
    if create_private_dir(dir).is_err() {
        return;
    }
    for (i, text) in texts.iter().enumerate() {
        if let Some(text) = text.as_deref().filter(|t| !t.is_empty()) {
            let _ = write_private(&dir.join(format!("{i}.txt")), text.as_bytes());
        }
    }
}

/// Lê (e apaga) os textos gravados: o da aba `i` vira a saída que aparece antes do prompt, com a linha
/// `separator` embaixo (`{when}` vira a hora em que foi gravado).
pub fn take_texts(dir: &Path, count: usize, separator: impl Fn(&str) -> String) -> Vec<Option<Vec<u8>>> {
    let texts = (0..count)
        .map(|i| {
            let path = dir.join(format!("{i}.txt"));
            let text = std::fs::read_to_string(&path).ok().filter(|t| !t.trim().is_empty())?;
            let when = std::fs::metadata(&path).and_then(|m| m.modified()).map(tea_core::os::local_date_time).unwrap_or_default();
            Some(preamble(&text, &separator(&when)))
        })
        .collect();
    let _ = std::fs::remove_dir_all(dir);
    texts
}

/// O texto como um programa o escreveria (`\r\n`), e a linha de separação apagada (`ESC[2m`).
fn preamble(text: &str, separator: &str) -> Vec<u8> {
    let mut out = text.replace('\n', "\r\n");
    out.push_str("\r\n\x1b[2m");
    out.push_str(separator);
    out.push_str("\x1b[0m\r\n");
    out.into_bytes()
}

fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
    }
    #[cfg(not(unix))]
    std::fs::create_dir_all(dir)
}

fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)?.write_all(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_other_root_and_empty() {
        let dir = std::env::temp_dir().join(format!("idetea-saved-terms-{}", std::process::id()));
        let path = dir.join("t.toml");
        let root = PathBuf::from("/x/app");
        let saved = Saved {
            root: root.clone(),
            tabs: vec![
                Tab { name: Some("servidor".into()), ..Tab::default() },
                Tab {
                    kind: Kind::Agent,
                    title: "Claude Code".into(),
                    command: "claude".into(),
                    worktree: Some("fix/login".into()),
                    session: Some("6f1c".into()),
                    ..Tab::default()
                },
            ],
        };
        saved.save(&path);
        assert_eq!(Saved::load(&path, &root), Some(saved.clone()));
        assert_eq!(Saved::load(&path, Path::new("/y/app")), None, "de outro projeto");
        Saved { root: root.clone(), tabs: Vec::new() }.save(&path);
        assert!(!path.exists(), "sem abas, sem arquivo");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn texts_are_private_read_once_and_get_a_separator() {
        let dir = std::env::temp_dir().join(format!("idetea-saved-texts-{}", std::process::id()));
        write_texts(&dir, &[Some("ls\nsaida".into()), None, Some("make".into())]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(dir.join("0.txt")).unwrap().permissions().mode() & 0o777, 0o600);
            assert_eq!(std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777, 0o700);
        }
        let texts = take_texts(&dir, 3, |when| format!("— antes, {when} —"));
        let first = String::from_utf8(texts[0].clone().unwrap()).unwrap();
        assert!(first.starts_with("ls\r\nsaida\r\n\x1b[2m— antes, "), "{first:?}");
        assert!(first.ends_with(" —\x1b[0m\r\n"));
        assert_eq!(texts[1], None);
        assert!(texts[2].is_some());
        assert!(!dir.exists(), "lido uma vez: a pasta some");
        write_texts(&dir, &[None, Some(String::new())]);
        assert!(!dir.exists(), "sem texto, sem pasta");
    }
}
