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

/// A troca que leva `old` a `new`: um trecho de `old` (em caracteres) e o texto que entra no lugar. Compara as linhas
/// iguais no começo e no fim, e por caractere só o que sobra; várias edições separadas viram um trecho só, que vai da
/// primeira à última. Serve para os diagnósticos acompanharem o texto (ver [`shift_range`]).
pub fn diff(old: &str, new: &str) -> (Range, String) {
    let old: Vec<&str> = old.split('\n').collect();
    let new: Vec<&str> = new.split('\n').collect();
    let top = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let room = old.len().min(new.len()) - top;
    let bottom = old.iter().rev().zip(new.iter().rev()).take(room).take_while(|(a, b)| a == b).count();
    // Cada linha com o seu `\n`, para uma linha inteira inserida ou apagada aparecer como tal.
    let window = |lines: &[&str]| -> Vec<char> { lines.iter().flat_map(|l| l.chars().chain(['\n'])).collect() };
    let a = window(&old[top..old.len() - bottom]);
    let b = window(&new[top..new.len() - bottom]);
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let room = a.len().min(b.len()) - prefix;
    let suffix = a.iter().rev().zip(b.iter().rev()).take(room).take_while(|(x, y)| x == y).count();
    let from = advance(Position::new(top as u32, 0), &a[..prefix]);
    let to = advance(from, &a[prefix..a.len() - suffix]);
    (Range { start: from, end: to }, b[prefix..b.len() - suffix].iter().collect())
}

/// `p` depois de andar pelos caracteres `chars`.
fn advance(p: Position, chars: &[char]) -> Position {
    chars.iter().fold(p, |p, &c| if c == '\n' { Position::new(p.line + 1, 0) } else { Position::new(p.line, p.character + 1) })
}

/// Onde `r` fica depois de o trecho `change` virar `text` (posições em caracteres). Um diagnóstico acompanha o texto
/// editado antes de o servidor mandar a lista nova (o `cargo check` do rust-analyzer só roda ao salvar). Texto
/// inserido logo antes empurra o intervalo; logo depois do fim, não o estica. Um ponto dentro do trecho trocado fica
/// onde está (limitado ao fim do texto novo): o editor manda linhas inteiras reescritas, e às vezes o documento todo,
/// e levar o ponto para a borda do trecho esticaria o sublinhado pela linha ou o jogaria no começo do arquivo.
pub fn shift_range(r: Range, change: Range, text: &str) -> Range {
    let (cs, ce) = (change.start, change.end);
    let lines = text.matches('\n').count() as u32;
    let last = text.rsplit('\n').next().unwrap_or_default().chars().count() as u32;
    let new_end = if lines == 0 { Position::new(cs.line, cs.character + last) } else { Position::new(cs.line + lines, last) };
    let after = |p: Position| {
        if p.line == ce.line {
            Position::new(new_end.line, new_end.character + (p.character - ce.character))
        } else {
            Position::new(p.line - ce.line + new_end.line, p.character)
        }
    };
    let map = |p: Position, is_end: bool| {
        if p < cs || (is_end && p == cs) {
            p
        } else if p >= ce {
            after(p)
        } else {
            p.min(new_end)
        }
    };
    let start = map(r.start, false);
    let end = map(r.end, true).max(start);
    Range { start, end }
}

#[cfg(test)]
mod tests {
    #[test]
    fn diff_finds_the_exact_change() {
        let r = |a: (u32, u32), b: (u32, u32)| Range { start: Position::new(a.0, a.1), end: Position::new(b.0, b.1) };
        let src = "fn main() {\n    let t = soma(2, 3);\n}\n";
        assert_eq!(diff(src, &format!("\n{src}")), (r((0, 0), (0, 0)), "\n".to_string()), "Enter no começo");
        let with_use = format!("use std::collections::HashMap;\n\n{src}");
        assert_eq!(diff(src, &with_use), (r((0, 0), (0, 0)), "use std::collections::HashMap;\n\n".to_string()));
        assert_eq!(diff(src, &src.replace("let t", "let tt")), (r((1, 9), (1, 9)), "t".to_string()), "letra no meio");
        assert_eq!(diff(src, &src.replace("soma", "")), (r((1, 12), (1, 16)), String::new()), "apagar");
        assert_eq!(diff(src, &src.replace("    let t = soma(2, 3);\n", "")), (r((1, 0), (2, 0)), String::new()), "linha apagada");
        let (range, text) = diff(src, src);
        assert!(range.start == range.end && text.is_empty(), "nada mudou: {range:?} {text:?}");
        assert_eq!(diff("aa", "aaa"), (r((0, 2), (0, 2)), "a".to_string()), "o começo comum não invade o fim comum");
        // Duas edições separadas: um trecho só, da primeira à última.
        assert_eq!(diff("a b c", "x b y"), (r((0, 0), (0, 5)), "x b y".to_string()));
    }

    #[test]
    fn diagnostics_follow_edits() {
        let r = |a: (u32, u32), b: (u32, u32)| Range { start: Position::new(a.0, a.1), end: Position::new(b.0, b.1) };
        let at = |l, c| r((l, c), (l, c));
        let soma = r((3, 16), (3, 20));
        // `use` e uma linha em branco inseridos no topo: desce duas linhas.
        assert_eq!(shift_range(soma, at(0, 0), "use std::collections::HashMap;\n\n"), r((5, 16), (5, 20)));
        // Texto antes, na mesma linha: anda para a direita; em outra linha depois: nada muda.
        assert_eq!(shift_range(soma, at(3, 4), "mut "), r((3, 20), (3, 24)));
        assert_eq!(shift_range(soma, at(5, 0), "x"), soma);
        // Digitar logo depois do fim não estica; logo antes do começo empurra.
        assert_eq!(shift_range(soma, at(3, 20), "x"), soma);
        assert_eq!(shift_range(soma, at(3, 16), "x"), r((3, 17), (3, 21)));
        // Linhas apagadas acima sobem; o trecho apagado por dentro encolhe; apagar tudo deixa vazio no lugar.
        assert_eq!(shift_range(soma, r((1, 0), (3, 0)), ""), r((1, 16), (1, 20)));
        assert_eq!(shift_range(soma, r((3, 17), (3, 19)), ""), r((3, 16), (3, 18)));
        assert_eq!(shift_range(soma, r((3, 10), (3, 30)), ""), at(3, 10));
        // Quebra de linha no meio do intervalo: o fim vai para a linha nova.
        assert_eq!(shift_range(soma, at(3, 18), "\n    "), r((3, 16), (4, 6)));
        // Como o editor manda: a linha inteira reescrita com uma letra a mais antes do erro. O sublinhado fica (não
        // estica pela linha).
        let linha = "    let total = soma(2, 3);";
        assert_eq!(shift_range(soma, r((3, 0), (3, 26)), &format!("x{linha}")), soma);
        // O `use` aplicado pela ação, reescrevendo a primeira linha: as de baixo descem duas.
        assert_eq!(shift_range(soma, r((0, 0), (0, 11)), "use std::collections::HashMap;\n\nfn main() {"), r((5, 16), (5, 20)));
        // O documento inteiro trocado: fica onde está, limitado ao texto novo.
        assert_eq!(shift_range(soma, r((0, 0), (8, 1)), "fn main() {}\n\n\nfn soma() {}\n"), soma);
        assert_eq!(shift_range(soma, r((0, 0), (8, 1)), "fn main() {}\n"), at(1, 0));
        // Fim depois do trecho trocado, na mesma linha do fim dele.
        assert_eq!(shift_range(r((2, 0), (3, 8)), r((3, 0), (3, 4)), "ab\nc"), r((2, 0), (4, 5)));
    }

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
