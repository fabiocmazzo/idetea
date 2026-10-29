//! Janelas nativas do sistema para escolher arquivo ou pasta (`rfd`).
//! No Linux usa o portal do desktop (xdg-desktop-portal, via D-Bus) e, sem portal, o `zenity`.
//! A janela roda numa thread do `rfd`; aqui só esperamos a resposta.

use std::path::{Path, PathBuf};

use rfd::AsyncFileDialog;

/// O que o usuário escolheu. `None` = cancelou (ou nenhuma janela pôde abrir).
#[derive(Debug, Clone)]
pub enum Picked {
    /// Salvar a aba `tab` neste caminho.
    SaveAs { tab: u64, path: Option<PathBuf> },
    Files(Option<Vec<PathBuf>>),
    Folder(Option<PathBuf>),
    /// Executável para depurar.
    DebugProgram(Option<PathBuf>),
}

pub async fn save_as(tab: u64, dir: PathBuf, name: String) -> Picked {
    let mut dialog = AsyncFileDialog::new().set_title("Salvar como").set_directory(&dir);
    if !name.is_empty() {
        dialog = dialog.set_file_name(name);
    }
    let path = dialog.save_file().await.map(|f| f.path().to_path_buf());
    Picked::SaveAs { tab, path }
}

pub async fn open_files(dir: PathBuf) -> Picked {
    let files = AsyncFileDialog::new().set_title("Abrir arquivos").set_directory(&dir).pick_files().await;
    Picked::Files(files.map(|f| f.iter().map(|h| h.path().to_path_buf()).collect()))
}

pub async fn open_folder(dir: PathBuf) -> Picked {
    let folder = AsyncFileDialog::new().set_title("Abrir pasta").set_directory(&dir).pick_folder().await;
    Picked::Folder(folder.map(|f| f.path().to_path_buf()))
}

pub async fn debug_program(dir: PathBuf) -> Picked {
    let file = AsyncFileDialog::new().set_title("Programa para depurar").set_directory(&dir).pick_file().await;
    Picked::DebugProgram(file.map(|f| f.path().to_path_buf()))
}

/// Abre outra janela da IDE com `dir` como raiz (a raiz desta janela não muda).
pub fn open_window(dir: &Path) -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let mut child = std::process::Command::new(exe)
        .arg(dir)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .spawn()?;
    // Espera numa thread para não deixar processo zumbi quando a outra janela fechar.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
