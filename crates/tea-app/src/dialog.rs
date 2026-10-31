//! Diálogo modal: uma caixa no meio da janela, com o resto escurecido e bloqueado.

use iced::widget::{button, column, container, mouse_area, opaque, row, space, text};
use iced::{Color, Element, Fill, Theme};
use tea_core::{ButtonKind, Dialog, ExtMessage};

/// Quem abriu o diálogo e recebe a resposta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// Índice da extensão.
    Ext(usize),
    /// O próprio app (fechar aba, sair...).
    Core,
}

pub struct Open {
    pub dialog: Dialog,
    pub owner: Owner,
    /// Botão em destaque (Enter aciona).
    pub selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    /// `None` = fechar sem ação.
    Choose(Option<usize>),
}

impl Open {
    pub fn new(dialog: Dialog, owner: Owner) -> Self {
        let selected = dialog.default_button();
        Self { dialog, owner, selected }
    }

    /// Mensagem do botão `i` (ou `None` para cancelar).
    pub fn action(&self, i: Option<usize>) -> Option<ExtMessage> {
        i.and_then(|i| self.dialog.buttons.get(i)).and_then(|b| b.action.clone())
    }

    /// ←/→/Tab: anda pelos botões, dando a volta.
    pub fn move_selection(&mut self, delta: isize) {
        let n = self.dialog.buttons.len() as isize;
        if n == 0 {
            return;
        }
        let current = self.selected.map_or(if delta > 0 { -1 } else { 0 }, |i| i as isize);
        self.selected = Some((current + delta).rem_euclid(n) as usize);
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let buttons = row(self.dialog.buttons.iter().enumerate().map(|(i, b)| {
            let selected = self.selected == Some(i);
            let kind = b.kind;
            button(text(&b.label).size(13))
                .on_press(Msg::Choose(if b.action.is_some() { Some(i) } else { None }))
                .padding([5, 14])
                .style(move |theme: &Theme, status| {
                    let mut style = match kind {
                        ButtonKind::Primary => button::primary(theme, status),
                        ButtonKind::Danger => button::danger(theme, status),
                        ButtonKind::Secondary => button::secondary(theme, status),
                    };
                    style.border.radius = 5.0.into();
                    if selected {
                        // Anel de foco: mostra o que o Enter vai fazer.
                        style.border.width = 2.0;
                        style.border.color = theme.extended_palette().background.base.text;
                    }
                    style
                })
                .into()
        }))
        .spacing(8);

        let card = container(
            column![
                text(&self.dialog.title).size(16),
                text(&self.dialog.message).size(13),
                row![
                    text(fl!("dialog-hint")).size(11).style(text::secondary),
                    space::horizontal(),
                    buttons
                ]
                .align_y(iced::Center),
            ]
            .spacing(14),
        )
        .width(460)
        .padding(18)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.weak.color.into()),
                border: iced::Border { color: p.background.strong.color, width: 1.0, radius: 10.0.into() },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 30.0,
                },
                ..container::Style::default()
            }
        });

        // `opaque` na caixa: clicar dentro dela não conta como "clique fora".
        let scrim = container(opaque(card)).center(Fill).style(|_: &Theme| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.35).into()),
            ..container::Style::default()
        });
        opaque(mouse_area(scrim).on_press(Msg::Choose(None)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_navigation() {
        let d = Dialog::new("t", "m").cancel("Cancelar").button("Apagar", ButtonKind::Danger, 1u8).cancel("Outro");
        let mut open = Open::new(d, Owner::Core);
        assert_eq!(open.selected, Some(1), "Enter vai no primeiro botão que não é secundário");
        open.move_selection(1);
        assert_eq!(open.selected, Some(2));
        open.move_selection(1);
        assert_eq!(open.selected, Some(0), "dá a volta");
        open.move_selection(-1);
        assert_eq!(open.selected, Some(2));
        assert!(open.action(Some(1)).and_then(|m| m.downcast_ref::<u8>().copied()) == Some(1));
        assert!(open.action(Some(0)).is_none(), "cancelar não tem ação");
        assert!(open.action(None).is_none());
    }

    #[test]
    fn only_cancel_has_no_default() {
        let open = Open::new(Dialog::new("t", "m").cancel("Ok"), Owner::Core);
        assert_eq!(open.selected, None);
    }
}
