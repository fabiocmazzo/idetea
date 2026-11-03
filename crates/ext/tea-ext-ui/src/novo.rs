//! "Novo projeto" (RFC 0005, etapa 3): cria a pasta, o git, as skills do modelo, o `AGENTS.md` com o pedido inicial
//! e as instruções do painel. Quem cria o projeto de verdade é o agente, seguindo as skills.
//!
//! Pela paleta, a IDE mostra o formulário no próprio painel Interfaces (o documento [`DOC_NAME`], escrito e lido
//! por ela); pela linha de comando, `idetea new <pasta> [--modelo id] [--descricao texto] [--sem-git]`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tea_skills::models;

use crate::{store, teach};

/// O documento do formulário, no `.idetea/ui/` do projeto aberto. A IDE escreve e lê a resposta ela mesma.
pub const DOC_NAME: &str = "idetea-novo-projeto";

/// O que criar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// A pasta do projeto (ainda não existe, ou está vazia).
    pub dir: PathBuf,
    pub model: String,
    /// O que o usuário disse que o projeto é (opcional): vai no pedido inicial.
    pub description: String,
    pub git: bool,
    /// O agente a abrir num terminal da IDE na janela nova (`claude`, `codex`…), ou nenhum.
    pub agent: Option<String>,
}

/// Agentes que o formulário oferece, na ordem; aparecem os que estão no `PATH`.
pub const AGENTS: [&str; 4] = ["claude", "codex", "gemini", "opencode"];

/// O pedido de terminal que a janela nova mostra para o usuário confirmar (`.idetea/agents/<AGENT_REQUEST>.json`).
pub const AGENT_REQUEST: &str = "agente";

/// O que o agente recebe ao abrir: começar pelo pedido inicial do `AGENTS.md`.
const AGENT_PROMPT: &str = "Start with the initial request in AGENTS.md.";

/// Um nome de agente que vale: só letras, dígitos, `-` e `_` (vai dentro de um comando de shell).
pub fn valid_agent(name: &str) -> bool {
    !name.is_empty() && name.len() <= 40 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// O executável está no `PATH`?
fn in_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|d| {
            let f = d.join(name);
            f.is_file() && std::fs::metadata(&f).is_ok_and(|m| {
                use std::os::unix::fs::PermissionsExt;
                m.permissions().mode() & 0o111 != 0
            })
        })
    })
}

/// Os agentes de [`AGENTS`] instalados aqui.
pub fn installed_agents() -> Vec<&'static str> {
    AGENTS.into_iter().filter(|a| in_path(a)).collect()
}

/// Grava o pedido de terminal do agente no projeto novo (temporário e `rename`, como os agentes fazem).
fn write_agent_request(dir: &Path, agent: &str) -> Result<(), String> {
    if !valid_agent(agent) {
        return Err(fl!("new-bad-agent", agent = agent.to_string()));
    }
    let agents = dir.join(".idetea").join("agents");
    std::fs::create_dir_all(&agents).map_err(|e| e.to_string())?;
    std::fs::write(agents.join(".gitignore"), "*\n").map_err(|e| e.to_string())?;
    let request = json!({"format": "idetea-terminal", "version": 1, "title": fl!("new-agent-title", agent = agent.to_string()),
                         "source": "IDE TEA", "command": format!("{agent} '{AGENT_PROMPT}'"), "cwd": "."});
    let tmp = agents.join(format!(".{AGENT_REQUEST}.json.tmp"));
    std::fs::write(&tmp, serde_json::to_string_pretty(&request).unwrap_or_default()).map_err(|e| e.to_string())?;
    std::fs::rename(tmp, agents.join(format!("{AGENT_REQUEST}.json"))).map_err(|e| e.to_string())
}

fn git_init(dir: &Path) -> Result<(), String> {
    let out = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
}

/// O `AGENTS.md` antes das instruções do painel: o nome e o pedido inicial (o agente tira a seção no fim).
fn agents_head(name: &str, model: &models::Model, description: &str) -> String {
    let mut out = format!("# {name}\n\n## Initial request\n\n");
    if !description.trim().is_empty() {
        out.push_str(&format!("What this project is, in the user's words: {}\n\n", description.trim()));
    }
    out.push_str(&format!("{}\n\nRemove this section in the first commit, once the project is created.\n", model.request));
    out
}

/// Cria o projeto. Devolve o que foi feito, uma linha por passo. Se falhar antes de terminar, a pasta criada
/// aqui é apagada (ela estava vazia ou não existia).
pub fn create(req: &Request, cache: &Path) -> Result<Vec<String>, String> {
    let model = models::get(&req.model).ok_or_else(|| fl!("new-bad-model", model = req.model.clone()))?;
    let name = req.dir.file_name().and_then(|n| n.to_str()).filter(|n| !n.trim().is_empty()).ok_or_else(|| fl!("new-bad-dir", dir = req.dir.display().to_string()))?.to_string();
    let existed = req.dir.exists();
    if existed && std::fs::read_dir(&req.dir).map_err(|e| e.to_string())?.next().is_some() {
        return Err(fl!("new-not-empty", dir = req.dir.display().to_string()));
    }
    std::fs::create_dir_all(&req.dir).map_err(|e| format!("{}: {e}", req.dir.display()))?;
    let result = fill(req, &model, &name, cache);
    if result.is_err() {
        // Só o que esta chamada criou: a pasta estava vazia.
        if existed {
            if let Ok(entries) = std::fs::read_dir(&req.dir) {
                for e in entries.flatten() {
                    let _ = if e.path().is_dir() { std::fs::remove_dir_all(e.path()) } else { std::fs::remove_file(e.path()) };
                }
            }
        } else {
            let _ = std::fs::remove_dir_all(&req.dir);
        }
    }
    result
}

fn fill(req: &Request, model: &models::Model, name: &str, cache: &Path) -> Result<Vec<String>, String> {
    let mut done = vec![fl!("new-done-dir", dir = req.dir.display().to_string())];
    // As skills primeiro: é o passo que usa a rede, e o que mais pode falhar.
    let project = tea_skills::Project { root: &req.dir, cache };
    for (source, names) in models::by_source(model) {
        let mut out = Vec::new();
        project.add(&source, &names, false, &mut out).map_err(|e| e.message().to_string())?;
        done.extend(out);
    }
    if req.git {
        git_init(&req.dir)?;
        done.push(fl!("new-done-git"));
    }
    std::fs::write(req.dir.join("AGENTS.md"), agents_head(name, model, &req.description)).map_err(|e| e.to_string())?;
    // O Claude Code lê o CLAUDE.md; os outros agentes, o AGENTS.md.
    std::fs::write(req.dir.join("CLAUDE.md"), "@AGENTS.md\n").map_err(|e| e.to_string())?;
    let plan = teach::plan(&req.dir, teach::INSTRUCTIONS).map_err(|e| e.to_string())?;
    teach::apply(&plan).map_err(|e| e.to_string())?;
    done.push(fl!("new-done-agents"));
    store::prepare(&req.dir).map_err(|e| e.to_string())?;
    if let Some(agent) = &req.agent {
        write_agent_request(&req.dir, agent)?;
        done.push(fl!("new-done-agent", agent = agent.clone()));
    }
    Ok(done)
}

/// Abre o projeto numa janela nova (o mesmo executável desta IDE).
pub fn open_window(dir: &Path) -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let mut child = Command::new(exe).arg(dir).current_dir(dir).stdin(Stdio::null()).spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// O formulário no painel. `parent` é a pasta onde criar (padrão: a de cima do projeto aberto); `error`, o motivo
/// de a última tentativa não ter dado certo.
pub fn form(revision: u64, values: &Value, error: Option<&str>) -> Value {
    let options: Vec<Value> = models::all().iter().map(|m| json!({"value": m.id, "label": fl!("new-model", model = m.id.clone())})).collect();
    let mut agents: Vec<Value> = vec![json!({"value": "nenhum", "label": fl!("new-agent-none")})];
    let mut known: Vec<String> = installed_agents().into_iter().map(String::from).collect();
    // O que estava escolhido continua na lista, mesmo que tenha saído do PATH.
    if let Some(a) = values.get("agente").and_then(Value::as_str).filter(|a| *a != "nenhum" && !known.iter().any(|k| k == a)) {
        known.push(a.to_string());
    }
    agents.extend(known.into_iter().map(|a| json!({"value": a, "label": a})));
    let mut components = vec![json!({"id": "intro", "component": "markdown", "text": fl!("new-intro")})];
    if let Some(e) = error {
        components.push(json!({"id": "erro", "component": "text", "style": "error", "text": e}));
    }
    components.extend([
        json!({"id": "pasta", "component": "input", "label": fl!("new-field-parent"), "value": "/pasta"}),
        json!({"id": "nome", "component": "input", "label": fl!("new-field-name"), "placeholder": fl!("new-field-name-hint"), "value": "/nome"}),
        json!({"id": "modelo", "component": "select", "label": fl!("new-field-model"), "value": "/modelo", "options": options}),
        json!({"id": "descricao", "component": "input", "label": fl!("new-field-description"), "placeholder": fl!("new-field-description-hint"), "value": "/descricao"}),
        json!({"id": "git", "component": "checkbox", "label": fl!("new-field-git"), "value": "/git"}),
        json!({"id": "agente", "component": "select", "label": fl!("new-field-agent"), "value": "/agente", "options": agents}),
        json!({"id": "criar", "component": "button", "label": fl!("new-create"), "requires": ["/pasta", "/nome"],
               "action": {"name": "criar", "context": ["/pasta", "/nome", "/modelo", "/descricao", "/git", "/agente"]}}),
        json!({"id": "cancelar", "component": "button", "label": fl!("ui-cancel"), "style": "secondary", "action": {"name": "cancelar"}}),
    ]);
    json!({"format": "idetea-ui", "version": 1, "revision": revision, "title": fl!("new-title"), "source": "IDE TEA",
           "data": values, "components": components})
}

/// Os valores iniciais do formulário.
pub fn initial_values(root: &Path) -> Value {
    let parent = root.parent().unwrap_or(root).display().to_string();
    let agent = installed_agents().first().copied().unwrap_or("nenhum");
    json!({"pasta": parent, "nome": "", "modelo": "vazio", "descricao": "", "git": true, "agente": agent})
}

/// Grava o formulário em `.idetea/ui/` (temporário e `rename`, como pedimos aos agentes).
pub fn write_form(root: &Path, doc: &Value) -> std::io::Result<()> {
    let ui = store::ui_dir(root);
    std::fs::create_dir_all(&ui)?;
    let tmp = ui.join(format!(".{DOC_NAME}.json.tmp"));
    std::fs::write(&tmp, serde_json::to_string_pretty(doc).unwrap_or_default())?;
    std::fs::rename(tmp, ui.join(format!("{DOC_NAME}.json")))
}

/// O pedido a partir da resposta do formulário (`/pasta`, `/nome`…).
pub fn request_from(values: &std::collections::BTreeMap<String, Value>) -> Result<Request, String> {
    let text = |k: &str| values.get(k).and_then(Value::as_str).unwrap_or("").trim().to_string();
    let name = text("/nome");
    if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(fl!("new-bad-name", name = name));
    }
    let parent = text("/pasta");
    let parent = match parent.strip_prefix("~/") {
        Some(rest) => std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(rest),
        None => PathBuf::from(parent),
    };
    if !parent.is_absolute() {
        return Err(fl!("new-bad-dir", dir = parent.display().to_string()));
    }
    let model = Some(text("/modelo")).filter(|m| !m.is_empty()).unwrap_or_else(|| "vazio".into());
    let agent = Some(text("/agente")).filter(|a| !a.is_empty() && a != "nenhum");
    if let Some(a) = agent.as_deref().filter(|a| !valid_agent(a)) {
        return Err(fl!("new-bad-agent", agent = a.to_string()));
    }
    let git = values.get("/git").and_then(Value::as_bool).unwrap_or(true);
    Ok(Request { dir: parent.join(name), model, description: text("/descricao"), git, agent })
}

/// `idetea new <pasta> [--modelo id] [--descricao texto] [--sem-git] [--agente nome]`. Saída 0 (criado), 1 (erro), 2 (uso).
pub fn run(args: &[String], cwd: &Path) -> i32 {
    let mut dir = None;
    let mut model = "vazio".to_string();
    let mut description = String::new();
    let mut git = true;
    let mut agent = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--modelo" | "--model" => match it.next() {
                Some(m) => model = m.clone(),
                None => return usage(a),
            },
            "--descricao" | "--description" => match it.next() {
                Some(d) => description = d.clone(),
                None => return usage(a),
            },
            "--sem-git" | "--no-git" => git = false,
            "--agente" | "--agent" => match it.next() {
                Some(a) if valid_agent(a) => agent = Some(a.clone()),
                _ => return usage(a),
            },
            other if other.starts_with('-') || dir.is_some() => return usage(other),
            other => dir = Some(cwd.join(other)),
        }
    }
    let Some(dir) = dir else { return usage("") };
    let Some(cache) = tea_skills::cache_dir() else {
        eprintln!("{}", fl!("new-failed", error = "HOME"));
        return 1;
    };
    match create(&Request { dir: dir.clone(), model, description, git, agent }, &cache) {
        Ok(done) => {
            for line in done {
                println!("{line}");
            }
            println!("{}", fl!("new-open-hint", dir = dir.display().to_string()));
            0
        }
        Err(e) => {
            eprintln!("{}", fl!("new-failed", error = e));
            1
        }
    }
}

fn usage(arg: &str) -> i32 {
    let models: Vec<String> = models::all().into_iter().map(|m| m.id).collect();
    eprintln!("{}", fl!("new-usage", arg = arg.to_string(), models = models.join(", ")));
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-novo-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn reads_the_form() {
        let values: std::collections::BTreeMap<String, Value> =
            serde_json::from_value(json!({"/pasta": "/tmp/x", "/nome": "meu-app", "/modelo": "rust", "/descricao": " um app ", "/git": false, "/agente": "nenhum"})).unwrap();
        let r = request_from(&values).unwrap();
        assert_eq!(r, Request { dir: "/tmp/x/meu-app".into(), model: "rust".into(), description: "um app".into(), git: false, agent: None });
        let mut v = values.clone();
        v.insert("/agente".into(), json!("claude"));
        assert_eq!(request_from(&v).unwrap().agent.as_deref(), Some("claude"));
        v.insert("/agente".into(), json!("claude; rm -rf ~"));
        assert!(request_from(&v).is_err(), "o nome do agente vai num comando: só letras, dígitos, - e _");
        for bad in ["", "a/b", ".."] {
            let mut v = values.clone();
            v.insert("/nome".into(), json!(bad));
            assert!(request_from(&v).is_err(), "{bad}");
        }
        let mut v = values.clone();
        v.insert("/pasta".into(), json!("relativa"));
        assert!(request_from(&v).is_err());
        let doc = form(1, &initial_values(Path::new("/home/u/proj")), Some("deu ruim"));
        assert_eq!(doc["data"]["pasta"], "/home/u");
        let parsed = tea_declarative_ui::parse(doc.to_string().as_bytes(), &tea_declarative_ui::Limits::default());
        assert!(parsed.as_ref().is_ok_and(|d| d.warnings.is_empty()), "o formulário é um documento válido: {parsed:?}");
    }

    #[test]
    fn refuses_a_folder_with_files_and_cleans_up_on_failure() {
        let dir = temp("cheia");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "x").unwrap();
        let cache = temp("cache");
        let req = Request { dir: dir.clone(), model: "vazio".into(), description: String::new(), git: true, agent: None };
        assert!(create(&req, &cache).unwrap_err().contains(&dir.display().to_string()));
        assert!(dir.join("a.txt").exists(), "não mexe numa pasta com arquivos");
        // Modelo que não existe: nada é criado.
        let nova = temp("nova");
        assert!(create(&Request { dir: nova.clone(), model: "cobol".into(), ..req.clone() }, &cache).is_err());
        assert!(!nova.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_agent_is_asked_for_in_a_terminal_request() {
        let dir = temp("agente");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(write_agent_request(&dir, "a b").is_err());
        write_agent_request(&dir, "claude").unwrap();
        let agents = dir.join(".idetea/agents");
        assert_eq!(std::fs::read_to_string(agents.join(".gitignore")).unwrap(), "*\n");
        let req: Value = serde_json::from_str(&std::fs::read_to_string(agents.join("agente.json")).unwrap()).unwrap();
        assert_eq!(req["format"], "idetea-terminal");
        assert_eq!(req["version"], 1);
        assert_eq!(req["cwd"], ".");
        assert_eq!(req["command"], "claude 'Start with the initial request in AGENTS.md.'");
        assert!(!agents.join(".agente.json.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_initial_request_goes_in_agents_md() {
        let m = models::get("rust").unwrap();
        let head = agents_head("meu-app", &m, "um conversor de unidades");
        assert!(head.starts_with("# meu-app\n\n## Initial request\n"));
        assert!(head.contains("in the user's words: um conversor de unidades"));
        assert!(head.contains(&m.request));
    }
}
