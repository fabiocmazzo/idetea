//! Árvore de chamadas: quem chama uma função, ou quem ela chama
//! ([`CoreEvent::Calls`](crate::CoreEvent::Calls)). Vem do servidor de linguagem e abre aos
//! poucos: cada nó aberto é um pedido novo ([`Context::call_children`](crate::Context::call_children)).

use std::path::PathBuf;

use crate::outline::Kind;

/// Quem chama a função, ou quem ela chama.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    #[default]
    Incoming,
    Outgoing,
}

/// Uma função na árvore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Identifica o nó nos pedidos de filhos. Vale até a próxima [`Event::Root`].
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    /// Ex.: a assinatura.
    pub detail: Option<String>,
    pub path: PathBuf,
    /// Onde o clique leva (começam em 1, coluna em caracteres): o lugar da chamada, se houver,
    /// senão o nome na declaração.
    pub line: usize,
    pub column: usize,
    /// Quantas chamadas há nesse lugar (0 na raiz).
    pub sites: usize,
}

#[derive(Debug, Clone)]
pub enum Event {
    /// Nova árvore, a partir da função onde o cursor estava. Substitui a anterior.
    Root { direction: Direction, roots: Vec<Node> },
    /// Filhos de `parent` (quem chama, ou quem é chamado, conforme `direction`).
    Children { parent: u64, direction: Direction, nodes: Vec<Node> },
}
