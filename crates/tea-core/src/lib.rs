//! Núcleo da IDE TEA: a API que toda extensão usa.
//!
//! O núcleo não conhece nenhuma extensão específica. Extensões falam com o
//! app só por [`Context`] (pedidos), [`CoreEvent`] (notificações) e comandos.

use std::any::Any;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::{Element, Task};

pub mod config;
pub mod keymap;

/// Onde um painel aparece na janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Slot {
    Left,
    Right,
    Bottom,
    /// Faixa fina no rodapé da janela (barra de status).
    Status,
}

/// Identifica um painel dentro da extensão que o registrou.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PanelId(pub &'static str);

/// Algo que aconteceu no núcleo e que as extensões podem querer saber.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    FileOpened(PathBuf),
    FileSaved(PathBuf),
    ActiveFileChanged(Option<PathBuf>),
    /// Estado do editor ativo mudou (cursor, linguagem, modificado...).
    EditorStatus(EditorStatus),
    /// Mensagem curta para o usuário (vinda do núcleo ou de outra extensão).
    Notification(String),
    /// Config recarregado (ex.: depois de salvar o config.toml).
    ConfigReloaded,
}

/// Resumo do editor ativo, para barras de status e afins.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorStatus {
    pub path: Option<PathBuf>,
    /// Linha e coluna, começando em 1.
    pub line: usize,
    pub column: usize,
    pub language: String,
    pub modified: bool,
    pub encoding: &'static str,
    pub line_ending: &'static str,
}

/// Pedido de uma extensão para o app. O app executa depois que a chamada retorna.
#[derive(Debug, Clone)]
pub enum Request {
    OpenFile(PathBuf),
    RegisterPanel { id: PanelId, title: String, slot: Slot },
    RegisterCommand { id: String, title: String },
    RunCommand(String),
    Notify(String),
    /// Muda a visibilidade de um painel desta extensão. `None` alterna.
    PanelVisibility { id: PanelId, visible: Option<bool> },
    /// A extensão renomeou um arquivo ou pasta no disco: o app atualiza as abas.
    PathRenamed { from: PathBuf, to: PathBuf },
    /// A extensão apagou um arquivo ou pasta no disco: o app fecha as abas sem alterações.
    PathDeleted(PathBuf),
}

/// O que uma extensão pode ler e pedir durante uma chamada.
pub struct Context<'a> {
    root: &'a Path,
    active_file: Option<&'a Path>,
    requests: Vec<Request>,
}

impl<'a> Context<'a> {
    pub fn new(root: &'a Path, active_file: Option<&'a Path>) -> Self {
        Self { root, active_file, requests: Vec::new() }
    }

    /// Pasta do projeto aberto.
    pub fn root(&self) -> &Path {
        self.root
    }

    pub fn active_file(&self) -> Option<&Path> {
        self.active_file
    }

    pub fn open_file(&mut self, path: impl Into<PathBuf>) {
        self.requests.push(Request::OpenFile(path.into()));
    }

    pub fn register_panel(&mut self, id: PanelId, title: impl Into<String>, slot: Slot) {
        self.requests.push(Request::RegisterPanel { id, title: title.into(), slot });
    }

    /// Registra um comando. O id deve ter o prefixo da extensão, ex.: `"demo.refresh"`.
    pub fn register_command(&mut self, id: impl Into<String>, title: impl Into<String>) {
        self.requests.push(Request::RegisterCommand { id: id.into(), title: title.into() });
    }

    pub fn run_command(&mut self, id: impl Into<String>) {
        self.requests.push(Request::RunCommand(id.into()));
    }

    pub fn notify(&mut self, text: impl Into<String>) {
        self.requests.push(Request::Notify(text.into()));
    }

    pub fn toggle_panel(&mut self, id: PanelId) {
        self.requests.push(Request::PanelVisibility { id, visible: None });
    }

    pub fn show_panel(&mut self, id: PanelId) {
        self.requests.push(Request::PanelVisibility { id, visible: Some(true) });
    }

    pub fn path_renamed(&mut self, from: impl Into<PathBuf>, to: impl Into<PathBuf>) {
        self.requests.push(Request::PathRenamed { from: from.into(), to: to.into() });
    }

    pub fn path_deleted(&mut self, path: impl Into<PathBuf>) {
        self.requests.push(Request::PathDeleted(path.into()));
    }

    pub fn into_requests(self) -> Vec<Request> {
        self.requests
    }
}

/// A trait que toda extensão implementa (internas hoje, WASM no futuro via adaptador).
///
/// Cada extensão tem o próprio tipo de mensagem. O app guarda as extensões
/// como [`AnyExtension`] e apaga o tipo na fronteira.
pub trait Extension: 'static {
    type Message: Clone + fmt::Debug + Send + Sync + 'static;

    /// Id estável e único, ex.: `"filetree"`. Também é o prefixo dos comandos.
    fn id(&self) -> &'static str;

    /// Chamado uma vez na carga: registre painéis e comandos aqui.
    fn activate(&mut self, ctx: &mut Context) -> Task<Self::Message>;

    fn on_event(&mut self, _event: &CoreEvent, _ctx: &mut Context) -> Task<Self::Message> {
        Task::none()
    }

    /// Um comando desta extensão foi disparado (palette, atalho ou outra extensão).
    fn on_command(&mut self, _command: &str, _ctx: &mut Context) -> Task<Self::Message> {
        Task::none()
    }

    fn update(&mut self, message: Self::Message, ctx: &mut Context) -> Task<Self::Message>;

    fn view(&self, panel: PanelId) -> Element<'_, Self::Message>;
}

/// Mensagem de extensão com o tipo apagado. Só a extensão dona sabe abrir.
#[derive(Clone)]
pub struct ExtMessage(Arc<dyn Any + Send + Sync>);

impl ExtMessage {
    fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl fmt::Debug for ExtMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExtMessage(..)")
    }
}

/// Versão "object safe" de [`Extension`], usada pelo app.
pub trait AnyExtension {
    fn id(&self) -> &'static str;
    fn activate(&mut self, ctx: &mut Context) -> Task<ExtMessage>;
    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<ExtMessage>;
    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<ExtMessage>;
    fn update(&mut self, message: ExtMessage, ctx: &mut Context) -> Task<ExtMessage>;
    fn view(&self, panel: PanelId) -> Element<'_, ExtMessage>;
}

impl<E: Extension> AnyExtension for E {
    fn id(&self) -> &'static str {
        Extension::id(self)
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<ExtMessage> {
        Extension::activate(self, ctx).map(ExtMessage::new)
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<ExtMessage> {
        Extension::on_event(self, event, ctx).map(ExtMessage::new)
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<ExtMessage> {
        Extension::on_command(self, command, ctx).map(ExtMessage::new)
    }

    fn update(&mut self, message: ExtMessage, ctx: &mut Context) -> Task<ExtMessage> {
        match message.0.downcast_ref::<E::Message>() {
            Some(m) => Extension::update(self, m.clone(), ctx).map(ExtMessage::new),
            // Mensagem roteada para a extensão errada: bug no app, ignora.
            None => Task::none(),
        }
    }

    fn view(&self, panel: PanelId) -> Element<'_, ExtMessage> {
        Extension::view(self, panel).map(ExtMessage::new)
    }
}
