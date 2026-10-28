//! Busca no projeto: painel inferior com busca enquanto digita, respeitando `.gitignore`.
//!
//! A busca roda numa thread à parte. Cada nova busca cancela a anterior.

pub mod engine;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced::futures::channel::oneshot;
use iced::widget::{Id, button, column, operation, row, scrollable, space, text, text_input};
use iced::{Element, Fill, Task};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

use engine::{Options, Results};

const PANEL: PanelId = PanelId("search");
/// Espera um pouco antes de buscar, para não buscar a cada tecla.
const DEBOUNCE: Duration = Duration::from_millis(150);
const MIN_QUERY_CHARS: usize = 2;

fn input_id() -> Id {
    Id::new("search-input")
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
        ctx.register_panel_hidden(PANEL, "Busca", Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Search);
        ctx.register_command("search.focus", "Busca: buscar no projeto");
        ctx.register_command("search.clear", "Busca: limpar");
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        if let CoreEvent::PanelShown(_) = event {
            return Task::batch([operation::focus(input_id()), operation::select_all(input_id())]);
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
                }
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
            text_input("Buscar no projeto (Enter busca de novo)", &self.query)
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

        let summary = match (&self.error, &self.results) {
            (Some(e), _) => text(format!("Regex inválida: {e}")).style(text::danger),
            (None, _) if self.running() => text("Buscando…").style(text::secondary),
            (None, Some(r)) => {
                let more = if r.truncated { format!(" (parou em {})", engine::MAX_MATCHES) } else { String::new() };
                text(format!(
                    "{} resultado(s) em {} arquivo(s){more} · {} arquivos lidos",
                    r.total,
                    r.files.len(),
                    r.files_scanned
                ))
                .style(text::secondary)
            }
            (None, None) => text(format!("Digite ao menos {MIN_QUERY_CHARS} caracteres")).style(text::secondary),
        }
        .size(12);

        let mut list = column![];
        for file in self.results.iter().flat_map(|r| r.files.iter()) {
            let collapsed = self.collapsed.contains(&file.path);
            let header = row![
                text(if collapsed { "▸" } else { "▾" }).size(12).width(14),
                text(self.rel(&file.path)).size(13),
                text(format!("  {}", file.matches.len())).size(12).style(text::secondary),
            ];
            list = list.push(
                button(header)
                    .on_press(Message::ToggleFile(file.path.clone()))
                    .style(button::text)
                    .padding([2, 4])
                    .width(Fill),
            );
            if collapsed {
                continue;
            }
            for m in &file.matches {
                let line = row![
                    space::horizontal().width(14),
                    text(format!("{:>5}", m.line)).size(12).style(text::secondary).font(iced::Font::MONOSPACE),
                    text(&m.text).size(12).font(iced::Font::MONOSPACE),
                ]
                .spacing(8);
                list = list.push(
                    button(line)
                        .on_press(Message::Open { path: file.path.clone(), line: m.line, column: m.column })
                        .style(button::text)
                        .padding([1, 4])
                        .width(Fill),
                );
            }
        }

        column![bar, summary, scrollable(list).height(Fill)].spacing(4).into()
    }
}
