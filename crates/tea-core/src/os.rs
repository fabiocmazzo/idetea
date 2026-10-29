//! Integração com o sistema operacional.

use std::path::Path;

/// Abre a pasta no gerenciador de arquivos do sistema (sem esperar ele fechar).
pub fn open_in_file_manager(dir: &Path) -> std::io::Result<()> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let mut child = std::process::Command::new(program)
        .arg(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    // Espera numa thread para o processo não virar zumbi.
    std::thread::spawn(move || child.wait());
    Ok(())
}
