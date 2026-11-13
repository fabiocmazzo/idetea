//! "Registrar como decisão" (RFC 0016, etapa 4): uma pergunta respondida no painel vira um ADR em
//! `docs/decisions/`, no modelo MADR, já com o contexto (os textos da pergunta), as opções, a escolha e o
//! comentário. Fica com `status: proposed`: a pessoa revisa, completa as consequências e aceita.

use serde_json::Value;

use crate::history::Entry;

/// Os títulos das seções, no idioma da interface.
pub struct Labels {
    pub context: String,
    pub options: String,
    pub decision: String,
    pub chosen: String,
    pub consequences: String,
    pub consequences_todo: String,
    pub origin: String,
}

/// O texto do ADR. `date`: `AAAA-MM-DD`.
pub fn text(entry: &Entry, date: &str, l: &Labels) -> String {
    let doc = &entry.document;
    let components: Vec<&Value> = doc.get("components").and_then(Value::as_array).map(|a| a.iter().collect()).unwrap_or_default();
    let kind = |c: &Value| c.get("component").and_then(Value::as_str).unwrap_or_default().to_string();
    let str_of = |c: &Value, k: &str| c.get(k).and_then(Value::as_str).map(str::to_string);

    let context: Vec<String> = components
        .iter()
        .filter(|c| matches!(kind(c).as_str(), "text" | "markdown"))
        .filter_map(|c| str_of(c, "text"))
        .filter(|t| !t.trim().is_empty())
        .collect();

    let option_label = |o: &Value| match o {
        Value::String(s) => s.clone(),
        o => o.get("label").or_else(|| o.get("value")).and_then(Value::as_str).unwrap_or_default().to_string(),
    };
    let mut options: Vec<String> = components
        .iter()
        .filter(|c| kind(c) == "select")
        .map(|c| {
            let list: Vec<String> = c.get("options").and_then(Value::as_array).map(|a| a.iter().map(option_label).collect()).unwrap_or_default();
            match str_of(c, "label") {
                Some(label) => format!("- **{label}**: {}", list.join(", ")),
                None => format!("- {}", list.join(", ")),
            }
        })
        .collect();
    let buttons: Vec<(String, String)> = components
        .iter()
        .filter(|c| kind(c) == "button")
        .filter_map(|c| {
            let name = c.get("action")?.get("name")?.as_str()?.to_string();
            Some((name, str_of(c, "label").unwrap_or_default()))
        })
        .collect();
    if options.is_empty() {
        options = buttons.iter().map(|(_, label)| format!("- {label}")).collect();
    }

    let mut decision = Vec::new();
    if let Some(r) = &entry.response {
        let clicked = buttons.iter().find(|(name, _)| *name == r.action).map_or(r.action.clone(), |(_, label)| label.clone());
        decision.push(format!("{}: **{clicked}**", l.chosen));
        let label_for = |pointer: &str| {
            components
                .iter()
                .find(|c| c.get("value").and_then(Value::as_str) == Some(pointer))
                .and_then(|c| str_of(c, "label").or_else(|| str_of(c, "id")))
                .unwrap_or_else(|| pointer.trim_start_matches('/').to_string())
        };
        for (pointer, value) in &r.values {
            let shown = match value {
                Value::Null => continue,
                Value::String(s) => s.clone(),
                Value::Array(a) => a.iter().map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string)).collect::<Vec<_>>().join(", "),
                v => v.to_string(),
            };
            decision.push(format!("- {}: {shown}", label_for(pointer)));
        }
        if let Some(c) = r.comment.as_deref().filter(|c| !c.trim().is_empty()) {
            decision.push(String::new());
            decision.extend(c.lines().map(|line| format!("> {line}")));
        }
    }

    let mut out = format!("---\nstatus: proposed\ndate: {date}\n---\n\n# {}\n\n## {}\n\n", entry.title(), l.context);
    out.push_str(&context.join("\n\n"));
    out.push_str(&format!("\n\n## {}\n\n{}\n\n## {}\n\n{}\n\n## {}\n\n{}\n\n---\n\n{}\n", l.options, options.join("\n"), l.decision, decision.join("\n"), l.consequences, l.consequences_todo, l.origin));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tea_declarative_ui::Response;

    fn labels() -> Labels {
        Labels {
            context: "Contexto".into(),
            options: "Opções".into(),
            decision: "Decisão".into(),
            chosen: "Escolhido".into(),
            consequences: "Consequências".into(),
            consequences_todo: "_A preencher._".into(),
            origin: "Da pergunta \"cache\".".into(),
        }
    }

    #[test]
    fn a_question_becomes_an_adr() {
        let doc = json!({"format": "idetea-ui", "version": 1, "revision": 1, "title": "Cache", "data": {},
            "components": [
                {"id": "q", "component": "markdown", "text": "Onde guardar o **cache**?"},
                {"id": "b", "component": "select", "label": "Backend", "value": "/backend", "options": ["sqlite", {"value": "json", "label": "Arquivo JSON"}]},
                {"id": "t", "component": "checkbox", "label": "Escrever testes", "value": "/tests"},
                {"id": "ok", "component": "button", "label": "Confirmar", "action": {"name": "ok"}},
                {"id": "no", "component": "button", "label": "Cancelar", "action": {"name": "cancel"}}
            ]});
        let mut values = std::collections::BTreeMap::new();
        values.insert("/backend".to_string(), json!("sqlite"));
        values.insert("/tests".to_string(), json!(true));
        values.insert("/nome".to_string(), Value::Null);
        let mut response = Response::new("cache", 1, 1, "e".into(), "ok".into(), "ok", "2026-11-13T10:00:00Z".into(), values);
        response.comment = Some("Mais simples.".into());
        let entry: Entry = serde_json::from_value(json!({
            "format": "idetea-ui-history", "version": 1, "name": "cache", "title": "Cache", "revision": 1,
            "outcome": "answered", "document": doc, "response": response
        }))
        .unwrap();
        let text = text(&entry, "2026-11-13", &labels());
        assert_eq!(
            text,
            "---\nstatus: proposed\ndate: 2026-11-13\n---\n\n# Cache\n\n## Contexto\n\nOnde guardar o **cache**?\n\n\
             ## Opções\n\n- **Backend**: sqlite, Arquivo JSON\n\n## Decisão\n\nEscolhido: **Confirmar**\n- Backend: sqlite\n\
             - Escrever testes: true\n\n> Mais simples.\n\n## Consequências\n\n_A preencher._\n\n---\n\nDa pergunta \"cache\".\n"
        );
    }
}
