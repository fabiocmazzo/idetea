//! Inlay hints com servidores de verdade: os nomes dos parâmetros numa chamada (clangd) e o tipo inferido de uma
//! variável (rust-analyzer). Pula o que não estiver instalado.

use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use tea_lsp::{Client, Event, EventSink, InlayHint, InlayHintKind, Position, Range, ServerConfig};

#[test]
fn clangd_parameter_names() {
    if !runs("clangd") {
        eprintln!("sem clangd: teste pulado");
        return;
    }
    let src = "int add(int left, int right) { return left + right; }\nint main(void) { return add(1, 2); }\n";
    let dir = temp_dir("inlay-c");
    let file = dir.join("t.c");
    std::fs::write(&file, src).unwrap();
    let (client, rx) = start("clangd", &dir, ("c", "c"));
    let _guard = KillOnDrop(client.clone());
    client.did_open(&file, "c", 1, src);
    let hints = hints_until(&client, &rx, &file, |h| h.len() >= 2);
    let params: Vec<_> = hints.iter().filter(|h| h.kind == InlayHintKind::Parameter).collect();
    assert_eq!(params.len(), 2, "{hints:?}");
    assert!(params[0].label.starts_with("left"), "{hints:?}");
    // Antes do `1` em `return add(1, 2);`.
    assert_eq!(params[0].position, Position::new(1, 28), "{hints:?}");
    assert!(params[1].label.starts_with("right"), "{hints:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rust_analyzer_inferred_type() {
    if !runs("rust-analyzer") {
        eprintln!("sem rust-analyzer: teste pulado");
        return;
    }
    let dir = temp_dir("inlay-ra");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
    // Acentos antes da variável: a posição volta em caracteres, não em UTF-16 nem bytes.
    let src = "fn main() {\n    /* ação */ let total = 2u32 + 3;\n    drop(total);\n}\n";
    let file = dir.join("src/main.rs");
    std::fs::write(&file, src).unwrap();
    let (client, rx) = start("rust-analyzer", &dir, ("rs", "rust"));
    let _guard = KillOnDrop(client.clone());
    client.did_open(&file, "rust", 1, src);
    let hints = hints_until(&client, &rx, &file, |h| h.iter().any(|h| h.kind == InlayHintKind::Type));
    let ty = hints.iter().find(|h| h.kind == InlayHintKind::Type).unwrap();
    assert!(ty.label.contains("u32"), "{hints:?}");
    assert_eq!(ty.position, Position::new(1, 24), "logo depois de `total`: {hints:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Pede os hints do arquivo todo até `done` (o servidor pode ainda estar carregando o projeto).
fn hints_until(client: &Client, rx: &mpsc::Receiver<Event>, file: &Path, done: impl Fn(&[InlayHint]) -> bool) -> Vec<InlayHint> {
    let all = Range { start: Position::new(0, 0), end: Position::new(10, 0) };
    let deadline = std::time::Instant::now() + Duration::from_secs(90);
    for token in 1.. {
        assert!(std::time::Instant::now() < deadline, "tempo esgotado esperando os hints");
        client.inlay_hints(file, all, token);
        loop {
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(Event::InlayHints { token: t, hints, .. }) if t == token => {
                    if done(&hints) {
                        return hints;
                    }
                    break;
                }
                Ok(_) => continue,
                // O servidor ainda não disse que dá hints (handshake): o pedido não saiu.
                Err(_) => break,
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    unreachable!()
}

fn start(command: &str, dir: &Path, (ext, lang): (&str, &str)) -> (Client, mpsc::Receiver<Event>) {
    let (tx, rx) = mpsc::channel();
    let sink: EventSink = Arc::new(move |e| {
        let _ = tx.send(e);
    });
    let config = ServerConfig {
        name: command.into(),
        command: command.into(),
        args: vec![],
        languages: vec![(ext.into(), lang.into())],
    };
    // O pedido só sai depois do handshake (antes dele, o cliente não sabe se o servidor dá hints).
    let client = Client::start(&config, dir, sink).unwrap();
    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Event::Ready { .. }) => break,
            Ok(_) => continue,
            Err(_) => panic!("{command} não ficou pronto"),
        }
    }
    (client, rx)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("idetea-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    PathBuf::from(dir.canonicalize().unwrap().to_string_lossy().trim_start_matches(r"\\?\").to_string())
}

struct KillOnDrop(Client);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        self.0.kill_now();
    }
}

/// O servidor existe e responde (o `rust-analyzer` do rustup existe sem o componente instalado e falha).
fn runs(command: &str) -> bool {
    std::process::Command::new(command).arg("--version").output().is_ok_and(|o| o.status.success())
}
