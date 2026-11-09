//! No Windows (MSVC), compila o ícone `assets/idetea.ico` como recurso Win32 do `idetea.exe`
//! usando o `rc.exe` do Windows SDK (quando disponível na máquina).

fn main() {
    println!("cargo:rerun-if-changed=../../assets/idetea.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        embed_windows_icon();
    }
}

#[cfg(windows)]
fn embed_windows_icon() {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from) else { return };
    let Some(out_dir) = std::env::var_os("OUT_DIR").map(PathBuf::from) else { return };
    let ico = manifest_dir.join("../../assets/idetea.ico");
    let Ok(ico) = ico.canonicalize() else { return };
    let ico_str = ico.to_string_lossy().trim_start_matches(r"\\?\").replace('\\', "\\\\");

    let rc_file = out_dir.join("idetea.rc");
    let res_file = out_dir.join("idetea.res");
    if std::fs::write(&rc_file, format!("1 ICON \"{ico_str}\"\r\n")).is_err() {
        return;
    }

    let rc_exe = find_rc_exe().unwrap_or_else(|| PathBuf::from("rc.exe"));
    let ok = Command::new(rc_exe)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_file)
        .arg(&rc_file)
        .status()
        .is_ok_and(|s| s.success());
    if ok && res_file.is_file() {
        println!("cargo:rustc-link-arg-bins={}", res_file.display());
    }
}

#[cfg(windows)]
fn find_rc_exe() -> Option<std::path::PathBuf> {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    if Command::new("rc.exe").arg("/?").output().is_ok() {
        return Some(PathBuf::from("rc.exe"));
    }
    let kits = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(kits)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs.reverse();
    dirs.into_iter().map(|d| d.join("x64").join("rc.exe")).find(|p| p.is_file())
}
