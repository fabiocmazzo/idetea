//! Projetos e arquivos abertos por último, para "Abrir recente" (`Ctrl+R`).
//! Uma lista para todas as janelas, em `$XDG_STATE_HOME/idetea/recent.toml`. Cada mudança relê o arquivo
//! antes de gravar: duas janelas abertas não apagam o que a outra acrescentou.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const MAX_PROJECTS: usize = 20;
const MAX_FILES: usize = 50;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Recent {
    /// Do mais novo para o mais velho.
    pub projects: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

impl Recent {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|t| toml::from_str(&t).ok()).unwrap_or_default()
    }

    fn save(&self, path: &Path) {
        let Some(dir) = path.parent() else { return };
        let _ = std::fs::create_dir_all(dir);
        let Ok(text) = toml::to_string(self) else { return };
        let tmp = path.with_extension("toml.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    /// Põe o projeto no topo e grava.
    pub fn add_project(path: &Path, root: &Path) {
        let mut r = Self::load(path);
        if push_front(&mut r.projects, root, MAX_PROJECTS) {
            r.save(path);
        }
    }

    /// Põe o arquivo no topo e grava.
    pub fn add_file(path: &Path, file: &Path) {
        let mut r = Self::load(path);
        if push_front(&mut r.files, file, MAX_FILES) {
            r.save(path);
        }
    }
}

/// Move (ou acrescenta) `item` para o começo, sem passar de `max`. `false` se já estava no topo.
fn push_front(list: &mut Vec<PathBuf>, item: &Path, max: usize) -> bool {
    if list.first().map(PathBuf::as_path) == Some(item) {
        return false;
    }
    list.retain(|p| p != item);
    list.insert(0, item.to_path_buf());
    list.truncate(max);
    true
}

pub fn path() -> Option<PathBuf> {
    Some(crate::ui_state::state_path()?.parent()?.join("recent.toml"))
}

/// Caminho para mostrar: `~` no lugar da pasta pessoal.
pub fn display(path: &Path) -> String {
    if let Some(home) = tea_core::os::home_dir()
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return Path::new("~").join(rest).display().to_string();
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_without_repeats() {
        let dir = std::env::temp_dir().join(format!("idetea-recent-{}", std::process::id()));
        let path = dir.join("recent.toml");
        Recent::add_project(&path, Path::new("/a"));
        Recent::add_project(&path, Path::new("/b"));
        Recent::add_project(&path, Path::new("/a"));
        Recent::add_file(&path, Path::new("/a/x.rs"));
        let r = Recent::load(&path);
        assert_eq!(r.projects, [PathBuf::from("/a"), PathBuf::from("/b")]);
        assert_eq!(r.files, [PathBuf::from("/a/x.rs")]);

        let mut list: Vec<PathBuf> = (0..5).map(|i| PathBuf::from(format!("/{i}"))).collect();
        assert!(push_front(&mut list, Path::new("/9"), 5));
        assert_eq!(list.len(), 5, "não passa do máximo");
        assert_eq!(list[0], PathBuf::from("/9"));
        assert!(!push_front(&mut list, Path::new("/9"), 5), "já no topo: nada a gravar");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
