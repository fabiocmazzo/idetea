//! Plugin de exemplo: um painel com um contador, um comando para zerar, a contagem dos arquivos da raiz do
//! projeto (lida pelo WASI, em `/project`) e um comando externo configurável (`host.run` + `host.config`):
//!
//! ```toml
//! [plugins.contador]
//! comando = ["git", "status", "--short"]   # o padrão
//! ```

// O código gerado pelo wit-bindgen tem funções com muitos argumentos.
#![allow(clippy::too_many_arguments)]

wit_bindgen::generate!({ world: "plugin", path: "../../../crates/tea-plugins/wit" });

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use idetea::plugin::host;
use idetea::plugin::types::Command;

static COUNT: AtomicU32 = AtomicU32::new(0);
static SAVES: AtomicU32 = AtomicU32::new(0);
/// Id do último `host.run` (para ignorar um resultado velho).
static RUN: AtomicU32 = AtomicU32::new(0);
/// O que mostrar do comando externo.
static OUTPUT: Mutex<String> = Mutex::new(String::new());

/// `comando` da config (lista de textos); sem ele, `git status --short`.
fn configured_command() -> Vec<String> {
    let config: serde_json::Value = serde_json::from_str(&host::config()).unwrap_or_default();
    let list: Option<Vec<String>> = config.get("comando").and_then(|c| c.as_array()).map(|a| {
        a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
    });
    match list {
        Some(l) if !l.is_empty() => l,
        _ => ["git", "status", "--short"].map(String::from).to_vec(),
    }
}

fn set_output(text: String) {
    *OUTPUT.lock().unwrap() = text;
}

struct Contador;

fn files_in_root() -> usize {
    std::fs::read_dir("/project").map(|d| d.count()).unwrap_or(0)
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Redesenha o painel. A `revision` muda a cada vez para os botões religarem.
fn render() {
    let count = COUNT.load(Ordering::Relaxed);
    let saves = SAVES.load(Ordering::Relaxed);
    let info = format!("{} itens na raiz do projeto; {saves} arquivos salvos desde que o plugin abriu.", files_in_root());
    let command = format!("Comando: {}", configured_command().join(" "));
    let output = OUTPUT.lock().unwrap().clone();
    let doc = format!(
        r#"{{"format":"idetea-ui","version":1,"revision":{rev},"submit":"many","title":"Contador",
        "components":[
          {{"id":"valor","component":"text","text":{valor}}},
          {{"id":"info","component":"text","text":{info},"style":"muted"}},
          {{"id":"mais","component":"button","label":"+1","action":{{"name":"mais"}}}},
          {{"id":"zerar","component":"button","label":"Zerar","style":"secondary","action":{{"name":"zerar"}}}},
          {{"id":"comando","component":"text","text":{command},"style":"muted"}},
          {{"id":"rodar","component":"button","label":"Rodar","style":"secondary","action":{{"name":"rodar"}}}},
          {{"id":"saida","component":"text","text":{output}}}
        ]}}"#,
        rev = count + saves * 1000 + RUN.load(Ordering::Relaxed) * 1_000_000,
        command = json_string(&command),
        output = json_string(&output),
        valor = json_string(&format!("Valor: {count}")),
        info = json_string(&info),
    );
    host::set_panel(&doc);
}

/// Roda o comando da config sem esperar: a saída chega em `Event::RunFinished`.
fn run() {
    let mut words = configured_command().into_iter();
    let program = words.next().unwrap_or_default();
    let id = RUN.fetch_add(1, Ordering::Relaxed) + 1;
    set_output(format!("Rodando {program}..."));
    host::run(id, &Command { program, args: words.collect(), cwd: None, stdin: None, timeout_ms: 5000 });
    render();
}

impl Guest for Contador {
    fn activate() -> Result<(), String> {
        host::register_command("contador.zerar", "Contador: zerar");
        host::register_command("contador.mostrar", "Contador: mostrar o painel");
        host::register_command("contador.rodar", "Contador: rodar o comando da config");
        // Demonstração do prazo: a IDE interrompe o plugin e o desliga, sem travar.
        host::register_command("contador.travar", "Contador: travar (demonstra o prazo)");
        host::log("contador ativado");
        render();
        Ok(())
    }

    fn on_command(id: String) {
        match id.as_str() {
            "contador.zerar" => {
                COUNT.store(0, Ordering::Relaxed);
                host::notify("Contador zerado.");
                render();
            }
            "contador.mostrar" => host::show_panel(),
            "contador.rodar" => run(),
            "contador.travar" => {
                #[allow(clippy::empty_loop)]
                loop {}
            }
            _ => {}
        }
    }

    fn on_event(event: Event) {
        match event {
            Event::FileSaved(_) => {
                SAVES.fetch_add(1, Ordering::Relaxed);
            }
            Event::RunFinished(out) if out.id == RUN.load(Ordering::Relaxed) => {
                let text = match (&out.error, out.status) {
                    (Some(e), _) => format!("Erro: {e}"),
                    (None, Some(0)) => {
                        let lines: Vec<&str> = out.stdout.lines().take(20).collect();
                        if lines.is_empty() { "Terminou sem saída.".into() } else { lines.join("\n") }
                    }
                    (None, status) => format!("Saiu com {status:?}: {}", out.stderr.trim()),
                };
                set_output(text);
            }
            Event::ConfigChanged => host::log("config mudou"),
            _ => return,
        }
        render();
    }

    fn on_action(action: UiAction) {
        match action.name.as_str() {
            "mais" => {
                COUNT.fetch_add(1, Ordering::Relaxed);
            }
            "zerar" => COUNT.store(0, Ordering::Relaxed),
            "rodar" => return run(),
            _ => {}
        }
        render();
    }
}

export!(Contador);
