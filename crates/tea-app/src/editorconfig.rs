//! `.editorconfig` (editorconfig.org): recuo, tabs ou espaços e limpeza ao salvar, por arquivo.
//!
//! Lê os `.editorconfig` da pasta do arquivo para cima, até um com `root = true`. O mais perto vence, e
//! dentro de um arquivo a seção de baixo vence. Padrões como `{1..3}` (faixa de números) não são aceitos:
//! a seção é ignorada.

use std::path::Path;

use globset::GlobBuilder;

/// O que o `.editorconfig` diz para um arquivo. `None`: não diz (vale o config da IDE).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Props {
    pub insert_spaces: Option<bool>,
    pub tab_size: Option<u8>,
    pub trim_trailing_whitespace: Option<bool>,
    pub insert_final_newline: Option<bool>,
}

impl Props {
    pub fn for_file(path: &Path) -> Self {
        let mut files = Vec::new();
        let mut dir = path.parent();
        while let Some(d) = dir {
            if let Ok(text) = std::fs::read_to_string(d.join(".editorconfig")) {
                let root = is_root(&text);
                files.push((d.to_path_buf(), text));
                if root {
                    break;
                }
            }
            dir = d.parent();
        }
        let mut props = Props::default();
        // Do mais longe para o mais perto: o mais perto sobrescreve.
        for (dir, text) in files.iter().rev() {
            let Ok(rel) = path.strip_prefix(dir) else { continue };
            props.apply(text, rel);
        }
        props
    }

    /// Aplica as seções de um `.editorconfig` que valem para `rel` (caminho relativo à pasta dele).
    fn apply(&mut self, text: &str, rel: &Path) {
        let rel = rel.to_string_lossy().replace('\\', "/");
        let mut matches = false;
        let (mut indent_size, mut tab_width) = (None, None);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some(glob) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                matches = glob_matches(glob, &rel);
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            if !matches {
                continue;
            }
            let (key, value) = (key.trim().to_lowercase(), value.trim().to_lowercase());
            let flag = || match value.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
            match key.as_str() {
                "indent_style" => {
                    self.insert_spaces = match value.as_str() {
                        "space" => Some(true),
                        "tab" => Some(false),
                        _ => None,
                    }
                }
                // `tab`: o tamanho é o do `tab_width`.
                "indent_size" => indent_size = Some(value.parse::<u8>().ok().filter(|n| *n > 0)),
                "tab_width" => tab_width = value.parse::<u8>().ok().filter(|n| *n > 0),
                "trim_trailing_whitespace" => self.trim_trailing_whitespace = flag(),
                "insert_final_newline" => self.insert_final_newline = flag(),
                _ => {}
            }
        }
        match indent_size {
            Some(Some(n)) => self.tab_size = Some(n.min(16)),
            Some(None) => self.tab_size = tab_width.or(self.tab_size),
            None if tab_width.is_some() && self.tab_size.is_none() => self.tab_size = tab_width,
            None => {}
        }
    }
}

/// `root = true` antes da primeira seção.
fn is_root(text: &str) -> bool {
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            return false;
        }
        if let Some((k, v)) = line.split_once('=')
            && k.trim().eq_ignore_ascii_case("root")
        {
            return v.trim().eq_ignore_ascii_case("true");
        }
    }
    false
}

/// Padrão sem `/` vale em qualquer pasta (`*.rs`); com `/`, é relativo à pasta do `.editorconfig`.
/// `*` não passa de uma pasta para outra; `**` passa (o `globset` só aceita `**` entre barras, então um
/// padrão com `**` vira `*` que passa barras).
fn glob_matches(glob: &str, rel: &str) -> bool {
    let (glob, rel) = if glob.contains('/') { (glob.trim_start_matches('/'), rel) } else {
        // Sem barra: compara só com o nome do arquivo.
        (glob, rel.rsplit('/').next().unwrap_or(rel))
    };
    let deep = glob.contains("**");
    let pattern = if deep { glob.replace("**", "*") } else { glob.to_string() };
    GlobBuilder::new(&pattern)
        .literal_separator(!deep)
        .build()
        .is_ok_and(|g| g.compile_matcher().is_match(rel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_and_later_win() {
        let dir = std::env::temp_dir().join(format!("idetea-ec-{}", std::process::id()));
        let sub = dir.join("proj/web");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(dir.join(".editorconfig"), "[*]\nindent_size = 8\n").unwrap();
        std::fs::write(
            dir.join("proj/.editorconfig"),
            "root = true\n\n[*]\nindent_style = space\nindent_size = 4\ntrim_trailing_whitespace = true\n\
             insert_final_newline = true\n\n[*.md]\ntrim_trailing_whitespace = false\n\n\
             [{Makefile,*.mk}]\nindent_style = tab\nindent_size = tab\ntab_width = 8\n\n[web/**.js]\nindent_size = 2\n",
        )
        .unwrap();
        std::fs::write(sub.join(".editorconfig"), "[*.css]\nindent_size = 3\n").unwrap();

        let p = Props::for_file(&dir.join("proj/src/main.rs"));
        assert_eq!(
            p,
            Props {
                insert_spaces: Some(true),
                tab_size: Some(4),
                trim_trailing_whitespace: Some(true),
                insert_final_newline: Some(true)
            },
            "root = true: o de cima (indent_size = 8) não conta"
        );
        assert_eq!(Props::for_file(&dir.join("proj/README.md")).trim_trailing_whitespace, Some(false));
        let make = Props::for_file(&dir.join("proj/Makefile"));
        assert_eq!((make.insert_spaces, make.tab_size), (Some(false), Some(8)));
        assert_eq!(Props::for_file(&dir.join("proj/web/a/b.js")).tab_size, Some(2), "padrão com / é da pasta");
        assert_eq!(Props::for_file(&dir.join("proj/x/web/b.js")).tab_size, Some(4));
        assert_eq!(Props::for_file(&sub.join("x.css")).tab_size, Some(3), "o mais perto vence");
        assert_eq!(Props::for_file(&dir.join("solto.rs")).tab_size, Some(8));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
