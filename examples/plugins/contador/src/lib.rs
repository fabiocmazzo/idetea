//! Plugin de exemplo: um painel com um contador, um comando para zerar e a contagem dos arquivos da raiz do
//! projeto (lida pelo WASI, em `/project`).

wit_bindgen::generate!({ world: "plugin", path: "../../../crates/tea-plugins/wit" });

use std::sync::atomic::{AtomicU32, Ordering};

use idetea::plugin::host;

static COUNT: AtomicU32 = AtomicU32::new(0);
static SAVES: AtomicU32 = AtomicU32::new(0);

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
    let doc = format!(
        r#"{{"format":"idetea-ui","version":1,"revision":{rev},"submit":"many","title":"Contador",
        "components":[
          {{"id":"valor","component":"text","text":{valor}}},
          {{"id":"info","component":"text","text":{info},"style":"muted"}},
          {{"id":"mais","component":"button","label":"+1","action":{{"name":"mais"}}}},
          {{"id":"zerar","component":"button","label":"Zerar","style":"secondary","action":{{"name":"zerar"}}}}
        ]}}"#,
        rev = count + saves * 1000,
        valor = json_string(&format!("Valor: {count}")),
        info = json_string(&info),
    );
    host::set_panel(&doc);
}

impl Guest for Contador {
    fn activate() -> Result<(), String> {
        host::register_command("contador.zerar", "Contador: zerar");
        host::register_command("contador.mostrar", "Contador: mostrar o painel");
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
            "contador.travar" => {
                #[allow(clippy::empty_loop)]
                loop {}
            }
            _ => {}
        }
    }

    fn on_event(event: Event) {
        if let Event::FileSaved(_) = event {
            SAVES.fetch_add(1, Ordering::Relaxed);
            render();
        }
    }

    fn on_action(action: UiAction) {
        match action.name.as_str() {
            "mais" => {
                COUNT.fetch_add(1, Ordering::Relaxed);
            }
            "zerar" => COUNT.store(0, Ordering::Relaxed),
            _ => {}
        }
        render();
    }
}

export!(Contador);
