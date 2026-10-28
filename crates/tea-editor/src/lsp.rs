//! Cola entre o widget de edição e o cliente `tea-lsp`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::Point;
use iced_code_editor::{
    LspClient, LspDocument, LspFormattingOptions, LspPosition, LspRange, LspTextChange,
};
use tea_lsp::{Client, Position, Range};

/// Tempo parado com o mouse até pedir o hover.
pub const HOVER_DELAY: Duration = Duration::from_millis(450);

/// Implementa a trait do widget repassando para o nosso cliente.
pub struct Bridge {
    pub client: Client,
    pub path: PathBuf,
}

fn pos(p: LspPosition) -> Position {
    Position::new(p.line, p.character)
}

fn range(r: LspRange) -> Range {
    Range { start: pos(r.start), end: pos(r.end) }
}

pub fn to_widget_range(r: Range) -> LspRange {
    LspRange {
        start: LspPosition { line: r.start.line, character: r.start.character },
        end: LspPosition { line: r.end.line, character: r.end.character },
    }
}

impl LspClient for Bridge {
    fn did_open(&mut self, document: &LspDocument, text: &str) {
        self.client.did_open(&self.path, &document.language_id, document.version, text);
    }

    fn did_change(&mut self, document: &LspDocument, changes: &[LspTextChange]) {
        let changes: Vec<(Range, String)> = changes.iter().map(|c| (range(c.range), c.text.clone())).collect();
        self.client.did_change(&self.path, document.version, &changes);
    }

    fn did_save(&mut self, _document: &LspDocument, text: &str) {
        self.client.did_save(&self.path, text);
    }

    fn did_close(&mut self, _document: &LspDocument) {
        self.client.did_close(&self.path);
    }

    fn request_hover(&mut self, _document: &LspDocument, position: LspPosition) {
        self.client.hover(&self.path, pos(position));
    }

    fn request_completion(&mut self, _document: &LspDocument, position: LspPosition) {
        self.client.completion(&self.path, pos(position));
    }

    fn request_definition(&mut self, _document: &LspDocument, position: LspPosition) {
        self.client.definition(&self.path, pos(position));
    }

    fn request_formatting(&mut self, _document: &LspDocument, options: LspFormattingOptions) {
        self.client.formatting(&self.path, options.tab_size, options.insert_spaces);
    }
}

/// Hover esperando o mouse ficar parado.
pub struct PendingHover {
    pub point: Point,
    pub since: Instant,
}

impl PendingHover {
    pub fn is_due(&self, now: Instant) -> bool {
        now.duration_since(self.since) >= HOVER_DELAY
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Começo (em caracteres) da palavra que termina em `column`.
pub fn word_start(line: &str, column: usize) -> usize {
    let before: Vec<char> = line.chars().take(column).collect();
    let mut start = before.len();
    while start > 0 && is_word_char(before[start - 1]) {
        start -= 1;
    }
    start
}

/// Prefixo digitado para filtrar o autocomplete e se o menu ainda faz sentido.
/// O menu continua aberto com prefixo vazio logo depois de `.` ou `::`.
pub fn completion_prefix(line: &str, column: usize) -> (String, bool) {
    let start = word_start(line, column);
    let prefix: String = line.chars().skip(start).take(column - start.min(column)).collect();
    let before = line.chars().take(start).last();
    let keep = !prefix.is_empty() || matches!(before, Some('.') | Some(':') | Some('>'));
    (prefix, keep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundaries() {
        assert_eq!(word_start("let açaí", 8), 4);
        assert_eq!(word_start("v.pu", 4), 2);
        assert_eq!(word_start("", 0), 0);
        assert_eq!(word_start("abc", 99), 0);
    }

    #[test]
    fn prefix_rules() {
        assert_eq!(completion_prefix("    v.pu", 8), ("pu".to_string(), true));
        assert_eq!(completion_prefix("    v.", 6), (String::new(), true));
        assert_eq!(completion_prefix("Vec::", 5), (String::new(), true));
        assert_eq!(completion_prefix("x = ", 4), (String::new(), false));
    }
}
