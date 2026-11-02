//! Desenho de uma interface em iced: os componentes um embaixo do outro; botões seguidos ficam na mesma linha.

use iced::widget::{
    Column, button, checkbox, column, container, image, markdown, pick_list, row, stack, text, text_input,
};
use std::collections::HashMap;

use iced::{Element, Fill, Theme};
use serde_json::Value;

use crate::document::{ButtonStyle, Choice, Component, Document, Kind, Look, Pick, TextStyle};
use crate::media::{Media, display_size};
use crate::state::State;

#[derive(Debug, Clone, PartialEq)]
pub enum UiMessage {
    /// Um campo mudou: grava `value` em `path` (ver [`State::set`]).
    Set { path: String, value: Value },
    /// O botão com este id foi clicado (ver [`State::press`]).
    Press(String),
    /// Clique num componente `file`: abrir `path` (relativo à raiz) na linha e coluna (0 = início).
    Open { path: String, line: u64, column: u64 },
    /// Clique num link do Markdown (ver [`crate::follow`]).
    Link(String),
    /// O "comentário para o agente" mudou.
    Comment(String),
    /// Botão "copiar" de um bloco de código do Markdown: o texto vai para a área de transferência.
    Copy(String),
}

/// O Markdown de um documento já lido, por id do componente. Fica com quem desenha (as peças do iced não podem
/// ir entre threads, e o documento vai); refaça com [`Rendered::new`] quando o documento mudar.
#[derive(Default)]
pub struct Rendered(HashMap<String, Vec<markdown::Item>>);

impl Rendered {
    pub fn new(doc: &Document) -> Self {
        let parsed = doc.components.iter().filter_map(|c| match &c.kind {
            Kind::Markdown { text } => Some((c.id.clone(), markdown::parse(text).collect())),
            _ => None,
        });
        // Cada passo de um `steps` também é Markdown: chave `<componente>#<passo>`.
        let steps = doc.components.iter().flat_map(|c| match &c.kind {
            Kind::Steps { items, .. } => {
                items.iter().map(|s| (format!("{}#{}", c.id, s.id), markdown::parse(&s.text).collect())).collect()
            }
            _ => Vec::new(),
        });
        Self(parsed.chain(steps).collect())
    }
}

/// Markdown de agente: imagens viram o texto alternativo (nada é carregado do disco ou da rede).
struct Viewer;

impl<'a> markdown::Viewer<'a, UiMessage> for Viewer {
    fn on_link_click(url: markdown::Uri) -> UiMessage {
        UiMessage::Link(url)
    }

    fn image(
        &self,
        settings: markdown::Settings,
        _url: &'a markdown::Uri,
        _title: &'a str,
        alt: &markdown::Text,
    ) -> Element<'a, UiMessage> {
        let alt: String = alt.spans(settings.style).iter().map(|s| s.text.as_ref()).collect();
        container(text(alt).size(12)).padding(6).style(container::bordered_box).into()
    }

    /// O bloco do iced com um botão "copiar" no canto (comandos que o agente pede para rodar).
    fn code_block(
        &self,
        settings: markdown::Settings,
        _language: Option<&'a str>,
        code: &'a str,
        lines: &'a [markdown::Text],
    ) -> Element<'a, UiMessage> {
        let block = markdown::code_block(settings, lines, UiMessage::Link);
        let copy = button(text(fl!("ui-copy")).size(11))
            .padding([2, 8])
            .style(button::secondary)
            .on_press(UiMessage::Copy(code.trim_end().to_string()));
        stack![block, container(copy).width(Fill).align_right(Fill).padding(4)].into()
    }
}

/// A interface. Quem chama põe título, rolagem e o status em volta. `theme`: as cores do Markdown.
/// `comment`: quem chama grava o comentário na resposta (o painel Interfaces sim; o dos plugins, não).
/// `media`: as imagens do documento, já lidas por quem chama ([`Media::load`]).
pub fn view<'a>(
    doc: &'a Document,
    state: &'a State,
    rendered: &'a Rendered,
    media: &'a Media,
    theme: &Theme,
    comment: bool,
) -> Element<'a, UiMessage> {
    let settings = markdown::Settings::with_style(theme);
    let locked = state.locked(doc);
    let mut out: Column<'a, UiMessage> = column![].spacing(12).width(Fill);
    // O comentário fica logo antes dos botões do fim (ou no fim, se o documento não termina em botões).
    let has_buttons = doc.components.iter().any(|c| matches!(c.kind, Kind::Button { .. }));
    let comment_at = (comment && doc.comment && has_buttons).then(|| {
        doc.components.iter().rposition(|c| !matches!(c.kind, Kind::Button { .. })).map_or(0, |p| p + 1)
    });
    let comment_field = || -> Element<'a, UiMessage> {
        let input = text_input(&fl!("ui-comment-placeholder"), &state.comment)
            .id("ui-comment")
            .on_input_maybe((!locked).then_some(UiMessage::Comment as fn(String) -> UiMessage))
            .width(Fill);
        column![text(fl!("ui-comment-label")).size(13).style(text::secondary), input].spacing(4).into()
    };
    let mut i = 0;
    while i < doc.components.len() {
        if comment_at == Some(i) {
            out = out.push(comment_field());
        }
        // Botões seguidos: uma linha só (quebra se não couber).
        let buttons: Vec<&Component> =
            doc.components[i..].iter().take_while(|c| matches!(c.kind, Kind::Button { .. })).collect();
        if !buttons.is_empty() {
            i += buttons.len();
            let waiting = buttons.iter().map(|c| state.waiting(doc, &c.kind)).max().unwrap_or(0);
            let line = row(buttons.into_iter().map(|c| button_view(c, locked || state.waiting(doc, &c.kind) > 0)))
                .spacing(8)
                .wrap()
                .vertical_spacing(8);
            out = out.push(line);
            if waiting > 0 && !locked {
                out = out.push(text(fl!("ui-steps-missing", count = waiting)).size(12).style(text::secondary));
            }
            continue;
        }
        out = out.push(component_view(doc, state, rendered, media, &doc.components[i], settings));
        i += 1;
    }
    if comment_at == Some(doc.components.len()) {
        out = out.push(comment_field());
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

/// Uma imagem do documento: clique abre na aba de imagem. Sem a imagem lida, o motivo no lugar.
fn picture_view<'a>(media: &'a Media, path: &'a str, look: Look) -> Element<'a, UiMessage> {
    let p = match media.picture(path) {
        Some(Ok(p)) => p,
        Some(Err(e)) => return text(format!("{path}: {e}")).size(12).style(text::warning).into(),
        None => return text(fl!("ui-image-loading", path = path)).size(12).style(text::secondary).into(),
    };
    let (w, h) = display_size(p, look.height);
    let filter = if look.pixelated { image::FilterMethod::Nearest } else { image::FilterMethod::Linear };
    let img = image(p.handle.clone()).width(w as f32).height(h as f32).filter_method(filter);
    let shown: Element<'a, UiMessage> = match media.checker((w, h)) {
        Some(bg) if look.checker => stack![image(bg.clone()).width(w as f32).height(h as f32), img].into(),
        _ => img.into(),
    };
    button(shown)
        .padding(0)
        .style(button::text)
        .on_press(UiMessage::Open { path: path.to_string(), line: 0, column: 0 })
        .into()
}

fn component_view<'a>(
    doc: &'a Document,
    state: &'a State,
    rendered: &'a Rendered,
    media: &'a Media,
    c: &'a Component,
    settings: markdown::Settings,
) -> Element<'a, UiMessage> {
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
        Kind::File { path, line, column, label, .. } => {
            let place = if *line > 0 { format!("{path}:{line}") } else { path.clone() };
            let open = UiMessage::Open { path: path.clone(), line: *line, column: *column };
            let link = button(text(label.clone().unwrap_or_else(|| place.clone())).size(13))
                .style(button::secondary)
                .on_press(open);
            match label {
                Some(_) => column![link, text(place).size(12).style(text::secondary)].spacing(4).into(),
                None => link.into(),
            }
        }
        Kind::Markdown { text: plain } => match rendered.0.get(&c.id) {
            Some(items) => markdown::view_with(items, settings, &Viewer),
            None => text(plain).width(Fill).into(),
        },
        Kind::Image { path, label, look } => {
            let pic = picture_view(media, path, *look);
            let caption = text(label.clone().unwrap_or_else(|| path.clone())).size(12).style(text::secondary);
            column![pic, caption].spacing(4).into()
        }
        Kind::Gallery { items, pick, value, look } => {
            let chosen: Vec<String> = match (pick, value) {
                (Pick::One, Some(v)) => state.value(doc, v).and_then(Value::as_str).map(|s| vec![s.to_string()]).unwrap_or_default(),
                (Pick::Many, Some(v)) => state.checked(doc, v).into_iter().map(str::to_string).collect(),
                _ => Vec::new(),
            };
            let cells = items.iter().map(|item| {
                let label = item.label.clone().unwrap_or_else(|| item.id.clone());
                let on = chosen.contains(&item.id);
                let choose: Element<'a, UiMessage> = match (pick, value) {
                    (Pick::One, Some(path)) => {
                        // Uma escolha só: marcar troca a escolha; desmarcar a atual deixa sem nenhuma.
                        let value = if on { Value::Null } else { Value::String(item.id.clone()) };
                        let msg = UiMessage::Set { path: path.clone(), value };
                        checkbox(on).label(label).size(14).text_size(12).on_toggle(move |_| msg.clone()).into()
                    }
                    (Pick::Many, Some(path)) => {
                        // A lista inteira, na ordem dos itens.
                        let list: Vec<Value> = items
                            .iter()
                            .filter(|i| if i.id == item.id { !on } else { chosen.contains(&i.id) })
                            .map(|i| Value::String(i.id.clone()))
                            .collect();
                        let msg = UiMessage::Set { path: path.clone(), value: Value::Array(list) };
                        checkbox(on).label(label).size(14).text_size(12).on_toggle(move |_| msg.clone()).into()
                    }
                    _ => text(label).size(12).style(text::secondary).into(),
                };
                column![picture_view(media, &item.path, *look), choose].spacing(4).into()
            });
            row(cells).spacing(12).wrap().vertical_spacing(12).into()
        }
        Kind::Steps { items, value } => {
            let checked = state.checked(doc, value);
            let list = items.iter().enumerate().map(|(n, step)| {
                let on = checked.contains(&step.id.as_str());
                // Marcar ou desmarcar grava a lista inteira, na ordem dos passos.
                let toggled: Vec<Value> = items
                    .iter()
                    .filter(|s| if s.id == step.id { !on } else { checked.contains(&s.id.as_str()) })
                    .map(|s| Value::String(s.id.clone()))
                    .collect();
                let msg = UiMessage::Set { path: value.clone(), value: Value::Array(toggled) };
                let label: Element<'a, UiMessage> = match rendered.0.get(&format!("{}#{}", c.id, step.id)) {
                    Some(md) => markdown::view_with(md, settings, &Viewer),
                    None => text(&step.text).into(),
                };
                let number = checkbox(on).label(format!("{}.", n + 1)).on_toggle(move |_| msg.clone());
                let mut line = row![number, label]
                    .spacing(8)
                    .align_y(iced::Alignment::Start);
                if step.optional {
                    line = line.push(text(fl!("ui-step-optional")).size(12).style(text::secondary));
                }
                line.into()
            });
            column(list).spacing(6).width(Fill).into()
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

    /// Um `Media` vazio que vive o teste inteiro.
    fn no_media() -> &'static Media {
        Box::leak(Box::new(Media::default()))
    }

    fn doc() -> Document {
        let v = json!({
            "format": "idetea-ui", "version": 1, "revision": 1,
            "data": { "nome": "Ana", "ok": false },
            "components": [
                { "id": "t", "component": "text", "text": "Confirme os dados" },
                { "id": "n", "component": "input", "label": "Nome", "value": { "path": "/nome" } },
                { "id": "c", "component": "checkbox", "label": "Concordo", "value": { "path": "/ok" } },
                { "id": "x", "component": "grafico" },
                { "id": "f", "component": "file", "path": "notas/rascunho.md", "line": 4, "label": "Escreva aqui" },
                { "id": "md", "component": "markdown", "text": "Leia **com calma** e [abra o arquivo](src/a.rs:3)." },
                { "id": "cmd", "component": "markdown", "text": "Rode:\n\n```sh\ncargo test\n```\n" },
                { "id": "enviar", "component": "button", "label": "Enviar", "action": { "name": "enviar", "context": ["/nome", "/ok"] } },
                { "id": "cancelar", "component": "button", "label": "Cancelar", "style": "secondary", "action": { "name": "cancelar" } }
            ]
        });
        parse(v.to_string().as_bytes(), &Limits::default()).unwrap()
    }

    #[test]
    fn clicks_and_typing_become_messages() {
        let d = doc();
        let rendered = Rendered::new(&d);
        let mut state = State::new(&d);
        let mut ui = simulator(view(&d, &state, &rendered, no_media(), &Theme::Dark, true));
        assert!(ui.find("Confirme os dados").is_ok());
        let unknown = fl!("ui-unknown-component", component = "grafico");
        assert!(ui.find(unknown.as_str()).is_ok(), "aviso no lugar do componente desconhecido");
        assert!(ui.find("notas/rascunho.md:4").is_ok(), "o caminho embaixo do rótulo");
        assert_eq!(rendered.0["md"].len(), 1, "o Markdown foi lido (um parágrafo)");
        let comment = fl!("ui-comment-label");
        assert!(ui.find(comment.as_str()).is_ok(), "campo de comentário antes dos botões");
        ui.click(fl!("ui-copy").as_str()).unwrap();
        ui.click("Escreva aqui").unwrap();
        ui.click("Concordo").unwrap();
        ui.click("Enviar").unwrap();
        let msgs: Vec<UiMessage> = ui.into_messages().collect();
        assert_eq!(msgs, vec![
            UiMessage::Copy("cargo test".into()),
            UiMessage::Open { path: "notas/rascunho.md".into(), line: 4, column: 0 },
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
        let mut ui = simulator(view(&d, &state, &rendered, no_media(), &Theme::Dark, true));
        let _ = ui.click("Cancelar");
        assert_eq!(ui.into_messages().count(), 0, "botões desligados depois de responder");
    }

    #[test]
    fn comment_field_only_where_it_makes_sense() {
        let d = doc();
        let rendered = Rendered::new(&d);
        let state = State::new(&d);
        let comment = fl!("ui-comment-label");
        let shows = |d: &Document, host: bool| simulator(view(d, &state, &rendered, no_media(), &Theme::Dark, host)).find(comment.as_str()).is_ok();
        assert!(shows(&d, true));
        assert!(!shows(&d, false), "painel de plugin: sem comentário");
        let mut off = d.clone();
        off.comment = false;
        assert!(!shows(&off, true), "\"comment\": false");
        let mut no_buttons = d.clone();
        no_buttons.components.retain(|c| !matches!(c.kind, Kind::Button { .. }));
        assert!(!shows(&no_buttons, true), "sem botões não há o que mandar");
    }

    #[test]
    fn steps_hold_the_button_until_done() {
        let v = json!({
            "format": "idetea-ui", "version": 1, "revision": 1,
            "components": [
                { "id": "p", "component": "steps", "value": "/feitos", "items": [
                    { "id": "copiar", "text": "Clique em **Copiar**" },
                    { "id": "ensinar", "text": "Rode `ensinar`" },
                    { "id": "extra", "text": "Se quiser, reinicie", "optional": true } ] },
                { "id": "ok", "component": "button", "label": "Aprovado", "requires": "/feitos",
                  "action": { "name": "ok", "context": ["/feitos"] } },
                { "id": "falhou", "component": "button", "label": "Falhou", "action": { "name": "falhou" } }
            ]
        });
        let d = parse(v.to_string().as_bytes(), &Limits::default()).unwrap();
        assert!(d.warnings.is_empty(), "{:?}", d.warnings);
        let rendered = Rendered::new(&d);
        let mut state = State::new(&d);
        assert_eq!(state.missing(&d, "/feitos"), 2, "o opcional não conta");

        let mut ui = simulator(view(&d, &state, &rendered, no_media(), &Theme::Dark, true));
        let missing = fl!("ui-steps-missing", count = 2);
        assert!(ui.find(missing.as_str()).is_ok());
        let _ = ui.click("Aprovado");
        ui.click("Falhou").unwrap();
        assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![UiMessage::Press("falhou".into())], "Aprovado esperando");
        assert!(state.press(&d, "ok").is_none(), "nem pelo estado");

        // Marcar: a lista vai na ordem dos passos.
        state.set("/feitos", json!(["ensinar"]));
        let mut ui = simulator(view(&d, &state, &rendered, no_media(), &Theme::Dark, true));
        ui.click("1.").unwrap();
        let set = UiMessage::Set { path: "/feitos".into(), value: json!(["copiar", "ensinar"]) };
        assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![set]);
        state.set("/feitos", json!(["copiar", "ensinar"]));
        assert_eq!(state.missing(&d, "/feitos"), 0);
        let (_, values) = state.press(&d, "ok").unwrap();
        assert_eq!(values["/feitos"], json!(["copiar", "ensinar"]));

        // `requires` que não aponta um `steps`: aviso, e o botão não espera nada.
        let mut bad = v.clone();
        bad["components"][1]["requires"] = json!("/outro");
        let d = parse(bad.to_string().as_bytes(), &Limits::default()).unwrap();
        assert_eq!(d.warnings.len(), 1, "{:?}", d.warnings);
        assert!(State::new(&d).press(&d, "ok").is_some());
    }

    #[test]
    fn gallery_picks_and_images_open() {
        let root = std::env::temp_dir().join(format!("idetea-view-gallery-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for name in ["a.png", "b.png"] {
            ::image::RgbaImage::new(4, 4).save(root.join(name)).unwrap();
        }
        let v = json!({
            "format": "idetea-ui", "version": 1,
            "components": [
                { "id": "um", "component": "gallery", "select": "one", "value": "/um", "pixelated": true, "items": [
                    { "id": "a", "path": "a.png", "label": "Opção A" }, { "id": "b", "path": "b.png", "label": "Opção B" } ] },
                { "id": "varios", "component": "gallery", "select": "many", "value": "/varios", "items": [
                    { "id": "a", "path": "a.png", "label": "Ajustar A" }, { "id": "b", "path": "b.png", "label": "Ajustar B" } ] },
                { "id": "so", "component": "image", "path": "a.png", "label": "Só olhar", "background": "checker" },
                { "id": "falta", "component": "image", "path": "falta.png" }
            ]
        });
        let d = parse(v.to_string().as_bytes(), &Limits::default()).unwrap();
        assert!(d.warnings.is_empty(), "{:?}", d.warnings);
        let media: &'static Media =
            Box::leak(Box::new(Media::load(&root, &d, &crate::media::MediaLimits::default())));
        let mut state = State::new(&d);
        state.set("/varios", json!(["b"]));
        let rendered = Rendered::new(&d);
        let mut ui = simulator(view(&d, &state, &rendered, media, &Theme::Dark, true));
        assert!(ui.find("Só olhar").is_ok());
        assert!(ui.find("falta.png").is_ok(), "o caminho e o erro no lugar da imagem que falta");
        ui.click("Opção B").unwrap();
        ui.click("Ajustar A").unwrap();
        assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![
            UiMessage::Set { path: "/um".into(), value: json!("b") },
            UiMessage::Set { path: "/varios".into(), value: json!(["a", "b"]) },
        ]);
        let _ = std::fs::remove_dir_all(&root);
    }
}
