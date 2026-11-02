//! Links do Markdown de uma interface e o componente `file`: o que cada um abre.
//!
//! O documento vem de um agente, então só quatro tipos de destino funcionam: a web (`http`, `https`, `mailto`, no
//! navegador), arquivos do projeto (`src/main.rs:42:5`, na IDE), o config da IDE (`idetea:config`,
//! `idetea:config-project`) e um arquivo do projeto no app do sistema (`idetea:open/index.html`), só com as
//! extensões de [`OPEN_EXTENSIONS`]. Qualquer outro só gera um aviso.

use std::path::Path;

use tea_core::Context;

use crate::document::relative_inside;

/// O que `idetea:open/` abre no app do sistema. O `xdg-open` roda um `.desktop` (e o `open` do macOS, um
/// `.app` ou `.command`): por isso uma lista do que é documento ou mídia, e nada que execute.
pub const OPEN_EXTENSIONS: &[&str] = &[
    "html", "htm", "pdf", "png", "jpg", "jpeg", "gif", "webp", "bmp", "wav", "ogg", "oga", "opus", "flac", "mp3", "mp4",
    "webm",
];

/// Para onde um link leva.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    Web(String),
    /// Relativo à raiz do projeto; linha e coluna começam em 1 (0 = não veio).
    File { path: String, line: u64, column: u64 },
    /// O config global (`false`) ou o do projeto (`true`).
    Config { project: bool },
    /// Um arquivo do projeto no app do sistema (relativo à raiz, extensão da lista).
    System(String),
    /// `idetea:open/` com uma extensão fora da lista.
    Blocked(String),
    Ignored,
}

pub fn link(url: &str) -> Link {
    let lower = url.to_ascii_lowercase();
    if ["http://", "https://", "mailto:"].iter().any(|p| lower.starts_with(p)) {
        return Link::Web(url.to_string());
    }
    match lower.as_str() {
        "idetea:config" => return Link::Config { project: false },
        "idetea:config-project" => return Link::Config { project: true },
        _ => {}
    }
    if lower.starts_with("idetea:open/") {
        let path = &url["idetea:open/".len()..];
        if path.is_empty() || path.contains(':') || !relative_inside(path) {
            return Link::Ignored;
        }
        let ext = Path::new(path).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
        return if OPEN_EXTENSIONS.contains(&ext.as_str()) { Link::System(path.to_string()) } else { Link::Blocked(path.to_string()) };
    }
    if url.contains("://") || url.starts_with('#') {
        return Link::Ignored;
    }
    let (path, line, column) = split_position(url);
    // Um `:` que sobrou é de um esquema (`javascript:`, `C:`...): não é arquivo do projeto.
    if !path.contains(':') && relative_inside(path) {
        Link::File { path: path.to_string(), line, column }
    } else {
        Link::Ignored
    }
}

/// `caminho`, `caminho:linha` ou `caminho:linha:coluna`.
fn split_position(url: &str) -> (&str, u64, u64) {
    let number = |s: &str| s.parse::<u64>().ok();
    if let Some((rest, last)) = url.rsplit_once(':')
        && let Some(n) = number(last)
    {
        if let Some((path, prev)) = rest.rsplit_once(':')
            && let Some(line) = number(prev)
        {
            return (path, line, n);
        }
        return (rest, n, 0);
    }
    (url, 0, 0)
}

/// Abre um arquivo do projeto na posição (linha 0: só abre).
pub fn open_file(root: &Path, path: &str, line: u64, column: u64, ctx: &mut Context) {
    let path = root.join(path);
    if line == 0 {
        ctx.open_file(path);
    } else {
        ctx.open_file_at(path, line as usize, column.max(1) as usize);
    }
}

/// O botão "copiar" de um bloco de código: o texto vai para a área de transferência, com um aviso.
pub fn copy<T>(code: String, ctx: &mut Context) -> iced::Task<T> {
    ctx.notify(fl!("ui-copied"));
    iced::clipboard::write(code)
}

/// Segue um link clicado no Markdown de uma interface.
pub fn follow(url: &str, root: &Path, ctx: &mut Context) {
    match link(url) {
        Link::Web(url) => {
            if let Err(e) = tea_core::os::open_url(&url) {
                ctx.notify(fl!("ui-link-error", url = url, error = e.to_string()));
            }
        }
        Link::File { path, line, column } => open_file(root, &path, line, column, ctx),
        Link::Config { project } => ctx.run_command(if project { "config.open_project" } else { "config.open" }),
        Link::System(path) => {
            // Um symlink com nome `.html` poderia apontar para um `.desktop`: só arquivo comum.
            let full = root.join(&path);
            let plain = std::fs::symlink_metadata(&full).is_ok_and(|m| m.file_type().is_file());
            if !plain {
                ctx.notify(fl!("ui-open-not-file", path = path));
            } else if let Err(e) = tea_core::os::open_path(&full) {
                ctx.notify(fl!("ui-link-error", url = url, error = e.to_string()));
            }
        }
        Link::Blocked(path) => ctx.notify(fl!("ui-open-blocked", path = path, list = OPEN_EXTENSIONS.join(", "))),
        Link::Ignored => ctx.notify(fl!("ui-link-ignored", url = url)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, line: u64, column: u64) -> Link {
        Link::File { path: path.into(), line, column }
    }

    #[test]
    fn link_targets() {
        assert_eq!(link("https://a.b/c"), Link::Web("https://a.b/c".into()));
        assert_eq!(link("idetea:config"), Link::Config { project: false });
        assert_eq!(link("idetea:config-project"), Link::Config { project: true });
        assert_eq!(link("src/main.rs"), file("src/main.rs", 0, 0));
        assert_eq!(link("src/main.rs:42"), file("src/main.rs", 42, 0));
        assert_eq!(link("src/main.rs:42:5"), file("src/main.rs", 42, 5));
        assert_eq!(link("idetea:open/index.html"), Link::System("index.html".into()));
        assert_eq!(link("idetea:open/docs/Manual.PDF"), Link::System("docs/Manual.PDF".into()));
        assert_eq!(link("idetea:open/jogo.desktop"), Link::Blocked("jogo.desktop".into()));
        assert_eq!(link("idetea:open/rodar.sh"), Link::Blocked("rodar.sh".into()));
        assert_eq!(link("idetea:open/semextensao"), Link::Blocked("semextensao".into()));
        for bad in ["idetea:open/", "idetea:open/../x.html", "idetea:open//etc/a.html", "idetea:open/C:/a.html"] {
            assert_eq!(link(bad), Link::Ignored, "{bad}");
        }
        for bad in ["../x", "/etc/passwd", "file:///etc/passwd", "javascript:alert(1)", "#topo", "ftp://x/y"] {
            assert_eq!(link(bad), Link::Ignored, "{bad}");
        }
    }
}
