//! Painel "Chamadas": quem chama a função onde o cursor estava (e quem chama essas, e assim por
//! diante), ou o que ela chama. A árvore abre aos poucos: cada nó aberto pede os filhos ao núcleo
//! ([`Context::call_children`]), que pergunta ao servidor de linguagem. Clique leva até a chamada
//! (ou até a declaração, em "o que chama").

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use iced::widget::{Column, button, column, row, scrollable, space, text};
use iced::{Element, Fill, Task, Theme};
use tea_core::calls::{Direction, Event, Node};
use tea_core::icons::{self, Icon};
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("calls");
/// Assinaturas compridas são cortadas na tela.
const MAX_DETAIL_CHARS: usize = 70;

#[derive(Debug, Clone)]
pub enum Message {
    Open(u64),
    Toggle(u64),
    SetDirection(Direction),
}

#[derive(Default)]
pub struct CallsExtension {
    root: PathBuf,
    direction: Direction,
    roots: Vec<Node>,
    /// Filhos já recebidos de cada nó (vazio = ninguém chama / não chama nada).
    children: HashMap<u64, Vec<Node>>,
    /// Nós abertos.
    open: HashSet<u64>,
    /// Nós esperando a resposta do servidor.
    loading: HashSet<u64>,
    /// Último nó aberto no editor (fica destacado).
    current: Option<u64>,
}

fn one_line(s: &str) -> String {
    let line = s.lines().next().unwrap_or_default().trim();
    match line.char_indices().nth(MAX_DETAIL_CHARS) {
        Some((i, _)) => format!("{}…", &line[..i]),
        None => line.to_string(),
    }
}

impl CallsExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn rel<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, str> {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy()
    }

    fn find(&self, id: u64) -> Option<&Node> {
        self.roots.iter().chain(self.children.values().flatten()).find(|n| n.id == id)
    }

    fn expand(&mut self, id: u64, ctx: &mut Context) {
        self.open.insert(id);
        if !self.children.contains_key(&id) && self.loading.insert(id) {
            ctx.call_children(id, self.direction);
        }
    }

    /// Recomeça a árvore a partir das raízes (nova árvore ou direção trocada).
    fn restart(&mut self, ctx: &mut Context) {
        self.children.clear();
        self.open.clear();
        self.loading.clear();
        self.current = None;
        let ids: Vec<u64> = self.roots.iter().map(|n| n.id).collect();
        for id in ids {
            self.expand(id, ctx);
        }
    }

    fn push_rows<'a>(&'a self, mut list: Column<'a, Message>, nodes: &'a [Node], depth: u16) -> Column<'a, Message> {
        for n in nodes {
            let open = self.open.contains(&n.id);
            let loaded = self.children.get(&n.id);
            let lead: Element<'_, Message> = match loaded {
                Some(c) if c.is_empty() => space::horizontal().width(18).into(),
                _ => {
                    let chevron = icons::icon(if open { Icon::ChevronDown } else { Icon::ChevronRight }).size(12);
                    button(chevron).on_press(Message::Toggle(n.id)).style(button::text).padding([0, 3]).into()
                }
            };
            let (icon, color) = n.kind.icon();
            let mut line = row![
                space::horizontal().width(f32::from(depth) * 16.0),
                lead,
                icons::with_tip(icons::icon(icon).size(14).color_maybe(color), n.kind.label()),
                text(&n.name).size(13),
            ]
            .spacing(4)
            .align_y(iced::Center);
            if n.sites > 1 {
                line = line.push(text(format!("{}×", n.sites)).size(11).style(text::secondary));
            }
            if let Some(d) = &n.detail {
                line = line.push(text(one_line(d)).size(11).style(text::secondary));
            }
            line = line.push(text(format!("{}:{}", self.rel(&n.path), n.line)).size(11).style(text::secondary));
            let selected = self.current == Some(n.id);
            list = list.push(button(line).on_press(Message::Open(n.id)).padding([1, 4]).width(Fill).style(
                move |t: &Theme, st| {
                    let base = button::text(t, st);
                    if selected {
                        let p = t.extended_palette();
                        button::Style { background: Some(p.primary.weak.color.scale_alpha(0.35).into()), ..base }
                    } else {
                        base
                    }
                },
            ));
            if !open {
                continue;
            }
            let pad = space::horizontal().width(f32::from(depth + 1) * 16.0 + 22.0);
            match loaded {
                None => list = list.push(row![pad, text("…").size(12).style(text::secondary)]),
                Some(c) if c.is_empty() && depth == 0 => {
                    let note = match self.direction {
                        Direction::Incoming => "Ninguém chama esta função",
                        Direction::Outgoing => "Esta função não chama outras",
                    };
                    list = list.push(row![pad, text(note).size(12).style(text::secondary)]);
                }
                Some(c) => list = self.push_rows(list, c, depth + 1),
            }
        }
        list
    }
}

fn direction_button<'a>(label: &'a str, icon: Icon, d: Direction, active: Direction) -> Element<'a, Message> {
    let on = d == active;
    button(row![icons::icon(icon).size(13), text(label).size(12)].spacing(4).align_y(iced::Center))
        .on_press(Message::SetDirection(d))
        .padding([2, 8])
        .style(move |t: &Theme, st| {
            let base = button::text(t, st);
            if on {
                let p = t.extended_palette();
                button::Style { background: Some(p.primary.weak.color.scale_alpha(0.35).into()), ..base }
            } else {
                base
            }
        })
        .into()
}

impl Extension for CallsExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "calls"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, "Chamadas", Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::CallIncoming);
        ctx.register_command("calls.focus", "Chamadas: mostrar painel");
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        let CoreEvent::Calls(event) = event else { return Task::none() };
        match event {
            Event::Root { direction, roots } => {
                self.direction = *direction;
                self.roots = roots.clone();
                self.restart(ctx);
                ctx.show_panel(PANEL);
            }
            Event::Children { parent, direction, nodes } => {
                // Resposta de antes de trocar a direção (ou de uma árvore antiga): não serve.
                if *direction == self.direction && self.loading.remove(parent) {
                    self.children.insert(*parent, nodes.clone());
                }
            }
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "calls.focus" {
            ctx.show_panel(PANEL);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Open(id) => {
                if let Some(n) = self.find(id) {
                    ctx.open_file_at(n.path.clone(), n.line, n.column);
                    self.current = Some(id);
                }
            }
            Message::Toggle(id) => {
                if !self.open.remove(&id) {
                    self.expand(id, ctx);
                }
            }
            Message::SetDirection(d) => {
                if d != self.direction {
                    self.direction = d;
                    self.restart(ctx);
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        if self.roots.is_empty() {
            return text("Nenhuma busca ainda. Alt+Shift+H dentro de uma função mostra quem a chama.")
                .size(13)
                .style(text::secondary)
                .into();
        }
        let header = row![
            direction_button("Quem chama", Icon::CallIncoming, Direction::Incoming, self.direction),
            direction_button("O que chama", Icon::CallOutgoing, Direction::Outgoing, self.direction),
        ]
        .spacing(4);
        let list = self.push_rows(column![].spacing(0), &self.roots, 0);
        column![header, scrollable(list).height(Fill)].spacing(4).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_core::Request;
    use tea_core::outline::Kind;

    fn node(id: u64, name: &str, line: usize) -> Node {
        Node {
            id,
            name: name.into(),
            kind: Kind::Function,
            detail: None,
            path: "/p/a.rs".into(),
            line,
            column: 5,
            sites: 1,
        }
    }

    fn children_asked(ctx: Context) -> Vec<(u64, Direction)> {
        ctx.into_requests()
            .into_iter()
            .filter_map(|r| match r {
                Request::CallChildren { node, direction } => Some((node, direction)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn root_opens_and_asks_children_lazily() {
        let root = PathBuf::from("/p");
        let mut ext = CallsExtension::new();
        let mut ctx = Context::new(&root, None);
        let event = Event::Root { direction: Direction::Incoming, roots: vec![node(1, "soma", 1)] };
        let _ = ext.on_event(&CoreEvent::Calls(event), &mut ctx);
        assert_eq!(children_asked(ctx), [(1, Direction::Incoming)]);

        // Chegam quem chama; abrir um deles pede os filhos dele, uma vez só.
        let mut ctx = Context::new(&root, None);
        let kids = vec![node(2, "main", 6), node(3, "teste", 9)];
        let event = Event::Children { parent: 1, direction: Direction::Incoming, nodes: kids };
        let _ = ext.on_event(&CoreEvent::Calls(event), &mut ctx);
        let _ = ext.update(Message::Toggle(2), &mut ctx);
        let _ = ext.update(Message::Toggle(2), &mut ctx);
        let _ = ext.update(Message::Toggle(2), &mut ctx);
        assert_eq!(children_asked(ctx), [(2, Direction::Incoming)]);

        // Clique leva até a chamada.
        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::Open(3), &mut ctx);
        assert!(ctx.into_requests().iter().any(|r| matches!(r, Request::OpenFileAt { line: 9, .. })));
    }

    #[test]
    fn switching_direction_restarts_and_drops_stale_answers() {
        let root = PathBuf::from("/p");
        let mut ext = CallsExtension::new();
        let mut ctx = Context::new(&root, None);
        let event = Event::Root { direction: Direction::Incoming, roots: vec![node(1, "soma", 1)] };
        let _ = ext.on_event(&CoreEvent::Calls(event), &mut ctx);

        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::SetDirection(Direction::Outgoing), &mut ctx);
        // A resposta de "quem chama" chega atrasada: é ignorada.
        let stale = Event::Children { parent: 1, direction: Direction::Incoming, nodes: vec![node(2, "main", 6)] };
        let _ = ext.on_event(&CoreEvent::Calls(stale), &mut ctx);
        assert!(!ext.children.contains_key(&1));
        assert_eq!(children_asked(ctx), [(1, Direction::Outgoing)]);
    }
}
