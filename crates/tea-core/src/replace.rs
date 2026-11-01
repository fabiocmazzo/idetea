//! Substituir em vários arquivos: o pedido que uma extensão (a Busca) manda para o app, e a troca linha
//! a linha que os dois usam (a extensão para a prévia, o app para aplicar).
//!
//! A troca é por linha, como a busca: a regex nunca pega uma quebra de linha.

use std::borrow::Cow;
use std::path::PathBuf;

use regex::{NoExpand, Regex, RegexBuilder};

/// O que trocar, e onde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replace {
    /// Regex já pronta (com `(?i)` se não diferencia maiúsculas).
    pub pattern: String,
    pub replacement: String,
    /// `$1`, `${nome}` na troca valem (modo regex). Senão o texto vai como está.
    pub expand: bool,
    pub targets: Vec<Target>,
}

/// Um arquivo e, se for só parte dele, as linhas (começam em 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub path: PathBuf,
    /// `None`: o arquivo todo.
    pub lines: Option<Vec<usize>>,
}

impl Replace {
    pub fn regex(&self) -> Result<Regex, regex::Error> {
        build(&self.pattern)
    }
}

pub fn build(pattern: &str) -> Result<Regex, regex::Error> {
    RegexBuilder::new(pattern).size_limit(10 * (1 << 20)).build()
}

/// Troca numa linha (sem o fim de linha). Devolve o texto e quantas trocas.
pub fn replace_line<'a>(line: &'a str, re: &Regex, replacement: &str, expand: bool) -> (Cow<'a, str>, usize) {
    let count = re.find_iter(line).filter(|m| !m.is_empty()).count();
    if count == 0 {
        return (Cow::Borrowed(line), 0);
    }
    let out = if expand { re.replace_all(line, replacement) } else { re.replace_all(line, NoExpand(replacement)) };
    (out, count)
}

/// Troca no texto todo, ou só nas linhas pedidas. Mantém os fins de linha como estão (`\n` ou `\r\n`).
pub fn replace_text(text: &str, re: &Regex, replacement: &str, expand: bool, lines: Option<&[usize]>) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut total = 0;
    for (i, piece) in text.split_inclusive('\n').enumerate() {
        let body = piece.strip_suffix('\n').map_or(piece, |b| b.strip_suffix('\r').unwrap_or(b));
        let end = &piece[body.len()..];
        if lines.is_some_and(|l| !l.contains(&(i + 1))) {
            out.push_str(piece);
            continue;
        }
        let (new, n) = replace_line(body, re, replacement, expand);
        total += n;
        out.push_str(&new);
        out.push_str(end);
    }
    (out, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_line_keeps_line_endings() {
        let re = build("(?i)foo").unwrap();
        let (out, n) = replace_text("Foo a\r\nb foo foo\nfoo", &re, "x", false, None);
        assert_eq!(out, "x a\r\nb x x\nx");
        assert_eq!(n, 4);
        let (out, n) = replace_text("foo\nfoo\nfoo\n", &re, "x", false, Some(&[2]));
        assert_eq!((out.as_str(), n), ("foo\nx\nfoo\n", 1));
    }

    #[test]
    fn expand_only_in_regex_mode() {
        let re = build(r"(\w+)=(\w+)").unwrap();
        assert_eq!(replace_line("a=b", &re, "$2=$1", true).0, "b=a");
        assert_eq!(replace_line("a=b", &re, "$2=$1", false).0, "$2=$1", "fora do modo regex, $ é texto");
    }

    #[test]
    fn never_crosses_lines() {
        let re = build(r"a\s+b").unwrap();
        assert_eq!(replace_text("a\nb", &re, "x", true, None), ("a\nb".to_string(), 0));
        let empty = build("x*").unwrap();
        assert_eq!(replace_text("ab", &empty, "-", true, None).1, 0, "trecho vazio não conta");
    }
}
