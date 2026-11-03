//! Um worktree do git por agente (RFC 0005, etapa 5): o pedido de terminal com `"worktree": "<branch>"` roda numa
//! cópia própria do repositório, em `.idetea/worktrees/<nome>`, para dois agentes não mexerem nos mesmos arquivos.
//! O branch fica sempre; o worktree sai quando o pedido é apagado, se não tiver nada por commitar.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const DIR: &str = ".idetea/worktrees";

/// Onde fica o worktree do pedido `name`.
pub fn path(root: &Path, name: &str) -> PathBuf {
    root.join(DIR).join(name)
}

/// Um nome de branch que vale: letras, dígitos, `-`, `_`, `.` e `/`, sem os casos que o git recusa.
pub fn valid_branch(branch: &str) -> bool {
    !branch.is_empty()
        && branch.len() <= 100
        && branch.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/'))
        && !branch.starts_with(['-', '.', '/'])
        && !branch.ends_with(['.', '/'])
        && !branch.ends_with(".lock")
        && !branch.contains("..")
        && !branch.contains("//")
        && !branch.contains("/.")
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!("git {}: {}", args.first().copied().unwrap_or(""), String::from_utf8_lossy(&out.stderr).trim()))
    }
}

/// Cria o worktree (um branch novo a partir do HEAD, ou o branch que já existe) ou reusa o que já está lá com o
/// mesmo branch. Devolve a pasta.
pub fn ensure(root: &Path, name: &str, branch: &str) -> Result<PathBuf, String> {
    if !valid_branch(branch) {
        return Err(format!("worktree: nome de branch que não vale: {branch}"));
    }
    let dir = path(root, name);
    if dir.join(".git").exists() {
        let current = git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        if current != branch {
            return Err(format!("worktree: {} já existe com o branch {current}", dir.display()));
        }
        return Ok(dir);
    }
    let base = root.join(DIR);
    std::fs::create_dir_all(&base).map_err(|e| format!("{}: {e}", base.display()))?;
    let ignore = base.join(".gitignore");
    if !ignore.exists() {
        std::fs::write(&ignore, "*\n").map_err(|e| e.to_string())?;
    }
    let target = dir.to_string_lossy().into_owned();
    let exists = git(root, &["rev-parse", "--verify", "--quiet", &format!("refs/heads/{branch}")]).is_ok();
    if exists {
        git(root, &["worktree", "add", "--quiet", &target, branch])?;
    } else {
        git(root, &["worktree", "add", "--quiet", "-b", branch, &target, "HEAD"])?;
    }
    Ok(dir)
}

/// Tira o worktree do pedido, se existir. Sem `--force`: com mudanças por commitar (ou arquivos novos), o git
/// recusa e o worktree fica. `Ok(true)` = saiu.
pub fn remove(root: &Path, name: &str) -> Result<bool, String> {
    let dir = path(root, name);
    if !dir.join(".git").exists() {
        return Ok(false);
    }
    git(root, &["worktree", "remove", &dir.to_string_lossy()]).map(|_| true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_names() {
        for ok in ["revisao", "agente/revisao-1", "fix_2.0"] {
            assert!(valid_branch(ok), "{ok}");
        }
        for bad in ["", "-x", ".x", "/x", "x/", "x.", "a..b", "a//b", "a/.b", "x.lock", "a b", "a;b", "a~1", "a^", "a:b", "a@{1}"] {
            assert!(!valid_branch(bad), "{bad}");
        }
    }

    #[test]
    fn creates_reuses_and_removes_only_when_clean() {
        let root = std::env::temp_dir().join(format!("idetea-worktree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let run = |args: &[&str]| git(&root, args).unwrap();
        run(&["init", "-q"]);
        run(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "início"]);

        let dir = ensure(&root, "ajudante", "agente/ajudante").unwrap();
        assert!(dir.join(".git").exists());
        assert_eq!(git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap(), "agente/ajudante");
        assert_eq!(std::fs::read_to_string(root.join(DIR).join(".gitignore")).unwrap(), "*\n");
        assert_eq!(run(&["status", "--porcelain"]), "", "o projeto não vê o worktree");
        // De novo, com o mesmo branch: reusa. Com outro: recusa.
        assert_eq!(ensure(&root, "ajudante", "agente/ajudante").unwrap(), dir);
        assert!(ensure(&root, "ajudante", "outro").is_err());

        // Com um arquivo novo, não sai; limpo, sai, e o branch fica.
        std::fs::write(dir.join("novo.txt"), "x").unwrap();
        assert!(remove(&root, "ajudante").is_err());
        assert!(dir.exists());
        std::fs::remove_file(dir.join("novo.txt")).unwrap();
        assert_eq!(remove(&root, "ajudante"), Ok(true));
        assert!(!dir.exists());
        assert!(run(&["branch", "--list", "agente/ajudante"]).contains("agente/ajudante"));
        assert_eq!(remove(&root, "ajudante"), Ok(false));

        // O branch que já existe volta num worktree novo.
        assert!(ensure(&root, "ajudante", "agente/ajudante").is_ok());
        assert!(ensure(&root, "x", "a b").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
