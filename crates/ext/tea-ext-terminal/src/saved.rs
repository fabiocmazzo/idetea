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
}
