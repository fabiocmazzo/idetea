//! Menu de contexto aberto: desenho (no ponto do clique, sem sair da janela) e navegação.

use iced::widget::{button, column, container, mouse_area, opaque, pin, responsive, row, rule, space, text};
use iced::{Color, Element, Fill, Point, Size, Theme};
use tea_core::icons::icon;
use tea_core::keymap::Keymap;
use tea_core::menu::{Menu, MenuAction, MenuItem};

const WIDTH: f32 = 250.0;
const ITEM_HEIGHT: f32 = 26.0;
const SEPARATOR_HEIGHT: f32 = 9.0;
const PADDING: f32 = 4.0;
/// Distância mínima da borda da janela.
const MARGIN: f32 = 4.0;

pub struct Open {
    pub menu: Menu,
    /// Extensão que abriu (recebe as mensagens dos itens).
    pub owner: usize,
    /// Item em destaque pelo teclado.
    pub selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Choose(usize),
    Close,
}

impl Open {
    pub fn new(mut menu: Menu, owner: usize) -> Self {
        if matches!(menu.items.last(), Some(MenuItem::Separator)) {
            menu.items.pop();
        }
        Self { menu, owner, selected: None }
    }

    pub fn action(&self, i: usize) -> Option<MenuAction> {
        match self.menu.items.get(i) {
            Some(MenuItem::Action { action, .. }) => action.clone(),
            _ => None,
        }
    }

    /// ↑/↓: próximo item habilitado, dando a volta.
    pub fn move_selection(&mut self, delta: isize) {
        let n = self.menu.items.len() as isize;
        let mut i = self.selected.map_or(if delta > 0 { -1 } else { n }, |i| i as isize);
        for _ in 0..n {
            i = (i + delta).rem_euclid(n);
            if self.menu.enabled(i as usize) {
                self.selected = Some(i as usize);
                return;
            }
        }
    }

    fn height(&self) -> f32 {
        let items: f32 = self
            .menu
            .items
            .iter()
            .map(|i| if matches!(i, MenuItem::Separator) { SEPARATOR_HEIGHT } else { ITEM_HEIGHT })
            .sum();
        items + PADDING * 2.0 + 2.0
    }

    pub fn view<'a>(&'a self, keymap: &'a Keymap) -> Element<'a, Msg> {
        let position = self.menu.position;
        let height = self.height();
        // `responsive` dá o tamanho da janela no layout; o cartão é montado aqui dentro.
        let placed = responsive(move |size| {
            let p = clamp(position, Size::new(WIDTH, height), size);
            // `opaque`: clicar dentro do cartão não conta como "clique fora".
            pin(opaque(self.card(keymap))).x(p.x).y(p.y).into()
        });
        // Clique (esquerdo ou direito) fora do menu fecha.
        opaque(mouse_area(container(placed).width(Fill).height(Fill)).on_press(Msg::Close).on_right_press(Msg::Close))
    }

    fn card<'a>(&'a self, keymap: &'a Keymap) -> Element<'a, Msg> {
        let mut list = column![];
        for (i, item) in self.menu.items.iter().enumerate() {
            match item {
                MenuItem::Separator => {
                    list = list.push(container(rule::horizontal(1)).padding([4, 6]).height(SEPARATOR_HEIGHT));
                }
                MenuItem::Action { label, icon: ic, action, danger } => {
                    let hint = match action {
                        Some(MenuAction::Command(c)) => keymap.chord_for(c).map(|k| k.to_string()),
                        _ => None,
                    };
                    let enabled = action.is_some();
                    let ic: Element<'a, Msg> = match ic {
                        Some(i) => icon(*i).size(14).into(),
                        None => space::horizontal().width(14).into(),
                    };
                    let content = row![
                        container(ic).width(22),
                        text(label.as_str()).size(13).width(Fill),
                        text(hint.unwrap_or_default()).size(11).style(text::secondary),
                    ]
                    .align_y(iced::Center);
                    let (selected, danger) = (self.selected == Some(i), *danger);
                    list = list.push(
                        button(content)
                            .on_press_maybe(enabled.then_some(Msg::Choose(i)))
                            .width(Fill)
                            .height(ITEM_HEIGHT)
                            .padding([4, 8])
                            .style(move |t: &Theme, s| item_style(t, s, selected, danger)),
                    );
                }
            }
        }
        container(list)
            .width(WIDTH)
            .padding(PADDING)
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.weak.color.into()),
                    border: iced::Border { color: p.background.strong.color, width: 1.0, radius: 6.0.into() },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 16.0,
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }
}

fn item_style(theme: &Theme, status: button::Status, selected: bool, danger: bool) -> button::Style {
    let p = theme.extended_palette();
    let text_color = if danger { p.danger.base.color } else { p.background.base.text };
    let base = button::Style {
        background: None,
        text_color,
        border: iced::Border { radius: 4.0.into(), ..iced::Border::default() },
        ..button::Style::default()
    };
    let highlight = button::Style {
        background: Some(p.primary.weak.color.into()),
        text_color: if danger { text_color } else { p.primary.weak.text },
        ..base
    };
    match status {
        button::Status::Disabled => button::Style { text_color: text_color.scale_alpha(0.35), ..base },
        button::Status::Hovered | button::Status::Pressed => highlight,
        button::Status::Active if selected => highlight,
        button::Status::Active => base,
    }
}

/// Posição final: no ponto do clique, mas sem sair da janela.
/// Perto da borda direita/de baixo, o menu abre para a esquerda/para cima.
fn clamp(at: Point, menu: Size, window: Size) -> Point {
    let x = if at.x + menu.width + MARGIN > window.width { at.x - menu.width } else { at.x };
    let y = if at.y + menu.height + MARGIN > window.height { at.y - menu.height } else { at.y };
    let max_x = (window.width - menu.width - MARGIN).max(MARGIN);
    let max_y = (window.height - menu.height - MARGIN).max(MARGIN);
    Point::new(x.clamp(MARGIN, max_x), y.clamp(MARGIN, max_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_flips_near_edges() {
        let win = Size::new(1000.0, 800.0);
        let menu = Size::new(250.0, 200.0);
        assert_eq!(clamp(Point::new(100.0, 100.0), menu, win), Point::new(100.0, 100.0));
        assert_eq!(clamp(Point::new(900.0, 100.0), menu, win), Point::new(650.0, 100.0), "abre para a esquerda");
        assert_eq!(clamp(Point::new(100.0, 750.0), menu, win), Point::new(100.0, 550.0), "abre para cima");
        // Janela menor que o menu: fica colado na margem.
        assert_eq!(clamp(Point::new(50.0, 50.0), menu, Size::new(200.0, 150.0)), Point::new(MARGIN, MARGIN));
    }

    #[test]
    fn keyboard_skips_separators_and_disabled() {
        let menu = Menu::new(Point::ORIGIN)
            .item(None, "a", 1u8)
            .separator()
            .disabled(None, "b")
            .command(None, "c", "x.y")
            .separator();
        let mut open = Open::new(menu, 0);
        assert_eq!(open.menu.items.len(), 4, "separador no fim sai");
        open.move_selection(1);
        assert_eq!(open.selected, Some(0));
        open.move_selection(1);
        assert_eq!(open.selected, Some(3), "pula separador e desabilitado");
        open.move_selection(1);
        assert_eq!(open.selected, Some(0), "dá a volta");
        open.move_selection(-1);
        assert_eq!(open.selected, Some(3));
        assert!(matches!(open.action(3), Some(MenuAction::Command(c)) if c == "x.y"));
        assert!(open.action(2).is_none());
    }
}
