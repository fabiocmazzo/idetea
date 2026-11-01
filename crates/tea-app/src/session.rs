//! Sessão de cada projeto: abas (com o cursor), aba ativa e painéis abertos, para reabrir como estava.
//! Fica em `$XDG_STATE_HOME/idetea/sessions/<pasta>-<hash da raiz>.toml`: fora do projeto, nada para o git.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    /// Raiz do projeto (confere que o arquivo é mesmo deste projeto).
    pub root: PathBuf,
    pub active: usize,
    /// Painel em destaque no slot de baixo.
    pub bottom: Option<String>,
    /// Painel → visível. Painel que não está aqui (extensão nova) fica como a extensão quer.
    pub panels: BTreeMap<String, bool>,
    pub tabs: Vec<SessionTab>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionTab {
    pub path: PathBuf,
    /// Começam em 1.
    pub line: usize,
    pub column: usize,
}

impl Session {
    /// Lê a sessão do projeto. Arquivo faltando, estragado ou de outra raiz: `None` (abre vazio, sem avisar).
    pub fn load(path: &Path, root: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let session: Session = toml::from_str(&text).ok()?;
        (session.root == root).then_some(session)
    }

    /// Grava num arquivo temporário e troca: um travamento no meio não deixa a sessão pela metade.
    pub fn save(&self, path: &Path) {
        let Some(dir) = path.parent() else { return };
        let _ = std::fs::create_dir_all(dir);
        let Ok(text) = toml::to_string(self) else { return };
        let tmp = path.with_extension("toml.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    /// O que muda a "forma" da sessão (abas, aba ativa, painéis), sem o cursor. Mudou: vale gravar.
    pub fn same_shape(&self, other: &Session) -> bool {
        let paths = |s: &Session| s.tabs.iter().map(|t| t.path.clone()).collect::<Vec<_>>();
        self.active == other.active
            && self.bottom == other.bottom
            && self.panels == other.panels
            && paths(self) == paths(other)
    }
}

/// Onde fica a sessão de um projeto. O nome leva a pasta (para achar à mão) e um hash do caminho inteiro
/// (duas pastas `app` em lugares diferentes não se misturam).
pub fn path_for(root: &Path) -> Option<PathBuf> {
    let dir = crate::ui_state::state_path()?.parent()?.join("sessions");
    let name: String = root
        .file_name()
        .map(|n| n.to_string_lossy().chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect())
        .unwrap_or_default();
    Some(dir.join(format!("{name}-{:016x}.toml", fnv1a(root.as_os_str().as_encoded_bytes()))))
}

/// FNV-1a de 64 bits: estável entre versões do Rust (o `DefaultHasher` não promete isso).
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_other_root() {
        let dir = std::env::temp_dir().join(format!("idetea-session-{}", std::process::id()));
        let path = dir.join("s.toml");
        let root = PathBuf::from("/home/x/proj");
        let s = Session {
            root: root.clone(),
            active: 1,
            bottom: Some("git".into()),
            panels: [("git".to_string(), true), ("outline".to_string(), false)].into(),
            tabs: vec![
                SessionTab { path: root.join("a.rs"), line: 10, column: 3 },
                SessionTab { path: root.join("b.rs"), line: 1, column: 1 },
            ],
        };
        s.save(&path);
        assert_eq!(Session::load(&path, &root), Some(s.clone()));
        assert_eq!(Session::load(&path, Path::new("/outro")), None, "sessão de outro projeto");
        std::fs::write(&path, "tabs = 3").unwrap();
        assert_eq!(Session::load(&path, &root), None, "arquivo estragado");

        let mut moved = s.clone();
        moved.tabs[0].line = 99;
        assert!(s.same_shape(&moved), "só o cursor mudou");
        moved.active = 0;
        assert!(!s.same_shape(&moved));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_name_per_project() {
        let a = path_for(Path::new("/home/x/app")).unwrap();
        let b = path_for(Path::new("/srv/app")).unwrap();
        assert_ne!(a, b);
        assert!(a.file_name().unwrap().to_string_lossy().starts_with("app-"));
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
    }
}
