//! O que o núcleo conta para as extensões sobre a sessão de debug ([`CoreEvent::Debug`]).
//! O núcleo fala com o adaptador (gdb); o painel só mostra e pede.
//!
//! [`CoreEvent::Debug`]: crate::CoreEvent::Debug

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Idle,
    /// Compilando antes de iniciar.
    Building,
    Running,
    /// Parado (breakpoint, passo, pausa). `reason` já em português.
    Stopped { reason: String },
}

/// Um quadro da pilha de chamadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: i64,
    pub name: String,
    pub path: Option<PathBuf>,
    /// 1-based (0 = sem linha).
    pub line: usize,
}

/// Um grupo de variáveis de um quadro ("Locals", "Registers"...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub name: String,
    /// Pede as variáveis com [`Context::debug_variables`](crate::Context::debug_variables).
    pub reference: i64,
    /// O adaptador avisa que é caro buscar (ex.: registradores): não abrir sozinho.
    pub expensive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    /// > 0: tem filhos (struct, vetor...), pedidos com `debug_variables`.
    pub reference: i64,
}

#[derive(Debug, Clone)]
pub enum Event {
    Status(Status),
    /// Pilha de quando parou, e qual quadro está selecionado.
    Stack { frames: Vec<Frame>, selected: usize },
    /// Grupos de variáveis do quadro selecionado (substituem os anteriores).
    Scopes { frame_id: i64, scopes: Vec<Scope> },
    /// Resposta de `debug_variables` (vazia se falhou).
    Variables { reference: i64, variables: Vec<Variable> },
    /// Saída do programa, do adaptador ou da compilação (`category`: stdout, stderr, console, build).
    Output { category: String, text: String },
    /// Nova sessão: a saída antiga pode ser apagada.
    OutputCleared,
}
