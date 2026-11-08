//! Cópia das abas com alterações não salvas, para recuperar depois de um travamento.
//!
//! Fica em `$XDG_STATE_HOME/idetea/backups/<projeto>/`: para cada aba, `<pid>-<chave>.txt` (o texto) e
//! `<pid>-<chave>.toml` (de que arquivo é). O `pid` diz de qual janela é a cópia: a de uma janela que ainda
//! está aberta não é oferecida para recuperar. Sair normalmente apaga as cópias da janela.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// De que aba é a cópia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    /// `None`: aba sem nome.
    pub path: Option<PathBuf>,
    pub pid: u32,
    /// Segundos desde 1970 (para ficar com a mais nova quando há duas do mesmo arquivo).
    pub saved: u64,
}

/// Uma cópia deixada por uma janela que não está mais aberta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orphan {
    pub meta: Meta,
    pub text: String,
    files: [PathBuf; 2],
}

impl Orphan {
    pub fn remove(&self) {
        for f in &self.files {
            let _ = std::fs::remove_file(f);
        }
    }
}

/// Pasta das cópias de um projeto.
pub fn dir_for(root: &Path) -> Option<PathBuf> {
    let session = crate::session::path_for(root)?;
    let name = session.file_stem()?.to_owned();
    Some(session.parent()?.parent()?.join("backups").join(name))
}

/// Chave de uma aba: o hash do caminho, ou o id da aba (sem nome).
pub fn key(path: Option<&Path>, tab_id: u64) -> String {
    match path {
        Some(p) => format!("{:016x}", crate::session::fnv1a(p.as_os_str().as_encoded_bytes())),
        None => format!("sem-nome-{tab_id}"),
    }
}

fn files(dir: &Path, pid: u32, key: &str) -> [PathBuf; 2] {
    [dir.join(format!("{pid}-{key}.txt")), dir.join(format!("{pid}-{key}.toml"))]
}

/// Grava a cópia: o texto primeiro, a descrição depois (sem a descrição, a cópia não conta).
pub fn write(dir: &Path, key: &str, path: Option<&Path>, text: &str) {
    let pid = std::process::id();
    let [txt, meta] = files(dir, pid, key);
    let saved = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let info = Meta { path: path.map(Path::to_path_buf), pid, saved };
    let _ = std::fs::create_dir_all(dir);
    if std::fs::write(&txt, text).is_ok()
        && let Ok(t) = toml::to_string(&info)
    {
        let _ = std::fs::write(meta, t);
    }
}

pub fn remove(dir: &Path, key: &str) {
    for f in files(dir, std::process::id(), key) {
        let _ = std::fs::remove_file(f);
    }
}

/// Apaga todas as cópias desta janela (ao sair normalmente).
pub fn remove_mine(dir: &Path) {
    let prefix = format!("{}-", std::process::id());
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        if e.file_name().to_string_lossy().starts_with(&prefix) {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let _ = std::fs::remove_dir(dir);
}

/// Cópias de janelas que não estão mais abertas, uma por arquivo (a mais nova), sem as que já batem com o
/// disco (travou logo depois de salvar).
pub fn orphans(dir: &Path) -> Vec<Orphan> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<Orphan> = Vec::new();
    for e in entries.flatten() {
        let meta_path = e.path();
        if meta_path.extension().is_none_or(|x| x != "toml") {
            continue;
        }
        let Some(meta) = std::fs::read_to_string(&meta_path).ok().and_then(|t| toml::from_str::<Meta>(&t).ok()) else {
            continue;
        };
        if is_running(meta.pid) {
            continue;
        }
        let txt = meta_path.with_extension("txt");
        let Ok(text) = std::fs::read_to_string(&txt) else { continue };
        let orphan = Orphan { meta, text, files: [txt, meta_path] };
        let same_as_disk = orphan.meta.path.as_ref().is_some_and(|p| std::fs::read_to_string(p).is_ok_and(|d| d == orphan.text));
        if same_as_disk {
            orphan.remove();
            continue;
        }
        // Duas cópias do mesmo arquivo (duas janelas travaram): fica a mais nova, a outra sai.
        if let Some(other) = orphan.meta.path.as_ref().and_then(|p| found.iter_mut().find(|o| o.meta.path.as_ref() == Some(p))) {
            if orphan.meta.saved > other.meta.saved {
                other.remove();
                *other = orphan;
            } else {
                orphan.remove();
            }
            continue;
        }
        found.push(orphan);
    }
    found.sort_by(|a, b| a.meta.path.cmp(&b.meta.path));
    found
}

/// A janela dona da cópia ainda está aberta? No Linux olha `/proc` (e confere que o processo é a IDE, por
/// causa de `pid` reaproveitado depois de reiniciar o computador). Em macOS e Windows, confere se o `pid` está vivo.
fn is_running(pid: u32) -> bool {
    if pid == std::process::id() {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        let exe = std::fs::read_link(format!("/proc/{pid}/exe"));
        match (exe, std::env::current_exe()) {
            (Ok(theirs), Ok(ours)) => theirs == ours,
            _ => false,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tea_core::os::process_alive(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_find_and_clean() {
        let dir = std::env::temp_dir().join(format!("idetea-backup-{}", std::process::id()));
        let file = dir.join("a.rs");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&file, "no disco").unwrap();
        let k = key(Some(&file), 1);
        write(&dir, &k, Some(&file), "alterado");
        assert!(orphans(&dir).is_empty(), "a cópia é desta janela, que está aberta");

        // Uma cópia de uma janela que já fechou (pid que não existe).
        let fake = |pid: u32, key: &str, path: Option<&Path>, text: &str, saved: u64| {
            let [txt, meta] = files(&dir, pid, key);
            std::fs::write(txt, text).unwrap();
            std::fs::write(meta, toml::to_string(&Meta { path: path.map(Path::to_path_buf), pid, saved }).unwrap()).unwrap();
        };
        fake(999_999_001, &k, Some(&file), "velho", 10);
        fake(999_999_002, &k, Some(&file), "novo", 20);
        fake(999_999_003, "sem-nome-4", None, "rascunho", 5);
        fake(999_999_004, "igual", Some(&file), "no disco", 30);
        let found = orphans(&dir);
        let texts: Vec<_> = found.iter().map(|o| o.text.as_str()).collect();
        assert_eq!(texts, ["rascunho", "novo"], "sem nome primeiro; do mesmo arquivo fica a mais nova; igual ao disco sai");
        for o in &found {
            o.remove();
        }
        assert!(orphans(&dir).is_empty());

        remove_mine(&dir);
        assert!(!files(&dir, std::process::id(), &k)[0].exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
