//! Núcleo da IDE TEA: a API que toda extensão usa.
//!
//! O núcleo não conhece nenhuma extensão específica. Extensões falam com o
//! app só por [`Context`] (pedidos), [`CoreEvent`] (notificações) e comandos.

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::{Element, Subscription, Task, Theme};

// Para a macro `i18n_loader!` (que usa o caminho `tea_core::`) funcionar aqui dentro também.
extern crate self as tea_core;

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

pub mod calls;
pub mod coding_agents;
pub mod config;
pub mod config_edit;
pub mod debug;
pub mod file_icons;
pub mod fonts;
pub mod i18n;
pub mod icons;
pub mod menu;
pub mod syntaxes;
pub mod outline;
pub mod os;
pub mod pick;
pub mod replace;
pub mod style;
pub mod keymap;
pub mod languages;

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
    /// O executável da IDE foi recompilado ou atualizado depois que ela abriu: esta IDE está desatualizada.
    BinaryChanged,
    /// Diagnósticos de um arquivo (substituem os anteriores desse arquivo e dessa fonte).
    Diagnostics { source: String, path: PathBuf, items: Vec<Diagnostic> },
    /// Trabalho em segundo plano (ex.: "rust-analyzer: indexando 40%"). `text: None` = terminou.
    Activity { source: String, text: Option<String> },
    /// Tema ou tamanho da fonte mudou. Também é enviado logo depois da ativação.
    Appearance { theme: Theme, font_size: f32, code_font: crate::fonts::CodeFont },
    /// Um painel desta extensão foi aberto pelo usuário (barra de ícones). Só o dono recebe.
    PanelShown(PanelId),
    /// Status de controle de versão dos arquivos (vindo da extensão git). Substitui o anterior.
    /// Caminhos absolutos; uma pasta nova ou ignorada inteira aparece como a própria pasta.
    FileStatuses(Arc<HashMap<PathBuf, FileStatus>>),
    /// Arquivos foram criados, apagados ou mudados no disco por uma extensão (não pelo editor).
    FilesChanged,
    /// Sessão de debug (ver [`debug`]).
    Debug(debug::Event),
    /// Estrutura do arquivo ativo. `note` explica uma lista vazia (sem servidor, arquivo sem nome...).
    Outline { path: Option<PathBuf>, symbols: Arc<Vec<outline::Symbol>>, note: Option<String> },
    /// Lista de lugares de um símbolo (onde é usado, implementações...). Substitui a anterior.
    Locations { kind: LocationKind, symbol: String, items: Vec<Reference> },
    /// Árvore de chamadas (ver [`calls`]).
    Calls(calls::Event),
    /// Texto do arquivo ativo, para quem pediu com [`Context::watch_text`] (só arquivos com as extensões
    /// pedidas). Chega ao trocar de aba e um instante depois de parar de digitar. Arquivo ativo de
    /// outro tipo: só [`CoreEvent::ActiveFileChanged`].
    ActiveText { path: PathBuf, text: Arc<str> },
    /// Resposta de [`Request::Replace`]: quantas trocas, em quais arquivos, e os que não deu para mudar
    /// (sem permissão, não é UTF-8...).
    Replaced { count: usize, files: Vec<PathBuf>, failed: Vec<PathBuf> },
    /// Rodar uma tarefa no terminal (quem cuida é a extensão do terminal): `name` para mostrar, `command`
    /// para o shell, na pasta `cwd`. `done` é o texto do fim, com `{code}` no lugar do código de saída.
    RunTask { name: String, command: String, cwd: PathBuf, done: String },
    /// Abrir um agente numa aba de terminal, pelo "Agentes: iniciar…" (RFC 0016; quem cuida é a extensão do
    /// terminal). `title` é o nome da aba. Com `worktree`, roda numa cópia do repositório com esse branch, como os
    /// terminais pedidos por agentes; o worktree sai ao fechar a aba, se não tiver nada por commitar.
    LaunchAgent { title: String, command: String, worktree: Option<String> },
    /// Colar texto na aba de agente (`Ctrl+L`, RFC 0016): a que está na tela, senão a mais recente. Sem nenhuma,
    /// a extensão do terminal abre o lançador e cola na aba que ele abrir.
    SendToAgent { text: String },
}

/// O que uma lista de lugares ([`CoreEvent::Locations`]) mostra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationKind {
    References,
    Definitions,
    Implementations,
    TypeDefinitions,
}

impl LocationKind {
    /// "3 implementações", "1 referência"...
    pub fn count_label(self, n: usize) -> String {
        match self {
            LocationKind::References => fl!("count-references", count = n),
            LocationKind::Definitions => fl!("count-definitions", count = n),
            LocationKind::Implementations => fl!("count-implementations", count = n),
            LocationKind::TypeDefinitions => fl!("count-types", count = n),
        }
    }
}

/// Um lugar de um símbolo (uso, definição, implementação...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub path: PathBuf,
    /// Começam em 1 (coluna em caracteres).
    pub line: usize,
    pub column: usize,
    /// Tamanho do nome em caracteres (para destacar na prévia).
    pub len: usize,
    /// A linha inteira, sem o recuo.
    pub preview: String,
    /// Recuo tirado da prévia (em caracteres): `column - 1 - indent` é onde o nome começa nela.
    pub indent: usize,
}

/// Status de um arquivo no controle de versão. A ordem vai do mais fraco ao mais forte
/// (uma pasta mostra o mais forte entre os arquivos de dentro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileStatus {
    Ignored,
    Untracked,
    Added,
    Modified,
    Deleted,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Erro, aviso ou dica sobre um trecho de arquivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Começam em 1 (coluna em caracteres).
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub message: String,
    /// Quem gerou: "rustc", "clippy", "clangd"...
    pub source: Option<String>,
    pub code: Option<String>,
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
    /// Abre e posiciona o cursor. Linha e coluna começam em 1 (coluna em caracteres).
    OpenFileAt { path: PathBuf, line: usize, column: usize },
    /// Como [`Request::OpenFileAt`], mas sem tirar o usuário do que ele está digitando: se ele digitou no editor
    /// há pouco, o app só mostra `busy_note`.
    OpenFileWhenIdle { path: PathBuf, line: usize, column: usize, busy_note: String },
    /// Salva as abas destes arquivos que têm alterações e depois manda `then` de volta para a extensão. Se
    /// alguma não puder ser salva, o app avisa e `then` não vai.
    SaveFiles { paths: Vec<PathBuf>, then: ExtMessage },
    RegisterPanel { id: PanelId, title: String, slot: Slot, visible: bool },
    RegisterCommand { id: String, title: String },
    RunCommand(String),
    Notify(String),
    /// Mandar texto à aba de agente (RFC 0016): o app repassa como [`CoreEvent::SendToAgent`].
    SendToAgent(String),
    /// Muda a visibilidade de um painel desta extensão. `None` alterna.
    PanelVisibility { id: PanelId, visible: Option<bool> },
    /// A extensão renomeou um arquivo ou pasta no disco: o app atualiza as abas.
    PathRenamed { from: PathBuf, to: PathBuf },
    /// A extensão apagou um arquivo ou pasta no disco: o app fecha as abas sem alterações.
    PathDeleted(PathBuf),
    /// Mostra um diálogo modal. O botão escolhido manda a mensagem dele de volta para a extensão.
    Dialog(Dialog),
    /// Mostra um menu de contexto (ver [`menu`]).
    Menu(menu::Menu),
    /// Ícone do painel na barra de ícones.
    PanelIcon { id: PanelId, icon: icons::Icon },
    /// A extensão mudou um arquivo no disco (ex.: git descartou alterações): abas abertas recarregam.
    FileChanged(PathBuf),
    /// Repassa [`CoreEvent::FileStatuses`] às outras extensões.
    FileStatuses(Arc<HashMap<PathBuf, FileStatus>>),
    /// Repassa [`CoreEvent::FilesChanged`] às outras extensões.
    FilesChanged,
    /// Debug: seleciona um quadro da pilha (índice): abre o arquivo e manda as variáveis dele.
    DebugSelectFrame(usize),
    /// Debug: pede os filhos de uma variável ou grupo (resposta em [`debug::Event::Variables`]).
    DebugVariables(i64),
    /// Botão na barra de abas para arquivos com estas extensões (ver [`Context::file_action`]).
    FileAction(FileAction),
    /// Quer o texto do arquivo ativo quando ele tem uma destas extensões (ver [`CoreEvent::ActiveText`]).
    WatchText(Vec<String>),
    /// Árvore de chamadas: pede os filhos de um nó (resposta em [`calls::Event::Children`]).
    CallChildren { node: u64, direction: calls::Direction },
    /// Substitui em vários arquivos: abas abertas mudam na memória (dá para desfazer), os outros são
    /// gravados no disco. A resposta chega em [`CoreEvent::Replaced`].
    Replace(replace::Replace),
    /// Versão de referência de um arquivo para as marcas na margem (o git manda a do último commit).
    /// `None`: sem marcas (arquivo novo, fora do repositório).
    DiffBase { path: PathBuf, text: Option<Arc<str>> },
    /// Mostra uma lista para escolher na paleta (ver [`pick`]).
    Pick(pick::Pick),
}

/// Botão no canto da barra de abas, que aparece só quando o arquivo ativo é de certos tipos
/// (ex.: a prévia, para Markdown e SVG). Clicar roda `command`.
#[derive(Debug, Clone)]
pub struct FileAction {
    /// Extensões sem o ponto, minúsculas (`["md", "svg"]`).
    pub extensions: Vec<String>,
    pub icon: icons::Icon,
    pub tip: String,
    pub command: String,
    /// O botão fica destacado enquanto este painel (da mesma extensão) está aberto.
    pub panel: Option<PanelId>,
}

/// Estilo de um botão de diálogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// Ação recomendada.
    Primary,
    /// Ação destrutiva (apagar, descartar).
    Danger,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct DialogButton {
    pub label: String,
    pub kind: ButtonKind,
    /// `None` = só fecha o diálogo (cancelar).
    pub action: Option<ExtMessage>,
}

/// Diálogo modal desenhado pela IDE. Enter aciona o botão em destaque (o primeiro
/// `Primary` ou `Danger`), Esc ou clique fora fecham sem ação, ←/→/Tab trocam o destaque.
///
/// ```ignore
/// ctx.show_dialog(
///     Dialog::new("Apagar arquivo?", "main.rs vai para a lixeira.")
///         .button("Mover para a lixeira", ButtonKind::Danger, Message::DeleteConfirmed(path))
///         .cancel("Cancelar"),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Dialog {
    pub title: String,
    pub message: String,
    pub buttons: Vec<DialogButton>,
}

impl Dialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self { title: title.into(), message: message.into(), buttons: Vec::new() }
    }

    /// `on_press` precisa ser do tipo `Message` da própria extensão (senão é ignorada).
    pub fn button<M: Any + Send + Sync>(mut self, label: impl Into<String>, kind: ButtonKind, on_press: M) -> Self {
        self.buttons.push(DialogButton { label: label.into(), kind, action: Some(ExtMessage::new(on_press)) });
        self
    }

    pub fn cancel(mut self, label: impl Into<String>) -> Self {
        self.buttons.push(DialogButton { label: label.into(), kind: ButtonKind::Secondary, action: None });
        self
    }

    /// Botão que o Enter aciona.
    pub fn default_button(&self) -> Option<usize> {
        self.buttons.iter().position(|b| b.kind != ButtonKind::Secondary)
    }
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

    pub fn open_file_at(&mut self, path: impl Into<PathBuf>, line: usize, column: usize) {
        self.requests.push(Request::OpenFileAt { path: path.into(), line, column });
    }

    /// Abre como [`Context::open_file_at`] se o usuário não estiver digitando; senão só avisa com `busy_note`.
    pub fn open_file_when_idle(&mut self, path: impl Into<PathBuf>, line: usize, column: usize, busy_note: impl Into<String>) {
        self.requests.push(Request::OpenFileWhenIdle { path: path.into(), line, column, busy_note: busy_note.into() });
    }

    /// Salva as abas alteradas destes arquivos e então manda `then` para a extensão (ver [`Request::SaveFiles`]).
    pub fn save_files<M: Any + Send + Sync>(&mut self, paths: Vec<PathBuf>, then: M) {
        self.requests.push(Request::SaveFiles { paths, then: ExtMessage::new(then) });
    }

    pub fn register_panel(&mut self, id: PanelId, title: impl Into<String>, slot: Slot) {
        self.requests.push(Request::RegisterPanel { id, title: title.into(), slot, visible: true });
    }

    /// Registra um painel que só aparece quando a extensão pedir (`show_panel`).
    pub fn register_panel_hidden(&mut self, id: PanelId, title: impl Into<String>, slot: Slot) {
        self.requests.push(Request::RegisterPanel { id, title: title.into(), slot, visible: false });
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

    /// Manda texto (um lugar, um diagnóstico...) à aba de agente, como o `Ctrl+L` do editor.
    pub fn send_to_agent(&mut self, text: impl Into<String>) {
        self.requests.push(Request::SendToAgent(text.into()));
    }

    pub fn toggle_panel(&mut self, id: PanelId) {
        self.requests.push(Request::PanelVisibility { id, visible: None });
    }

    pub fn show_panel(&mut self, id: PanelId) {
        self.requests.push(Request::PanelVisibility { id, visible: Some(true) });
    }

    pub fn show_dialog(&mut self, dialog: Dialog) {
        self.requests.push(Request::Dialog(dialog));
    }

    /// Ícone do painel na barra de ícones da borda (sem ícone, o painel não aparece nela).
    pub fn panel_icon(&mut self, id: PanelId, icon: icons::Icon) {
        self.requests.push(Request::PanelIcon { id, icon });
    }

    /// Avisa que a extensão mudou (ou apagou) este arquivo no disco.
    pub fn file_changed(&mut self, path: impl Into<PathBuf>) {
        self.requests.push(Request::FileChanged(path.into()));
    }

    /// Publica o status de controle de versão dos arquivos (ver [`CoreEvent::FileStatuses`]).
    pub fn file_statuses(&mut self, statuses: Arc<HashMap<PathBuf, FileStatus>>) {
        self.requests.push(Request::FileStatuses(statuses));
    }

    /// Avisa que arquivos foram criados/apagados/renomeados no disco (ver [`CoreEvent::FilesChanged`]).
    pub fn files_changed(&mut self) {
        self.requests.push(Request::FilesChanged);
    }

    pub fn show_menu(&mut self, menu: menu::Menu) {
        self.requests.push(Request::Menu(menu));
    }

    pub fn hide_panel(&mut self, id: PanelId) {
        self.requests.push(Request::PanelVisibility { id, visible: Some(false) });
    }

    pub fn path_renamed(&mut self, from: impl Into<PathBuf>, to: impl Into<PathBuf>) {
        self.requests.push(Request::PathRenamed { from: from.into(), to: to.into() });
    }

    pub fn path_deleted(&mut self, path: impl Into<PathBuf>) {
        self.requests.push(Request::PathDeleted(path.into()));
    }

    pub fn debug_select_frame(&mut self, index: usize) {
        self.requests.push(Request::DebugSelectFrame(index));
    }

    pub fn debug_variables(&mut self, reference: i64) {
        self.requests.push(Request::DebugVariables(reference));
    }

    /// Põe um botão na barra de abas, visível quando o arquivo ativo tem uma destas extensões.
    pub fn file_action(&mut self, action: FileAction) {
        self.requests.push(Request::FileAction(action));
    }

    /// Pede o texto do arquivo ativo sempre que ele for de um destes tipos (extensões sem o ponto,
    /// ex.: `["md", "svg"]`). Ele chega em [`CoreEvent::ActiveText`].
    pub fn watch_text<S: Into<String>>(&mut self, extensions: impl IntoIterator<Item = S>) {
        self.requests.push(Request::WatchText(extensions.into_iter().map(Into::into).collect()));
    }

    /// Pede quem chama o nó (ou quem ele chama). A resposta chega em [`calls::Event::Children`].
    pub fn call_children(&mut self, node: u64, direction: calls::Direction) {
        self.requests.push(Request::CallChildren { node, direction });
    }

    pub fn pick(&mut self, pick: pick::Pick) {
        self.requests.push(Request::Pick(pick));
    }

    pub fn diff_base(&mut self, path: impl Into<PathBuf>, text: Option<Arc<str>>) {
        self.requests.push(Request::DiffBase { path: path.into(), text });
    }

    pub fn replace(&mut self, replace: replace::Replace) {
        self.requests.push(Request::Replace(replace));
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

    /// Fontes de eventos contínuas (ex.: saída de um processo). Mensagens daqui
    /// não tiram o foco do editor, ao contrário das que vêm da interface.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }
}

/// Mensagem de extensão com o tipo apagado. Só a extensão dona sabe abrir.
#[derive(Clone)]
pub struct ExtMessage(Arc<dyn Any + Send + Sync>);

impl ExtMessage {
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.0.downcast_ref()
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
    fn subscription(&self) -> Subscription<ExtMessage>;
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

    fn subscription(&self) -> Subscription<ExtMessage> {
        Extension::subscription(self).map(ExtMessage::new)
    }
}
