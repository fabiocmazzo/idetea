//! Git: branch na barra de status e um painel com as mudanças, stage/unstage,
//! commit e diff do arquivo selecionado. Usa o binário `git` do sistema.

mod git;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use iced::futures::channel::{mpsc, oneshot};
use iced::widget::{Id, button, column, container, operation, row, rule, scrollable, space, text, text_input};
use iced::{Element, Fill, Font, Subscription, Task, Theme};
use tea_core::icons::{Icon, icon, icon_button};
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

pub use git::{Entry, Repo, Status};
use git::{DiffLine, LineKind};

const PANEL: PanelId = PanelId("git");
const BAR: PanelId = PanelId("git-bar");
/// Diffs maiores são cortados (a tela não precisa de mais que isso).
const MAX_DIFF_LINES: usize = 3000;
/// De quanto em quanto tempo olhar se o `.git` mudou (commit no terminal, checkout...).
const WATCH_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
pub enum Message {
    Discovered(Option<Repo>),
    Refresh,
    Refreshed(u64, Arc<Result<Status, String>>),
    Select { path: String, staged: bool },
    DiffLoaded { path: String, staged: bool, diff: Arc<Result<String, String>> },
    Open(String),
    OpenAt(String, usize),
    Stage(String),
    Unstage(String),
    StageAll,
    UnstageAll,
    CommitMessage(String),
    Commit,
    /// Uma ação terminou: `Ok(Some(msg))` vira notificação.
    Done(Arc<Result<Option<String>, String>>),
    GitDirChanged,
}

/// Arquivo cujo diff está na tela.
struct Selected {
    path: String,
    staged: bool,
    lines: Vec<DiffLine>,
    truncated: bool,
    error: Option<String>,
}

#[derive(Default)]
pub struct GitExtension {
    root: PathBuf,
    repo: Option<Repo>,
    status: Option<Status>,
    error: Option<String>,
    generation: u64,
    selected: Option<Selected>,
    message: String,
    busy: bool,
}

/// Roda `f` numa thread e entrega o resultado como mensagem.
fn spawn<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static, done: fn(T) -> Message) -> Task<Message> {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    Task::future(async move { rx.await.ok() }).and_then(move |r| Task::done(done(r)))
}

fn commit_input_id() -> Id {
    Id::new("git-commit-message")
}

impl GitExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn refresh(&mut self) -> Task<Message> {
        let Some(repo) = self.repo.clone() else { return Task::none() };
        self.generation += 1;
        let generation = self.generation;
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(git::status(&repo));
        });
        Task::future(async move { rx.await.ok() })
            .and_then(move |r| Task::done(Message::Refreshed(generation, Arc::new(r))))
    }

    fn entry(&self, path: &str) -> Option<&Entry> {
        self.status.as_ref()?.entries.iter().find(|e| e.path == path)
    }

    fn load_diff(&self, path: String, staged: bool) -> Task<Message> {
        let (Some(repo), Some(entry)) = (self.repo.clone(), self.entry(&path).cloned()) else {
            return Task::none();
        };
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(git::diff(&repo, &entry, staged));
        });
        Task::future(async move { rx.await.ok() })
            .and_then(move |d| Task::done(Message::DiffLoaded { path: path.clone(), staged, diff: Arc::new(d) }))
    }

    /// Roda uma ação do git e depois atualiza a lista.
    fn action(&mut self, f: impl FnOnce(&Repo) -> Result<Option<String>, String> + Send + 'static) -> Task<Message> {
        let Some(repo) = self.repo.clone() else { return Task::none() };
        self.busy = true;
        spawn(move || f(&repo), |r| Message::Done(Arc::new(r)))
    }

    /// Caminho e, numa renomeação, o nome antigo (os dois lados entram e saem do stage juntos).
    fn paths_of(&self, path: &str) -> Vec<String> {
        let mut paths = vec![path.to_string()];
        if let Some(orig) = self.entry(path).and_then(|e| e.orig.clone()) {
            paths.push(orig);
        }
        paths
    }

    fn abs(&self, path: &str) -> PathBuf {
        self.repo.as_ref().map_or_else(|| self.root.join(path), |r| r.root.join(path))
    }
}

impl Extension for GitExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "git"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, "Git", Slot::Bottom);
        ctx.register_panel(BAR, "Git", Slot::Status);
        ctx.register_command("git.focus", "Git: mostrar mudanças");
        ctx.register_command("git.refresh", "Git: atualizar");
        ctx.register_command("git.stage_all", "Git: adicionar tudo ao stage");
        ctx.register_command("git.unstage_all", "Git: tirar tudo do stage");
        ctx.register_command("git.commit", "Git: commit (foca a mensagem)");
        let root = self.root.clone();
        spawn(move || git::discover(&root), Message::Discovered)
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::FileSaved(path) if self.repo.as_ref().is_some_and(|r| path.starts_with(&r.root)) => {
                self.refresh()
            }
            _ => Task::none(),
        }
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if self.repo.is_none() {
            ctx.notify("Git: esta pasta não está num repositório");
            return Task::none();
        }
        match command {
            "git.focus" => {
                ctx.show_panel(PANEL);
                self.refresh()
            }
            "git.refresh" => self.refresh(),
            "git.stage_all" => self.update(Message::StageAll, ctx),
            "git.unstage_all" => self.update(Message::UnstageAll, ctx),
            "git.commit" => {
                ctx.show_panel(PANEL);
                operation::focus(commit_input_id())
            }
            _ => Task::none(),
        }
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Discovered(repo) => {
                self.repo = repo;
                return self.refresh();
            }
            Message::Refresh | Message::GitDirChanged => return self.refresh(),
            Message::Refreshed(generation, result) => {
                if generation != self.generation {
                    return Task::none();
                }
                match &*result {
                    Ok(st) => {
                        self.status = Some(st.clone());
                        self.error = None;
                    }
                    Err(e) => self.error = Some(e.clone()),
                }
                // O diff na tela pode ter mudado (arquivo salvo, stage...).
                if let Some(sel) = &self.selected {
                    let still = self.entry(&sel.path).is_some_and(|e| {
                        if sel.staged { e.staged.is_some() } else { e.unstaged.is_some() }
                    });
                    if still {
                        return self.load_diff(sel.path.clone(), sel.staged);
                    }
                    self.selected = None;
                }
            }
            Message::Select { path, staged } => return self.load_diff(path, staged),
            Message::DiffLoaded { path, staged, diff } => {
                let (lines, truncated, error) = match &*diff {
                    Ok(out) => {
                        let (lines, truncated) = git::parse_diff(out, MAX_DIFF_LINES);
                        (lines, truncated, None)
                    }
                    Err(e) => (Vec::new(), false, Some(e.clone())),
                };
                self.selected = Some(Selected { path, staged, lines, truncated, error });
            }
            Message::Open(path) => ctx.open_file(self.abs(&path)),
            Message::OpenAt(path, line) => ctx.open_file_at(self.abs(&path), line, 1),
            Message::Stage(path) => {
                let paths = self.paths_of(&path);
                return self.action(move |r| git::stage(r, &paths).map(|_| None));
            }
            Message::Unstage(path) => {
                let paths = self.paths_of(&path);
                return self.action(move |r| git::unstage(r, &paths).map(|_| None));
            }
            Message::StageAll => return self.action(|r| git::stage_all(r).map(|_| None)),
            Message::UnstageAll => return self.action(|r| git::unstage_all(r).map(|_| None)),
            Message::CommitMessage(m) => self.message = m,
            Message::Commit => {
                let msg = self.message.trim().to_string();
                if msg.is_empty() {
                    ctx.notify("Git: escreva a mensagem do commit");
                } else if self.status.as_ref().is_none_or(Status::is_staged_empty) {
                    ctx.notify("Git: nada no stage (use + para adicionar)");
                } else if !self.busy {
                    return self.action(move |r| git::commit(r, &msg).map(|c| Some(format!("Commit {c}"))));
                }
            }
            Message::Done(result) => {
                self.busy = false;
                match &*result {
                    Ok(Some(msg)) => {
                        if msg.starts_with("Commit ") {
                            self.message.clear();
                        }
                        ctx.notify(format!("Git: {}", msg.trim_start_matches("Commit ")));
                    }
                    Ok(None) => {}
                    Err(e) => ctx.notify(format!("Git: {e}")),
                }
                return self.refresh();
            }
        }
        Task::none()
    }

    fn view(&self, panel: PanelId) -> Element<'_, Message> {
        if panel == BAR {
            return self.bar_view();
        }
        if self.repo.is_none() {
            return text("Esta pasta não está num repositório git.").size(13).style(text::secondary).into();
        }
        row![container(self.changes_view()).width(340).height(Fill), rule::vertical(1), self.diff_view()]
            .spacing(6)
            .height(Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.repo {
            Some(repo) => Subscription::run_with(repo.git_dir.clone(), watch_git_dir),
            None => Subscription::none(),
        }
    }
}

// ---- Telas -------------------------------------------------------------------

fn status_color(c: char) -> fn(&Theme) -> text::Style {
    match c {
        'A' | '?' => |t| text::Style { color: Some(t.extended_palette().success.base.color) },
        'D' | 'U' => |t| text::Style { color: Some(t.extended_palette().danger.base.color) },
        'R' | 'C' => |t| text::Style { color: Some(t.extended_palette().primary.base.color) },
        _ => |t| text::Style { color: Some(t.extended_palette().warning.base.color) },
    }
}


impl GitExtension {
    fn bar_view(&self) -> Element<'_, Message> {
        let Some(st) = &self.status else { return space::horizontal().width(0).into() };
        let mut label = row![icon(Icon::SourceControl).size(13), text(st.head_label()).size(12)].spacing(4);
        if st.ahead > 0 {
            label = label.push(row![icon(Icon::ArrowUp).size(12), text(st.ahead).size(12)]);
        }
        if st.behind > 0 {
            label = label.push(row![icon(Icon::ArrowDown).size(12), text(st.behind).size(12)]);
        }
        if !st.entries.is_empty() {
            label = label.push(text(format!("●{}", st.entries.len())).size(12));
        }
        button(label.align_y(iced::Center)).on_press(Message::Refresh).style(button::text).padding([0, 4]).into()
    }

    fn changes_view(&self) -> Element<'_, Message> {
        let head = self.status.as_ref().map(Status::head_label).unwrap_or_default();
        let header = row![
            icon(Icon::SourceControl).size(14),
            text(head).size(13),
            space::horizontal(),
            icon_button(Icon::Refresh, "Atualizar", Some(Message::Refresh)),
        ]
        .align_y(iced::Center);

        let mut commit = text_input("Mensagem do commit (Enter)", &self.message)
            .id(commit_input_id())
            .size(13)
            .padding([3, 6]);
        if !self.busy {
            commit = commit.on_input(Message::CommitMessage).on_submit(Message::Commit);
        }

        let mut list = column![].spacing(1);
        if let Some(e) = &self.error {
            list = list.push(text(e).size(12).style(text::danger));
        }
        if let Some(st) = &self.status {
            let staged: Vec<&Entry> = st.entries.iter().filter(|e| e.staged.is_some()).collect();
            let unstaged: Vec<&Entry> = st.entries.iter().filter(|e| e.unstaged.is_some()).collect();
            if !staged.is_empty() {
                list = list.push(self.section("No stage", staged.len(), icon_button(Icon::Remove, "Tirar tudo do stage", Some(Message::UnstageAll))));
                for e in staged {
                    list = list.push(self.entry_row(e, true));
                }
            }
            if !unstaged.is_empty() {
                list = list.push(self.section("Mudanças", unstaged.len(), icon_button(Icon::Add, "Adicionar tudo ao stage", Some(Message::StageAll))));
                for e in unstaged {
                    list = list.push(self.entry_row(e, false));
                }
            }
            if st.entries.is_empty() {
                list = list.push(text("Nada para commitar.").size(12).style(text::secondary));
            }
        }
        column![header, commit, scrollable(list).height(Fill)].spacing(4).into()
    }

    fn section<'a>(&self, title: &'a str, n: usize, action: Element<'a, Message>) -> Element<'a, Message> {
        row![text(format!("{} ({n})", title.to_uppercase())).size(11).style(text::secondary), space::horizontal(), action]
            .align_y(iced::Center)
            .padding([2, 0])
            .into()
    }

    fn entry_row<'a>(&self, e: &'a Entry, staged: bool) -> Element<'a, Message> {
        let letter = if staged { e.staged } else { e.unstaged }.unwrap_or(' ');
        let letter = if e.untracked { 'U' } else { letter };
        let color = if e.untracked { status_color('?') } else { status_color(letter) };
        let (name, dir) = match e.path.rsplit_once('/') {
            Some((dir, name)) => (name, dir),
            None => (e.path.as_str(), ""),
        };
        let selected = self.selected.as_ref().is_some_and(|s| s.path == e.path && s.staged == staged);
        let label = row![
            text(name).size(12),
            text(dir).size(11).style(text::secondary),
            space::horizontal(),
            text(letter.to_string()).size(12).font(Font::MONOSPACE).style(color),
        ]
        .spacing(6)
        .align_y(iced::Center);
        let toggle = if staged {
            icon_button(Icon::Remove, "Tirar do stage", Some(Message::Unstage(e.path.clone())))
        } else {
            icon_button(Icon::Add, "Adicionar ao stage", Some(Message::Stage(e.path.clone())))
        };
        row![
            button(label)
                .on_press(Message::Select { path: e.path.clone(), staged })
                .style(if selected { button::secondary } else { button::text })
                .padding([1, 4])
                .width(Fill),
            icon_button(Icon::GoToFile, "Abrir arquivo", Some(Message::Open(e.path.clone()))),
            toggle,
        ]
        .align_y(iced::Center)
        .into()
    }

    fn diff_view(&self) -> Element<'_, Message> {
        let Some(sel) = &self.selected else {
            return container(text("Selecione um arquivo para ver o diff.").size(12).style(text::secondary))
                .center(Fill)
                .into();
        };
        let title = row![
            text(&sel.path).size(12),
            text(if sel.staged { "  (stage)" } else { "" }).size(11).style(text::secondary),
            space::horizontal(),
            icon_button(Icon::GoToFile, "Abrir arquivo", Some(Message::Open(sel.path.clone()))),
        ]
        .align_y(iced::Center);
        if let Some(e) = &sel.error {
            return column![title, text(e).size(12).style(text::danger)].spacing(4).into();
        }
        let mut lines = column![];
        for l in &sel.lines {
            let style: fn(&Theme) -> text::Style = match l.kind {
                LineKind::Added => |t| text::Style { color: Some(t.extended_palette().success.base.color) },
                LineKind::Removed => |t| text::Style { color: Some(t.extended_palette().danger.base.color) },
                LineKind::Hunk => |t| text::Style { color: Some(t.extended_palette().primary.base.color) },
                LineKind::Header => text::secondary,
                LineKind::Context => text::default,
            };
            let line = text(if l.text.is_empty() { " " } else { &l.text }).size(12).font(Font::MONOSPACE).style(style);
            lines = lines.push(match l.new_line {
                Some(n) => button(line)
                    .on_press(Message::OpenAt(sel.path.clone(), n))
                    .style(button::text)
                    .padding(0)
                    .width(Fill)
                    .into(),
                None => Element::from(line),
            });
        }
        if sel.truncated {
            lines = lines.push(text(format!("… diff cortado em {MAX_DIFF_LINES} linhas")).size(12).style(text::secondary));
        }
        if sel.lines.is_empty() {
            lines = lines.push(text("Sem diferenças (arquivo binário ou só permissão).").size(12).style(text::secondary));
        }
        column![
            title,
            scrollable(lines).direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::default(),
                horizontal: scrollable::Scrollbar::default(),
            })
        ]
        .spacing(4)
        .width(Fill)
        .into()
    }
}

// ---- Observador do .git --------------------------------------------------------

/// Arquivos que mudam em commit, stage, checkout, reset, fetch.
fn watched(git_dir: &Path) -> [PathBuf; 4] {
    ["index", "HEAD", "logs/HEAD", "FETCH_HEAD"].map(|f| git_dir.join(f))
}

fn stamp(files: &[PathBuf]) -> Vec<Option<SystemTime>> {
    files.iter().map(|f| std::fs::metadata(f).and_then(|m| m.modified()).ok()).collect()
}

/// Uma thread olha as datas de alguns arquivos do `.git` a cada segundo (só `stat`, barato).
/// Termina sozinha quando a subscription some (o envio falha).
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
fn watch_git_dir(git_dir: &PathBuf) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let files = watched(git_dir);
    std::thread::spawn(move || {
        let mut last = stamp(&files);
        loop {
            std::thread::sleep(WATCH_INTERVAL);
            if tx.is_closed() {
                return;
            }
            let now = stamp(&files);
            if now != last {
                last = now;
                if tx.unbounded_send(Message::GitDirChanged).is_err() {
                    return;
                }
            }
        }
    });
    rx
}
