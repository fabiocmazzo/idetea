//! Busca no projeto: painel inferior com busca enquanto digita, respeitando `.gitignore`.
//!
//! A busca roda numa thread à parte. Cada nova busca cancela a anterior. Escopo (pasta ou arquivo),
//! globs e linhas de contexto vão juntos para o motor; substituir usa os mesmos resultados.
//!
//! Substituir: com a linha "Substituir por" aberta, cada resultado mostra como fica. A troca em si é
//! do app ([`tea_core::replace`]): abas abertas mudam na memória, os outros arquivos são gravados.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

pub mod engine;
mod keys;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced::futures::channel::oneshot;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{Id, button, column, container, operation, rich_text, row, space, span, text, text_input};
use iced::{Element, Fill, Font, Task, Theme};
use regex::Regex;
use tea_core::icons::{Icon, toggle_button, with_tip};
use tea_core::replace::{Replace, Target};
use tea_core::{ButtonKind, Context, CoreEvent, Dialog, Extension, PanelId, Slot};

use engine::{Options, Results};

const PANEL: PanelId = PanelId("search");
/// Espera um pouco antes de buscar, para não buscar a cada tecla.
const DEBOUNCE: Duration = Duration::from_millis(150);
const MIN_QUERY_CHARS: usize = 2;
const HISTORY_LIMIT: usize = 20;
const ROW_HEADER: f32 = 28.0;
const ROW_LINE: f32 = 20.0;

fn input_id() -> Id {
    Id::new("search-input")
}

fn replace_id() -> Id {
    Id::new("search-replace")
}

fn results_id() -> Id {
    Id::new("search-results")
}

fn include_id() -> Id {
    Id::new("search-include")
}

fn exclude_id() -> Id {
    Id::new("search-exclude")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKey {
    Up,
    Down,
    Tab,
    BackTab,
    Enter,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// Trecho que combinou.
    Hit,
    /// Trecho antigo na prévia da troca.
    Old,
    /// Texto novo na prévia da troca.
    New,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Sel {
    file: usize,
    /// `None` = o cabeçalho do arquivo.
    line: Option<usize>,
}

#[derive(Debug, Clone)]
struct Preview {
    text: String,
    spans: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    IncludeChanged(String),
    ExcludeChanged(String),
    Submit,
    QueryKey(QueryKey),
    ToggleCase,
    ToggleRegex,
    ToggleWord,
    ToggleFilters,
    ToggleContext,
    ToggleHistory,
    PickHistory(usize),
    ClearScope,
    ToggleFile(PathBuf),
    Open { file: usize, index: usize, path: PathBuf, line: usize, column: usize },
    Done(u64, Result<Arc<Results>, String>),
    ToggleReplace,
    ReplacementChanged(String),
    ReplaceLine { path: PathBuf, line: usize },
    ReplaceFile(PathBuf),
    /// Pede confirmação.
    ReplaceAll,
    ReplaceAllConfirmed,
}

pub struct SearchExtension {
    root: PathBuf,
    query: String,
    include: String,
    exclude: String,
    /// Pasta ou arquivo em que a busca está limitada. `None` = o projeto inteiro.
    scope: Option<PathBuf>,
    filters_open: bool,
    context_on: bool,
    /// Valor de `[search] context_lines`. O botão liga o contexto; 0 no config vira 1 linha.
    context_configured: usize,
    opts: Options,
    results: Option<Arc<Results>>,
    error: Option<String>,
    generation: u64,
    cancel: Option<Arc<AtomicBool>>,
    collapsed: HashSet<PathBuf>,
    selected: Option<Sel>,
    /// Setas esquerda/direita fecham grupos em vez de mover o cursor do campo.
    nav_keys: bool,
    history: Vec<String>,
    history_browse: Option<usize>,
    history_open: bool,
    replace_open: bool,
    replacement: String,
    /// Regex dos resultados mostrados (a da busca que terminou) e se `$1` vale na troca.
    searched: Option<(Regex, bool)>,
    /// A busca que está rodando, para virar `searched` quando terminar.
    pending_search: Option<(Regex, bool)>,
    /// Como fica cada linha dos resultados, na mesma ordem.
    previews: Vec<Vec<Preview>>,
    /// Pedidos de troca mandados ao app e ainda sem resposta.
    sent: Vec<Target>,
    /// Pôr o foco em "Substituir por" quando o painel aparecer.
    focus_replace: bool,
    theme: Theme,
}

impl Default for SearchExtension {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            query: String::new(),
            include: String::new(),
            exclude: String::new(),
            scope: None,
            filters_open: false,
            context_on: false,
            context_configured: 0,
            opts: Options::default(),
            results: None,
            error: None,
            generation: 0,
            cancel: None,
            collapsed: HashSet::new(),
            selected: None,
            nav_keys: false,
            history: Vec::new(),
            history_browse: None,
            history_open: false,
            replace_open: false,
            replacement: String::new(),
            searched: None,
            pending_search: None,
            previews: Vec::new(),
            sent: Vec::new(),
            focus_replace: false,
            theme: Theme::Dark,
        }
    }
}

impl SearchExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn running(&self) -> bool {
        self.cancel.is_some()
    }

    fn current_scope(&self) -> engine::Scope {
        engine::Scope { path: self.scope.clone(), include: self.include.clone(), exclude: self.exclude.clone() }
    }

    fn current_opts(&self) -> Options {
        Options {
            context_lines: if self.context_on { self.context_configured.max(1) } else { 0 },
            ..self.opts.clone()
        }
    }

    /// Cancela a busca em andamento e começa outra (se a busca tiver texto suficiente).
    fn restart(&mut self) -> Task<Message> {
        if let Some(c) = self.cancel.take() {
            c.store(true, Ordering::Relaxed);
        }
        self.generation += 1;
        self.error = None;
        if self.query.chars().count() < MIN_QUERY_CHARS {
            self.results = None;
            self.selected = None;
            self.previews.clear();
            self.searched = None;
            return Task::none();
        }
        let opts = self.current_opts();
        let matcher = match engine::build_matcher(&self.query, &opts) {
            Ok(m) => m,
            Err(e) => {
                self.error = Some(e);
                self.results = None;
                self.selected = None;
                return Task::none();
            }
        };
        self.pending_search = Some((matcher.clone(), opts.regex));
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        let generation = self.generation;
        let root = self.root.clone();
        let scope = self.current_scope();
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(DEBOUNCE);
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let _ = tx.send(engine::search(&root, &matcher, &scope, &opts, &cancel).map(Arc::new));
        });
        // Se a busca foi cancelada, o canal fecha e nada chega.
        Task::future(async move { rx.await.ok() }).and_then(move |r| Task::done(Message::Done(generation, r)))
    }

    /// Dá para substituir: os resultados na tela são da busca que terminou, e são do texto que está no campo.
    fn can_replace(&self) -> bool {
        !self.running() && self.error.is_none() && self.searched.is_some() && self.results.as_ref().is_some_and(|r| r.total > 0)
    }

    /// Recalcula a prévia de cada linha (ao chegar resultado ou mudar o texto da troca).
    fn refresh_previews(&mut self) {
        self.previews.clear();
        let (Some(results), Some((re, expand)), true) = (&self.results, &self.searched, self.replace_open) else { return };
        self.previews = results
            .files
            .iter()
            .map(|f| {
                f.matches
                    .iter()
                    .map(|m| {
                        let (text, spans) = engine::preview_replacement(&m.text, re, &self.replacement, *expand);
                        Preview { text, spans }
                    })
                    .collect()
            })
            .collect();
    }

    /// Manda a troca para o app (a resposta chega em [`CoreEvent::Replaced`]).
    fn send(&mut self, targets: Vec<Target>, ctx: &mut Context) {
        let Some((re, expand)) = &self.searched else { return };
        if !self.can_replace() || targets.is_empty() {
            return;
        }
        self.sent.extend(targets.iter().cloned());
        ctx.replace(Replace {
            pattern: re.as_str().to_string(),
            replacement: self.replacement.clone(),
            expand: *expand,
            targets,
        });
    }

    /// Tira dos resultados o que já foi trocado (buscar de novo leria o disco, e as abas abertas ainda não
    /// foram salvas).
    fn drop_replaced(&mut self, failed: &[PathBuf]) {
        let sent = std::mem::take(&mut self.sent);
        let Some(results) = self.results.as_mut() else { return };
        let r = Arc::make_mut(results);
        for t in sent.iter().filter(|t| !failed.contains(&t.path)) {
            let Some(i) = r.files.iter().position(|f| f.path == t.path) else { continue };
            match &t.lines {
                None => {
                    r.files.remove(i);
                }
                Some(lines) => {
                    r.files[i].matches.retain(|m| !lines.contains(&m.line));
                    if r.files[i].matches.is_empty() {
                        r.files.remove(i);
                    }
                }
            }
        }
        r.total = r.files.iter().map(|f| f.matches.len()).sum();
        if self.selected.is_some_and(|s| self.results.as_ref().is_none_or(|r| r.files.get(s.file).is_none())) {
            self.selected = None;
        }
        self.select_first_if_none();
        self.refresh_previews();
    }

    fn rel<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, str> {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy()
    }

    fn scope_label(&self, path: &Path) -> String {
        let rel = path.strip_prefix(&self.root).unwrap_or(path);
        if rel.as_os_str().is_empty() { ".".into() } else { rel.to_string_lossy().replace('\\', "/") }
    }

    fn remember_query(&mut self) {
        let before = self.history.clone();
        engine::push_history(&mut self.history, &self.query, HISTORY_LIMIT);
        if self.history != before {
            save_history(&self.history);
        }
    }

    fn select_first_if_none(&mut self) {
        if self.selected.is_some() {
            return;
        }
        self.select_first();
    }

    fn select_first(&mut self) {
        let Some(results) = &self.results else {
            self.selected = None;
            return;
        };
        for (fi, file) in results.files.iter().enumerate() {
            if !file.matches.is_empty() {
                self.selected = Some(Sel { file: fi, line: Some(0) });
                return;
            }
        }
        self.selected = results.files.first().map(|_| Sel { file: 0, line: None });
    }

    fn visible_items(&self) -> Vec<Sel> {
        let Some(results) = &self.results else { return Vec::new() };
        let mut out = Vec::new();
        for (fi, file) in results.files.iter().enumerate() {
            out.push(Sel { file: fi, line: None });
            if !self.collapsed.contains(&file.path) {
                for mi in 0..file.matches.len() {
                    out.push(Sel { file: fi, line: Some(mi) });
                }
            }
        }
        out
    }

    fn flat_matches(&self) -> Vec<(usize, usize)> {
        let Some(results) = &self.results else { return Vec::new() };
        let mut out = Vec::new();
        for (fi, file) in results.files.iter().enumerate() {
            for mi in 0..file.matches.len() {
                out.push((fi, mi));
            }
        }
        out
    }

    fn move_sel(&mut self, delta: isize) -> Task<Message> {
        let items = self.visible_items();
        if items.is_empty() {
            return Task::none();
        }
        let current = self.selected.and_then(|sel| items.iter().position(|item| *item == sel));
        let next = match current {
            Some(i) => (i as isize + delta).rem_euclid(items.len() as isize) as usize,
            None if delta < 0 => items.len() - 1,
            None => 0,
        };
        self.selected = Some(items[next]);
        self.nav_keys = true;
        self.history_browse = None;
        self.scroll_selected()
    }

    fn collapse_selected(&mut self) {
        let Some(sel) = self.selected else { return };
        let Some(path) = self.results.as_ref().and_then(|r| r.files.get(sel.file)).map(|f| f.path.clone()) else { return };
        self.collapsed.insert(path);
        self.selected = Some(Sel { file: sel.file, line: None });
    }

    fn expand_selected(&mut self) {
        let Some(sel) = self.selected else { return };
        let Some(path) = self.results.as_ref().and_then(|r| r.files.get(sel.file)).map(|f| f.path.clone()) else { return };
        self.collapsed.remove(&path);
    }

    /// Abre o resultado selecionado, ou abre/fecha o grupo se a seleção é o arquivo. `false` = não havia o que abrir.
    fn open_selected(&mut self, ctx: &mut Context) -> bool {
        let Some(sel) = self.selected else { return false };
        let Some(file) = self.results.as_ref().and_then(|r| r.files.get(sel.file)) else { return false };
        if let Some(mi) = sel.line {
            let Some(m) = file.matches.get(mi) else { return false };
            let path = file.path.clone();
            let (line, column) = (m.line, m.column);
            ctx.open_file_at(path, line, column);
            true
        } else {
            let path = file.path.clone();
            if !self.collapsed.remove(&path) {
                self.collapsed.insert(path);
            }
            true
        }
    }

    fn step_match(&mut self, delta: isize, ctx: &mut Context) -> Task<Message> {
        let all = self.flat_matches();
        if all.is_empty() {
            ctx.notify(fl!("search-no-results"));
            return Task::none();
        }
        let index = match self.selected {
            Some(Sel { file, line: Some(mi) }) => {
                let i = all.iter().position(|&(f, m)| f == file && m == mi).unwrap_or(0);
                (i as isize + delta).rem_euclid(all.len() as isize) as usize
            }
            Some(Sel { file, line: None }) => {
                if delta < 0 {
                    all.iter().rposition(|&(f, _)| f < file).unwrap_or(all.len() - 1)
                } else {
                    all.iter().position(|&(f, _)| f >= file).unwrap_or(0)
                }
            }
            None => {
                if delta < 0 { all.len() - 1 } else { 0 }
            }
        };
        let (fi, mi) = all[index];
        let Some(found) = self.results.as_ref().and_then(|r| r.files.get(fi)).and_then(|f| {
            f.matches.get(mi).map(|m| (f.path.clone(), m.line, m.column))
        }) else {
            return Task::none();
        };
        let (path, line, column) = found;
        self.collapsed.remove(&path);
        self.selected = Some(Sel { file: fi, line: Some(mi) });
        self.nav_keys = true;
        ctx.show_panel(PANEL);
        ctx.open_file_at(path, line, column);
        self.scroll_selected()
    }

    fn browse_history(&mut self, delta: isize) -> Task<Message> {
        if self.history.is_empty() {
            return Task::none();
        }
        let len = self.history.len() as isize;
        let cur = self.history_browse.map(|i| i as isize).unwrap_or(if delta > 0 { -1 } else { 0 });
        let next = cur + delta;
        if next < 0 {
            self.history_browse = None;
            self.query.clear();
            self.nav_keys = false;
            return self.restart();
        }
        if next >= len {
            return Task::none();
        }
        self.history_browse = Some(next as usize);
        self.query = self.history[next as usize].clone();
        self.nav_keys = false;
        self.history_open = false;
        self.restart()
    }

    fn scroll_selected(&self) -> Task<Message> {
        let Some(y) = self.selection_y() else { return Task::none() };
        operation::scroll_to(results_id(), operation::AbsoluteOffset { x: None, y: Some(y) })
    }

    fn selection_y(&self) -> Option<f32> {
        let sel = self.selected?;
        let results = self.results.as_ref()?;
        let mut y: f32 = 0.0;
        for (fi, file) in results.files.iter().enumerate() {
            if sel == (Sel { file: fi, line: None }) {
                return Some((y - 4.0).max(0.0_f32));
            }
            y += ROW_HEADER;
            if self.collapsed.contains(&file.path) {
                continue;
            }
            for (mi, m) in file.matches.iter().enumerate() {
                if sel == (Sel { file: fi, line: Some(mi) }) {
                    return Some((y - 4.0).max(0.0_f32));
                }
                y += ROW_LINE * (1 + m.before.len() + m.after.len()) as f32;
                if self.replace_open {
                    y += ROW_LINE;
                }
            }
        }
        None
    }

    fn focus_query() -> Task<Message> {
        Task::batch([operation::focus(input_id()), operation::select_all(input_id())])
    }
}

impl Extension for SearchExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "search"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        self.history = load_history();
        self.context_configured = configured_context(&self.root);
        self.context_on = self.context_configured > 0;
        ctx.register_panel_hidden(PANEL, fl!("search-panel"), Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Search);
        ctx.register_command("search.focus", fl!("cmd-search-focus"));
        ctx.register_command("search.clear", fl!("cmd-search-clear"));
        ctx.register_command("search.replace", fl!("cmd-search-replace"));
        ctx.register_command("search.next", fl!("cmd-search-next"));
        ctx.register_command("search.prev", fl!("cmd-search-prev"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        if let CoreEvent::PanelShown(_) = event {
            let id = if std::mem::take(&mut self.focus_replace) { replace_id() } else { input_id() };
            return Task::batch([operation::focus(id.clone()), operation::select_all(id)]);
        }
        if let CoreEvent::SearchIn(path) = event {
            self.scope = Some(path.clone());
            self.history_open = false;
            ctx.show_panel(PANEL);
            return Task::batch([self.restart(), Self::focus_query()]);
        }
        if let CoreEvent::Appearance { theme, .. } = event {
            self.theme = theme.clone();
            return Task::none();
        }
        if let CoreEvent::ConfigReloaded = event {
            self.context_configured = configured_context(&self.root);
            if self.context_on && self.results.is_some() {
                return self.restart();
            }
            return Task::none();
        }
        if let CoreEvent::Replaced { failed, .. } = event
            && !self.sent.is_empty()
        {
            self.drop_replaced(failed);
            return Task::none();
        }
        // Resultados velhos depois de salvar: busca de novo.
        if let CoreEvent::FileSaved(_) = event
            && self.results.is_some()
        {
            return self.restart();
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "search.focus" => {
                ctx.show_panel(PANEL);
                Self::focus_query()
            }
            "search.replace" => {
                self.replace_open = true;
                self.focus_replace = true;
                self.refresh_previews();
                ctx.show_panel(PANEL);
                Task::batch([operation::focus(replace_id()), operation::select_all(replace_id())])
            }
            "search.clear" => {
                self.query.clear();
                self.collapsed.clear();
                self.selected = None;
                self.nav_keys = false;
                self.history_browse = None;
                self.restart()
            }
            "search.next" => self.step_match(1, ctx),
            "search.prev" => self.step_match(-1, ctx),
            _ => Task::none(),
        }
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::QueryChanged(q) => {
                self.query = q;
                self.history_browse = None;
                self.nav_keys = false;
                self.restart()
            }
            Message::IncludeChanged(text) => {
                self.include = text;
                self.restart()
            }
            Message::ExcludeChanged(text) => {
                self.exclude = text;
                self.restart()
            }
            Message::Submit => {
                self.history_browse = None;
                self.restart()
            }
            Message::QueryKey(key) => match key {
                QueryKey::Up | QueryKey::Down if self.history_browse.is_some() || (self.query.is_empty() && !self.history.is_empty()) => {
                    let delta = if key == QueryKey::Up { -1 } else { 1 };
                    self.browse_history(delta)
                }
                QueryKey::Up => self.move_sel(-1),
                QueryKey::Down => self.move_sel(1),
                QueryKey::Tab => self.move_sel(1),
                QueryKey::BackTab => self.move_sel(-1),
                QueryKey::Left => {
                    self.collapse_selected();
                    self.scroll_selected()
                }
                QueryKey::Right => {
                    self.expand_selected();
                    self.scroll_selected()
                }
                QueryKey::Enter => {
                    self.history_browse = None;
                    if self.open_selected(ctx) { self.scroll_selected() } else { self.restart() }
                }
            },
            Message::ToggleCase => {
                self.opts.case_sensitive = !self.opts.case_sensitive;
                self.restart()
            }
            Message::ToggleRegex => {
                self.opts.regex = !self.opts.regex;
                self.restart()
            }
            Message::ToggleWord => {
                self.opts.whole_word = !self.opts.whole_word;
                self.restart()
            }
            Message::ToggleFilters => {
                self.filters_open = !self.filters_open;
                if self.filters_open { operation::focus(include_id()) } else { Task::none() }
            }
            Message::ToggleContext => {
                self.context_on = !self.context_on;
                self.restart()
            }
            Message::ToggleHistory => {
                self.history_open = !self.history_open;
                Task::none()
            }
            Message::PickHistory(i) => {
                if let Some(query) = self.history.get(i).cloned() {
                    self.query = query;
                    self.history_browse = None;
                    self.history_open = false;
                    self.nav_keys = false;
                    return Task::batch([self.restart(), Self::focus_query()]);
                }
                Task::none()
            }
            Message::ClearScope => {
                self.scope = None;
                self.restart()
            }
            Message::ToggleFile(path) => {
                if let Some(fi) = self.results.as_ref().and_then(|r| r.files.iter().position(|f| f.path == path)) {
                    self.selected = Some(Sel { file: fi, line: None });
                    self.nav_keys = true;
                }
                if !self.collapsed.remove(&path) {
                    self.collapsed.insert(path);
                }
                self.scroll_selected()
            }
            Message::Open { file, index, path, line, column } => {
                self.selected = Some(Sel { file, line: Some(index) });
                self.nav_keys = true;
                ctx.open_file_at(path, line, column);
                self.scroll_selected()
            }
            Message::Done(generation, result) => {
                if generation == self.generation {
                    self.cancel = None;
                    match result {
                        Ok(results) => {
                            self.results = Some(results);
                            self.searched = self.pending_search.take();
                            self.selected = None;
                            self.select_first();
                            self.nav_keys = false;
                            self.remember_query();
                            self.refresh_previews();
                        }
                        Err(error) => {
                            self.error = Some(error);
                            self.results = None;
                            self.selected = None;
                            self.searched = None;
                            self.previews.clear();
                        }
                    }
                }
                Task::none()
            }
            Message::ToggleReplace => {
                self.replace_open = !self.replace_open;
                self.refresh_previews();
                if self.replace_open { operation::focus(replace_id()) } else { Task::none() }
            }
            Message::ReplacementChanged(text) => {
                self.replacement = text;
                self.refresh_previews();
                Task::none()
            }
            Message::ReplaceLine { path, line } => {
                self.send(vec![Target { path, lines: Some(vec![line]) }], ctx);
                Task::none()
            }
            Message::ReplaceFile(path) => {
                self.send(vec![Target { path, lines: None }], ctx);
                Task::none()
            }
            Message::ReplaceAll => {
                let Some(r) = self.results.as_ref().filter(|_| self.can_replace()) else { return Task::none() };
                let files = r.files.len();
                let mut message = fl!(
                    "search-replace-all-message",
                    results = r.total,
                    query = self.query.as_str(),
                    replacement = self.replacement.as_str()
                );
                if r.truncated {
                    message.push_str("\n\n");
                    message.push_str(&fl!("search-replace-all-truncated", count = engine::MAX_MATCHES));
                }
                ctx.show_dialog(
                    Dialog::new(fl!("search-replace-all-title", files = files), message)
                        .button(fl!("search-replace-all"), ButtonKind::Danger, Message::ReplaceAllConfirmed)
                        .cancel(fl!("search-cancel")),
                );
                Task::none()
            }
            Message::ReplaceAllConfirmed => {
                let targets = self
                    .results
                    .iter()
                    .flat_map(|r| r.files.iter())
                    .map(|f| Target { path: f.path.clone(), lines: None })
                    .collect();
                self.send(targets, ctx);
                Task::none()
            }
        }
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let toggle = |label: String, on: bool, msg: Message| {
            button(text(label).size(12)).on_press(msg).padding([3, 6]).style(if on { button::primary } else { button::text })
        };
        let nav_keys = self.nav_keys;
        let query = text_input(&fl!("search-placeholder"), &self.query)
            .id(input_id())
            .on_input(Message::QueryChanged)
            .on_submit(Message::Submit)
            .size(13)
            .padding(4);
        let query = keys::Intercept::new(query, move |key: &Key, modifiers: Modifiers| {
            if modifiers.control() || modifiers.alt() || modifiers.logo() {
                return None;
            }
            match key {
                Key::Named(Named::ArrowUp) => Some(Message::QueryKey(QueryKey::Up)),
                Key::Named(Named::ArrowDown) => Some(Message::QueryKey(QueryKey::Down)),
                Key::Named(Named::Tab) => Some(Message::QueryKey(if modifiers.shift() { QueryKey::BackTab } else { QueryKey::Tab })),
                Key::Named(Named::Enter) => Some(Message::QueryKey(QueryKey::Enter)),
                Key::Named(Named::ArrowLeft) if nav_keys => Some(Message::QueryKey(QueryKey::Left)),
                Key::Named(Named::ArrowRight) if nav_keys => Some(Message::QueryKey(QueryKey::Right)),
                _ => None,
            }
        });
        let bar = row![
            toggle(if self.replace_open { "▾".into() } else { "▸".into() }, false, Message::ToggleReplace),
            query,
            toggle("Aa".into(), self.opts.case_sensitive, Message::ToggleCase),
            toggle("ab".into(), self.opts.whole_word, Message::ToggleWord),
            toggle(".*".into(), self.opts.regex, Message::ToggleRegex),
            toggle(fl!("search-filters"), self.filters_open, Message::ToggleFilters),
            toggle(fl!("search-context"), self.context_on, Message::ToggleContext),
            toggle_button(Icon::History, fl!("search-history"), self.history_open, Message::ToggleHistory),
        ]
        .spacing(4)
        .align_y(iced::Center);

        let scope_chip = self.scope.as_ref().map(|path| {
            let label = self.scope_label(path);
            let caption = if path.is_file() {
                fl!("search-scope-file", path = label.as_str())
            } else {
                fl!("search-scope-folder", path = label.as_str())
            };
            container(
                row![
                    text(caption).size(12),
                    with_tip(
                        button(text("×").size(12)).on_press(Message::ClearScope).padding([0, 4]).style(button::text),
                        fl!("search-scope-clear"),
                    ),
                ]
                .spacing(4)
                .align_y(iced::Center),
            )
            .padding([2, 8])
            .style(|theme: &Theme| {
                let color = theme.extended_palette().primary.weak.color;
                container::Style {
                    background: Some(color.into()),
                    border: iced::Border { radius: 10.0.into(), ..iced::Border::default() },
                    ..container::Style::default()
                }
            })
        });

        let filters = self.filters_open.then(|| {
            column![
                text_input(&fl!("search-include-placeholder"), &self.include)
                    .id(include_id())
                    .on_input(Message::IncludeChanged)
                    .on_submit(Message::Submit)
                    .size(13)
                    .padding(4),
                text_input(&fl!("search-exclude-placeholder"), &self.exclude)
                    .id(exclude_id())
                    .on_input(Message::ExcludeChanged)
                    .on_submit(Message::Submit)
                    .size(13)
                    .padding(4),
            ]
            .spacing(4)
        });

        let history_list = self.history_open.then(|| {
            if self.history.is_empty() {
                Element::from(text(fl!("search-history-empty")).size(12).style(text::secondary))
            } else {
                let mut list = column![].spacing(1);
                for (i, query) in self.history.iter().enumerate() {
                    list = list.push(
                        button(text(query.clone()).size(12))
                            .on_press(Message::PickHistory(i))
                            .width(Fill)
                            .padding([2, 6])
                            .style(|t: &Theme, st| tea_core::style::list_item(t, st, false)),
                    );
                }
                list.into()
            }
        });

        let can_replace = self.can_replace();
        let replace_bar = self.replace_open.then(|| {
            row![
                space::horizontal().width(22),
                text_input(&fl!("search-replace-placeholder"), &self.replacement)
                    .id(replace_id())
                    .on_input(Message::ReplacementChanged)
                    .on_submit(Message::ReplaceAll)
                    .size(13)
                    .padding(4),
                button(text(fl!("search-replace-all")).size(12))
                    .on_press_maybe(can_replace.then_some(Message::ReplaceAll))
                    .padding([3, 8])
                    .style(button::secondary),
            ]
            .spacing(4)
            .align_y(iced::Center)
        });
        let replace_button = |msg: Message| {
            button(text(fl!("search-replace-one")).size(11))
                .on_press_maybe(can_replace.then_some(msg))
                .padding([1, 6])
                .style(button::text)
        };

        let summary = match (&self.error, &self.results) {
            (Some(e), _) if engine::build_matcher(&self.query, &self.opts).is_err() => {
                text(fl!("search-bad-regex", error = e.as_str())).style(text::danger)
            }
            (Some(e), _) => text(fl!("search-bad-glob", error = e.as_str())).style(text::danger),
            (None, _) if self.running() => text(fl!("search-running")).style(text::secondary),
            (None, Some(r)) => {
                let more = if r.truncated { fl!("search-truncated", count = engine::MAX_MATCHES) } else { String::new() };
                text(fl!(
                    "search-summary",
                    results = r.total,
                    files = r.files.len(),
                    more = more,
                    scanned = r.files_scanned
                ))
                .style(text::secondary)
            }
            (None, None) => text(fl!("search-min-chars", count = MIN_QUERY_CHARS)).style(text::secondary),
        }
        .size(12);

        let mut list = column![];
        for (fi, file) in self.results.iter().flat_map(|r| r.files.iter()).enumerate() {
            let collapsed = self.collapsed.contains(&file.path);
            let header_selected = self.selected == Some(Sel { file: fi, line: None });
            let header = row![
                text(if collapsed { "▸" } else { "▾" }).size(12).width(14),
                tea_core::file_icons::icon(&file.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(), 13.0, true),
                text(self.rel(&file.path)).size(13),
                text(format!("  {}", file.matches.len())).size(12).style(text::secondary),
            ];
            let header = button(header)
                .on_press(Message::ToggleFile(file.path.clone()))
                .style(move |t: &Theme, st| tea_core::style::list_item(t, st, header_selected))
                .padding([2, 4])
                .width(Fill);
            list = list.push(if self.replace_open {
                Element::from(row![header, replace_button(Message::ReplaceFile(file.path.clone()))].align_y(iced::Center))
            } else {
                header.into()
            });
            if collapsed {
                continue;
            }
            for (mi, m) in file.matches.iter().enumerate() {
                let selected = self.selected == Some(Sel { file: fi, line: Some(mi) });
                let number = |n: usize| text(format!("{n:>5}")).size(12).style(text::secondary).font(Font::MONOSPACE);
                let mut block = column![];
                for ctx_line in &m.before {
                    block = block.push(context_row(ctx_line));
                }
                let old = highlighted(&self.theme, &m.text, &m.spans, m.prefix_ellipsis, m.suffix_ellipsis, if self.replace_open { Mark::Old } else { Mark::Hit });
                let body: Element<'_, Message> = match self.previews.get(fi).and_then(|p| p.get(mi)) {
                    Some(preview) => column![
                        row![number(m.line), old].spacing(8),
                        row![
                            text("→").size(12).style(text::secondary).font(Font::MONOSPACE),
                            highlighted(&self.theme, &preview.text, &preview.spans, m.prefix_ellipsis, m.suffix_ellipsis, Mark::New),
                        ]
                        .spacing(6),
                    ]
                    .into(),
                    None => row![number(m.line), old].spacing(8).into(),
                };
                block = block.push(body);
                for ctx_line in &m.after {
                    block = block.push(context_row(ctx_line));
                }
                let line = row![space::horizontal().width(14), block].spacing(8);
                let line = button(line)
                    .on_press(Message::Open {
                        file: fi,
                        index: mi,
                        path: file.path.clone(),
                        line: m.line,
                        column: m.column,
                    })
                    .style(move |t: &Theme, st| tea_core::style::list_item(t, st, selected))
                    .padding([1, 4])
                    .width(Fill);
                list = list.push(match self.replace_open {
                    true => Element::from(
                        row![line, replace_button(Message::ReplaceLine { path: file.path.clone(), line: m.line })].align_y(iced::Center),
                    ),
                    false => line.into(),
                });
            }
        }

        column![bar]
            .push(scope_chip)
            .push(filters)
            .push(history_list)
            .push(replace_bar)
            .push(summary)
            .push(tea_core::style::scrollable(list).id(results_id()).height(Fill))
            .spacing(4)
            .into()
    }
}

fn context_row(line: &engine::ContextLine) -> Element<'static, Message> {
    row![
        text(format!("{:>5}", line.line)).size(12).style(text::secondary).font(Font::MONOSPACE),
        text(line.text.clone()).size(12).style(text::secondary).font(Font::MONOSPACE),
    ]
    .spacing(8)
    .into()
}

fn highlighted(theme: &Theme, body: &str, spans: &[(usize, usize)], prefix: bool, suffix: bool, kind: Mark) -> Element<'static, Message> {
    let palette = theme.extended_palette();
    let chars: Vec<char> = body.chars().collect();
    let mono = Font::MONOSPACE;
    let mut pieces: Vec<iced::widget::text::Span<'static, (), Font>> = Vec::new();
    if prefix {
        pieces.push(span("…").font(mono).color(palette.background.base.text.scale_alpha(0.6)));
    }
    let mut cursor = 0;
    for &(start, end) in spans {
        let start = start.min(chars.len());
        let end = end.min(chars.len()).max(start);
        if cursor < start {
            pieces.push(span(chars[cursor..start].iter().collect::<String>()).font(mono));
        }
        if start < end {
            let word: String = chars[start..end].iter().collect();
            let piece = span(word).font(Font { weight: iced::font::Weight::Semibold, ..mono });
            let piece = match kind {
                Mark::Hit => piece.color(palette.primary.strong.color).background(palette.primary.weak.color),
                Mark::Old => piece.strikethrough(true).color(palette.background.base.text.scale_alpha(0.55)),
                Mark::New => piece.color(palette.success.strong.color).background(palette.success.weak.color),
            };
            pieces.push(piece);
        }
        cursor = end;
    }
    if cursor < chars.len() {
        pieces.push(span(chars[cursor..].iter().collect::<String>()).font(mono));
    }
    if suffix {
        pieces.push(span("…").font(mono).color(palette.background.base.text.scale_alpha(0.6)));
    }
    if pieces.is_empty() {
        pieces.push(span(" ").font(mono));
    }
    rich_text(pieces).size(12).into()
}

fn history_path() -> Option<PathBuf> {
    Some(tea_core::os::state_dir()?.join("idetea").join("search-history.txt"))
}

fn load_history() -> Vec<String> {
    let Some(path) = history_path() else { return Vec::new() };
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    text.lines().map(str::trim).filter(|line| !line.is_empty()).take(HISTORY_LIMIT).map(str::to_string).collect()
}

fn save_history(items: &[String]) {
    let Some(path) = history_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut text = items.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    let _ = std::fs::write(path, text);
}

fn configured_context(root: &Path) -> usize {
    let global = tea_core::config::global_config_path();
    let project = tea_core::config::project_config_path(root);
    tea_core::config::Config::load(global.as_deref(), Some(&project)).search.context_lines as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::{ContextLine, FileMatches, LineMatch};
    use tea_core::Request;

    fn line(n: usize, text: &str) -> LineMatch {
        let end = text.chars().count();
        LineMatch {
            line: n,
            column: 1,
            text: text.into(),
            spans: vec![(0, end)],
            prefix_ellipsis: false,
            suffix_ellipsis: false,
            before: Vec::new(),
            after: Vec::new(),
        }
    }

    fn sample() -> Results {
        Results {
            files: vec![
                FileMatches {
                    path: PathBuf::from("/p/a.rs"),
                    matches: vec![line(1, "one"), line(2, "two")],
                },
                FileMatches { path: PathBuf::from("/p/b.rs"), matches: vec![line(4, "three")] },
            ],
            total: 3,
            ..Results::default()
        }
    }

    #[test]
    fn selection_walks_across_files_and_collapse() {
        let mut ext = SearchExtension { results: Some(Arc::new(sample())), ..SearchExtension::default() };
        ext.select_first();
        assert_eq!(ext.selected, Some(Sel { file: 0, line: Some(0) }));
        let _ = ext.move_sel(1);
        assert_eq!(ext.selected, Some(Sel { file: 0, line: Some(1) }));
        let _ = ext.move_sel(1);
        assert_eq!(ext.selected, Some(Sel { file: 1, line: None }), "o próximo item visível é o arquivo");
        let _ = ext.move_sel(1);
        assert_eq!(ext.selected, Some(Sel { file: 1, line: Some(0) }));
        ext.collapse_selected();
        assert!(ext.collapsed.contains(Path::new("/p/b.rs")));
        assert_eq!(ext.selected, Some(Sel { file: 1, line: None }));
        assert!(!ext.visible_items().iter().any(|s| s.file == 1 && s.line.is_some()));
        ext.expand_selected();
        assert!(ext.visible_items().iter().any(|s| s == &Sel { file: 1, line: Some(0) }));
    }

    #[test]
    fn next_and_previous_wrap_matches_and_expand() {
        let root = PathBuf::from("/p");
        let mut ext = SearchExtension { root: root.clone(), results: Some(Arc::new(sample())), ..SearchExtension::default() };
        ext.collapsed.insert(PathBuf::from("/p/b.rs"));
        ext.select_first();
        let mut ctx = Context::new(&root, None);
        let _ = ext.step_match(1, &mut ctx);
        assert_eq!(ext.selected, Some(Sel { file: 0, line: Some(1) }));
        let _ = ext.step_match(1, &mut ctx);
        assert_eq!(ext.selected, Some(Sel { file: 1, line: Some(0) }));
        assert!(!ext.collapsed.contains(Path::new("/p/b.rs")), "o próximo resultado abre o grupo");
        let _ = ext.step_match(1, &mut ctx);
        assert_eq!(ext.selected, Some(Sel { file: 0, line: Some(0) }), "volta ao começo");
        let _ = ext.step_match(-1, &mut ctx);
        assert_eq!(ext.selected, Some(Sel { file: 1, line: Some(0) }));
        let reqs = ctx.into_requests();
        assert!(reqs.iter().any(|r| matches!(r, Request::OpenFileAt { line: 4, .. })));
        assert!(reqs.iter().any(|r| matches!(r, Request::PanelVisibility { visible: Some(true), .. })));
    }

    #[test]
    fn search_in_sets_scope_chip_source() {
        let root = PathBuf::from("/p");
        let mut ext = SearchExtension { root: root.clone(), ..SearchExtension::default() };
        let mut ctx = Context::new(&root, None);
        let _ = ext.on_event(&CoreEvent::SearchIn(PathBuf::from("/p/src")), &mut ctx);
        assert_eq!(ext.scope, Some(PathBuf::from("/p/src")));
        assert!(ctx.into_requests().iter().any(|r| matches!(r, Request::PanelVisibility { visible: Some(true), .. })));
        assert_eq!(ext.scope_label(Path::new("/p/src")), "src");
    }

    #[test]
    fn context_row_keeps_its_line_number() {
        let row = ContextLine { line: 9, text: "perto".into() };
        assert_eq!(row.line, 9);
        assert_eq!(row.text, "perto");
    }
}
