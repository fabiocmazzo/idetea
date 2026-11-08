//! Histórico das interfaces (RFC 0007): a IDE guarda cada pergunta que passou pelo painel.
//!
//! Quem grava é a IDE, não o agente: no clique (o documento daquela revisão e a resposta), quando o agente apaga a
//! resposta ("lida pelo agente") e quando o agente apaga um documento que ninguém respondeu. O protocolo não muda.
//!
//! `.idetea/history/<AAAA-MM>/<AAAAMMDD-HHMMSS>-<nome>-r<revisão>[-<evento>].json`, um arquivo por entrada, fora
//! do git (`.idetea/history/.gitignore` com `*`). Fica até o usuário apagar.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tea_declarative_ui::{Document, Response, timestamp};

use crate::store;

pub const FORMAT: &str = "idetea-ui-history";

/// Saídas do `idetea history`.
pub const EXIT_OK: i32 = 0;
pub const EXIT_INVALID: i32 = 2;

pub fn dir(root: &Path) -> PathBuf {
    root.join(".idetea").join("history")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// O usuário clicou num botão.
    Answered,
    /// O agente apagou o documento sem resposta (ou ele não tinha botões).
    Unanswered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub format: String,
    pub version: u64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub revision: u64,
    pub outcome: Outcome,
    /// Quando a IDE viu esta revisão pela primeira vez.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answered: Option<String>,
    /// Quando o agente apagou a resposta (sem isto: ainda não leu, ou a resposta se perdeu).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_by_agent: Option<String>,
    /// Quando o agente apagou o documento sem resposta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<String>,
    /// O documento daquela revisão, como o agente escreveu.
    pub document: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<Response>,
}

impl Entry {
    fn new(name: &str, doc: &Document, raw: &[u8], shown: Option<String>, outcome: Outcome) -> Self {
        Self {
            format: FORMAT.into(),
            version: 1,
            name: name.into(),
            title: doc.title.clone(),
            source: doc.source.clone(),
            revision: doc.revision,
            outcome,
            shown,
            answered: None,
            read_by_agent: None,
            closed: None,
            document: serde_json::from_slice(raw).unwrap_or(Value::Null),
            response: None,
        }
    }

    pub fn answered(name: &str, doc: &Document, raw: &[u8], shown: Option<String>, response: Response) -> Self {
        let mut e = Self::new(name, doc, raw, shown, Outcome::Answered);
        e.answered = Some(response.time.clone());
        e.response = Some(response);
        e
    }

    pub fn unanswered(name: &str, doc: &Document, raw: &[u8], shown: Option<String>, closed: SystemTime) -> Self {
        let mut e = Self::new(name, doc, raw, shown, Outcome::Unanswered);
        e.closed = Some(timestamp(closed));
        e
    }

    /// A hora que ordena o histórico: a da resposta, ou a de quando foi fechada.
    pub fn time(&self) -> &str {
        self.answered.as_deref().or(self.closed.as_deref()).or(self.shown.as_deref()).unwrap_or("")
    }

    pub fn title(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }

    /// Texto onde a busca procura: título, nome, quem perguntou, botão, valores e comentário.
    pub fn haystack(&self) -> String {
        let mut out = format!("{} {} {}", self.title(), self.name, self.source.as_deref().unwrap_or(""));
        if let Some(r) = &self.response {
            out.push(' ');
            out.push_str(&r.action);
            for (k, v) in &r.values {
                out.push_str(&format!(" {k} {v}"));
            }
            if let Some(c) = &r.comment {
                out.push(' ');
                out.push_str(c);
            }
        }
        out.to_lowercase()
    }
}

fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// Só letras, dígitos, `-` e `_` num pedaço de nome de arquivo.
fn clean(part: &str) -> String {
    part.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect()
}

/// Cria a pasta do mês (e o `.gitignore` do histórico). Recusa symlinks no caminho.
fn month_dir(root: &Path, time: &str) -> std::io::Result<PathBuf> {
    let idetea = root.join(".idetea");
    let history = dir(root);
    let month = history.join(clean(time.get(..7).unwrap_or("0000-00")));
    for d in [&idetea, &history, &month] {
        if is_link(d) {
            return Err(std::io::Error::other(format!("{}: symlink", d.display())));
        }
    }
    std::fs::create_dir_all(&month)?;
    let ignore = history.join(".gitignore");
    if std::fs::symlink_metadata(&ignore).is_err() {
        let _ = std::fs::write(&ignore, "*\n");
    }
    Ok(month)
}

/// Grava pelo temporário (arquivo novo, nunca segue um link) e `rename`.
fn write_atomic(path: &Path, entry: &Entry) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("entrada.json");
    let tmp = dir.join(format!(".{name}.tmp"));
    let _ = std::fs::remove_file(&tmp);
    let text = serde_json::to_string_pretty(entry).map_err(std::io::Error::other)?;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
    file.write_all(text.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    std::fs::rename(&tmp, path)
}

/// Grava uma entrada nova. Devolve o caminho.
pub fn record(root: &Path, entry: &Entry) -> std::io::Result<PathBuf> {
    let time = entry.time().to_string();
    let month = month_dir(root, &time)?;
    let stamp: String = time.chars().filter(char::is_ascii_digit).take(14).collect();
    let stamp = if stamp.len() == 14 { format!("{}-{}", &stamp[..8], &stamp[8..]) } else { "00000000-000000".into() };
    let event = entry.response.as_ref().map(|r| format!("-{}", clean(&r.event))).unwrap_or_default();
    let base = format!("{stamp}-{}-r{}{event}", clean(&entry.name), entry.revision);
    let mut path = month.join(format!("{base}.json"));
    let mut n = 2;
    while std::fs::symlink_metadata(&path).is_ok() {
        path = month.join(format!("{base}-{n}.json"));
        n += 1;
    }
    write_atomic(&path, entry)?;
    Ok(path)
}

/// As pastas dos meses, da mais nova para a mais velha (sem symlinks).
fn months(root: &Path) -> Vec<PathBuf> {
    let history = dir(root);
    if is_link(&root.join(".idetea")) || is_link(&history) {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&history) else { return Vec::new() };
    let mut out: Vec<PathBuf> =
        entries.flatten().filter(|e| e.file_type().is_ok_and(|t| t.is_dir())).map(|e| e.path()).collect();
    out.sort();
    out.reverse();
    out
}

/// Os arquivos de entrada de um mês (sem ocultos, temporários e links).
fn entry_files(month: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(month) else { return Vec::new() };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter(|e| e.file_name().to_str().is_some_and(|n| !n.starts_with('.') && n.ends_with(".json")))
        .map(|e| e.path())
        .collect()
}

fn read(path: &Path) -> Option<Entry> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    // Uma entrada tem o tamanho de um documento (no máximo 1 MiB) mais a resposta.
    if !meta.file_type().is_file() || meta.len() > 4 * 1024 * 1024 {
        return None;
    }
    let entry: Entry = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    (entry.format == FORMAT).then_some(entry)
}

/// O agente apagou a resposta `event`: anota na entrada dela. `false`: nenhuma entrada com esse evento.
pub fn mark_read(root: &Path, event: &str, when: SystemTime) -> bool {
    let suffix = format!("-{}.json", clean(event));
    for month in months(root) {
        let found = entry_files(&month).into_iter().find(|p| p.to_str().is_some_and(|s| s.ends_with(&suffix)));
        if let Some(path) = found {
            if let Some(mut entry) = read(&path)
                && entry.read_by_agent.is_none()
            {
                entry.read_by_agent = Some(timestamp(when));
                let _ = write_atomic(&path, &entry);
            }
            return true;
        }
    }
    false
}

/// Todas as entradas, da mais nova para a mais velha. Entradas versionadas no git ficam de fora (podem ter vindo
/// de um repositório clonado).
pub fn list(root: &Path) -> Vec<(PathBuf, Entry)> {
    let tracked = store::tracked_under(root, ".idetea/history");
    let mut out: Vec<(PathBuf, Entry)> = months(root)
        .iter()
        .flat_map(|m| entry_files(m))
        .filter(|p| {
            let rel = p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/");
            !tracked.contains(&rel)
        })
        .filter_map(|p| read(&p).map(|e| (p, e)))
        .collect();
    out.sort_by(|a, b| b.1.time().cmp(a.1.time()).then_with(|| b.0.cmp(&a.0)));
    out
}

/// Apaga o histórico inteiro (só as entradas e as pastas dos meses que ficarem vazias). Devolve quantas.
pub fn clear(root: &Path) -> std::io::Result<usize> {
    let mut n = 0;
    for month in months(root) {
        for path in entry_files(&month) {
            std::fs::remove_file(&path)?;
            n += 1;
        }
        let _ = std::fs::remove_dir(&month);
    }
    Ok(n)
}

/// `AAAA-MM-DDTHH:MM:SSZ` → segundos desde 1970.
fn epoch(ts: &str) -> Option<i64> {
    let n = |r: std::ops::Range<usize>| ts.get(r)?.parse::<i64>().ok();
    let (y, m, d, hh, mm, ss) = (n(0..4)?, n(5..7)?, n(8..10)?, n(11..13)?, n(14..16)?, n(17..19)?);
    // Dias desde 1970-01-01 (Howard Hinnant, o inverso do de `timestamp`).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Data e hora locais de um `timestamp`: (`AAAA-MM-DD`, `HH:MM`). Sem fuso conhecido, fica a UTC.
pub fn local(ts: &str) -> Option<(String, String)> {
    let secs = epoch(ts)?;
    #[cfg(unix)]
    {
        // `time_t` é i32 em sistemas de 32 bits.
        #[allow(clippy::useless_conversion)]
        let t: libc::time_t = secs.try_into().ok()?;
        // SAFETY: `localtime_r` só escreve no `tm` que passamos; os dois ponteiros valem durante a chamada.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if !unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
            return Some((
                format!("{:04}-{:02}-{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday),
                format!("{:02}:{:02}", tm.tm_hour, tm.tm_min),
            ));
        }
    }
    let _ = secs;
    Some((ts.get(..10)?.to_string(), ts.get(11..16)?.to_string()))
}

/// `AAAA-MM-DD HH:MM` local.
pub fn local_text(ts: &str) -> String {
    local(ts).map_or_else(|| ts.to_string(), |(d, t)| format!("{d} {t}"))
}

/// Hoje, local (`AAAA-MM-DD`), e ontem.
pub fn today_and_yesterday() -> (String, String) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let day = |secs: u64| local(&timestamp(UNIX_EPOCH + std::time::Duration::from_secs(secs))).map(|(d, _)| d).unwrap_or_default();
    (day(now), day(now.saturating_sub(86_400)))
}

// ---------------------------------------------------------------------------------------------------------------
// `idetea history`

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    json: bool,
    name: Option<String>,
    since: Option<String>,
    last: Option<usize>,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args { json: false, name: None, since: None, last: Some(20) };
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut next = |flag: &str| it.next().cloned().ok_or_else(|| flag.to_string());
        match arg.as_str() {
            "--json" => a.json = true,
            "--name" => a.name = Some(next(arg)?),
            "--since" => {
                let v = next(arg)?;
                if epoch(&format!("{v}T00:00:00Z")).is_none() || v.len() != 10 {
                    return Err(v);
                }
                a.since = Some(v);
            }
            "--last" => a.last = Some(next(arg)?.parse().map_err(|_| arg.clone())?),
            "--all" => a.last = None,
            other => return Err(other.to_string()),
        }
    }
    Ok(a)
}

fn short_value(v: &Value) -> String {
    let s = match v {
        Value::String(s) => format!("{s:?}"),
        other => other.to_string(),
    };
    if s.chars().count() > 120 { format!("{}…", s.chars().take(119).collect::<String>()) } else { s }
}

/// Uma entrada em texto, para o agente ler sem `--json`.
fn describe(e: &Entry) -> String {
    let mut out = format!("{}  {} r{}  {:?}", local_text(e.time()), e.name, e.revision, e.title());
    if let Some(s) = &e.source {
        out.push_str(&format!("  ({s})"));
    }
    out.push('\n');
    match (&e.outcome, &e.response) {
        (Outcome::Answered, Some(r)) => {
            let values: Vec<String> = r.values.iter().map(|(k, v)| format!("{k}={}", short_value(v))).collect();
            out.push_str(&format!("  -> {}  {}\n", r.action, values.join(" ")));
            if let Some(c) = &r.comment {
                out.push_str(&format!("  {}: {c:?}\n", fl!("history-cli-comment")));
            }
            if e.read_by_agent.is_none() {
                out.push_str(&format!("  ! {}\n", fl!("history-cli-not-read")));
            }
        }
        _ => out.push_str(&format!("  {}\n", fl!("history-cli-unanswered"))),
    }
    out
}

pub fn run(args: &[String], cwd: &Path) -> i32 {
    let a = match parse(args) {
        Ok(a) => a,
        Err(bad) => {
            eprintln!("{}", fl!("history-cli-bad", arg = bad));
            eprintln!("{}", fl!("history-cli-usage"));
            return EXIT_INVALID;
        }
    };
    let root = crate::ask::project_root(cwd);
    let mut entries: Vec<Entry> = list(&root)
        .into_iter()
        .map(|(_, e)| e)
        .filter(|e| a.name.as_ref().is_none_or(|n| &e.name == n))
        .filter(|e| a.since.as_ref().is_none_or(|s| local(e.time()).is_some_and(|(d, _)| d.as_str() >= s.as_str())))
        .collect();
    if let Some(n) = a.last {
        entries.truncate(n);
    }
    // Mais velha primeiro: lê como uma conversa.
    entries.reverse();
    if a.json {
        println!("{}", serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".into()));
    } else if entries.is_empty() {
        eprintln!("{}", fl!("history-cli-empty"));
    } else {
        for e in &entries {
            print!("{}", describe(e));
        }
    }
    EXIT_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::time::Duration;
    use tea_declarative_ui::{Limits, parse as parse_doc};

    fn count_by_outcome(entries: &[(PathBuf, Entry)]) -> BTreeMap<&'static str, usize> {
        let mut out = BTreeMap::new();
        for (_, e) in entries {
            *out.entry(match e.outcome {
                Outcome::Answered => "answered",
                Outcome::Unanswered => "unanswered",
            })
            .or_insert(0) += 1;
        }
        out
    }

    fn tmp_root(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tea-history-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join(".idetea")).unwrap();
        d
    }

    fn doc(rev: u64) -> (Document, Vec<u8>) {
        let raw = serde_json::json!({
            "format": "idetea-ui", "version": 1, "revision": rev, "title": "Pergunta", "source": "teste",
            "data": {"a": "x"},
            "components": [
                {"id": "a", "component": "input", "value": "/a"},
                {"id": "ok", "component": "button", "label": "OK", "action": {"name": "ok", "context": ["/a"]}}
            ]
        })
        .to_string()
        .into_bytes();
        (parse_doc(&raw, &Limits::default()).unwrap(), raw)
    }

    fn response(event: &str, time: &str) -> Response {
        let values = BTreeMap::from([("/a".to_string(), Value::String("y".into()))]);
        Response::new("p", 1, 1, event.into(), "ok".into(), "ok", time.into(), values)
    }

    #[test]
    fn records_marks_lists_and_clears() {
        let root = tmp_root("ciclo");
        let (d, raw) = doc(1);
        let e = Entry::answered("p", &d, &raw, Some("2026-11-04T19:30:00Z".into()), response("abc-1-1", "2026-11-04T19:31:00Z"));
        let path = record(&root, &e).unwrap();
        assert!(path.ends_with("2026-11/20261104-193100-p-r1-abc-1-1.json"), "{}", path.display());
        assert_eq!(std::fs::read_to_string(dir(&root).join(".gitignore")).unwrap(), "*\n");

        let closed = UNIX_EPOCH + Duration::from_secs(1_793_900_000);
        record(&root, &Entry::unanswered("q", &d, &raw, None, closed)).unwrap();

        assert!(mark_read(&root, "abc-1-1", UNIX_EPOCH + Duration::from_secs(1_793_800_000)));
        assert!(!mark_read(&root, "nao-existe", SystemTime::now()));

        let all = list(&root);
        assert_eq!(all.len(), 2);
        let answered = all.iter().find(|(_, e)| e.name == "p").unwrap();
        assert!(answered.1.read_by_agent.is_some());
        assert_eq!(answered.1.document["data"]["a"], "x");
        assert!(answered.1.haystack().contains("\"y\""));
        assert_eq!(count_by_outcome(&all)["unanswered"], 1);
        // Mais nova primeiro.
        assert!(all[0].1.time() >= all[1].1.time());

        assert_eq!(clear(&root).unwrap(), 2);
        assert!(list(&root).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn same_second_does_not_overwrite() {
        let root = tmp_root("mesmo-segundo");
        let (d, raw) = doc(3);
        let when = UNIX_EPOCH + Duration::from_secs(1_793_900_000);
        let a = record(&root, &Entry::unanswered("p", &d, &raw, None, when)).unwrap();
        let b = record(&root, &Entry::unanswered("p", &d, &raw, None, when)).unwrap();
        assert_ne!(a, b);
        assert_eq!(list(&root).len(), 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_linked_history() {
        let root = tmp_root("link");
        let outside = root.join("fora");
        std::fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, dir(&root)).unwrap();
        let (d, raw) = doc(1);
        assert!(record(&root, &Entry::unanswered("p", &d, &raw, None, SystemTime::now())).is_err());
        assert!(list(&root).is_empty());
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn epoch_is_the_inverse_of_timestamp() {
        for secs in [0_u64, 951_782_400, 1_793_900_000, 4_102_444_799] {
            let ts = timestamp(UNIX_EPOCH + Duration::from_secs(secs));
            assert_eq!(epoch(&ts), Some(secs as i64), "{ts}");
        }
    }

    #[test]
    fn cli_arguments() {
        let s = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(parse(&s(&[])).unwrap().last, Some(20));
        let a = parse(&s(&["--json", "--name", "passo", "--since", "2026-11-01", "--all"])).unwrap();
        assert!(a.json && a.last.is_none());
        assert_eq!(a.name.as_deref(), Some("passo"));
        assert!(parse(&s(&["--since", "ontem"])).is_err());
        assert!(parse(&s(&["--last", "x"])).is_err());
        assert!(parse(&s(&["--nada"])).is_err());
    }
}
