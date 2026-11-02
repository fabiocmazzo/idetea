//! Interfaces declarativas: um documento JSON descreve uma interface pequena (textos, botões, caixas de
//! marcar, listas de escolha e campos de texto); este crate valida o documento, guarda o que o usuário mudou,
//! desenha a interface em iced e monta a resposta quando um botão é clicado.
//!
//! Não lê nem grava arquivos: quem faz isso é a extensão (`tea-ext-ui`). O formato está em
//! `docs/interfaces/formato.md` e as decisões em `docs/rfc/0001-interfaces-declarativas.md`.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod document;
pub mod pointer;
mod response;
mod state;
mod view;

pub use document::{
    Action, ButtonStyle, Choice, Component, Document, FORMAT, Kind, Limits, Submit, TextStyle, VERSION, parse,
};
pub use response::{RESPONSE_FORMAT, Response, file_name, timestamp};
pub use state::State;
pub use view::{UiMessage, view};
