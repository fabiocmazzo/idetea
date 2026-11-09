//! Sintaxes de cor além das que vêm no editor (as do `two-face`): as embutidas aqui e as do usuário.
//!
//! - Embutida: ZX, do [Ziex](https://ziex.dev) (Zig com marcação), para `.zx` e `.mdzx`. Gerada por
//!   `tools/zx-syntax.sh` a partir da gramática do VS Code do Ziex (MIT, ver `assets/NOTICE.md`). As sintaxes
//!   registradas formam um conjunto à parte no editor e não enxergam as dele, então a do Zig que o ZX inclui vem
//!   junto, escondida e sem extensões (os `.zig` continuam com a do editor).
//! - Language packs: Hare e MoonBit trazem suas gramáticas em `assets/languages` (origem e licença no README de cada
//!   pacote e em `assets/NOTICE.md`).
//! - Do usuário: os `*.sublime-syntax` de `~/.config/idetea/syntaxes/`, lidos ao abrir a IDE. Uma sintaxe do
//!   usuário para uma extensão que já tem cor fica no lugar da embutida.
//!
//! Quem registra no editor é o `tea-editor` (`syntaxes::register`), antes da primeira aba.

use std::path::{Path, PathBuf};

/// A sintaxe do ZX (Ziex).
pub const ZX: &str = include_str!("../assets/syntaxes/zx.sublime-syntax");
/// A do Zig, só para o ZX incluir.
pub const ZIG: &str = include_str!("../assets/syntaxes/zig.sublime-syntax");

/// Uma sintaxe para registrar: de onde veio (para os avisos) e o texto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub label: String,
    pub text: String,
}

/// Pasta das sintaxes do usuário (ao lado do config global).
pub fn user_dir() -> Option<PathBuf> {
    Some(crate::config::global_config_path()?.parent()?.join("syntaxes"))
}

/// As embutidas e, depois, as do usuário (em ordem de nome), com os arquivos que não deu para ler.
pub fn sources(user_dir: Option<&Path>) -> (Vec<Source>, Vec<(String, String)>) {
    let mut out = vec![
        Source { label: "zig.sublime-syntax".into(), text: ZIG.into() },
        Source { label: "zx.sublime-syntax".into(), text: ZX.into() },
    ];
    let (packs, pack_errors) = crate::languages::builtins();
    for pack in packs {
        out.extend(pack.syntaxes);
    }
    let mut errors: Vec<(String, String)> = pack_errors.into_iter().map(|e| ("language packs".into(), e)).collect();
    let Some(dir) = user_dir else { return (out, errors) };
    let Ok(entries) = std::fs::read_dir(dir) else { return (out, errors) };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sublime-syntax") && p.is_file())
        .collect();
    files.sort();
    for path in files {
        let label = path.display().to_string();
        match std::fs::read_to_string(&path) {
            Ok(text) => out.push(Source { label, text }),
            Err(e) => errors.push((label, e.to_string())),
        }
    }
    (out, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_pack_syntaxes_are_embedded() {
        let (found, errors) = sources(None);
        assert!(errors.is_empty(), "{errors:?}");
        let labels: Vec<_> = found.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, ["zig.sublime-syntax", "zx.sublime-syntax", "hare/hare.sublime-syntax", "moonbit/moonbit.sublime-syntax"]);
    }

    #[test]
    fn embedded_first_then_user_files_in_name_order() {
        let dir = std::env::temp_dir().join(format!("idetea-syntaxes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("b.sublime-syntax"), "b").unwrap();
        std::fs::write(dir.join("a.sublime-syntax"), "a").unwrap();
        std::fs::write(dir.join("leia-me.txt"), "não é sintaxe").unwrap();
        let (found, errors) = sources(Some(&dir));
        let texts: Vec<_> = found.iter().map(|s| s.text.as_str()).collect();
        assert!(texts.starts_with(&[ZIG, ZX]));
        assert_eq!(&texts[texts.len() - 2..], ["a", "b"]);
        assert!(errors.is_empty());
        assert_eq!(sources(Some(&dir.join("nao-existe"))).0.len(), 4);
        assert!(ZIG.contains("hidden: true") && ZIG.contains("file_extensions: []"));
        assert!(ZX.contains("scope: \"source.zx\""));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
