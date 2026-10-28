//! Ícones (fonte Codicons, embutida) e botões só com ícone + tooltip.
//!
//! O app carrega [`FONT_BYTES`] na inicialização; extensões só usam [`icon`] e [`icon_button`].

use std::time::Duration;

use iced::widget::{Text, button, container, text, tooltip};
use iced::{Background, Border, Color, Element, Font, Theme};

/// Arquivo da fonte (80 KB). Licença em `assets/NOTICE.md`.
pub const FONT_BYTES: &[u8] = include_bytes!("../assets/codicon.ttf");
/// Nome da família dentro do arquivo.
pub const FONT: Font = Font::with_name("codicon");

/// Tamanho padrão dos ícones (px).
pub const SIZE: f32 = 16.0;
/// Tempo com o mouse parado até o tooltip aparecer.
const TOOLTIP_DELAY: Duration = Duration::from_millis(450);

/// Ícones disponíveis. Para adicionar um, procure o código em `codicon.csv` do pacote `@vscode/codicons`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Add,
    Remove,
    NewFile,
    NewFolder,
    Edit,
    Trash,
    Refresh,
    CollapseAll,
    ChevronRight,
    ChevronDown,
    Folder,
    FolderOpened,
    File,
    GoToFile,
    SourceControl,
    GitCommit,
    Terminal,
    Search,
    Error,
    Warning,
    Info,
    Check,
    Close,
    ArrowUp,
    ArrowDown,
    Copy,
    LinkExternal,
}

impl Icon {
    pub fn char(self) -> char {
        let code = match self {
            Icon::Add => 0xEA60,
            Icon::Remove => 0xEB3B,
            Icon::NewFile => 0xEA7F,
            Icon::NewFolder => 0xEA80,
            Icon::Edit => 0xEA73,
            Icon::Trash => 0xEA81,
            Icon::Refresh => 0xEB37,
            Icon::CollapseAll => 0xEAC5,
            Icon::ChevronRight => 0xEAB6,
            Icon::ChevronDown => 0xEAB4,
            Icon::Folder => 0xEA83,
            Icon::FolderOpened => 0xEAF7,
            Icon::File => 0xEA7B,
            Icon::GoToFile => 0xEA94,
            Icon::SourceControl => 0xEA68,
            Icon::GitCommit => 0xEAFC,
            Icon::Terminal => 0xEA85,
            Icon::Search => 0xEA6D,
            Icon::Error => 0xEA87,
            Icon::Warning => 0xEA6C,
            Icon::Info => 0xEA74,
            Icon::Check => 0xEAB2,
            Icon::Close => 0xEA76,
            Icon::ArrowUp => 0xEAA1,
            Icon::ArrowDown => 0xEA9A,
            Icon::Copy => 0xEBCC,
            Icon::LinkExternal => 0xEB14,
        };
        char::from_u32(code).unwrap_or('?')
    }
}

/// O ícone como texto (herda a cor do tema; dá para mudar com `.style`).
pub fn icon<'a>(i: Icon) -> Text<'a> {
    text(i.char()).font(FONT).size(SIZE).line_height(1.0)
}

/// Botão pequeno só com ícone, fundo só no hover e tooltip com `tip`.
/// `on_press: None` desenha o ícone apagado (ação indisponível).
pub fn icon_button<'a, M: Clone + 'a>(i: Icon, tip: impl Into<String>, on_press: Option<M>) -> Element<'a, M> {
    let b = button(icon(i)).on_press_maybe(on_press).padding(3).style(icon_button_style);
    tooltip(b, container(text(tip.into()).size(12)).padding([3, 8]).style(tooltip_style), tooltip::Position::Bottom)
        .gap(4)
        .delay(TOOLTIP_DELAY)
        .into()
}

fn icon_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let base = button::Style {
        background: None,
        text_color: p.background.base.text,
        border: Border { radius: 4.0.into(), ..Border::default() },
        ..button::Style::default()
    };
    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style { background: Some(Background::Color(p.background.strong.color)), ..base },
        button::Status::Pressed => {
            button::Style { background: Some(Background::Color(p.background.strongest.color)), ..base }
        }
        button::Status::Disabled => button::Style { text_color: p.background.base.text.scale_alpha(0.3), ..base },
    }
}

fn tooltip_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(p.background.strong.color.into()),
        text_color: Some(p.background.strong.text),
        border: Border { color: p.background.strongest.color, width: 1.0, radius: 4.0.into() },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_is_embedded_and_icons_are_private_use() {
        assert!(FONT_BYTES.len() > 10_000);
        assert_eq!(&FONT_BYTES[..4], &[0, 1, 0, 0], "TrueType");
        for i in [Icon::Add, Icon::Trash, Icon::ArrowDown, Icon::SourceControl] {
            assert!(('\u{E000}'..='\u{F8FF}').contains(&i.char()), "{i:?}");
        }
    }
}
