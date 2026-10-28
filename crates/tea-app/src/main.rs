//! Binário da IDE TEA.
//!
//! Uso: `idetea [--no-splash] [caminho...]` (ver `cli.rs`).

mod cli;
mod lsp;
mod palette;
mod splash;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;

use iced::widget::{button, column, container, mouse_area, opaque, operation, row, rule, scrollable, stack, text};
use iced::{Element, Fill, Subscription, Task, Theme, keyboard, mouse, window};
use tea_core::config::{self, Config};
use tea_core::keymap::{Chord, Keymap};
use tea_core::{AnyExtension, Context, CoreEvent, Diagnostic, EditorStatus, ExtMessage, PanelId, Request, Severity, Slot};
use tea_editor::{Editor, EditorAction, EditorMessage};

use palette::{CommandInfo, Palette};
use splash::Splash;

/// Argumentos já interpretados, para o `boot` do iced (que não recebe parâmetros).
static LAUNCH: OnceLock<cli::Launch> = OnceLock::new();
static STARTED: OnceLock<Instant> = OnceLock::new();

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
    list
}

/// Comandos do núcleo: (id, título).
const CORE_COMMANDS: &[(&str, &str)] = &[
    ("palette.open", "Paleta de comandos"),
    ("file.new", "Arquivo: novo"),
    ("file.save", "Arquivo: salvar"),
    ("file.save_as", "Arquivo: salvar como…"),
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
const SIDE_WIDTH: f32 = 260.0;

struct Tab {
    id: u64,
    editor: Editor,
}

struct Panel {
    ext: usize,
    id: PanelId,
    title: String,
    slot: Slot,
    visible: bool,
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
    splash: Option<Splash>,
    /// Ponteiro sobre um painel lateral: um clique ali tira o foco do editor.
    pointer_over_panel: bool,
    /// Aba com alterações à espera de um segundo "fechar" para descartar.
    pending_close: Option<u64>,
    pending_quit: bool,
    last_status: EditorStatus,
}

#[derive(Debug, Clone)]
enum Message {
    Editor(u64, EditorMessage),
    Ext(usize, ExtMessage),
    SelectTab(u64),
    CloseTab(u64),
    SelectBottom(usize),
    Key { key: keyboard::Key, modifiers: keyboard::Modifiers, captured: bool },
    Palette(palette::Msg),
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
        let extensions =
            builtin_extensions().into_iter().filter(|e| config.extension(e.id()).enabled).collect();

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
            splash: show_splash.then(|| Splash::new(started)),
            pointer_over_panel: false,
            pending_close: None,
            pending_quit: false,
            last_status: EditorStatus::default(),
            config,
        };
        app.push_tab(Editor::scratch(""));

        let mut tasks = vec![Task::run(lsp_rx, Message::Lsp)];
        // Extensões primeiro: assim elas recebem os eventos dos arquivos abertos abaixo.
        for i in 0..app.extensions.len() {
            tasks.push(app.call_ext(i, |e, ctx| e.activate(ctx)));
        }
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
                    let ext_id = self.extensions[ext].id();
                    let slot = self.config.extension(ext_id).slot.unwrap_or(slot);
                    self.panels.push(Panel { ext, id, title, slot, visible });
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
                Request::PathRenamed { from, to } => tasks.push(self.path_renamed(&from, &to)),
                Request::PathDeleted(path) => tasks.push(self.path_deleted(&path)),
            }
        }
        Task::batch(tasks)
    }

    fn broadcast(&mut self, event: CoreEvent) -> Task<Message> {
        let tasks: Vec<_> =
            (0..self.extensions.len()).map(|i| self.call_ext(i, |e, ctx| e.on_event(&event, ctx))).collect();
        Task::batch(tasks)
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
        self.pending_close = None;
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
        if editor.is_modified() && !force && self.pending_close != Some(id) {
            self.pending_close = Some(id);
            let name = editor.file_name();
            return self.notify(format!("\"{name}\" tem alterações. Feche de novo para descartar, ou salve."));
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

    fn open_save_as(&mut self) -> Task<Message> {
        let initial = match self.active_path() {
            Some(p) => display_rel(p, &self.root),
            None => String::new(),
        };
        self.open_palette(Palette::save_as(initial))
    }

    fn save_as(&mut self, input: &str) -> Task<Message> {
        let input = input.trim();
        if input.is_empty() {
            return Task::none();
        }
        let path = expand_home(input);
        let path = if path.is_absolute() { path } else { self.root.join(path) };
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
                self.pending_close = None;
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
        let msg = if problems.is_empty() {
            "Config recarregado (ligar/desligar extensões só vale ao reiniciar)".to_string()
        } else {
            format!("Config recarregado com problemas: {}", problems.join(" · "))
        };
        Task::batch([self.broadcast(CoreEvent::ConfigReloaded), self.notify(msg)])
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
        if id != "app.quit" {
            self.pending_quit = false;
        }
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
            "file.new" => {
                let i = self.push_tab(Editor::scratch(""));
                self.set_active(i)
            }
            "file.save" => self.save_active(),
            "file.save_as" => self.open_save_as(),
            "file.close" => self.close_tab(self.tabs[self.active].id, false),
            "tab.next" => self.set_active((self.active + 1) % self.tabs.len()),
            "tab.prev" => self.set_active((self.active + self.tabs.len() - 1) % self.tabs.len()),
            "theme.toggle" => {
                let next = if self.theme.extended_palette().is_dark { LIGHT_THEME } else { DARK_THEME };
                self.set_theme(next);
                Task::none()
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
        Task::none()
    }

    fn request_quit(&mut self) -> Task<Message> {
        let unsaved = self.tabs.iter().filter(|t| t.editor.is_modified()).count();
        if unsaved > 0 && !self.pending_quit {
            self.pending_quit = true;
            return self.notify(format!("{unsaved} aba(s) com alterações. Saia de novo para descartar."));
        }
        self.lsp.shutdown_all();
        iced::exit()
    }

    fn open_palette(&mut self, palette: Palette) -> Task<Message> {
        self.editor_mut().lose_focus();
        self.palette = Some(palette);
        Task::batch([operation::focus(palette::input_id()), operation::move_cursor_to_end(palette::input_id())])
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
            },
        }
    }

    fn on_key(&mut self, key: keyboard::Key, modifiers: keyboard::Modifiers, captured: bool) -> Task<Message> {
        use keyboard::key::Named;
        let chord = Chord::from_event(&key, modifiers);
        let command = chord.as_ref().and_then(|c| self.keymap.command_for(c)).map(str::to_string);

        if let Some(p) = &mut self.palette {
            match key {
                keyboard::Key::Named(Named::Escape) => self.close_palette(),
                keyboard::Key::Named(Named::ArrowDown | Named::ArrowUp) => {
                    let total = p.matches(&self.commands).len();
                    let delta = if key == keyboard::Key::Named(Named::ArrowDown) { 1 } else { -1 };
                    p.move_selection(delta, total);
                    return p.scroll_to_selected();
                }
                // O mesmo atalho fecha a palette.
                _ if command.as_deref() == Some("palette.open") => self.close_palette(),
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
            Message::Ext(i, msg) => {
                self.editor_mut().lose_focus();
                self.call_ext(i, |e, ctx| e.update(msg, ctx))
            }
            Message::SelectTab(id) => match self.tab_index(id) {
                Some(i) => self.set_active(i),
                None => Task::none(),
            },
            Message::CloseTab(id) => self.close_tab(id, false),
            Message::SelectBottom(i) => {
                self.active_bottom = Some(i);
                Task::none()
            }
            Message::Key { key, modifiers, captured } => self.on_key(key, modifiers, captured),
            Message::Palette(msg) => self.palette_update(msg),
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
        // Quadros só quando há animação (splash) ou hover esperando o mouse parar.
        let frames = if self.splash.is_some() || self.editor().needs_tick() {
            window::frames().map(Message::Frame)
        } else {
            Subscription::none()
        };
        Subscription::batch([input, close, frames])
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
                button(row![text(t.editor.title()).size(13), close].spacing(6).align_y(iced::Center))
                    .on_press(Message::SelectTab(t.id))
                    .style(style)
                    .padding([3, 8])
                    .into()
            }))
            .spacing(2),
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new().width(2).scroller_width(2)));

        let tab = &self.tabs[self.active];
        let tab_id = tab.id;
        let editor = tab.editor.view().map(move |m| Message::Editor(tab_id, m));

        let mut center = column![tabs, editor].spacing(4).width(Fill).height(Fill);
        if let Some(bottom) = self.bottom_view() {
            center = center.push(rule::horizontal(1)).push(container(bottom).height(260));
        }

        let mut body = row![].height(Fill);
        if let Some(left) = self.slot_view(Slot::Left) {
            body = body.push(container(left).width(SIDE_WIDTH).height(Fill)).push(rule::vertical(1));
        }
        body = body.push(center);
        if let Some(right) = self.slot_view(Slot::Right) {
            body = body.push(rule::vertical(1)).push(container(right).width(SIDE_WIDTH).height(Fill));
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
        let content = column![tabs, body].spacing(4).padding(4);
        Some(
            mouse_area(content)
                .on_enter(Message::PointerOverPanel(true))
                .on_exit(Message::PointerOverPanel(false))
                .into(),
        )
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
                column![text(p.title.to_uppercase()).size(11).style(text::secondary), body].spacing(4).into()
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

        let ext_commands = ["filetree.toggle", "filetree.reveal", "search.focus", "problems.focus", "problems.next"];
        let cfg = Config::from_layers(&[("padrão".into(), config::DEFAULT_CONFIG.into())]);
        for (chord, cmd) in &cfg.keys {
            assert!(ids.contains(&cmd.as_str()) || ext_commands.contains(&cmd.as_str()), "{chord} -> {cmd}");
        }
    }
}
