//! Integração com o sistema operacional: pastas do usuário, busca no `PATH`, abertura de arquivos/URLs e
//! verificação de processos vivos (Linux, macOS e Windows).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Pasta pessoal do usuário (`HOME` no Unix/macOS; `HOME` ou `USERPROFILE` no Windows).
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var_os("USERPROFILE").filter(|v| !v.is_empty()))
        .map(PathBuf::from)
}

/// Pasta base de configuração (`$XDG_CONFIG_HOME`, `%APPDATA%` no Windows ou `~/.config`).
pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("APPDATA").filter(|v| !v.is_empty()).map(PathBuf::from))
        .or_else(|| home_dir().map(|h| h.join(".config")))
}

/// Pasta base de estado local (`$XDG_STATE_HOME`, `%LOCALAPPDATA%` no Windows ou `~/.local/state`).
pub fn state_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_STATE_HOME").map(PathBuf::from).filter(|p| p.is_absolute()) {
        return Some(xdg);
    }
    #[cfg(windows)]
    if let Some(local) = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
    {
        return Some(local);
    }
    home_dir().map(|h| h.join(".local").join("state"))
}

/// Pasta base de dados do usuário (`$XDG_DATA_HOME`, `%LOCALAPPDATA%` no Windows ou `~/.local/share`).
pub fn data_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).filter(|p| p.is_absolute()) {
        return Some(xdg);
    }
    #[cfg(windows)]
    if let Some(local) = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
    {
        return Some(local);
    }
    home_dir().map(|h| h.join(".local").join("share"))
}

/// Pasta base de cache (`$XDG_CACHE_HOME`, `%LOCALAPPDATA%` no Windows ou `~/.cache`).
pub fn cache_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from).filter(|p| p.is_absolute()) {
        return Some(xdg);
    }
    #[cfg(windows)]
    if let Some(local) = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty()).map(PathBuf::from) {
        return Some(local);
    }
    home_dir().map(|h| h.join(".cache"))
}

/// Confere se `path` é um arquivo executável.
pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// Pastas comuns de ferramentas de desenvolvimento (`~/.cargo/bin`, `~/.local/bin`, o `bin` do MoonBit em
/// `$MOON_HOME` ou `~/.moon` e, no macOS, Homebrew) que podem faltar no `PATH` quando a IDE é aberta pelo Finder/Dock ou pelo menu de aplicativos.
pub fn extra_tool_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = home_dir() {
        dirs.push(home.join(".cargo").join("bin"));
        dirs.push(home.join(".local").join("bin"));
    }
    // O instalador do MoonBit põe `moon` em `$MOON_HOME/bin` (padrão `~/.moon/bin`) e só ajusta o rc do shell.
    match std::env::var_os("MOON_HOME").map(PathBuf::from).filter(|p| p.is_absolute()) {
        Some(moon_home) => dirs.push(moon_home.join("bin")),
        None => dirs.extend(home_dir().map(|h| h.join(".moon").join("bin"))),
    }
    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
        dirs.push(PathBuf::from("/opt/homebrew/sbin"));
        dirs.push(PathBuf::from("/usr/local/bin"));
    }
    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

/// Garante que as pastas de ferramentas do usuário (`~/.cargo/bin`, `~/.local/bin`, `~/.moon/bin`, Homebrew no macOS)
/// estejam no `PATH` do processo. Chame no início do `main()`, antes de criar threads.
pub fn ensure_gui_path() {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::env::split_paths(&current).collect();
    let mut changed = false;
    for dir in extra_tool_dirs() {
        if !paths.iter().any(|p| p == &dir) {
            paths.push(dir);
            changed = true;
        }
    }
    if changed && let Ok(joined) = std::env::join_paths(paths) {
        // SAFETY: chamado no início de `main()` antes de qualquer outra thread existir.
        unsafe { std::env::set_var("PATH", joined) };
    }
}

/// Procura um comando no `PATH` (no Windows também tenta as extensões de `PATHEXT`, como `.exe`, `.cmd`, `.bat`).
pub fn find_in_path(name: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        if Path::new(name).extension().is_none() {
            let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into());
            for ext in exts.split(';').filter(|e| !e.is_empty()) {
                let with_ext = dir.join(format!("{name}{ext}"));
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }
    }
    None
}

/// Verifica se o processo com `pid` ainda está vivo no sistema.
pub fn process_alive(pid: u32) -> bool {
    if pid == std::process::id() {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        let proc = Path::new("/proc");
        !proc.is_dir() || proc.join(pid.to_string()).exists()
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let Ok(p) = i32::try_from(pid) else { return false };
        if p <= 0 {
            return false;
        }
        // SAFETY: `kill(pid, 0)` não envia sinal algum; apenas verifica se o processo existe.
        unsafe { libc::kill(p, 0) == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) }
    }
    #[cfg(windows)]
    {
        let filter = format!("PID eq {pid}");
        let mut cmd = std::process::Command::new("tasklist");
        hide_console(&mut cmd);
        match cmd
            .args(["/FI", &filter, "/NH"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).contains(&pid.to_string()),
            Err(_) => true,
        }
    }
}

/// No Windows, evita que um comando de segundo plano (`git`, servidor LSP, `gdb`, `tasklist`) abra
/// uma janela rápida de console (`conhost.exe`) quando a IDE está rodando sem terminal.
pub fn hide_console(cmd: &mut std::process::Command) -> &mut std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Abre a pasta no gerenciador de arquivos do sistema (sem esperar ele fechar).
pub fn open_in_file_manager(dir: &Path) -> std::io::Result<()> {
    open_with_system(dir.as_os_str(), false)
}

/// Abre um endereço (`https://...`, `mailto:...`) no programa padrão do sistema.
pub fn open_url(url: &str) -> std::io::Result<()> {
    open_with_system(OsStr::new(url), true)
}

/// Abre um arquivo no programa padrão do sistema para o tipo dele.
pub fn open_path(path: &Path) -> std::io::Result<()> {
    open_with_system(path.as_os_str(), false)
}

fn open_with_system(target: &OsStr, is_url: bool) -> std::io::Result<()> {
    let mut cmd = if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg(target);
        c
    } else if cfg!(target_os = "windows") {
        if is_url {
            let mut c = std::process::Command::new("rundll32");
            c.arg("url.dll,FileProtocolHandler").arg(target);
            c
        } else {
            let mut c = std::process::Command::new("explorer");
            c.arg(target);
            c
        }
    } else {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(target);
        c
    };
    let mut child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    // Espera numa thread para o processo não virar zumbi.
    std::thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_is_alive_and_bogus_pid_is_dead() {
        assert!(process_alive(std::process::id()));
        assert!(!process_alive(999_999_999));
    }

    #[test]
    fn user_dirs_return_some_when_home_exists() {
        if home_dir().is_some() {
            assert!(config_dir().is_some());
            assert!(state_dir().is_some());
            assert!(data_dir().is_some());
            assert!(cache_dir().is_some());
        }
    }
}
