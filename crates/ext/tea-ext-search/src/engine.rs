//! Motor da busca no projeto, sem UI: percorre a pasta respeitando `.gitignore`
//! e devolve as linhas que combinam.
//!
//! Escopo (uma pasta ou um arquivo), globs de incluir/excluir e linhas de contexto entram aqui.
//! O caminho quente — projeto inteiro, sem glob e sem contexto — continua o mesmo passeio.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ignore::WalkState;
use ignore::overrides::OverrideBuilder;
use regex::Regex;

/// Limite de resultados para a UI continuar leve.
pub const MAX_MATCHES: usize = 2_000;
/// Arquivos maiores que isso são pulados (provavelmente gerados).
pub const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// Linhas longas são cortadas na exibição.
const MAX_LINE_CHARS: usize = 240;
/// Teto das linhas de contexto, para um config absurdo não guardar o arquivo inteiro.
const MAX_CONTEXT: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub case_sensitive: bool,
    pub regex: bool,
    pub whole_word: bool,
    /// Linhas antes e depois de cada resultado. 0 = nenhuma.
    pub context_lines: usize,
}

/// Onde buscar, além das opções do texto. Caminho vazio = o projeto inteiro.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    /// Pasta ou arquivo, absoluto ou relativo à raiz. Fora da raiz, a busca não acha nada.
    pub path: Option<PathBuf>,
    /// Globs para ficar (separados por vírgula). Vazio = todos os arquivos.
    pub include: String,
    /// Globs para pular (separados por vírgula), somados ao `.gitignore`.
    pub exclude: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLine {
    /// Começa em 1.
    pub line: usize,
    /// Sem o recuo, cortada se for longa (reticências no fim quando couber).
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMatch {
    /// Começa em 1.
    pub line: usize,
    /// Coluna do primeiro trecho, em caracteres, começando em 1 (na linha original).
    pub column: usize,
    /// Trecho visível, já sem o recuo e sem as reticências.
    pub text: String,
    /// Intervalos `[início, fim)` em caracteres de `text`: cada trecho que combinou e coube na janela.
    pub spans: Vec<(usize, usize)>,
    /// A janela não começa no início da linha.
    pub prefix_ellipsis: bool,
    /// A janela não vai até o fim da linha.
    pub suffix_ellipsis: bool,
    pub before: Vec<ContextLine>,
    pub after: Vec<ContextLine>,
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

/// Como a linha visível fica depois da troca, e onde o texto novo entra (em caracteres).
pub fn preview_replacement(text: &str, re: &Regex, replacement: &str, expand: bool) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(text.len());
    let mut spans = Vec::new();
    let mut last = 0;
    for m in re.find_iter(text) {
        if m.is_empty() {
            continue;
        }
        out.push_str(&text[last..m.start()]);
        let rep = if expand { re.replace(m.as_str(), replacement).into_owned() } else { replacement.to_string() };
        let start = out.chars().count();
        out.push_str(&rep);
        let end = out.chars().count();
        if start < end {
            spans.push((start, end));
        }
        last = m.end();
    }
    out.push_str(&text[last..]);
    (out, spans)
}

/// Busca nos arquivos de `root` (ou só no [`Scope`]), em paralelo. Para cedo se `cancel` virar `true`.
/// Erro = glob inválido (a busca nem começa).
pub fn search(root: &Path, matcher: &Regex, scope: &Scope, opts: &Options, cancel: &Arc<AtomicBool>) -> Result<Results, String> {
    if cancel.load(Ordering::Relaxed) {
        return Ok(Results { cancelled: true, ..Results::default() });
    }
    let context = opts.context_lines.min(MAX_CONTEXT);
    let overrides = build_overrides(root, scope)?;
    let target = match resolve_scope(root, scope.path.as_deref()) {
        ScopeTarget::Missing => return Ok(Results::default()),
        ScopeTarget::File(path) => {
            if path.metadata().map_or(true, |m| m.len() > MAX_FILE_BYTES) {
                return Ok(Results::default());
            }
            if !overrides.is_empty() && overrides.matched(&path, false).is_ignore() {
                return Ok(Results::default());
            }
            let total = AtomicUsize::new(0);
            let found = search_file(&path, matcher, &total, context);
            let total = total.into_inner();
            return Ok(Results {
                files: found.into_iter().collect(),
                total: total.min(MAX_MATCHES),
                files_scanned: 1,
                truncated: total >= MAX_MATCHES,
                cancelled: cancel.load(Ordering::Relaxed),
            });
        }
        ScopeTarget::Dir(path) => path,
    };

    let files = Mutex::new(Vec::new());
    let total = AtomicUsize::new(0);
    let scanned = AtomicUsize::new(0);
    let mut walk = ignore::WalkBuilder::new(&target);
    walk.hidden(false) // mostra dotfiles (.github, .idetea...), mas...
        .parents(true)
        .filter_entry(|e| e.file_name() != ".git"); // ...nunca entra no .git
    if !overrides.is_empty() {
        walk.overrides(overrides);
    }
    walk.build_parallel().run(|| {
        Box::new(|entry| {
            if cancel.load(Ordering::Relaxed) || total.load(Ordering::Relaxed) >= MAX_MATCHES {
                return WalkState::Quit;
            }
            let Ok(entry) = entry else { return WalkState::Continue };
            if !entry.file_type().is_some_and(|t| t.is_file()) || entry.metadata().map_or(true, |m| m.len() > MAX_FILE_BYTES) {
                return WalkState::Continue;
            }
            scanned.fetch_add(1, Ordering::Relaxed);
            if let Some(found) = search_file(entry.path(), matcher, &total, context) {
                files.lock().unwrap_or_else(|e| e.into_inner()).push(found);
            }
            WalkState::Continue
        })
    });

    let mut files = files.into_inner().unwrap_or_else(|e| e.into_inner());
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let total = total.into_inner();
    Ok(Results {
        total: total.min(MAX_MATCHES),
        truncated: total >= MAX_MATCHES,
        files_scanned: scanned.into_inner(),
        cancelled: cancel.load(Ordering::Relaxed),
        files,
    })
}

enum ScopeTarget {
    Dir(PathBuf),
    File(PathBuf),
    Missing,
}

fn resolve_scope(root: &Path, scope: Option<&Path>) -> ScopeTarget {
    let Some(path) = scope else { return ScopeTarget::Dir(root.to_path_buf()) };
    let path = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
    if !path.starts_with(root) {
        return ScopeTarget::Missing;
    }
    if path.is_file() {
        ScopeTarget::File(path)
    } else if path.is_dir() {
        ScopeTarget::Dir(path)
    } else {
        ScopeTarget::Missing
    }
}

/// Globs no formato do `ignore`: sem `!` inclui (e, se houver algum, o resto fica de fora);
/// com `!` exclui. Vírgula separa, menos dentro de `{a,b}`.
fn build_overrides(root: &Path, scope: &Scope) -> Result<ignore::overrides::Override, String> {
    let includes = split_globs(&scope.include);
    let excludes = split_globs(&scope.exclude);
    if includes.is_empty() && excludes.is_empty() {
        return Ok(ignore::overrides::Override::empty());
    }
    let mut builder = OverrideBuilder::new(root);
    for pat in includes {
        let pat = pat.trim_start_matches('!');
        if pat.is_empty() {
            continue;
        }
        builder.add(pat).map_err(|e| e.to_string())?;
    }
    for pat in excludes {
        let pat = pat.trim_start_matches('!');
        if pat.is_empty() {
            continue;
        }
        builder.add(&format!("!{pat}")).map_err(|e| e.to_string())?;
    }
    builder.build().map_err(|e| e.to_string())
}

/// Separa por vírgula, sem partir uma lista `{a,b}`.
pub fn split_globs(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (i, c) in input.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let piece = input[start..i].trim();
                if !piece.is_empty() {
                    out.push(piece);
                }
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    let piece = input[start..].trim();
    if !piece.is_empty() {
        out.push(piece);
    }
    out
}

fn search_file(path: &Path, matcher: &Regex, total: &AtomicUsize, context: usize) -> Option<FileMatches> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return None;
    }
    // UTF-8 (quase todos) sem copiar; senão, como o editor lê (RFC 0021): BOM, UTF-16, Windows-1252. Binário
    // (byte zero sem BOM de UTF-16) fica de fora.
    let text: std::borrow::Cow<'_, str> = match std::str::from_utf8(&bytes) {
        Ok(t) if !t.as_bytes().iter().take(8192).any(|&b| b == 0) => t.into(),
        Ok(_) => return None,
        Err(_) => tea_core::encoding::decode(&bytes, None, tea_core::encoding::Encoding::for_label(b"windows-1252")?).ok()?.0.into(),
    };
    // Atalho: a maioria dos arquivos não tem nada; evita quebrar em linhas à toa.
    if !matcher.is_match(&text) {
        return None;
    }
    let mut matches = Vec::new();
    let mut prev: VecDeque<ContextLine> = VecDeque::new();
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        if context > 0
            && let Some(start) = matches.iter().position(|m: &LineMatch| m.after.len() < context)
        {
            let ctx = context_line(line_no, line);
            for m in &mut matches[start..] {
                if m.after.len() < context {
                    m.after.push(ctx.clone());
                }
            }
        }
        if let Some(spans) = match_spans(line, matcher) {
            if total.fetch_add(1, Ordering::Relaxed) >= MAX_MATCHES {
                break;
            }
            let mut found = make_match(line_no, line, &spans);
            if context > 0 {
                found.before = prev.iter().cloned().collect();
            }
            matches.push(found);
        }
        if context > 0 {
            if prev.len() == context {
                prev.pop_front();
            }
            prev.push_back(context_line(line_no, line));
        }
    }
    (!matches.is_empty()).then(|| FileMatches { path: path.to_path_buf(), matches })
}

fn match_spans(line: &str, matcher: &Regex) -> Option<Vec<(usize, usize)>> {
    let spans: Vec<(usize, usize)> = matcher.find_iter(line).filter(|m| !m.is_empty()).map(|m| (m.start(), m.end())).collect();
    (!spans.is_empty()).then_some(spans)
}

fn context_line(line: usize, text: &str) -> ContextLine {
    let trimmed = text.trim_start();
    let (shown, cut) = take_prefix(trimmed, MAX_LINE_CHARS);
    let mut text = shown.to_string();
    if cut {
        text.push('…');
    }
    ContextLine { line, text }
}

fn make_match(line: usize, text: &str, byte_spans: &[(usize, usize)]) -> LineMatch {
    let column = text[..byte_spans[0].0].chars().count() + 1;
    let lead = text.len() - text.trim_start().len();
    let trimmed = &text[lead..];
    let spans: Vec<(usize, usize)> = byte_spans
        .iter()
        .filter_map(|&(start, end)| {
            if end <= lead {
                None
            } else {
                Some((start.max(lead) - lead, end.saturating_sub(lead)))
            }
        })
        .collect();
    let (shown, spans, prefix_ellipsis, suffix_ellipsis) = layout_line(trimmed, &spans);
    LineMatch { line, column, text: shown, spans, prefix_ellipsis, suffix_ellipsis, before: Vec::new(), after: Vec::new() }
}

/// Corta `trimmed` em no máximo [`MAX_LINE_CHARS`] caracteres. Se o primeiro trecho começa depois
/// desse corte, a janela fica centrada nele (e encostada no fim da linha, se não houver espaço).
fn layout_line(trimmed: &str, byte_spans: &[(usize, usize)]) -> (String, Vec<(usize, usize)>, bool, bool) {
    let Some(&(first, first_end)) = byte_spans.first() else {
        let (body, cut) = take_prefix(trimmed, MAX_LINE_CHARS);
        return (body.to_string(), Vec::new(), false, cut);
    };
    let first_char = trimmed[..first].chars().count();
    if first_char < MAX_LINE_CHARS {
        let (start_byte, end_byte, _) = skip_take(trimmed, 0, MAX_LINE_CHARS);
        let slice = &trimmed[start_byte..end_byte];
        return (slice.to_string(), clip_spans(slice, byte_spans, start_byte), false, end_byte < trimmed.len());
    }
    let match_chars = trimmed[first..first_end].chars().count().min(MAX_LINE_CHARS);
    let pad = (MAX_LINE_CHARS - match_chars) / 2;
    let mut start_char = first_char.saturating_sub(pad);
    let (mut start_byte, mut end_byte, got) = skip_take(trimmed, start_char, MAX_LINE_CHARS);
    if got < MAX_LINE_CHARS && start_char > 0 {
        start_char -= (MAX_LINE_CHARS - got).min(start_char);
        (start_byte, end_byte, _) = skip_take(trimmed, start_char, MAX_LINE_CHARS);
    }
    let slice = &trimmed[start_byte..end_byte];
    (slice.to_string(), clip_spans(slice, byte_spans, start_byte), start_char > 0, end_byte < trimmed.len())
}

fn clip_spans(slice: &str, byte_spans: &[(usize, usize)], start_byte: usize) -> Vec<(usize, usize)> {
    let end_byte = start_byte + slice.len();
    let mut out = Vec::new();
    for &(start, end) in byte_spans {
        let start = start.max(start_byte);
        let end = end.min(end_byte);
        if start >= end {
            continue;
        }
        let from = start - start_byte;
        let to = end - start_byte;
        let cs = slice[..from].chars().count();
        let ce = slice[..to].chars().count();
        if cs < ce {
            out.push((cs, ce));
        }
    }
    out
}

fn take_prefix(text: &str, max_chars: usize) -> (&str, bool) {
    let (start, end, taken) = skip_take(text, 0, max_chars);
    (&text[start..end], taken == max_chars && end < text.len())
}

/// `(byte inicial, byte final, caracteres lidos)` pulando `skip` caracteres e lendo até `take`.
fn skip_take(text: &str, skip: usize, take: usize) -> (usize, usize, usize) {
    let mut seen = 0;
    let mut start_byte = text.len();
    let mut end_byte = text.len();
    let mut taken = 0;
    let mut started = false;
    for (i, ch) in text.char_indices() {
        if !started {
            if seen == skip {
                start_byte = i;
                started = true;
            } else {
                seen += 1;
                continue;
            }
        }
        if taken == take {
            end_byte = i;
            return (start_byte, end_byte, taken);
        }
        taken += 1;
        end_byte = i + ch.len_utf8();
    }
    if !started {
        start_byte = text.len();
        end_byte = text.len();
    }
    (start_byte, end_byte, taken)
}

/// Últimas buscas: a mais recente primeiro, sem repetir, no máximo `limit`.
pub fn push_history(history: &mut Vec<String>, query: &str, limit: usize) {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return;
    }
    history.retain(|item| item != query);
    history.insert(0, query.to_string());
    history.truncate(limit);
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
        run_scoped(root, q, opts, Scope::default())
    }

    fn run_scoped(root: &Path, q: &str, opts: Options, scope: Scope) -> Results {
        let m = build_matcher(q, &opts).unwrap();
        search(root, &m, &scope, &opts, &Arc::new(AtomicBool::new(false))).unwrap()
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

    /// RFC 0021: um Latin-1 e um UTF-16 com BOM são lidos como o editor lê; um binário continua de fora.
    #[test]
    fn finds_text_in_legacy_encodings() {
        let root = project("encodings");
        std::fs::write(root.join("src/latin1.txt"), b"uma a\xe7\xe3o aqui\n").unwrap();
        let utf16: Vec<u8> = [0xFF, 0xFE].into_iter().chain("outra ação\n".encode_utf16().flat_map(u16::to_le_bytes)).collect();
        std::fs::write(root.join("src/utf16.txt"), utf16).unwrap();
        let r = run(&root, "ação", Options::default());
        assert_eq!(found(&r, &root), ["src/latin1.txt:1:5", "src/utf16.txt:1:7"]);
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
        let r = search(&root, &m, &Scope::default(), &Options::default(), &Arc::new(AtomicBool::new(true))).unwrap();
        assert!(r.cancelled);
        assert_eq!(r.total, 0);
    }

    #[test]
    fn include_and_exclude_globs() {
        let root = project("globs");
        std::fs::create_dir_all(root.join("src/keep")).unwrap();
        std::fs::write(root.join("src/keep/ok.rs"), "NEEDLE here\n").unwrap();
        std::fs::write(root.join("src/keep/ok.test.rs"), "NEEDLE test\n").unwrap();
        std::fs::write(root.join("src/skip.txt"), "NEEDLE txt\n").unwrap();
        std::fs::write(root.join("notes.txt"), "NEEDLE root\n").unwrap();

        let only_keep = run_scoped(
            &root,
            "NEEDLE",
            Options::default(),
            Scope { include: "src/keep/**, *.test.rs".into(), ..Scope::default() },
        );
        assert_eq!(found(&only_keep, &root), ["src/keep/ok.rs:1:1", "src/keep/ok.test.rs:1:1"]);

        let no_tests = run_scoped(
            &root,
            "NEEDLE",
            Options::default(),
            Scope { include: "src/**".into(), exclude: "*.test.rs, *.txt".into(), ..Scope::default() },
        );
        assert_eq!(found(&no_tests, &root), ["src/keep/ok.rs:1:1"]);

        let braced = split_globs("*.{rs,txt}, src/**");
        assert_eq!(braced, ["*.{rs,txt}", "src/**"]);
        let m = build_matcher("NEEDLE", &Options::default()).unwrap();
        let err = search(&root, &m, &Scope { include: "[".into(), ..Scope::default() }, &Options::default(), &Arc::new(AtomicBool::new(false)));
        assert!(err.is_err(), "glob aberto é erro");
    }

    #[test]
    fn folder_and_file_scope() {
        let root = project("scope");
        std::fs::create_dir_all(root.join("src/keep")).unwrap();
        std::fs::write(root.join("src/keep/only.rs"), "SCOPE here\n").unwrap();
        std::fs::write(root.join("src/b.rs"), "SCOPE in b\n").unwrap();
        let folder = run_scoped(&root, "SCOPE", Options::default(), Scope { path: Some(root.join("src/keep")), ..Scope::default() });
        assert_eq!(found(&folder, &root), ["src/keep/only.rs:1:1"]);
        let file = run_scoped(&root, "SCOPE", Options::default(), Scope { path: Some(root.join("src/b.rs")), ..Scope::default() });
        assert_eq!(found(&file, &root), ["src/b.rs:1:1"]);
        // Fora do projeto: não vaza para o resto do disco.
        let outside = run_scoped(&root, "SCOPE", Options::default(), Scope { path: Some(PathBuf::from("/tmp")), ..Scope::default() });
        assert_eq!(outside.total, 0);
        // A pasta ainda respeita o .gitignore do pai.
        std::fs::create_dir_all(root.join("src/target")).unwrap();
        std::fs::write(root.join("target/hidden.rs"), "SCOPE hidden\n").unwrap();
        let src = run_scoped(&root, "SCOPE", Options::default(), Scope { path: Some(root.join("src")), ..Scope::default() });
        assert!(found(&src, &root).iter().all(|p| !p.contains("target")), "{:?}", found(&src, &root));
    }

    #[test]
    fn highlights_every_span_and_centers_past_the_cut() {
        let (text, spans, prefix, suffix) = layout_line("foo bar foo", &[(0, 3), (8, 11)]);
        assert_eq!(text, "foo bar foo");
        assert_eq!(spans, vec![(0, 3), (8, 11)]);
        assert!(!prefix && !suffix);

        let long = format!("{}TARGET{}", "a".repeat(400), "b".repeat(400));
        let at = 400;
        let (text, spans, prefix, suffix) = layout_line(&long, &[(at, at + "TARGET".len())]);
        assert!(prefix && suffix);
        assert_eq!(text.chars().count(), MAX_LINE_CHARS);
        let (s, e) = spans[0];
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars[s..e].iter().collect::<String>(), "TARGET");
        // Centrada: o trecho fica longe das duas pontas da janela.
        assert!(s > 40 && e + 40 < chars.len(), "span em {s}..{e} de {}", chars.len());

        // O começo do trecho cabe no corte inicial: a janela continua sendo o prefixo, e o trecho é cortado.
        let wide = format!("xx{}", "M".repeat(300));
        let (text, spans, prefix, suffix) = layout_line(&wide, &[(2, 2 + 300)]);
        assert!(!prefix && suffix);
        assert_eq!(text.chars().count(), MAX_LINE_CHARS);
        assert_eq!(spans, vec![(2, MAX_LINE_CHARS)]);
    }

    #[test]
    fn context_lines_around_each_match() {
        let root = project("context");
        std::fs::write(root.join("src/c.txt"), "a\nb\nMATCH\nc\nd\nMATCH2\ne\n").unwrap();
        let r = run(&root, "MATCH", Options { context_lines: 1, ..Default::default() });
        let file = r.files.iter().find(|f| f.path.ends_with("c.txt")).unwrap();
        assert_eq!(file.matches.len(), 2);
        assert_eq!(file.matches[0].before.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["b"]);
        assert_eq!(file.matches[0].after.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["c"]);
        assert_eq!(file.matches[1].before.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["d"]);
        assert_eq!(file.matches[1].after.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["e"]);
        assert_eq!(file.matches[0].line, 3);
        assert!(file.matches[0].spans.iter().any(|&(s, e)| file.matches[0].text[s..e].contains("MATCH") || {
            let chars: Vec<char> = file.matches[0].text.chars().collect();
            chars[s..e].iter().collect::<String>().contains("MATCH")
        }));
    }

    #[test]
    fn history_is_recent_first_and_capped() {
        let mut history = Vec::new();
        push_history(&mut history, "foo", 20);
        push_history(&mut history, "  foo  ", 20);
        push_history(&mut history, "bar", 20);
        assert_eq!(history, ["bar", "foo"]);
        for i in 0..25 {
            push_history(&mut history, &format!("q{i}"), 20);
        }
        assert_eq!(history.len(), 20);
        assert_eq!(history[0], "q24");
        assert!(!history.iter().any(|q| q == "foo" || q == "bar"));
    }

    #[test]
    fn replacement_preview_marks_the_new_span() {
        let re = build_matcher("foo", &Options::default()).unwrap();
        let (text, spans) = preview_replacement("foo and FOO", &re, "x", false);
        assert_eq!(text, "x and x");
        assert_eq!(spans, vec![(0, 1), (6, 7)]);
        let re = build_matcher(r"(\w+)=(\w+)", &Options { regex: true, ..Default::default() }).unwrap();
        let (text, spans) = preview_replacement("a=b", &re, "$2=$1", true);
        assert_eq!(text, "b=a");
        assert_eq!(spans, vec![(0, 3)]);
    }
}
