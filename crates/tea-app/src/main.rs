//! Binário da IDE TEA.
//!
//! Uso: `idetea [--no-splash] [caminho...]` (ver `cli.rs`).

mod cli;
mod context_menu;
mod dialog;
mod divider;
mod help;
mod overlay;
mod lsp;
mod palette;
mod picker;
mod settings;
mod pass_keys;
mod splash;
mod ui_state;
mod watcher;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use iced::widget::{button, column, container, mouse_area, opaque, operation, row, rule, scrollable, stack, text};
use iced::{Element, Fill, Subscription, Task, Theme, keyboard, mouse, window};
use tea_core::config::{self, Config};
use tea_core::keymap::{Chord, Keymap};
use tea_core::{
    AnyExtension, ButtonKind, Context, CoreEvent, Diagnostic, Dialog, EditorStatus, ExtMessage, PanelId, Request, Severity,
    Slot,
};
use tea_editor::{Editor, EditorAction, EditorMessage};

use palette::{CommandInfo, Palette};
use splash::Splash;

/// Argumentos já interpretados, para o `boot` do iced (que não recebe parâmetros).
static LAUNCH: OnceLock<cli::Launch> = OnceLock::new();
static STARTED: OnceLock<Instant> = OnceLock::new();

/// Id da janela no Wayland/X11. O painel do desktop acha o ícone pelo arquivo
/// `idetea.desktop` com este mesmo nome (`just desktop` instala).
const APP_ID: &str = "idetea";

fn main() -> iced::Result {
    STARTED.get_or_init(Instant::now);
    let cwd = std::env::current_dir().unwrap_or_default();
    let env_no_splash = cli::env_flag(std::env::var_os("IDETEA_NO_SPLASH"));
    match cli::parse(std::env::args_os().skip(1), &cwd, env_no_splash) {
        cli::Cli::Help => {
            println!("{}", cli::USAGE);
            Ok(())
        }
        cli::Cli::Version => {
            println!("idetea {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        cli::Cli::Run(launch) => {
            let _ = LAUNCH.set(launch);
            iced::application(App::boot, App::update, App::view)
                .title(App::title)
                .subscription(App::subscription)
                .theme(App::theme)
                .font(tea_core::icons::FONT_BYTES)
                .window(window::Settings {
                    platform_specific: window::settings::PlatformSpecific {
                        application_id: APP_ID.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .exit_on_close_request(false)
                .run()
        }
    }
}

/// Extensões compiladas nesta build. Cada uma entra por uma feature do cargo.
#[allow(unused_mut, clippy::vec_init_then_push)] // cada push depende de uma feature
fn builtin_extensions() -> Vec<Box<dyn AnyExtension>> {
    let mut list: Vec<Box<dyn AnyExtension>> = Vec::new();
    #[cfg(feature = "ext-filetree")]
    list.push(Box::new(tea_ext_filetree::FileTreeExtension::new()));
    #[cfg(feature = "ext-statusbar")]
    list.push(Box::new(tea_ext_statusbar::StatusBarExtension::new()));
    #[cfg(feature = "ext-search")]
    list.push(Box::new(tea_ext_search::SearchExtension::new()));
    #[cfg(feature = "ext-problems")]
    list.push(Box::new(tea_ext_problems::ProblemsExtension::new()));
    #[cfg(feature = "ext-terminal")]
    list.push(Box::new(tea_ext_terminal::TerminalExtension::new()));
    #[cfg(feature = "ext-git")]
    list.push(Box::new(tea_ext_git::GitExtension::new()));
    list
}

/// Comandos do núcleo: (id, título).
const CORE_COMMANDS: &[(&str, &str)] = &[
    ("palette.open", "Paleta de comandos"),
    ("help.open", "Ajuda: atalhos e guia"),
    ("settings.open", "Configurações"),
    ("file.new", "Arquivo: novo"),
    ("file.save", "Arquivo: salvar"),
    ("file.open", "Arquivo: abrir…"),
    ("file.quick_open", "Arquivo: abrir pelo nome…"),
    ("file.open_folder", "Arquivo: abrir pasta em nova janela…"),
    ("file.save_as", "Arquivo: salvar como…"),
    ("file.save_as_path", "Arquivo: salvar como (digitar caminho)…"),
    ("file.close", "Arquivo: fechar aba"),
    ("tab.next", "Abas: próxima"),
    ("tab.prev", "Abas: anterior"),
    ("editor.find", "Editor: buscar"),
    ("editor.replace", "Editor: buscar e substituir"),
    ("editor.goto_line", "Editor: ir para linha"),
    ("editor.undo", "Editor: desfazer"),
    ("editor.redo", "Editor: refazer"),
    ("editor.select_all", "Editor: selecionar tudo"),
    ("editor.fold_all", "Editor: recolher todos os blocos"),
    ("editor.unfold_all", "Editor: expandir todos os blocos"),
    ("editor.toggle_vim", "Editor: ligar/desligar modo vim"),
    ("lsp.definition", "LSP: ir para definição"),
    ("lsp.hover", "LSP: mostrar documentação no cursor"),
    ("lsp.format", "LSP: formatar arquivo"),
    ("lsp.restart", "LSP: reiniciar servidores"),
    ("theme.toggle", "Tema: alternar claro/escuro"),
    ("config.open", "Config: abrir config global"),
    ("config.open_project", "Config: abrir config do projeto"),
    ("config.reload", "Config: recarregar"),
    ("view.toggle_left", "Layout: mostrar/ocultar lateral esquerda"),
    ("view.toggle_right", "Layout: mostrar/ocultar lateral direita"),
    ("view.toggle_bottom", "Layout: mostrar/ocultar painel inferior"),
    ("app.quit", "Sair"),
];

const DARK_THEME: Theme = Theme::TokyoNight;
const LIGHT_THEME: Theme = Theme::Light;

/// Respostas dos diálogos do próprio app.
#[derive(Debug, Clone)]
enum CoreDialog {
    SaveAndClose(u64),
    DiscardAndClose(u64),
    SaveAllAndQuit,
    QuitWithoutSaving,
    /// Fechar várias abas (salvando ou descartando as alteradas).
    CloseMany { ids: Vec<u64>, save: bool },
}

/// Itens dos menus de contexto do próprio app (abas).
#[derive(Debug, Clone)]
enum CoreMenu {
    Close(u64),
    CloseOthers(u64),
    CloseRight(u64),
    CloseSaved,
    CloseAll,
    CopyPath(String),
    Reveal(u64),
    OpenInFileManager(PathBuf),
}

struct Tab {
    id: u64,
    editor: Editor,
}

struct Panel {
    ext: usize,
    id: PanelId,
    title: String,
    slot: Slot,
    /// Slot pedido pela extensão (o config pode mudar `slot`).
    default_slot: Slot,
    visible: bool,
    /// Ícone na barra de ícones (sem ícone, o painel não aparece nela).
    icon: Option<tea_core::icons::Icon>,
}

/// Borda arrastável de um painel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Right,
    Bottom,
}

struct App {
    root: PathBuf,
    config: Config,
    global_config: Option<PathBuf>,
    keymap: Keymap,
    theme: Theme,

    tabs: Vec<Tab>,
    active: usize,
    next_tab_id: u64,

    extensions: Vec<Box<dyn AnyExtension>>,
    panels: Vec<Panel>,
    lsp: lsp::LspManager,
    commands: Vec<CommandInfo>,
    /// Painel em destaque no slot inferior (índice em `panels`), que mostra um painel por vez em abas.
    active_bottom: Option<usize>,

    palette: Option<Palette>,
    /// Quando a própria IDE salvou cada arquivo: o aviso de "mudou no disco" desse save não conta.
    own_saves: std::collections::HashMap<PathBuf, Instant>,
    /// Última lista de arquivos do projeto (para `Ctrl+P` abrir na hora; é relida a cada abertura).
    file_list: std::sync::Arc<Vec<String>>,
    splash: Option<Splash>,
    /// Ponteiro sobre um painel lateral: um clique ali tira o foco do editor.
    pointer_over_panel: bool,
    dialog: Option<dialog::Open>,
    menu: Option<context_menu::Open>,
    /// Uma janela nativa de arquivo/pasta está aberta (não abre outra por cima).
    picking: bool,
    help: Option<help::Help>,
    settings: Option<settings::Settings>,
    /// Ids de todas as extensões compiladas (ligadas ou não), para a tela de configurações.
    all_extensions: Vec<&'static str>,
    /// Tamanho dos painéis (salvo entre execuções).
    ui_state: ui_state::UiState,
    state_path: Option<PathBuf>,
    window_size: Option<iced::Size>,
    last_status: EditorStatus,
}

#[derive(Debug, Clone)]
enum Message {
    Editor(u64, EditorMessage),
    /// Vinda da interface de uma extensão.
    Ext(usize, ExtMessage),
    /// Vinda da subscription de uma extensão.
    ExtSub(usize, ExtMessage),
    SelectTab(u64),
    /// Botão direito numa aba.
    TabMenu(u64, iced::Point),
    CloseTab(u64),
    SelectBottom(usize),
    Key { key: keyboard::Key, modifiers: keyboard::Modifiers, captured: bool },
    Palette(palette::Msg),
    /// Arquivos mudaram no disco (observador do projeto).
    Disk(watcher::Change),
    /// A lista de arquivos do projeto terminou de ser lida (para `Ctrl+P`).
    FilesListed(std::sync::Arc<Vec<String>>),
    Dialog(dialog::Msg),
    Menu(context_menu::Msg),
    /// Resposta da janela nativa de arquivo/pasta.
    Picked(picker::Picked),
    Help(help::Msg),
    Settings(settings::Msg),
    /// Clique num ícone da barra de ícones (índice em `panels`).
    Activity(usize),
    Resize(Edge, f32),
    ResizeDone,
    ResizeReset(Edge),
    WindowResized(iced::Size),
    Frame(Instant),
    PointerOverPanel(bool),
    MousePressed,
    CloseRequested,
    Lsp(tea_lsp::Event),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let started = *STARTED.get_or_init(Instant::now);
        let launch = LAUNCH.get().expect("argumentos interpretados em main");
        let root = launch.root.clone();
        let global_config = config::global_config_path();
        let config = Config::load(global_config.as_deref(), Some(&config::project_config_path(&root)));
        let (keymap, key_errors) = Keymap::from_pairs(config.keys.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let (theme, theme_error) = resolve_theme(&config.ui.theme);

        let show_splash = config.ui.splash && !launch.no_splash;
        let state_path = ui_state::state_path();
        let builtin = builtin_extensions();
        let all_extensions: Vec<&'static str> = builtin.iter().map(|e| e.id()).collect();
        let extensions = builtin.into_iter().filter(|e| config.extension(e.id()).enabled).collect();

        // Eventos dos servidores LSP chegam por este canal (de outras threads).
        let (lsp_tx, lsp_rx) = iced::futures::channel::mpsc::unbounded();
        let sink: tea_lsp::EventSink = std::sync::Arc::new(move |e| {
            let _ = lsp_tx.unbounded_send(e);
        });
        let lsp = lsp::LspManager::new(&root, &config.lsp, sink);

        let mut app = Self {
            lsp,
            root,
            global_config,
            keymap,
            theme,
            tabs: Vec::new(),
            active: 0,
            next_tab_id: 0,
            extensions,
            panels: Vec::new(),
            active_bottom: None,
            commands: CORE_COMMANDS
                .iter()
                .map(|(id, title)| CommandInfo { id: id.to_string(), title: title.to_string() })
                .collect(),
            palette: None,
            file_list: Default::default(),
            own_saves: Default::default(),
            splash: show_splash.then(|| Splash::new(started)),
            pointer_over_panel: false,
            dialog: None,
            menu: None,
            picking: false,
            help: None,
            settings: None,
            all_extensions,
            ui_state: ui_state::UiState::load(state_path.as_deref()),
            state_path,
            window_size: None,
            last_status: EditorStatus::default(),
            config,
        };
        app.push_tab(Editor::scratch(""));

        let mut tasks = vec![Task::run(lsp_rx, Message::Lsp)];
        // Extensões primeiro: assim elas recebem os eventos dos arquivos abertos abaixo.
        for i in 0..app.extensions.len() {
            tasks.push(app.call_ext(i, |e, ctx| e.activate(ctx)));
        }
        tasks.push(app.broadcast_appearance());
        for file in launch.files.clone() {
            tasks.push(app.open_file(file));
        }
        tasks.push(app.emit_status(true));

        let mut problems: Vec<String> = launch.warnings.clone();
        problems.extend(app.config.errors.iter().cloned());
        problems.extend(key_errors);
        problems.extend(theme_error);
        if !problems.is_empty() {
            for p in &problems {
                eprintln!("[idetea] {p}");
            }
            tasks.push(app.notify(format!("Config: {}", problems.join(" · "))));
        }

        if let Some(s) = &mut app.splash {
            s.mark_ready(Instant::now());
            // Nada de digitar "às cegas" por baixo da splash.
            app.editor_mut().lose_focus();
        }
        eprintln!("[idetea] boot em {:?}", started.elapsed());
        (app, Task::batch(tasks))
    }

    fn title(&self) -> String {
        let project = self.root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        format!("{} — {project} — IDE TEA", self.tabs[self.active].editor.title())
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn editor(&self) -> &Editor {
        &self.tabs[self.active].editor
    }

    fn editor_mut(&mut self) -> &mut Editor {
        &mut self.tabs[self.active].editor
    }

    fn active_path(&self) -> Option<&Path> {
        self.tabs.get(self.active).and_then(|t| t.editor.path())
    }

    // ---- Ponte com as extensões ------------------------------------------

    /// Chama uma extensão com um `Context` novo e executa os pedidos que ela fez.
    fn call_ext(
        &mut self,
        i: usize,
        f: impl FnOnce(&mut dyn AnyExtension, &mut Context) -> Task<ExtMessage>,
    ) -> Task<Message> {
        let active = self.tabs.get(self.active).and_then(|t| t.editor.path());
        let mut ctx = Context::new(&self.root, active);
        let task = f(self.extensions[i].as_mut(), &mut ctx).map(move |m| Message::Ext(i, m));
        let requests = ctx.into_requests();
        let more = self.handle_requests(i, requests);
        Task::batch([task, more])
    }

    fn handle_requests(&mut self, ext: usize, requests: Vec<Request>) -> Task<Message> {
        let mut tasks = Vec::new();
        for request in requests {
            match request {
                Request::OpenFile(path) => tasks.push(self.open_file(path)),
                Request::OpenFileAt { path, line, column } => tasks.push(self.open_file_at(path, line, column)),
                Request::RegisterPanel { id, title, slot, visible } => {
                    let default_slot = slot;
                    let slot = self.slot_for(ext, default_slot);
                    self.panels.push(Panel { ext, id, title, slot, default_slot, visible, icon: None });
                    if slot == Slot::Bottom && visible && self.active_bottom.is_none() {
                        self.active_bottom = Some(self.panels.len() - 1);
                    }
                }
                Request::RegisterCommand { id, title } => self.commands.push(CommandInfo { id, title }),
                Request::RunCommand(id) => tasks.push(self.run_command(&id)),
                Request::Notify(msg) => tasks.push(self.notify(msg)),
                Request::PanelVisibility { id, visible } => {
                    if let Some(i) = self.panels.iter().position(|p| p.ext == ext && p.id == id) {
                        self.set_panel_visible(i, visible);
                    }
                }
                Request::PathRenamed { from, to } => {
                    tasks.push(self.path_renamed(&from, &to));
                    tasks.push(self.broadcast(CoreEvent::FilesChanged));
                }
                Request::PathDeleted(path) => {
                    tasks.push(self.path_deleted(&path));
                    tasks.push(self.broadcast(CoreEvent::FilesChanged));
                }
                Request::FileStatuses(statuses) => tasks.push(self.broadcast(CoreEvent::FileStatuses(statuses))),
                Request::FilesChanged => {
                    tasks.push(self.reload_changed_tabs());
                    tasks.push(self.broadcast(CoreEvent::FilesChanged));
                }
                Request::Dialog(dialog) => tasks.push(self.open_dialog(dialog, dialog::Owner::Ext(ext))),
                Request::FileChanged(path) => {
                    tasks.push(self.reload_from_disk(&path));
                    tasks.push(self.broadcast(CoreEvent::FilesChanged));
                }
                Request::Menu(menu) => tasks.push(self.open_menu(menu, dialog::Owner::Ext(ext))),
                Request::PanelIcon { id, icon } => {
                    if let Some(p) = self.panels.iter_mut().find(|p| p.ext == ext && p.id == id) {
                        p.icon = Some(icon);
                    }
                }
            }
        }
        Task::batch(tasks)
    }

    fn broadcast(&mut self, event: CoreEvent) -> Task<Message> {
        let tasks: Vec<_> =
            (0..self.extensions.len()).map(|i| self.call_ext(i, |e, ctx| e.on_event(&event, ctx))).collect();
        Task::batch(tasks)
    }

    fn broadcast_appearance(&mut self) -> Task<Message> {
        self.broadcast(CoreEvent::Appearance { theme: self.theme.clone(), font_size: self.config.ui.font_size })
    }

    fn notify(&mut self, msg: impl Into<String>) -> Task<Message> {
        self.broadcast(CoreEvent::Notification(msg.into()))
    }

    /// Avisa as extensões se o estado do editor ativo mudou.
    fn emit_status(&mut self, force: bool) -> Task<Message> {
        let ed = self.editor();
        let (line, column) = ed.cursor();
        let status = EditorStatus {
            path: ed.path().map(Path::to_path_buf),
            line,
            column,
            language: ed.language().to_string(),
            modified: ed.is_modified(),
            encoding: "UTF-8",
            line_ending: ed.line_ending().label(),
        };
        if !force && status == self.last_status {
            return Task::none();
        }
        self.last_status = status.clone();
        self.broadcast(CoreEvent::EditorStatus(status))
    }

    // ---- Abas e arquivos ----------------------------------------------------

    fn push_tab(&mut self, mut editor: Editor) -> usize {
        editor.set_theme(&self.theme);
        editor.set_font_size(self.config.ui.font_size);
        self.tabs.push(Tab { id: self.next_tab_id, editor });
        self.next_tab_id += 1;
        self.tabs.len() - 1
    }

    fn tab_index(&self, id: u64) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }

    fn open_file(&mut self, path: PathBuf) -> Task<Message> {
        if let Some(i) = self.tabs.iter().position(|t| t.editor.path() == Some(path.as_path())) {
            return self.set_active(i);
        }
        match Editor::open(&path) {
            Ok(mut editor) => {
                // Troca a aba vazia inicial em vez de acumular abas "sem título".
                let i = if self.tabs.len() == 1 && self.tabs[0].editor.is_blank() {
                    editor.set_theme(&self.theme);
                    editor.set_font_size(self.config.ui.font_size);
                    self.tabs[0].editor = editor;
                    0
                } else {
                    self.push_tab(editor)
                };
                let note = if self.tabs[i].editor.is_new_file() {
                    self.notify(format!("Arquivo novo: {} (criado ao salvar)", display_rel(&path, &self.root)))
                } else {
                    Task::none()
                };
                let lsp = self.ensure_lsp(i);
                let opened = self.broadcast(CoreEvent::FileOpened(path));
                Task::batch([opened, note, lsp, self.set_active(i)])
            }
            Err(err) => self.notify(format!("Erro ao abrir {}: {err}", path.display())),
        }
    }

    /// O arquivo mudou no disco por fora do editor. Aba sem alterações recarrega (ou fecha, se ele sumiu);
    /// aba com alterações fica como está, com um aviso.
    fn reload_from_disk(&mut self, path: &Path) -> Task<Message> {
        let Some(i) = self.tabs.iter().position(|t| t.editor.path() == Some(path)) else { return Task::none() };
        if !path.exists() {
            return self.path_deleted(path);
        }
        if self.tabs[i].editor.is_modified() {
            let name = self.tabs[i].editor.file_name();
            return self.notify(format!("\"{name}\" mudou no disco, mas a aba tem alterações: ela foi mantida"));
        }
        match Editor::open(path) {
            Ok(mut editor) => {
                editor.set_theme(&self.theme);
                editor.set_font_size(self.config.ui.font_size);
                // A aba antiga sai (e avisa o LSP); a nova entra no lugar, com o mesmo id.
                self.tabs[i].editor = editor;
                let lsp = self.ensure_lsp(i);
                let refresh = if i == self.active { self.set_active(i) } else { Task::none() };
                Task::batch([lsp, refresh])
            }
            Err(e) => self.notify(format!("Erro ao recarregar {}: {e}", display_rel(path, &self.root))),
        }
    }

    /// O observador viu mudanças no disco: abas sem alterações recarregam, abas com alterações
    /// avisam (uma vez por mudança), e as extensões ficam sabendo (árvore e git atualizam).
    fn disk_changed(&mut self, paths: &[PathBuf]) -> Task<Message> {
        let now = Instant::now();
        self.own_saves.retain(|_, t| now.duration_since(*t) < Duration::from_secs(2));
        let mut tasks = vec![self.reload_changed_tabs()];
        let conflicts: Vec<String> = self
            .tabs
            .iter()
            .filter(|t| t.editor.is_modified())
            .filter_map(|t| t.editor.path())
            .filter(|p| paths.iter().any(|c| c == p) && !self.own_saves.contains_key(*p))
            .map(|p| display_rel(p, &self.root))
            .collect();
        for name in conflicts {
            tasks.push(self.notify(format!("\"{name}\" mudou no disco, mas a aba tem alterações: salvar sobrescreve")));
        }
        tasks.push(self.broadcast(CoreEvent::FilesChanged));
        Task::batch(tasks)
    }

    /// Arquivos mudaram no disco sem dizer quais (ex.: git pull): recarrega as abas sem alterações
    /// cujo conteúdo no disco ficou diferente (ou fecha, se o arquivo sumiu).
    fn reload_changed_tabs(&mut self) -> Task<Message> {
        let stale: Vec<PathBuf> = self
            .tabs
            .iter()
            .filter(|t| !t.editor.is_modified() && !t.editor.is_new_file())
            .filter_map(|t| {
                let path = t.editor.path()?;
                let changed = match std::fs::read_to_string(path) {
                    Ok(disk) => disk.replace("\r\n", "\n") != t.editor.text(),
                    Err(e) => e.kind() == std::io::ErrorKind::NotFound,
                };
                changed.then(|| path.to_path_buf())
            })
            .collect();
        let tasks: Vec<_> = stale.iter().map(|p| self.reload_from_disk(p)).collect();
        Task::batch(tasks)
    }

    fn open_file_at(&mut self, path: PathBuf, line: usize, column: usize) -> Task<Message> {
        let open = self.open_file(path.clone());
        // Só posiciona se o arquivo abriu (ou já estava aberto).
        if self.active_path() != Some(path.as_path()) {
            return open;
        }
        let tab_id = self.tabs[self.active].id;
        let goto = self.editor_mut().goto(line, column).map(move |m| Message::Editor(tab_id, m));
        Task::batch([open, goto, self.emit_status(false)])
    }

    /// Liga a aba ao servidor de linguagem do arquivo, se houver.
    fn ensure_lsp(&mut self, i: usize) -> Task<Message> {
        let editor = &self.tabs[i].editor;
        let Some(path) = editor.path().map(Path::to_path_buf) else { return Task::none() };
        if editor.lsp_server().is_some() {
            return Task::none();
        }
        match self.lsp.client_for(&path) {
            Ok(Some((client, language))) => {
                self.tabs[i].editor.attach_lsp(client, &language);
                Task::none()
            }
            Ok(None) => Task::none(),
            Err(msg) => self.notify(msg),
        }
    }

    fn handle_lsp(&mut self, event: tea_lsp::Event) -> Task<Message> {
        use tea_lsp::Event as E;
        let tab_for = |app: &Self, path: &Path| app.tabs.iter().position(|t| t.editor.path() == Some(path));
        match event {
            E::Diagnostics { server, path, diagnostics } => {
                let items = diagnostics
                    .into_iter()
                    .map(|d| Diagnostic {
                        line: d.range.start.line as usize + 1,
                        column: d.range.start.character as usize + 1,
                        severity: match d.severity {
                            tea_lsp::Severity::Error => Severity::Error,
                            tea_lsp::Severity::Warning => Severity::Warning,
                            tea_lsp::Severity::Info => Severity::Info,
                            tea_lsp::Severity::Hint => Severity::Hint,
                        },
                        message: d.message,
                        source: d.source,
                        code: d.code,
                    })
                    .collect();
                self.broadcast(CoreEvent::Diagnostics { source: server, path, items })
            }
            E::Hover { path, text } => {
                if self.active_path() == Some(path.as_path()) {
                    self.editor_mut().show_hover(text);
                }
                Task::none()
            }
            E::Completion { path, items } => {
                if self.active_path() == Some(path.as_path()) {
                    self.editor_mut().show_completions(items);
                }
                Task::none()
            }
            E::Definition { path, position } => {
                self.open_file_at(path, position.line as usize + 1, position.character as usize + 1)
            }
            E::DefinitionNotFound => self.notify("Definição não encontrada"),
            E::Formatting { path, edits } => {
                if let Some(i) = tab_for(self, &path) {
                    self.tabs[i].editor.apply_formatting(&edits);
                    return Task::batch([self.notify("Arquivo formatado"), self.emit_status(false)]);
                }
                Task::none()
            }
            E::Progress { server, title, message, percentage, done } => {
                let text = (!done).then(|| {
                    let mut t = title;
                    if let Some(m) = message {
                        t = if t.is_empty() { m } else { format!("{t} {m}") };
                    }
                    if let Some(p) = percentage {
                        t = format!("{t} {p}%");
                    }
                    t
                });
                self.broadcast(CoreEvent::Activity { source: server, text })
            }
            E::Message { server, level, text } => {
                let prefix = match level {
                    tea_lsp::MessageLevel::Error => "erro: ",
                    tea_lsp::MessageLevel::Warning => "aviso: ",
                    tea_lsp::MessageLevel::Info => "",
                };
                self.notify(format!("{server}: {prefix}{text}"))
            }
            E::Log { server, text } => {
                if std::env::var_os("IDETEA_LSP_LOG").is_some() {
                    eprintln!("[{server}] {text}");
                }
                Task::none()
            }
            E::Ready { server } => self.broadcast(CoreEvent::Activity { source: server, text: None }),
            E::Exited { server, reason } => {
                self.lsp.on_exit(&server, &reason);
                for tab in &mut self.tabs {
                    if tab.editor.lsp_server() == Some(server.as_str()) {
                        tab.editor.detach_lsp();
                    }
                }
                let activity = self.broadcast(CoreEvent::Activity { source: server.clone(), text: None });
                let note = self.notify(format!("{server} parou — {reason}. Use \"LSP: reiniciar servidores\"."));
                Task::batch([activity, note])
            }
        }
    }

    fn set_active(&mut self, i: usize) -> Task<Message> {
        self.active = i.min(self.tabs.len() - 1);
        if self.palette.is_none() && self.splash.is_none() {
            self.editor_mut().focus();
        }
        let path = self.active_path().map(Path::to_path_buf);
        let changed = self.broadcast(CoreEvent::ActiveFileChanged(path));
        Task::batch([changed, self.emit_status(false)])
    }

    fn close_tab(&mut self, id: u64, force: bool) -> Task<Message> {
        let Some(i) = self.tab_index(id) else { return Task::none() };
        let editor = &self.tabs[i].editor;
        if editor.is_modified() && !force {
            let name = editor.file_name();
            let dialog = Dialog::new("Salvar alterações?", format!("\"{name}\" tem alterações que não foram salvas."))
                .button("Salvar", ButtonKind::Primary, CoreDialog::SaveAndClose(id))
                .button("Descartar", ButtonKind::Danger, CoreDialog::DiscardAndClose(id))
                .cancel("Cancelar");
            return self.open_dialog(dialog, dialog::Owner::Core);
        }
        self.tabs.remove(i);
        if self.tabs.is_empty() {
            self.push_tab(Editor::scratch(""));
        }
        let next = if i < self.active || self.active >= self.tabs.len() {
            self.active.saturating_sub(1)
        } else {
            self.active
        };
        self.set_active(next)
    }

    fn save_active(&mut self) -> Task<Message> {
        if self.editor().path().is_none() {
            return self.open_save_as();
        }
        let result = self.editor_mut().save();
        self.after_save(result)
    }

    /// "Salvar como" com a janela nativa do sistema.
    fn open_save_as(&mut self) -> Task<Message> {
        let name = self.active_path().and_then(Path::file_name).map(|n| n.to_string_lossy().into_owned());
        let (tab, dir) = (self.tabs[self.active].id, self.start_dir());
        self.pick(picker::save_as(tab, dir, name.unwrap_or_default()))
    }

    /// "Salvar como" digitando o caminho na paleta (sem janela do sistema).
    fn open_save_as_typed(&mut self) -> Task<Message> {
        let initial = match self.active_path() {
            Some(p) => display_rel(p, &self.root),
            None => String::new(),
        };
        self.open_palette(Palette::save_as(initial))
    }

    /// Pasta onde as janelas de arquivo começam: a do arquivo ativo, ou a raiz.
    fn start_dir(&self) -> PathBuf {
        self.active_path().and_then(Path::parent).map_or_else(|| self.root.clone(), Path::to_path_buf)
    }

    fn pick(&mut self, dialog: impl Future<Output = picker::Picked> + Send + 'static) -> Task<Message> {
        if self.picking {
            return self.notify("Já há uma janela de arquivos aberta");
        }
        self.picking = true;
        Task::perform(dialog, Message::Picked)
    }

    fn picked(&mut self, picked: picker::Picked) -> Task<Message> {
        self.picking = false;
        match picked {
            picker::Picked::SaveAs { tab, path: Some(path) } => {
                let Some(i) = self.tab_index(tab) else { return self.notify("A aba foi fechada antes de salvar") };
                let activate = self.set_active(i);
                Task::batch([activate, self.save_as_path(path)])
            }
            picker::Picked::Files(Some(files)) => Task::batch(files.into_iter().map(|f| self.open_file(f)).collect::<Vec<_>>()),
            picker::Picked::Folder(Some(dir)) => match picker::open_window(&dir) {
                Ok(()) => self.notify(format!("Abrindo {} numa nova janela", dir.display())),
                Err(e) => self.notify(format!("Não foi possível abrir a pasta: {e}")),
            },
            // Cancelou.
            _ => Task::none(),
        }
    }

    fn save_as(&mut self, input: &str) -> Task<Message> {
        let input = input.trim();
        if input.is_empty() {
            return Task::none();
        }
        let path = expand_home(input);
        let path = if path.is_absolute() { path } else { self.root.join(path) };
        self.save_as_path(path)
    }

    fn save_as_path(&mut self, path: PathBuf) -> Task<Message> {
        let result = self.editor_mut().save_as(&path);
        let ok = result.is_ok();
        let lsp = if ok { self.ensure_lsp(self.active) } else { Task::none() };
        let saved = Task::batch([self.after_save(result), lsp]);
        let changed = if ok { self.broadcast(CoreEvent::ActiveFileChanged(Some(path))) } else { Task::none() };
        Task::batch([saved, changed])
    }

    fn after_save(&mut self, result: std::io::Result<()>) -> Task<Message> {
        match result {
            Ok(()) => {
                let path = self.active_path().map(Path::to_path_buf).unwrap_or_default();
                self.own_saves.insert(path.clone(), Instant::now());
                let is_config = Some(path.as_path()) == self.global_config.as_deref()
                    || path == config::project_config_path(&self.root);
                let saved = self.broadcast(CoreEvent::FileSaved(path.clone()));
                let note = self.notify(format!("Salvo: {}", display_rel(&path, &self.root)));
                let reload = if is_config { self.reload_config() } else { Task::none() };
                Task::batch([saved, note, reload, self.emit_status(false)])
            }
            Err(err) => self.notify(format!("Erro ao salvar: {err}")),
        }
    }

    fn path_renamed(&mut self, from: &Path, to: &Path) -> Task<Message> {
        let mut active_changed = false;
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            if let Some(p) = tab.editor.path()
                && let Ok(rel) = p.strip_prefix(from)
            {
                let new = if rel.as_os_str().is_empty() { to.to_path_buf() } else { to.join(rel) };
                tab.editor.set_path(new);
                active_changed |= i == self.active;
            }
        }
        let relink: Vec<Task<Message>> = (0..self.tabs.len()).map(|i| self.ensure_lsp(i)).collect();
        let relink = Task::batch(relink);
        if active_changed {
            let path = self.active_path().map(Path::to_path_buf);
            Task::batch([relink, self.broadcast(CoreEvent::ActiveFileChanged(path)), self.emit_status(false)])
        } else {
            relink
        }
    }

    fn path_deleted(&mut self, deleted: &Path) -> Task<Message> {
        let gone: Vec<(u64, bool)> = self
            .tabs
            .iter()
            .filter(|t| t.editor.path().is_some_and(|p| p.starts_with(deleted)))
            .map(|t| (t.id, t.editor.is_modified()))
            .collect();
        let mut tasks = Vec::new();
        for (id, modified) in gone {
            if modified {
                tasks.push(self.notify("Arquivo apagado no disco; a aba com alterações foi mantida"));
            } else {
                tasks.push(self.close_tab(id, true));
            }
        }
        Task::batch(tasks)
    }

    // ---- Config e tema -------------------------------------------------------

    fn reload_config(&mut self) -> Task<Message> {
        self.reload_config_with(true)
    }

    /// `announce: false` = só avisa se houver problema (mudanças feitas pela tela de configurações).
    fn reload_config_with(&mut self, announce: bool) -> Task<Message> {
        let config = Config::load(self.global_config.as_deref(), Some(&config::project_config_path(&self.root)));
        let (keymap, key_errors) = Keymap::from_pairs(config.keys.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let (theme, theme_error) = resolve_theme(&config.ui.theme);
        let mut problems = config.errors.clone();
        problems.extend(key_errors);
        problems.extend(theme_error);
        self.keymap = keymap;
        self.lsp.update_config(&config.lsp);
        self.config = config;
        self.set_theme(theme);
        let font_size = self.config.ui.font_size;
        for tab in &mut self.tabs {
            tab.editor.set_font_size(font_size);
        }
        self.apply_panel_slots();
        let msg = if !problems.is_empty() {
            Some(format!("Config recarregado com problemas: {}", problems.join(" · ")))
        } else if announce {
            Some("Config recarregado (ligar/desligar extensões só vale ao reiniciar)".to_string())
        } else {
            None
        };
        let note = msg.map_or_else(Task::none, |m| self.notify(m));
        Task::batch([self.broadcast(CoreEvent::ConfigReloaded), self.broadcast_appearance(), note])
    }

    /// Slot de um painel: o do config, se houver; itens da barra de status ficam nela.
    fn slot_for(&self, ext: usize, default_slot: Slot) -> Slot {
        if default_slot == Slot::Status {
            return default_slot;
        }
        self.config.extension(self.extensions[ext].id()).slot.unwrap_or(default_slot)
    }

    /// Depois de recarregar o config: painéis mudam de lado na hora.
    fn apply_panel_slots(&mut self) {
        for i in 0..self.panels.len() {
            let slot = self.slot_for(self.panels[i].ext, self.panels[i].default_slot);
            self.panels[i].slot = slot;
        }
        let valid = self.active_bottom.is_some_and(|i| self.panels[i].slot == Slot::Bottom && self.panels[i].visible);
        if !valid {
            self.active_bottom = self.panels.iter().position(|p| p.slot == Slot::Bottom && p.visible);
        }
    }

    fn set_theme(&mut self, theme: Theme) {
        for tab in &mut self.tabs {
            tab.editor.set_theme(&theme);
        }
        self.theme = theme;
    }

    fn open_config(&mut self, global: bool) -> Task<Message> {
        let path = if global {
            let Some(path) = self.global_config.clone() else {
                return self.notify("Não foi possível descobrir a pasta de config (HOME não definido)");
            };
            if !path.exists() {
                let created = path
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&path, config::DEFAULT_CONFIG));
                if let Err(e) = created {
                    return self.notify(format!("Erro ao criar {}: {e}", path.display()));
                }
            }
            path
        } else {
            config::project_config_path(&self.root)
        };
        self.open_file(path)
    }

    // ---- Comandos ------------------------------------------------------------

    fn run_command(&mut self, id: &str) -> Task<Message> {
        let action = match id {
            "editor.find" => Some(EditorAction::Find),
            "editor.replace" => Some(EditorAction::Replace),
            "editor.goto_line" => Some(EditorAction::GotoLine),
            "editor.undo" => Some(EditorAction::Undo),
            "editor.redo" => Some(EditorAction::Redo),
            "editor.select_all" => Some(EditorAction::SelectAll),
            "editor.fold_all" => Some(EditorAction::FoldAll),
            "editor.unfold_all" => Some(EditorAction::UnfoldAll),
            "editor.toggle_vim" => Some(EditorAction::ToggleVim),
            _ => None,
        };
        if let Some(action) = action {
            let tab_id = self.tabs[self.active].id;
            let task = self.editor_mut().run(action).map(move |m| Message::Editor(tab_id, m));
            return Task::batch([task, self.emit_status(false)]);
        }

        match id {
            "palette.open" => self.open_palette(Palette::commands()),
            "file.quick_open" => self.open_quick_open(),
            "settings.open" => {
                if self.settings.is_some() {
                    self.close_settings();
                    Task::none()
                } else {
                    self.open_settings()
                }
            }
            "help.open" => {
                if self.help.is_some() {
                    self.close_help();
                    Task::none()
                } else {
                    self.open_help()
                }
            }
            "file.new" => {
                let i = self.push_tab(Editor::scratch(""));
                self.set_active(i)
            }
            "file.save" => self.save_active(),
            "file.save_as" => self.open_save_as(),
            "file.save_as_path" => self.open_save_as_typed(),
            "file.open" => {
                let dir = self.start_dir();
                self.pick(picker::open_files(dir))
            }
            "file.open_folder" => {
                let dir = self.root.parent().map_or_else(|| self.root.clone(), Path::to_path_buf);
                self.pick(picker::open_folder(dir))
            }
            "file.close" => self.close_tab(self.tabs[self.active].id, false),
            "tab.next" => self.set_active((self.active + 1) % self.tabs.len()),
            "tab.prev" => self.set_active((self.active + self.tabs.len() - 1) % self.tabs.len()),
            "theme.toggle" => {
                let next = if self.theme.extended_palette().is_dark { LIGHT_THEME } else { DARK_THEME };
                self.set_theme(next);
                self.broadcast_appearance()
            }
            "lsp.definition" => {
                if self.editor_mut().request_definition() {
                    Task::none()
                } else {
                    self.notify("Nenhum servidor de linguagem para este arquivo")
                }
            }
            "lsp.hover" => {
                self.editor_mut().request_hover();
                Task::none()
            }
            "lsp.format" => {
                if self.editor_mut().request_formatting() {
                    Task::none()
                } else {
                    self.notify("Formatação indisponível para este arquivo")
                }
            }
            "lsp.restart" => {
                self.lsp.restart();
                for tab in &mut self.tabs {
                    tab.editor.detach_lsp();
                }
                let relink: Vec<Task<Message>> = (0..self.tabs.len()).map(|i| self.ensure_lsp(i)).collect();
                Task::batch([Task::batch(relink), self.notify("Servidores de linguagem reiniciados")])
            }
            "config.open" => self.open_config(true),
            "config.open_project" => self.open_config(false),
            "config.reload" => self.reload_config(),
            "view.toggle_left" => self.toggle_slot(Slot::Left),
            "view.toggle_right" => self.toggle_slot(Slot::Right),
            "view.toggle_bottom" => self.toggle_slot(Slot::Bottom),
            "app.quit" => self.request_quit(),
            _ => {
                // Comandos de extensão: o prefixo antes do ponto é o id da extensão.
                let prefix = id.split('.').next().unwrap_or_default();
                match self.extensions.iter().position(|e| e.id() == prefix) {
                    Some(i) => {
                        // A extensão pode abrir um campo de texto: o editor não pode ficar com o teclado.
                        self.editor_mut().lose_focus();
                        self.call_ext(i, |e, ctx| e.on_command(id, ctx))
                    }
                    None => self.notify(format!("Comando desconhecido: {id}")),
                }
            }
        }
    }

    /// `None` alterna. No slot inferior, mostrar um painel também o coloca em destaque.
    fn set_panel_visible(&mut self, i: usize, visible: Option<bool>) {
        let p = &mut self.panels[i];
        if p.slot == Slot::Bottom {
            let showing = self.active_bottom == Some(i) && p.visible;
            let show = visible.unwrap_or(!showing);
            if show {
                p.visible = true;
                self.active_bottom = Some(i);
            } else if self.active_bottom == Some(i) {
                p.visible = false;
                // Passa o destaque para outro painel inferior visível, se houver.
                self.active_bottom =
                    self.panels.iter().position(|q| q.slot == Slot::Bottom && q.visible);
            }
        } else {
            p.visible = visible.unwrap_or(!p.visible);
        }
    }

    fn toggle_slot(&mut self, slot: Slot) -> Task<Message> {
        let any_visible = self.panels.iter().any(|p| p.slot == slot && p.visible);
        for p in self.panels.iter_mut().filter(|p| p.slot == slot) {
            p.visible = !any_visible;
        }
        if any_visible && self.palette.is_none() {
            // O painel que tinha o foco sumiu: o teclado volta para o editor.
            self.editor_mut().focus();
        }
        Task::none()
    }

    fn request_quit(&mut self) -> Task<Message> {
        let unsaved: Vec<String> =
            self.tabs.iter().filter(|t| t.editor.is_modified()).map(|t| t.editor.file_name()).collect();
        if unsaved.is_empty() {
            return self.quit();
        }
        let mut names = unsaved.iter().take(5).map(|n| format!("• {n}")).collect::<Vec<_>>().join("\n");
        if unsaved.len() > 5 {
            names.push_str(&format!("\n… e mais {}", unsaved.len() - 5));
        }
        let dialog = Dialog::new("Sair com alterações não salvas?", names)
            .button("Salvar tudo e sair", ButtonKind::Primary, CoreDialog::SaveAllAndQuit)
            .button("Sair sem salvar", ButtonKind::Danger, CoreDialog::QuitWithoutSaving)
            .cancel("Cancelar");
        self.open_dialog(dialog, dialog::Owner::Core)
    }

    fn quit(&mut self) -> Task<Message> {
        self.lsp.shutdown_all();
        iced::exit()
    }

    /// Salva todas as abas alteradas. `false` se alguma não pôde ser salva (erro ou sem nome).
    fn save_all(&mut self) -> (bool, Task<Message>) {
        let ids: Vec<u64> = self.tabs.iter().filter(|t| t.editor.is_modified()).map(|t| t.id).collect();
        let (failed, task) = self.save_tabs(&ids);
        (failed.is_empty(), task)
    }

    /// Salva as abas indicadas. Devolve as que não deu para salvar (erro ou aba sem nome).
    fn save_tabs(&mut self, ids: &[u64]) -> (Vec<u64>, Task<Message>) {
        let mut tasks = Vec::new();
        let mut failed = Vec::new();
        for &id in ids {
            let Some(i) = self.tab_index(id) else { continue };
            let Some(path) = self.tabs[i].editor.path().map(Path::to_path_buf) else {
                failed.push(id);
                let name = self.tabs[i].editor.file_name();
                tasks.push(self.notify(format!("\"{name}\" não tem nome: use \"Salvar como\"")));
                continue;
            };
            match self.tabs[i].editor.save() {
                Ok(()) => {
                    self.own_saves.insert(path.clone(), Instant::now());
                    tasks.push(self.broadcast(CoreEvent::FileSaved(path)));
                }
                Err(e) => {
                    failed.push(id);
                    tasks.push(self.notify(format!("Erro ao salvar {}: {e}", display_rel(&path, &self.root))));
                }
            }
        }
        (failed, Task::batch(tasks))
    }

    fn is_tab_modified(&self, id: u64) -> bool {
        self.tab_index(id).is_some_and(|i| self.tabs[i].editor.is_modified())
    }

    /// Fecha as abas sem perguntar (quem chama já resolveu as alterações).
    fn close_ids(&mut self, ids: &[u64]) -> Task<Message> {
        let tasks: Vec<_> = ids.iter().map(|&id| self.close_tab(id, true)).collect();
        Task::batch(tasks)
    }

    /// Fecha várias abas; se alguma tiver alterações, pergunta uma vez só para todas.
    fn close_many(&mut self, ids: Vec<u64>) -> Task<Message> {
        let modified: Vec<String> = ids
            .iter()
            .filter_map(|&id| self.tab_index(id))
            .filter(|&i| self.tabs[i].editor.is_modified())
            .map(|i| self.tabs[i].editor.file_name())
            .collect();
        if modified.is_empty() {
            return self.close_ids(&ids);
        }
        let mut names = modified.iter().take(5).map(|n| format!("• {n}")).collect::<Vec<_>>().join("\n");
        if modified.len() > 5 {
            names.push_str(&format!("\n… e mais {}", modified.len() - 5));
        }
        let title = format!("Fechar {} abas?", ids.len());
        let dialog = Dialog::new(title, format!("Com alterações não salvas:\n{names}"))
            .button("Salvar e fechar", ButtonKind::Primary, CoreDialog::CloseMany { ids: ids.clone(), save: true })
            .button("Descartar alterações", ButtonKind::Danger, CoreDialog::CloseMany { ids, save: false })
            .cancel("Cancelar");
        self.open_dialog(dialog, dialog::Owner::Core)
    }

    fn tab_menu(&self, id: u64, at: iced::Point) -> tea_core::menu::Menu {
        use tea_core::icons::Icon;
        let i = self.tab_index(id).unwrap_or(0);
        let n = self.tabs.len();
        let mut menu = tea_core::menu::Menu::new(at).item(Some(Icon::Close), "Fechar", CoreMenu::Close(id));
        menu = if n > 1 { menu.item(None, "Fechar outras", CoreMenu::CloseOthers(id)) } else { menu.disabled(None, "Fechar outras") };
        menu = if i + 1 < n {
            menu.item(None, "Fechar à direita", CoreMenu::CloseRight(id))
        } else {
            menu.disabled(None, "Fechar à direita")
        };
        menu = menu.item(None, "Fechar as salvas", CoreMenu::CloseSaved).item(None, "Fechar todas", CoreMenu::CloseAll);
        if let Some(path) = self.tabs.get(i).and_then(|t| t.editor.path()) {
            let rel = display_rel(path, &self.root);
            let dir = path.parent().unwrap_or(&self.root).to_path_buf();
            menu = menu
                .separator()
                .item(Some(Icon::Copy), "Copiar caminho", CoreMenu::CopyPath(path.display().to_string()))
                .item(None, "Copiar caminho relativo", CoreMenu::CopyPath(rel));
            if self.extensions.iter().any(|e| e.id() == "filetree") {
                menu = menu.item(Some(Icon::Files), "Revelar na árvore", CoreMenu::Reveal(id));
            }
            menu = menu.item(Some(Icon::LinkExternal), "Abrir no gerenciador de arquivos", CoreMenu::OpenInFileManager(dir));
        }
        menu
    }

    fn core_menu(&mut self, action: CoreMenu) -> Task<Message> {
        let ids = |app: &Self| app.tabs.iter().map(|t| t.id).collect::<Vec<u64>>();
        match action {
            CoreMenu::Close(id) => self.close_tab(id, false),
            CoreMenu::CloseOthers(id) => {
                let others = ids(self).into_iter().filter(|&t| t != id).collect();
                self.close_many(others)
            }
            CoreMenu::CloseRight(id) => {
                let all = ids(self);
                let right = all.iter().position(|&t| t == id).map_or_else(Vec::new, |i| all[i + 1..].to_vec());
                self.close_many(right)
            }
            CoreMenu::CloseSaved => {
                let saved: Vec<u64> = self.tabs.iter().filter(|t| !t.editor.is_modified()).map(|t| t.id).collect();
                self.close_ids(&saved)
            }
            CoreMenu::CloseAll => {
                let all = ids(self);
                self.close_many(all)
            }
            CoreMenu::CopyPath(text) => {
                let note = self.notify(format!("Copiado: {text}"));
                Task::batch([iced::clipboard::write(text), note])
            }
            CoreMenu::Reveal(id) => {
                let Some(i) = self.tab_index(id) else { return Task::none() };
                let activate = self.set_active(i);
                Task::batch([activate, self.run_command("filetree.reveal")])
            }
            CoreMenu::OpenInFileManager(dir) => match tea_core::os::open_in_file_manager(&dir) {
                Ok(()) => Task::none(),
                Err(e) => self.notify(format!("Não foi possível abrir o gerenciador de arquivos: {e}")),
            },
        }
    }

    // ---- Diálogos --------------------------------------------------------------

    // ---- Barra de ícones e tamanho dos painéis ----------------------------------

    /// Mostra/oculta o painel. Ao abrir, avisa o dono (o terminal sobe o shell, a busca pega o teclado...).
    fn activity_click(&mut self, i: usize) -> Task<Message> {
        let was = self.panel_shown(i);
        self.set_panel_visible(i, Some(!was));
        let p = &self.panels[i];
        if was {
            // Escondeu: o teclado volta para o editor.
            if self.palette.is_none() {
                self.editor_mut().focus();
            }
            return Task::none();
        }
        let (ext, id, bottom) = (p.ext, p.id, p.slot == Slot::Bottom);
        if bottom {
            // Painéis de baixo costumam querer o teclado (terminal, busca).
            self.editor_mut().lose_focus();
        }
        self.call_ext(ext, |e, ctx| e.on_event(&CoreEvent::PanelShown(id), ctx))
    }

    /// O painel está na tela? (No slot de baixo, só o painel em destaque aparece.)
    fn panel_shown(&self, i: usize) -> bool {
        let p = &self.panels[i];
        p.visible && (p.slot != Slot::Bottom || self.active_bottom == Some(i))
    }

    fn resize_panel(&mut self, edge: Edge, delta: f32) {
        use ui_state::{MIN_BOTTOM, MIN_SIDE, resize};
        let s = &mut self.ui_state;
        let w = self.window_size.map(|z| z.width);
        let h = self.window_size.map(|z| z.height);
        match edge {
            Edge::Left => s.left_width = resize(s.left_width, delta, MIN_SIDE, w.map(|w| w - s.right_width)),
            Edge::Right => s.right_width = resize(s.right_width, -delta, MIN_SIDE, w.map(|w| w - s.left_width)),
            // Abas e barra de status também precisam caber.
            Edge::Bottom => s.bottom_height = resize(s.bottom_height, -delta, MIN_BOTTOM, h.map(|h| h - 60.0)),
        }
    }

    fn activity_bar(&self) -> Element<'_, Message> {
        let item = |i: usize| -> Option<Element<'_, Message>> {
            let p = &self.panels[i];
            let ic = p.icon?;
            let shown = self.panel_shown(i);
            let ext_id = self.extensions[p.ext].id();
            let key = [format!("{ext_id}.toggle"), format!("{ext_id}.focus")]
                .iter()
                .find_map(|c| self.keymap.chord_for(c))
                .map(|k| format!(" ({k})"))
                .unwrap_or_default();
            let b = button(container(tea_core::icons::icon(ic).size(18)).center_x(Fill))
                .on_press(Message::Activity(i))
                .width(Fill)
                .padding([7, 0])
                .style(move |t: &Theme, status| activity_style(t, status, shown));
            let tip = container(text(format!("{}{key}", p.title)).size(12)).padding([3, 8]).style(container::bordered_box);
            Some(iced::widget::tooltip(b, tip, iced::widget::tooltip::Position::Right).gap(4).into())
        };
        let side = (0..self.panels.len())
            .filter(|&i| matches!(self.panels[i].slot, Slot::Left | Slot::Right))
            .filter_map(item);
        let bottom = (0..self.panels.len()).filter(|&i| self.panels[i].slot == Slot::Bottom).filter_map(item);
        container(column![column(side).spacing(2), iced::widget::space::vertical(), column(bottom).spacing(2)].height(Fill))
            .width(38)
            .height(Fill)
            .padding([2, 3])
            .into()
    }

    // ---- Configurações ---------------------------------------------------------

    fn open_settings(&mut self) -> Task<Message> {
        self.palette = None;
        self.menu = None;
        self.help = None;
        self.editor_mut().lose_focus();
        let st = settings::Settings::new(self.server_rows(), self.ext_rows(), config::project_config_path(&self.root));
        self.settings = Some(st);
        operation::focus(iced::widget::Id::unique())
    }

    fn close_settings(&mut self) {
        self.settings = None;
        if self.dialog.is_none() && self.palette.is_none() {
            self.editor_mut().focus();
        }
    }

    fn ext_rows(&self) -> Vec<settings::ExtRow> {
        self.all_extensions
            .iter()
            .map(|&id| {
                let loaded = self.extensions.iter().position(|e| e.id() == id);
                let movable = match loaded {
                    Some(i) => self.panels.iter().any(|p| p.ext == i && p.default_slot != Slot::Status),
                    None => self.config.extension(id).slot.is_some(),
                };
                settings::ExtRow { id, loaded: loaded.is_some(), movable }
            })
            .collect()
    }

    /// Servidores embutidos + os do config, com o comando efetivo e se ele existe no PATH.
    fn server_rows(&self) -> Vec<settings::ServerRow> {
        let mut rows: Vec<settings::ServerRow> = tea_lsp::default_servers()
            .into_iter()
            .map(|s| {
                let command = self.config.lsp.servers.get(&s.name).and_then(|o| o.command.clone()).unwrap_or(s.command);
                settings::ServerRow { name: s.name, found: tea_lsp::find_in_path(&command).is_some(), command }
            })
            .collect();
        for (name, o) in &self.config.lsp.servers {
            if let Some(command) = &o.command
                && !rows.iter().any(|r| &r.name == name)
            {
                let found = tea_lsp::find_in_path(command).is_some();
                rows.push(settings::ServerRow { name: name.clone(), command: command.clone(), found });
            }
        }
        rows
    }

    fn settings_target_file(&self) -> Result<tea_core::config_edit::ConfigFile, String> {
        use tea_core::config_edit::ConfigFile;
        match self.settings.as_ref().map(|s| s.target) {
            Some(settings::Target::Project) => ConfigFile::open_project(&config::project_config_path(&self.root)),
            _ => match &self.global_config {
                Some(p) => ConfigFile::open_global(p),
                None => Err("não foi possível descobrir a pasta de config (HOME não definido)".into()),
            },
        }
    }

    /// Muda o arquivo escolhido na tela, grava e recarrega (sem anunciar).
    fn edit_config(&mut self, f: impl FnOnce(&mut tea_core::config_edit::ConfigFile)) -> Task<Message> {
        let mut file = match self.settings_target_file() {
            Ok(file) => file,
            Err(e) => return self.notify(format!("Config: {e}")),
        };
        f(&mut file);
        if let Err(e) = file.save() {
            return self.notify(format!("Erro ao gravar {}: {e}", file.path().display()));
        }
        self.reload_config_with(false)
    }

    /// Remove um atalho. Se outra camada (padrão ou global) ainda ligar a tecla ao comando, grava `""` para desligar.
    fn unbind(&mut self, chord: String, command: String) -> Task<Message> {
        let read = |p: Option<&Path>| p.and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
        let global = read(self.global_config.as_deref());
        let project = read(Some(&config::project_config_path(&self.root)));
        let to_project = self.settings.as_ref().is_some_and(|s| s.target == settings::Target::Project);
        self.edit_config(move |f| {
            f.remove_key(&chord);
            let (g, p) = if to_project { (global, f.text()) } else { (f.text(), project) };
            if still_bound(&g, &p, &chord, &command) {
                f.bind(&chord, "");
            }
        })
    }

    fn command_title(&self, id: &str) -> String {
        self.commands.iter().find(|c| c.id == id).map_or_else(|| id.to_string(), |c| c.title.clone())
    }

    fn settings_key(
        &mut self,
        key: &keyboard::Key,
        modifiers: keyboard::Modifiers,
        command: Option<&str>,
        captured: bool,
    ) -> Task<Message> {
        let escape = *key == keyboard::Key::Named(keyboard::key::Named::Escape);
        let Some(st) = &mut self.settings else { return Task::none() };
        let Some(cmd) = st.recording.clone() else {
            if escape || command == Some("settings.open") {
                self.close_settings();
            } else if let Some(id) = command.filter(|_| !captured) {
                self.close_settings();
                return self.run_command(id);
            }
            return Task::none();
        };
        if escape {
            st.recording = None;
            st.record_hint = None;
            return Task::none();
        }
        // Só modificador (Ctrl sozinho...): espera o resto.
        let Some(chord) = Chord::from_event(key, modifiers) else { return Task::none() };
        if !chord.is_bindable() {
            st.record_hint = Some("Use Ctrl, Alt ou uma tecla especial (F1–F12, PageUp…). Esc cancela.".into());
            return Task::none();
        }
        st.recording = None;
        st.record_hint = None;
        let previous = self.keymap.command_for(&chord).map(str::to_string);
        let (shown, title) = (chord.to_string(), self.command_title(&cmd));
        let note = match previous {
            Some(p) if p == cmd => format!("{shown} já era de \"{title}\""),
            Some(p) => format!("{shown} era de \"{}\"; agora é de \"{title}\"", self.command_title(&p)),
            None => format!("{shown} → \"{title}\""),
        };
        let config_chord = chord.to_config();
        let saved = self.edit_config(|f| f.bind(&config_chord, &cmd));
        Task::batch([saved, self.notify(note)])
    }

    fn settings_update(&mut self, msg: settings::Msg) -> Task<Message> {
        use settings::Msg as M;
        let Some(st) = &mut self.settings else { return Task::none() };
        match msg {
            M::Tab(tab) => {
                st.tab = tab;
                st.recording = None;
                if tab == settings::Tab::Shortcuts {
                    return operation::focus(settings::filter_id());
                }
                Task::none()
            }
            M::Target(t) => {
                st.target = t;
                Task::none()
            }
            M::Filter(q) => {
                st.filter = q;
                Task::none()
            }
            M::Record(cmd) => {
                st.recording = Some(cmd);
                st.record_hint = None;
                // A tecla não pode cair no campo de filtro.
                operation::focus(iced::widget::Id::unique())
            }
            M::CancelRecord => {
                st.recording = None;
                st.record_hint = None;
                Task::none()
            }
            M::Theme(t) => self.edit_config(|f| f.set_theme(&t.0)),
            M::FontSize(v) => {
                let v = v.clamp(settings::FONT_MIN, settings::FONT_MAX);
                self.edit_config(|f| f.set_font_size(v))
            }
            M::Splash(on) => self.edit_config(|f| f.set_splash(on)),
            M::ActivityBar(on) => self.edit_config(|f| f.set_activity_bar(on)),
            M::ExtEnabled(id, on) => self.edit_config(|f| f.set_extension_enabled(&id, on)),
            M::ExtSlot(id, slot) => self.edit_config(|f| f.set_extension_slot(&id, slot.0)),
            M::LspEnabled(on) => self.edit_config(|f| f.set_lsp_enabled(on)),
            M::LspServer(name, on) => self.edit_config(|f| f.set_lsp_server_enabled(&name, on)),
            M::RestartLsp => self.run_command("lsp.restart"),
            M::Unbind { chord, command } => self.unbind(chord, command),
            M::OpenFile => {
                let global = st.target == settings::Target::Global;
                self.close_settings();
                self.open_config(global)
            }
            M::Close => {
                self.close_settings();
                Task::none()
            }
        }
    }

    fn open_help(&mut self) -> Task<Message> {
        self.settings = None;
        self.palette = None;
        self.menu = None;
        self.editor_mut().lose_focus();
        self.help = Some(help::Help::new(help::Tab::Shortcuts));
        operation::focus(help::input_id())
    }

    fn close_help(&mut self) {
        self.help = None;
        if self.dialog.is_none() && self.palette.is_none() {
            self.editor_mut().focus();
        }
    }

    fn open_menu(&mut self, menu: tea_core::menu::Menu, owner: dialog::Owner) -> Task<Message> {
        if self.dialog.is_some() {
            return Task::none();
        }
        self.palette = None;
        self.editor_mut().lose_focus();
        self.menu = Some(context_menu::Open::new(menu, owner));
        operation::focus(iced::widget::Id::unique())
    }

    fn choose_menu(&mut self, i: Option<usize>) -> Task<Message> {
        let Some(open) = self.menu.take() else { return Task::none() };
        match i.and_then(|i| open.action(i)) {
            Some(tea_core::menu::MenuAction::Message(msg)) => match open.owner {
                dialog::Owner::Ext(i) => self.call_ext(i, |e, ctx| e.update(msg, ctx)),
                dialog::Owner::Core => match msg.downcast_ref::<CoreMenu>() {
                    Some(action) => self.core_menu(action.clone()),
                    None => Task::none(),
                },
            },
            Some(tea_core::menu::MenuAction::Command(id)) => self.run_command(&id),
            None => Task::none(),
        }
    }

    fn open_dialog(&mut self, dialog: Dialog, owner: dialog::Owner) -> Task<Message> {
        self.palette = None;
        self.menu = None;
        self.editor_mut().lose_focus();
        self.dialog = Some(dialog::Open::new(dialog, owner));
        // Ninguém embaixo do diálogo fica com o teclado (campos de texto, terminal).
        operation::focus(iced::widget::Id::unique())
    }

    fn choose_dialog(&mut self, choice: Option<usize>) -> Task<Message> {
        let Some(open) = self.dialog.take() else { return Task::none() };
        let action = open.action(choice);
        let task = match (open.owner, action) {
            (dialog::Owner::Ext(i), Some(msg)) => self.call_ext(i, |e, ctx| e.update(msg, ctx)),
            (dialog::Owner::Core, Some(msg)) => match msg.downcast_ref::<CoreDialog>() {
                Some(action) => self.core_dialog(action.clone()),
                None => Task::none(),
            },
            (_, None) => Task::none(),
        };
        // Diálogos do app vêm do editor: o teclado volta para ele.
        if open.owner == dialog::Owner::Core && self.dialog.is_none() && self.palette.is_none() {
            self.editor_mut().focus();
        }
        task
    }

    fn core_dialog(&mut self, action: CoreDialog) -> Task<Message> {
        match action {
            CoreDialog::DiscardAndClose(id) => self.close_tab(id, true),
            CoreDialog::SaveAndClose(id) => {
                let Some(i) = self.tab_index(id) else { return Task::none() };
                let activate = self.set_active(i);
                if self.editor().path().is_none() {
                    let note = self.notify("Escolha onde salvar; depois feche a aba de novo");
                    return Task::batch([activate, self.open_save_as(), note]);
                }
                let saved = self.save_active();
                let close = if self.editor().is_modified() { Task::none() } else { self.close_tab(id, true) };
                Task::batch([activate, saved, close])
            }
            CoreDialog::SaveAllAndQuit => {
                let (ok, task) = self.save_all();
                if ok { Task::batch([task, self.quit()]) } else { task }
            }
            CoreDialog::QuitWithoutSaving => self.quit(),
            CoreDialog::CloseMany { ids, save } => {
                let mut tasks = Vec::new();
                let mut ids = ids;
                if save {
                    let modified: Vec<u64> = ids.iter().copied().filter(|&id| self.is_tab_modified(id)).collect();
                    let (failed, task) = self.save_tabs(&modified);
                    tasks.push(task);
                    // As que não deu para salvar ficam abertas.
                    ids.retain(|id| !failed.contains(id));
                }
                tasks.push(self.close_ids(&ids));
                Task::batch(tasks)
            }
        }
    }

    fn open_palette(&mut self, palette: Palette) -> Task<Message> {
        self.menu = None;
        self.editor_mut().lose_focus();
        self.palette = Some(palette);
        Task::batch([operation::focus(palette::input_id()), operation::move_cursor_to_end(palette::input_id())])
    }

    /// `Ctrl+P`: abre já com a última lista conhecida e relê os arquivos numa thread.
    fn open_quick_open(&mut self) -> Task<Message> {
        let active = self.active_path().map(Path::to_path_buf);
        let recent: Vec<String> = self
            .tabs
            .iter()
            .filter_map(|t| t.editor.path())
            .filter(|p| Some(*p) != active.as_deref())
            .filter_map(|p| p.strip_prefix(&self.root).ok())
            .map(|p| p.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"))
            .collect();
        let open = self.open_palette(Palette::files(self.file_list.clone(), recent, true));
        let root = self.root.clone();
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(palette::list_files(&root));
        });
        let list = Task::future(async move { rx.await.ok() })
            .and_then(|files| Task::done(Message::FilesListed(std::sync::Arc::new(files))));
        Task::batch([open, list])
    }

    fn close_palette(&mut self) {
        self.palette = None;
        self.editor_mut().focus();
    }

    fn palette_update(&mut self, msg: palette::Msg) -> Task<Message> {
        let Some(p) = &mut self.palette else { return Task::none() };
        match msg {
            palette::Msg::Query(q) => {
                p.set_query(q);
                Task::none()
            }
            palette::Msg::Close => {
                self.close_palette();
                Task::none()
            }
            palette::Msg::Pick(i) => {
                let id = self.commands[i].id.clone();
                self.close_palette();
                self.run_command(&id)
            }
            palette::Msg::PickFile(i) => self.open_chosen_file(Some(i)),
            palette::Msg::Submit => match p.mode {
                palette::Mode::Commands => match p.matches(&self.commands).get(p.selected).copied() {
                    Some(i) => self.palette_update(palette::Msg::Pick(i)),
                    None => Task::none(),
                },
                palette::Mode::SaveAs => {
                    let input = p.query.clone();
                    self.close_palette();
                    self.save_as(&input)
                }
                palette::Mode::Files => self.open_chosen_file(None),
            },
        }
    }

    /// Modo arquivos da paleta: abre o escolhido (`index`) ou o selecionado, na linha se a busca for `nome:42`.
    fn open_chosen_file(&mut self, index: Option<usize>) -> Task<Message> {
        let Some((rel, line)) = self.palette.as_ref().and_then(|p| p.chosen_file(index)) else { return Task::none() };
        self.close_palette();
        let path = self.root.join(rel);
        match line {
            Some(line) => self.open_file_at(path, line, 1),
            None => self.open_file(path),
        }
    }

    fn on_key(&mut self, key: keyboard::Key, modifiers: keyboard::Modifiers, captured: bool) -> Task<Message> {
        use keyboard::key::Named;
        let chord = Chord::from_event(&key, modifiers);
        let command = chord.as_ref().and_then(|c| self.keymap.command_for(c)).map(str::to_string);

        // Menu de contexto aberto: só ele responde ao teclado.
        if let Some(m) = &mut self.menu {
            match key {
                keyboard::Key::Named(Named::Escape) => return self.choose_menu(None),
                keyboard::Key::Named(Named::Enter) => {
                    let selected = m.selected;
                    return self.choose_menu(selected);
                }
                keyboard::Key::Named(Named::ArrowDown) => m.move_selection(1),
                keyboard::Key::Named(Named::ArrowUp) => m.move_selection(-1),
                _ => {}
            }
            return Task::none();
        }
        // Ajuda aberta: Esc ou o próprio atalho fecham. Outro atalho da IDE (que a busca não usou)
        // fecha a ajuda e roda o comando: dá para ir direto para a paleta ou as configurações.
        if self.dialog.is_none() && self.help.is_some() {
            let escape = key == keyboard::Key::Named(Named::Escape);
            if escape || command.as_deref() == Some("help.open") {
                self.close_help();
            } else if let Some(id) = command.filter(|_| !captured) {
                self.close_help();
                return self.run_command(&id);
            }
            return Task::none();
        }
        // Configurações abertas: gravando tecla, ou mesma regra da ajuda.
        if self.dialog.is_none() && self.settings.is_some() {
            return self.settings_key(&key, modifiers, command.as_deref(), captured);
        }
        // Diálogo aberto: só ele responde ao teclado.
        if let Some(d) = &mut self.dialog {
            match key {
                keyboard::Key::Named(Named::Escape) => return self.choose_dialog(None),
                keyboard::Key::Named(Named::Enter) => {
                    let selected = d.selected;
                    return self.choose_dialog(selected);
                }
                keyboard::Key::Named(Named::ArrowRight) => d.move_selection(1),
                keyboard::Key::Named(Named::ArrowLeft) => d.move_selection(-1),
                keyboard::Key::Named(Named::Tab) => d.move_selection(if modifiers.shift() { -1 } else { 1 }),
                _ => {}
            }
            return Task::none();
        }
        if let Some(p) = &mut self.palette {
            match key {
                keyboard::Key::Named(Named::Escape) => self.close_palette(),
                keyboard::Key::Named(Named::ArrowDown | Named::ArrowUp) => {
                    let total = p.total(&self.commands);
                    let delta = if key == keyboard::Key::Named(Named::ArrowDown) { 1 } else { -1 };
                    p.move_selection(delta, total);
                    return p.scroll_to_selected();
                }
                // O atalho do próprio modo fecha; o do outro modo troca (Ctrl+P ⇄ Ctrl+Shift+P).
                _ if matches!(command.as_deref(), Some("palette.open" | "file.quick_open")) => {
                    let files = command.as_deref() == Some("file.quick_open");
                    if files == (p.mode == palette::Mode::Files) {
                        self.close_palette();
                    } else if files {
                        return self.open_quick_open();
                    } else {
                        return self.open_palette(Palette::commands());
                    }
                }
                _ => {}
            }
            return Task::none();
        }
        // O editor (ou outro widget) já tratou a tecla: ele tem prioridade.
        if captured {
            return Task::none();
        }
        match command {
            Some(id) => self.run_command(&id),
            None => Task::none(),
        }
    }

    // ---- Loop do iced ------------------------------------------------------

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(id, msg) => {
                let Some(i) = self.tab_index(id) else { return Task::none() };
                if msg.is_save_request() {
                    return if i == self.active { self.save_active() } else { Task::none() };
                }
                let task = self.tabs[i].editor.update(msg).map(move |m| Message::Editor(id, m));
                if i == self.active { Task::batch([task, self.emit_status(false)]) } else { task }
            }
            // O foco do editor sai por clique no painel (`MousePressed`) ou por comando,
            // não aqui: painéis também mandam mensagens sozinhos (ex.: terminal ao mudar de tamanho).
            Message::Ext(i, msg) | Message::ExtSub(i, msg) => self.call_ext(i, |e, ctx| e.update(msg, ctx)),
            Message::SelectTab(id) => match self.tab_index(id) {
                Some(i) => self.set_active(i),
                None => Task::none(),
            },
            Message::CloseTab(id) => self.close_tab(id, false),
            Message::TabMenu(id, at) => {
                let menu = self.tab_menu(id, at);
                self.open_menu(menu, dialog::Owner::Core)
            }
            Message::SelectBottom(i) => {
                self.active_bottom = Some(i);
                Task::none()
            }
            Message::Key { key, modifiers, captured } => self.on_key(key, modifiers, captured),
            Message::Palette(msg) => self.palette_update(msg),
            Message::Disk(watcher::Change::Paths(paths)) => self.disk_changed(&paths),
            Message::Disk(watcher::Change::Problem(msg)) => self.notify(msg),
            Message::FilesListed(files) => {
                self.file_list = files.clone();
                if let Some(p) = &mut self.palette
                    && p.mode == palette::Mode::Files
                {
                    p.set_files(files);
                }
                Task::none()
            }
            Message::Dialog(dialog::Msg::Choose(choice)) => self.choose_dialog(choice),
            Message::Menu(context_menu::Msg::Choose(i)) => self.choose_menu(Some(i)),
            Message::Menu(context_menu::Msg::Close) => self.choose_menu(None),
            Message::Picked(picked) => self.picked(picked),
            Message::Settings(msg) => self.settings_update(msg),
            Message::Activity(i) => self.activity_click(i),
            Message::Resize(edge, delta) => {
                self.resize_panel(edge, delta);
                Task::none()
            }
            Message::ResizeDone => {
                self.ui_state.save(self.state_path.as_deref());
                Task::none()
            }
            Message::ResizeReset(edge) => {
                let d = ui_state::UiState::default();
                match edge {
                    Edge::Left => self.ui_state.left_width = d.left_width,
                    Edge::Right => self.ui_state.right_width = d.right_width,
                    Edge::Bottom => self.ui_state.bottom_height = d.bottom_height,
                }
                self.ui_state.save(self.state_path.as_deref());
                Task::none()
            }
            Message::WindowResized(size) => {
                self.window_size = Some(size);
                Task::none()
            }
            Message::Help(help::Msg::Close) => {
                self.close_help();
                Task::none()
            }
            Message::Help(msg) => {
                let was = self.help.as_ref().map(help::Help::tab);
                if let Some(h) = &mut self.help {
                    h.update(msg);
                }
                // Voltou para "Atalhos": o campo de busca pega o teclado de novo.
                let now = self.help.as_ref().map(help::Help::tab);
                if was != now && now == Some(help::Tab::Shortcuts) { operation::focus(help::input_id()) } else { Task::none() }
            }
            Message::Frame(now) => {
                self.editor_mut().tick(now);
                if let Some(s) = &mut self.splash {
                    s.tick(now);
                    if s.is_finished() {
                        self.splash = None;
                        if self.palette.is_none() {
                            self.editor_mut().focus();
                        }
                    }
                }
                Task::none()
            }
            Message::PointerOverPanel(over) => {
                self.pointer_over_panel = over;
                Task::none()
            }
            Message::MousePressed => {
                if self.pointer_over_panel {
                    self.editor_mut().lose_focus();
                }
                Task::none()
            }
            Message::CloseRequested => self.request_quit(),
            Message::Lsp(event) => self.handle_lsp(event),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // `listen_with` vê também as teclas que um widget consumiu (`captured`).
        let input = iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => Some(Message::Key {
                key,
                modifiers,
                captured: status == iced::event::Status::Captured,
            }),
            iced::Event::Mouse(mouse::Event::ButtonPressed(_)) => Some(Message::MousePressed),
            _ => None,
        });
        let close = window::close_requests().map(|_| Message::CloseRequested);
        let resized = window::resize_events().map(|(_, size)| Message::WindowResized(size));
        // Quadros só quando há animação (splash) ou hover esperando o mouse parar.
        let frames = if self.splash.is_some() || self.editor().needs_tick() {
            window::frames().map(Message::Frame)
        } else {
            Subscription::none()
        };
        // `with(i)` e não uma closure: `map` não aceita closures que capturam.
        let extensions = Subscription::batch(
            self.extensions
                .iter()
                .enumerate()
                .map(|(i, e)| e.subscription().with(i).map(|(i, m)| Message::ExtSub(i, m))),
        );
        let disk = Subscription::run_with(self.root.clone(), watcher::watch).map(Message::Disk);
        Subscription::batch([input, close, frames, extensions, resized, disk])
    }

    fn view(&self) -> Element<'_, Message> {
        let mut layers = stack![self.workspace_view()];
        if let Some(p) = &self.palette {
            // `opaque` na caixa: clique dentro dela não chega no "clique fora fecha".
            let overlay = container(opaque(p.view(&self.commands, &self.keymap).map(Message::Palette)))
                .width(Fill)
                .height(Fill)
                .padding(iced::Padding { top: 60.0, ..iced::Padding::ZERO })
                .align_x(iced::Center);
            // Clique fora fecha.
            layers = layers.push(opaque(mouse_area(overlay).on_press(Message::Palette(palette::Msg::Close))));
        }
        if let Some(st) = &self.settings {
            let view = st.view(&self.config, &self.keymap, &self.commands, self.global_config.as_deref());
            layers = layers.push(view.map(Message::Settings));
        }
        if let Some(h) = &self.help {
            let about = help::AboutInfo {
                root: &self.root,
                global_config: self.global_config.as_deref(),
                project_config: config::project_config_path(&self.root),
            };
            layers = layers.push(h.view(&self.commands, &self.keymap, about).map(Message::Help));
        }
        if let Some(m) = &self.menu {
            layers = layers.push(m.view(&self.keymap).map(Message::Menu));
        }
        if let Some(d) = &self.dialog {
            layers = layers.push(d.view().map(Message::Dialog));
        }
        if let Some(s) = &self.splash {
            layers = layers.push(opaque(s.view()));
        }
        layers.into()
    }

    fn workspace_view(&self) -> Element<'_, Message> {
        let tabs = scrollable(
            row(self.tabs.iter().enumerate().map(|(i, t)| {
                let style = if i == self.active { button::primary } else { button::text };
                let close =
                    button(text("×").size(13)).on_press(Message::CloseTab(t.id)).style(button::text).padding([0, 4]);
                let id = t.id;
                let tab = button(row![text(t.editor.title()).size(13), close].spacing(6).align_y(iced::Center))
                    .on_press(Message::SelectTab(id))
                    .style(style)
                    .padding([3, 8]);
                // Botão do meio fecha; botão direito abre o menu.
                let tab = mouse_area(tab).on_middle_press(Message::CloseTab(id));
                tea_core::menu::context_area(tab, move |at| Message::TabMenu(id, at))
            }))
            .spacing(2),
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new().width(2).scroller_width(2)));

        let tab = &self.tabs[self.active];
        let tab_id = tab.id;
        let editor = tab.editor.view().map(move |m| Message::Editor(tab_id, m));

        let mut center = column![tabs, editor].spacing(4).width(Fill).height(Fill);
        if let Some(bottom) = self.bottom_view() {
            let handle = divider::divider(
                divider::Axis::Horizontal,
                |d| Message::Resize(Edge::Bottom, d),
                Message::ResizeDone,
                Message::ResizeReset(Edge::Bottom),
            );
            center = center.push(handle).push(container(bottom).height(self.ui_state.bottom_height));
        }

        let mut body = row![].height(Fill);
        if self.config.ui.activity_bar {
            body = body.push(self.activity_bar()).push(rule::vertical(1));
        }
        if let Some(left) = self.slot_view(Slot::Left) {
            let handle = divider::divider(
                divider::Axis::Vertical,
                |d| Message::Resize(Edge::Left, d),
                Message::ResizeDone,
                Message::ResizeReset(Edge::Left),
            );
            body = body.push(container(left).width(self.ui_state.left_width).height(Fill)).push(handle);
        }
        body = body.push(center);
        if let Some(right) = self.slot_view(Slot::Right) {
            let handle = divider::divider(
                divider::Axis::Vertical,
                |d| Message::Resize(Edge::Right, d),
                Message::ResizeDone,
                Message::ResizeReset(Edge::Right),
            );
            body = body.push(handle).push(container(right).width(self.ui_state.right_width).height(Fill));
        }

        let mut page = column![body].spacing(4).padding(6);
        if let Some(status) = self.slot_view(Slot::Status) {
            page = page.push(rule::horizontal(1)).push(status);
        }
        page.into()
    }

    /// Slot inferior: abas com os painéis visíveis e só o painel em destaque aberto.
    fn bottom_view(&self) -> Option<Element<'_, Message>> {
        let visible: Vec<usize> =
            (0..self.panels.len()).filter(|&i| self.panels[i].slot == Slot::Bottom && self.panels[i].visible).collect();
        let active = self.active_bottom.filter(|i| visible.contains(i)).or(visible.first().copied())?;
        let tabs = row(visible.iter().map(|&i| {
            let style = if i == active { button::secondary } else { button::text };
            button(text(self.panels[i].title.to_uppercase()).size(11))
                .on_press(Message::SelectBottom(i))
                .style(style)
                .padding([2, 8])
                .into()
        }))
        .spacing(2);
        let p = &self.panels[active];
        let ext = p.ext;
        let body = self.extensions[ext].view(p.id).map(move |m| Message::Ext(ext, m));
        let content = column![tabs, self.pass_ide_keys(body)].spacing(4).padding(4);
        Some(
            mouse_area(content)
                .on_enter(Message::PointerOverPanel(true))
                .on_exit(Message::PointerOverPanel(false))
                .into(),
        )
    }

    /// Atalhos Ctrl+Shift da IDE valem mesmo com o foco no painel (ver `Keymap::wins_over_panels`).
    fn pass_ide_keys<'a>(&'a self, body: Element<'a, Message>) -> Element<'a, Message> {
        pass_keys::PassKeys::new(body, |key, modifiers| {
            Chord::from_event(key, modifiers).is_some_and(|c| self.keymap.wins_over_panels(&c))
        })
        .into()
    }

    /// Painéis visíveis de um slot, um embaixo do outro (ou lado a lado na barra de status).
    fn slot_view(&self, slot: Slot) -> Option<Element<'_, Message>> {
        let panels: Vec<&Panel> = self.panels.iter().filter(|p| p.slot == slot && p.visible).collect();
        if panels.is_empty() {
            return None;
        }
        let views = panels.into_iter().map(|p| {
            let ext = p.ext;
            let body = self.extensions[ext].view(p.id).map(move |m| Message::Ext(ext, m));
            if slot == Slot::Status {
                body
            } else {
                column![text(p.title.to_uppercase()).size(11).style(text::secondary), self.pass_ide_keys(body)]
                    .spacing(4)
                    .into()
            }
        });
        let content: Element<'_, Message> = if slot == Slot::Status {
            row(views).spacing(16).into()
        } else {
            column(views).spacing(8).padding(4).into()
        };
        Some(
            mouse_area(content)
                .on_enter(Message::PointerOverPanel(true))
                .on_exit(Message::PointerOverPanel(false))
                .into(),
        )
    }
}

/// "dark", "light" ou o nome de um tema do iced (sem diferenciar maiúsculas).
fn resolve_theme(name: &str) -> (Theme, Option<String>) {
    match name.trim().to_lowercase().as_str() {
        "dark" | "escuro" => (DARK_THEME, None),
        "light" | "claro" => (LIGHT_THEME, None),
        other => match Theme::ALL.iter().find(|t| t.to_string().to_lowercase() == other) {
            Some(t) => (t.clone(), None),
            None => (DARK_THEME, Some(format!("tema desconhecido {name:?}, usando \"dark\""))),
        },
    }
}

/// Ícone da barra de ícones: destacado quando o painel está aberto.
fn activity_style(theme: &Theme, status: button::Status, shown: bool) -> button::Style {
    let p = theme.extended_palette();
    let text_color = if shown { p.background.base.text } else { p.background.base.text.scale_alpha(0.55) };
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(p.background.strong.color.into()),
        _ if shown => Some(p.background.weak.color.into()),
        _ => None,
    };
    button::Style {
        background,
        text_color,
        border: iced::Border {
            radius: 5.0.into(),
            width: if shown { 1.0 } else { 0.0 },
            color: p.primary.base.color.scale_alpha(0.6),
        },
        ..button::Style::default()
    }
}

/// Com estes textos de config global e de projeto, `chord` ainda roda `command`?
fn still_bound(global: &str, project: &str, chord: &str, command: &str) -> bool {
    let layers = [
        ("padrão".to_string(), config::DEFAULT_CONFIG.to_string()),
        ("global".to_string(), global.to_string()),
        ("projeto".to_string(), project.to_string()),
    ];
    let merged = Config::from_layers(&layers);
    let (keymap, _) = Keymap::from_pairs(merged.keys.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    Chord::parse(chord).ok().and_then(|c| keymap.command_for(&c).map(str::to_string)).as_deref() == Some(command)
}

fn expand_home(input: &str) -> PathBuf {
    match input.strip_prefix("~/") {
        Some(rest) => std::env::var_os("HOME").map(|h| PathBuf::from(h).join(rest)).unwrap_or_else(|| input.into()),
        None => PathBuf::from(input),
    }
}

fn display_rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbind_rule() {
        // Atalho padrão: tirar a linha não basta, ele continua valendo.
        assert!(still_bound("", "", "ctrl+b", "filetree.toggle"));
        // Atalho criado pelo usuário em cima de um padrão: tirar a linha devolve o padrão.
        assert!(!still_bound("", "", "ctrl+b", "meu.comando"));
        // Já desligado no global.
        assert!(!still_bound("[keys]\n\"ctrl+b\" = \"\"", "", "ctrl+b", "filetree.toggle"));
        // Ligado no global, mexendo no projeto: continua valendo.
        assert!(still_bound("[keys]\n\"ctrl+alt+x\" = \"a.b\"", "", "ctrl+alt+x", "a.b"));
    }

    #[test]
    fn theme_names() {
        assert_eq!(resolve_theme("dark").0, DARK_THEME);
        assert_eq!(resolve_theme("Claro").0, LIGHT_THEME);
        assert_eq!(resolve_theme("nord").0, Theme::Nord);
        let (t, err) = resolve_theme("inexistente");
        assert_eq!(t, DARK_THEME);
        assert!(err.is_some());
    }

    #[test]
    fn default_keys_point_to_existing_commands() {
        let mut ids: Vec<_> = CORE_COMMANDS.iter().map(|(id, _)| *id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CORE_COMMANDS.len(), "ids repetidos");

        let ext_commands = ["filetree.toggle", "filetree.reveal", "search.focus", "problems.focus", "problems.next", "terminal.focus", "git.focus"];
        let cfg = Config::from_layers(&[("padrão".into(), config::DEFAULT_CONFIG.into())]);
        for (chord, cmd) in &cfg.keys {
            assert!(ids.contains(&cmd.as_str()) || ext_commands.contains(&cmd.as_str()), "{chord} -> {cmd}");
        }
    }
}
