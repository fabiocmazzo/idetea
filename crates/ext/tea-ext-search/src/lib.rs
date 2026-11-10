//! Busca no projeto: painel inferior com busca enquanto digita, respeitando `.gitignore`.
//!
//! A busca roda numa thread à parte. Cada nova busca cancela a anterior.
//!
//! Substituir: com a linha "Substituir por" aberta, cada resultado mostra como fica. A troca em si é
//! do app ([`tea_core::replace`]): abas abertas mudam na memória, os outros arquivos são gravados.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

pub mod engine;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced::futures::channel::oneshot;
use regex::Regex;
use iced::widget::{Id, button, column, operation, row, space, text, text_input};
use iced::{Element, Fill, Task};
use tea_core::icons::Icon;
use tea_core::replace::{Replace, Target};
use tea_core::{ButtonKind, Context, CoreEvent, Dialog, Extension, PanelId, Slot};

use engine::{Options, Results};

const PANEL: PanelId = PanelId("search");
/// Espera um pouco antes de buscar, para não buscar a cada tecla.
const DEBOUNCE: Duration = Duration::from_millis(150);
const MIN_QUERY_CHARS: usize = 2;

fn input_id() -> Id {
    Id::new("search-input")
}

fn replace_id() -> Id {
    Id::new("search-replace")
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    Submit,
    ToggleCase,
    ToggleRegex,
    ToggleWord,
    ToggleFile(PathBuf),
    Open { path: PathBuf, line: usize, column: usize },
    Done(u64, Arc<Results>),
    ToggleReplace,
    ReplacementChanged(String),
    ReplaceLine { path: PathBuf, line: usize },
    ReplaceFile(PathBuf),
    /// Pede confirmação.
    ReplaceAll,
    ReplaceAllConfirmed,
}

#[derive(Default)]
pub struct SearchExtension {
    root: PathBuf,
    query: String,
    opts: Options,
    results: Option<Arc<Results>>,
    error: Option<String>,
    generation: u64,
    cancel: Option<Arc<AtomicBool>>,
    collapsed: HashSet<PathBuf>,
    replace_open: bool,
    replacement: String,
    /// Regex dos resultados mostrados (a da busca que terminou) e se `$1` vale na troca.
    searched: Option<(Regex, bool)>,
    /// A busca que está rodando, para virar `searched` quando terminar.
    pending_search: Option<(Regex, bool)>,
    /// Como fica cada linha dos resultados, na mesma ordem.
    previews: Vec<Vec<String>>,
    /// Pedidos de troca mandados ao app e ainda sem resposta.
    sent: Vec<Target>,
    /// Pôr o foco em "Substituir por" quando o painel aparecer.
    focus_replace: bool,
}

impl SearchExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn running(&self) -> bool {
        self.cancel.is_some()
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
            return Task::none();
        }
        let matcher = match engine::build_matcher(&self.query, &self.opts) {
            Ok(m) => m,
            Err(e) => {
                self.error = Some(e);
                return Task::none();
            }
        };
        self.pending_search = Some((matcher.clone(), self.opts.regex));
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        let generation = self.generation;
        let root = self.root.clone();
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(DEBOUNCE);
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let _ = tx.send(engine::search(&root, &matcher, &cancel));
        });
        // Se a busca foi cancelada, o canal fecha e nada chega.
        Task::future(async move { rx.await.ok() })
            .and_then(move |r| Task::done(Message::Done(generation, Arc::new(r))))
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
                    .map(|m| tea_core::replace::replace_line(&m.text, re, &self.replacement, *expand).0.into_owned())
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
        self.refresh_previews();
    }

    fn rel<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, str> {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy()
    }
}

impl Extension for SearchExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "search"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("search-panel"), Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Search);
        ctx.register_command("search.focus", fl!("cmd-search-focus"));
        ctx.register_command("search.clear", fl!("cmd-search-clear"));
        ctx.register_command("search.replace", fl!("cmd-search-replace"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        if let CoreEvent::PanelShown(_) = event {
            let id = if std::mem::take(&mut self.focus_replace) { replace_id() } else { input_id() };
            return Task::batch([operation::focus(id.clone()), operation::select_all(id)]);
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
                Task::batch([operation::focus(input_id()), operation::select_all(input_id())])
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
                self.restart()
            }
            _ => Task::none(),
        }
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::QueryChanged(q) => {
                self.query = q;
                self.restart()
            }
            Message::Submit => self.restart(),
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
            Message::ToggleFile(path) => {
                if !self.collapsed.remove(&path) {
                    self.collapsed.insert(path);
                }
                Task::none()
            }
            Message::Open { path, line, column } => {
                ctx.open_file_at(path, line, column);
                Task::none()
            }
            Message::Done(generation, results) => {
                if generation == self.generation {
                    self.cancel = None;
                    self.results = Some(results);
                    self.searched = self.pending_search.take();
                    self.refresh_previews();
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
        let toggle = |label: &'static str, on: bool, msg: Message| {
            button(text(label).size(12))
                .on_press(msg)
                .padding([3, 6])
                .style(if on { button::primary } else { button::text })
        };
        let bar = row![
            toggle(if self.replace_open { "▾" } else { "▸" }, false, Message::ToggleReplace),
            text_input(&fl!("search-placeholder"), &self.query)
                .id(input_id())
                .on_input(Message::QueryChanged)
                .on_submit(Message::Submit)
                .size(13)
                .padding(4),
            toggle("Aa", self.opts.case_sensitive, Message::ToggleCase),
            toggle("ab", self.opts.whole_word, Message::ToggleWord),
            toggle(".*", self.opts.regex, Message::ToggleRegex),
        ]
        .spacing(4)
        .align_y(iced::Center);

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
            (Some(e), _) => text(fl!("search-bad-regex", error = e.as_str())).style(text::danger),
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
            let header = row![
                text(if collapsed { "▸" } else { "▾" }).size(12).width(14),
                tea_core::file_icons::icon(
                    &file.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                    13.0,
                    true
                ),
                text(self.rel(&file.path)).size(13),
                text(format!("  {}", file.matches.len())).size(12).style(text::secondary),
            ];
            let header = button(header)
                .on_press(Message::ToggleFile(file.path.clone()))
                .style(button::text)
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
                let mono = |s: &str| text(s.to_string()).size(12).font(iced::Font::MONOSPACE);
                let number = text(format!("{:>5}", m.line)).size(12).style(text::secondary).font(iced::Font::MONOSPACE);
                let preview = self.previews.get(fi).and_then(|p| p.get(mi));
                let body: Element<'_, Message> = match preview {
                    Some(new) => column![
                        mono(&m.text).style(text::danger),
                        row![mono("→").style(text::secondary), mono(new).style(text::success)].spacing(6),
                    ]
                    .into(),
                    None => mono(&m.text).into(),
                };
                let line = row![space::horizontal().width(14), number, body].spacing(8);
                let line = button(line)
                    .on_press(Message::Open { path: file.path.clone(), line: m.line, column: m.column })
                    .style(button::text)
                    .padding([1, 4])
                    .width(Fill);
                list = list.push(match preview {
                    Some(_) => Element::from(
                        row![line, replace_button(Message::ReplaceLine { path: file.path.clone(), line: m.line })]
                            .align_y(iced::Center),
                    ),
                    None => line.into(),
                });
            }
        }

        column![bar].push(replace_bar).push(summary).push(tea_core::style::scrollable(list).height(Fill)).spacing(4).into()
    }
}
