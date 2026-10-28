//! Teste manual contra um servidor real:
//! `cargo run -p tea-lsp --example smoke -- [clangd|rust-analyzer]`
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use tea_lsp::{Client, Event, Position, default_servers};

fn main() {
    let server = std::env::args().nth(1).unwrap_or_else(|| "clangd".into());
    let dir = std::env::temp_dir().join(format!("tea-lsp-smoke-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    // Linha 0 tem acento e emoji: testa a conversão de colunas.
    let (file, lang, text, call, member) = if server == "clangd" {
        (dir.join("main.c"), "c",
         "// ação 😀\n#include <stdlib.h>\nint soma(int a, int b) { return a + b; }\nstruct P { int x; int y; };\nint main(void) {\n    char *s = soma(1, 2);\n    struct P p;\n    p.\n}\n",
         Position::new(5, 15), Position::new(7, 6))
    } else {
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"smoke\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        (dir.join("src/main.rs"), "rust",
         "// ação 😀\nfn soma(a: i32, b: i32) -> i32 { a + b }\nfn main() {\n    let x: String = soma(1, 2);\n    let v: Vec<i32> = Vec::new();\n    v.\n}\n",
         Position::new(3, 21), Position::new(5, 6))
    };
    std::fs::write(&file, text).unwrap();

    let cfg = default_servers().into_iter().find(|s| s.name == server).expect("servidor conhecido");
    let (tx, rx) = mpsc::channel();
    let t0 = Instant::now();
    let client = Client::start(&cfg, &dir, Arc::new(move |e| { let _ = tx.send(e); })).expect("iniciar servidor");
    client.did_open(&file, lang, 1, text);
    client.hover(&file, call);
    client.definition(&file, call);

    let mut asked_completion = false;
    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        let Ok(e) = rx.recv_timeout(Duration::from_millis(500)) else { continue };
        let t = t0.elapsed();
        match &e {
            Event::Log { .. } | Event::Progress { .. } => continue,
            Event::Diagnostics { diagnostics, .. } => {
                if diagnostics.is_empty() { continue }
                for d in diagnostics {
                    println!("[{t:>8.2?}] DIAG {:?} {}:{} {}", d.severity, d.range.start.line + 1,
                        d.range.start.character + 1, d.message.lines().next().unwrap_or(""));
                }
            }
            Event::Hover { text, .. } => println!("[{t:>8.2?}] HOVER {:?}", text.replace('\n', " ⏎ ")),
            Event::Completion { items, .. } => {
                let names: Vec<_> = items.iter().take(6).map(|i| i.label.as_str()).collect();
                println!("[{t:>8.2?}] COMPLETION {} itens: {names:?}", items.len());
                break;
            }
            other => println!("[{t:>8.2?}] {other:?}"),
        }
        if !asked_completion && matches!(e, Event::Diagnostics { .. } | Event::Definition { .. }) {
            asked_completion = true;
            client.completion(&file, member);
        }
        if matches!(e, Event::Exited { .. }) { break }
    }
    client.shutdown();
    std::thread::sleep(Duration::from_millis(300));
    let _ = std::fs::remove_dir_all(&dir);
}
