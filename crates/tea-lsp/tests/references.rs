//! Referências e "já estou na declaração" com um servidor de verdade (clangd). Pula sem clangd.

use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::Duration;

use tea_lsp::{CallDirection, Client, Event, EventSink, Position, ServerConfig};

const SRC: &str = "int soma(int a, int b) {\n    return a + b;\n}\n\nint main(void) {\n    int x = soma(1, 2);\n    return soma(x, 3);\n}\n";

#[test]
fn clangd_references_and_at_definition() {
    if std::process::Command::new("clangd").arg("--version").output().is_err() {
        eprintln!("sem clangd: teste pulado");
        return;
    }
    let dir = std::env::temp_dir().join(format!("idetea-refs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    #[cfg(unix)]
    let dir = dir.canonicalize().unwrap();
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

    // `soma` no uso da linha 6 (0-based 5), coluna 12: a declaração e os dois usos.
    client.references(&file, Position::new(5, 13));
    let Event::Locations { locations, .. } = wait(&|e| matches!(e, Event::Locations { .. })) else { unreachable!() };
    let lines: Vec<(u32, u32)> = locations.iter().map(|l| (l.range.start.line, l.range.start.character)).collect();
    assert_eq!(lines, [(0, 4), (5, 12), (6, 11)], "{locations:?}");

    // Definição pedida em cima da própria declaração.
    client.definition(&file, Position::new(0, 6));
    let e = wait(&|e| matches!(e, Event::AtDefinition { .. } | Event::Definition { .. }));
    assert!(matches!(e, Event::AtDefinition { position, .. } if position == Position::new(0, 6)), "{e:?}");

    // E num uso, vai para a declaração.
    client.definition(&file, Position::new(6, 12));
    let e = wait(&|e| matches!(e, Event::AtDefinition { .. } | Event::Definition { .. }));
    assert!(matches!(e, Event::Definition { position, .. } if position.line == 0), "{e:?}");

    // Estrutura do arquivo, com as posições do nome.
    client.document_symbols(&file);
    let Event::DocumentSymbols { symbols, .. } = wait(&|e| matches!(e, Event::DocumentSymbols { .. })) else {
        unreachable!()
    };
    let names: Vec<_> = symbols.iter().map(|s| (s.name.as_str(), s.selection.start)).collect();
    assert_eq!(names, [("soma", Position::new(0, 4)), ("main", Position::new(4, 4))], "{symbols:?}");

    // Árvore de chamadas: `soma` é chamada duas vezes por `main`; `main` chama `soma`.
    client.prepare_call_hierarchy(&file, Position::new(0, 5));
    let Event::CallRoots { items, .. } = wait(&|e| matches!(e, Event::CallRoots { .. })) else { unreachable!() };
    assert_eq!(items.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(), ["soma"], "{items:?}");
    client.calls(&items[0], CallDirection::Incoming, 7);
    let Event::Calls { token, calls, .. } = wait(&|e| matches!(e, Event::Calls { .. })) else { unreachable!() };
    assert_eq!(token, 7);
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0].item.name, "main");
    let sites: Vec<_> = calls[0].sites.iter().map(|r| r.start).collect();
    assert_eq!(sites, [Position::new(5, 12), Position::new(6, 11)]);

    client.prepare_call_hierarchy(&file, Position::new(4, 5));
    let Event::CallRoots { items, .. } = wait(&|e| matches!(e, Event::CallRoots { .. })) else { unreachable!() };
    client.calls(&items[0], CallDirection::Outgoing, 8);
    let Event::Calls { calls, .. } = wait(&|e| matches!(e, Event::Calls { .. })) else { unreachable!() };
    assert_eq!(calls.iter().map(|c| c.item.name.as_str()).collect::<Vec<_>>(), ["soma"], "{calls:?}");
    assert_eq!(calls[0].sites_path, file);

    // Renomear `soma`: a declaração e os dois usos, só neste arquivo; nada muda antes de aplicar.
    client.rename(&file, Position::new(5, 13), "somar");
    let Event::Rename { new_name, files } = wait(&|e| matches!(e, Event::Rename { .. } | Event::RenameFailed { .. })) else {
        panic!("o clangd recusou renomear")
    };
    assert_eq!(new_name, "somar");
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!(files[0].path, file);
    let mut starts: Vec<_> = files[0].edits.iter().map(|e| (e.range.start, e.new_text.as_str())).collect();
    starts.sort();
    assert_eq!(starts, [(Position::new(0, 4), "somar"), (Position::new(5, 12), "somar"), (Position::new(6, 11), "somar")]);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), SRC);

    // Num lugar sem nome (o `{`): o servidor recusa.
    client.rename(&file, Position::new(0, 23), "x");
    let e = wait(&|e| matches!(e, Event::Rename { .. } | Event::RenameFailed { .. }));
    let refused = match &e {
        Event::RenameFailed { .. } => true,
        Event::Rename { files, .. } => files.is_empty(),
        _ => false,
    };
    assert!(refused, "{e:?}");

    // Dica de parâmetros em `soma(1, 2)`: logo depois do `(` é o `a`; depois da vírgula, o `b`.
    assert!(client.signature_triggers().contains(&'('), "{:?}", client.signature_triggers());
    let active = |column: u32| {
        client.signature_help(&file, Position::new(5, column));
        let Event::SignatureHelp { help, .. } = wait(&|e| matches!(e, Event::SignatureHelp { .. })) else { unreachable!() };
        let help = help.expect("dica dentro da chamada");
        let (s, e) = help.active.expect("parâmetro atual");
        help.label.chars().skip(s).take(e - s).collect::<String>()
    };
    assert_eq!(active(17), "int a");
    assert_eq!(active(20), "int b");
    // Fora de uma chamada: nada.
    client.signature_help(&file, Position::new(1, 4));
    let e = wait(&|e| matches!(e, Event::SignatureHelp { .. }));
    assert!(matches!(e, Event::SignatureHelp { help: None, .. }), "{e:?}");

    client.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
}
