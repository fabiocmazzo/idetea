//! Referências e "já estou na declaração" com um servidor de verdade (clangd). Pula sem clangd.

use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::Duration;

use tea_lsp::{Client, Event, EventSink, Position, ServerConfig};

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

    client.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
}
