//! Estrutura de um arquivo (funções, structs, métodos...), para o painel Estrutura
//! ([`CoreEvent::Outline`](crate::CoreEvent::Outline)). Vem do servidor de linguagem; a
//! visibilidade o núcleo tira do texto, porque o LSP não informa.

use iced::widget::{container, space, text};
use iced::{Color, Element, Theme};

use crate::icons::{self, Icon};

/// Tipo de símbolo (os do LSP, agrupados no que interessa mostrar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Module,
    Class,
    Struct,
    Interface,
    Enum,
    EnumMember,
    Method,
    Function,
    Constructor,
    Field,
    Property,
    Variable,
    Constant,
    TypeParameter,
    /// `impl` do Rust e afins: um bloco que agrupa métodos.
    Block,
    Other,
}

impl Kind {
    /// `SymbolKind` do LSP.
    pub fn from_lsp(kind: u32) -> Self {
        match kind {
            1..=4 => Kind::Module,
            5 => Kind::Class,
            6 => Kind::Method,
            7 => Kind::Property,
            8 => Kind::Field,
            9 => Kind::Constructor,
            10 => Kind::Enum,
            11 => Kind::Interface,
            12 => Kind::Function,
            13 => Kind::Variable,
            14 => Kind::Constant,
            19 => Kind::Block,
            22 => Kind::EnumMember,
            23 => Kind::Struct,
            26 => Kind::TypeParameter,
            _ => Kind::Other,
        }
    }

    /// Ícone e cor (as cores dos ícones de símbolo do VS Code, que funcionam no tema claro e no escuro).
    pub fn icon(self) -> (Icon, Option<Color>) {
        let purple = Color::from_rgb8(0xB1, 0x80, 0xD7);
        let orange = Color::from_rgb8(0xEE, 0x9D, 0x28);
        let blue = Color::from_rgb8(0x75, 0xBE, 0xFF);
        match self {
            Kind::Method | Kind::Function | Kind::Constructor => (Icon::SymbolMethod, Some(purple)),
            Kind::Class => (Icon::SymbolClass, Some(orange)),
            Kind::Struct => (Icon::SymbolStructure, Some(orange)),
            Kind::Enum => (Icon::SymbolEnum, Some(orange)),
            Kind::EnumMember => (Icon::SymbolEnumMember, Some(blue)),
            Kind::Interface => (Icon::SymbolInterface, Some(blue)),
            Kind::Field => (Icon::SymbolField, Some(blue)),
            Kind::Property => (Icon::SymbolProperty, Some(blue)),
            Kind::Variable => (Icon::SymbolVariable, Some(blue)),
            Kind::Constant => (Icon::SymbolConstant, Some(blue)),
            Kind::TypeParameter => (Icon::SymbolParameter, None),
            Kind::Module => (Icon::SymbolNamespace, None),
            Kind::Block | Kind::Other => (Icon::SymbolMisc, None),
        }
    }

    pub fn label(self) -> String {
        match self {
            Kind::Module => fl!("kind-module"),
            Kind::Class => fl!("kind-class"),
            Kind::Struct => fl!("kind-struct"),
            Kind::Interface => fl!("kind-interface"),
            Kind::Enum => fl!("kind-enum"),
            Kind::EnumMember => fl!("kind-variant"),
            Kind::Method => fl!("kind-method"),
            Kind::Function => fl!("kind-function"),
            Kind::Constructor => fl!("kind-constructor"),
            Kind::Field => fl!("kind-field"),
            Kind::Property => fl!("kind-property"),
            Kind::Variable => fl!("kind-variable"),
            Kind::Constant => fl!("kind-constant"),
            Kind::TypeParameter => fl!("kind-type-parameter"),
            Kind::Block => fl!("kind-block"),
            Kind::Other => fl!("kind-other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    /// Visível só em parte do código. O texto diz como: `pub(crate)`, `protected`, `pacote`...
    Restricted(String),
    Private,
    /// Linguagem sem regra conhecida, ou símbolo em que isso não se aplica (variáveis locais, `impl`).
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: Kind,
    /// Ex.: a assinatura.
    pub detail: Option<String>,
    pub visibility: Visibility,
    /// Onde o nome está. Começam em 1 (coluna em caracteres).
    pub line: usize,
    pub column: usize,
    /// Linhas que o símbolo ocupa (1-based, inclusivo), para saber em qual o cursor está.
    pub start_line: usize,
    pub end_line: usize,
    pub children: Vec<Symbol>,
}

/// Ícone de visibilidade (14 px de largura, vazio para `Unknown`), com o detalhe ao parar o mouse:
/// cadeado aberto verde (público), fechado vermelho (privado), chave amarela (restrito).
pub fn visibility_badge<'a, M: 'a>(v: &Visibility) -> Element<'a, M> {
    let (icon, tip, color): (Icon, String, fn(&Theme) -> Color) = match v {
        Visibility::Public => (Icon::Unlock, fl!("visibility-public"), |t| t.extended_palette().success.base.color),
        Visibility::Private => (Icon::Lock, fl!("visibility-private"), |t| t.extended_palette().danger.base.color),
        Visibility::Restricted(label) => (Icon::Key, label.clone(), |t| t.extended_palette().warning.base.color),
        Visibility::Unknown => return space::horizontal().width(14).into(),
    };
    let i = icons::icon(icon).size(13).style(move |t: &Theme| text::Style { color: Some(color(t)) });
    icons::with_tip(container(i).width(14), tip)
}
