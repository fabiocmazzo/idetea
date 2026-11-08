//! `idetea --install-desktop`: põe a IDE no menu de aplicativos do usuário (`~/.local/share/applications`) com o
//! ícone (`~/.local/share/icons/hicolor/scalable/apps`). O `.desktop` aponta para este executável (ou para o
//! AppImage, se for ele que está rodando). O nome `idetea.desktop` é o app id da janela: é por ele que o
//! painel do desktop acha o ícone.

use std::path::{Path, PathBuf};

#[cfg(any(not(target_os = "macos"), test))]
const DESKTOP: &str = include_str!("../../../assets/idetea.desktop");
#[cfg(not(target_os = "macos"))]
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

#[cfg(not(target_os = "macos"))]
fn data_dir() -> Option<PathBuf> {
    tea_core::os::data_dir()
}

#[cfg(not(target_os = "macos"))]
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
    #[cfg(target_os = "macos")]
    {
        let home = tea_core::os::home_dir().ok_or_else(|| fl!("desktop-no-home"))?;
        let app = home.join("Applications/IDE TEA.app");
        let contents = app.join("Contents");
        let macos_dir = contents.join("MacOS");
        let err = |p: &Path, e: std::io::Error| fl!("desktop-error", error = format!("{}: {e}", p.display()));
        std::fs::create_dir_all(&macos_dir).map_err(|e| err(&macos_dir, e))?;
        let plist = contents.join("Info.plist");
        std::fs::write(&plist, macos_info_plist(env!("CARGO_PKG_VERSION"))).map_err(|e| err(&plist, e))?;
        let launcher = macos_dir.join("idetea");
        let quoted = exe.to_string_lossy().replace('\'', r"'\''");
        let script = format!("#!/bin/sh\nexec '{quoted}' --menu \"$@\"\n");
        std::fs::write(&launcher, script).map_err(|e| err(&launcher, e))?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).map_err(|e| err(&launcher, e))?;
        // Assinatura ad-hoc para o Apple Silicon aceitar o bundle `.app` sem erro de "aplicativo danificado".
        let _ = std::process::Command::new("codesign")
            .args(["--force", "--deep", "--sign", "-"])
            .arg(&app)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        Ok(fl!("desktop-installed", desktop = app.display().to_string(), exe = exe.display().to_string()))
    }
    #[cfg(not(target_os = "macos"))]
    {
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
}

fn uninstall() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let home = tea_core::os::home_dir().ok_or_else(|| fl!("desktop-no-home"))?;
        let app = home.join("Applications/IDE TEA.app");
        match std::fs::remove_dir_all(&app) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(fl!("desktop-error", error = format!("{}: {e}", app.display()))),
        }
        Ok(fl!("desktop-removed"))
    }
    #[cfg(not(target_os = "macos"))]
    {
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
}

/// `Info.plist` para o bundle `IDE TEA.app` no macOS.
#[cfg(any(target_os = "macos", test))]
pub fn macos_info_plist(version: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>IDE TEA</string>
  <key>CFBundleDisplayName</key><string>IDE TEA</string>
  <key>CFBundleIdentifier</key><string>dev.idetea.app</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleExecutable</key><string>idetea</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"#
    )
}

/// O menu e os ícones atualizam sozinhos na maioria dos desktops; estes comandos só aceleram (se existirem).
#[cfg(not(target_os = "macos"))]
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
#[cfg(any(not(target_os = "macos"), test))]
fn desktop_entry(exe: &Path) -> String {
    DESKTOP
        .replace("\r\n", "\n")
        .replace("@EXEC@", &exec_arg(&exe.to_string_lossy()))
}

/// Caminho no `Exec` do `.desktop`: entre aspas se tiver algo além de letras, números e `/._-+`; dentro das
/// aspas, `"`, `` ` ``, `$` e `\` levam `\` (e o `\` dobra de novo, porque o valor do arquivo também escapa).
#[cfg(any(not(target_os = "macos"), test))]
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
        let plist = macos_info_plist("0.4.0");
        assert!(plist.contains("<string>dev.idetea.app</string>") && plist.contains("<string>0.4.0</string>"));
    }
}
