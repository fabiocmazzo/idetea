//! Ações de código com um servidor de verdade (clangd): o erro de um `;` que falta, a correção oferecida para ele e
//! as edições que ela traz. Pula sem clangd.

use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::Duration;

use tea_lsp::{Client, Event, EventSink, Position, Range, ServerConfig};

const SRC: &str = "int main(void) {\n    int x = 1\n    return x;\n}\n";

#[test]
fn clangd_quick_fix_for_missing_semicolon() {
    if !runs("clangd") {
        eprintln!("sem clangd: teste pulado");
        return;
    }
    let dir = std::env::temp_dir().join(format!("idetea-actions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = PathBuf::from(dir.canonicalize().unwrap().to_string_lossy().trim_start_matches(r"\\?\").to_string());
    let file: PathBuf = dir.join("t.c");
    std::fs::write(&file, SRC).unwrap();

    let (tx, rx) = mpsc::channel();
    let sink: EventSink = Arc::new(move |e| {
        let _ = tx.send(e);
    });
    let config = ServerConfig {
        name: "clangd".into(),
        command: "clangd".into(),
        args: vec![],
        languages: vec![("c".into(), "c".into())],
    };
    let client = Client::start(&config, &dir, sink).unwrap();
    struct KillOnDrop(Client);
    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            self.0.kill_now();
        }
    }
    let _guard = KillOnDrop(client.clone());
    client.did_open(&file, "c", 1, SRC);
    let wait = |pred: &dyn Fn(&Event) -> bool| loop {
        match rx.recv_timeout(Duration::from_secs(20)) {
            Ok(e) if pred(&e) => return e,
            Ok(_) => continue,
            Err(_) => panic!("tempo esgotado esperando o clangd"),
        }
    };

    // O erro vem antes: sem ele no pedido, o clangd não oferece a correção.
    let Event::Diagnostics { diagnostics, .. } = wait(&|e| matches!(e, Event::Diagnostics { diagnostics, .. } if !diagnostics.is_empty()))
    else {
        unreachable!()
    };
    let error = diagnostics[0].range;

    // Cursor no erro (fim de `int x = 1`).
    client.code_actions(&file, Range { start: error.start, end: error.start });
    let Event::CodeActions { actions, .. } = wait(&|e| matches!(e, Event::CodeActions { .. })) else { unreachable!() };
    let fix = actions.iter().find(|a| a.kind == "quickfix").unwrap_or_else(|| panic!("sem correção: {actions:?}"));
    assert!(fix.title.contains(';'), "{}", fix.title);

    client.apply_code_action(fix);
    let Event::ApplyEdit { files, .. } = wait(&|e| matches!(e, Event::ApplyEdit { .. })) else { unreachable!() };
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, file);
    let edit = &files[0].edits[0];
    assert_eq!(edit.new_text, ";");
    assert_eq!(edit.range.start, Position::new(1, 13), "logo depois de `int x = 1`");

    // Fora de qualquer erro: nenhuma correção.
    client.code_actions(&file, Range { start: Position::new(3, 0), end: Position::new(3, 0) });
    let Event::CodeActions { actions, .. } = wait(&|e| matches!(e, Event::CodeActions { .. })) else { unreachable!() };
    assert!(actions.iter().all(|a| a.kind != "quickfix"), "{actions:?}");
    let _ = std::fs::remove_dir_all(&dir);
}


/// rust-analyzer manda a lista sem as edições: elas vêm com `codeAction/resolve` depois da escolha. Pula sem ele.
#[test]
fn rust_analyzer_resolves_the_chosen_action() {
    if !runs("rust-analyzer") {
        eprintln!("sem rust-analyzer: teste pulado");
        return;
    }
    let dir = std::env::temp_dir().join(format!("idetea-actions-ra-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let dir = PathBuf::from(dir.canonicalize().unwrap().to_string_lossy().trim_start_matches(r"\\?\").to_string());
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
    let src = "fn main() {\n    let m: HashMap<u8, u8> = Default::default();\n    drop(m);\n}\n";
    let file = dir.join("src/main.rs");
    std::fs::write(&file, src).unwrap();

    let (tx, rx) = mpsc::channel();
    let sink: EventSink = Arc::new(move |e| {
        let _ = tx.send(e);
    });
    let config = ServerConfig {
        name: "rust-analyzer".into(),
        command: "rust-analyzer".into(),
        args: vec![],
        languages: vec![("rs".into(), "rust".into())],
    };
    let client = Client::start(&config, &dir, sink).unwrap();
    struct KillOnDrop(Client);
    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            self.0.kill_now();
        }
    }
    let _guard = KillOnDrop(client.clone());
    client.did_open(&file, "rust", 1, src);
    let wait = |pred: &dyn Fn(&Event) -> bool| loop {
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(e) if pred(&e) => return e,
            Ok(e) => eprintln!("evento: {e:?}"),
            Err(_) => panic!("tempo esgotado esperando o rust-analyzer"),
        }
    };
    let at = Position::new(1, 12);
    let fix = loop {
        client.code_actions(&file, Range { start: at, end: at });
        // Ainda indexando: o servidor responde "conteúdo mudou" (vira aviso) ou uma lista sem o import.
        if let Event::CodeActions { actions, .. } = wait(&|e| matches!(e, Event::CodeActions { .. } | Event::Message { .. }))
            && let Some(a) = actions.into_iter().find(|a| a.title.contains("std::collections::HashMap"))
        {
            break a;
        }
        std::thread::sleep(Duration::from_millis(500)); // ainda indexando
    };
    client.apply_code_action(&fix);
    let e = wait(&|e| matches!(e, Event::ApplyEdit { .. } | Event::Message { .. }));
    let Event::ApplyEdit { files, .. } = e else { panic!("{e:?}") };
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].edits.iter().any(|e| e.new_text.contains("use std::collections::HashMap;")), "{files:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// O servidor existe e responde (o `rust-analyzer` do rustup existe sem o componente instalado e falha).
fn runs(command: &str) -> bool {
    std::process::Command::new(command).arg("--version").output().is_ok_and(|o| o.status.success())
}
