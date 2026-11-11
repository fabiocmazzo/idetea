//! Janela grande sobre a IDE (ajuda, configurações): fundo escurecido, clique fora fecha.

use iced::widget::{container, mouse_area, opaque};
use iced::{Color, Element, Fill};

pub fn modal<'a, M: Clone + 'a>(content: impl Into<Element<'a, M>>, max_width: f32, on_close: M) -> Element<'a, M> {
    let card = container(content).max_width(max_width).height(Fill).padding(16).style(tea_core::style::dialog);
    // `opaque` no cartão: clicar dentro dele não conta como "clique fora".
    // O fundo escurecido aparece aos poucos e o cartão desliza até o lugar (RFC 0015).
    let scrim = crate::appear::appear(container(opaque(card)).center_x(Fill).height(Fill).padding(40))
        .backdrop(Color::from_rgba(0.0, 0.0, 0.0, 0.35));
    opaque(mouse_area(scrim).on_press(on_close))
}
