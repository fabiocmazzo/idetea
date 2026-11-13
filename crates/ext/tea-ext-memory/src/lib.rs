//! Painel "Memória do projeto" (RFC 0016, etapa 4): os fatos de `docs/memory/` e as decisões de `docs/decisions/`,
//! com busca, abrir e apagar, e avisos de fato velho, de caminho citado que sumiu, de algo com cara de credencial e
//! de fato fora do índice. Quem grava são os agentes (e as pessoas); o painel mostra e ajuda a podar.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::widget::{Column, button, column, row, text, text_input};
use iced::{Element, Fill, Subscription, Task, Theme};
use tea_core::icons::Icon;
use tea_core::memory::{self, Memory, Warning};
use tea_core::menu::{Menu, context_area};
use tea_core::{ButtonKind, Context, CoreEvent, Dialog, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("memory");
/// De quanto em quanto tempo olhar as pastas (só `stat`; lê tudo quando algo muda).
const POLL: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Arc<Memory>),
    Query(String),
    Open(PathBuf),
    Menu { path: PathBuf, fact: bool, at: iced::Point },
    AskDelete(PathBuf),
    Delete(PathBuf),
    Teach,
}

#[derive(Default)]
pub struct MemoryExtension {
    root: PathBuf,
    memory: Arc<Memory>,
    query: String,
}

impl MemoryExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn reload(&mut self) {
        self.memory = Arc::new(memory::load(&self.root, memory::today()));
    }

    fn rel(&self, path: &Path) -> String {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy().replace('\\', "/")
    }

    fn view_panel(&self) -> Element<'_, Message> {
        let m = &self.memory;
        let mut out: Column<'_, Message> = column![].spacing(8).padding(12).width(Fill);
        if m.is_empty() {
            out = out
                .push(text(fl!("memory-empty")).size(13).width(Fill))
                .push(button(text(fl!("memory-teach")).size(12)).on_press(Message::Teach))
                .push(text(fl!("memory-empty-note")).size(12).style(text::secondary).width(Fill));
            return tea_core::style::scrollable(out).height(Fill).into();
        }
        out = out.push(text_input(&fl!("memory-search"), &self.query).on_input(Message::Query).size(13).width(Fill));
        let query = self.query.trim().to_lowercase();
        let hit = |parts: &[&str]| query.is_empty() || parts.iter().any(|p| p.to_lowercase().contains(&query));

        out = out.push(text(fl!("memory-facts", count = m.facts.len())).size(12).style(text::secondary));
        if !m.has_index && !m.facts.is_empty() {
            out = out.push(text(fl!("memory-no-index")).size(12).style(text::warning).width(Fill));
        }
        for file in &m.broken {
            out = out.push(text(fl!("memory-broken", file = file.clone())).size(12).style(text::warning).width(Fill));
        }
        let mut shown = 0;
        for f in m.facts.iter().filter(|f| hit(&[&f.name, &f.description, &f.kind, &f.body])) {
            shown += 1;
            let top = row![text(f.name.clone()).size(13).width(Fill), text(f.kind.clone()).size(11).style(muted)].spacing(6);
            let mut card = column![top].spacing(3);
            if !f.description.is_empty() {
                card = card.push(text(f.description.clone()).size(12).style(muted));
            }
            for w in &f.warnings {
                card = card.push(text(format!("⚠ {}", warning_text(w))).size(11).style(text::warning));
            }
            let path = f.path.clone();
            out = out.push(context_area(
                button(card).style(tea_core::style::card).padding([8, 10]).width(Fill).on_press(Message::Open(f.path.clone())),
                move |at| Message::Menu { path: path.clone(), fact: true, at },
            ));
        }

        out = out.push(text(fl!("memory-decisions", count = m.decisions.len())).size(12).style(text::secondary));
        for d in m.decisions.iter().filter(|d| hit(&[&d.title, &d.status])) {
            shown += 1;
            let top = row![
                text(format!("{:04}", d.number)).size(12).style(muted),
                text(d.title.clone()).size(13).width(Fill),
                text(d.status.clone()).size(11).style(muted),
            ]
            .spacing(6);
            let path = d.path.clone();
            out = out.push(context_area(
                button(top).style(tea_core::style::card).padding([8, 10]).width(Fill).on_press(Message::Open(d.path.clone())),
                move |at| Message::Menu { path: path.clone(), fact: false, at },
            ));
        }
        if shown == 0 && !query.is_empty() {
            out = out.push(text(fl!("memory-no-match")).size(13).style(text::secondary).width(Fill));
        }
        tea_core::style::scrollable(out).height(Fill).into()
    }
}

fn muted(theme: &Theme) -> text::Style {
    text::Style { color: Some(theme.extended_palette().background.weak.text.scale_alpha(0.72)) }
}

fn warning_text(w: &Warning) -> String {
    match w {
        Warning::Stale(Some(days)) => fl!("memory-stale-days", days = days.to_string()),
        Warning::Stale(None) => fl!("memory-stale-undated"),
        Warning::MissingPath(p) => fl!("memory-missing-path", path = p.clone()),
        Warning::Secret => fl!("memory-secret"),
        Warning::NotInIndex => fl!("memory-not-in-index"),
    }
}

impl Extension for MemoryExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "memory"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("memory-panel"), Slot::Left);
        ctx.panel_icon(PANEL, Icon::Book);
        ctx.register_command("memory.focus", fl!("cmd-memory-focus"));
        self.reload();
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        if matches!(event, CoreEvent::PanelShown(PANEL)) {
            self.reload();
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "memory.focus" {
            self.reload();
            ctx.show_panel(PANEL);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Loaded(m) => self.memory = m,
            Message::Query(q) => self.query = q,
            Message::Open(path) => ctx.open_file(path),
            Message::Menu { path, fact, at } => {
                let mut menu = Menu::new(at).item(Some(Icon::GoToFile), fl!("memory-open"), Message::Open(path.clone()));
                if fact {
                    menu = menu.separator().danger(Some(Icon::Trash), fl!("memory-delete"), Message::AskDelete(path));
                }
                ctx.show_menu(menu);
            }
            Message::AskDelete(path) => {
                let name = self.rel(&path);
                ctx.show_dialog(
                    Dialog::new(fl!("memory-delete-title"), fl!("memory-delete-detail", name = name))
                        .button(fl!("memory-delete"), ButtonKind::Danger, Message::Delete(path))
                        .cancel(fl!("memory-cancel")),
                );
            }
            Message::Delete(path) => {
                match memory::delete_fact(&self.root, &path) {
                    Ok(()) => {
                        ctx.path_deleted(path);
                        ctx.files_changed();
                    }
                    Err(e) => ctx.notify(fl!("memory-delete-failed", error = e.to_string())),
                }
                self.reload();
            }
            Message::Teach => ctx.run_command("interfaces.teach"),
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        self.view_panel()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.root.as_os_str().is_empty() {
            return Subscription::none();
        }
        Subscription::run_with(self.root.clone(), watch)
    }
}

/// Olha as duas pastas; quando algo muda (um agente gravou um fato), lê tudo e manda.
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
fn watch(root: &PathBuf) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let root = root.clone();
    std::thread::spawn(move || {
        let mut last = memory::signature(&root);
        loop {
            std::thread::sleep(POLL);
            if tx.is_closed() {
                return;
            }
            let now = memory::signature(&root);
            if now != last {
                last = now;
                if tx.unbounded_send(Message::Loaded(Arc::new(memory::load(&root, memory::today())))).is_err() {
                    return;
                }
            }
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_core::Request;

    #[test]
    fn deletes_a_fact_and_tells_the_app() {
        let root = std::env::temp_dir().join(format!("idetea-ext-memory-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join(memory::MEMORY_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "- [A](a.md) — a\n").unwrap();
        std::fs::write(dir.join("a.md"), "---\nname: a\nupdated: 2026-11-13\n---\nx\n").unwrap();
        let mut ext = MemoryExtension::new();
        let mut ctx = Context::new(&root, None);
        let _ = ext.activate(&mut ctx);
        assert_eq!(ext.memory.facts.len(), 1);
        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::Delete(dir.join("a.md")), &mut ctx);
        assert!(ext.memory.facts.is_empty());
        assert!(ctx.into_requests().iter().any(|r| matches!(r, Request::PathDeleted(_))));
        assert_eq!(std::fs::read_to_string(dir.join("README.md")).unwrap(), "");
        let _ = std::fs::remove_dir_all(&root);
    }
}
