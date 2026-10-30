//! Árvore de chamadas: guarda o item do servidor de cada nó (para pedir os filhos depois) e
//! converte as respostas do LSP nos nós do núcleo ([`tea_core::calls`]).

use std::collections::HashMap;

use tea_core::calls::{Direction, Node};
use tea_core::outline::Kind;
use tea_lsp::{Call, CallDirection, CallItem, Client};

#[derive(Default)]
pub struct CallTree {
    /// Item do servidor de cada nó da árvore atual (e quem respondeu por ele).
    nodes: HashMap<u64, (Client, CallItem)>,
    /// Ids nunca se repetem: resposta atrasada de uma árvore antiga não acha o pai e é ignorada.
    next: u64,
    /// Direção pedida no último comando (a raiz chega depois, em `CallRoots`).
    pub direction: Direction,
}

impl CallTree {
    /// Nova árvore: esquece os nós anteriores e devolve as raízes.
    pub fn set_roots(&mut self, client: &Client, items: Vec<CallItem>) -> Vec<Node> {
        self.nodes.clear();
        items
            .into_iter()
            .map(|item| {
                let id = self.add(client, item.clone());
                node(id, &item, item.selection.start, 0)
            })
            .collect()
    }

    /// Pede os filhos de um nó. `false` se o nó não é da árvore atual.
    pub fn request(&self, node: u64, direction: Direction) -> bool {
        let Some((client, item)) = self.nodes.get(&node) else { return false };
        client.calls(item, lsp_direction(direction), node);
        true
    }

    /// Filhos de `parent` que chegaram do servidor. `None` se o pai não é da árvore atual.
    pub fn children(&mut self, parent: u64, direction: CallDirection, calls: Vec<Call>) -> Option<Vec<Node>> {
        let client = self.nodes.get(&parent)?.0.clone();
        let mut nodes: Vec<Node> = calls
            .into_iter()
            .map(|call| {
                // Quem chama: o clique vai para a chamada. Quem é chamado: para a declaração dele.
                let at = match (direction, call.sites.first()) {
                    (CallDirection::Incoming, Some(site)) => site.start,
                    _ => call.item.selection.start,
                };
                let id = self.add(&client, call.item.clone());
                node(id, &call.item, at, call.sites.len())
            })
            .collect();
        nodes.sort_by(|a, b| (&a.path, a.line, a.column).cmp(&(&b.path, b.line, b.column)));
        Some(nodes)
    }

    fn add(&mut self, client: &Client, item: CallItem) -> u64 {
        self.next += 1;
        self.nodes.insert(self.next, (client.clone(), item));
        self.next
    }
}

pub fn lsp_direction(d: Direction) -> CallDirection {
    match d {
        Direction::Incoming => CallDirection::Incoming,
        Direction::Outgoing => CallDirection::Outgoing,
    }
}

pub fn core_direction(d: CallDirection) -> Direction {
    match d {
        CallDirection::Incoming => Direction::Incoming,
        CallDirection::Outgoing => Direction::Outgoing,
    }
}

fn node(id: u64, item: &CallItem, at: tea_lsp::Position, sites: usize) -> Node {
    Node {
        id,
        name: item.name.clone(),
        kind: Kind::from_lsp(item.kind),
        detail: item.detail.clone(),
        path: item.path.clone(),
        line: at.line as usize + 1,
        column: at.character as usize + 1,
        sites,
    }
}
