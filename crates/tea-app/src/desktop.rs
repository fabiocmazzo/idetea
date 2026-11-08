//! `idetea --install-desktop`: põe a IDE no menu de aplicativos do usuário (`~/.local/share/applications`) com o
//! ícone (`~/.local/share/icons/hicolor/scalable/apps`). O `.desktop` aponta para este executável (ou para o
//! AppImage, se for ele que está rodando). O nome `idetea.desktop` é o app id da janela: é por ele que o
//! painel do desktop acha o ícone.

use std::path::{Path, PathBuf};

const DESKTOP: &str = include_str!("../../../assets/idetea.desktop");
const ICON: &str = include_str!("../../../assets/idetea.svg");

pub fn run(install: bool) -> iced::Result {
    tea_core::i18n::init("auto");
    let done = if install { self::install() } else { uninstall() };
    match done {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn data_dir() -> Option<PathBuf> {
    tea_core::os::data_dir()
}

fn paths() -> Result<(PathBuf, PathBuf), String> {
    let data = data_dir().ok_or_else(|| fl!("desktop-no-home"))?;
    Ok((data.join("applications/idetea.desktop"), data.join("icons/hicolor/scalable/apps/idetea.svg")))
}

fn install() -> Result<String, String> {
    // Dentro de um AppImage o executável fica num ponto de montagem temporário: o que vale é o arquivo .AppImage.
    let exe = match std::env::var_os("APPIMAGE") {
        Some(appimage) => PathBuf::from(appimage),
        None => std::env::current_exe().map_err(|e| fl!("desktop-error", error = e.to_string()))?,
    };
    let (desktop, icon) = paths()?;
    let write = |path: &Path, text: &str| -> Result<(), String> {
        let err = |e: std::io::Error| fl!("desktop-error", error = format!("{}: {e}", path.display()));
        std::fs::create_dir_all(path.parent().unwrap_or(path)).map_err(err)?;
        std::fs::write(path, text).map_err(err)
    };
    write(&desktop, &desktop_entry(&exe))?;
    write(&icon, ICON)?;
    refresh_caches(&desktop, &icon);
    Ok(fl!("desktop-installed", desktop = desktop.display().to_string(), exe = exe.display().to_string()))
}

fn uninstall() -> Result<String, String> {
    let (desktop, icon) = paths()?;
    for p in [&desktop, &icon] {
        match std::fs::remove_file(p) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(fl!("desktop-error", error = format!("{}: {e}", p.display()))),
        }
    }
    refresh_caches(&desktop, &icon);
    Ok(fl!("desktop-removed"))
}

/// O menu e os ícones atualizam sozinhos na maioria dos desktops; estes comandos só aceleram (se existirem).
fn refresh_caches(desktop: &Path, icon: &Path) {
    let quiet = |cmd: &str, args: &[&std::ffi::OsStr]| {
        let _ = std::process::Command::new(cmd)
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    };
    if let Some(apps) = desktop.parent() {
        quiet("update-desktop-database", &["-q".as_ref(), apps.as_os_str()]);
    }
    // .../icons/hicolor/scalable/apps/idetea.svg → .../icons/hicolor
    if let Some(theme) = icon.ancestors().nth(3) {
        quiet("gtk-update-icon-cache", &["-q".as_ref(), "-t".as_ref(), theme.as_os_str()]);
    }
}

/// O `.desktop` de `assets/` com o caminho do executável no `Exec`.
fn desktop_entry(exe: &Path) -> String {
    DESKTOP.replace("@EXEC@", &exec_arg(&exe.to_string_lossy()))
}

/// Caminho no `Exec` do `.desktop`: entre aspas se tiver algo além de letras, números e `/._-+`; dentro das
/// aspas, `"`, `` ` ``, `$` e `\` levam `\` (e o `\` dobra de novo, porque o valor do arquivo também escapa).
fn exec_arg(path: &str) -> String {
    if path.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+".contains(c)) {
        return path.to_string();
    }
    let mut out = String::from("\"");
    for c in path.chars() {
        match c {
            '\\' => out.push_str("\\\\\\\\"),
            '"' | '`' | '$' => {
                out.push_str("\\\\");
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_points_to_the_executable() {
        let e = desktop_entry(Path::new("/home/x/.cargo/bin/idetea"));
        assert!(e.contains("\nExec=/home/x/.cargo/bin/idetea --menu %F\n"), "{e}");
        assert!(!e.contains("NoDisplay"), "aparece no menu");
        assert_eq!(exec_arg("/opt/IDE TEA/idetea"), "\"/opt/IDE TEA/idetea\"");
        assert_eq!(exec_arg("/a$b"), "\"/a\\\\$b\"");
    }
}
