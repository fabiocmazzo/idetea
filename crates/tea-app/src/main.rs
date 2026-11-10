//! Binário da IDE TEA.
//!
//! Uso: `idetea [--no-splash] [caminho...]` (ver `cli.rs`).

// Textos da interface (`i18n/<idioma>/tea-app.ftl`). Antes dos `mod`: a macro vale para todos.
tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod backup;
mod binary;
mod calls;
mod cli;
mod context_menu;
mod debug;
mod debug_build;
mod detach;
mod desktop;
mod dialog;
mod editorconfig;
mod divider;
mod help;
mod ide_state;
mod mascot;
mod overlay;
mod lsp;
mod nav;
mod outline;
mod palette;
mod picker;
mod settings;
mod pass_keys;
mod splash;
mod tasks;
mod recent;
mod session;
mod ui_state;
mod watcher;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use iced::widget::{button, column, container, mouse_area, opaque, operation, row, rule, scrollable, stack, text};
use iced::{Element, Fill, Subscription, Task, Theme, keyboard, mouse, window};
use tea_core::config::{self, Config};
use tea_core::debug::{Event as DebugEvent, Status as DebugStatus};
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
/// `idetea.desktop` com este mesmo nome (`idetea --install-desktop` ou `just desktop` instalam).
#[cfg(target_os = "linux")]
const APP_ID: &str = "idetea";

/// O idioma de um subcomando: o mesmo da IDE (`[ui] language` do config do usuário e do projeto da pasta atual).
fn cli_language(cwd: &std::path::Path) {
    tea_core::i18n::init(&config::peek_language(config::global_config_path().as_deref(), Some(&config::project_config_path(cwd))));
}

fn main() -> iced::Result {
    STARTED.get_or_init(Instant::now);
    tea_core::os::ensure_gui_path();
    let cwd = std::env::current_dir().unwrap_or_default();
    let env_no_splash = cli::env_flag(std::env::var_os("IDETEA_NO_SPLASH"));
    match cli::parse(std::env::args_os().skip(1), &cwd, env_no_splash) {
        cli::Cli::Help => {
            cli_language(&cwd);
            println!("{}", cli::usage());
            Ok(())
        }
        cli::Cli::Version => {
            println!("idetea {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        cli::Cli::UiCheck(files) => {
            cli_language(&cwd);
            #[cfg(feature = "ext-ui")]
            std::process::exit(tea_ext_ui::check::run(&files, &cwd));
            #[cfg(not(feature = "ext-ui"))]
            {
                let _ = files;
                eprintln!("idetea: compilada sem a extensão Interfaces (feature ext-ui)");
                std::process::exit(2);
            }
        }
        cli::Cli::UiFix(args) => {
            cli_language(&cwd);
            #[cfg(feature = "ext-ui")]
            std::process::exit(tea_ext_ui::fix::run(&args, &cwd));
            #[cfg(not(feature = "ext-ui"))]
            {
                let _ = args;
                eprintln!("idetea: compilada sem a extensão Interfaces (feature ext-ui)");
                std::process::exit(2);
            }
        }
        cli::Cli::New(args) => {
            cli_language(&cwd);
            #[cfg(feature = "ext-ui")]
            std::process::exit(tea_ext_ui::novo::run(&args, &cwd));
            #[cfg(not(feature = "ext-ui"))]
            {
                let _ = args;
                eprintln!("idetea: compilada sem a extensão Interfaces (feature ext-ui)");
                std::process::exit(2);
            }
        }
        cli::Cli::Ask(args) => {
            cli_language(&cwd);
            #[cfg(feature = "ext-ui")]
            std::process::exit(tea_ext_ui::ask::run(&args, &cwd));
            #[cfg(not(feature = "ext-ui"))]
            {
                let _ = args;
                eprintln!("idetea: compilada sem a extensão Interfaces (feature ext-ui)");
                std::process::exit(2);
            }
        }
        cli::Cli::RunAgent(args) => {
            cli_language(&cwd);
            #[cfg(feature = "ext-terminal")]
            std::process::exit(tea_ext_terminal::cli::run(&args, &cwd));
            #[cfg(not(feature = "ext-terminal"))]
            {
                let _ = args;
                eprintln!("idetea: compilada sem a extensão do terminal (feature ext-terminal)");
                std::process::exit(2);
            }
        }
        cli::Cli::History(args) => {
            cli_language(&cwd);
            #[cfg(feature = "ext-ui")]
            std::process::exit(tea_ext_ui::history::run(&args, &cwd));
            #[cfg(not(feature = "ext-ui"))]
            {
                let _ = args;
                eprintln!("idetea: compilada sem a extensão Interfaces (feature ext-ui)");
                std::process::exit(2);
            }
        }
        cli::Cli::Status(args) => {
            cli_language(&cwd);
            std::process::exit(ide_state::run_status(&args, &cwd));
        }
        cli::Cli::Open(args) => {
            cli_language(&cwd);
            std::process::exit(ide_state::run_open(&args, &cwd));
        }
        cli::Cli::Skills(args) => {
            cli_language(&cwd);
            std::process::exit(tea_skills::run(&args, &cwd));
        }
        cli::Cli::Mascot(args) => {
            cli_language(&cwd);
            std::process::exit(mascot::run_cli(&args, &cwd));
        }
        cli::Cli::InstallDesktop => desktop::run(true),
        cli::Cli::UninstallDesktop => desktop::run(false),
        cli::Cli::Run(mut launch) => {
            // Aberta de um terminal: devolve o prompt (a cópia solta segue com os mesmos argumentos).
            if detach::wanted(launch.foreground) && detach::spawn().is_ok() {
                return Ok(());
            }
            if launch.menu && !launch.root_given {
                let recent = recent::path().map(|p| recent::Recent::load(&p)).unwrap_or_default();
                if let Some(root) = cli::menu_root(&launch.files, &recent.projects) {
                    launch.root = root;
                }
            }
            let _ = LAUNCH.set(launch);
            let saved = ui_state::UiState::load(ui_state::state_path().as_deref());
            // Antes de `boot`, que já abre os editores da sessão (ver `load_embedded`).
            tea_core::fonts::load_embedded();
            // Também antes da primeira aba: o editor monta as sintaxes na primeira vez que colore.
            tea_editor::syntaxes::register(tea_core::syntaxes::user_dir().as_deref());
            iced::application(App::boot, App::update, App::view)
                .title(App::title)
                .subscription(App::subscription)
                .theme(App::theme)
                .font(tea_core::icons::FONT_BYTES)
                .window(window::Settings {
                    size: saved.window().unwrap_or(window::Settings::default().size),
                    icon: window::icon::from_file_data(include_bytes!("../../../assets/idetea.png"), None).ok(),
                    platform_specific: window::settings::PlatformSpecific {
                        #[cfg(target_os = "linux")]
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

/// Digitou há menos que isso: um arquivo pedido por uma extensão não abre por cima (ver `OpenFileWhenIdle`).
const TYPING_PAUSE: std::time::Duration = std::time::Duration::from_secs(2);
/// Quantas abas fechadas o Ctrl+Shift+T lembra.
const CLOSED_TABS_MAX: usize = 20;

/// Um tique a cada 3 s, para as cópias de recuperação.
fn backup_ticks() -> iced::futures::channel::mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            if tx.unbounded_send(Message::BackupTick).is_err() {
                return;
            }
        }
    });
    rx
}

fn ide_state_ticks() -> iced::futures::channel::mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(ide_state::MIN_GAP);
            if tx.unbounded_send(Message::IdeStateTick).is_err() {
                return;
            }
        }
    });
    rx
}

fn media_ticks() -> iced::futures::channel::mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if tx.unbounded_send(Message::MediaTick).is_err() {
                return;
            }
        }
    });
    rx
}

#[allow(clippy::ptr_arg)]
fn mascot_ticks(ms: &u64) -> iced::futures::channel::mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced::futures::channel::mpsc::unbounded();
    let d = Duration::from_millis((*ms).max(40));
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(d);
            if tx.unbounded_send(Message::Mascot(mascot::Msg::Tick)).is_err() {
                return;
            }
        }
    });
    rx
}

/// Modo da paleta que um comando abre (para o atalho fechar ou trocar de modo com ela aberta).
fn palette_mode_of(command: &str) -> Option<palette::Mode> {
    Some(match command {
        "palette.open" => palette::Mode::Commands,
        "file.quick_open" => palette::Mode::Files,
        "nav.file_symbol" => palette::Mode::FileSymbols,
        "nav.project_symbol" => palette::Mode::ProjectSymbols,
        "file.open_recent" => palette::Mode::Recent,
        _ => return None,
    })
}

/// A estrutura do arquivo como lista plana para a paleta, com o caminho dos pais (`Conta › impl Conta`).
fn symbol_items(path: &Path, symbols: &[tea_core::outline::Symbol]) -> Vec<palette::SymbolItem> {
    fn walk(path: &Path, list: &[tea_core::outline::Symbol], parents: &str, out: &mut Vec<palette::SymbolItem>) {
        for s in list {
            out.push(palette::SymbolItem {
                name: s.name.clone(),
                container: parents.to_string(),
                kind: s.kind,
                visibility: s.visibility.clone(),
                path: path.to_path_buf(),
                line: s.line,
                column: s.column,
            });
            let inner = if parents.is_empty() { s.name.clone() } else { format!("{parents} › {}", s.name) };
            walk(path, &s.children, &inner, out);
        }
    }
    let mut out = Vec::new();
    walk(path, symbols, "", &mut out);
    out
}

/// A palavra (letras, números e `_`) que contém a coluna `column` (em caracteres).
fn word_at(line: &str, column: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let is_word = |c: &char| c.is_alphanumeric() || *c == '_';
    let column = column.min(chars.len());
    let start = chars[..column].iter().rposition(|c| !is_word(c)).map_or(0, |i| i + 1);
    let end = chars[column..].iter().position(|c| !is_word(c)).map_or(chars.len(), |i| column + i);
    chars[start..end].iter().collect()
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
    #[cfg(feature = "ext-debug")]
    list.push(Box::new(tea_ext_debug::DebugExtension::new()));
    #[cfg(feature = "ext-references")]
    list.push(Box::new(tea_ext_references::ReferencesExtension::new()));
    #[cfg(feature = "ext-outline")]
    list.push(Box::new(tea_ext_outline::OutlineExtension::new()));
    #[cfg(feature = "ext-calls")]
    list.push(Box::new(tea_ext_calls::CallsExtension::new()));
    #[cfg(feature = "ext-preview")]
    list.push(Box::new(tea_ext_preview::PreviewExtension::new()));
    #[cfg(feature = "ext-ui")]
    list.push(Box::new(tea_ext_ui::UiExtension::new()));
    list
}

/// Plugins WASM instalados (protótipo da Fase 4, feature `plugins`). Não podem usar o id de uma extensão de
/// dentro nem o prefixo de um comando do núcleo: o comando iria para o lugar errado.
#[cfg(feature = "plugins")]
fn installed_plugins(builtin: &[Box<dyn AnyExtension>]) -> Vec<Box<dyn AnyExtension>> {
    let Some(dir) = tea_plugins::plugins_dir() else { return Vec::new() };
    let mut taken: Vec<&str> = builtin.iter().map(|e| e.id()).collect();
    taken.extend(CORE_COMMANDS.iter().filter_map(|(id, _)| id.split('.').next()));
    taken.extend(["plugin", "plugins", "core", "app", "idetea"]);
    tea_plugins::discover(&dir, &taken).into_iter().map(|p| Box::new(p) as Box<dyn AnyExtension>).collect()
}

/// Comandos do núcleo: (id, mensagem do título em `i18n/*/tea-app.ftl`).
const CORE_COMMANDS: &[(&str, &str)] = &[
    ("palette.open", "cmd-palette-open"),
    ("help.open", "cmd-help-open"),
    ("settings.open", "cmd-settings-open"),
    ("file.new", "cmd-file-new"),
    ("file.save", "cmd-file-save"),
    ("file.open", "cmd-file-open"),
    ("file.quick_open", "cmd-file-quick-open"),
    ("debug.continue", "cmd-debug-continue"),
    ("debug.stop", "cmd-debug-stop"),
    ("debug.pause", "cmd-debug-pause"),
    ("debug.next", "cmd-debug-next"),
    ("debug.step_in", "cmd-debug-step-in"),
    ("debug.step_out", "cmd-debug-step-out"),
    ("debug.toggle_breakpoint", "cmd-debug-toggle-breakpoint"),
    ("debug.choose_program", "cmd-debug-choose-program"),
    ("file.open_folder", "cmd-file-open-folder"),
    ("file.open_recent", "cmd-file-open-recent"),
    ("file.save_as", "cmd-file-save-as"),
    ("file.save_as_path", "cmd-file-save-as-path"),
    ("file.save_all", "cmd-file-save-all"),
    ("file.close", "cmd-file-close"),
    ("tab.next", "cmd-tab-next"),
    ("tab.prev", "cmd-tab-prev"),
    ("tab.reopen", "cmd-tab-reopen"),
    ("editor.split", "cmd-editor-split"),
    ("editor.focus_left", "cmd-editor-focus-left"),
    ("editor.focus_right", "cmd-editor-focus-right"),
    ("editor.find", "cmd-editor-find"),
    ("editor.replace", "cmd-editor-replace"),
    ("editor.goto_line", "cmd-editor-goto-line"),
    ("editor.undo", "cmd-editor-undo"),
    ("editor.redo", "cmd-editor-redo"),
    ("editor.select_all", "cmd-editor-select-all"),
    ("editor.fold_all", "cmd-editor-fold-all"),
    ("editor.unfold_all", "cmd-editor-unfold-all"),
    ("editor.toggle_fold", "cmd-editor-toggle-fold"),
    ("editor.toggle_vim", "cmd-editor-toggle-vim"),
    ("editor.zoom_in", "cmd-editor-zoom-in"),
    ("editor.zoom_out", "cmd-editor-zoom-out"),
    ("editor.zoom_reset", "cmd-editor-zoom-reset"),
    ("lsp.definition", "cmd-lsp-definition"),
    ("lsp.references", "cmd-lsp-references"),
    ("lsp.implementation", "cmd-lsp-implementation"),
    ("lsp.type_definition", "cmd-lsp-type-definition"),
    ("lsp.incoming_calls", "cmd-lsp-incoming-calls"),
    ("lsp.outgoing_calls", "cmd-lsp-outgoing-calls"),
    ("nav.file_symbol", "cmd-nav-file-symbol"),
    ("nav.project_symbol", "cmd-nav-project-symbol"),
    ("nav.back", "cmd-nav-back"),
    ("nav.forward", "cmd-nav-forward"),
    ("lsp.hover", "cmd-lsp-hover"),
    ("lsp.format", "cmd-lsp-format"),
    ("lsp.rename", "cmd-lsp-rename"),
    ("lsp.restart", "cmd-lsp-restart"),
    ("theme.toggle", "cmd-theme-toggle"),
    ("config.open", "cmd-config-open"),
    ("config.open_project", "cmd-config-open-project"),
    ("config.reload", "cmd-config-reload"),
    ("view.toggle_left", "cmd-view-toggle-left"),
    ("view.toggle_right", "cmd-view-toggle-right"),
    ("view.toggle_bottom", "cmd-view-toggle-bottom"),
    ("tasks.run", "cmd-tasks-run"),
    ("tasks.rerun", "cmd-tasks-rerun"),
    ("tasks.test_here", "cmd-tasks-test-here"),
    ("mascot.toggle", "cmd-mascot-toggle"),
    ("app.restart", "cmd-app-restart"),
    ("app.quit", "cmd-app-quit"),
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
    /// Depurar este executável (havia mais de um).
    DebugProgram(PathBuf),
    /// Aplicar as edições do "renomear símbolo" que estão esperando.
    ApplyRename,
    /// Cópias de alterações deixadas por uma janela que travou: abrir de volta ou apagar.
    Recover,
    DiscardRecovery,
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
    /// Editor dividido: mandar a aba para o outro lado.
    OtherSide(u64),
}

struct Tab {
    id: u64,
    editor: Editor,
    /// Editor dividido: a aba fica no lado direito.
    right: bool,
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
    /// Entre os dois lados do editor dividido.
    Split,
}

struct EditorZoom {
    base: f32,
    size: f32,
}

impl EditorZoom {
    fn new(base: f32) -> Self {
        Self { base, size: base }
    }

    fn size(&self) -> f32 {
        self.size
    }

    fn adjust(&mut self, delta: f32) {
        self.size = (self.size + delta).clamp(settings::FONT_MIN, settings::FONT_MAX);
    }

    fn reset(&mut self) {
        self.size = self.base;
    }

    fn reload_base(&mut self, base: f32) {
        if base != self.base {
            self.base = base;
            self.reset();
        }
    }
}

fn wheel_zoom(y: f32) -> Option<f32> {
    (y != 0.0).then(|| y.signum())
}

struct App {
    root: PathBuf,
    config: Config,
    editor_zoom: EditorZoom,
    modifiers: keyboard::Modifiers,
    global_config: Option<PathBuf>,
    keymap: Keymap,
    theme: Theme,

    tabs: Vec<Tab>,
    /// A aba com foco (no editor dividido, a do lado com foco).
    active: usize,
    /// Editor dividido: a aba mostrada no lado sem foco (`None`: um lado só).
    other_tab: Option<u64>,
    next_tab_id: u64,
    /// Abas fechadas (arquivo e cursor), a última no fim: Ctrl+Shift+T reabre.
    closed_tabs: Vec<(PathBuf, (usize, usize))>,

    extensions: Vec<Box<dyn AnyExtension>>,
    panels: Vec<Panel>,
    lsp: lsp::LspManager,
    commands: Vec<CommandInfo>,
    /// Painel em destaque no slot inferior (índice em `panels`), que mostra um painel por vez em abas.
    active_bottom: Option<usize>,

    palette: Option<Palette>,
    debug: debug::Debugger,
    /// De onde se saltou (definição, referência, Ctrl+P...): `Alt+←` volta, `Alt+→` avança.
    nav: nav::History,
    /// Estrutura: aba e revisão do texto do último pedido, e o número da espera em curso
    /// (só a última espera depois de digitar vale).
    outline_seen: (u64, u64),
    outline_wait: u64,
    /// Última estrutura enviada (arquivo, símbolos, motivo da lista vazia): `Ctrl+Shift+O` usa.
    last_outline: (Option<PathBuf>, std::sync::Arc<Vec<tea_core::outline::Symbol>>, Option<String>),
    /// Busca de símbolos no projeto: só a última espera depois de digitar vale.
    symbol_wait: u64,
    /// Árvore de chamadas (quem chama / quem é chamado).
    calls: calls::CallTree,
    /// Extensões de arquivo (minúsculas) cujo texto alguma extensão quer ver (`CoreEvent::ActiveText`).
    text_watch: std::collections::HashSet<String>,
    /// Último texto enviado (não manda de novo se nada mudou) e a espera em curso depois de digitar.
    text_sent: Option<(PathBuf, std::sync::Arc<str>)>,
    text_wait: u64,
    /// Ctrl+S com "formatar ao salvar": a aba espera a resposta do servidor para gravar (e a espera em curso).
    format_save: Option<u64>,
    format_save_wait: u64,
    /// Versão de referência de cada arquivo para as marcas do git (vem do git; vale ao abrir de novo).
    diff_bases: std::collections::HashMap<PathBuf, std::sync::Arc<str>>,
    diff_wait: u64,
    /// Espera para religar servidores que caíram (cada edição adia; ver `relink_later`).
    relink_wait: u64,
    /// Última tarefa rodada (`Ctrl+Shift+R` roda de novo).
    last_task: Option<tasks::TaskDef>,
    /// Botões da barra de abas por tipo de arquivo (extensão dona, ação).
    file_actions: Vec<(usize, tea_core::FileAction)>,
    /// Quando a própria IDE salvou cada arquivo: o aviso de "mudou no disco" desse save não conta.
    own_saves: std::collections::HashMap<PathBuf, Instant>,
    /// Última tecla que mudou o texto num editor (para não abrir arquivos por cima de quem está digitando).
    last_typed: Option<Instant>,
    /// O executável na abertura (`None`: não deu para ler).
    binary: Option<binary::Stamp>,
    /// O executável foi recompilado depois que a IDE abriu.
    binary_changed: bool,
    /// Sair para reiniciar (o "Reiniciar a IDE" passa pelo mesmo diálogo de arquivos não salvos que o "Sair").
    restart: bool,
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
    /// Onde gravar a sessão do projeto (`None`: desligada ou `idetea arquivo...`) e a última gravada.
    session_path: Option<PathBuf>,
    session_saved: Option<session::Session>,
    /// Lista de recentes (`None`: sem pasta de estado).
    recent_path: Option<PathBuf>,
    /// Cópias das abas alteradas: pasta do projeto e, por aba, a chave e o hash do texto copiado.
    backup_dir: Option<PathBuf>,
    backed_up: std::collections::HashMap<u64, (String, u64)>,
    /// Cópias de uma janela que travou, esperando a resposta do diálogo.
    pending_recovery: Vec<backup::Orphan>,
    /// Renomear símbolo: o que foi pedido (aba, linha, coluna, nome atual) e, depois da resposta, as
    /// edições esperando a confirmação na prévia.
    rename_target: Option<(u64, usize, usize, String)>,
    rename_edits: Option<(String, Vec<tea_lsp::FileEdits>)>,
    /// O que foi gravado em `.idetea/state.json` (para `idetea status`), quando, e se há mudança esperando o
    /// intervalo mínimo.
    ide_state: Option<ide_state::State>,
    ide_state_at: Option<Instant>,
    ide_state_due: bool,
    mascot: mascot::Mascot,
    mascot_diag: std::collections::HashMap<(PathBuf, String), usize>,
}

#[derive(Debug, Clone)]
enum Message {
    Editor(u64, EditorMessage),
    /// Vinda da interface de uma extensão.
    Ext(usize, ExtMessage),
    /// Vinda da subscription de uma extensão.
    ExtSub(usize, ExtMessage),
    SelectTab(u64),
    /// Breadcrumbs: pasta clicada (mostra o arquivo da aba na árvore).
    BreadcrumbReveal(u64),
    /// Breadcrumbs: símbolo clicado (arquivo, linha, coluna).
    BreadcrumbSymbol(PathBuf, usize, usize),
    /// Botão de uma extensão na barra de abas: roda o comando.
    RunCommand(String),
    /// Botão direito numa aba.
    TabMenu(u64, iced::Point),
    CloseTab(u64),
    SelectBottom(usize),
    /// `modified`: o caractere que a tecla produziu (com Shift/AltGr), para atalhos como `ctrl+\\`.
    Key { key: keyboard::Key, modified: keyboard::Key, modifiers: keyboard::Modifiers, captured: bool },
    Palette(palette::Msg),
    /// Arquivos mudaram no disco (observador do projeto).
    Disk(watcher::Change),
    /// Evento do debugger (número da sessão, evento).
    Dap(u64, tea_dap::Event),
    /// A lista de arquivos do projeto terminou de ser lida (para `Ctrl+P`).
    FilesListed(std::sync::Arc<Vec<String>>),
    /// A compilação antes do debug terminou.
    DebugBuilt(std::sync::Arc<debug_build::Outcome>),
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
    ModifiersChanged(keyboard::Modifiers),
    WheelScrolled(f32),
    /// Hora de copiar as abas com alterações (recuperação depois de um travamento).
    BackupTick,
    /// O executável da IDE mudou no disco.
    BinaryChanged,
    /// Hora de conferir se o arquivo de uma aba de imagem ou PDF mudou no disco.
    MediaTick,
    /// Hora de gravar o `state.json` que esperava o intervalo mínimo.
    IdeStateTick,
    /// `idetea open`: mostrar o arquivo com o cursor no lugar.
    OpenRequest(ide_state::OpenRequest),
    /// Passou a espera depois de digitar: pede a estrutura de novo (se for a última espera).
    OutlineDue(u64),
    /// Hora de mandar o texto do arquivo ativo (só a última espera vale).
    TextDue(u64),
    /// O servidor demorou para formatar: salva assim mesmo.
    FormatSaveDue(u64),
    /// Hora de comparar o arquivo ativo com o último commit (só a última espera vale).
    DiffDue(u64),
    /// A digitação pausou: religa os servidores que caíram (só a última espera vale).
    RelinkDue(u64),
    /// Passou a espera depois de digitar na busca de símbolos do projeto.
    SymbolQueryDue(u64),
    /// Botões voltar/avançar do mouse (`true` = avançar).
    Nav(bool),
    CloseRequested,
    Lsp(tea_lsp::Event),
    Mascot(mascot::Msg),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let started = *STARTED.get_or_init(Instant::now);
        let launch = LAUNCH.get().expect("argumentos interpretados em main");
        let root = launch.root.clone();
        let global_config = config::global_config_path();
        let project_config = config::project_config_path(&root);
        // Antes de qualquer texto (os avisos do config, os títulos que as extensões registram).
        tea_core::i18n::init(&config::peek_language(global_config.as_deref(), Some(&project_config)));
        let config = Config::load(global_config.as_deref(), Some(&project_config));
        let (keymap, key_errors) = Keymap::from_pairs(config.keys.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let (theme, theme_error) = resolve_theme(&config.ui.theme);

        let show_splash = config.ui.splash && !launch.no_splash;
        let state_path = ui_state::state_path();
        // `idetea arquivo...` é uso rápido: abre só o pedido e não mexe na sessão do projeto.
        let session_path =
            (config.ui.restore_session && launch.files.is_empty()).then(|| session::path_for(&root)).flatten();
        let restored = session_path.as_deref().and_then(|p| session::Session::load(p, &root));
        #[cfg_attr(not(feature = "plugins"), allow(unused_mut))]
        let mut builtin = builtin_extensions();
        #[cfg(feature = "plugins")]
        builtin.extend(installed_plugins(&builtin));
        let all_extensions: Vec<&'static str> = builtin.iter().map(|e| e.id()).collect();
        let extensions = builtin.into_iter().filter(|e| config.extension(e.id()).enabled).collect();

        // Eventos dos servidores LSP chegam por este canal (de outras threads).
        let (lsp_tx, lsp_rx) = iced::futures::channel::mpsc::unbounded();
        let sink: tea_lsp::EventSink = std::sync::Arc::new(move |e| {
            let _ = lsp_tx.unbounded_send(e);
        });
        let lsp = lsp::LspManager::new(&root, &config.lsp, sink);
        let (dap_tx, dap_rx) = iced::futures::channel::mpsc::unbounded();

        let ui_state = ui_state::UiState::load(state_path.as_deref());
        let mascot = mascot::Mascot::new(ui_state.mascot_collapsed);

        let mut app = Self {
            lsp,
            debug: debug::Debugger::new(dap_tx),
            root,
            global_config,
            keymap,
            editor_zoom: EditorZoom::new(config.ui.font_size),
            modifiers: keyboard::Modifiers::empty(),
            theme,
            tabs: Vec::new(),
            active: 0,
            other_tab: None,
            next_tab_id: 0,
            closed_tabs: Vec::new(),
            extensions,
            panels: Vec::new(),
            active_bottom: None,
            commands: CORE_COMMANDS
                .iter()
                .map(|(id, title)| CommandInfo { id: id.to_string(), title: LOADER.get(title) })
                .collect(),
            palette: None,
            file_list: Default::default(),
            own_saves: Default::default(),
            last_typed: None,
            binary: binary::Stamp::current(),
            binary_changed: false,
            restart: false,
            splash: show_splash.then(|| Splash::new(started)),
            pointer_over_panel: false,
            dialog: None,
            menu: None,
            picking: false,
            help: None,
            settings: None,
            all_extensions,
            ui_state,
            state_path,
            window_size: None,
            session_path,
            session_saved: None,
            recent_path: recent::path(),
            backup_dir: backup::dir_for(&launch.root),
            backed_up: Default::default(),
            pending_recovery: Vec::new(),
            rename_target: None,
            rename_edits: None,
            ide_state: None,
            ide_state_at: None,
            ide_state_due: false,
            mascot,
            mascot_diag: Default::default(),
            last_status: EditorStatus::default(),
            nav: nav::History::default(),
            outline_seen: (u64::MAX, 0),
            outline_wait: 0,
            last_outline: (None, Default::default(), None),
            symbol_wait: 0,
            calls: Default::default(),
            text_watch: Default::default(),
            text_sent: None,
            text_wait: 0,
            format_save: None,
            format_save_wait: 0,
            diff_bases: std::collections::HashMap::new(),
            diff_wait: 0,
            relink_wait: 0,
            last_task: None,
            file_actions: Vec::new(),
            config,
        };
        app.push_tab(Editor::scratch(""));
        if let Some(path) = &app.recent_path {
            recent::Recent::add_project(path, &app.root);
        }

        let mut tasks = vec![Task::run(lsp_rx, Message::Lsp), Task::run(dap_rx, |(g, e)| Message::Dap(g, e))];
        // Extensões primeiro: assim elas recebem os eventos dos arquivos abertos abaixo.
        for i in 0..app.extensions.len() {
            tasks.push(app.call_ext(i, |e, ctx| e.activate(ctx)));
        }
        tasks.push(app.broadcast_appearance());
        for file in launch.files.clone() {
            tasks.push(app.open_file(file));
        }
        if let Some(s) = restored {
            tasks.push(app.restore_session(s));
        }
        app.session_saved = Some(app.session_snapshot());
        if let Some(dir) = &app.backup_dir {
            app.pending_recovery = backup::orphans(dir);
        }
        if !app.pending_recovery.is_empty() {
            tasks.push(app.offer_recovery());
        }
        tasks.push(app.emit_status(true));

        let mut problems: Vec<String> = launch.warnings.iter().map(ToString::to_string).collect();
        problems.extend(app.config.errors.iter().cloned());
        problems.extend(key_errors);
        problems.extend(theme_error);
        problems.extend(tea_core::fonts::problem(&app.config.editor));
        problems.extend(tea_core::file_icons::problem(&app.config.ui.file_icons));
        problems.extend(
            tea_editor::syntaxes::problems()
                .iter()
                .map(|(file, error)| fl!("config-syntax-error", file = file.as_str(), error = error.as_str())),
        );
        tea_core::file_icons::set_enabled(app.config.ui.file_icons.trim() != "none");
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
        let title = format!("{} — {project} — IDE TEA", self.tabs[self.active].editor.title());
        if self.binary_changed { format!("{title} {}", fl!("binary-changed-title")) } else { title }
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
                Request::OpenFileAt { path, line, column } => {
                    self.remember_spot(&path, Some(line));
                    tasks.push(self.open_file_at(path, line, column));
                }
                Request::OpenFileWhenIdle { path, line, column, busy_note } => {
                    if self.last_typed.is_some_and(|t| t.elapsed() < TYPING_PAUSE) {
                        tasks.push(self.notify(busy_note));
                    } else if line == 0 {
                        tasks.push(self.open_file(path));
                    } else {
                        self.remember_spot(&path, Some(line));
                        tasks.push(self.open_file_at(path, line, column.max(1)));
                    }
                }
                Request::SaveFiles { paths, then } => {
                    let ids: Vec<u64> = self
                        .tabs
                        .iter()
                        .filter(|t| t.editor.is_modified() && t.editor.path().is_some_and(|p| paths.iter().any(|q| q == p)))
                        .map(|t| t.id)
                        .collect();
                    let (failed, saved) = self.save_tabs(&ids);
                    tasks.push(saved);
                    // Não salvou (o aviso já saiu): a extensão não segue com arquivos velhos no disco.
                    if failed.is_empty() {
                        tasks.push(Task::done(Message::Ext(ext, then)));
                    }
                }
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
                Request::Pick(pick) => tasks.push(self.open_palette(Palette::pick(Some(ext), pick))),
                Request::DebugSelectFrame(i) => tasks.push(self.debug_select_frame(i)),
                Request::DebugVariables(reference) => {
                    if let Some(session) = self.debug.session() {
                        session.variables(reference);
                    }
                }
                Request::FileAction(mut action) => {
                    action.extensions.iter_mut().for_each(|e| *e = e.to_lowercase());
                    self.file_actions.push((ext, action));
                }
                Request::WatchText(extensions) => {
                    self.text_watch.extend(extensions.into_iter().map(|e| e.to_lowercase()));
                    self.text_sent = None;
                    tasks.push(self.send_active_text());
                }
                Request::CallChildren { node, direction } => {
                    self.calls.request(node, direction);
                }
                Request::Replace(replace) => tasks.push(self.replace_in_files(&replace)),
                Request::DiffBase { path, text } => {
                    for t in self.tabs.iter_mut().filter(|t| t.editor.path() == Some(path.as_path())) {
                        t.editor.set_diff_base(text.clone());
                    }
                    match text {
                        Some(text) => self.diff_bases.insert(path, text),
                        None => self.diff_bases.remove(&path),
                    };
                }
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
        match &event {
            CoreEvent::Activity { source, text } => self.mascot.set_lsp_activity(source, text.as_deref()),
            CoreEvent::RunTask { name, .. } => self.mascot.notify_task_started(name),
            CoreEvent::Diagnostics { source, path, items } => {
                let errs = items.iter().filter(|d| d.severity == Severity::Error).count();
                if errs == 0 {
                    self.mascot_diag.remove(&(path.clone(), source.clone()));
                } else {
                    self.mascot_diag.insert((path.clone(), source.clone()), errs);
                }
                self.mascot.set_lsp_errors(self.mascot_diag.values().sum());
            }
            CoreEvent::EditorStatus(_) => self.mascot.touch_user_activity(),
            _ => {}
        }
        let tasks: Vec<_> =
            (0..self.extensions.len()).map(|i| self.call_ext(i, |e, ctx| e.on_event(&event, ctx))).collect();
        Task::batch(tasks)
    }

    /// Fonte do código, do `[editor]` do config.
    fn code_font(&self) -> tea_core::fonts::CodeFont {
        tea_core::fonts::CodeFont::from_config(&self.config.editor)
    }

    fn broadcast_appearance(&mut self) -> Task<Message> {
        self.broadcast(CoreEvent::Appearance {
            theme: self.theme.clone(),
            font_size: self.config.ui.font_size,
            code_font: self.code_font(),
        })
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
            language: ed.language(),
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

    /// Tema, fonte e opções do editor (config `[editor]` + `.editorconfig`).
    fn style_editor(&self, editor: &mut Editor) {
        if let Some(base) = editor.path().and_then(|p| self.diff_bases.get(p)) {
            editor.set_diff_base(Some(base.clone()));
        }
        editor.set_theme(&self.theme);
        editor.set_code_font(self.editor_zoom.size(), self.code_font());
        editor.set_options(&self.editor_options(editor));
    }

    fn editor_options(&self, editor: &Editor) -> tea_editor::Options {
        let c = &self.config.editor;
        let mut o = tea_editor::Options {
            wrap: c.wrap,
            show_whitespace: c.show_whitespace,
            indent_guides: c.indent_guides,
            tab_size: c.tab_size,
            insert_spaces: c.insert_spaces,
            trim_trailing_whitespace: c.trim_trailing_whitespace,
            insert_final_newline: c.insert_final_newline,
        };
        let Some(path) = editor.path() else { return o };
        // Em Markdown, dois espaços no fim quebram a linha: só tira se o `.editorconfig` mandar.
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
        if matches!(ext.as_deref(), Some("md" | "markdown")) {
            o.trim_trailing_whitespace = false;
        }
        let ec = editorconfig::Props::for_file(path);
        match ec.insert_spaces {
            Some(spaces) => o.insert_spaces = spaces,
            // Sem `.editorconfig`: segue o que o arquivo já usa (Makefile, Go...).
            None => {
                if let Some(tabs) = editor.indents_with_tabs() {
                    o.insert_spaces = !tabs;
                }
            }
        }
        o.tab_size = ec.tab_size.unwrap_or(o.tab_size);
        o.trim_trailing_whitespace = ec.trim_trailing_whitespace.unwrap_or(o.trim_trailing_whitespace);
        o.insert_final_newline = ec.insert_final_newline.unwrap_or(o.insert_final_newline);
        o
    }

    /// Recalcula as opções de todas as abas (config recarregado, `.editorconfig` salvo, arquivo renomeado).
    fn refresh_editor_options(&mut self) {
        for i in 0..self.tabs.len() {
            let options = self.editor_options(&self.tabs[i].editor);
            self.tabs[i].editor.set_options(&options);
        }
    }

    fn push_tab(&mut self, mut editor: Editor) -> usize {
        self.style_editor(&mut editor);
        // Abre no lado com foco.
        let right = self.tabs.get(self.active).is_some_and(|t| t.right);
        self.tabs.push(Tab { id: self.next_tab_id, editor, right });
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
                    self.style_editor(&mut editor);
                    self.tabs[0].editor = editor;
                    0
                } else {
                    self.push_tab(editor)
                };
                let note = if self.tabs[i].editor.is_new_file() {
                    self.notify(fl!("file-new-created-on-save", path = display_rel(&path, &self.root)))
                } else if self.tabs[i].editor.is_light() {
                    self.notify(fl!("file-light-mode", name = self.tabs[i].editor.file_name()))
                } else {
                    Task::none()
                };
                let lsp = self.ensure_lsp(i);
                if let Some(list) = &self.recent_path
                    && path.is_file()
                {
                    recent::Recent::add_file(list, &path);
                }
                let opened = self.broadcast(CoreEvent::FileOpened(path));
                Task::batch([opened, note, lsp, self.set_active(i)])
            }
            Err(err) => self.notify(fl!("file-open-error", path = path.display().to_string(), error = err.to_string())),
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
            return self.notify(fl!("file-changed-on-disk-kept", name = name));
        }
        match Editor::open(path) {
            Ok(mut editor) => {
                self.style_editor(&mut editor);
                // A aba antiga sai (e avisa o LSP); a nova entra no lugar, com o mesmo id.
                self.tabs[i].editor = editor;
                let lsp = self.ensure_lsp(i);
                let refresh = if i == self.active { self.set_active(i) } else { Task::none() };
                Task::batch([lsp, refresh])
            }
            Err(e) => self.notify(fl!("file-reload-error", path = display_rel(path, &self.root), error = e.to_string())),
        }
    }

    // ---- Debug -------------------------------------------------------------

    /// Breakpoints de todas as abas: arquivo → linhas (1-based).
    fn all_breakpoints(&self) -> std::collections::HashMap<PathBuf, Vec<usize>> {
        self.tabs
            .iter()
            .filter_map(|t| {
                let lines: Vec<usize> = t.editor.breakpoints().map(|l| l + 1).collect();
                t.editor.path().map(|p| (p.to_path_buf(), lines))
            })
            .filter(|(_, lines)| !lines.is_empty())
            .collect()
    }

    /// Breakpoints da aba mudaram: com o debug rodando, valem na hora.
    fn sync_breakpoints(&self, tab: usize) {
        let (Some(session), Some(path)) = (self.debug.session(), self.tabs[tab].editor.path()) else { return };
        session.set_breakpoints(path, self.tabs[tab].editor.breakpoints().map(|l| l + 1).collect());
    }

    /// F5: inicia (compilando antes, se for o caso) ou continua.
    fn debug_continue(&mut self) -> Task<Message> {
        match self.debug.session() {
            None if self.debug.building => self.notify(fl!("debug-still-building")),
            None => self.debug_build(),
            Some(_) if self.debug.stopped.is_none() => self.notify(fl!("debug-already-running")),
            Some(session) => {
                session.continue_();
                Task::none()
            }
        }
    }

    /// Compila numa thread (cargo, ou o `build` do config); sem nada para compilar, já inicia.
    fn debug_build(&mut self) -> Task<Message> {
        let cleared = self.debug_event(DebugEvent::OutputCleared);
        let Some(build) = debug_build::plan(&self.config.debug, &self.root) else {
            return Task::batch([cleared, self.debug_choose(&[])]);
        };
        self.debug.building = true;
        let status = self.debug_event(DebugEvent::Status(DebugStatus::Building));
        let note = self.notify(fl!("debug-building", command = build.label()));
        let root = self.root.clone();
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(debug_build::run(&build, &root));
        });
        let done = Task::future(async move { rx.await.ok() })
            .and_then(|outcome| Task::done(Message::DebugBuilt(std::sync::Arc::new(outcome))));
        Task::batch([cleared, status, note, done])
    }

    fn debug_built(&mut self, outcome: &debug_build::Outcome) -> Task<Message> {
        // Parou (Shift+F5) durante a compilação: o resultado não vale mais.
        if !self.debug.building {
            return Task::none();
        }
        self.debug.building = false;
        let log = self.debug_event(DebugEvent::Output { category: "build".into(), text: outcome.log.clone() });
        if !outcome.ok {
            let error = outcome.error.clone().unwrap_or_else(|| fl!("unknown-error"));
            let idle = self.debug_event(DebugEvent::Status(DebugStatus::Idle));
            return Task::batch([log, idle, self.notify(fl!("debug-build-failed", error = error))]);
        }
        let next = self.debug_choose(&outcome.executables);
        // Escolheu nada (diálogo ou janela de arquivos abertos): o painel volta a "sem sessão".
        let idle = if self.debug.is_running() {
            Task::none()
        } else {
            self.debug_event(DebugEvent::Status(DebugStatus::Idle))
        };
        Task::batch([log, next, idle])
    }

    /// Qual programa rodar: o do config, o único que o cargo gerou, o que já foi escolhido
    /// antes, uma pergunta (vários executáveis) ou a janela de arquivos.
    fn debug_choose(&mut self, executables: &[PathBuf]) -> Task<Message> {
        if let Some(program) = self.config.debug.program.as_deref().filter(|p| !p.trim().is_empty()) {
            self.debug.program = Some(self.root.join(program));
            return self.debug_start();
        }
        match executables {
            [only] => {
                self.debug.program = Some(only.clone());
                self.debug_start()
            }
            [] if self.debug.program.is_some() => self.debug_start(),
            [] => {
                let dir = self.root.clone();
                self.pick(picker::debug_program(dir))
            }
            many if self.debug.program.as_ref().is_some_and(|p| many.contains(p)) => self.debug_start(),
            many => {
                let mut dialog =
                    Dialog::new(fl!("debug-choose-title"), fl!("debug-choose-message"));
                for (i, exe) in many.iter().enumerate() {
                    let name = exe.file_name().map_or_else(|| exe.display().to_string(), |n| n.to_string_lossy().into_owned());
                    let kind = if i == 0 { ButtonKind::Primary } else { ButtonKind::Secondary };
                    dialog = dialog.button(name, kind, CoreDialog::DebugProgram(exe.clone()));
                }
                self.open_dialog(dialog.cancel(fl!("cancel")), dialog::Owner::Core)
            }
        }
    }

    fn debug_start(&mut self) -> Task<Message> {
        let Some(program) = self.debug.program.clone() else { return Task::none() };
        let breakpoints = self.all_breakpoints();
        let cfg = &self.config.debug;
        let cwd = cfg.cwd.as_deref().map_or_else(|| self.root.clone(), |c| self.root.join(c));
        let (status, note) = match self.debug.start(program.clone(), cfg.args.clone(), cwd, breakpoints) {
            Ok(()) => (DebugStatus::Running, fl!("debug-running", program = display_rel(&program, &self.root))),
            Err(e) => (DebugStatus::Idle, fl!("debug-error", error = e.to_string())),
        };
        Task::batch([self.debug_event(DebugEvent::Status(status)), self.notify(note)])
    }

    fn debug_stop(&mut self) -> Task<Message> {
        if self.debug.building {
            self.debug.stop();
            let idle = self.debug_event(DebugEvent::Status(DebugStatus::Idle));
            return Task::batch([idle, self.notify(fl!("debug-cancelled"))]);
        }
        if !self.debug.is_running() {
            return Task::none();
        }
        self.debug.stop();
        self.clear_exec_lines();
        let idle = self.debug_event(DebugEvent::Status(DebugStatus::Idle));
        Task::batch([idle, self.notify(fl!("debug-stopped"))])
    }

    fn debug_event(&mut self, event: DebugEvent) -> Task<Message> {
        self.broadcast(CoreEvent::Debug(event))
    }

    /// O painel escolheu um quadro da pilha: mostra onde ele está e as variáveis dele.
    fn debug_select_frame(&mut self, index: usize) -> Task<Message> {
        let Some(frame) = self.debug.stopped.as_ref().and_then(|f| f.get(index)).cloned() else { return Task::none() };
        if let Some(session) = self.debug.session() {
            session.scopes(frame.id);
        }
        let frames = self.debug.stopped.as_ref().map(|f| f.iter().map(debug::Debugger::to_panel).collect());
        let stack = self.debug_event(DebugEvent::Stack { frames: frames.unwrap_or_default(), selected: index });
        let open = match frame.path.filter(|p| p.is_file() && frame.line > 0) {
            Some(path) => self.open_file_at(path, frame.line, 1),
            None => self.notify(fl!("debug-no-source", name = frame.name.clone())),
        };
        Task::batch([stack, open])
    }

    fn clear_exec_lines(&mut self) {
        for t in &mut self.tabs {
            t.editor.set_exec_line(None);
        }
    }

    fn on_dap(&mut self, generation: u64, event: tea_dap::Event) -> Task<Message> {
        use tea_dap::Event as E;
        if !self.debug.is_current(generation) {
            return Task::none();
        }
        match event {
            E::Stopped { reason, frames, .. } => {
                self.clear_exec_lines();
                let reason = debug::reason_label(&reason);
                let top = frames.first().map(|f| f.name.clone()).unwrap_or_else(|| "?".into());
                // O primeiro quadro com código-fonte que existe (passos dentro da std não têm).
                let shown = frames.iter().position(|f| f.path.as_deref().is_some_and(Path::is_file) && f.line > 0);
                let status = self.debug_event(DebugEvent::Status(DebugStatus::Stopped { reason: reason.clone() }));
                let stack = self.debug_event(DebugEvent::Stack {
                    frames: frames.iter().map(debug::Debugger::to_panel).collect(),
                    selected: shown.unwrap_or(0),
                });
                if let (Some(session), Some(f)) = (self.debug.session(), frames.get(shown.unwrap_or(0))) {
                    session.scopes(f.id);
                }
                let shown = shown.map(|i| frames[i].clone());
                self.debug.stopped = Some(frames);
                let Some(frame) = shown else {
                    let note = self.notify(fl!("debug-paused-no-source", name = top, reason = reason));
                    return Task::batch([status, stack, note]);
                };
                let path = frame.path.clone().unwrap_or_default();
                let open = self.open_file_at(path.clone(), frame.line, 1);
                if let Some(t) = self.tabs.iter_mut().find(|t| t.editor.path() == Some(path.as_path())) {
                    t.editor.set_exec_line(Some(frame.line - 1));
                }
                let note =
                    self.notify(fl!("debug-paused-at", path = display_rel(&path, &self.root), line = frame.line, reason = reason));
                Task::batch([status, stack, open, note])
            }
            E::Continued => {
                self.debug.stopped = None;
                self.clear_exec_lines();
                self.debug_event(DebugEvent::Status(DebugStatus::Running))
            }
            E::Output { category, text } => self.debug_event(DebugEvent::Output { category, text }),
            E::Scopes { frame_id, scopes } => {
                let scopes = scopes
                    .into_iter()
                    .map(|s| tea_core::debug::Scope { name: s.name, reference: s.reference, expensive: s.expensive })
                    .collect();
                self.debug_event(DebugEvent::Scopes { frame_id, scopes })
            }
            E::Variables { reference, variables } => {
                let variables = variables
                    .into_iter()
                    .map(|v| tea_core::debug::Variable {
                        name: v.name,
                        value: v.value,
                        type_name: v.type_name,
                        reference: v.reference,
                    })
                    .collect();
                self.debug_event(DebugEvent::Variables { reference, variables })
            }
            E::Exited { code } => {
                self.notify(match code {
                    Some(c) => fl!("debug-exited-code", code = c),
                    None => fl!("debug-exited"),
                })
            }
            E::Terminated => {
                self.debug.ended();
                self.clear_exec_lines();
                self.debug_event(DebugEvent::Status(DebugStatus::Idle))
            }
            E::Error { command, message } => self.notify(fl!("debug-command-failed", command = command, message = message)),
            E::AdapterExited { reason } => {
                self.debug.ended();
                self.clear_exec_lines();
                let idle = self.debug_event(DebugEvent::Status(DebugStatus::Idle));
                Task::batch([idle, self.notify(fl!("debug-gdb-exited", reason = reason))])
            }
        }
    }

    /// O observador viu mudanças no disco: abas sem alterações recarregam, abas com alterações
    /// avisam (uma vez por mudança), e as extensões ficam sabendo (árvore e git atualizam).
    fn disk_changed(&mut self, paths: &[PathBuf]) -> Task<Message> {
        let now = Instant::now();
        self.own_saves.retain(|_, t| now.duration_since(*t) < Duration::from_secs(2));
        let mut tasks = vec![self.reload_changed_tabs()];
        // Abas de imagem e PDF: o `reload_changed_tabs` compara o texto, que elas não têm; recarrega pelo caminho.
        let images: Vec<PathBuf> = self
            .tabs
            .iter()
            .filter(|t| t.editor.is_image())
            .filter_map(|t| t.editor.path())
            .filter(|p| paths.iter().any(|c| c == p))
            .map(Path::to_path_buf)
            .collect();
        tasks.extend(images.iter().map(|p| self.reload_from_disk(p)).collect::<Vec<_>>());
        let conflicts: Vec<String> = self
            .tabs
            .iter()
            .filter(|t| t.editor.is_modified())
            .filter_map(|t| t.editor.path())
            .filter(|p| paths.iter().any(|c| c == p) && !self.own_saves.contains_key(*p))
            .map(|p| display_rel(p, &self.root))
            .collect();
        for name in conflicts {
            tasks.push(self.notify(fl!("file-changed-on-disk-overwrite", name = name)));
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

    // ---- Sessão -------------------------------------------------------------

    /// Abas, aba ativa e painéis de agora (abas sem nome ficam de fora).
    fn session_snapshot(&self) -> session::Session {
        let tabs: Vec<session::SessionTab> = self
            .tabs
            .iter()
            .filter_map(|t| {
                let path = t.editor.path()?.to_path_buf();
                let (line, column) = t.editor.cursor();
                Some(session::SessionTab { path, line, column, right: t.right })
            })
            .collect();
        let active = self.active_path().and_then(|p| tabs.iter().position(|t| t.path == p)).unwrap_or(0);
        let other = self.other_index().and_then(|i| self.tabs[i].editor.path()).and_then(|p| tabs.iter().position(|t| t.path == p));
        session::Session {
            root: self.root.clone(),
            active,
            bottom: self.active_bottom.and_then(|i| self.panels.get(i)).map(|p| p.id.0.to_string()),
            panels: self.panels.iter().filter(|p| p.slot != Slot::Status).map(|p| (p.id.0.to_string(), p.visible)).collect(),
            tabs,
            other,
        }
    }

    /// Grava a sessão se as abas ou os painéis mudaram (`force`: sempre, com o cursor de agora).
    fn save_session(&mut self, force: bool) {
        let Some(path) = &self.session_path else { return };
        let now = self.session_snapshot();
        if !force && self.session_saved.as_ref().is_some_and(|s| s.same_shape(&now)) {
            return;
        }
        now.save(path);
        self.session_saved = Some(now);
    }

    /// Reabre o projeto como estava: painéis, abas (com o cursor) e a aba ativa.
    fn restore_session(&mut self, s: session::Session) -> Task<Message> {
        for p in self.panels.iter_mut().filter(|p| p.slot != Slot::Status) {
            if let Some(visible) = s.panels.get(p.id.0) {
                p.visible = *visible;
            }
        }
        let bottom = s.bottom.as_deref().and_then(|b| {
            self.panels.iter().position(|p| p.id.0 == b && p.slot == Slot::Bottom && p.visible)
        });
        match bottom {
            Some(i) => self.active_bottom = Some(i),
            None => self.apply_panel_slots(),
        }
        let active = s.tabs.get(s.active).map(|t| t.path.clone());
        let other = s.other.and_then(|i| s.tabs.get(i)).map(|t| t.path.clone());
        let right: std::collections::HashMap<PathBuf, bool> = s.tabs.iter().map(|t| (t.path.clone(), t.right)).collect();
        let mut tasks = Vec::new();
        for t in s.tabs.into_iter().filter(|t| t.path.is_file()) {
            tasks.push(self.open_file_at(t.path, t.line, t.column));
        }
        // Editor dividido: cada aba no seu lado, a do outro lado aparecendo.
        for t in &mut self.tabs {
            t.right = t.editor.path().and_then(|p| right.get(p)).copied().unwrap_or(false);
        }
        if let Some(path) = active.filter(|p| p.is_file()) {
            tasks.push(self.open_file(path));
        }
        self.other_tab = other.and_then(|p| self.tabs.iter().find(|t| t.editor.path() == Some(p.as_path()))).map(|t| t.id);
        self.normalize_split();
        Task::batch(tasks)
    }

    // ---- Renomear símbolo --------------------------------------------------

    /// `F2`: pede o novo nome na paleta, já com o nome atual.
    fn start_rename(&mut self) -> Task<Message> {
        if self.editor().lsp_server().is_none() {
            return self.no_lsp_note();
        }
        let Some((word, line, column)) = self.editor().word_at_cursor() else {
            return self.notify(fl!("rename-no-word"));
        };
        self.rename_target = Some((self.tabs[self.active].id, line, column, word.clone()));
        self.open_palette(Palette::rename(word))
    }

    fn request_rename(&mut self, new_name: String) -> Task<Message> {
        let Some((tab, line, column, old)) = self.rename_target.clone() else { return Task::none() };
        if new_name.is_empty() || new_name == old {
            return Task::none();
        }
        let Some(i) = self.tab_index(tab) else { return Task::none() };
        if self.tabs[i].editor.request_rename(line, column, &new_name) {
            self.notify(fl!("rename-asking", name = old))
        } else {
            self.no_lsp_note()
        }
    }

    /// Resposta do servidor: mostra quantas mudanças e onde, e espera a confirmação.
    fn preview_rename(&mut self, new_name: String, files: Vec<tea_lsp::FileEdits>) -> Task<Message> {
        let old = self.rename_target.as_ref().map(|t| t.3.clone()).unwrap_or_default();
        if files.is_empty() {
            return self.notify(fl!("rename-nothing", name = old));
        }
        let changes: usize = files.iter().map(|f| f.edits.len()).sum();
        let mut list: Vec<String> = files
            .iter()
            .take(10)
            .map(|f| format!("• {} ({})", display_rel(&f.path, &self.root), f.edits.len()))
            .collect();
        if files.len() > 10 {
            list.push(fl!("and-more", count = (files.len() - 10)));
        }
        let body = format!(
            "{}\n\n{}\n\n{}",
            fl!("rename-summary", changes = changes, files = files.len()),
            list.join("\n"),
            fl!("rename-detail")
        );
        let dialog = Dialog::new(fl!("rename-title", old = old, new = new_name.as_str()), body)
            .button(fl!("rename"), ButtonKind::Primary, CoreDialog::ApplyRename)
            .cancel(fl!("cancel"));
        self.rename_edits = Some((new_name, files));
        self.open_dialog(dialog, dialog::Owner::Core)
    }

    /// Abre os arquivos que faltam e aplica as edições como alterações não salvas (uma por arquivo no desfazer).
    fn apply_rename(&mut self) -> Task<Message> {
        let Some((_, files)) = self.rename_edits.take() else { return Task::none() };
        let active = self.tabs.get(self.active).map(|t| t.id);
        let mut tasks = Vec::new();
        let mut done = 0;
        for f in files {
            if !self.tabs.iter().any(|t| t.editor.path() == Some(f.path.as_path())) {
                tasks.push(self.open_file(f.path.clone()));
            }
            let Some(i) = self.tabs.iter().position(|t| t.editor.path() == Some(f.path.as_path())) else { continue };
            self.tabs[i].editor.apply_formatting(&f.edits);
            done += 1;
        }
        // Volta para a aba de onde pediu (abrir os arquivos trocou a ativa).
        if let Some(i) = active.and_then(|id| self.tab_index(id)) {
            tasks.push(self.set_active(i));
        }
        tasks.push(self.text_later());
        tasks.push(self.notify(fl!("renamed", count = done)));
        Task::batch(tasks)
    }

    // ---- Substituir no projeto ---------------------------------------------

    /// Troca em cada arquivo: aberto numa aba, muda o texto da aba (um passo de desfazer, fica para
    /// salvar); fechado, grava no disco.
    fn replace_in_files(&mut self, replace: &tea_core::replace::Replace) -> Task<Message> {
        let (mut count, mut files, mut failed) = (0, Vec::new(), Vec::new());
        let re = match replace.regex() {
            Ok(re) => re,
            Err(e) => return self.notify(e.to_string()),
        };
        let mut wrote = false;
        for target in &replace.targets {
            let lines = target.lines.as_deref();
            let tab = self.tabs.iter_mut().find(|t| t.editor.path() == Some(target.path.as_path()));
            let n = match tab {
                Some(t) if t.editor.is_image() => None,
                Some(t) => {
                    let (new, n) =
                        tea_core::replace::replace_text(&t.editor.text(), &re, &replace.replacement, replace.expand, lines);
                    if n > 0 {
                        t.editor.replace_text(&new);
                    }
                    Some(n)
                }
                None => std::fs::read_to_string(&target.path).ok().and_then(|text| {
                    let (new, n) =
                        tea_core::replace::replace_text(&text, &re, &replace.replacement, replace.expand, lines);
                    if n == 0 {
                        return Some(0);
                    }
                    wrote = true;
                    std::fs::write(&target.path, new).ok().map(|()| n)
                }),
            };
            match n {
                Some(0) => {}
                Some(n) => {
                    count += n;
                    files.push(target.path.clone());
                }
                None => failed.push(target.path.clone()),
            }
        }
        let mut tasks = vec![self.text_later()];
        if wrote {
            tasks.push(self.broadcast(CoreEvent::FilesChanged));
        }
        let note = if failed.is_empty() {
            fl!("replaced", count = count, files = files.len())
        } else {
            fl!("replaced-failed", count = count, files = files.len(), failed = failed.len())
        };
        tasks.push(self.notify(note));
        tasks.push(self.broadcast(CoreEvent::Replaced { count, files, failed }));
        Task::batch(tasks)
    }

    // ---- Recuperar alterações não salvas -----------------------------------

    /// Copia as abas com alterações que mudaram desde a última cópia; apaga as que não servem mais
    /// (aba salva, sem alterações ou fechada).
    fn backup_tick(&mut self) {
        let Some(dir) = self.backup_dir.clone() else { return };
        let mut alive = std::collections::HashSet::new();
        for t in &self.tabs {
            if !t.editor.is_modified() || t.editor.is_image() {
                continue;
            }
            alive.insert(t.id);
            let text = t.editor.text();
            let hash = session::fnv1a(text.as_bytes());
            let key = backup::key(t.editor.path(), t.id);
            match self.backed_up.get(&t.id) {
                Some((k, h)) if *k == key && *h == hash => {}
                old => {
                    // Salvou com outro nome: a cópia antiga sai.
                    if let Some((k, _)) = old.filter(|(k, _)| *k != key) {
                        backup::remove(&dir, k);
                    }
                    backup::write(&dir, &key, t.editor.path(), &text);
                    self.backed_up.insert(t.id, (key, hash));
                }
            }
        }
        self.backed_up.retain(|id, (key, _)| {
            let keep = alive.contains(id);
            if !keep {
                backup::remove(&dir, key);
            }
            keep
        });
    }

    fn offer_recovery(&mut self) -> Task<Message> {
        let names: Vec<String> = self
            .pending_recovery
            .iter()
            .map(|o| match &o.meta.path {
                Some(p) => format!("• {}", display_rel(p, &self.root)),
                None => format!("• {}", fl!("untitled-tab")),
            })
            .collect();
        let mut body = names.iter().take(8).cloned().collect::<Vec<_>>().join("\n");
        if names.len() > 8 {
            body.push_str(&format!("\n{}", fl!("and-more", count = (names.len() - 8))));
        }
        let dialog = Dialog::new(fl!("recover-title", count = names.len()), format!("{body}\n\n{}", fl!("recover-detail")))
            .button(fl!("recover"), ButtonKind::Primary, CoreDialog::Recover)
            .button(fl!("discard"), ButtonKind::Danger, CoreDialog::DiscardRecovery)
            .cancel(fl!("recover-later"));
        self.open_dialog(dialog, dialog::Owner::Core)
    }

    /// Abre cada arquivo (ou uma aba sem nome) com o texto da cópia, como alteração não salva.
    fn recover(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();
        let mut last = None;
        let pending = std::mem::take(&mut self.pending_recovery);
        let count = pending.len();
        for o in pending {
            let i = match &o.meta.path {
                Some(p) => {
                    tasks.push(self.open_file(p.clone()));
                    match self.tabs.iter().position(|t| t.editor.path() == Some(p.as_path())) {
                        Some(i) => i,
                        // Não abriu (o erro já apareceu): a cópia fica para a próxima.
                        None => continue,
                    }
                }
                None => self.push_tab(Editor::scratch("")),
            };
            self.tabs[i].editor.replace_text(&o.text);
            o.remove();
            last = Some(i);
        }
        if let Some(i) = last {
            tasks.push(self.set_active(i));
        }
        tasks.push(self.notify(fl!("recovered", count = count)));
        Task::batch(tasks)
    }

    // ---- Navegação ---------------------------------------------------------

    /// Onde o cursor está agora.
    fn here(&self) -> Option<nav::Spot> {
        let tab = self.tabs.get(self.active)?;
        let (line, column) = tab.editor.cursor();
        Some(nav::Spot { path: tab.editor.path()?.to_path_buf(), line, column })
    }

    /// Antes de saltar para `path`: guarda onde estava, para voltar.
    fn remember_spot(&mut self, path: &Path, line: Option<usize>) {
        if let Some(here) = self.here() {
            self.nav.remember(here, path, line);
        }
    }

    fn navigate(&mut self, forward: bool) -> Task<Message> {
        match self.nav.step(forward, self.here(), Path::is_file) {
            Some(to) => self.open_file_at(to.path, to.line, to.column),
            None => self.notify(if forward { fl!("nav-nothing-forward") } else { fl!("nav-nothing-back") }),
        }
    }

    /// Lista de lugares que o servidor mandou. Referências vão sempre para o painel; nas outras
    /// buscas, um resultado só salta direto para ele.
    fn show_locations(
        &mut self,
        lookup: tea_lsp::Lookup,
        origin: &Path,
        position: tea_lsp::Position,
        locations: &[tea_lsp::Location],
    ) -> Task<Message> {
        use tea_core::LocationKind as K;
        use tea_lsp::Lookup as L;
        let kind = match lookup {
            L::References => K::References,
            L::Definition => K::Definitions,
            L::Implementation => K::Implementations,
            L::TypeDefinition => K::TypeDefinitions,
        };
        match locations {
            [] => {
                return self.notify(match kind {
                    K::References => fl!("lsp-no-references"),
                    K::Definitions => fl!("lsp-no-definition"),
                    K::Implementations => fl!("lsp-no-implementations"),
                    K::TypeDefinitions => fl!("lsp-no-type"),
                });
            }
            [one] if kind != K::References => {
                let line = one.range.start.line as usize + 1;
                self.remember_spot(&one.path, Some(line));
                return self.open_file_at(one.path.clone(), line, one.range.start.character as usize + 1);
            }
            _ => {}
        }
        // Linhas de cada arquivo: da aba aberta (pode ter alterações) ou do disco.
        let mut files: std::collections::HashMap<PathBuf, Vec<String>> = std::collections::HashMap::new();
        let mut line_of = |app: &Self, path: &Path, line: usize| -> String {
            let lines = files.entry(path.to_path_buf()).or_insert_with(|| {
                let text = match app.tabs.iter().find(|t| t.editor.path() == Some(path)) {
                    Some(t) => t.editor.text(),
                    None => std::fs::read_to_string(path).unwrap_or_default(),
                };
                text.lines().map(str::to_string).collect()
            });
            lines.get(line).cloned().unwrap_or_default()
        };
        // O nome procurado: a palavra na posição de onde se pediu.
        let symbol = word_at(&line_of(self, origin, position.line as usize), position.character as usize);
        let items: Vec<_> = locations
            .iter()
            .map(|loc| {
                let (start, end) = (loc.range.start, loc.range.end);
                let full = line_of(self, &loc.path, start.line as usize);
                let len = if end.line == start.line { end.character.saturating_sub(start.character) as usize } else { 0 };
                tea_core::Reference {
                    path: loc.path.clone(),
                    line: start.line as usize + 1,
                    column: start.character as usize + 1,
                    len,
                    preview: full.trim().to_string(),
                    indent: full.chars().take_while(|c| c.is_whitespace()).count(),
                }
            })
            .collect();
        let only_declaration = kind == K::References && items.len() == 1;
        let shown = self.broadcast(CoreEvent::Locations { kind, symbol: symbol.clone(), items });
        if only_declaration {
            return Task::batch([shown, self.notify(fl!("lsp-not-used-elsewhere", symbol = symbol))]);
        }
        shown
    }

    // ---- Estrutura ---------------------------------------------------------

    /// Pede a estrutura do arquivo ativo. Sem servidor, manda a lista vazia com o motivo.
    fn request_outline(&mut self) -> Task<Message> {
        let tab = &mut self.tabs[self.active];
        self.outline_seen = (tab.id, tab.editor.text_revision());
        let note = match tab.editor.path().map(Path::to_path_buf) {
            None => fl!("outline-save-first"),
            Some(_) if tab.editor.request_document_symbols() => return Task::none(),
            Some(_) => self.why_no_lsp(),
        };
        let path = self.active_path().map(Path::to_path_buf);
        self.set_outline(path, Default::default(), Some(note))
    }

    /// Guarda a estrutura (para `Ctrl+Shift+O`), atualiza a paleta se ela estiver mostrando os
    /// símbolos do arquivo e avisa as extensões (painel Estrutura).
    fn set_outline(
        &mut self,
        path: Option<PathBuf>,
        symbols: std::sync::Arc<Vec<tea_core::outline::Symbol>>,
        note: Option<String>,
    ) -> Task<Message> {
        self.last_outline = (path.clone(), symbols.clone(), note.clone());
        if let Some(p) = &mut self.palette
            && p.mode == palette::Mode::FileSymbols
        {
            p.set_symbols(symbol_items(path.as_deref().unwrap_or(Path::new("")), &symbols), note.clone());
        }
        self.broadcast(CoreEvent::Outline { path, symbols, note })
    }

    // ---- Símbolos na paleta ----------------------------------------------------

    /// `Ctrl+Shift+O`: símbolos do arquivo ativo (da última estrutura, ou pedida agora).
    fn open_file_symbols(&mut self) -> Task<Message> {
        let active = self.active_path().map(Path::to_path_buf);
        let (path, symbols, note) = self.last_outline.clone();
        if active.is_some() && path == active {
            let items = symbol_items(path.as_deref().unwrap_or(Path::new("")), &symbols);
            return self.open_palette(Palette::symbols(false, &self.root, items, note));
        }
        let open = self.open_palette(Palette::symbols(false, &self.root, Vec::new(), Some(fl!("outline-reading"))));
        Task::batch([open, self.request_outline()])
    }

    /// `Ctrl+T`: símbolos do projeto; a busca vai ao servidor conforme se digita.
    fn open_project_symbols(&mut self) -> Task<Message> {
        let note = if self.project_symbols_client().is_some() {
            fl!("symbols-type-a-name")
        } else {
            self.why_no_lsp()
        };
        self.open_palette(Palette::symbols(true, &self.root, Vec::new(), Some(note)))
    }

    /// O servidor da aba ativa, ou qualquer um rodando.
    fn project_symbols_client(&mut self) -> Option<tea_lsp::Client> {
        let active = self.active_path().map(Path::to_path_buf);
        let own = active.and_then(|p| self.lsp.client_for(&p).ok().flatten()).map(|(c, _)| c);
        own.or_else(|| self.lsp.any_running())
    }

    /// Mudou o texto da busca de símbolos do projeto: busca daqui a pouco (cada tecla adia).
    fn project_symbols_later(&mut self) -> Task<Message> {
        self.symbol_wait += 1;
        let wait = self.symbol_wait;
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            let _ = tx.send(wait);
        });
        Task::future(async move { rx.await.ok() }).and_then(|wait| Task::done(Message::SymbolQueryDue(wait)))
    }

    fn search_project_symbols(&mut self) -> Task<Message> {
        let Some(query) = self
            .palette
            .as_ref()
            .filter(|p| p.mode == palette::Mode::ProjectSymbols)
            .map(|p| p.query.trim().to_string())
        else {
            return Task::none();
        };
        let client = self.project_symbols_client();
        let Some(p) = &mut self.palette else { return Task::none() };
        match client {
            _ if query.is_empty() => p.set_symbols(Vec::new(), Some(fl!("symbols-type-a-name"))),
            Some(client) => {
                client.workspace_symbols(&query);
                if p.symbols.is_empty() {
                    p.set_symbols(Vec::new(), Some(fl!("searching")));
                }
            }
            None => {
                let why = self.why_no_lsp();
                if let Some(p) = &mut self.palette {
                    p.set_symbols(Vec::new(), Some(why));
                }
            }
        }
        Task::none()
    }

    fn show_project_symbols(&mut self, query: &str, symbols: Vec<tea_lsp::WorkspaceSymbol>) -> Task<Message> {
        let Some(p) = &mut self.palette else { return Task::none() };
        // Resposta de uma busca velha (o texto já mudou).
        if p.mode != palette::Mode::ProjectSymbols || p.query.trim() != query {
            return Task::none();
        }
        let items = symbols
            .into_iter()
            .map(|s| palette::SymbolItem {
                name: s.name,
                container: s.container.unwrap_or_default(),
                kind: tea_core::outline::Kind::from_lsp(s.kind),
                visibility: tea_core::outline::Visibility::Unknown,
                path: s.path,
                line: s.position.line as usize + 1,
                column: s.position.character as usize + 1,
            })
            .collect::<Vec<_>>();
        let note = items
            .is_empty()
            .then(|| fl!("symbols-none-found"));
        p.set_symbols(items, note);
        Task::none()
    }

    /// Escolheu um símbolo na paleta (`index` em `symbols`, ou o selecionado).
    fn open_chosen_symbol(&mut self, index: Option<usize>) -> Task<Message> {
        let Some(s) = self.palette.as_ref().and_then(|p| p.chosen_symbol(index)).cloned() else { return Task::none() };
        self.close_palette();
        self.remember_spot(&s.path, Some(s.line));
        self.open_file_at(s.path, s.line, s.column)
    }

    // ---- Texto para as extensões (prévia) -----------------------------------

    /// Alguma extensão quer o texto do arquivo ativo.
    fn watching_active_text(&self) -> bool {
        let ext = self.active_path().and_then(|p| p.extension()).map(|e| e.to_string_lossy().to_lowercase());
        ext.is_some_and(|e| self.text_watch.contains(&e))
    }

    /// Manda o texto do arquivo ativo (se alguém quer e se mudou desde o último envio).
    fn send_active_text(&mut self) -> Task<Message> {
        if !self.watching_active_text() {
            return Task::none();
        }
        let Some(path) = self.active_path().map(Path::to_path_buf) else { return Task::none() };
        let text = self.tabs[self.active].editor.text();
        if let Some((p, t)) = &self.text_sent
            && *p == path
            && **t == *text
        {
            return Task::none();
        }
        let text: std::sync::Arc<str> = text.into();
        self.text_sent = Some((path.clone(), text.clone()));
        self.broadcast(CoreEvent::ActiveText { path, text })
    }

    /// Compara com o último commit daqui a pouco (cada edição adia; só a última espera vale).
    fn diff_later(&mut self) -> Task<Message> {
        self.diff_wait += 1;
        let wait = self.diff_wait;
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            let _ = tx.send(wait);
        });
        Task::future(async move { rx.await.ok() }).and_then(|wait| Task::done(Message::DiffDue(wait)))
    }

    /// Religa daqui a pouco os servidores que caíram (cada edição adia; só a última espera vale). Esperar a pausa
    /// evita reabrir um comando pela metade, que pode derrubar o servidor de novo (ex.: `hare-lsp` sem o `;`).
    fn relink_later(&mut self) -> Task<Message> {
        self.relink_wait += 1;
        let wait = self.relink_wait;
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            let _ = tx.send(wait);
        });
        Task::future(async move { rx.await.ok() }).and_then(|wait| Task::done(Message::RelinkDue(wait)))
    }

    /// Manda o texto daqui a pouco (cada edição adia; só a última espera vale).
    fn text_later(&mut self) -> Task<Message> {
        self.text_wait += 1;
        let wait = self.text_wait;
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            let _ = tx.send(wait);
        });
        Task::future(async move { rx.await.ok() }).and_then(|wait| Task::done(Message::TextDue(wait)))
    }

    /// Pede a estrutura daqui a pouco (cada edição adia; só a última espera vale).
    fn outline_later(&mut self) -> Task<Message> {
        self.outline_wait += 1;
        let wait = self.outline_wait;
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            let _ = tx.send(wait);
        });
        Task::future(async move { rx.await.ok() }).and_then(|wait| Task::done(Message::OutlineDue(wait)))
    }

    fn show_outline(&mut self, path: &Path, symbols: &[tea_lsp::DocumentSymbol]) -> Task<Message> {
        // Resposta atrasada de um arquivo que não está mais ativo.
        if self.active_path() != Some(path) {
            return Task::none();
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        let symbols = outline::build(symbols, &self.editor().text(), ext);
        let note = symbols.is_empty().then(|| fl!("symbols-none-in-file"));
        self.set_outline(Some(path.to_path_buf()), std::sync::Arc::new(symbols), note)
    }

    /// Pediu algo do LSP (definição, referências) num arquivo sem servidor: diz por quê.
    fn no_lsp_note(&mut self) -> Task<Message> {
        let why = self.why_no_lsp();
        self.notify(why)
    }

    fn why_no_lsp(&self) -> String {
        if self.editor().is_light() {
            return fl!("lsp-light-mode");
        }
        match self.active_path() {
            Some(path) => self.lsp.why_missing(path),
            None => fl!("lsp-save-first"),
        }
    }

    /// Liga a aba ao servidor de linguagem do arquivo, se houver.
    fn ensure_lsp(&mut self, i: usize) -> Task<Message> {
        let editor = &self.tabs[i].editor;
        let Some(path) = editor.path().map(Path::to_path_buf) else { return Task::none() };
        if editor.lsp_server().is_some() || editor.is_light() {
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
                if let Some(i) = tab_for(self, &path) {
                    self.tabs[i].editor.set_diagnostics(&server, &diagnostics);
                }
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
            E::SignatureHelp { path, help } => {
                if let Some(i) = tab_for(self, &path) {
                    self.tabs[i].editor.show_signature(help);
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
                let line = position.line as usize + 1;
                self.remember_spot(&path, Some(line));
                self.open_file_at(path, line, position.character as usize + 1)
            }
            E::DefinitionNotFound => self.notify(fl!("lsp-no-definition")),
            // Ctrl+clique (ou F12) na própria declaração: lista onde ela é usada.
            E::AtDefinition { path, position } => match self.lsp.client_for(&path) {
                Ok(Some((client, _))) => {
                    client.references(&path, position);
                    self.notify(fl!("lsp-finding-references"))
                }
                _ => Task::none(),
            },
            E::Locations { lookup, path, position, locations } => self.show_locations(lookup, &path, position, &locations),
            E::CallRoots { path, items, .. } => {
                if items.is_empty() {
                    return self.notify(fl!("calls-not-in-function"));
                }
                let Ok(Some((client, _))) = self.lsp.client_for(&path) else { return Task::none() };
                let roots = self.calls.set_roots(&client, items);
                let direction = self.calls.direction;
                self.broadcast(CoreEvent::Calls(tea_core::calls::Event::Root { direction, roots }))
            }
            E::Calls { token, direction, calls } => match self.calls.children(token, direction, calls) {
                Some(nodes) => {
                    let direction = calls::core_direction(direction);
                    self.broadcast(CoreEvent::Calls(tea_core::calls::Event::Children { parent: token, direction, nodes }))
                }
                None => Task::none(),
            },
            E::Rename { new_name, files } => self.preview_rename(new_name, files),
            E::RenameFailed { message } => self.notify(fl!("rename-failed", message = message)),
            E::Formatting { path, edits } => {
                // Formatou para salvar (Ctrl+S): grava em seguida, sem o aviso de "formatado".
                if let Some(i) = self.format_save.and_then(|id| self.tab_index(id))
                    && self.tabs[i].editor.path() == Some(path.as_path())
                {
                    self.tabs[i].editor.apply_formatting(&edits);
                    let text = self.text_later();
                    return Task::batch([self.finish_format_save(), text]);
                }
                if let Some(i) = tab_for(self, &path) {
                    self.tabs[i].editor.apply_formatting(&edits);
                    let text = self.text_later();
                    return Task::batch([self.notify(fl!("lsp-formatted")), self.emit_status(false), text]);
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
                    tea_lsp::MessageLevel::Error => fl!("lsp-level-error"),
                    tea_lsp::MessageLevel::Warning => fl!("lsp-level-warning"),
                    tea_lsp::MessageLevel::Info => String::new(),
                };
                self.notify(format!("{server}: {prefix}{text}"))
            }
            E::Log { server, text } => {
                if std::env::var_os("IDETEA_LSP_LOG").is_some() {
                    eprintln!("[{server}] {text}");
                }
                Task::none()
            }
            E::Ready { server } => {
                let activity = self.broadcast(CoreEvent::Activity { source: server, text: None });
                Task::batch([activity, self.request_outline()])
            }
            E::DocumentSymbols { path, symbols } => self.show_outline(&path, &symbols),
            E::WorkspaceSymbols { query, symbols } => self.show_project_symbols(&query, symbols),
            E::Exited { server, client, reason } => {
                let exit = self.lsp.on_exit(&server, client, &reason);
                if exit == lsp::Exit::Stale {
                    return Task::none();
                }
                for tab in &mut self.tabs {
                    if tab.editor.lsp_server() == Some(server.as_str()) {
                        tab.editor.detach_lsp();
                    }
                }
                let activity = self.broadcast(CoreEvent::Activity { source: server.clone(), text: None });
                let outline = self.request_outline();
                let reason = lsp::with_hint(&server, &reason);
                let text = match exit {
                    // Quedas seguidas (ex.: a cada `;` faltando no Hare): avisa só a primeira.
                    lsp::Exit::WillRestart { first: false } => return Task::batch([activity, outline]),
                    lsp::Exit::WillRestart { first: true } => {
                        fl!("lsp-server-crashed", server = server.clone(), reason = reason)
                    }
                    lsp::Exit::TooManyCrashes => fl!(
                        "lsp-server-crashed-too-often",
                        server = server.clone(),
                        count = (lsp::MAX_CRASHES + 1).to_string(),
                        minutes = (lsp::CRASH_WINDOW.as_secs() / 60).to_string(),
                        reason = reason
                    ),
                    _ => fl!("lsp-server-stopped", server = server.clone(), reason = reason),
                };
                let note = self.notify(text);
                Task::batch([activity, outline, note])
            }
        }
    }

    fn set_active(&mut self, i: usize) -> Task<Message> {
        let i = i.min(self.tabs.len() - 1);
        // Trocou de lado: a aba de antes continua aparecendo no outro.
        let right = self.tabs[i].right;
        if let Some(old) = self.tabs.get_mut(self.active)
            && old.right != right
        {
            self.other_tab = Some(old.id);
            old.editor.lose_focus();
        }
        self.active = i;
        if self.palette.is_none() && self.splash.is_none() {
            self.editor_mut().focus();
        }
        let path = self.active_path().map(Path::to_path_buf);
        let changed = self.broadcast(CoreEvent::ActiveFileChanged(path));
        // As extensões esquecem o texto ao trocar de arquivo: manda de novo mesmo sem mudança.
        self.text_sent = None;
        Task::batch([changed, self.emit_status(false), self.request_outline(), self.send_active_text()])
    }

    fn close_tab(&mut self, id: u64, force: bool) -> Task<Message> {
        let Some(i) = self.tab_index(id) else { return Task::none() };
        let editor = &self.tabs[i].editor;
        if editor.is_modified() && !force {
            let name = editor.file_name();
            let dialog = Dialog::new(fl!("save-changes-title"), fl!("save-changes-message", name = name))
                .button(fl!("save"), ButtonKind::Primary, CoreDialog::SaveAndClose(id))
                .button(fl!("discard"), ButtonKind::Danger, CoreDialog::DiscardAndClose(id))
                .cancel(fl!("cancel"));
            return self.open_dialog(dialog, dialog::Owner::Core);
        }
        if let Some(path) = editor.path() {
            let entry = (path.to_path_buf(), editor.cursor());
            self.closed_tabs.retain(|(p, _)| *p != entry.0);
            self.closed_tabs.push(entry);
            let excess = self.closed_tabs.len().saturating_sub(CLOSED_TABS_MAX);
            self.closed_tabs.drain(..excess);
        }
        let right = self.tabs[i].right;
        let active_id = self.tabs[self.active].id;
        let removed = self.tabs.remove(i).id;
        if self.tabs.is_empty() {
            (self.active, self.other_tab) = (0, None);
            self.push_tab(Editor::scratch(""));
        } else if removed == active_id {
            // A vizinha do mesmo lado; lado vazio: o foco vai para o outro.
            match self.side_neighbor(i, right, None) {
                Some(j) => self.active = j,
                // (`other_index` compara com a ativa, que acabou de sair.)
                None => self.active = self.other_tab.and_then(|id| self.tab_index(id)).unwrap_or(0),
            }
        } else {
            if self.other_tab == Some(removed) {
                self.other_tab = self.side_neighbor(i, right, None).map(|j| self.tabs[j].id);
            }
            self.active = self.tab_index(active_id).unwrap_or(0);
        }
        self.normalize_split();
        self.set_active(self.active)
    }

    /// Ctrl+Shift+T: reabre a última aba fechada, no mesmo ponto. Pula as que já estão abertas de novo e os
    /// arquivos que não existem mais.
    fn reopen_closed_tab(&mut self) -> Task<Message> {
        while let Some((path, (line, column))) = self.closed_tabs.pop() {
            let open = self.tabs.iter().any(|t| t.editor.path() == Some(path.as_path()));
            if !open && path.is_file() {
                return self.open_file_at(path, line, column);
            }
        }
        Task::none()
    }

    /// Editor dividido: índice da aba mostrada no lado sem foco.
    fn other_index(&self) -> Option<usize> {
        let i = self.tab_index(self.other_tab?)?;
        (self.tabs[i].right != self.tabs.get(self.active)?.right).then_some(i)
    }

    /// Aba do lado `right` mais perto da posição `i` (a seguinte, senão a anterior), sem contar `skip`.
    fn side_neighbor(&self, i: usize, right: bool, skip: Option<usize>) -> Option<usize> {
        let ok = |j: &usize| self.tabs[*j].right == right && Some(*j) != skip;
        (i..self.tabs.len()).find(ok).or_else(|| (0..i.min(self.tabs.len())).rev().find(ok))
    }

    /// Um lado ficou vazio: volta a ser um editor só, tudo do lado esquerdo.
    fn normalize_split(&mut self) {
        if self.other_index().is_none() {
            self.other_tab = None;
            self.tabs.iter_mut().for_each(|t| t.right = false);
        }
    }

    /// `Ctrl+\`: manda a aba para o outro lado (sem divisão, divide). Era a única do lado: junta tudo de novo.
    fn move_to_other_side(&mut self, id: u64) -> Task<Message> {
        let Some(i) = self.tab_index(id) else { return Task::none() };
        if self.tabs.len() < 2 {
            return self.notify(fl!("split-one-tab"));
        }
        let focus = self.set_active(i);
        let from = self.tabs[i].right;
        match self.side_neighbor(i, from, Some(i)) {
            Some(stay) => {
                self.other_tab = Some(self.tabs[stay].id);
                self.tabs[i].right = !from;
            }
            None => {
                self.other_tab = None;
                self.normalize_split();
            }
        }
        Task::batch([focus, self.emit_status(true)])
    }

    /// `Ctrl+1` / `Ctrl+2`: foco no lado esquerdo / direito.
    fn focus_side(&mut self, right: bool) -> Task<Message> {
        if self.tabs[self.active].right == right {
            self.editor_mut().focus();
            return Task::none();
        }
        match self.other_index() {
            Some(j) => self.set_active(j),
            None => Task::none(),
        }
    }

    /// `Ctrl+PageDown` / `PageUp`: próxima aba do mesmo lado.
    fn cycle_tab(&mut self, forward: bool) -> Task<Message> {
        let right = self.tabs[self.active].right;
        let side: Vec<usize> = (0..self.tabs.len()).filter(|&j| self.tabs[j].right == right).collect();
        let pos = side.iter().position(|&j| j == self.active).unwrap_or(0);
        let next = if forward { (pos + 1) % side.len() } else { (pos + side.len() - 1) % side.len() };
        self.set_active(side[next])
    }

    fn save_active(&mut self) -> Task<Message> {
        // Imagem não se edita: não há o que salvar (nem aviso de "salvo").
        if self.editor().is_image() {
            return Task::none();
        }
        if self.editor().path().is_none() {
            return self.open_save_as();
        }
        if self.config.editor.format_on_save && self.format_save.is_none() && self.editor_mut().request_formatting() {
            self.format_save = Some(self.tabs[self.active].id);
            return self.format_save_later();
        }
        self.save_active_now()
    }

    fn save_active_now(&mut self) -> Task<Message> {
        let result = self.editor_mut().save();
        self.after_save(result)
    }

    /// Espera a formatação por até 2 s; depois disso grava sem formatar.
    fn format_save_later(&mut self) -> Task<Message> {
        self.format_save_wait += 1;
        let wait = self.format_save_wait;
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            let _ = tx.send(wait);
        });
        Task::future(async move { rx.await.ok() }).and_then(|wait| Task::done(Message::FormatSaveDue(wait)))
    }

    /// Grava a aba que esperava a formatação.
    fn finish_format_save(&mut self) -> Task<Message> {
        let Some(id) = self.format_save.take() else { return Task::none() };
        match self.tab_index(id) {
            Some(i) if i == self.active => self.save_active_now(),
            Some(_) => self.save_tabs(&[id]).1,
            None => Task::none(),
        }
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
            return self.notify(fl!("picker-already-open"));
        }
        self.picking = true;
        Task::perform(dialog, Message::Picked)
    }

    fn picked(&mut self, picked: picker::Picked) -> Task<Message> {
        self.picking = false;
        match picked {
            picker::Picked::SaveAs { tab, path: Some(path) } => {
                let Some(i) = self.tab_index(tab) else { return self.notify(fl!("save-tab-closed")) };
                let activate = self.set_active(i);
                Task::batch([activate, self.save_as_path(path)])
            }
            picker::Picked::Files(Some(files)) => Task::batch(files.into_iter().map(|f| self.open_file(f)).collect::<Vec<_>>()),
            picker::Picked::DebugProgram(Some(program)) => {
                self.debug.program = Some(program);
                self.debug_start()
            }
            picker::Picked::Folder(Some(dir)) => match picker::open_window(&dir) {
                Ok(()) => self.notify(fl!("folder-opening", path = dir.display().to_string())),
                Err(e) => self.notify(fl!("folder-open-error", error = e.to_string())),
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
        if ok {
            // Outro nome, outra extensão: Markdown, `.editorconfig` de outra pasta...
            let options = self.editor_options(self.editor());
            self.editor_mut().set_options(&options);
        }
        let lsp = if ok { self.ensure_lsp(self.active) } else { Task::none() };
        let saved = Task::batch([self.after_save(result), lsp]);
        let changed = if ok { self.broadcast(CoreEvent::ActiveFileChanged(Some(path))) } else { Task::none() };
        let outline = if ok { self.request_outline() } else { Task::none() };
        let text = if ok { self.send_active_text() } else { Task::none() };
        Task::batch([saved, changed, outline, text])
    }

    fn after_save(&mut self, result: std::io::Result<()>) -> Task<Message> {
        match result {
            Ok(()) => {
                let path = self.active_path().map(Path::to_path_buf).unwrap_or_default();
                self.own_saves.insert(path.clone(), Instant::now());
                let is_config = Some(path.as_path()) == self.global_config.as_deref()
                    || path == config::project_config_path(&self.root);
                if path.file_name().is_some_and(|n| n == ".editorconfig") {
                    self.refresh_editor_options();
                }
                let saved = self.broadcast(CoreEvent::FileSaved(path.clone()));
                let note = self.notify(fl!("saved", path = display_rel(&path, &self.root)));
                let reload = if is_config { self.reload_config() } else { Task::none() };
                Task::batch([saved, note, reload, self.emit_status(false)])
            }
            Err(err) => self.notify(fl!("save-error", error = err.to_string())),
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
        self.refresh_editor_options();
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
                tasks.push(self.notify(fl!("file-deleted-kept")));
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
        problems.extend(tea_core::fonts::problem(&config.editor));
        problems.extend(tea_core::file_icons::problem(&config.ui.file_icons));
        tea_core::file_icons::set_enabled(config.ui.file_icons.trim() != "none");
        self.keymap = keymap;
        self.lsp.update_config(&config.lsp);
        self.config = config;
        self.editor_zoom.reload_base(self.config.ui.font_size);
        self.set_theme(theme);
        let (font_size, code_font) = (self.editor_zoom.size(), self.code_font());
        for tab in &mut self.tabs {
            tab.editor.set_code_font(font_size, code_font);
        }
        self.refresh_editor_options();
        self.apply_panel_slots();
        let msg = if !problems.is_empty() {
            Some(fl!("config-reloaded-problems", problems = problems.join(" · ")))
        } else if announce {
            Some(fl!("config-reloaded"))
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
                return self.notify(fl!("config-no-home"));
            };
            if !path.exists() {
                let created = path
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&path, config::user_config_template()));
                if let Err(e) = created {
                    return self.notify(fl!("create-error", path = path.display().to_string(), error = e.to_string()));
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
            "editor.toggle_fold" => Some(EditorAction::ToggleFold),
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
            "file.open_recent" => self.open_recent(),
            "debug.toggle_breakpoint" => {
                let line = self.editor().cursor().0 - 1;
                self.editor_mut().toggle_breakpoint(line);
                self.sync_breakpoints(self.active);
                Task::none()
            }
            "debug.continue" => self.debug_continue(),
            "debug.stop" => self.debug_stop(),
            "debug.choose_program" => {
                let dir = self.root.clone();
                self.pick(picker::debug_program(dir))
            }
            "debug.pause" | "debug.next" | "debug.step_in" | "debug.step_out" => {
                let Some(session) = self.debug.session() else {
                    return self.notify(fl!("debug-no-session"));
                };
                let stopped = self.debug.stopped.is_some();
                match id {
                    "debug.pause" if !stopped => session.pause(),
                    "debug.next" if stopped => session.next(),
                    "debug.step_in" if stopped => session.step_in(),
                    "debug.step_out" if stopped => session.step_out(),
                    "debug.pause" => return self.notify(fl!("debug-already-paused")),
                    _ => return self.notify(fl!("debug-pause-first")),
                }
                Task::none()
            }
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
            "file.save_all" => {
                let (ok, task) = self.save_all();
                if ok { task } else { Task::batch([task, self.notify(fl!("save-all-untitled"))]) }
            }
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
            "tab.next" => self.cycle_tab(true),
            "tab.prev" => self.cycle_tab(false),
            "tab.reopen" => self.reopen_closed_tab(),
            "editor.split" => self.move_to_other_side(self.tabs[self.active].id),
            "editor.focus_left" => self.focus_side(false),
            "editor.focus_right" => self.focus_side(true),
            "editor.zoom_in" | "editor.zoom_out" | "editor.zoom_reset" => {
                match id {
                    "editor.zoom_in" => self.editor_zoom.adjust(1.0),
                    "editor.zoom_out" => self.editor_zoom.adjust(-1.0),
                    _ => self.editor_zoom.reset(),
                }
                let (font_size, code_font) = (self.editor_zoom.size(), self.code_font());
                for tab in &mut self.tabs {
                    tab.editor.set_code_font(font_size, code_font);
                }
                Task::none()
            }
            "theme.toggle" => {
                let next = if self.theme.extended_palette().is_dark { LIGHT_THEME } else { DARK_THEME };
                self.set_theme(next);
                self.broadcast_appearance()
            }
            "lsp.definition" => {
                if self.editor_mut().request_definition() { Task::none() } else { self.no_lsp_note() }
            }
            "lsp.references" => {
                if self.editor_mut().request_references() {
                    self.notify(fl!("lsp-finding-references"))
                } else {
                    self.no_lsp_note()
                }
            }
            "lsp.implementation" => {
                if self.editor_mut().request_implementation() {
                    self.notify(fl!("lsp-finding-implementations"))
                } else {
                    self.no_lsp_note()
                }
            }
            "lsp.incoming_calls" | "lsp.outgoing_calls" => {
                self.calls.direction = if id == "lsp.incoming_calls" {
                    tea_core::calls::Direction::Incoming
                } else {
                    tea_core::calls::Direction::Outgoing
                };
                if self.editor_mut().request_call_hierarchy() { Task::none() } else { self.no_lsp_note() }
            }
            "lsp.type_definition" => {
                if self.editor_mut().request_type_definition() { Task::none() } else { self.no_lsp_note() }
            }
            "nav.file_symbol" => self.open_file_symbols(),
            "nav.project_symbol" => self.open_project_symbols(),
            "nav.back" => self.navigate(false),
            "nav.forward" => self.navigate(true),
            "lsp.hover" => {
                self.editor_mut().request_hover();
                Task::none()
            }
            "lsp.rename" => self.start_rename(),
            "lsp.format" => {
                if self.editor_mut().request_formatting() {
                    Task::none()
                } else {
                    self.notify(fl!("lsp-no-formatting"))
                }
            }
            "lsp.restart" => {
                self.lsp.restart();
                for tab in &mut self.tabs {
                    tab.editor.detach_lsp();
                }
                let relink: Vec<Task<Message>> = (0..self.tabs.len()).map(|i| self.ensure_lsp(i)).collect();
                Task::batch([Task::batch(relink), self.notify(fl!("lsp-restarted"))])
            }
            "config.open" => self.open_config(true),
            "config.open_project" => self.open_config(false),
            "config.reload" => self.reload_config(),
            "view.toggle_left" => self.toggle_slot(Slot::Left),
            "view.toggle_right" => self.toggle_slot(Slot::Right),
            "view.toggle_bottom" => self.toggle_slot(Slot::Bottom),
            "app.quit" => {
                self.restart = false;
                self.request_quit()
            }
            "app.restart" => {
                self.restart = true;
                self.request_quit()
            }
            "tasks.run" => self.open_tasks(),
            "tasks.rerun" => match self.last_task.clone() {
                Some(task) => self.run_task(task),
                None => self.open_tasks(),
            },
            "tasks.test_here" => self.test_here(),
            "mascot.toggle" => {
                self.mascot.collapsed = !self.mascot.collapsed;
                self.ui_state.mascot_collapsed = self.mascot.collapsed;
                self.ui_state.save(self.state_path.as_deref());
                Task::none()
            }
            _ => {
                // Comandos de extensão: o prefixo antes do ponto é o id da extensão.
                let prefix = id.split('.').next().unwrap_or_default();
                match self.extensions.iter().position(|e| e.id() == prefix) {
                    Some(i) => {
                        // A extensão pode abrir um campo de texto: o editor não pode ficar com o teclado.
                        self.editor_mut().lose_focus();
                        self.call_ext(i, |e, ctx| e.on_command(id, ctx))
                    }
                    None => self.notify(fl!("unknown-command", id = id.to_string())),
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
            names.push_str(&format!("\n{}", fl!("and-more", count = (unsaved.len() - 5))));
        }
        let dialog = Dialog::new(fl!("quit-unsaved-title"), names)
            .button(fl!("quit-save-all"), ButtonKind::Primary, CoreDialog::SaveAllAndQuit)
            .button(fl!("quit-without-saving"), ButtonKind::Danger, CoreDialog::QuitWithoutSaving)
            .cancel(fl!("cancel"));
        self.open_dialog(dialog, dialog::Owner::Core)
    }

    fn quit(&mut self) -> Task<Message> {
        self.save_session(true);
        // Saiu pelo caminho normal (salvou ou escolheu descartar): as cópias não servem mais.
        if let Some(dir) = &self.backup_dir {
            backup::remove_mine(dir);
        }
        if let Some(size) = self.window_size {
            self.ui_state.set_window(size);
            self.ui_state.save(self.state_path.as_deref());
        }
        self.lsp.shutdown_all();
        self.debug.stop();
        ide_state::remove(&self.root, std::process::id());
        if self.restart
            && let Some(stamp) = &self.binary
            && let Err(e) = binary::restart(stamp)
        {
            // Não abriu a nova: fica nesta (a sessão já foi salva).
            self.restart = false;
            return self.notify(fl!("binary-restart-error", error = e.to_string()));
        }
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
                tasks.push(self.notify(fl!("save-needs-name", name = name)));
                continue;
            };
            match self.tabs[i].editor.save() {
                Ok(()) => {
                    if path.file_name().is_some_and(|n| n == ".editorconfig") {
                        self.refresh_editor_options();
                    }
                    self.own_saves.insert(path.clone(), Instant::now());
                    tasks.push(self.broadcast(CoreEvent::FileSaved(path)));
                }
                Err(e) => {
                    failed.push(id);
                    tasks.push(self.notify(fl!("save-file-error", path = display_rel(&path, &self.root), error = e.to_string())));
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
            names.push_str(&format!("\n{}", fl!("and-more", count = (modified.len() - 5))));
        }
        let title = fl!("close-tabs-title", count = ids.len());
        let dialog = Dialog::new(title, format!("{}\n{names}", fl!("close-tabs-unsaved")))
            .button(fl!("save-and-close"), ButtonKind::Primary, CoreDialog::CloseMany { ids: ids.clone(), save: true })
            .button(fl!("discard-changes"), ButtonKind::Danger, CoreDialog::CloseMany { ids, save: false })
            .cancel(fl!("cancel"));
        self.open_dialog(dialog, dialog::Owner::Core)
    }

    /// Abas do mesmo lado que `id`, na ordem da barra.
    fn side_ids(&self, id: u64) -> Vec<u64> {
        let right = self.tab_index(id).is_some_and(|i| self.tabs[i].right);
        self.tabs.iter().filter(|t| t.right == right).map(|t| t.id).collect()
    }

    fn tab_menu(&self, id: u64, at: iced::Point) -> tea_core::menu::Menu {
        use tea_core::icons::Icon;
        let i = self.tab_index(id).unwrap_or(0);
        // "Outras" e "à direita": do mesmo lado.
        let side = self.side_ids(id);
        let n = side.len();
        let at_side = side.iter().position(|&t| t == id).unwrap_or(0);
        let mut menu = tea_core::menu::Menu::new(at).item(Some(Icon::Close), fl!("tab-close"), CoreMenu::Close(id));
        menu = if n > 1 { menu.item(None, fl!("tab-close-others"), CoreMenu::CloseOthers(id)) } else { menu.disabled(None, fl!("tab-close-others")) };
        menu = if at_side + 1 < n {
            menu.item(None, fl!("tab-close-right"), CoreMenu::CloseRight(id))
        } else {
            menu.disabled(None, fl!("tab-close-right"))
        };
        menu = menu.item(None, fl!("tab-close-saved"), CoreMenu::CloseSaved).item(None, fl!("tab-close-all"), CoreMenu::CloseAll);
        let other = if self.tabs.get(i).is_some_and(|t| t.right) { fl!("tab-move-left") } else { fl!("tab-move-right") };
        menu = if self.tabs.len() > 1 {
            menu.separator().item(None, other, CoreMenu::OtherSide(id))
        } else {
            menu.separator().disabled(None, other)
        };
        if let Some(path) = self.tabs.get(i).and_then(|t| t.editor.path()) {
            let rel = display_rel(path, &self.root);
            let dir = path.parent().unwrap_or(&self.root).to_path_buf();
            menu = menu
                .separator()
                .item(Some(Icon::Copy), fl!("copy-path"), CoreMenu::CopyPath(path.display().to_string()))
                .item(None, fl!("copy-relative-path"), CoreMenu::CopyPath(rel));
            if self.extensions.iter().any(|e| e.id() == "filetree") {
                menu = menu.item(Some(Icon::Files), fl!("reveal-in-tree"), CoreMenu::Reveal(id));
            }
            menu = menu.item(Some(Icon::LinkExternal), fl!("open-in-file-manager"), CoreMenu::OpenInFileManager(dir));
        }
        menu
    }

    fn core_menu(&mut self, action: CoreMenu) -> Task<Message> {
        let ids = |app: &Self| app.tabs.iter().map(|t| t.id).collect::<Vec<u64>>();
        match action {
            CoreMenu::Close(id) => self.close_tab(id, false),
            CoreMenu::CloseOthers(id) => {
                let others = self.side_ids(id).into_iter().filter(|&t| t != id).collect();
                self.close_many(others)
            }
            CoreMenu::OtherSide(id) => self.move_to_other_side(id),
            CoreMenu::CloseRight(id) => {
                let all = self.side_ids(id);
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
                let note = self.notify(fl!("copied", text = text.clone()));
                Task::batch([iced::clipboard::write(text), note])
            }
            CoreMenu::Reveal(id) => {
                let Some(i) = self.tab_index(id) else { return Task::none() };
                let activate = self.set_active(i);
                Task::batch([activate, self.run_command("filetree.reveal")])
            }
            CoreMenu::OpenInFileManager(dir) => match tea_core::os::open_in_file_manager(&dir) {
                Ok(()) => Task::none(),
                Err(e) => self.notify(fl!("file-manager-error", error = e.to_string())),
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
        // Largura do meio (sem os painéis dos lados), para o arraste da divisão do editor.
        let shown = |slot: Slot| self.panels.iter().any(|p| p.slot == slot && p.visible);
        let center = self.window_size.map_or(1000.0, |z| {
            z.width - if shown(Slot::Left) { self.ui_state.left_width } else { 0.0 }
                - if shown(Slot::Right) { self.ui_state.right_width } else { 0.0 }
                - 60.0
        });
        let s = &mut self.ui_state;
        let w = self.window_size.map(|z| z.width);
        let h = self.window_size.map(|z| z.height);
        match edge {
            Edge::Left => s.left_width = resize(s.left_width, delta, MIN_SIDE, w.map(|w| w - s.right_width)),
            Edge::Right => s.right_width = resize(s.right_width, -delta, MIN_SIDE, w.map(|w| w - s.left_width)),
            // Abas e barra de status também precisam caber.
            Edge::Bottom => s.bottom_height = resize(s.bottom_height, -delta, MIN_BOTTOM, h.map(|h| h - 60.0)),
            Edge::Split => {
                let lim = ui_state::MIN_SPLIT;
                s.split = (s.split + delta / center.max(1.0)).clamp(lim, 1.0 - lim);
            }
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
                None => Err(fl!("config-no-home")),
            },
        }
    }

    /// Muda o arquivo escolhido na tela, grava e recarrega (sem anunciar).
    fn edit_config(&mut self, f: impl FnOnce(&mut tea_core::config_edit::ConfigFile)) -> Task<Message> {
        let mut file = match self.settings_target_file() {
            Ok(file) => file,
            Err(e) => return self.notify(fl!("config-error", error = e.to_string())),
        };
        f(&mut file);
        if let Err(e) = file.save() {
            return self.notify(fl!("write-error", path = file.path().display().to_string(), error = e.to_string()));
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
            st.record_hint = Some(fl!("keys-record-hint"));
            return Task::none();
        }
        st.recording = None;
        st.record_hint = None;
        let previous = self.keymap.command_for(&chord).map(str::to_string);
        let (shown, title) = (chord.to_string(), self.command_title(&cmd));
        let note = match previous {
            Some(p) if p == cmd => fl!("keys-already-bound", key = shown.clone(), command = title.clone()),
            Some(p) => fl!("keys-rebound", key = shown.clone(), old = self.command_title(&p), command = title.clone()),
            None => fl!("keys-bound", key = shown.clone(), command = title.clone()),
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
            M::CodeFont(name) => self.edit_config(|f| f.set_editor_text("font", &name)),
            M::FontWeight(w) => self.edit_config(|f| f.set_editor_text("font_weight", &w)),
            M::LineHeight(v) => {
                let v = v.clamp(tea_core::fonts::LINE_HEIGHT_MIN, tea_core::fonts::LINE_HEIGHT_MAX);
                self.edit_config(|f| f.set_line_height(v))
            }
            M::FontSize(v) => {
                let v = v.clamp(settings::FONT_MIN, settings::FONT_MAX);
                self.edit_config(|f| f.set_font_size(v))
            }
            M::Splash(on) => self.edit_config(|f| f.set_splash(on)),
            M::FileIcons(on) => self.edit_config(|f| f.set_file_icons(on)),
            M::RestoreSession(on) => self.edit_config(|f| f.set_restore_session(on)),
            M::EditorFlag(key, on) => self.edit_config(|f| f.set_editor_flag(key, on)),
            M::TabSize(n) => {
                let n = n.clamp(settings::TAB_MIN, settings::TAB_MAX);
                self.edit_config(|f| f.set_tab_size(n))
            }
            M::ActivityBar(on) => self.edit_config(|f| f.set_activity_bar(on)),
            M::Language(choice) => self.edit_config(|f| f.set_language(choice.tag())),
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
                    let note = self.notify(fl!("save-then-close"));
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
            CoreDialog::DebugProgram(program) => {
                self.debug.program = Some(program);
                self.debug_start()
            }
            CoreDialog::Recover => self.recover(),
            CoreDialog::ApplyRename => self.apply_rename(),
            CoreDialog::DiscardRecovery => {
                for o in std::mem::take(&mut self.pending_recovery) {
                    o.remove();
                }
                Task::none()
            }
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
                if p.mode == palette::Mode::ProjectSymbols { self.project_symbols_later() } else { Task::none() }
            }
            palette::Msg::PickSymbol(i) => self.open_chosen_symbol(Some(i)),
            palette::Msg::PickRecent(i) => self.open_chosen_recent(Some(i)),
            palette::Msg::PickItem(_) | palette::Msg::PickCreate => self.choose_pick(Some(&msg)),
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
                palette::Mode::Rename => {
                    let new_name = p.query.trim().to_string();
                    self.close_palette();
                    self.request_rename(new_name)
                }
                palette::Mode::SaveAs => {
                    let input = p.query.clone();
                    self.close_palette();
                    self.save_as(&input)
                }
                palette::Mode::Files => self.open_chosen_file(None),
                palette::Mode::FileSymbols | palette::Mode::ProjectSymbols => self.open_chosen_symbol(None),
                palette::Mode::Recent => self.open_chosen_recent(None),
                palette::Mode::Pick => self.choose_pick(None),
            },
        }
    }

    /// Item da lista de uma extensão: fecha a paleta e manda a mensagem dele para a extensão.
    fn choose_pick(&mut self, msg: Option<&palette::Msg>) -> Task<Message> {
        let Some(p) = &self.palette else { return Task::none() };
        let Some(chosen) = p.chosen_pick(msg) else { return Task::none() };
        let owner = p.pick_owner;
        self.close_palette();
        match owner {
            Some(owner) => self.call_ext(owner, |e, ctx| e.update(chosen, ctx)),
            None => match chosen.downcast_ref::<tasks::TaskDef>() {
                Some(task) => self.run_task(task.clone()),
                None => Task::none(),
            },
        }
    }

    /// `Ctrl+Shift+B`: lista das tarefas do projeto (a última rodada vem primeiro).
    fn open_tasks(&mut self) -> Task<Message> {
        let mut list = tasks::list_for(&self.root, self.active_path(), &self.config.tasks);
        if let Some(last) = &self.last_task {
            match list.iter().position(|t| t == last) {
                Some(i) => list[..=i].rotate_right(1),
                None => list.insert(0, last.clone()),
            }
        }
        if list.is_empty() {
            return self.notify(fl!("tasks-none"));
        }
        let mut pick = tea_core::pick::Pick::new(fl!("tasks-placeholder"));
        for task in list {
            let detail = if task.command == task.name { String::new() } else { task.command.clone() };
            pick = pick.item(Some(tea_core::icons::Icon::Terminal), task.name.clone(), detail, task);
        }
        self.editor_mut().lose_focus();
        self.open_palette(Palette::pick(None, pick))
    }

    /// Manda a tarefa para a extensão do terminal.
    fn run_task(&mut self, task: tasks::TaskDef) -> Task<Message> {
        if !self.extensions.iter().any(|e| e.id() == "terminal") {
            return self.notify(fl!("tasks-no-terminal"));
        }
        self.last_task = Some(task.clone());
        let done = fl!("tasks-done", name = task.name.clone(), code = "{code}");
        self.broadcast(CoreEvent::RunTask { name: task.name, command: task.command, cwd: task.cwd, done })
    }

    /// Rust: roda o teste em volta do cursor.
    fn test_here(&mut self) -> Task<Message> {
        let Some(path) = self.active_path().map(Path::to_path_buf) else { return self.notify(fl!("tasks-no-test")) };
        let ed = self.editor();
        let found = if path.extension().is_some_and(|e| e == "rs") { tasks::rust_test_at(&ed.text(), ed.cursor().0) } else { None };
        let Some(name) = found else { return self.notify(fl!("tasks-no-test")) };
        match tasks::rust_test_task(&self.root, &path, &name) {
            Some(task) => self.run_task(task),
            None => self.notify(fl!("tasks-no-package")),
        }
    }

    /// `Ctrl+R`: projetos (menos este) e arquivos (menos o ativo) que ainda existem.
    fn open_recent(&mut self) -> Task<Message> {
        let list = self.recent_path.as_deref().map(recent::Recent::load).unwrap_or_default();
        let active = self.active_path().map(Path::to_path_buf);
        let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.display().to_string());
        let place = |p: &Path| p.parent().map(recent::display).unwrap_or_default();
        let projects = list.projects.into_iter().filter(|p| *p != self.root && p.is_dir()).map(|p| palette::RecentItem {
            project: true,
            name: name(&p),
            place: place(&p),
            path: p,
        });
        let files = list.files.into_iter().filter(|p| Some(p) != active.as_ref() && p.is_file()).map(|p| palette::RecentItem {
            project: false,
            name: name(&p),
            place: place(&p),
            path: p,
        });
        self.open_palette(Palette::recent(projects.chain(files).collect()))
    }

    /// Recente escolhido: projeto abre numa nova janela, arquivo numa aba.
    fn open_chosen_recent(&mut self, index: Option<usize>) -> Task<Message> {
        let Some(item) = self.palette.as_ref().and_then(|p| p.chosen_recent(index)).cloned() else { return Task::none() };
        self.close_palette();
        if !item.project {
            self.remember_spot(&item.path, None);
            return self.open_file(item.path);
        }
        match picker::open_window(&item.path) {
            Ok(()) => self.notify(fl!("folder-opening", path = item.path.display().to_string())),
            Err(e) => self.notify(fl!("folder-open-error", error = e.to_string())),
        }
    }

    /// Modo arquivos da paleta: abre o escolhido (`index`) ou o selecionado, na linha se a busca for `nome:42`.
    fn open_chosen_file(&mut self, index: Option<usize>) -> Task<Message> {
        let Some((rel, line)) = self.palette.as_ref().and_then(|p| p.chosen_file(index)) else { return Task::none() };
        self.close_palette();
        let path = self.root.join(rel);
        self.remember_spot(&path, line);
        match line {
            Some(line) => self.open_file_at(path, line, 1),
            None => self.open_file(path),
        }
    }

    fn on_key(
        &mut self,
        key: keyboard::Key,
        modified: &keyboard::Key,
        modifiers: keyboard::Modifiers,
        captured: bool,
    ) -> Task<Message> {
        use keyboard::key::Named;
        let command = self.keymap.command_for_event(&key, modified, modifiers).map(str::to_string);

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
                // O atalho do próprio modo fecha; o de outro modo troca (Ctrl+P ⇄ Ctrl+Shift+P ⇄ Ctrl+T...).
                _ if let Some(mode) = command.as_deref().and_then(palette_mode_of) => {
                    if mode == p.mode {
                        self.close_palette();
                    } else {
                        let id = command.unwrap_or_default();
                        self.close_palette();
                        return self.run_command(&id);
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
        let task = self.handle(message);
        self.save_session(false);
        self.publish_ide_state();
        task
    }

    /// O que a IDE mostra agora, para `idetea status`.
    fn ide_state_snapshot(&self) -> ide_state::State {
        let files = self
            .tabs
            .iter()
            .filter_map(|t| {
                let path = t.editor.path()?;
                let (line, column) = t.editor.cursor();
                Some(ide_state::FileState { path: ide_state::shown_path(&self.root, path), modified: t.editor.is_modified(), line, column })
            })
            .collect();
        let active = self.active_path().map(|p| ide_state::shown_path(&self.root, p));
        let cursor = active.as_ref().map(|_| self.editor().cursor());
        ide_state::State {
            format: ide_state::FORMAT.into(),
            version: 1,
            pid: std::process::id(),
            active,
            line: cursor.map(|c| c.0),
            column: cursor.map(|c| c.1),
            files,
        }
    }

    /// Grava o `state.json` se mudou, no máximo a cada `MIN_GAP` (o resto espera o `IdeStateTick`).
    fn publish_ide_state(&mut self) {
        let now = self.ide_state_snapshot();
        if self.ide_state.as_ref() == Some(&now) {
            self.ide_state_due = false;
            return;
        }
        if self.ide_state_at.is_some_and(|at| at.elapsed() < ide_state::MIN_GAP) {
            self.ide_state_due = true;
            return;
        }
        ide_state::write(&self.root, &now);
        self.ide_state = Some(now);
        self.ide_state_at = Some(Instant::now());
        self.ide_state_due = false;
    }

    fn handle(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(id, msg) => {
                let Some(i) = self.tab_index(id) else { return Task::none() };
                if msg.is_save_request() {
                    return if i == self.active { self.save_active() } else { Task::none() };
                }
                // Ctrl+clique sem servidor: o editor não tem a quem perguntar, então avisamos aqui.
                if msg.is_jump_click() && self.tabs[i].editor.lsp_server().is_none() {
                    let task = self.tabs[i].editor.update(msg).map(move |m| Message::Editor(id, m));
                    return Task::batch([task, self.no_lsp_note()]);
                }
                let hover = msg.is_mouse_hover();
                if msg.is_typing() {
                    self.last_typed = Some(Instant::now());
                }
                let toggled = msg.is_breakpoint_toggle();
                let press = msg.is_press();
                let revision = self.tabs[i].editor.text_revision();
                let mut task = self.tabs[i].editor.update(msg).map(move |m| Message::Editor(id, m));
                // O servidor desta aba caiu: religa quando a digitação pausar (talvez já com o que o derrubou consertado).
                if self.tabs[i].editor.text_revision() != revision
                    && self.tabs[i].editor.lsp_server().is_none()
                    && self.tabs[i].editor.path().is_some_and(|p| self.lsp.awaits_restart(p))
                {
                    task = Task::batch([task, self.relink_later()]);
                }
                if toggled {
                    self.sync_breakpoints(i);
                }
                if i != self.active {
                    // Clique no editor do outro lado: o foco vai para lá.
                    if press && self.other_tab == Some(id) {
                        return Task::batch([task, self.set_active(i)]);
                    }
                    return task;
                }
                // O texto mudou: a estrutura é pedida de novo quando parar de digitar.
                let seen = (id, self.tabs[i].editor.text_revision());
                let outline = if seen != self.outline_seen {
                    self.outline_seen = seen;
                    self.outline_later()
                } else {
                    Task::none()
                };
                let text = if !hover && self.watching_active_text() { self.text_later() } else { Task::none() };
                let diff = if !hover && self.tabs[i].editor.has_diff_base() { self.diff_later() } else { Task::none() };
                Task::batch([task, self.emit_status(false), outline, text, diff])
            }
            // O foco do editor sai por clique no painel (`MousePressed`) ou por comando,
            // não aqui: painéis também mandam mensagens sozinhos (ex.: terminal ao mudar de tamanho).
            Message::Ext(i, msg) | Message::ExtSub(i, msg) => self.call_ext(i, |e, ctx| e.update(msg, ctx)),
            Message::BreadcrumbReveal(id) => {
                let Some(i) = self.tab_index(id) else { return Task::none() };
                let activate = self.set_active(i);
                Task::batch([activate, self.run_command("filetree.reveal")])
            }
            Message::BreadcrumbSymbol(path, line, column) => self.open_file_at(path, line, column),
            Message::SelectTab(id) => match self.tab_index(id) {
                Some(i) => self.set_active(i),
                None => Task::none(),
            },
            Message::CloseTab(id) => self.close_tab(id, false),
            Message::RunCommand(id) => {
                let task = self.run_command(&id);
                self.editor_mut().focus();
                task
            }
            Message::TabMenu(id, at) => {
                let menu = self.tab_menu(id, at);
                self.open_menu(menu, dialog::Owner::Core)
            }
            Message::SelectBottom(i) => {
                self.active_bottom = Some(i);
                Task::none()
            }
            Message::Key { key, modified, modifiers, captured } => self.on_key(key, &modified, modifiers, captured),
            Message::Palette(msg) => self.palette_update(msg),
            Message::Disk(watcher::Change::Paths(paths)) => self.disk_changed(&paths),
            Message::Dap(generation, event) => self.on_dap(generation, event),
            Message::DebugBuilt(outcome) => self.debug_built(&outcome),
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
                    Edge::Split => self.ui_state.split = d.split,
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
            Message::BackupTick => {
                self.backup_tick();
                Task::none()
            }
            // `update` grava o estado depois de cada mensagem.
            Message::IdeStateTick => Task::none(),
            Message::OpenRequest(request) => {
                let open = self.open_file_at(request.path, request.line, request.column);
                // Traz a janela para a frente (o compositor pode só marcar a janela, como no Wayland).
                Task::batch([open, window::latest().and_then(window::gain_focus)])
            }
            Message::MediaTick => {
                let changed: Vec<PathBuf> = self
                    .tabs
                    .iter()
                    .filter(|t| t.editor.is_image() && t.editor.file_changed_on_disk())
                    .filter_map(|t| t.editor.path().map(Path::to_path_buf))
                    .collect();
                Task::batch(changed.iter().map(|p| self.reload_from_disk(p)).collect::<Vec<_>>())
            }
            Message::BinaryChanged => {
                self.binary_changed = true;
                let note = self.notify(fl!("binary-changed"));
                Task::batch([self.broadcast(CoreEvent::BinaryChanged), note])
            }
            Message::MousePressed => {
                if self.pointer_over_panel {
                    self.editor_mut().lose_focus();
                }
                Task::none()
            }
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            Message::WheelScrolled(y) => {
                if self.modifiers.control()
                    && !self.pointer_over_panel
                    && let Some(delta) = wheel_zoom(y)
                {
                    return self.run_command(if delta > 0.0 { "editor.zoom_in" } else { "editor.zoom_out" });
                }
                Task::none()
            }
            Message::CloseRequested => {
                self.restart = false;
                self.request_quit()
            }
            Message::Nav(forward) => self.navigate(forward),
            Message::SymbolQueryDue(wait) => {
                if wait == self.symbol_wait { self.search_project_symbols() } else { Task::none() }
            }
            Message::TextDue(wait) => {
                if wait == self.text_wait { self.send_active_text() } else { Task::none() }
            }
            Message::RelinkDue(wait) => {
                if wait != self.relink_wait {
                    return Task::none();
                }
                let relink: Vec<Task<Message>> = (0..self.tabs.len()).map(|t| self.ensure_lsp(t)).collect();
                Task::batch(relink)
            }
            Message::DiffDue(wait) => {
                if wait == self.diff_wait {
                    self.editor_mut().refresh_diff();
                }
                Task::none()
            }
            Message::FormatSaveDue(wait) => {
                if wait == self.format_save_wait && self.format_save.is_some() {
                    let note = self.notify(fl!("format-save-timeout"));
                    Task::batch([self.finish_format_save(), note])
                } else {
                    Task::none()
                }
            }
            Message::OutlineDue(wait) => {
                if wait == self.outline_wait { self.request_outline() } else { Task::none() }
            }
            Message::Lsp(event) => self.handle_lsp(event),
            Message::Mascot(msg) => match msg {
                mascot::Msg::Tick => {
                    self.mascot.tick();
                    Task::none()
                }
                mascot::Msg::Snapshot(snap) => {
                    self.mascot.apply_snapshot((*snap).clone());
                    Task::none()
                }
                mascot::Msg::ToggleCollapsed => {
                    self.mascot.collapsed = !self.mascot.collapsed;
                    self.ui_state.mascot_collapsed = self.mascot.collapsed;
                    self.ui_state.save(self.state_path.as_deref());
                    Task::none()
                }
                mascot::Msg::ActivateTarget => {
                    let (_, _, target) = self.mascot.resolved();
                    match target.command() {
                        Some(cmd) => self.run_command(cmd),
                        None => Task::none(),
                    }
                }
                mascot::Msg::PointerEntered => {
                    self.mascot.on_pointer_entered();
                    Task::none()
                }
                mascot::Msg::PointerMoved(pt) => {
                    self.mascot.on_pointer_moved(pt);
                    Task::none()
                }
                mascot::Msg::PointerExited => {
                    self.mascot.on_pointer_exited();
                    Task::none()
                }
                mascot::Msg::Poke => {
                    self.mascot.on_poke();
                    Task::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // `listen_with` vê também as teclas que um widget consumiu (`captured`).
        let input = iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modified_key, modifiers, .. }) => Some(Message::Key {
                key,
                modified: modified_key,
                modifiers,
                captured: status == iced::event::Status::Captured,
            }),
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::ModifiersChanged(modifiers))
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back)) => Some(Message::Nav(false)),
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) => Some(Message::Nav(true)),
            iced::Event::Mouse(mouse::Event::ButtonPressed(_)) => Some(Message::MousePressed),
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                Some(Message::WheelScrolled(y))
            }
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
        // Cópias só enquanto há alterações (ou cópias para apagar).
        let backups = if self.backup_dir.is_some()
            && (!self.backed_up.is_empty() || self.tabs.iter().any(|t| t.editor.is_modified()))
        {
            Subscription::run(backup_ticks)
        } else {
            Subscription::none()
        };
        let binary = match &self.binary {
            Some(stamp) if !self.binary_changed => {
                Subscription::run_with(stamp.clone(), binary::watch).map(|()| Message::BinaryChanged)
            }
            _ => Subscription::none(),
        };
        // Abas de imagem e PDF: além do observador, conferem tamanho e data a cada segundo (um arquivo regerado
        // no lugar às vezes não recarregava a aba; assim não depende só dos eventos).
        let media = if self.tabs.iter().any(|t| t.editor.is_image()) {
            Subscription::run(media_ticks)
        } else {
            Subscription::none()
        };
        let state = if self.ide_state_due { Subscription::run(ide_state_ticks) } else { Subscription::none() };
        let open = Subscription::run_with(self.root.clone(), ide_state::watch_open).map(Message::OpenRequest);
        let mascot_watch = if self.config.mascot.enabled {
            Subscription::run_with((self.root.clone(), self.config.mascot.pack.clone()), mascot::watch)
                .map(|s| Message::Mascot(mascot::Msg::Snapshot(s)))
        } else {
            Subscription::none()
        };
        let mascot_anim = match self.mascot.frame_interval(self.config.mascot.enabled) {
            Some(d) => Subscription::run_with(d.as_millis() as u64, mascot_ticks),
            None => Subscription::none(),
        };
        Subscription::batch([
            input,
            close,
            frames,
            extensions,
            resized,
            disk,
            backups,
            binary,
            media,
            state,
            open,
            mascot_watch,
            mascot_anim,
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let mut layers = stack![self.workspace_view()];
        if let Some(m) = self.mascot.overlay_view(self.config.mascot.enabled) {
            layers = layers.push(m.map(Message::Mascot));
        }
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

    /// Um lado do editor (o único, sem divisão): as abas dele e o editor da aba mostrada.
    fn side_view(&self, right: bool) -> Element<'_, Message> {
        let focused = self.tabs[self.active].right == right;
        let shown = if focused { Some(self.active) } else { self.other_index() };
        let tabs = tea_core::style::scrollable(
            row(self.tabs.iter().enumerate().filter(|(_, t)| t.right == right).map(|(i, t)| {
                let style = if i == self.active {
                    button::primary
                } else if Some(i) == shown {
                    button::secondary
                } else {
                    button::text
                };
                let close =
                    button(text("×").size(13)).on_press(Message::CloseTab(t.id)).style(button::text).padding([0, 4]);
                let id = t.id;
                let title = t.editor.title();
                // Na aba ativa o fundo é da cor de destaque: o ícone fica na cor do texto.
                let kind = tea_core::file_icons::icon(&t.editor.file_name(), 13.0, i != self.active);
                let tab = button(row![kind, text(title).size(13), close].spacing(6).align_y(iced::Center))
                    .on_press(Message::SelectTab(id))
                    .style(style)
                    .padding([3, 8]);
                // Botão do meio fecha; botão direito abre o menu.
                let tab = mouse_area(tab).on_middle_press(Message::CloseTab(id));
                tea_core::menu::context_area(tab, move |at| Message::TabMenu(id, at))
            }))
            .spacing(2),
        )
        .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new().width(2).scroller_width(2)))
        .width(Fill);
        let mut tabs = row![tabs].spacing(4).align_y(iced::Center);
        if focused {
            tabs = tabs.extend(self.file_action_buttons());
        }
        let editor: Element<'_, Message> = match shown {
            Some(i) => {
                let tab_id = self.tabs[i].id;
                self.tabs[i].editor.view().map(move |m| Message::Editor(tab_id, m))
            }
            None => iced::widget::space().into(),
        };
        let mut side = column![tabs].spacing(4).width(Fill).height(Fill);
        if self.config.editor.breadcrumbs
            && let Some(crumbs) = shown.and_then(|i| self.breadcrumbs_view(i))
        {
            side = side.push(crumbs);
        }
        side.push(editor).into()
    }

    /// Breadcrumbs da aba `i`: a pasta do projeto, as pastas, o arquivo e, na aba ativa, os símbolos onde o cursor
    /// está (da última estrutura). Pasta: mostra na árvore; símbolo: vai até ele.
    fn breadcrumbs_view(&self, i: usize) -> Option<Element<'_, Message>> {
        let tab = &self.tabs[i];
        let path = tab.editor.path()?;
        let id = tab.id;
        let mut parts: Vec<Element<'_, Message>> = Vec::new();
        let sep = || -> Element<'_, Message> { text("›").size(12).style(text::secondary).into() };
        let crumb = |label: String, on: Message| -> Element<'_, Message> {
            button(text(label).size(12).style(text::secondary)).style(button::text).padding([0, 2]).on_press(on).into()
        };
        let (names, file) = crumb_names(&self.root, path);
        for name in names {
            parts.push(crumb(name, Message::BreadcrumbReveal(id)));
            parts.push(sep());
        }
        parts.push(
            row![tea_core::file_icons::icon(&file, 12.0, true), text(file).size(12)]
                .spacing(3)
                .align_y(iced::Center)
                .into(),
        );
        // Símbolos: só da aba ativa, e só se a estrutura guardada é deste arquivo.
        if i == self.active && self.last_outline.0.as_deref() == Some(path) {
            let (line, _) = tab.editor.cursor();
            for s in symbol_chain(&self.last_outline.1, line) {
                let (glyph, color) = s.kind.icon();
                let label = row![
                    tea_core::icons::icon(glyph).size(12).color_maybe(color),
                    text(s.name.clone()).size(12).style(text::secondary)
                ]
                .spacing(3)
                .align_y(iced::Center);
                parts.push(sep());
                parts.push(
                    button(label)
                        .style(button::text)
                        .padding([0, 2])
                        .on_press(Message::BreadcrumbSymbol(path.to_path_buf(), s.line, s.column))
                        .into(),
                );
            }
        }
        // Um respiro à esquerda: colado na borda do painel, o primeiro caractere fica cortado.
        let line = row(parts).spacing(2).align_y(iced::Center).padding(iced::Padding { left: 6.0, right: 6.0, ..iced::Padding::ZERO });
        Some(
            tea_core::style::scrollable(line)
                .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::new().width(2).scroller_width(2)))
                .width(Fill)
                .into(),
        )
    }

    fn workspace_view(&self) -> Element<'_, Message> {
        use tea_core::style::{GAP, ISLAND_INSET};
        // RFC 0015: cada área numa ilha arredondada sobre um fundo mais escuro; o vão entre elas é o divisor.
        let islands = self.config.ui.style != "classic";
        fn island(islands: bool, content: Element<'_, Message>) -> Element<'_, Message> {
            if islands {
                container(content)
                    .padding(ISLAND_INSET)
                    .width(Fill)
                    .height(Fill)
                    .style(tea_core::style::island)
                    .into()
            } else {
                content
            }
        }
        let island = |content| island(islands, content);
        let divider = |axis, edge| {
            divider::divider(axis, move |d| Message::Resize(edge, d), Message::ResizeDone, Message::ResizeReset(edge))
                .islands(islands)
        };

        let editors: Element<'_, Message> = if self.other_index().is_some() {
            let left = (self.ui_state.split * 1000.0).round() as u16;
            row![
                container(island(self.side_view(false))).width(iced::FillPortion(left)),
                divider(divider::Axis::Vertical, Edge::Split),
                container(island(self.side_view(true))).width(iced::FillPortion(1000 - left)),
            ]
            .height(Fill)
            .into()
        } else {
            island(self.side_view(false))
        };

        let mut center = column![editors].spacing(if islands { 0 } else { 4 }).width(Fill).height(Fill);
        if let Some(bottom) = self.bottom_view() {
            center = center
                .push(divider(divider::Axis::Horizontal, Edge::Bottom))
                .push(container(island(bottom)).height(self.ui_state.bottom_height));
        }

        let mut body = row![].height(Fill);
        if self.config.ui.activity_bar {
            body = body.push(self.activity_bar());
            body = if islands { body.push(iced::widget::space().width(GAP)) } else { body.push(rule::vertical(1)) };
        }
        if let Some(left) = self.slot_view(Slot::Left) {
            body = body
                .push(container(island(left)).width(self.ui_state.left_width).height(Fill))
                .push(divider(divider::Axis::Vertical, Edge::Left));
        }
        body = body.push(center);
        if let Some(right) = self.slot_view(Slot::Right) {
            body = body
                .push(divider(divider::Axis::Vertical, Edge::Right))
                .push(container(island(right)).width(self.ui_state.right_width).height(Fill));
        }

        let mut page = column![body].spacing(if islands { GAP } else { 4.0 }).padding(if islands { GAP } else { 6.0 });
        // Nas ilhas a barra de status fica no fundo da janela, sem linha: o contraste já separa.
        let status_rule = || -> Element<'_, Message> {
            if islands { iced::widget::space().height(0).into() } else { rule::horizontal(1).into() }
        };
        let status_view = self.slot_view(Slot::Status);
        let mascot_btn = self.mascot.status_bar_button(self.config.mascot.enabled).map(|b| b.map(Message::Mascot));
        match (status_view, mascot_btn) {
            (Some(status), Some(btn)) => {
                page = page.push(status_rule()).push(row![status, btn].spacing(12).align_y(iced::Center));
            }
            (Some(status), None) => {
                page = page.push(status_rule()).push(status);
            }
            (None, Some(btn)) => {
                page = page.push(status_rule()).push(row![iced::widget::space::horizontal(), btn].align_y(iced::Center));
            }
            (None, None) => {}
        }
        if islands {
            container(page).width(Fill).height(Fill).style(tea_core::style::canvas_style).into()
        } else {
            page.into()
        }
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
    /// Botões das extensões para o tipo do arquivo ativo (ex.: prévia de Markdown), no fim da barra de abas.
    fn file_action_buttons(&self) -> Vec<Element<'_, Message>> {
        let ext = self.active_path().and_then(|p| p.extension()).map(|e| e.to_string_lossy().to_lowercase());
        let Some(ext) = ext else { return Vec::new() };
        self.file_actions
            .iter()
            .filter(|(_, a)| a.extensions.contains(&ext))
            .map(|(owner, a)| {
                let on = a.panel.is_some_and(|id| self.panels.iter().any(|p| p.ext == *owner && p.id == id && p.visible));
                let tip = match self.keymap.chord_for(&a.command) {
                    Some(chord) => format!("{} ({chord})", a.tip),
                    None => a.tip.clone(),
                };
                tea_core::icons::toggle_button(a.icon, tip, on, Message::RunCommand(a.command.clone()))
            })
            .collect()
    }

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

/// Breadcrumbs: as pastas (a do projeto primeiro) e o nome do arquivo. Fora do projeto, o caminho todo.
fn crumb_names(root: &Path, path: &Path) -> (Vec<String>, String) {
    let name = |c: std::path::Component<'_>| c.as_os_str().to_string_lossy().into_owned();
    let file = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    match path.strip_prefix(root) {
        Ok(rel) => {
            let project = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let mut names = vec![project];
            names.extend(rel.parent().into_iter().flat_map(Path::components).map(name));
            (names, file)
        }
        Err(_) => {
            let names = path
                .parent()
                .into_iter()
                .flat_map(Path::components)
                .filter(|c| !matches!(c, std::path::Component::RootDir))
                .map(name)
                .collect();
            (names, file)
        }
    }
}

/// Os símbolos onde a linha está, do mais de fora para o mais de dentro (`impl App` › `fn update`).
fn symbol_chain(symbols: &[tea_core::outline::Symbol], line: usize) -> Vec<&tea_core::outline::Symbol> {
    let mut out = Vec::new();
    let mut level = symbols;
    while let Some(s) = level.iter().find(|s| s.start_line <= line && line <= s.end_line) {
        out.push(s);
        level = &s.children;
    }
    out
}

/// "dark", "light" ou o nome de um tema do iced (sem diferenciar maiúsculas).
fn resolve_theme(name: &str) -> (Theme, Option<String>) {
    match name.trim().to_lowercase().as_str() {
        "dark" | "escuro" => (DARK_THEME, None),
        "light" | "claro" => (LIGHT_THEME, None),
        other => match Theme::ALL.iter().find(|t| t.to_string().to_lowercase() == other) {
            Some(t) => (t.clone(), None),
            None => (DARK_THEME, Some(fl!("theme-unknown", name = name.to_string()))),
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
        (fl!("default"), config::DEFAULT_CONFIG.to_string()),
        ("global".to_string(), global.to_string()),
        ("projeto".to_string(), project.to_string()),
    ];
    let merged = Config::from_layers(&layers);
    let (keymap, _) = Keymap::from_pairs(merged.keys.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    Chord::parse(chord).ok().and_then(|c| keymap.command_for(&c).map(str::to_string)).as_deref() == Some(command)
}

fn expand_home(input: &str) -> PathBuf {
    match input.strip_prefix("~/").or_else(|| input.strip_prefix("~\\")) {
        Some(rest) => tea_core::os::home_dir().map(|h| h.join(rest)).unwrap_or_else(|| input.into()),
        None => PathBuf::from(input),
    }
}

fn display_rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn breadcrumbs_path_and_symbols() {
        use tea_core::outline::{Kind, Symbol, Visibility};
        let (names, file) = crumb_names(Path::new("/p/idetea"), Path::new("/p/idetea/crates/tea-app/src/main.rs"));
        assert_eq!(names, ["idetea", "crates", "tea-app", "src"]);
        assert_eq!(file, "main.rs");
        let (names, file) = crumb_names(Path::new("/p/idetea"), Path::new("/p/idetea/README.md"));
        assert_eq!((names, file.as_str()), (vec!["idetea".to_string()], "README.md"));
        let (names, _) = crumb_names(Path::new("/p/idetea"), Path::new("/etc/hosts"));
        assert_eq!(names, ["etc"], "fora do projeto: o caminho todo, sem a raiz /");

        let sym = |name: &str, start: usize, end: usize, children: Vec<Symbol>| Symbol {
            name: name.into(),
            kind: Kind::Function,
            detail: None,
            visibility: Visibility::Unknown,
            line: start,
            column: 1,
            start_line: start,
            end_line: end,
            children,
        };
        let symbols = vec![sym("A", 1, 3, vec![]), sym("impl App", 5, 40, vec![sym("new", 6, 10, vec![]), sym("update", 12, 39, vec![])])];
        let names = |line| symbol_chain(&symbols, line).iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(20), ["impl App", "update"]);
        assert_eq!(names(5), ["impl App"]);
        assert_eq!(names(2), ["A"]);
        assert!(names(4).is_empty());
    }

    #[test]
    fn translations_are_complete() {
        // Todos os crates com textos: inglês e espanhol têm as mesmas mensagens e valores que o português.
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(workspace.join("crates")).unwrap().flatten().map(|e| e.path()).collect();
        dirs.extend(std::fs::read_dir(workspace.join("crates/ext")).unwrap().flatten().map(|e| e.path()));
        let mut checked = 0;
        let mut problems = Vec::new();
        for dir in dirs.iter().map(|d| d.join("i18n")).filter(|d| d.is_dir()) {
            checked += 1;
            problems.extend(tea_core::i18n::check_translations(&dir));
        }
        assert!(checked >= 1, "nenhuma pasta i18n achada");
        assert!(problems.is_empty(), "traduções:\n{}", problems.join("\n"));
    }

    #[test]
    fn texts_load_in_each_language() {
        use tea_core::i18n::{Language, load_in};
        let en = load_in("tea-app", &Localizations, Language::English);
        assert_eq!(en.get("settings-theme"), "Theme");
        let es = load_in("tea-app", &Localizations, Language::Spanish);
        let args = std::collections::HashMap::from([("language", "English".into())]);
        // Sem as marcas Unicode de isolamento em volta do valor.
        assert_eq!(es.get_args_concrete("settings-language-auto", args), "Automático (English)");
        // Testes sem `init`: português, os textos de hoje.
        assert_eq!(fl!("settings-theme-detail"), "Cores da IDE, do editor e do terminal.");
    }

    #[test]
    fn command_titles_exist() {
        // Os títulos são buscados pelo id em tempo de execução (o fl! não confere esses).
        let missing: Vec<_> = CORE_COMMANDS.iter().filter(|(_, t)| !LOADER.has(t)).map(|(_, t)| *t).collect();
        assert!(missing.is_empty(), "títulos sem texto: {missing:?}");
        assert_eq!(LOADER.get("cmd-file-save"), "Arquivo: salvar");
    }

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
    fn outline_becomes_a_flat_list_with_parents() {
        use tea_core::outline::{Kind, Symbol, Visibility};
        let s = |name: &str, line, children| Symbol {
            name: name.into(),
            kind: Kind::Method,
            detail: None,
            visibility: Visibility::Public,
            line,
            column: 1,
            start_line: line,
            end_line: line,
            children,
        };
        let tree = [s("Conta", 1, vec![s("impl Conta", 2, vec![s("depositar", 3, vec![])])]), s("main", 9, vec![])];
        let items = symbol_items(Path::new("/p/a.rs"), &tree);
        let got: Vec<_> = items.iter().map(|i| (i.name.as_str(), i.container.as_str(), i.line)).collect();
        assert_eq!(got, [("Conta", "", 1), ("impl Conta", "Conta", 2), ("depositar", "Conta › impl Conta", 3), ("main", "", 9)]);
        assert_eq!(palette_mode_of("nav.project_symbol"), Some(palette::Mode::ProjectSymbols));
        assert_eq!(palette_mode_of("file.save"), None);
    }

    #[test]
    fn word_under_cursor() {
        assert_eq!(word_at("    let x = soma(1, 2);", 13), "soma");
        assert_eq!(word_at("    let x = soma(1, 2);", 16), "soma", "logo depois do nome");
        assert_eq!(word_at("ação_1 + b", 2), "ação_1");
        assert_eq!(word_at("", 5), "");
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
    fn editor_zoom_changes_size_with_limits_and_reset() {
        let mut zoom = EditorZoom::new(14.0);
        zoom.adjust(1.0);
        assert_eq!(zoom.size(), 15.0);
        zoom.adjust(-2.0);
        assert_eq!(zoom.size(), 13.0);
        for _ in 0..30 {
            zoom.adjust(1.0);
        }
        assert_eq!(zoom.size(), settings::FONT_MAX);
        for _ in 0..40 {
            zoom.adjust(-1.0);
        }
        assert_eq!(zoom.size(), settings::FONT_MIN);
        zoom.reset();
        assert_eq!(zoom.size(), 14.0);
    }

    #[test]
    fn editor_zoom_only_resets_when_the_configured_base_changes() {
        let mut zoom = EditorZoom::new(14.0);
        zoom.adjust(3.0);
        zoom.reload_base(14.0);
        assert_eq!(zoom.size(), 17.0);
        zoom.reload_base(16.0);
        assert_eq!(zoom.size(), 16.0);
    }

    #[test]
    fn ctrl_wheel_zooms_up_or_down() {
        assert_eq!(wheel_zoom(1.0), Some(1.0));
        assert_eq!(wheel_zoom(-0.5), Some(-1.0));
        assert_eq!(wheel_zoom(0.0), None);
    }

    #[test]
    fn default_keys_include_editor_zoom() {
        use iced::keyboard::{Key, Modifiers};
        let cfg = Config::from_layers(&[("padrão".into(), config::DEFAULT_CONFIG.into())]);
        let (keymap, errors) = Keymap::from_pairs(cfg.keys.iter().map(|(k, c)| (k.as_str(), c.as_str())));
        assert!(errors.is_empty(), "{errors:?}");
        let command = |key: &str, modifiers| {
            let key = Key::Character(key.into());
            keymap.command_for_event(&key, &key, modifiers)
        };
        assert_eq!(command("=", Modifiers::CTRL), Some("editor.zoom_in"));
        assert_eq!(command("-", Modifiers::CTRL), Some("editor.zoom_out"));
        assert_eq!(command("0", Modifiers::CTRL), Some("editor.zoom_reset"));

        let plus = Key::Character("+".into());
        assert_eq!(keymap.command_for_event(&plus, &plus, Modifiers::CTRL), Some("editor.zoom_in"));
        let equals = Key::Character("=".into());
        assert_eq!(keymap.command_for_event(&equals, &plus, Modifiers::CTRL | Modifiers::SHIFT), Some("editor.zoom_in"));
    }

    #[test]
    fn default_keys_point_to_existing_commands() {
        let mut ids: Vec<_> = CORE_COMMANDS.iter().map(|(id, _)| *id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CORE_COMMANDS.len(), "ids repetidos");

        let ext_commands = ["filetree.toggle", "filetree.reveal", "search.focus", "search.replace", "problems.focus", "problems.next", "terminal.focus", "git.focus", "outline.toggle", "preview.toggle", "tasks.run", "tasks.rerun"];
        let cfg = Config::from_layers(&[("padrão".into(), config::DEFAULT_CONFIG.into())]);
        for (chord, cmd) in &cfg.keys {
            assert!(ids.contains(&cmd.as_str()) || ext_commands.contains(&cmd.as_str()), "{chord} -> {cmd}");
        }
    }

    #[test]
    fn default_keys_follow_vs_code_where_they_clashed() {
        use iced::keyboard::{Key, Modifiers};
        let cfg = Config::from_layers(&[("padrão".into(), config::DEFAULT_CONFIG.into())]);
        let (keymap, errors) = Keymap::from_pairs(cfg.keys.iter().map(|(k, c)| (k.as_str(), c.as_str())));
        assert!(errors.is_empty(), "{errors:?}");
        let ctrl = Modifiers::CTRL;
        let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;
        let command = |key: &str, modifiers| {
            let key = Key::Character(key.into());
            keymap.command_for_event(&key, &key, modifiers).map(str::to_string)
        };
        assert_eq!(command("[", ctrl_shift).as_deref(), Some("editor.toggle_fold"));
        assert_eq!(command("]", ctrl_shift).as_deref(), Some("editor.toggle_fold"));
        assert_eq!(command("t", ctrl_shift).as_deref(), Some("tab.reopen"));
        assert_eq!(command("'", ctrl).as_deref(), Some("terminal.focus"));
        assert_eq!(command("`", ctrl).as_deref(), Some("terminal.focus"));
        assert_eq!(command("j", ctrl).as_deref(), Some("view.toggle_bottom"));
        assert_eq!(command("k", ctrl_shift), None);
        assert_eq!(command(".", ctrl), None);
    }
}
