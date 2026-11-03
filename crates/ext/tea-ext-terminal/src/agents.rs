//! Terminais pedidos por um agente (RFC 0005, etapa 4).
//!
//! O agente grava `.idetea/agents/<nome>.json` com o comando; a IDE mostra o pedido inteiro no painel do terminal
//! e só roda depois que o usuário permite. O andamento vai para `.idetea/agents/<nome>.status.json`: `pending`
//! (a IDE viu e espera o usuário), `running`, `exited` (com `code`), `refused`, `closed` (o usuário fechou a aba
//! antes do fim), `invalid` (com `error`) e `lost` (a IDE que rodava fechou). Apagar o pedido encerra o terminal.
//!
//! O que aparece na tela e o que roda saem da mesma cópia em memória: regravar o arquivo depois de permitido não
//! muda o que roda, e um pedido alterado enquanto espera aparece de novo, já alterado.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Deserialize;

pub const DIR: &str = ".idetea/agents";
pub const FORMAT: &str = "idetea-terminal";
pub const STATUS_FORMAT: &str = "idetea-terminal-status";

/// Limites: o pedido inteiro precisa caber na tela de confirmação.
const MAX_FILE: u64 = 64 * 1024;
const MAX_COMMAND: usize = 4000;
const MAX_TITLE: usize = 80;
const MAX_SOURCE: usize = 40;

/// Um pedido válido, como vai aparecer e rodar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub name: String,
    pub title: String,
    pub source: Option<String>,
    pub command: String,
    /// Pasta onde roda (absoluta; dentro do projeto).
    pub cwd: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    format: String,
    version: u32,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    source: Option<String>,
    command: String,
    #[serde(default)]
    cwd: Option<String>,
}

/// O andamento gravado em `<nome>.status.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Pending,
    Running,
    /// Quem grava é o script do terminal (`AGENT_SCRIPT`), com o mesmo formato; aqui só nos testes.
    #[cfg_attr(not(test), allow(dead_code))]
    Exited(i32),
    Refused,
    Closed,
    Invalid(String),
    Lost,
}

impl State {
    fn word(&self) -> &'static str {
        match self {
            State::Pending => "pending",
            State::Running => "running",
            State::Exited(_) => "exited",
            State::Refused => "refused",
            State::Closed => "closed",
            State::Invalid(_) => "invalid",
            State::Lost => "lost",
        }
    }
}

/// O que está no disco: o estado (só a palavra) e a IDE que o gravou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnDisk {
    pub state: String,
    pub ide: Option<u32>,
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

pub fn dir(root: &Path) -> PathBuf {
    root.join(DIR)
}

pub fn status_path(root: &Path, name: &str) -> PathBuf {
    dir(root).join(format!("{name}.status.json"))
}

/// Os nomes dos pedidos na pasta (`<nome>.json`, sem os de status e os temporários).
fn names(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir(root)) else { return Vec::new() };
    let mut out: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().to_str()?.to_string();
            let name = file.strip_suffix(".json")?;
            if name.ends_with(".status") || name.starts_with('.') {
                return None;
            }
            Some((name.to_string(), e.path()))
        })
        .collect();
    out.sort();
    out
}

/// Muda quando um pedido ou um status entra, sai ou é regravado (só `stat`). O status conta: apagá-lo é como o
/// agente pede de novo com o mesmo nome.
pub fn signature(root: &Path) -> Vec<(String, u64, Option<SystemTime>)> {
    let Ok(entries) = std::fs::read_dir(dir(root)) else { return Vec::new() };
    let mut out: Vec<(String, u64, Option<SystemTime>)> = entries
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().to_str()?.to_string();
            if !file.ends_with(".json") || file.starts_with('.') {
                return None;
            }
            let meta = e.metadata().ok();
            Some((file, meta.as_ref().map_or(0, |m| m.len()), meta.and_then(|m| m.modified().ok())))
        })
        .collect();
    out.sort();
    out
}

/// Todos os pedidos da pasta: válido ou o motivo de não ser.
pub fn scan(root: &Path) -> BTreeMap<String, Result<Request, String>> {
    names(root).into_iter().map(|(name, path)| (name.clone(), read(root, &name, &path))).collect()
}

fn read(root: &Path, name: &str, path: &Path) -> Result<Request, String> {
    if !valid_name(name) {
        return Err(format!("nome inválido: use letras minúsculas, dígitos, _ e - (até 64): {name}"));
    }
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len > MAX_FILE {
        return Err(format!("arquivo grande demais ({len} bytes; até {MAX_FILE})"));
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let raw: Raw = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if raw.format != FORMAT {
        return Err(format!("format: esperado \"{FORMAT}\""));
    }
    if raw.version != 1 {
        return Err(format!("version: só existe a 1 (veio {})", raw.version));
    }
    let command = raw.command.trim().to_string();
    if command.is_empty() {
        return Err("command: vazio".into());
    }
    if command.chars().count() > MAX_COMMAND {
        return Err(format!("command: longo demais (até {MAX_COMMAND} caracteres)"));
    }
    // Controles (menos quebra de linha e tab) poderiam esconder parte do comando na tela.
    if command.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err("command: tem caracteres de controle".into());
    }
    let clean = |s: Option<String>, max: usize| -> Option<String> {
        let s: String = s?.chars().filter(|c| !c.is_control()).take(max).collect();
        let s = s.trim().to_string();
        (!s.is_empty()).then_some(s)
    };
    let title = clean(raw.title, MAX_TITLE).unwrap_or_else(|| name.to_string());
    let source = clean(raw.source, MAX_SOURCE);
    let cwd = match raw.cwd.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        None => root.to_path_buf(),
        Some(c) => {
            let p = Path::new(c);
            if p.is_absolute() {
                return Err("cwd: use um caminho relativo à raiz do projeto".into());
            }
            root.join(p)
        }
    };
    let cwd = cwd.canonicalize().map_err(|e| format!("cwd: {e}"))?;
    let base = root.canonicalize().map_err(|e| e.to_string())?;
    if !cwd.starts_with(&base) {
        return Err("cwd: fora do projeto".into());
    }
    if !cwd.is_dir() {
        return Err("cwd: não é uma pasta".into());
    }
    Ok(Request { name: name.to_string(), title, source, command, cwd })
}

pub fn read_status(root: &Path, name: &str) -> Option<OnDisk> {
    let text = std::fs::read_to_string(status_path(root, name)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(OnDisk {
        state: value.get("state")?.as_str()?.to_string(),
        ide: value.get("ide").and_then(serde_json::Value::as_u64).and_then(|p| u32::try_from(p).ok()),
    })
}

/// O JSON do status (o script do terminal grava o de `exited` igual, pelo `printf`).
pub fn status_json(state: &State, ide: u32) -> String {
    let mut v = serde_json::json!({ "format": STATUS_FORMAT, "version": 1, "state": state.word(), "ide": ide });
    match state {
        State::Exited(code) => v["code"] = (*code).into(),
        State::Invalid(error) => v["error"] = error.clone().into(),
        _ => {}
    }
    format!("{v}\n")
}

/// Grava o status por um temporário (quem espera nunca lê pela metade).
pub fn write_status(root: &Path, name: &str, state: &State) -> std::io::Result<()> {
    let path = status_path(root, name);
    let tmp = dir(root).join(format!(".{name}.status.json.tmp"));
    std::fs::write(&tmp, status_json(state, std::process::id()))?;
    std::fs::rename(tmp, path)
}

/// `.gitignore` com `*`: pedidos e status são da máquina, não do projeto.
pub fn ensure_gitignore(root: &Path) {
    let path = dir(root).join(".gitignore");
    if dir(root).is_dir() && !path.exists() {
        let _ = std::fs::write(path, "*\n");
    }
}

/// O processo ainda existe? Sem `/proc` (fora do Linux), supõe que sim: melhor não marcar como perdido o que roda.
pub fn alive(pid: u32) -> bool {
    let proc = Path::new("/proc");
    !proc.is_dir() || proc.join(pid.to_string()).exists()
}

/// O que fazer com um pedido que a IDE ainda não tratou, olhando o status no disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Mostrar ao usuário e esperar.
    Ask,
    /// Já tratado (por esta IDE antes, ou por outra aberta no mesmo projeto): não mexer.
    Leave,
    /// Rodava numa IDE que fechou: gravar `lost`.
    Lost,
}

pub fn decide(on_disk: Option<&OnDisk>, ours: u32) -> Decision {
    let Some(d) = on_disk else { return Decision::Ask };
    if owned_by_other(on_disk, ours) {
        return Decision::Leave;
    }
    match d.state.as_str() {
        "pending" => Decision::Ask,
        "running" => Decision::Lost,
        _ => Decision::Leave,
    }
}

/// Outra IDE, ainda aberta, espera o usuário ou roda este pedido.
pub fn owned_by_other(on_disk: Option<&OnDisk>, ours: u32) -> bool {
    on_disk.is_some_and(|d| {
        matches!(d.state.as_str(), "pending" | "running") && d.ide.is_some_and(|p| p != ours && alive(p))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-agents-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(DIR)).unwrap();
        dir
    }

    fn put(root: &Path, name: &str, text: &str) {
        std::fs::write(dir(root).join(format!("{name}.json")), text).unwrap();
    }

    #[test]
    fn reads_requests_and_says_what_is_wrong() {
        let root = tmp_root("scan");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        put(&root, "revisor", r#"{"format": "idetea-terminal", "version": 1, "title": "Revisor", "source": "claude", "command": "echo oi", "cwd": "sub"}"#);
        put(&root, "sem-titulo", r#"{"format": "idetea-terminal", "version": 1, "command": "ls"}"#);
        put(&root, "fora", r#"{"format": "idetea-terminal", "version": 1, "command": "ls", "cwd": ".."}"#);
        put(&root, "vazio", r#"{"format": "idetea-terminal", "version": 1, "command": "  "}"#);
        put(&root, "escondido", "{\"format\": \"idetea-terminal\", \"version\": 1, \"command\": \"ls\\u001b[2K rm\"}");
        put(&root, "Maiuscula", r#"{"format": "idetea-terminal", "version": 1, "command": "ls"}"#);
        // Não são pedidos:
        std::fs::write(dir(&root).join("revisor.status.json"), "{}").unwrap();
        std::fs::write(dir(&root).join(".revisor.json.tmp"), "{}").unwrap();
        std::fs::write(dir(&root).join(".x.json"), "{}").unwrap();

        let found = scan(&root);
        assert_eq!(found.keys().map(String::as_str).collect::<Vec<_>>(), ["Maiuscula", "escondido", "fora", "revisor", "sem-titulo", "vazio"]);
        let r = found["revisor"].as_ref().unwrap();
        assert_eq!((r.title.as_str(), r.source.as_deref(), r.command.as_str()), ("Revisor", Some("claude"), "echo oi"));
        assert_eq!(r.cwd, root.join("sub").canonicalize().unwrap());
        let r = found["sem-titulo"].as_ref().unwrap();
        assert_eq!((r.title.as_str(), r.cwd.clone()), ("sem-titulo", root.canonicalize().unwrap()));
        assert!(found["fora"].as_ref().unwrap_err().contains("fora do projeto"));
        assert!(found["vazio"].as_ref().unwrap_err().contains("vazio"));
        assert!(found["escondido"].as_ref().unwrap_err().contains("controle"));
        assert!(found["Maiuscula"].as_ref().unwrap_err().contains("nome"));
    }

    #[test]
    fn status_round_trip_and_decisions() {
        let root = tmp_root("status");
        write_status(&root, "a", &State::Exited(3)).unwrap();
        let text = std::fs::read_to_string(status_path(&root, "a")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!((v["state"].as_str(), v["code"].as_i64(), v["format"].as_str()), (Some("exited"), Some(3), Some(STATUS_FORMAT)));
        let me = std::process::id();
        assert_eq!(read_status(&root, "a"), Some(OnDisk { state: "exited".into(), ide: Some(me) }));

        let disk = |state: &str, ide: Option<u32>| OnDisk { state: state.into(), ide };
        // Um pid que não existe (acima do máximo do Linux).
        let dead = 4_194_400;
        assert_eq!(decide(None, me), Decision::Ask);
        assert_eq!(decide(Some(&disk("pending", Some(dead))), me), Decision::Ask);
        assert_eq!(decide(Some(&disk("running", Some(dead))), me), Decision::Lost);
        assert_eq!(decide(Some(&disk("running", Some(me))), me), Decision::Lost);
        assert_eq!(decide(Some(&disk("exited", Some(dead))), me), Decision::Leave);
        assert_eq!(decide(Some(&disk("refused", None)), me), Decision::Leave);
        // Outra IDE viva (o pid 1 sempre existe) é dona do pedido.
        assert_eq!(decide(Some(&disk("running", Some(1))), me), Decision::Leave);
        assert_eq!(decide(Some(&disk("pending", Some(1))), me), Decision::Leave);
    }
}
