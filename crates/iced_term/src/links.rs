//! What can be followed with `Ctrl+click` in a line of terminal text (IDE TEA RFC 0025): URLs and file paths
//! (`src/main.rs:42:5`, rustc's `--> path:l:c`, Python's `File "path", line N`, Node's `at f (path:l:c)`,
//! MSBuild's `path(l,c)`). Pure text functions; whether a path exists is decided by the caller.

use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use regex::Regex;

/// What a link points to, once resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// An address (`https://…`, `mailto:…`, `ssh:…`). What to do with each scheme is the embedder's choice.
    Url(String),
    /// A file or folder that exists, with the line and column when the text had them.
    Path { path: PathBuf, line: Option<usize>, column: Option<usize> },
}

/// A candidate found in the text, before checking the disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    Url(String),
    Path { path: String, line: Option<usize>, column: Option<usize> },
}

/// A candidate and where it is in the line, in characters (`start..end`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub start: usize,
    pub end: usize,
    pub found: Found,
}

/// Turns a path written in the terminal into an existing path on disk (relative to the shell's folder, the tab's,
/// the project's...). `shell_pid` is the process of the terminal's shell, when known.
#[derive(Clone)]
pub struct PathResolver(pub Arc<ResolveFn>);

/// `(written path, shell pid) -> existing path`.
pub type ResolveFn = dyn Fn(&str, Option<u32>) -> Option<PathBuf> + Send + Sync;

impl std::fmt::Debug for PathResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PathResolver")
    }
}

const URL: &str = r#"(?:ipfs:|ipns:|magnet:|mailto:|gemini://|gopher://|https://|http://|news:|file://|git://|ssh:|ftp://)[^\u{0000}-\u{001F}\u{007F}-\u{009F}<>"\s{-}\^⟨⟩`]+"#;

static URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(URL).expect("url regex"));

/// `File "path", line N` (Python tracebacks).
static PYTHON_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"File "(?P<path>[^"]+)", line (?P<line>\d+)"#).expect("python regex"));

/// A path, then `:line[:col]` or `(line,col)`. A path is a run of characters that are not spaces, quotes,
/// brackets or separators; a Windows drive (`C:\`) is allowed at the start.
static PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?P<path>(?:[A-Za-z]:[\\/])?[^\s"'`()<>\[\]{}|,;:]+)(?:(?::(?P<line>\d+)(?::(?P<col>\d+))?)|(?:\((?P<pline>\d+),(?P<pcol>\d+)\)))?"#,
    )
    .expect("path regex")
});

/// An extension: a dot, a letter, then up to 7 letters or digits, at the end.
static EXT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.[A-Za-z][A-Za-z0-9]{0,7}$").expect("ext regex"));

/// Every candidate in the line, left to right, without overlaps (URLs and Python's form win over plain paths).
pub fn candidates(text: &str) -> Vec<Hit> {
    let chars = |byte: usize| text[..byte].chars().count();
    let mut hits: Vec<Hit> = Vec::new();
    let overlaps = |hits: &[Hit], s: usize, e: usize| hits.iter().any(|h| s < h.end && h.start < e);

    for m in URL_RE.find_iter(text) {
        let url = m.as_str().trim_end_matches(['.', ',', ')', ']', '\'', ':', ';']);
        let (s, e) = (chars(m.start()), chars(m.start()) + url.chars().count());
        hits.push(Hit { start: s, end: e, found: Found::Url(url.to_string()) });
    }
    for c in PYTHON_RE.captures_iter(text) {
        let p = c.name("path").expect("path group");
        let (s, e) = (chars(p.start()), chars(p.end()));
        if !overlaps(&hits, s, e) {
            let line = c.name("line").and_then(|l| l.as_str().parse().ok());
            hits.push(Hit { start: s, end: e, found: Found::Path { path: p.as_str().to_string(), line, column: None } });
        }
    }
    for c in PATH_RE.captures_iter(text) {
        let whole = c.get(0).expect("match");
        let p = c.name("path").expect("path group");
        // Punctuation that ends a sentence is not part of the path ("see src/a.rs.").
        let path = p.as_str().trim_end_matches(['.', '\'']);
        if path.is_empty() || !looks_like_path(path) {
            continue;
        }
        let num = |n: &str| c.name(n).and_then(|v| v.as_str().parse::<usize>().ok());
        let (line, column) = match (num("line"), num("pline")) {
            (Some(l), _) => (Some(l), num("col")),
            (None, Some(l)) => (Some(l), num("pcol")),
            _ => (None, None),
        };
        let s = chars(p.start());
        let e = if line.is_some() && path.len() == p.as_str().len() { chars(whole.end()) } else { s + path.chars().count() };
        if !overlaps(&hits, s, e) {
            hits.push(Hit { start: s, end: e, found: Found::Path { path: path.to_string(), line, column } });
        }
    }
    hits.sort_by_key(|h| h.start);
    hits
}

/// The candidate under character `index`, if any.
pub fn at(text: &str, index: usize) -> Option<Hit> {
    candidates(text).into_iter().find(|h| h.start <= index && index < h.end)
}

/// A path has a separator, starts at home (`~`), or ends in an extension: `ok`, `main`, `1.2.3` are not paths.
fn looks_like_path(s: &str) -> bool {
    if s.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return false;
    }
    s.contains('/') || s.contains('\\') || s.starts_with('~') || EXT_RE.is_match(s)
}

/// The path of a `file://` URL (percent-decoded), for opening it like a written path.
pub fn file_url_path(url: &str) -> Option<String> {
    let rest = url.strip_prefix("file://")?;
    // `file://host/path`: only the local host (empty or `localhost`) is a local file.
    let path = match rest.find('/') {
        Some(0) => rest,
        Some(i) if &rest[..i] == "localhost" => &rest[i..],
        _ => return None,
    };
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        let decoded = match (bytes[i], bytes.get(i + 1), bytes.get(i + 2)) {
            (b'%', Some(&h), Some(&l)) => hex(h).zip(hex(l)).map(|(h, l)| (h * 16 + l) as u8),
            _ => None,
        };
        match decoded {
            Some(b) => {
                out.push(b);
                i += 3;
            },
            None => {
                out.push(bytes[i]);
                i += 1;
            },
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(text: &str) -> Vec<Found> {
        candidates(text).into_iter().map(|h| h.found).collect()
    }

    fn path(p: &str, line: Option<usize>, column: Option<usize>) -> Found {
        Found::Path { path: p.into(), line, column }
    }

    #[test]
    fn compilers_and_tools() {
        assert_eq!(found("error[E0308]: mismatched types\n"), vec![]);
        assert_eq!(found("   --> crates/tea-core/src/lib.rs:208:5"), vec![path("crates/tea-core/src/lib.rs", Some(208), Some(5))]);
        assert_eq!(found("src/main.c:12:3: error: expected ';'"), vec![path("src/main.c", Some(12), Some(3))]);
        assert_eq!(found("./build.zig:7:5: error"), vec![path("./build.zig", Some(7), Some(5))]);
        assert_eq!(found("src/App.cs(10,5): error CS1002"), vec![path("src/App.cs", Some(10), Some(5))]);
        assert_eq!(found("    at render (src/App.tsx:8:2)"), vec![path("src/App.tsx", Some(8), Some(2))]);
        assert_eq!(
            found(r#"  File "app/main.py", line 12, in main"#),
            vec![path("app/main.py", Some(12), None)]
        );
        assert_eq!(found("Edited README.md and ~/notes.txt."), vec![path("README.md", None, None), path("~/notes.txt", None, None)]);
        assert_eq!(found(r"C:\work\app\main.rs:3"), vec![path(r"C:\work\app\main.rs", Some(3), None)]);
    }

    #[test]
    fn not_paths() {
        assert_eq!(found("ok main 1.2.3 v0.18.0 3:14"), vec![]);
        assert_eq!(found("Finished `release` profile in 1m 47s"), vec![]);
    }

    #[test]
    fn urls_win_and_lose_trailing_punctuation() {
        assert_eq!(found("see https://github.com/fabiocmazzo/idetea/pull/24."), vec![Found::Url("https://github.com/fabiocmazzo/idetea/pull/24".into())]);
        assert_eq!(found("(http://localhost:3000)"), vec![Found::Url("http://localhost:3000".into())]);
        assert_eq!(found("ssh:git@host"), vec![Found::Url("ssh:git@host".into())]);
    }

    #[test]
    fn at_finds_the_one_under_the_mouse() {
        let line = "a.rs:1 and b/c.rs:2:3";
        assert_eq!(at(line, 0).map(|h| h.found), Some(path("a.rs", Some(1), None)));
        assert_eq!(at(line, 5).map(|h| (h.start, h.end)), Some((0, 6)), "the :1 is part of the link");
        assert_eq!(at(line, 7), None);
        assert_eq!(at(line, 12).map(|h| h.found), Some(path("b/c.rs", Some(2), Some(3))));
        // Characters, not bytes.
        assert_eq!(at("ção → src/ñ.rs:4", 8).map(|h| h.found), Some(path("src/ñ.rs", Some(4), None)));
    }

    #[test]
    fn file_urls() {
        assert_eq!(file_url_path("file:///home/a/My%20File.md").as_deref(), Some("/home/a/My File.md"));
        assert_eq!(file_url_path("file://localhost/etc/hosts").as_deref(), Some("/etc/hosts"));
        assert_eq!(file_url_path("file://otherhost/x"), None);
        assert_eq!(file_url_path("https://x"), None);
    }
}
