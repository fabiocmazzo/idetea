//! Binário da IDE TEA (spike da Fase 0).
//!
//! Uso: `idetea [pasta|arquivo]`

use std::path::{Path, PathBuf};
use std::time::Instant;

use iced::widget::{button, column, container, mouse_area, row, rule, text};
use iced::{Element, Fill, Subscription, Task, Theme, keyboard};
use tea_core::{AnyExtension, Context, CoreEvent, ExtMessage, PanelId, Request, Slot};
use tea_editor::{Editor, EditorMessage};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .theme(|_: &App| Theme::TokyoNight)
        .run()
}

/// Extensões ligadas nesta build. Cada uma entra por uma feature do cargo.
#[allow(unused_mut, clippy::vec_init_then_push)] // cada push depende de uma feature
fn builtin_extensions() -> Vec<Box<dyn AnyExtension>> {
    let mut list: Vec<Box<dyn AnyExtension>> = Vec::new();
    #[cfg(feature = "ext-demo")]
    list.push(Box::new(tea_ext_demo::DemoExtension::new()));
    list
}

struct Panel {
    ext: usize,
    id: PanelId,
    title: String,
    slot: Slot,
}

// `id` e `title` serão lidos pela command palette (Fase 1).
#[allow(dead_code)]
struct Command {
    id: String,
    title: String,
}

struct App {
    root: PathBuf,
    editors: Vec<Editor>,
    active: usize,
    extensions: Vec<Box<dyn AnyExtension>>,
    panels: Vec<Panel>,
    commands: Vec<Command>,
    left_visible: bool,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    Editor(usize, EditorMessage),
    Ext(usize, ExtMessage),
    SelectTab(usize),
    Command(String),
    PanelPressed,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let started = Instant::now();
        let arg = std::env::args().nth(1).map(PathBuf::from);
        let arg = arg.map(|p| p.canonicalize().unwrap_or(p));
        let (root, file) = match arg {
            Some(p) if p.is_file() => (p.parent().map(Path::to_path_buf).unwrap_or_default(), Some(p)),
            Some(p) => (p, None),
            None => (std::env::current_dir().unwrap_or_default(), None),
        };

        let mut app = Self {
            root,
            editors: vec![Editor::scratch("")],
            active: 0,
            extensions: builtin_extensions(),
            panels: Vec::new(),
            commands: vec![
                Command { id: "file.save".into(), title: "Salvar".into() },
                Command { id: "file.close".into(), title: "Fechar aba".into() },
                Command { id: "view.toggle_left".into(), title: "Mostrar/ocultar painel esquerdo".into() },
            ],
            left_visible: true,
            status: String::new(),
        };

        let mut tasks = Vec::new();
        for i in 0..app.extensions.len() {
            tasks.push(app.call_ext(i, |e, ctx| e.activate(ctx)));
        }
        if let Some(file) = file {
            tasks.push(app.open_file(file));
        }
        eprintln!("[idetea] boot em {:?}", started.elapsed());
        (app, Task::batch(tasks))
    }

    fn title(&self) -> String {
        format!("{} — IDE TEA", self.editors[self.active].title())
    }

    fn active_path(&self) -> Option<&Path> {
        self.editors.get(self.active).and_then(Editor::path)
    }

    // ---- Ponte com as extensões ------------------------------------------

    /// Chama uma extensão com um `Context` novo e executa os pedidos que ela fez.
    fn call_ext(
        &mut self,
        i: usize,
        f: impl FnOnce(&mut dyn AnyExtension, &mut Context) -> Task<ExtMessage>,
    ) -> Task<Message> {
        let active = self.editors.get(self.active).and_then(Editor::path);
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
                Request::RegisterPanel { id, title, slot } => {
                    self.panels.push(Panel { ext, id, title, slot });
                }
                Request::RegisterCommand { id, title } => self.commands.push(Command { id, title }),
                Request::RunCommand(id) => tasks.push(self.run_command(&id)),
                Request::Notify(msg) => self.status = msg,
            }
        }
        Task::batch(tasks)
    }

    fn broadcast(&mut self, event: CoreEvent) -> Task<Message> {
        let tasks: Vec<_> = (0..self.extensions.len())
            .map(|i| self.call_ext(i, |e, ctx| e.on_event(&event, ctx)))
            .collect();
        Task::batch(tasks)
    }

    // ---- Ações do núcleo ---------------------------------------------------

    fn open_file(&mut self, path: PathBuf) -> Task<Message> {
        if let Some(i) = self.editors.iter().position(|e| e.path() == Some(path.as_path())) {
            return self.set_active(i);
        }
        let started = Instant::now();
        match Editor::open(&path) {
            Ok(editor) => {
                self.status = format!("Aberto em {:.1?}", started.elapsed());
                // Troca a aba vazia inicial em vez de acumular abas "sem título".
                let replace_scratch = self.editors.len() == 1
                    && self.editors[0].path().is_none()
                    && !self.editors[0].is_modified();
                if replace_scratch {
                    self.editors[0] = editor;
                } else {
                    self.editors.push(editor);
                }
                let i = if replace_scratch { 0 } else { self.editors.len() - 1 };
                let opened = self.broadcast(CoreEvent::FileOpened(path));
                Task::batch([opened, self.set_active(i)])
            }
            Err(err) => {
                self.status = format!("Erro ao abrir {}: {err}", path.display());
                Task::none()
            }
        }
    }

    fn set_active(&mut self, i: usize) -> Task<Message> {
        self.active = i;
        self.editors[i].focus();
        let path = self.active_path().map(Path::to_path_buf);
        self.broadcast(CoreEvent::ActiveFileChanged(path))
    }

    fn run_command(&mut self, id: &str) -> Task<Message> {
        match id {
            "file.save" => {
                let editor = &mut self.editors[self.active];
                match editor.save() {
                    Ok(()) => {
                        let path = editor.path().map(Path::to_path_buf).unwrap_or_default();
                        self.status = format!("Salvo: {}", path.display());
                        self.broadcast(CoreEvent::FileSaved(path))
                    }
                    Err(err) => {
                        self.status = format!("Erro ao salvar: {err}");
                        Task::none()
                    }
                }
            }
            "file.close" => {
                // Fase 0: fecha sem perguntar, mesmo com alterações.
                self.editors.remove(self.active);
                if self.editors.is_empty() {
                    self.editors.push(Editor::scratch(""));
                }
                self.set_active(self.active.min(self.editors.len() - 1))
            }
            "view.toggle_left" => {
                self.left_visible = !self.left_visible;
                Task::none()
            }
            _ => {
                // Comandos de extensão: o prefixo antes do ponto é o id da extensão.
                let prefix = id.split('.').next().unwrap_or_default();
                match self.extensions.iter().position(|e| e.id() == prefix) {
                    Some(i) => self.call_ext(i, |e, ctx| e.on_command(id, ctx)),
                    None => {
                        self.status = format!("Comando desconhecido: {id}");
                        Task::none()
                    }
                }
            }
        }
    }

    // ---- Loop do iced ------------------------------------------------------

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(i, msg) => match self.editors.get_mut(i) {
                Some(editor) => editor.update(msg).map(move |m| Message::Editor(i, m)),
                None => Task::none(),
            },
            Message::Ext(i, msg) => {
                self.editors[self.active].lose_focus();
                self.call_ext(i, |e, ctx| e.update(msg, ctx))
            }
            Message::SelectTab(i) => self.set_active(i),
            Message::Command(id) => self.run_command(&id),
            Message::PanelPressed => {
                self.editors[self.active].lose_focus();
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // `listen_with` vê também as teclas que o editor consumiu.
        iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                shortcut(&key, modifiers).map(|c| Message::Command(c.to_string()))
            }
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Message> {
        let tabs = row(self.editors.iter().enumerate().map(|(i, e)| {
            let style = if i == self.active { button::primary } else { button::text };
            button(text(e.title()).size(13)).on_press(Message::SelectTab(i)).style(style).into()
        }))
        .spacing(4);

        let i = self.active;
        let editor = self.editors[i].view().map(move |m| Message::Editor(i, m));
        let center = column![tabs, editor].spacing(4).width(Fill).height(Fill);

        let mut body = row![].height(Fill);
        if self.left_visible
            && let Some(left) = self.slot_view(Slot::Left)
        {
            body = body.push(left).push(rule::vertical(1));
        }
        body = body.push(center);

        let (line, col) = self.editors[i].cursor();
        let status = row![
            text(format!("Ln {line}, Col {col}")).size(12),
            text(self.editors[i].syntax_name()).size(12),
            text(&self.status).size(12),
            iced::widget::space::horizontal(),
            text(format!("{} comandos", self.commands.len())).size(12),
        ]
        .spacing(16);

        column![body, rule::horizontal(1), status].padding(6).spacing(4).into()
    }

    /// Todos os painéis registrados num slot, um embaixo do outro.
    fn slot_view(&self, slot: Slot) -> Option<Element<'_, Message>> {
        let panels: Vec<_> = self.panels.iter().filter(|p| p.slot == slot).collect();
        if panels.is_empty() {
            return None;
        }
        let content = column(panels.into_iter().map(|p| {
            let ext = p.ext;
            let body = self.extensions[ext].view(p.id).map(move |m| Message::Ext(ext, m));
            column![text(&p.title).size(12), body].spacing(4).into()
        }));
        let panel = container(content).width(240).height(Fill).padding(4);
        Some(mouse_area(panel).on_press(Message::PanelPressed).into())
    }
}

/// Atalhos fixos da Fase 0. Na Fase 1 vêm do config.toml.
fn shortcut(key: &keyboard::Key, m: keyboard::Modifiers) -> Option<&'static str> {
    let keyboard::Key::Character(c) = key else { return None };
    if !m.command() {
        return None;
    }
    match (c.as_str(), m.shift()) {
        ("s", false) => Some("file.save"),
        ("w", false) => Some("file.close"),
        ("b", false) => Some("view.toggle_left"),
        ("R", true) | ("r", true) => Some("demo.refresh"),
        _ => None,
    }
}
