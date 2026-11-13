//! Memória do projeto (RFC 0016): fatos em `docs/memory/` (um por arquivo, com frontmatter e um índice em
//! `docs/memory/README.md`) e decisões de arquitetura em `docs/decisions/` (ADRs numerados, modelo MADR). Aqui fica
//! o que o painel "Memória do projeto" e o "Registrar como decisão" do histórico precisam: ler, avisar, apagar e
//! criar o próximo ADR. A convenção está em `docs/project-memory.md`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MEMORY_DIR: &str = "docs/memory";
pub const DECISIONS_DIR: &str = "docs/decisions";
pub const INDEX: &str = "README.md";
/// Um fato sem mudar há mais que isto ganha o aviso de velho.
pub const STALE_DAYS: i64 = 180;

/// Um fato (`docs/memory/<nome>.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub path: PathBuf,
    /// O `name` do frontmatter, ou o nome do arquivo.
    pub name: String,
    pub description: String,
    /// `constraint`, `convention`, `gotcha`, `reference`... (vazio sem frontmatter).
    pub kind: String,
    /// `AAAA-MM-DD`.
    pub updated: Option<String>,
    pub body: String,
    pub warnings: Vec<Warning>,
}

/// Uma decisão (`docs/decisions/NNNN-titulo.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub path: PathBuf,
    pub number: u32,
    pub title: String,
    pub status: String,
}

/// O que o painel avisa num fato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// `updated` falta ou é de mais de [`STALE_DAYS`] dias atrás (com os dias, quando há data).
    Stale(Option<i64>),
    /// Um caminho entre crases que não existe mais no projeto.
    MissingPath(String),
    /// Algo com cara de credencial (chave da AWS, chave privada, token...).
    Secret,
    /// O índice não tem uma linha para este fato.
    NotInIndex,
}

/// Tudo o que há na memória do projeto. `broken`: arquivos que o índice cita e não existem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    pub facts: Vec<Fact>,
    pub decisions: Vec<Decision>,
    pub broken: Vec<String>,
    pub has_index: bool,
}

impl Memory {
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty() && self.decisions.is_empty() && !self.has_index
    }
}

/// Lê a memória do projeto. `today`: dias desde 1970-01-01 (ver [`today`]), para o aviso de velho.
pub fn load(root: &Path, today: i64) -> Memory {
    let dir = root.join(MEMORY_DIR);
    let index = std::fs::read_to_string(dir.join(INDEX)).ok();
    let linked: Vec<String> = index.as_deref().map(index_links).unwrap_or_default();
    let mut facts: Vec<Fact> = md_files(&dir)
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n != INDEX))
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let file = path.file_name()?.to_string_lossy().into_owned();
            let mut fact = parse_fact(&path, &text);
            fact.warnings = warnings(root, &fact, &text, today);
            if index.is_some() && !linked.contains(&file) {
                fact.warnings.push(Warning::NotInIndex);
            }
            Some(fact)
        })
        .collect();
    facts.sort_by_key(|f| f.name.to_lowercase());
    let broken = linked.into_iter().filter(|l| !dir.join(l).exists()).collect();
    let mut decisions: Vec<Decision> = md_files(&root.join(DECISIONS_DIR))
        .into_iter()
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            parse_decision(&path, &text)
        })
        .collect();
    decisions.sort_by_key(|d| d.number);
    Memory { facts, decisions, broken, has_index: index.is_some() }
}

/// Uma assinatura barata das duas pastas (nomes, tamanhos e datas), para saber quando reler.
pub fn signature(root: &Path) -> Vec<(PathBuf, u64, Option<SystemTime>)> {
    let mut out = Vec::new();
    for dir in [MEMORY_DIR, DECISIONS_DIR] {
        for path in md_files(&root.join(dir)) {
            let meta = std::fs::metadata(&path).ok();
            out.push((path, meta.as_ref().map_or(0, |m| m.len()), meta.and_then(|m| m.modified().ok())));
        }
    }
    out
}

fn md_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")))
        .collect();
    out.sort();
    out
}

/// O frontmatter (`---` … `---`) como pares chave/valor, e o resto do texto.
fn frontmatter(text: &str) -> (Vec<(String, String)>, &str) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) else { return (Vec::new(), text) };
    let Some(end) = rest.find("\n---") else { return (Vec::new(), text) };
    let pairs = rest[..end]
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            // Comentário de YAML no fim do valor (`type: constraint  # …`).
            let v = v.split(" #").next().unwrap_or(v);
            Some((k.trim().to_string(), v.trim().trim_matches('"').trim_matches('\'').to_string()))
        })
        .collect();
    let body = rest[end + 4..].trim_start_matches(['-']).trim_start_matches(['\r', '\n']);
    (pairs, body)
}

fn get<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).filter(|v| !v.is_empty())
}

fn parse_fact(path: &Path, text: &str) -> Fact {
    let (pairs, body) = frontmatter(text);
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    Fact {
        path: path.to_path_buf(),
        name: get(&pairs, "name").unwrap_or(&stem).to_string(),
        description: get(&pairs, "description").unwrap_or_default().to_string(),
        kind: get(&pairs, "type").unwrap_or_default().to_string(),
        updated: get(&pairs, "updated").map(str::to_string),
        body: body.to_string(),
        warnings: Vec::new(),
    }
}

/// `0007-usar-sqlite.md` → 7, o título do primeiro `# ` (sem o `0007.` da frente) e o status (do frontmatter ou de
/// uma linha `Status: …`).
fn parse_decision(path: &Path, text: &str) -> Option<Decision> {
    let file = path.file_name()?.to_string_lossy();
    let digits: String = file.chars().take_while(char::is_ascii_digit).collect();
    let number = digits.parse().ok()?;
    let (pairs, body) = frontmatter(text);
    let title = body
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(|t| t.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start_matches(['.', ':', ' ']).trim().to_string())
        .unwrap_or_else(|| file.trim_end_matches(".md").to_string());
    let status = get(&pairs, "status").map(str::to_string).or_else(|| {
        body.lines().find_map(|l| {
            let l = l.trim().trim_start_matches(['*', '-', ' ']);
            let (k, v) = l.split_once(':')?;
            k.trim_matches('*').trim().eq_ignore_ascii_case("status").then(|| v.trim_matches('*').trim().to_string())
        })
    });
    Some(Decision { path: path.to_path_buf(), number, title, status: status.unwrap_or_default() })
}

/// Os arquivos que o índice cita: `[...](arquivo.md)`, só os relativos à pasta.
fn index_links(index: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = index;
    while let Some(i) = rest.find("](") {
        rest = &rest[i + 2..];
        let Some(end) = rest.find(')') else { break };
        let target = rest[..end].split('#').next().unwrap_or_default().trim();
        let target = target.strip_prefix("./").unwrap_or(target);
        if target.ends_with(".md") && !target.contains("://") && !target.contains('/') && target != INDEX {
            out.push(target.to_string());
        }
        rest = &rest[end..];
    }
    out
}

fn warnings(root: &Path, fact: &Fact, text: &str, today: i64) -> Vec<Warning> {
    let mut out = Vec::new();
    match fact.updated.as_deref().and_then(parse_date) {
        Some(day) if today - day > STALE_DAYS => out.push(Warning::Stale(Some(today - day))),
        Some(_) => {}
        None => out.push(Warning::Stale(None)),
    }
    for path in code_paths(&fact.body) {
        if !root.join(&path).exists() {
            out.push(Warning::MissingPath(path));
        }
    }
    if looks_secret(text) {
        out.push(Warning::Secret);
    }
    out
}

const PATH_EXTENSIONS: &[&str] = &[
    "rs", "md", "toml", "json", "yaml", "yml", "ts", "tsx", "js", "jsx", "py", "go", "c", "h", "cpp", "hpp", "java", "kt",
    "rb", "php", "cs", "swift", "zig", "sh", "txt", "lock", "html", "css", "scss", "sql", "proto", "ftl", "lua", "ex",
];

/// Os caminhos entre crases: com `/`, ou terminando numa extensão conhecida. Sem espaços, globs, URLs ou
/// placeholders; `arquivo.rs:42` vira `arquivo.rs`.
fn code_paths(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (i, span) in body.split('`').enumerate() {
        if i % 2 == 0 || span.is_empty() || span.contains(char::is_whitespace) {
            continue;
        }
        if span.contains("://") || span.contains(['*', '?', '<', '>', '{', '}', '$', '(', ')', '[', ']', '|', '=', ',', '~']) {
            continue;
        }
        let path = span.split(':').next().unwrap_or(span).trim_start_matches("./");
        let path = path.trim_end_matches('/');
        if path.is_empty() || path.starts_with('/') || path.starts_with('-') || path.starts_with("..") {
            continue;
        }
        let ext = Path::new(path).extension().map(|e| e.to_string_lossy().to_lowercase());
        let has_ext = ext.is_some_and(|e| PATH_EXTENSIONS.contains(&e.as_str()));
        // `a/b` parece caminho; `foo.bar` só se `bar` for uma extensão de arquivo.
        if ((path.contains('/') && !path.contains("::")) || has_ext) && !out.iter().any(|p| p == path) {
            out.push(path.to_string());
        }
    }
    out
}

/// Padrões comuns de credencial: chave da AWS, chave privada, tokens do GitHub, da OpenAI/Anthropic, do Slack.
pub fn looks_secret(text: &str) -> bool {
    let token_after = |prefix: &str, min: usize| {
        text.match_indices(prefix).any(|(i, _)| {
            let rest = &text[i + prefix.len()..];
            rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').count() >= min
        })
    };
    text.contains("-----BEGIN") && text.contains("PRIVATE KEY")
        || text.match_indices("AKIA").any(|(i, _)| {
            let rest = &text[i + 4..];
            rest.len() >= 16 && rest[..16].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        })
        || token_after("ghp_", 30)
        || token_after("github_pat_", 30)
        || token_after("sk-", 32)
        || token_after("xoxb-", 20)
        || token_after("xoxp-", 20)
}

/// Apaga um fato e as linhas do índice que apontam para ele.
pub fn delete_fact(root: &Path, path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)?;
    let index = root.join(MEMORY_DIR).join(INDEX);
    let Some(file) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else { return Ok(()) };
    if let Ok(text) = std::fs::read_to_string(&index) {
        let links = [format!("]({file})"), format!("](./{file})")];
        let kept: Vec<&str> = text.lines().filter(|l| !links.iter().any(|k| l.contains(k.as_str()))).collect();
        let mut new = kept.join("\n");
        if text.ends_with('\n') && !kept.is_empty() {
            new.push('\n');
        }
        if new != text {
            std::fs::write(&index, new)?;
        }
    }
    Ok(())
}

/// O próximo número de ADR (1 numa pasta vazia).
pub fn next_decision_number(root: &Path) -> u32 {
    md_files(&root.join(DECISIONS_DIR))
        .iter()
        .filter_map(|p| p.file_name()?.to_string_lossy().chars().take_while(char::is_ascii_digit).collect::<String>().parse::<u32>().ok())
        .max()
        .map_or(1, |n| n + 1)
}

/// Um nome de arquivo a partir do título: minúsculas, sem acento, `-` no lugar do resto (até 60 caracteres).
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars().flat_map(char::to_lowercase) {
        let c = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'ß' => 's',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let mut out: String = out.trim_end_matches('-').chars().take(60).collect();
    out = out.trim_end_matches('-').to_string();
    if out.is_empty() { "decision".into() } else { out }
}

/// Grava `docs/decisions/NNNN-<slug>.md` com o texto (que já começa no frontmatter ou no título) e devolve o
/// caminho. Nunca sobrescreve: com o nome ocupado, usa o número seguinte.
pub fn create_decision(root: &Path, title: &str, text: impl Fn(u32) -> String) -> std::io::Result<PathBuf> {
    let dir = root.join(DECISIONS_DIR);
    std::fs::create_dir_all(&dir)?;
    let mut n = next_decision_number(root);
    loop {
        let path = dir.join(format!("{n:04}-{}.md", slug(title)));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut f) => {
                use std::io::Write;
                f.write_all(text(n).as_bytes())?;
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => return Err(e),
        }
    }
}

/// Hoje, em dias desde 1970-01-01 (UTC).
pub fn today() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| (d.as_secs() / 86_400) as i64)
}

/// `AAAA-MM-DD` de um dia contado desde 1970-01-01.
pub fn date_text(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `AAAA-MM-DD` (ou o começo de uma data e hora) → dias desde 1970-01-01.
pub fn parse_date(s: &str) -> Option<i64> {
    let s = s.get(..10)?;
    let mut it = s.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    ((1..=12).contains(&m) && (1..=31).contains(&d)).then(|| days_from_civil(y, m, d))
}

// Algoritmos de Howard Hinnant (datas do calendário gregoriano ↔ dias).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { yoe + era * 400 + 1 } else { yoe + era * 400 }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("idetea-memory-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(MEMORY_DIR)).unwrap();
        root
    }

    #[test]
    fn dates_round_trip() {
        let d = parse_date("2026-11-13").unwrap();
        assert_eq!(date_text(d), "2026-11-13");
        assert_eq!(parse_date("2024-02-29T10:00:00Z").map(date_text).as_deref(), Some("2024-02-29"));
        assert_eq!(parse_date("1970-01-01"), Some(0));
        assert_eq!(parse_date("ontem"), None);
    }

    #[test]
    fn loads_facts_with_warnings_and_the_index() {
        let root = temp("load");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "").unwrap();
        let mem = root.join(MEMORY_DIR);
        std::fs::write(mem.join(INDEX), "# Memória\n\n- [Testes](testes.md) — offline\n- [Sumiu](sumiu.md) — x\n").unwrap();
        std::fs::write(
            mem.join("testes.md"),
            "---\nname: testes\ndescription: sem rede\ntype: constraint  # comentário\nupdated: 2026-11-01\n---\nVer `src/main.rs:3` e `tests/fixtures/` e `Vec::new` e `foo.bar`.\n",
        )
        .unwrap();
        std::fs::write(mem.join("solto.md"), "sem frontmatter, chave AKIAABCDEFGHIJKLMNOP\n").unwrap();
        let today = parse_date("2026-11-13").unwrap();
        let m = load(&root, today);
        assert_eq!(m.broken, ["sumiu.md"]);
        assert!(m.has_index);
        let names: Vec<&str> = m.facts.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["solto", "testes"]);
        let testes = &m.facts[1];
        assert_eq!((testes.kind.as_str(), testes.description.as_str()), ("constraint", "sem rede"));
        assert_eq!(testes.warnings, [Warning::MissingPath("tests/fixtures".into())]);
        assert_eq!(m.facts[0].warnings, [Warning::Stale(None), Warning::Secret, Warning::NotInIndex]);
        // Velho: mais de 180 dias.
        let later = load(&root, today + 200);
        assert!(matches!(later.facts[1].warnings[0], Warning::Stale(Some(212))));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn decisions_number_title_and_status() {
        let root = temp("adr");
        let dir = root.join(DECISIONS_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("0002-sqlite.md"), "# 0002. Usar SQLite\n\nStatus: accepted\n").unwrap();
        std::fs::write(dir.join("0001-edition.md"), "---\nstatus: aceita\n---\n\n# Usar a edition 2024\n").unwrap();
        std::fs::write(dir.join("notas.md"), "# Sem número\n").unwrap();
        let m = load(&root, 0);
        let got: Vec<(u32, &str, &str)> = m.decisions.iter().map(|d| (d.number, d.title.as_str(), d.status.as_str())).collect();
        assert_eq!(got, [(1, "Usar a edition 2024", "aceita"), (2, "Usar SQLite", "accepted")]);
        assert_eq!(next_decision_number(&root), 3);
        let path = create_decision(&root, "Cache em SQLite, não JSON!", |n| format!("# {n:04}. x\n")).unwrap();
        assert_eq!(path.file_name().unwrap(), "0003-cache-em-sqlite-nao-json.md");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# 0003. x\n");
        assert_eq!(slug("キャッシュ"), "decision");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn deleting_a_fact_drops_its_index_line() {
        let root = temp("del");
        let mem = root.join(MEMORY_DIR);
        std::fs::write(mem.join(INDEX), "# M\n\n- [A](a.md) — a\n- [B](./b.md) — b\n").unwrap();
        std::fs::write(mem.join("b.md"), "x").unwrap();
        delete_fact(&root, &mem.join("b.md")).unwrap();
        assert!(!mem.join("b.md").exists());
        assert_eq!(std::fs::read_to_string(mem.join(INDEX)).unwrap(), "# M\n\n- [A](a.md) — a\n");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn secrets() {
        assert!(looks_secret("-----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(looks_secret("token ghp_abcdefghijklmnopqrstuvwxyz0123456789"));
        assert!(looks_secret("sk-ant-api03-abcdefghijklmnopqrstuvwxyz012345"));
        assert!(!looks_secret("use sk-learn e o AKIA do exemplo"));
    }
}
