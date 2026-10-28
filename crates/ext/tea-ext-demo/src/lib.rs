//! Extensão de demonstração da Fase 0: lista os arquivos de uma pasta
//! num painel lateral e abre o arquivo clicado.
//!
//! Serve para validar a API de `tea-core`. Usa só a API pública,
//! como faria a futura extensão `filetree`.

use std::path::{Path, PathBuf};

use iced::widget::{button, column, scrollable, text};
use iced::{Element, Fill, Task};
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("files");

#[derive(Debug, Clone)]
pub enum Message {
    Clicked(PathBuf),
    Up,
}

#[derive(Default)]
pub struct DemoExtension {
    dir: PathBuf,
    root: PathBuf,
    entries: Vec<(PathBuf, bool)>,
    active: Option<PathBuf>,
}

impl DemoExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn refresh(&mut self) {
        self.entries = list_dir(&self.dir);
    }
}

fn list_dir(dir: &Path) -> Vec<(PathBuf, bool)> {
    let mut entries: Vec<(PathBuf, bool)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| {
            let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
            (e.path(), is_dir)
        })
        .collect();
    // Pastas primeiro, depois por nome.
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries
}

impl Extension for DemoExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "demo"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        self.dir = self.root.clone();
        self.refresh();
        ctx.register_panel(PANEL, "Arquivos", Slot::Left);
        ctx.register_command("demo.refresh", "Demo: recarregar lista");
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::ActiveFileChanged(path) => self.active = path.clone(),
            CoreEvent::FileSaved(_) => self.refresh(),
            CoreEvent::FileOpened(_) => {}
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "demo.refresh" {
            self.refresh();
            ctx.notify("Lista recarregada");
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Clicked(path) => {
                if path.is_dir() {
                    self.dir = path;
                    self.refresh();
                } else {
                    ctx.open_file(path);
                }
            }
            Message::Up => {
                if self.dir != self.root
                    && let Some(parent) = self.dir.parent()
                {
                    self.dir = parent.to_path_buf();
                    self.refresh();
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let mut list = column![].spacing(2);
        if self.dir != self.root {
            list = list.push(button(text("..")).on_press(Message::Up).style(button::text));
        }
        for (path, is_dir) in &self.entries {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let label = if *is_dir { format!("▸ {name}/") } else { name.into_owned() };
            let is_active = self.active.as_deref() == Some(path.as_path());
            let style = if is_active { button::secondary } else { button::text };
            list = list.push(
                button(text(label).size(14))
                    .width(Fill)
                    .on_press(Message::Clicked(path.clone()))
                    .style(style),
            );
        }
        scrollable(list).height(Fill).into()
    }
}
