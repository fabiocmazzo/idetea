//! Cliente LSP da IDE TEA. Não depende de UI.
//!
//! - Um [`Client`] por servidor (processo). Thread-safe: pode ser clonado e usado de qualquer lugar.
//! - Posições na API são em **caracteres** (0-based). A conversão para o servidor é interna.
//! - Tudo que vem do servidor chega como [`Event`] num callback.

mod client;
pub mod protocol;
pub mod text;
pub mod transport;
pub mod uri;

use std::path::PathBuf;

pub use client::{Client, EventSink};
pub use text::{Position, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Severity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    /// Texto a inserir (snippets já simplificados).
    pub insert: String,
    pub detail: Option<String>,
}

/// Um trecho de um arquivo (posições em caracteres).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: PathBuf,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Error,
    Warning,
    Info,
}

/// Algo que veio do servidor. Posições já em caracteres.
#[derive(Debug, Clone)]
pub enum Event {
    Diagnostics { server: String, path: PathBuf, diagnostics: Vec<Diagnostic> },
    Hover { path: PathBuf, text: String },
    Completion { path: PathBuf, items: Vec<CompletionItem> },
    Definition { path: PathBuf, position: Position },
    /// Nenhuma definição encontrada (para avisar o usuário).
    DefinitionNotFound,
    /// Pediu a definição em cima da própria declaração (`path`/`position` são de onde pediu).
    /// Quem usa costuma listar as referências, como o VS Code faz no Ctrl+clique.
    AtDefinition { path: PathBuf, position: Position },
    /// Onde o símbolo em `path`/`position` é usado (inclui a declaração). Vazio se nada.
    References { path: PathBuf, position: Position, locations: Vec<Location> },
    Formatting { path: PathBuf, edits: Vec<TextEdit> },
    Progress { server: String, title: String, message: Option<String>, percentage: Option<u32>, done: bool },
    Message { server: String, level: MessageLevel, text: String },
    Log { server: String, text: String },
    /// O servidor terminou o handshake e está pronto.
    Ready { server: String },
    Exited { server: String, reason: String },
}

/// Como iniciar um servidor e para quais arquivos ele serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// Nome curto, ex.: `"rust-analyzer"`.
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    /// Extensão de arquivo → languageId do LSP.
    pub languages: Vec<(String, String)>,
}

impl ServerConfig {
    pub fn language_for(&self, extension: &str) -> Option<&str> {
        let ext = extension.to_lowercase();
        self.languages.iter().find(|(e, _)| *e == ext).map(|(_, id)| id.as_str())
    }
}

/// Servidores conhecidos. Só são usados se o comando existir no PATH.
pub fn default_servers() -> Vec<ServerConfig> {
    fn cfg(name: &str, command: &str, args: &[&str], langs: &[(&str, &str)]) -> ServerConfig {
        ServerConfig {
            name: name.into(),
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            languages: langs.iter().map(|(e, l)| (e.to_string(), l.to_string())).collect(),
        }
    }
    vec![
        cfg("rust-analyzer", "rust-analyzer", &[], &[("rs", "rust")]),
        cfg(
            "clangd",
            "clangd",
            &[],
            &[("c", "c"), ("h", "c"), ("cc", "cpp"), ("cpp", "cpp"), ("cxx", "cpp"), ("hpp", "cpp"), ("hh", "cpp")],
        ),
        cfg("pyright", "pyright-langserver", &["--stdio"], &[("py", "python"), ("pyi", "python")]),
        cfg(
            "typescript",
            "typescript-language-server",
            &["--stdio"],
            &[("ts", "typescript"), ("tsx", "typescriptreact"), ("js", "javascript"), ("jsx", "javascriptreact")],
        ),
        cfg("gopls", "gopls", &[], &[("go", "go")]),
        cfg("lua", "lua-language-server", &[], &[("lua", "lua")]),
    ]
}

/// Procura um executável no PATH.
pub fn find_in_path(command: &str) -> Option<PathBuf> {
    let cmd = std::path::Path::new(command);
    if cmd.components().count() > 1 {
        return cmd.is_file().then(|| cmd.to_path_buf());
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|dir| dir.join(command)).find(|p| p.is_file())
}
