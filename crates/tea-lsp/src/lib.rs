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
use std::sync::Arc;

pub use client::{Client, EventSink};
pub use text::{Position, Range, diff, shift_range};

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

/// Um símbolo de um arquivo (função, struct, método...), com os de dentro em `children`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    /// `SymbolKind` do LSP (6 = método, 12 = função, 23 = struct...).
    pub kind: u32,
    /// Ex.: a assinatura (`fn(&self) -> bool`).
    pub detail: Option<String>,
    /// Do começo da declaração (atributos, comentário) até o fim do corpo.
    pub range: Range,
    /// Só o nome.
    pub selection: Range,
    pub children: Vec<DocumentSymbol>,
}

/// Um símbolo achado no projeto (busca por nome).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSymbol {
    pub name: String,
    /// `SymbolKind` do LSP.
    pub kind: u32,
    /// Onde ele está declarado (`Conta`, `crate::banco`...), se o servidor disser.
    pub container: Option<String>,
    pub path: PathBuf,
    /// Posição do nome (ou do começo da declaração), em caracteres.
    pub position: Position,
}

/// O que foi procurado numa lista de lugares ([`Event::Locations`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lookup {
    /// Mais de uma definição (uma só vem como [`Event::Definition`]).
    Definition,
    Implementation,
    TypeDefinition,
    References,
}

/// Um trecho de um arquivo (posições em caracteres).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: PathBuf,
    pub range: Range,
}

/// Uma função (ou método) na árvore de chamadas. Posições em caracteres.
#[derive(Debug, Clone, PartialEq)]
pub struct CallItem {
    pub name: String,
    /// `SymbolKind` do LSP.
    pub kind: u32,
    /// Ex.: a assinatura ou onde está (`impl Conta`).
    pub detail: Option<String>,
    pub path: PathBuf,
    /// A declaração inteira.
    pub range: Range,
    /// Só o nome.
    pub selection: Range,
    /// O item como o servidor mandou: vai de volta em [`Client::calls`].
    raw: std::sync::Arc<serde_json::Value>,
}

/// Quem chama a função, ou quem ela chama.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallDirection {
    Incoming,
    Outgoing,
}

/// Uma ligação na árvore de chamadas.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// Quem chama (`Incoming`) ou quem é chamado (`Outgoing`).
    pub item: CallItem,
    /// Arquivo onde as chamadas estão: o de quem chama.
    pub sites_path: PathBuf,
    /// Onde as chamadas acontecem (o nome chamado), em caracteres.
    pub sites: Vec<Range>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Assinatura da função sendo chamada, com o parâmetro em que o cursor está.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelp {
    /// Ex.: `fn soma(a: i32, b: i32) -> i32`.
    pub label: String,
    /// Onde está o parâmetro atual no `label`, em caracteres (início, fim).
    pub active: Option<(usize, usize)>,
    /// Documentação do parâmetro atual (primeiro parágrafo), se houver.
    pub doc: Option<String>,
    /// Qual das assinaturas (sobrecargas) é esta, e quantas há.
    pub index: usize,
    pub count: usize,
}

/// Edições num arquivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEdits {
    pub path: PathBuf,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Error,
    Warning,
    Info,
}

/// Uma ação de código oferecida pelo servidor (correção rápida, refatoração, organizar imports). Quem escolhe uma
/// passa para [`Client::apply_code_action`]: as edições voltam em [`Event::ApplyEdit`].
#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    /// `quickfix`, `refactor.extract`, `source.organizeImports`... Vazio se o servidor não diz.
    pub kind: String,
    /// A ação que o servidor recomenda para o problema (ex.: o import que falta).
    pub preferred: bool,
    /// Por que não dá para aplicar agora. A ação aparece na lista, mas não faz nada.
    pub disabled: Option<String>,
    /// O que o servidor mandou (`CodeAction` ou `Command`), para resolver e executar depois.
    pub(crate) raw: Arc<serde_json::Value>,
}

/// Um texto que o servidor pede para mostrar dentro do código, sem fazer parte do arquivo: o tipo inferido de uma
/// variável, o nome do parâmetro antes de um argumento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHint {
    /// Onde o texto aparece (antes do caractere nessa coluna), em caracteres.
    pub position: Position,
    /// O texto, com as partes já juntas.
    pub label: String,
    pub kind: InlayHintKind,
    /// Um espaço antes / depois do texto.
    pub padding_left: bool,
    pub padding_right: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlayHintKind {
    Type,
    Parameter,
    Other,
}

/// Um comando do servidor (`workspace/executeCommand`), a executar depois das edições de uma ação.
#[derive(Debug, Clone)]
pub struct ServerCommand {
    pub title: String,
    pub(crate) raw: Arc<serde_json::Value>,
}

/// Algo que veio do servidor. Posições já em caracteres.
#[derive(Debug, Clone)]
pub enum Event {
    Diagnostics { server: String, path: PathBuf, diagnostics: Vec<Diagnostic> },
    Hover { path: PathBuf, text: String },
    Completion { path: PathBuf, items: Vec<CompletionItem> },
    /// Resposta da dica de parâmetros. `None`: o cursor não está numa chamada (esconder a dica).
    SignatureHelp { path: PathBuf, help: Option<SignatureHelp> },
    Definition { path: PathBuf, position: Position },
    /// Nenhuma definição encontrada (para avisar o usuário).
    DefinitionNotFound,
    /// Pediu a definição em cima da própria declaração (`path`/`position` são de onde pediu).
    /// Quem usa costuma listar as referências, como o VS Code faz no Ctrl+clique.
    AtDefinition { path: PathBuf, position: Position },
    /// Lista de lugares para o símbolo em `path`/`position`: onde é usado (inclui a declaração),
    /// implementações, o tipo, ou várias definições. Ordenada por arquivo e posição; vazia se nada.
    Locations { lookup: Lookup, path: PathBuf, position: Position, locations: Vec<Location> },
    Formatting { path: PathBuf, edits: Vec<TextEdit> },
    /// Edições para renomear um símbolo, por arquivo (vazia: nada para renomear). Posições em caracteres,
    /// em relação ao texto que o servidor conhece (o da aba aberta, ou o do disco).
    Rename { new_name: String, files: Vec<FileEdits> },
    /// O servidor recusou renomear (a mensagem dele explica).
    RenameFailed { message: String },
    /// Ações de código para a seleção (ou o cursor) em `path`, a preferida primeiro. Vazia: nenhuma ali.
    CodeActions { path: PathBuf, actions: Vec<CodeAction> },
    /// Edições a aplicar agora: de uma ação escolhida, ou pedidas pelo servidor (`workspace/applyEdit`, ex.: ao
    /// executar um comando). Depois de aplicar, `command` (se houver) vai para [`Client::execute_command`].
    ApplyEdit { label: String, files: Vec<FileEdits>, command: Option<ServerCommand> },
    /// Inlay hints de `path` no intervalo pedido em [`Client::inlay_hints`] (`token` é o que foi passado lá).
    /// Vazia se não há nenhum ou o servidor não souber responder.
    InlayHints { path: PathBuf, token: u64, hints: Vec<InlayHint> },
    /// O servidor pede para pedir os inlay hints de novo (ex.: o rust-analyzer, quando termina de analisar).
    InlayHintsRefresh { server: String },
    /// Símbolos do projeto que combinam com `query` (vazia se o servidor não souber responder).
    WorkspaceSymbols { query: String, symbols: Vec<WorkspaceSymbol> },
    /// A função em `path`/`position`, para começar a árvore de chamadas (vazia se o cursor não
    /// está numa função ou o servidor não souber responder).
    CallRoots { path: PathBuf, position: Position, items: Vec<CallItem> },
    /// Ligações de um item da árvore de chamadas (`token` é o que foi passado em [`Client::calls`]).
    Calls { token: u64, direction: CallDirection, calls: Vec<Call> },
    /// Estrutura do arquivo (vazia se o servidor não souber responder).
    DocumentSymbols { path: PathBuf, symbols: Vec<DocumentSymbol> },
    Progress { server: String, title: String, message: Option<String>, percentage: Option<u32>, done: bool },
    Message { server: String, level: MessageLevel, text: String },
    Log { server: String, text: String },
    /// O servidor terminou o handshake e está pronto.
    Ready { server: String },
    /// O processo do servidor terminou. `client`: o [`Client::id`] de quem terminou (um cliente antigo, já
    /// substituído por um reinício, também avisa quando sai).
    Exited { server: String, client: u64, reason: String },
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
        cfg("zls", "zls", &[], &[("zig", "zig"), ("zon", "zig")]),
        // Ziex (Zig com marcação): o `zx lsp` cuida da marcação e repassa o Zig a um ZLS embutido. Sem o `zx` no
        // PATH, a IDE compila o servidor do projeto (ver `tea-app::lsp::ziex_fallback`).
        cfg("zxls", "zx", &["lsp"], &[("zx", "zx"), ("mdzx", "mdzx")]),
    ]
}

/// Procura um executável no PATH (no Windows, considera também `.exe`, `.cmd`, `.bat` e `.com`).
pub fn find_in_path(command: &str) -> Option<PathBuf> {
    let cmd = std::path::Path::new(command);
    if cmd.components().count() > 1 {
        return cmd.is_file().then(|| cmd.to_path_buf());
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        if cmd.extension().is_none() {
            let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string());
            for ext in exts.split(';').filter(|e| !e.is_empty()) {
                let with_ext = dir.join(format!("{command}{ext}"));
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }
    }
    None
}
