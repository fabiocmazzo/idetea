//! Posições e cópia do texto do documento.
//!
//! A IDE conta colunas em caracteres (`char`). O LSP conta em UTF-16 por padrão.
//! Pedimos UTF-32 (= caracteres) ao servidor; se ele não aceitar, convertemos
//! usando uma cópia do texto mantida em sincronia com o editor.

use std::path::Path;

/// Posição em caracteres, começando em 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Como o servidor conta colunas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Utf16,
    Utf32,
}

/// Cópia do texto de um documento, linha a linha.
#[derive(Debug, Clone, Default)]
pub struct DocText {
    lines: Vec<String>,
}

impl DocText {
    pub fn new(text: &str) -> Self {
        // `split('\n')` mantém a última linha vazia depois de um `\n` final, como o LSP espera.
        let lines = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_string()).collect();
        Self { lines }
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        std::fs::read_to_string(path).ok().map(|t| Self::new(&t))
    }

    pub fn line(&self, i: u32) -> Option<&str> {
        self.lines.get(i as usize).map(String::as_str)
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Aplica uma troca com posições em caracteres.
    pub fn apply(&mut self, range: Range, new_text: &str) {
        let start = self.clamp(range.start);
        let end = self.clamp(range.end).max(start);
        let first = &self.lines[start.line as usize];
        let last = &self.lines[end.line as usize];
        let prefix: String = first.chars().take(start.character as usize).collect();
        let suffix: String = last.chars().skip(end.character as usize).collect();
        let joined = format!("{prefix}{new_text}{suffix}");
        let replacement: Vec<String> =
            joined.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_string()).collect();
        self.lines.splice(start.line as usize..=end.line as usize, replacement);
    }

    fn clamp(&self, p: Position) -> Position {
        if self.lines.is_empty() {
            return Position::default();
        }
        let line = (p.line as usize).min(self.lines.len() - 1);
        let len = self.lines[line].chars().count() as u32;
        Position::new(line as u32, p.character.min(len))
    }

    /// Caracteres → unidade do servidor.
    pub fn to_server(&self, p: Position, enc: Encoding) -> Position {
        match enc {
            Encoding::Utf32 => p,
            Encoding::Utf16 => {
                let col = self
                    .line(p.line)
                    .map(|l| l.chars().take(p.character as usize).map(|c| c.len_utf16() as u32).sum())
                    .unwrap_or(p.character);
                Position::new(p.line, col)
            }
        }
    }

    /// Unidade do servidor → caracteres.
    pub fn from_server(&self, p: Position, enc: Encoding) -> Position {
        match enc {
            Encoding::Utf32 => p,
            Encoding::Utf16 => {
                let Some(line) = self.line(p.line) else { return p };
                let mut units = 0;
                let mut chars = 0;
                for c in line.chars() {
                    if units >= p.character {
                        break;
                    }
                    units += c.len_utf16() as u32;
                    chars += 1;
                }
                Position::new(p.line, chars)
            }
        }
    }

    pub fn range_to_server(&self, r: Range, enc: Encoding) -> Range {
        Range { start: self.to_server(r.start, enc), end: self.to_server(r.end, enc) }
    }

    pub fn range_from_server(&self, r: Range, enc: Encoding) -> Range {
        Range { start: self.from_server(r.start, enc), end: self.from_server(r.end, enc) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(l1: u32, c1: u32, l2: u32, c2: u32) -> Range {
        Range { start: Position::new(l1, c1), end: Position::new(l2, c2) }
    }

    #[test]
    fn apply_insert_delete_and_multiline() {
        let mut d = DocText::new("abc\ndef\n");
        d.apply(r(0, 1, 0, 1), "X");
        assert_eq!(d.text(), "aXbc\ndef\n");
        d.apply(r(0, 2, 1, 1), "");
        assert_eq!(d.text(), "aXef\n");
        d.apply(r(0, 4, 0, 4), "\nnova\n");
        assert_eq!(d.text(), "aXef\nnova\n\n");
    }

    #[test]
    fn apply_clamps_out_of_range() {
        let mut d = DocText::new("ab");
        d.apply(r(9, 9, 9, 9), "!");
        assert_eq!(d.text(), "ab!");
    }

    #[test]
    fn utf16_round_trip() {
        // "😀" ocupa 2 unidades UTF-16; "ç" ocupa 1.
        let d = DocText::new("ç😀x");
        let enc = Encoding::Utf16;
        assert_eq!(d.to_server(Position::new(0, 2), enc), Position::new(0, 3));
        assert_eq!(d.from_server(Position::new(0, 3), enc), Position::new(0, 2));
        assert_eq!(d.to_server(Position::new(0, 2), Encoding::Utf32), Position::new(0, 2));
    }

    #[test]
    fn crlf_lines_are_normalized() {
        let d = DocText::new("a\r\nb");
        assert_eq!(d.line(0), Some("a"));
        assert_eq!(d.text(), "a\nb");
    }
}
