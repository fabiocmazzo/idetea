//! Painel "Estrutura": funções, structs, métodos e campos do arquivo ativo, em árvore, com um
//! ícone de visibilidade (público, privado ou restrito, como `pub(crate)` e `protected`).
//! O símbolo onde o cursor está fica destacado; clique leva até ele.
//!
//! A lista vem do núcleo ([`CoreEvent::Outline`]), que pergunta ao servidor de linguagem e
//! descobre a visibilidade pelo texto.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{Column, button, column, row, scrollable, space, text};
use iced::{Element, Fill, Task, Theme};
use tea_core::icons::{self, Icon};
use tea_core::outline::Symbol;
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("outline");
/// Assinaturas compridas são cortadas na tela.
const MAX_DETAIL_CHARS: usize = 80;

#[derive(Debug, Clone)]
pub enum Message {
    Open { line: usize, column: usize },
    /// Abre/fecha um símbolo com filhos (pela chave, o caminho de nomes).
    Toggle(String),
    CollapseAll,
}

#[derive(Default)]
pub struct OutlineExtension {
    path: Option<PathBuf>,
    symbols: Arc<Vec<Symbol>>,
    note: Option<String>,
    /// Linha do cursor no arquivo ativo (1-based).
    cursor_line: usize,
    /// Símbolos fechados: (arquivo, caminho de nomes). Os outros ficam abertos.
    collapsed: HashSet<(PathBuf, String)>,
}

fn key(parent: &str, s: &Symbol) -> String {
    format!("{parent}/{}", s.name)
}

/// Chave do símbolo mais de dentro que contém a linha.
fn innermost(symbols: &[Symbol], line: usize, parent: &str) -> Option<String> {
    let s = symbols.iter().find(|s| s.start_line <= line && line <= s.end_line)?;
    let k = key(parent, s);
    innermost(&s.children, line, &k).or(Some(k))
}

fn one_line(s: &str) -> String {
    let line = s.lines().next().unwrap_or_default();
    match line.char_indices().nth(MAX_DETAIL_CHARS) {
        Some((i, _)) => format!("{}…", &line[..i]),
        None => line.to_string(),
    }
}

impl OutlineExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_open(&self, k: &str) -> bool {
        let Some(path) = &self.path else { return true };
        !self.collapsed.contains(&(path.clone(), k.to_string()))
    }

    fn push_rows<'a>(
        &'a self,
        mut list: Column<'a, Message>,
        symbols: &'a [Symbol],
        parent: &str,
        depth: u16,
        current: Option<&str>,
    ) -> Column<'a, Message> {
        for s in symbols {
            let k = key(parent, s);
            let open = self.is_open(&k);
            let lead: Element<'_, Message> = if s.children.is_empty() {
                space::horizontal().width(18).into()
            } else {
                let chevron = icons::icon(if open { Icon::ChevronDown } else { Icon::ChevronRight }).size(12);
                button(chevron).on_press(Message::Toggle(k.clone())).style(button::text).padding([0, 3]).into()
            };
            let (icon, color) = s.kind.icon();
            let kind_icon = icons::with_tip(icons::icon(icon).size(14).color_maybe(color), s.kind.label());
            let mut line = row![
                space::horizontal().width(f32::from(depth) * 14.0),
                lead,
                kind_icon,
                tea_core::outline::visibility_badge(&s.visibility),
                text(&s.name).size(13),
            ]
            .spacing(4)
            .align_y(iced::Center);
            if let Some(d) = &s.detail {
                line = line.push(text(one_line(d)).size(11).style(text::secondary));
            }
            let selected = current == Some(k.as_str());
            list = list.push(
                button(line)
                    .on_press(Message::Open { line: s.line, column: s.column })
                    .padding([1, 4])
                    .width(Fill)
                    .style(move |t: &Theme, st| {
                        let base = button::text(t, st);
                        if selected {
                            let p = t.extended_palette();
                            button::Style { background: Some(p.primary.weak.color.scale_alpha(0.35).into()), ..base }
                        } else {
                            base
                        }
                    }),
            );
            if open && !s.children.is_empty() {
                list = self.push_rows(list, &s.children, &k, depth + 1, current);
            }
        }
        list
    }

    /// Chaves de todos os símbolos com filhos.
    fn parents(symbols: &[Symbol], parent: &str, out: &mut Vec<String>) {
        for s in symbols.iter().filter(|s| !s.children.is_empty()) {
            let k = key(parent, s);
            Self::parents(&s.children, &k, out);
            out.push(k);
        }
    }
}

impl Extension for OutlineExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "outline"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        ctx.register_panel_hidden(PANEL, fl!("outline-panel"), Slot::Right);
        ctx.panel_icon(PANEL, Icon::ListTree);
        ctx.register_command("outline.toggle", fl!("cmd-outline-toggle"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::Outline { path, symbols, note } => {
                self.path = path.clone();
                self.symbols = symbols.clone();
                self.note = note.clone();
            }
            CoreEvent::EditorStatus(status) if status.path == self.path => self.cursor_line = status.line,
            _ => {}
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "outline.toggle" {
            ctx.toggle_panel(PANEL);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        let Some(path) = self.path.clone() else { return Task::none() };
        match message {
            Message::Open { line, column } => ctx.open_file_at(path, line, column),
            Message::Toggle(k) => {
                let entry = (path, k);
                if !self.collapsed.remove(&entry) {
                    self.collapsed.insert(entry);
                }
            }
            Message::CollapseAll => {
                let mut all = Vec::new();
                Self::parents(&self.symbols, "", &mut all);
                let any_open = all.iter().any(|k| self.is_open(k));
                for k in all {
                    let entry = (path.clone(), k);
                    if any_open {
                        self.collapsed.insert(entry);
                    } else {
                        self.collapsed.remove(&entry);
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let name = self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned());
        let header = row![
            text(name.unwrap_or_default()).size(12).style(text::secondary).width(Fill),
            icons::icon_button(
                Icon::CollapseAll,
                fl!("outline-collapse-all"),
                (!self.symbols.is_empty()).then_some(Message::CollapseAll)
            ),
        ]
        .align_y(iced::Center);
        if self.symbols.is_empty() {
            let note = self.note.clone().unwrap_or_else(|| fl!("outline-empty"));
            return column![header, text(note).size(12).style(text::secondary)].spacing(6).into();
        }
        let current = innermost(&self.symbols, self.cursor_line, "");
        let list = self.push_rows(column![].spacing(0), &self.symbols, "", 0, current.as_deref());
        column![header, scrollable(list).height(Fill)].spacing(4).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_core::outline::{Kind, Visibility};
    use tea_core::Request;

    fn sym(name: &str, start: usize, end: usize, children: Vec<Symbol>) -> Symbol {
        Symbol {
            name: name.into(),
            kind: Kind::Function,
            detail: None,
            visibility: Visibility::Public,
            line: start,
            column: 5,
            start_line: start,
            end_line: end,
            children,
        }
    }

    fn outline() -> Arc<Vec<Symbol>> {
        Arc::new(vec![
            sym("Conta", 1, 10, vec![sym("depositar", 2, 4, vec![]), sym("validar", 6, 9, vec![])]),
            sym("main", 12, 15, vec![]),
        ])
    }

    #[test]
    fn current_symbol_is_the_innermost() {
        let s = outline();
        assert_eq!(innermost(&s, 7, "").as_deref(), Some("/Conta/validar"));
        assert_eq!(innermost(&s, 5, "").as_deref(), Some("/Conta"), "entre dois métodos: a struct");
        assert_eq!(innermost(&s, 11, ""), None);
    }

    #[test]
    fn click_opens_and_collapse_all_toggles() {
        let root = PathBuf::from("/p");
        let mut ext = OutlineExtension::new();
        let mut ctx = Context::new(&root, None);
        let path = PathBuf::from("/p/a.rs");
        let _ = ext.on_event(&CoreEvent::Outline { path: Some(path.clone()), symbols: outline(), note: None }, &mut ctx);
        let _ = ext.update(Message::Open { line: 6, column: 5 }, &mut ctx);
        assert!(ctx.into_requests().iter().any(|r| matches!(r, Request::OpenFileAt { line: 6, column: 5, .. })));

        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::CollapseAll, &mut ctx);
        assert!(!ext.is_open("/Conta"));
        let _ = ext.update(Message::CollapseAll, &mut ctx);
        assert!(ext.is_open("/Conta"), "tudo fechado: o botão abre de novo");
        let _ = ext.update(Message::Toggle("/Conta".into()), &mut ctx);
        assert!(!ext.is_open("/Conta"));
    }
}
