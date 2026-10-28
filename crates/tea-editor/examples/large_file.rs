//! Mede o custo das operações do editor com arquivos grandes (sem renderizar).
//!
//! `cargo run --release -p tea-editor --example large_file -- [linhas] [--write arquivo.rs]`
//!
//! A renderização e o scroll precisam ser testados à mão no app:
//! `cargo run --release -- arquivo.rs`

use std::time::Instant;

use iced_code_editor::{ArrowDirection, CodeEditor, Message};

fn synthetic_rust(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 40);
    let mut n = 0;
    while n < lines {
        s.push_str(&format!("/// Função gerada número {n}.\n"));
        s.push_str(&format!("pub fn func_{n}(x: u64) -> u64 {{\n"));
        s.push_str("    let mut acc = x; // comentário\n");
        s.push_str("    for i in 0..10 { acc = acc.wrapping_mul(31).wrapping_add(i); }\n");
        s.push_str("    if acc % 2 == 0 { acc / 2 } else { \"ímpar\".len() as u64 }\n");
        s.push_str("}\n\n");
        n += 7;
    }
    s
}

fn time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let out = f();
    println!("{label:<40} {:>10.2?}", t.elapsed());
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let lines: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(50_000);
    let write_to = match (args.next().as_deref(), args.next()) {
        (Some("--write"), Some(path)) => Some(path),
        _ => None,
    };

    let text = synthetic_rust(lines);
    println!("{} linhas, {:.1} MB\n", text.lines().count(), text.len() as f64 / 1e6);
    if let Some(path) = write_to {
        std::fs::write(&path, &text).expect("falha ao gravar");
        println!("gravado em {path}\n");
    }

    time("CodeEditor::new com 1 linha (custo fixo)", || CodeEditor::new("fn x() {}", "rs"));
    time("CodeEditor::new com 1 linha (2ª vez)", || CodeEditor::new("fn x() {}", "rs"));
    let mut ed = time("CodeEditor::new (abrir)", || CodeEditor::new(&text, "rs"));
    ed.request_focus();
    let _ = ed.update(&Message::CanvasFocusGained);
    let before = ed.content().len();
    time("content() (salvar)", || ed.content().len());
    time("ir para o meio (GotoPosition)", || {
        let _ = ed.update(&Message::GotoPosition(lines / 2, 0));
    });
    time("digitar 1000 caracteres no meio", || {
        for c in "let y = 1; ".chars().cycle().take(1000) {
            let _ = ed.update(&Message::CharacterInput(c));
        }
    });
    println!("  -> tamanho depois de digitar: {}", ed.content().len());
    time("1000 setas para baixo", || {
        for _ in 0..1000 {
            let _ = ed.update(&Message::ArrowKey(ArrowDirection::Down, false));
        }
    });
    time("100 PageDown", || {
        for _ in 0..100 {
            let _ = ed.update(&Message::PageDown(false));
        }
    });
    time("Ctrl+End", || {
        let _ = ed.update(&Message::CtrlEnd);
    });
    time("buscar \"wrapping_mul\"", || {
        let _ = ed.update(&Message::OpenSearch);
        let _ = ed.update(&Message::SearchQueryChanged("wrapping_mul".into()));
    });
    time("undo de tudo que foi digitado", || {
        let _ = ed.update(&Message::CloseSearch);
        while ed.can_undo() {
            let _ = ed.update(&Message::Undo);
        }
    });
    let after_typing = ed.content().len();
    time("content() depois das edições", || ed.content().len());
    println!("\ntamanho: original {before}, final {after_typing} (undo deve voltar ao original)");
}
