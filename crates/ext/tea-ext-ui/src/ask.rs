//! `idetea ask` (RFC 0006, etapa 1): uma pergunta no painel Interfaces numa chamada só.
//!
//! Faz o que as instruções pedem ao agente passo a passo: apaga resposta velha, escolhe a revisão, confere o
//! documento como o painel, grava pelo temporário, espera tocando o `.wait`, lê a resposta da revisão certa, apaga
//! a resposta e (sem `--keep`) o documento. Os arquivos continuam sendo o protocolo; isto é o jeito fácil.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::{check, store};

/// Quanto esperar sem `--timeout`.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);
const POLL: Duration = Duration::from_millis(250);

/// Saídas: respondido, pergunta inválida (ou uso errado), "não" no `--confirm`, sem resposta no prazo.
pub const EXIT_OK: i32 = 0;
pub const EXIT_NO: i32 = 1;
pub const EXIT_INVALID: i32 = 2;
pub const EXIT_TIMEOUT: i32 = 4;

/// A forma curta escolhida, que decide o que sai no stdout.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Short {
    None,
    Options(Vec<String>),
    Confirm,
    Input(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    doc: Option<String>,
    name: Option<String>,
    title: Option<String>,
    text: Option<String>,
    source: Option<String>,
    short: Short,
    value: Option<String>,
    json: bool,
    keep: bool,
    timeout: Duration,
    resume: Option<String>,
    close: Option<String>,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args {
        doc: None,
        name: None,
        title: None,
        text: None,
        source: None,
        short: Short::None,
        value: None,
        json: false,
        keep: false,
        timeout: DEFAULT_TIMEOUT,
        resume: None,
        close: None,
    };
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut next = |flag: &str| it.next().cloned().ok_or_else(|| flag.to_string());
        match arg.as_str() {
            "--name" => a.name = Some(next(arg)?),
            "--title" => a.title = Some(next(arg)?),
            "--text" => a.text = Some(next(arg)?),
            "--source" => a.source = Some(next(arg)?),
            "--options" => {
                let list: Vec<String> = next(arg)?.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                if list.is_empty() {
                    return Err(arg.clone());
                }
                a.short = Short::Options(list);
            }
            "--confirm" => a.short = Short::Confirm,
            "--input" => a.short = Short::Input(next(arg)?),
            "--value" => a.value = Some(next(arg)?),
            "--json" => a.json = true,
            "--keep" => a.keep = true,
            "--timeout" => {
                let secs: u64 = next(arg)?.parse().map_err(|_| arg.clone())?;
                a.timeout = Duration::from_secs(secs);
            }
            "--resume" => a.resume = Some(next(arg)?),
            "--close" => a.close = Some(next(arg)?),
            other if other.starts_with("--") => return Err(other.to_string()),
            other if a.doc.is_none() => a.doc = Some(other.to_string()),
            other => return Err(other.to_string()),
        }
    }
    let has_short = a.short != Short::None || a.text.is_some();
    let modes = [a.doc.is_some(), has_short, a.resume.is_some(), a.close.is_some()].iter().filter(|m| **m).count();
    if modes != 1 {
        return Err(String::new());
    }
    for name in [&a.name, &a.resume, &a.close].into_iter().flatten() {
        if !store::valid_name(name) {
            return Err(name.clone());
        }
    }
    Ok(a)
}

/// A raiz do projeto: a pasta (desta ou de cima) que já tem `.idetea/`, senão a de cima com `.git`, senão esta.
pub fn project_root(cwd: &Path) -> PathBuf {
    cwd.ancestors()
        .find(|d| d.join(".idetea").is_dir())
        .or_else(|| cwd.ancestors().find(|d| d.join(".git").exists()))
        .unwrap_or(cwd)
        .to_path_buf()
}

/// O documento das formas curtas, com os botões no idioma da IDE.
fn short_doc(a: &Args) -> Value {
    let mut components = Vec::new();
    if let Some(text) = &a.text {
        components.push(json!({"id": "texto", "component": "markdown", "text": text}));
    }
    let mut data = json!({});
    match &a.short {
        Short::Options(list) => {
            components.push(json!({"id": "escolha", "component": "select", "label": fl!("ask-choose"), "value": "/escolha", "options": list}));
            components.push(json!({"id": "ok", "component": "button", "label": fl!("ask-ok"), "requires": "/escolha",
                                   "action": {"name": "ok", "context": ["/escolha"]}}));
        }
        Short::Confirm => {
            components.push(json!({"id": "sim", "component": "button", "label": fl!("ask-yes"), "action": {"name": "sim"}}));
            components.push(json!({"id": "nao", "component": "button", "label": fl!("ask-no"), "style": "secondary", "action": {"name": "nao"}}));
        }
        Short::Input(label) => {
            data = json!({"resposta": ""});
            components.push(json!({"id": "resposta", "component": "input", "label": label, "value": "/resposta"}));
            components.push(json!({"id": "ok", "component": "button", "label": fl!("ask-ok"), "requires": "/resposta",
                                   "action": {"name": "ok", "context": ["/resposta"]}}));
        }
        // Só texto: um aviso com "Ok" (o agente sabe que foi lido).
        Short::None => {
            components.push(json!({"id": "ok", "component": "button", "label": fl!("ask-ok"), "action": {"name": "ok"}}));
        }
    }
    json!({"format": "idetea-ui", "version": 1, "revision": 1, "title": a.title.clone().unwrap_or_else(|| fl!("ask-title")),
           "source": a.source.clone().unwrap_or_else(|| "agent".into()), "data": data, "components": components})
}

/// A maior revisão já usada com este nome: a do documento no painel, a das respostas que sobraram e a que o
/// próprio `ask` anotou em `.idetea/ui/.ask-revisions.json`.
fn last_revision(ui: &Path, name: &str) -> u64 {
    let read = |p: &Path| std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let doc = read(&ui.join(format!("{name}.json"))).and_then(|v| v["revision"].as_u64()).unwrap_or(0);
    let noted = read(&ui.join(".ask-revisions.json")).and_then(|v| v[name].as_u64()).unwrap_or(0);
    let answered = responses(ui, name).iter().filter_map(|p| read(p)).filter_map(|v| v["revision"].as_u64()).max().unwrap_or(0);
    doc.max(noted).max(answered)
}

fn note_revision(ui: &Path, name: &str, revision: u64) {
    let path = ui.join(".ask-revisions.json");
    let mut v = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).filter(Value::is_object).unwrap_or_else(|| json!({}));
    v[name] = revision.into();
    let tmp = ui.join(".ask-revisions.json.tmp");
    if std::fs::write(&tmp, v.to_string()).is_ok() {
        let _ = std::fs::rename(tmp, path);
    }
}

/// As respostas deste nome, em ordem.
fn responses(ui: &Path, name: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(ui.join("responses").join(name)) else { return Vec::new() };
    let mut out: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    out.sort();
    out
}

fn clear_responses(ui: &Path, name: &str) {
    for p in responses(ui, name) {
        let _ = std::fs::remove_file(p);
    }
}

/// Publica o documento (texto JSON) com o nome e a revisão certos. Erros do painel voltam como texto.
fn publish(root: &Path, name: &str, text: &str) -> Result<u64, String> {
    let mut doc: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if !doc.is_object() {
        return Err(fl!("ask-not-object"));
    }
    store::prepare(root).map_err(|e| e.to_string())?;
    let ui = store::ui_dir(root);
    // A do documento vale se for maior (quem já usou o nome sem o `ask` sabe a dele).
    let revision = (last_revision(&ui, name) + 1).max(doc["revision"].as_u64().unwrap_or(0));
    doc["revision"] = revision.into();
    // Resposta velha nunca é tomada pela nova.
    clear_responses(&ui, name);
    let tmp = ui.join(format!(".{name}.json.tmp"));
    std::fs::write(&tmp, serde_json::to_string_pretty(&doc).unwrap_or_default()).map_err(|e| e.to_string())?;
    let report = check::check(&tmp, root);
    if !report.ok {
        let _ = std::fs::remove_file(&tmp);
        return Err(report.lines.join("\n"));
    }
    for line in report.lines.iter().skip(1) {
        eprintln!("{line}");
    }
    std::fs::rename(&tmp, ui.join(format!("{name}.json"))).map_err(|e| e.to_string())?;
    note_revision(&ui, name, revision);
    Ok(revision)
}

/// Espera a resposta da `revision`, tocando o `.wait`. Respostas de outra revisão (um clique numa tela velha) são
/// apagadas e não contam.
fn wait(root: &Path, name: &str, revision: u64, timeout: Duration) -> Option<Value> {
    let ui = store::ui_dir(root);
    let beat = ui.join(format!("{name}.wait"));
    let end = Instant::now() + timeout;
    loop {
        for path in responses(&ui, name) {
            let Some(v) = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else { continue };
            let _ = std::fs::remove_file(&path);
            if v["revision"].as_u64() == Some(revision) {
                return Some(v);
            }
        }
        if Instant::now() >= end {
            return None;
        }
        let _ = std::fs::write(&beat, "");
        std::thread::sleep(POLL);
    }
}

fn close(root: &Path, name: &str) {
    let ui = store::ui_dir(root);
    for f in [format!("{name}.json"), format!("{name}.wait")] {
        let _ = std::fs::remove_file(ui.join(f));
    }
    clear_responses(&ui, name);
}

/// O valor de um ponteiro na resposta: texto sem aspas, o resto como JSON.
fn show(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Um nome que não colide com o de outro agente: `ask-<segundos em base 36>`.
fn fresh_name() -> String {
    let mut n = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let mut s = String::new();
    while n > 0 {
        let d = (n % 36) as u32;
        s.insert(0, char::from_digit(d, 36).unwrap_or('0'));
        n /= 36;
    }
    format!("ask-{s}")
}

pub fn run(args: &[String], cwd: &Path) -> i32 {
    let a = match parse(args) {
        Ok(a) => a,
        Err(arg) => {
            if !arg.is_empty() {
                eprintln!("{}", fl!("ask-bad-arg", arg = arg));
            }
            eprintln!("{}", fl!("ask-usage"));
            return EXIT_INVALID;
        }
    };
    let root = project_root(cwd);
    if let Some(name) = &a.close {
        close(&root, name);
        return EXIT_OK;
    }
    let (name, revision) = if let Some(name) = &a.resume {
        let ui = store::ui_dir(&root);
        let doc = std::fs::read_to_string(ui.join(format!("{name}.json"))).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
        match doc.and_then(|d| d["revision"].as_u64()) {
            Some(r) => (name.clone(), r),
            None => {
                eprintln!("{}", fl!("ask-no-such", name = name.clone()));
                return EXIT_INVALID;
            }
        }
    } else {
        let text = match (&a.doc, a.short != Short::None || a.text.is_some()) {
            (Some(d), _) if d == "-" => {
                let mut s = String::new();
                if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut s) {
                    eprintln!("{e}");
                    return EXIT_INVALID;
                }
                s
            }
            (Some(d), _) => match std::fs::read_to_string(cwd.join(d)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{d}: {e}");
                    return EXIT_INVALID;
                }
            },
            (None, _) => short_doc(&a).to_string(),
        };
        let name = a.name.clone().unwrap_or_else(fresh_name);
        match publish(&root, &name, &text) {
            Ok(r) => (name, r),
            Err(e) => {
                eprintln!("{}", fl!("ask-invalid", error = e));
                return EXIT_INVALID;
            }
        }
    };
    let Some(response) = wait(&root, &name, revision, a.timeout) else {
        eprintln!("{}", fl!("ask-timeout", name = name.clone()));
        return EXIT_TIMEOUT;
    };
    if !a.keep {
        close(&root, &name);
    }
    if response["ide"]["outdated"].as_bool() == Some(true) {
        eprintln!("{}", fl!("ask-outdated"));
    }
    if let Some(c) = response["comment"].as_str() {
        eprintln!("{}", fl!("ask-comment", comment = c.to_string()));
    }
    let action = response["action"].as_str().unwrap_or_default().to_string();
    let values = &response["values"];
    if a.json {
        println!("{}", serde_json::to_string_pretty(&response).unwrap_or_default());
    } else if let Some(pointer) = &a.value {
        println!("{}", show(&values[pointer.as_str()]));
    } else {
        match &a.short {
            Short::Options(_) => println!("{}", show(&values["/escolha"])),
            Short::Input(_) => println!("{}", show(&values["/resposta"])),
            Short::Confirm => println!("{}", if action == "sim" { "yes" } else { "no" }),
            Short::None if a.doc.is_none() && a.resume.is_none() => println!("{action}"),
            Short::None => println!("{}", serde_json::to_string_pretty(&response).unwrap_or_default()),
        }
    }
    if a.short == Short::Confirm && action != "sim" {
        return EXIT_NO;
    }
    EXIT_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn reads_the_arguments() {
        let a = parse(&s(&["--title", "T", "--options", "a| b |", "--timeout", "5"])).unwrap();
        assert_eq!(a.short, Short::Options(vec!["a".into(), "b".into()]));
        assert_eq!(a.timeout, Duration::from_secs(5));
        assert!(parse(&s(&["doc.json", "--confirm"])).is_err(), "um modo só");
        assert!(parse(&s(&[])).is_err());
        assert!(parse(&s(&["--confirm", "--name", "A B"])).is_err(), "nome do painel");
        assert!(parse(&s(&["--resume", "x"])).is_ok());
        assert!(parse(&s(&["--options", "|"])).is_err());
        assert!(parse(&s(&["--wat"])).is_err());
    }

    #[test]
    fn short_forms_are_valid_documents() {
        for a in [
            parse(&s(&["--title", "T", "--text", "**oi**", "--options", "a|b"])).unwrap(),
            parse(&s(&["--confirm"])).unwrap(),
            parse(&s(&["--input", "Nome"])).unwrap(),
            parse(&s(&["--text", "só um aviso"])).unwrap(),
        ] {
            let doc = short_doc(&a).to_string();
            let parsed = tea_declarative_ui::parse(doc.as_bytes(), &tea_declarative_ui::Limits::default());
            assert!(parsed.as_ref().is_ok_and(|d| d.warnings.is_empty()), "{doc}: {parsed:?}");
        }
    }

    #[test]
    fn publishes_waits_for_its_revision_and_cleans_up() {
        let root = std::env::temp_dir().join(format!("idetea-ask-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".idetea")).unwrap();
        let ui = store::ui_dir(&root);
        let doc = short_doc(&parse(&s(&["--options", "a|b"])).unwrap()).to_string();

        // Um documento antigo com revisão 7 e uma resposta velha: a nova é a 8, e a velha some antes de publicar.
        std::fs::create_dir_all(ui.join("responses/q")).unwrap();
        std::fs::write(ui.join("q.json"), doc.replace("\"revision\":1", "\"revision\":7")).unwrap();
        std::fs::write(ui.join("responses/q/000001-x.json"), r#"{"revision": 7, "action": "ok", "values": {"/escolha": "a"}}"#).unwrap();
        let revision = publish(&root, "q", &doc).unwrap();
        assert_eq!(revision, 8);
        assert!(responses(&ui, "q").is_empty(), "resposta velha apagada");
        assert!(!ui.join(".q.json.tmp").exists());

        // Um clique numa tela velha (revisão 7) não conta; o da 8, sim.
        std::fs::write(ui.join("responses/q/000002-y.json"), r#"{"revision": 7, "action": "ok"}"#).unwrap();
        std::fs::write(ui.join("responses/q/000003-z.json"), r#"{"revision": 8, "action": "ok", "values": {"/escolha": "b"}}"#).unwrap();
        let got = wait(&root, "q", revision, Duration::from_secs(2)).unwrap();
        assert_eq!(got["values"]["/escolha"], "b");
        assert!(responses(&ui, "q").is_empty(), "as respostas lidas saem");

        // A revisão anotada sobrevive ao documento apagado.
        close(&root, "q");
        assert!(!ui.join("q.json").exists());
        assert_eq!(publish(&root, "q", &doc).unwrap(), 9);
        // Uma revisão maior no documento vale.
        assert_eq!(publish(&root, "q", &doc.replace("\"revision\":1", "\"revision\":40")).unwrap(), 40);
        assert_eq!(publish(&root, "q", &doc).unwrap(), 41);
        close(&root, "q");
        assert_eq!(publish(&root, "q", &doc).unwrap(), 42);

        // Sem resposta: volta `None` no prazo e o documento fica.
        assert!(wait(&root, "q", 42, Duration::from_millis(300)).is_none());
        assert!(ui.join("q.json").exists());
        assert!(ui.join("q.wait").exists(), "tocou o .wait enquanto esperava");

        // Um documento que o painel recusa não é publicado.
        assert!(publish(&root, "ruim", r#"{"format": "idetea-ui", "version": 1, "components": [{"id": "x"}]}"#).is_err());
        assert!(!ui.join("ruim.json").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
