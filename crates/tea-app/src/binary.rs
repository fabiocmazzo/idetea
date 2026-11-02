//! O executável da IDE foi recompilado (ou atualizado) com ela aberta: avisar e oferecer reiniciar.
//!
//! Na abertura, guarda o caminho e a data do executável (no AppImage, o `.AppImage`). Uma thread olha a data a cada
//! poucos segundos; mudou, manda uma mensagem só (a IDE avisa e marca o título). "Reiniciar a IDE" roda o executável
//! novo com os mesmos argumentos, solto deste processo, e sai.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// De quanto em quanto tempo olhar o executável.
const EVERY: Duration = Duration::from_secs(3);

/// O executável e a data dele quando a IDE abriu.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stamp {
    pub path: PathBuf,
    modified: Option<SystemTime>,
}

impl Stamp {
    /// O executável deste processo. Leia na abertura: depois de recompilado, o Linux mostra o caminho com
    /// " (deleted)".
    pub fn current() -> Option<Stamp> {
        let path = std::env::var_os("APPIMAGE")
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| std::env::current_exe().ok())?;
        Some(Stamp::of(path))
    }

    fn of(path: PathBuf) -> Stamp {
        let modified = modified(&path);
        Stamp { path, modified }
    }

    /// O arquivo no caminho é outro: data diferente e ainda existe (no meio de uma compilação pode faltar).
    pub fn changed(&self) -> bool {
        modified(&self.path).is_some_and(|now| Some(now) != self.modified)
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Avisa (uma vez) quando o executável mudar. Espera a data parar de mudar entre duas olhadas: o linker ainda pode
/// estar escrevendo.
pub fn watch(stamp: &Stamp) -> iced::futures::channel::mpsc::UnboundedReceiver<()> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    let stamp = stamp.clone();
    std::thread::spawn(move || {
        let mut seen = None;
        loop {
            std::thread::sleep(EVERY);
            if tx.is_closed() {
                return;
            }
            if !stamp.changed() {
                continue;
            }
            let now = modified(&stamp.path);
            if seen.is_some() && seen == now {
                let _ = tx.unbounded_send(());
                return;
            }
            seen = now;
        }
    });
    rx
}

/// Roda o executável novo com os mesmos argumentos, solto deste processo (o log vai para o de sempre).
pub fn restart(stamp: &Stamp) -> std::io::Result<()> {
    crate::detach::spawn_exe(&stamp.path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notices_a_new_file() {
        let dir = std::env::temp_dir().join(format!("idetea-binary-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("idetea");
        std::fs::write(&exe, "velho").unwrap();
        let stamp = Stamp::of(exe.clone());
        assert!(!stamp.changed());

        // Compilar troca o arquivo (outro inode, outra data).
        std::fs::remove_file(&exe).unwrap();
        assert!(!stamp.changed(), "sumiu no meio da compilação");
        std::fs::write(&exe, "novo").unwrap();
        let later = SystemTime::now() + Duration::from_secs(5);
        std::fs::File::options().write(true).open(&exe).unwrap().set_modified(later).unwrap();
        assert!(stamp.changed());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
