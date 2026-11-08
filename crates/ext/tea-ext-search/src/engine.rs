//! Motor da busca no projeto, sem UI: percorre a pasta respeitando `.gitignore`
//! e devolve as linhas que combinam.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ignore::WalkState;
use regex::Regex;

/// Limite de resultados para a UI continuar leve.
pub const MAX_MATCHES: usize = 2_000;
/// Arquivos maiores que isso são pulados (provavelmente gerados).
pub const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// Linhas longas são cortadas na exibição.
const MAX_LINE_CHARS: usize = 240;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub case_sensitive: bool,
    pub regex: bool,
    pub whole_word: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMatch {
    /// Começa em 1.
    pub line: usize,
    /// Coluna do primeiro trecho, em caracteres, começando em 1.
    pub column: usize,
    /// Texto da linha (sem espaços à esquerda, cortado se for longo).
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMatches {
    pub path: PathBuf,
    pub matches: Vec<LineMatch>,
}

#[derive(Debug, Clone, Default)]
pub struct Results {
    pub files: Vec<FileMatches>,
    pub total: usize,
    pub files_scanned: usize,
    pub truncated: bool,
    pub cancelled: bool,
}

/// A regex da busca em texto (com `(?i)` se não diferencia maiúsculas): é o que vai para o app ao substituir.
pub fn pattern(query: &str, opts: &Options) -> String {
    let pattern = if opts.regex { query.to_string() } else { regex::escape(query) };
    let pattern = if opts.whole_word { format!(r"\b(?:{pattern})\b") } else { pattern };
    if opts.case_sensitive { pattern } else { format!("(?i){pattern}") }
}

/// Monta a regex da busca. Erro = regex inválida (mensagem para o usuário).
pub fn build_matcher(query: &str, opts: &Options) -> Result<Regex, String> {
    tea_core::replace::build(&pattern(query, opts))
        .map_err(|e| match e {
            regex::Error::Syntax(s) => s.lines().last().unwrap_or("invalid regex").trim().to_string(),
            other => other.to_string(),
        })
}

/// Busca em todos os arquivos de `root`, em paralelo. Para cedo se `cancel` virar `true`.
pub fn search(root: &Path, matcher: &Regex, cancel: &Arc<AtomicBool>) -> Results {
    let files = Mutex::new(Vec::new());
    let total = AtomicUsize::new(0);
    let scanned = AtomicUsize::new(0);

    ignore::WalkBuilder::new(root)
        .hidden(false) // mostra dotfiles (.github, .idetea...), mas...
        .filter_entry(|e| e.file_name() != ".git") // ...nunca entra no .git
        .build_parallel()
        .run(|| {
            Box::new(|entry| {
                if cancel.load(Ordering::Relaxed) || total.load(Ordering::Relaxed) >= MAX_MATCHES {
                    return WalkState::Quit;
                }
                let Ok(entry) = entry else { return WalkState::Continue };
                if !entry.file_type().is_some_and(|t| t.is_file())
                    || entry.metadata().map_or(true, |m| m.len() > MAX_FILE_BYTES)
                {
                    return WalkState::Continue;
                }
                scanned.fetch_add(1, Ordering::Relaxed);
                if let Some(found) = search_file(entry.path(), matcher, &total) {
                    files.lock().unwrap_or_else(|e| e.into_inner()).push(found);
                }
                WalkState::Continue
            })
        });

    let mut files = files.into_inner().unwrap_or_else(|e| e.into_inner());
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let total = total.into_inner();
    Results {
        total: total.min(MAX_MATCHES),
        truncated: total >= MAX_MATCHES,
        files_scanned: scanned.into_inner(),
        cancelled: cancel.load(Ordering::Relaxed),
        files,
    }
}

fn search_file(path: &Path, matcher: &Regex, total: &AtomicUsize) -> Option<FileMatches> {
    let bytes = std::fs::read(path).ok()?;
    // Binário: tem byte zero no começo.
    if bytes.iter().take(8192).any(|&b| b == 0) {
        return None;
    }
    // Atalho: a maioria dos arquivos não tem nada; evita quebrar em linhas à toa.
    let text = String::from_utf8_lossy(&bytes);
    if !matcher.is_match(&text) {
        return None;
    }
    let mut matches = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if let Some(m) = matcher.find(line) {
            if total.fetch_add(1, Ordering::Relaxed) >= MAX_MATCHES {
                break;
            }
            matches.push(make_match(i + 1, line, m.start()));
        }
    }
    (!matches.is_empty()).then(|| FileMatches { path: path.to_path_buf(), matches })
}

fn make_match(line: usize, text: &str, byte_start: usize) -> LineMatch {
    let column = text[..byte_start].chars().count() + 1;
    let trimmed = text.trim_start();
    let shown: String = trimmed.chars().take(MAX_LINE_CHARS).collect();
    let shown = if trimmed.chars().count() > MAX_LINE_CHARS { format!("{shown}…") } else { shown };
    LineMatch { line, column, text: shown }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tea-search-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("target")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        // `ignore` só respeita .gitignore dentro de um repositório git.
        std::fs::write(dir.join(".gitignore"), "target/\n").unwrap();
        std::fs::write(dir.join("src/a.rs"), "fn main() {\n    let açaí = Foo::new();\n}\n").unwrap();
        std::fs::write(dir.join("src/b.rs"), "struct Foo;\nimpl Foo {}\n// foobar\n").unwrap();
        std::fs::write(dir.join("target/gerado.rs"), "Foo Foo Foo\n").unwrap();
        std::fs::write(dir.join(".git/config"), "Foo\n").unwrap();
        std::fs::write(dir.join("bin.dat"), b"Foo\0\x01\x02").unwrap();
        dir
    }

    fn run(root: &Path, q: &str, opts: Options) -> Results {
        let m = build_matcher(q, &opts).unwrap();
        search(root, &m, &Arc::new(AtomicBool::new(false)))
    }

    fn found(r: &Results, root: &Path) -> Vec<String> {
        r.files
            .iter()
            .flat_map(|f| {
                f.matches.iter().map(move |m| {
                    format!(
                        "{}:{}:{}",
                        f.path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"),
                        m.line,
                        m.column
                    )
                })
            })
            .collect()
    }

    #[test]
    fn respects_gitignore_skips_git_and_binaries() {
        let root = project("ignore");
        let r = run(&root, "Foo", Options { case_sensitive: true, ..Default::default() });
        assert_eq!(found(&r, &root), ["src/a.rs:2:16", "src/b.rs:1:8", "src/b.rs:2:6"]);
    }

    #[test]
    fn case_insensitive_and_whole_word() {
        let root = project("case");
        let r = run(&root, "foo", Options::default());
        assert_eq!(r.total, 4, "inclui 'foobar'");
        let r = run(&root, "foo", Options { whole_word: true, ..Default::default() });
        assert_eq!(r.total, 3, "'foobar' não é palavra inteira");
    }

    #[test]
    fn columns_count_chars_not_bytes() {
        let root = project("utf8");
        let r = run(&root, "Foo::new", Options::default());
        // "    let açaí = " tem 15 caracteres (mas 17 bytes).
        assert_eq!(found(&r, &root), ["src/a.rs:2:16"]);
    }

    #[test]
    fn regex_mode_and_errors() {
        let root = project("regex");
        let r = run(&root, r"impl\s+\w+", Options { regex: true, ..Default::default() });
        assert_eq!(found(&r, &root), ["src/b.rs:2:1"]);
        // Sem modo regex, caracteres especiais são literais.
        assert_eq!(run(&root, "Foo::new()", Options::default()).total, 1);
        assert!(build_matcher("(", &Options { regex: true, ..Default::default() }).is_err());
    }

    #[test]
    fn cancel_stops_early() {
        let root = project("cancel");
        let m = build_matcher("Foo", &Options::default()).unwrap();
        let r = search(&root, &m, &Arc::new(AtomicBool::new(true)));
        assert!(r.cancelled);
        assert_eq!(r.total, 0);
    }
}
