//! Janela grande sobre a IDE (ajuda, configurações): fundo escurecido, clique fora fecha.

use iced::widget::{container, mouse_area, opaque};
use iced::{Color, Element, Fill, Theme};

pub fn modal<'a, M: Clone + 'a>(content: impl Into<Element<'a, M>>, max_width: f32, on_close: M) -> Element<'a, M> {
    let card = container(content).max_width(max_width).height(Fill).padding(16).style(|theme: &Theme| {
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
    // `opaque` no cartão: clicar dentro dele não conta como "clique fora".
    let scrim = container(opaque(card)).center_x(Fill).height(Fill).padding(40).style(|_: &Theme| container::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.35).into()),
        ..container::Style::default()
    });
    opaque(mouse_area(scrim).on_press(on_close))
}
