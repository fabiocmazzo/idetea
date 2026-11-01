//! Escolher da lista: a extensão manda os itens e a paleta da IDE mostra, com busca. O escolhido volta
//! como mensagem para a extensão (do tipo `Message` dela).
//!
//! ```ignore
//! ctx.pick(
//!     Pick::new("Trocar para a branch…")
//!         .item(Some(Icon::GitBranch), "main", "atual", Message::Switch("main".into()))
//!         .create("Criar branch", |name| Message::Create(name)),
//! );
//! ```

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::ExtMessage;
use crate::icons::Icon;

#[derive(Debug, Clone)]
pub struct PickItem {
    pub label: String,
    /// Texto menor à direita (data, "atual"...).
    pub detail: String,
    pub icon: Option<Icon>,
    pub action: ExtMessage,
}

/// O texto digitado vira uma ação (ex.: criar uma branch com esse nome).
#[derive(Clone)]
pub struct PickCreate {
    /// Começo da linha: "Criar branch" → `Criar branch “nome”`.
    pub label: String,
    make: Arc<dyn Fn(String) -> ExtMessage + Send + Sync>,
}

impl PickCreate {
    pub fn message(&self, text: String) -> ExtMessage {
        (self.make)(text)
    }
}

impl fmt::Debug for PickCreate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PickCreate").field("label", &self.label).finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct Pick {
    /// Texto do campo vazio.
    pub placeholder: String,
    pub items: Vec<PickItem>,
    pub create: Option<PickCreate>,
}

impl Pick {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self { placeholder: placeholder.into(), items: Vec::new(), create: None }
    }

    /// Item que manda `on_pick` (do tipo `Message` da própria extensão) de volta.
    pub fn item<M: Any + Send + Sync>(
        mut self,
        icon: Option<Icon>,
        label: impl Into<String>,
        detail: impl Into<String>,
        on_pick: M,
    ) -> Self {
        self.items.push(PickItem { label: label.into(), detail: detail.into(), icon, action: ExtMessage::new(on_pick) });
        self
    }

    /// Linha extra com o texto digitado (quando ele não é o nome de um item).
    pub fn create<M: Any + Send + Sync>(
        mut self,
        label: impl Into<String>,
        make: impl Fn(String) -> M + Send + Sync + 'static,
    ) -> Self {
        self.create = Some(PickCreate { label: label.into(), make: Arc::new(move |t| ExtMessage::new(make(t))) });
        self
    }
}
