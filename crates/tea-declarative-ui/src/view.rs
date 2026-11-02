//! Desenho de uma interface em iced: os componentes um embaixo do outro; botões seguidos ficam na mesma linha.

use iced::widget::{Column, button, checkbox, column, pick_list, row, text, text_input};
use iced::{Element, Fill};
use serde_json::Value;

use crate::document::{ButtonStyle, Choice, Component, Document, Kind, TextStyle};
use crate::state::State;

#[derive(Debug, Clone, PartialEq)]
pub enum UiMessage {
    /// Um campo mudou: grava `value` em `path` (ver [`State::set`]).
    Set { path: String, value: Value },
    /// O botão com este id foi clicado (ver [`State::press`]).
    Press(String),
}

/// A interface. Quem chama põe título, rolagem e o status em volta.
pub fn view<'a>(doc: &'a Document, state: &'a State) -> Element<'a, UiMessage> {
    let locked = state.locked(doc);
    let mut out: Column<'a, UiMessage> = column![].spacing(12).width(Fill);
    let mut i = 0;
    while i < doc.components.len() {
        // Botões seguidos: uma linha só (quebra se não couber).
        let buttons: Vec<&Component> =
            doc.components[i..].iter().take_while(|c| matches!(c.kind, Kind::Button { .. })).collect();
        if !buttons.is_empty() {
            i += buttons.len();
            let line = row(buttons.into_iter().map(|c| button_view(c, locked))).spacing(8).wrap().vertical_spacing(8);
            out = out.push(line);
            continue;
        }
        out = out.push(component_view(doc, state, &doc.components[i]));
        i += 1;
    }
    out.into()
}

fn button_view(c: &Component, locked: bool) -> Element<'_, UiMessage> {
    let Kind::Button { label, style, .. } = &c.kind else { return column![].into() };
    let style = match style {
        ButtonStyle::Primary => button::primary,
        ButtonStyle::Secondary => button::secondary,
        ButtonStyle::Danger => button::danger,
    };
    button(text(label)).style(style).on_press_maybe((!locked).then(|| UiMessage::Press(c.id.clone()))).into()
}

/// Um campo com o rótulo em cima (se houver).
fn labeled<'a>(label: Option<&'a str>, field: impl Into<Element<'a, UiMessage>>) -> Element<'a, UiMessage> {
    match label {
        Some(label) => column![text(label).size(13).style(text::secondary), field.into()].spacing(4).into(),
        None => field.into(),
    }
}

fn component_view<'a>(doc: &'a Document, state: &'a State, c: &'a Component) -> Element<'a, UiMessage> {
    let set = |path: &str| {
        let path = path.to_string();
        move |value: Value| UiMessage::Set { path: path.clone(), value }
    };
    match &c.kind {
        Kind::Text { text: content, style } => {
            let style = match style {
                TextStyle::Normal => text::base,
                TextStyle::Muted => text::secondary,
                TextStyle::Warning => text::warning,
                TextStyle::Error => text::danger,
            };
            text(content).style(style).width(Fill).into()
        }
        Kind::Checkbox { label, value } => {
            let checked = state.value(doc, value).and_then(Value::as_bool).unwrap_or(false);
            let set = set(value);
            checkbox(checked).label(label.as_str()).on_toggle(move |b| set(Value::Bool(b))).into()
        }
        Kind::Select { label, options, value } => {
            let current = state.value(doc, value).and_then(Value::as_str);
            let selected: Option<&Choice> = options.iter().find(|o| Some(o.value.as_str()) == current);
            let set = set(value);
            let list = pick_list(options.as_slice(), selected, move |o: Choice| set(Value::String(o.value)))
                .placeholder("—")
                .width(Fill);
            labeled(label.as_deref(), list)
        }
        Kind::Input { label, placeholder, value } => {
            let current = state.value(doc, value).and_then(Value::as_str).unwrap_or("");
            let set = set(value);
            // Id do componente no campo: o foco fica nele mesmo se o agente puser componentes acima.
            let input = text_input(placeholder, current)
                .id(format!("ui-input-{}", c.id))
                .on_input(move |s| set(Value::String(s)))
                .width(Fill);
            labeled(label.as_deref(), input)
        }
        Kind::Unknown { component } => {
            text(fl!("ui-unknown-component", component = component.clone())).size(13).style(text::warning).into()
        }
        Kind::Button { .. } => button_view(c, state.locked(doc)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Limits, parse};
    use iced_test::simulator;
    use serde_json::json;

    fn doc() -> Document {
        let v = json!({
            "format": "idetea-ui", "version": 1, "revision": 1,
            "data": { "nome": "Ana", "ok": false },
            "components": [
                { "id": "t", "component": "text", "text": "Confirme os dados" },
                { "id": "n", "component": "input", "label": "Nome", "value": { "path": "/nome" } },
                { "id": "c", "component": "checkbox", "label": "Concordo", "value": { "path": "/ok" } },
                { "id": "x", "component": "grafico" },
                { "id": "enviar", "component": "button", "label": "Enviar", "action": { "name": "enviar", "context": ["/nome", "/ok"] } },
                { "id": "cancelar", "component": "button", "label": "Cancelar", "style": "secondary", "action": { "name": "cancelar" } }
            ]
        });
        parse(v.to_string().as_bytes(), &Limits::default()).unwrap()
    }

    #[test]
    fn clicks_and_typing_become_messages() {
        let d = doc();
        let mut state = State::new(&d);
        let mut ui = simulator(view(&d, &state));
        assert!(ui.find("Confirme os dados").is_ok());
        let unknown = fl!("ui-unknown-component", component = "grafico");
        assert!(ui.find(unknown.as_str()).is_ok(), "aviso no lugar do componente desconhecido");
        ui.click("Concordo").unwrap();
        ui.click("Enviar").unwrap();
        let msgs: Vec<UiMessage> = ui.into_messages().collect();
        assert_eq!(msgs, vec![
            UiMessage::Set { path: "/ok".into(), value: json!(true) },
            UiMessage::Press("enviar".into()),
        ]);

        // Aplicando as mensagens: a resposta leva o que foi marcado.
        state.set("/ok", json!(true));
        let (name, values) = state.press(&d, "enviar").unwrap();
        assert_eq!(name, "enviar");
        assert_eq!(values["/ok"], json!(true));
        assert_eq!(values["/nome"], json!("Ana"));

        // Respondida (submit once): clicar de novo não gera nada.
        let mut ui = simulator(view(&d, &state));
        let _ = ui.click("Cancelar");
        assert_eq!(ui.into_messages().count(), 0, "botões desligados depois de responder");
    }
}
