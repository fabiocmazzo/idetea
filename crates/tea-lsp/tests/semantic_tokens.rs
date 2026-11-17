//! Semantic tokens com servidores de verdade: parâmetro x variável x função (clangd) e variável mutável com
//! acentos antes (rust-analyzer). Pula o que não estiver instalado.

use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use tea_lsp::{Client, Event, EventSink, Position, SemanticToken, ServerConfig};

#[test]
fn clangd_parameter_variable_function() {
    if !runs("clangd") {
        eprintln!("sem clangd: teste pulado");
        return;
    }
    let src = "int add(int left, int right) { int sum = left + right; return sum; }\n";
    let dir = temp_dir("semantic-c");
    let file = dir.join("t.c");
    std::fs::write(&file, src).unwrap();
    let (client, rx) = start("clangd", &dir, ("c", "c"));
    let _guard = KillOnDrop(client.clone());
    client.did_open(&file, "c", 1, src);
    let tokens = tokens_until(&client, &rx, &file, |t| t.iter().any(|t| t.kind == "parameter"));
    let at = |col: u32| tokens.iter().find(|t| t.range.start == Position::new(0, col)).unwrap_or_else(|| panic!("{col}: {tokens:?}"));
    assert_eq!(at(4).kind, "function", "add: {tokens:?}");
    assert_eq!(at(12).kind, "parameter", "left: {tokens:?}");
    assert_eq!(at(12).range.end, Position::new(0, 16), "{tokens:?}");
    assert_eq!(at(35).kind, "variable", "sum: {tokens:?}");
    assert!(at(35).modifiers.iter().any(|m| m == "declaration"), "{tokens:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rust_analyzer_mutable_variable() {
    if !runs("rust-analyzer") {
        eprintln!("sem rust-analyzer: teste pulado");
        return;
    }
    let dir = temp_dir("semantic-ra");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
    // Acentos antes da variável: a coluna volta em caracteres, não em UTF-16 nem bytes.
    let src = "fn main() {\n    /* ação */ let mut total = 2u32;\n    total += 1;\n    drop(total);\n}\n";
    let file = dir.join("src/main.rs");
    std::fs::write(&file, src).unwrap();
    let (client, rx) = start("rust-analyzer", &dir, ("rs", "rust"));
    let _guard = KillOnDrop(client.clone());
    client.did_open(&file, "rust", 1, src);
    // A primeira resposta pode vir antes de o projeto carregar, com o `total` sem `mutable` e os usos dele como
    // `const`: espera o da declaração.
    let is_total = |t: &SemanticToken| t.range.start == Position::new(1, 23);
    let tokens =
        tokens_until(&client, &rx, &file, |t| t.iter().any(|t| is_total(t) && t.modifiers.iter().any(|m| m == "mutable")));
    let total = tokens.iter().find(|t| is_total(t)).unwrap_or_else(|| panic!("{tokens:?}"));
    assert_eq!(total.kind, "variable", "{tokens:?}");
    assert_eq!(total.range.end, Position::new(1, 28), "{tokens:?}");
    assert!(total.modifiers.iter().any(|m| m == "mutable"), "{tokens:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Pede os tokens do arquivo até `done` (o servidor pode ainda estar carregando o projeto).
fn tokens_until(
    client: &Client,
    rx: &mpsc::Receiver<Event>,
    file: &Path,
    done: impl Fn(&[SemanticToken]) -> bool,
) -> Vec<SemanticToken> {
    let deadline = std::time::Instant::now() + Duration::from_secs(90);
    for token in 1.. {
        assert!(std::time::Instant::now() < deadline, "tempo esgotado esperando os tokens");
        client.semantic_tokens(file, token);
        loop {
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(Event::SemanticTokens { token: t, tokens, .. }) if t == token => {
                    if done(&tokens) {
                        return tokens;
                    }
                    break;
                }
                Ok(_) => continue,
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
