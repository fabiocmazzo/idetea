//! Painel "Referências": onde um símbolo é usado (ou as implementações dele, ou várias
//! definições), agrupado por arquivo, com a linha de cada lugar e o nome destacado. Clique leva
//! até lá. A lista vem do núcleo ([`CoreEvent::Locations`]), que pergunta ao servidor de linguagem.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::path::{Path, PathBuf};

use iced::widget::{button, column, row, space, text};
use iced::{Element, Fill, Font, Task, Theme};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Extension, LocationKind, PanelId, Reference, Slot};

const PANEL: PanelId = PanelId("references");

#[derive(Debug, Clone)]
pub enum Message {
    Open(usize),
}

#[derive(Default)]
pub struct ReferencesExtension {
    root: PathBuf,
    kind: Option<LocationKind>,
    symbol: String,
    items: Vec<Reference>,
    /// Último uso aberto (fica destacado).
    current: Option<usize>,
}

impl ReferencesExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn rel<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, str> {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy()
    }

    fn files(&self) -> usize {
        let mut n = 0;
        let mut last: Option<&Path> = None;
        for r in &self.items {
            if last != Some(r.path.as_path()) {
                n += 1;
                last = Some(&r.path);
            }
        }
        n
    }
}

/// Prévia partida em antes / nome / depois (posições em caracteres).
fn split_preview(r: &Reference) -> (String, String, String) {
    let start = (r.column - 1).saturating_sub(r.indent);
    let chars: Vec<char> = r.preview.chars().collect();
    let start = start.min(chars.len());
    let end = (start + r.len).min(chars.len());
    (chars[..start].iter().collect(), chars[start..end].iter().collect(), chars[end..].iter().collect())
}

impl Extension for ReferencesExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "references"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("references-panel"), Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::References);
        ctx.register_command("references.focus", fl!("cmd-references-focus"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        if let CoreEvent::Locations { kind, symbol, items } = event {
            self.kind = Some(*kind);
            self.symbol = symbol.clone();
            self.items = items.clone();
            self.current = None;
            ctx.show_panel(PANEL);
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "references.focus" {
            ctx.show_panel(PANEL);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        let Message::Open(i) = message;
        if let Some(r) = self.items.get(i) {
            ctx.open_file_at(r.path.clone(), r.line, r.column);
            self.current = Some(i);
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        if self.items.is_empty() {
            return text(fl!("references-empty"))
                .size(13)
                .style(text::secondary)
                .into();
        }
        let n = self.items.len();
        let files = self.files();
        let summary = fl!(
            "references-summary",
            what = self.kind.unwrap_or(LocationKind::References).count_label(n),
            symbol = self.symbol.as_str(),
            count = files
        );
        let mut list = column![text(summary).size(12).style(text::secondary)].spacing(1);
        let mut last: Option<&Path> = None;
        for (i, r) in self.items.iter().enumerate() {
            if last != Some(r.path.as_path()) {
                last = Some(&r.path);
                let count = self.items.iter().filter(|x| x.path == r.path).count();
                list = list.push(
                    row![text(self.rel(&r.path)).size(13), text(format!("{count}")).size(11).style(text::secondary)]
                        .spacing(8)
                        .padding([4, 4]),
                );
            }
            let (before, name, after) = split_preview(r);
            let line = row![
                space::horizontal().width(14),
                text(format!("{:>5}", r.line)).size(11).font(Font::MONOSPACE).style(text::secondary),
                space::horizontal().width(8),
                text(before).size(12).font(Font::MONOSPACE),
                text(name).size(12).font(Font { weight: iced::font::Weight::Bold, ..Font::MONOSPACE }).style(
                    |t: &Theme| text::Style { color: Some(t.extended_palette().primary.base.color) }
                ),
                text(after).size(12).font(Font::MONOSPACE),
            ];
            let selected = self.current == Some(i);
            list = list.push(
                button(line)
                    .on_press(Message::Open(i))
                    .padding([1, 4])
                    .width(Fill)
                    .style(move |t: &Theme, s| {
                        let base = button::text(t, s);
                        if selected {
                            let p = t.extended_palette();
                            button::Style { background: Some(p.primary.weak.color.scale_alpha(0.35).into()), ..base }
                        } else {
                            base
                        }
                    }),
            );
        }
        tea_core::style::scrollable(list).height(Fill).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_core::Request;

    fn reference(path: &str, line: usize, column: usize) -> Reference {
        Reference { path: path.into(), line, column, len: 4, preview: "let soma = 1;".into(), indent: 4 }
    }

    #[test]
    fn highlights_the_name_in_the_preview() {
        // "    let soma = 1;": o nome começa na coluna 9 (1-based); a prévia perdeu 4 espaços.
        let (a, b, c) = split_preview(&reference("/p/a.rs", 1, 9));
        assert_eq!((a.as_str(), b.as_str(), c.as_str()), ("let ", "soma", " = 1;"));
        // Coluna fora da linha (arquivo mudou): não quebra.
        let (_, b, _) = split_preview(&reference("/p/a.rs", 1, 99));
        assert_eq!(b, "");
    }

    #[test]
    fn new_list_shows_panel_and_click_opens() {
        let root = PathBuf::from("/p");
        let mut ext = ReferencesExtension::new();
        let mut ctx = Context::new(&root, None);
        let items = vec![reference("/p/a.rs", 1, 9), reference("/p/a.rs", 5, 9), reference("/p/b.rs", 2, 9)];
        let event = CoreEvent::Locations { kind: LocationKind::Implementations, symbol: "soma".into(), items };
        let _ = ext.on_event(&event, &mut ctx);
        assert_eq!(LocationKind::Implementations.count_label(3), "3 implementações");
        assert_eq!(ext.files(), 2);
        let _ = ext.update(Message::Open(2), &mut ctx);
        let reqs = ctx.into_requests();
        assert!(reqs.iter().any(|r| matches!(r, Request::PanelVisibility { visible: Some(true), .. })));
        assert!(reqs.iter().any(|r| matches!(r, Request::OpenFileAt { line: 2, column: 9, .. })));
    }
}
