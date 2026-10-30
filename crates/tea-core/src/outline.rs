//! Estrutura de um arquivo (funções, structs, métodos...), para o painel Estrutura
//! ([`CoreEvent::Outline`](crate::CoreEvent::Outline)). Vem do servidor de linguagem; a
//! visibilidade o núcleo tira do texto, porque o LSP não informa.

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

    pub fn label(self) -> &'static str {
        match self {
            Kind::Module => "módulo",
            Kind::Class => "classe",
            Kind::Struct => "struct",
            Kind::Interface => "interface/trait",
            Kind::Enum => "enum",
            Kind::EnumMember => "variante",
            Kind::Method => "método",
            Kind::Function => "função",
            Kind::Constructor => "construtor",
            Kind::Field => "campo",
            Kind::Property => "propriedade",
            Kind::Variable => "variável",
            Kind::Constant => "constante",
            Kind::TypeParameter => "parâmetro de tipo",
            Kind::Block => "bloco",
            Kind::Other => "símbolo",
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
