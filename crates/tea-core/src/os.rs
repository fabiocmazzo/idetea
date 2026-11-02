//! Integração com o sistema operacional.

use std::path::Path;

/// Abre a pasta no gerenciador de arquivos do sistema (sem esperar ele fechar).
pub fn open_in_file_manager(dir: &Path) -> std::io::Result<()> {
    open_with_system(dir.as_os_str())
}

/// Abre um endereço (`https://...`, `mailto:...`) no programa padrão do sistema.
pub fn open_url(url: &str) -> std::io::Result<()> {
    open_with_system(std::ffi::OsStr::new(url))
}

/// Abre um arquivo no programa padrão do sistema para o tipo dele.
pub fn open_path(path: &Path) -> std::io::Result<()> {
    open_with_system(path.as_os_str())
}

fn open_with_system(target: &std::ffi::OsStr) -> std::io::Result<()> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let mut child = std::process::Command::new(program)
        .arg(target)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    // Espera numa thread para o processo não virar zumbi.
    std::thread::spawn(move || child.wait());
    Ok(())
}
