//! Medidas e cores da UI num lugar só (RFC 0015): raios, vãos e as cores derivadas do tema para as ilhas.

use iced::widget::scrollable::{self, Rail, Scroller, Scrollable};
use iced::widget::{button, container};
use iced::{Border, Color, Element, Theme};

/// Raio dos cantos de uma ilha (painel, lado do editor).
pub const RADIUS_ISLAND: f32 = 10.0;
/// Vão entre duas ilhas; é também a área de arraste do divisor.
pub const GAP: f32 = 6.0;
/// Respiro dentro da ilha: o iced recorta em retângulo, e o conteúdo que pinta o próprio fundo (editor, terminal)
/// cobriria o canto arredondado. O arco passa a `r·(1 − 1/√2) ≈ 0,29·r` da quina; 0,3·r põe tudo dentro da curva.
pub const ISLAND_INSET: f32 = 3.0;
const _: () = assert!(ISLAND_INSET >= RADIUS_ISLAND * (1.0 - std::f32::consts::FRAC_1_SQRT_2));

/// Fundo da janela atrás das ilhas: o fundo do tema um pouco mais escuro (nos escuros) ou mais cinza (nos claros).
/// Nos temas quase pretos (Moonfly, Oxocarbon) não há como escurecer: o canvas fica um pouco mais claro.
pub fn canvas(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    let base = p.background.base.color;
    if !p.is_dark {
        mix(base, Color::from_rgb(0.5, 0.5, 0.5), 0.12)
    } else if luma(base) < 0.08 {
        mix(base, Color::WHITE, 0.07)
    } else {
        mix(base, Color::BLACK, 0.28)
    }
}

fn luma(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// A janela: só o fundo do canvas.
pub fn canvas_style(theme: &Theme) -> container::Style {
    container::Style::default().background(canvas(theme))
}

/// Uma ilha: fundo do tema e cantos arredondados. Nos temas claros, uma borda quase invisível ajuda o contraste
/// com o canvas, que é menor que nos escuros.
pub fn island(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    let border = if p.is_dark {
        Border { radius: RADIUS_ISLAND.into(), ..Border::default() }
    } else {
        Border { color: p.background.strong.color.scale_alpha(0.6), width: 1.0, radius: RADIUS_ISLAND.into() }
    };
    container::Style { background: Some(p.background.base.color.into()), border, ..container::Style::default() }
}

/// Barra de rolagem fina, afastada da borda: trilho de 8 px (a área que pega o mouse), polegar de 6 px, 2 px de
/// margem. Com espaço próprio (`spacing`): flutuando por cima, cobria o que fica à direita das linhas (atalhos na
/// paleta, letras do git na árvore).
pub fn scrollbar() -> scrollable::Scrollbar {
    scrollable::Scrollbar::new().width(8).scroller_width(6).margin(2).spacing(2)
}

/// Uma área rolável com a barra da IDE: vertical, fina, em pílula. Para outra direção, `.direction(…)` com
/// [`scrollbar`].
pub fn scrollable<'a, M>(content: impl Into<Element<'a, M>>) -> Scrollable<'a, M> {
    iced::widget::scrollable(content).direction(scrollable::Direction::Vertical(scrollbar())).style(scrollbar_style)
}

/// Barra em pílula sem trilho: quase some com o mouse fora da área, aparece com ele dentro e fica firme em cima do
/// polegar ou arrastando (efeito "aparece quando importa", sem timer).
pub fn scrollbar_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    use scrollable::Status;
    let p = theme.extended_palette();
    let (vertical, horizontal) = match status {
        Status::Active { .. } => (0.35, 0.35),
        Status::Hovered { is_vertical_scrollbar_hovered: v, is_horizontal_scrollbar_hovered: h, .. } => {
            (if v { 0.9 } else { 0.6 }, if h { 0.9 } else { 0.6 })
        }
        Status::Dragged { is_vertical_scrollbar_dragged: v, is_horizontal_scrollbar_dragged: h, .. } => {
            (if v { 1.0 } else { 0.6 }, if h { 1.0 } else { 0.6 })
        }
    };
    let thumb = p.background.strongest.color;
    let rail = |alpha: f32| Rail {
        background: None,
        border: Border::default(),
        // Raio maior que o polegar: o renderizador limita à metade do lado, e a ponta fica redonda.
        scroller: Scroller { background: thumb.scale_alpha(alpha).into(), border: iced::border::rounded(100) },
    };
    scrollable::Style {
        vertical_rail: rail(vertical),
        horizontal_rail: rail(horizontal),
        gap: None,
        ..scrollable::default(theme, status)
    }
}

/// A ilha que tem o foco do teclado: a mesma, com uma borda fina na cor de destaque.
pub fn island_focused(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    let border = Border { color: p.primary.base.color.scale_alpha(0.55), width: 1.0, radius: RADIUS_ISLAND.into() };
    container::Style { border, ..island(theme) }
}

/// Raio dos controles (abas, itens de lista, itens da barra de atividades, menus, botões de diálogo).
pub const RADIUS_CONTROL: f32 = 6.0;
/// Raio das peças miúdas (dicas, teclas, botões de ícone).
pub const RADIUS_SMALL: f32 = 4.0;
/// Altura do cabeçalho de painel (título e abas do painel de baixo).
pub const HEADER_HEIGHT: f32 = 24.0;

/// Sombra de menus e dicas.
pub const SHADOW_POPUP: iced::Shadow =
    iced::Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.35), offset: iced::Vector::new(0.0, 6.0), blur_radius: 16.0 };
/// Sombra das janelas sobre a IDE (diálogo, paleta, ajuda, configurações).
pub const SHADOW_DIALOG: iced::Shadow =
    iced::Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.45), offset: iced::Vector::new(0.0, 10.0), blur_radius: 30.0 };

/// Janela sobre a IDE (diálogo, paleta, ajuda, configurações): o mesmo raio das ilhas.
pub fn dialog(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.weak.color.into()),
        border: Border { color: p.background.strong.color, width: 1.0, radius: RADIUS_ISLAND.into() },
        shadow: SHADOW_DIALOG,
        ..container::Style::default()
    }
}

/// Menu de contexto e listas suspensas.
pub fn menu(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.weak.color.into()),
        border: Border { color: p.background.strong.color, width: 1.0, radius: RADIUS_CONTROL.into() },
        shadow: SHADOW_POPUP,
        ..container::Style::default()
    }
}

/// Dica ao parar o mouse.
pub fn tooltip(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.strong.color.into()),
        text_color: Some(p.background.strong.text),
        border: Border { color: p.background.strongest.color, width: 1.0, radius: RADIUS_SMALL.into() },
        shadow: iced::Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.25), offset: iced::Vector::new(0.0, 2.0), blur_radius: 6.0 },
        ..container::Style::default()
    }
}

/// Item de lista ou da árvore: pílula arredondada no hover e na seleção (não de ponta a ponta, em retângulo).
/// Um cartão clicável (histórico do painel, memória do projeto): a superfície do tema com borda, mais clara sob o
/// mouse.
pub fn card(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => p.background.strong.color,
        _ => p.background.weak.color,
    };
    button::Style {
        background: Some(background.into()),
        text_color: p.background.weak.text,
        border: iced::Border { color: p.background.strong.color, width: 1.0, radius: RADIUS_CONTROL.into() },
        ..button::Style::default()
    }
}

pub fn list_item(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let p = theme.extended_palette();
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let background = if selected {
        Some(mix(p.background.weak.color, p.primary.base.color, 0.22).into())
    } else if hovered {
        Some(p.background.weak.color.into())
    } else {
        None
    };
    let text_color = if matches!(status, button::Status::Disabled) {
        p.background.base.text.scale_alpha(0.35)
    } else {
        p.background.base.text
    };
    button::Style { background, text_color, border: Border { radius: RADIUS_CONTROL.into(), ..Border::default() }, ..button::Style::default() }
}

/// Como uma aba aparece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// A aba ativa do lado com o foco.
    Active,
    /// Aberta no outro lado do editor dividido, ou a aba escolhida de um painel.
    Shown,
    /// As demais.
    Idle,
}

/// Aba discreta: fundo um pouco mais claro na mostrada, borda fina na cor de destaque na ativa (como o item ativo
/// da barra de atividades), texto apagado nas demais.
pub fn tab(theme: &Theme, status: button::Status, kind: Tab) -> button::Style {
    let p = theme.extended_palette();
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let background = match kind {
        Tab::Active | Tab::Shown => Some(p.background.weak.color.into()),
        Tab::Idle if hovered => Some(p.background.weak.color.scale_alpha(0.6).into()),
        Tab::Idle => None,
    };
    let text_color = match kind {
        Tab::Active | Tab::Shown => p.background.base.text,
        Tab::Idle if hovered => p.background.base.text,
        Tab::Idle => p.background.base.text.scale_alpha(0.65),
    };
    let border = match kind {
        Tab::Active => Border { color: p.primary.base.color.scale_alpha(0.6), width: 1.0, radius: RADIUS_CONTROL.into() },
        _ => Border { radius: RADIUS_CONTROL.into(), ..Border::default() },
    };
    button::Style { background, text_color, border, ..button::Style::default() }
}

/// `a` misturada com `b` (`t` = 0 é só `a`, 1 é só `b`).
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t, a: a.a + (b.a - a.a) * t }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_canvas_stands_apart_from_the_islands_in_every_theme() {
        for theme in Theme::ALL {
            let p = theme.extended_palette();
            let (canvas, island) = (luma(canvas(theme)), luma(p.background.base.color));
            assert!((canvas - island).abs() > 0.02, "{theme}: canvas {canvas} × ilha {island}");
            if p.is_dark && island >= 0.08 {
                assert!(canvas < island, "{theme}: nos temas escuros o canvas é o mais escuro");
            }
        }
    }


    /// Toda área rolável da IDE usa a barra da IDE: um `scrollable(` solto voltaria à barra larga do iced.
    #[test]
    fn every_scrollable_uses_the_ide_scrollbar() {
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut stack = vec![crates.to_path_buf()];
        let mut bare = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().is_some_and(|n| n != "target") {
                        stack.push(path);
                    }
                } else if path.extension().is_some_and(|e| e == "rs") && !path.ends_with("tea-core/src/style.rs") {
                    let code = std::fs::read_to_string(&path).unwrap();
                    for (i, line) in code.lines().enumerate() {
                        let mut rest = line;
                        while let Some(at) = rest.find("scrollable(") {
                            let before = rest[..at].chars().last();
                            if !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '.') {
                                bare.push(format!("{}:{}", path.display(), i + 1));
                            }
                            rest = &rest[at + 1..];
                        }
                    }
                }
            }
        }
        assert!(bare.is_empty(), "use tea_core::style::scrollable em: {bare:#?}");
    }
}
