//! O que o `Ctrl+clique` num link do terminal faz (RFC 0025): de qual pasta um caminho relativo parte e para onde
//! cada link vai (a IDE, o navegador ou a área de transferência).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced_term::links::{Link, PathResolver};

/// Acha o arquivo (ou a pasta) que o texto `written` nomeia: absoluto ou com `~/` como está; relativo, a partir da
/// pasta atual do shell, da pasta onde a aba abriu e da raiz, nessa ordem. Só o que existe.
pub fn resolve(written: &str, shell_cwd: Option<&Path>, tab_dir: &Path, root: &Path) -> Option<PathBuf> {
    let path = match written.strip_prefix("~/").or((written == "~").then_some("")) {
        Some(rest) => tea_core::os::home_dir()?.join(rest),
        None => PathBuf::from(written),
    };
    if path.is_absolute() {
        return path.exists().then(|| clean(&path));
    }
    [shell_cwd, Some(tab_dir), Some(root)]
        .into_iter()
        .flatten()
        .map(|base| base.join(&path))
        .find(|p| p.exists())
        .map(|p| clean(&p))
}

/// Sem `./` e `..`, sem resolver links simbólicos (o caminho que o usuário reconhece).
fn clean(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// O resolvedor de um terminal que abriu em `tab_dir`, num projeto com raiz `root`.
pub fn resolver(tab_dir: PathBuf, root: PathBuf) -> PathResolver {
    PathResolver(Arc::new(move |written, pid| {
        let shell_cwd = pid.and_then(tea_core::os::process_cwd);
        resolve(written, shell_cwd.as_deref(), &tab_dir, &root)
    }))
}

/// O que fazer com um link clicado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Follow {
    /// Um arquivo: abre numa aba (linha e coluna começam em 1; 0 é "sem posição").
    Open { path: PathBuf, line: usize, column: usize },
    /// Uma pasta do projeto: mostra na árvore.
    Reveal(PathBuf),
    /// `http(s)` e `mailto`: o programa do sistema.
    System(String),
    /// Os outros esquemas (`ssh:`, `magnet:`…): não abrem, o endereço vai para a área de transferência.
    Copy(String),
    /// Uma pasta fora do projeto: nada.
    Nothing,
}

pub fn follow(link: Link, root: &Path) -> Follow {
    match link {
        Link::Url(url) => {
            let lower = url.to_ascii_lowercase();
            if ["http://", "https://", "mailto:"].iter().any(|s| lower.starts_with(s)) {
                Follow::System(url)
            } else {
                Follow::Copy(url)
            }
        }
        Link::Path { path, .. } if path.is_dir() => {
            if path.starts_with(root) { Follow::Reveal(path) } else { Follow::Nothing }
        }
        Link::Path { path, line, column } => Follow::Open { path, line: line.unwrap_or(0), column: column.unwrap_or(0) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirs(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!("idetea-links-{name}-{}", std::process::id()));
        let root = base.join("proj");
        let sub = root.join("crates/app");
        let tab = root.join(".idetea/worktrees/fix");
        for d in [&sub, &tab] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(root.join("README.md"), "").unwrap();
        std::fs::write(sub.join("main.rs"), "").unwrap();
        std::fs::write(tab.join("only-here.rs"), "").unwrap();
        (root, sub, tab)
    }

    #[test]
    fn relative_paths_try_the_shell_the_tab_then_the_root() {
        let (root, sub, tab) = dirs("resolve");
        assert_eq!(resolve("main.rs", Some(&sub), &tab, &root), Some(sub.join("main.rs")), "depois de um cd");
        assert_eq!(resolve("main.rs", None, &tab, &root), None, "sem a pasta do shell, não acha");
        assert_eq!(resolve("only-here.rs", Some(&sub), &tab, &root), Some(tab.join("only-here.rs")), "a pasta da aba");
        assert_eq!(resolve("README.md", Some(&sub), &tab, &root), Some(root.join("README.md")), "a raiz");
        assert_eq!(resolve("./crates/app/../app/main.rs", None, &root, &root), Some(sub.join("main.rs")));
        assert_eq!(resolve("nao-existe.rs", Some(&sub), &tab, &root), None);
        let abs = sub.join("main.rs");
        assert_eq!(resolve(abs.to_str().unwrap(), None, &tab, &root), Some(abs));
        let _ = std::fs::remove_dir_all(root.parent().unwrap());
    }

    #[test]
    fn what_each_link_does() {
        let (root, sub, _) = dirs("follow");
        let url = |u: &str| follow(Link::Url(u.into()), &root);
        assert_eq!(url("https://github.com/x"), Follow::System("https://github.com/x".into()));
        assert_eq!(url("mailto:a@b.c"), Follow::System("mailto:a@b.c".into()));
        assert_eq!(url("ssh:git@host"), Follow::Copy("ssh:git@host".into()));
        assert_eq!(url("magnet:?xt=1"), Follow::Copy("magnet:?xt=1".into()));
        let file = sub.join("main.rs");
        assert_eq!(
            follow(Link::Path { path: file.clone(), line: Some(4), column: None }, &root),
            Follow::Open { path: file, line: 4, column: 0 }
        );
        assert_eq!(follow(Link::Path { path: sub.clone(), line: None, column: None }, &root), Follow::Reveal(sub));
        let outside = std::env::temp_dir();
        assert_eq!(follow(Link::Path { path: outside, line: None, column: None }, &root), Follow::Nothing);
        let _ = std::fs::remove_dir_all(root.parent().unwrap());
    }
}
