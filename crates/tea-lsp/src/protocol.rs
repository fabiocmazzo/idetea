//! Interpretação das respostas e notificações do servidor (sem I/O).
//! As posições aqui ainda estão na unidade do servidor.

use serde_json::Value;

use crate::text::{Position, Range};
use crate::{CompletionItem, Severity};

pub fn position(v: &Value) -> Option<Position> {
    Some(Position::new(v.get("line")?.as_u64()? as u32, v.get("character")?.as_u64()? as u32))
}

pub fn range(v: &Value) -> Option<Range> {
    Some(Range { start: position(v.get("start")?)?, end: position(v.get("end")?)? })
}

pub fn position_json(p: Position) -> Value {
    serde_json::json!({ "line": p.line, "character": p.character })
}

pub fn range_json(r: Range) -> Value {
    serde_json::json!({ "start": position_json(r.start), "end": position_json(r.end) })
}

/// Texto do hover em markdown. Aceita todos os formatos do protocolo.
pub fn hover_text(result: &Value) -> Option<String> {
    fn piece(v: &Value) -> Option<String> {
        match v {
            Value::String(s) => Some(s.clone()),
            Value::Object(o) => {
                let value = o.get("value")?.as_str()?;
                // MarkedString com linguagem vira bloco de código.
                match o.get("language").and_then(Value::as_str) {
                    Some(lang) => Some(format!("```{lang}\n{value}\n```")),
                    None => Some(value.to_string()),
                }
            }
            _ => None,
        }
    }
    let contents = result.get("contents")?;
    let text = match contents {
        Value::Array(items) => items.iter().filter_map(piece).collect::<Vec<_>>().join("\n\n"),
        other => piece(other)?,
    };
    let text = text.trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Itens de autocomplete. Snippets viram texto simples (`${1:x}` → `x`).
pub fn completion_items(result: &Value) -> Vec<CompletionItem> {
    let items = match result {
        Value::Array(a) => a,
        Value::Object(o) => match o.get("items") {
            Some(Value::Array(a)) => a,
            _ => return Vec::new(),
        },
        _ => return Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?.trim().to_string();
            let is_snippet = item.get("insertTextFormat").and_then(Value::as_u64) == Some(2);
            let raw = item
                .get("textEdit")
                .and_then(|e| e.get("newText"))
                .or_else(|| item.get("insertText"))
                .and_then(Value::as_str)
                .unwrap_or(&label)
                .to_string();
            let insert = if is_snippet { strip_snippet(&raw) } else { raw };
            let detail = item.get("detail").and_then(Value::as_str).map(str::to_string);
            Some(CompletionItem { label, insert, detail })
        })
        .collect()
}

/// `foo(${1:a}, $2)$0` → `foo(a, )`.
pub fn strip_snippet(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            }
            '$' => match chars.peek() {
                Some('{') => {
                    chars.next();
                    // Pula o número e o ':' e mantém o texto padrão até a '}' correspondente.
                    while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                        chars.next();
                    }
                    if chars.peek() == Some(&':') {
                        chars.next();
                    }
                    let mut depth = 1;
                    for c in chars.by_ref() {
                        match c {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        if depth > 0 {
                            out.push(c);
                        }
                    }
                }
                Some(d) if d.is_ascii_digit() => {
                    while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                        chars.next();
                    }
                }
                _ => out.push('$'),
            },
            other => out.push(other),
        }
    }
    out
}

/// Primeiro destino de uma resposta de definição: `(uri, range)`.
pub fn first_location(result: &Value) -> Option<(String, Range)> {
    locations(result).into_iter().next()
}

/// `Location`, `Location[]` ou `LocationLink[]` (resposta de definição, referências...).
pub fn locations(result: &Value) -> Vec<(String, Range)> {
    let list = match result {
        Value::Array(a) => a.iter().collect(),
        Value::Object(_) => vec![result],
        _ => Vec::new(),
    };
    list.into_iter()
        .filter_map(|loc| {
            // Location tem `uri`/`range`; LocationLink tem `targetUri`/`targetSelectionRange`.
            let uri = loc.get("uri").or_else(|| loc.get("targetUri"))?.as_str()?.to_string();
            let r = loc.get("range").or_else(|| loc.get("targetSelectionRange")).or_else(|| loc.get("targetRange"))?;
            Some((uri, range(r)?))
        })
        .collect()
}

pub fn text_edits(result: &Value) -> Vec<(Range, String)> {
    result
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|e| Some((range(e.get("range")?)?, e.get("newText")?.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

pub struct RawDiagnostic {
    pub range: Range,
    pub severity: Severity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

/// `textDocument/publishDiagnostics` → `(uri, diagnósticos)`.
pub fn diagnostics(params: &Value) -> Option<(String, Vec<RawDiagnostic>)> {
    let uri = params.get("uri")?.as_str()?.to_string();
    let list = params.get("diagnostics")?.as_array()?;
    let items = list
        .iter()
        .filter_map(|d| {
            Some(RawDiagnostic {
                range: range(d.get("range")?)?,
                severity: match d.get("severity").and_then(Value::as_u64) {
                    Some(1) => Severity::Error,
                    Some(2) | None => Severity::Warning,
                    Some(3) => Severity::Info,
                    _ => Severity::Hint,
                },
                message: d.get("message")?.as_str()?.to_string(),
                source: d.get("source").and_then(Value::as_str).map(str::to_string),
                code: match d.get("code") {
                    Some(Value::String(s)) => Some(s.clone()),
                    Some(Value::Number(n)) => Some(n.to_string()),
                    _ => None,
                },
            })
        })
        .collect();
    Some((uri, items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hover_formats() {
        assert_eq!(hover_text(&json!({"contents": {"kind": "markdown", "value": "**x**"}})).unwrap(), "**x**");
        assert_eq!(hover_text(&json!({"contents": "texto"})).unwrap(), "texto");
        assert_eq!(
            hover_text(&json!({"contents": [{"language": "rust", "value": "fn a()"}, "doc"]})).unwrap(),
            "```rust\nfn a()\n```\n\ndoc"
        );
        assert!(hover_text(&json!({"contents": ""})).is_none());
        assert!(hover_text(&Value::Null).is_none());
    }

    #[test]
    fn completion_list_and_snippets() {
        let items = completion_items(&json!({"isIncomplete": false, "items": [
            {"label": "push", "insertText": "push(${1:value})$0", "insertTextFormat": 2, "detail": "fn(&mut self, T)"},
            {"label": "len", "textEdit": {"range": {}, "newText": "len()"}},
            {"label": "x"},
            {"nope": 1}
        ]}));
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].insert, "push(value)");
        assert_eq!(items[1].insert, "len()");
        assert_eq!(items[2].insert, "x");
        assert_eq!(completion_items(&json!([{"label": "a"}])).len(), 1);
    }

    #[test]
    fn snippet_edge_cases() {
        assert_eq!(strip_snippet("a$1b$0"), "ab");
        assert_eq!(strip_snippet("${1:{x}}"), "{x}");
        assert_eq!(strip_snippet("\\$x ${2}"), "$x ");
        assert_eq!(strip_snippet("custo $"), "custo $");
    }

    #[test]
    fn locations() {
        let r = json!({"start": {"line": 1, "character": 2}, "end": {"line": 1, "character": 5}});
        assert_eq!(first_location(&json!({"uri": "file:///a", "range": r})).unwrap().0, "file:///a");
        let link = json!([{"targetUri": "file:///b", "targetRange": r, "targetSelectionRange": r}]);
        let (uri, range) = first_location(&link).unwrap();
        assert_eq!((uri.as_str(), range.start.line), ("file:///b", 1));
        assert!(first_location(&json!([])).is_none());
        assert!(first_location(&Value::Null).is_none());
        let many = json!([{"uri": "file:///a", "range": r}, {"uri": "file:///b", "range": r}, {"bad": 1}]);
        assert_eq!(super::locations(&many).len(), 2, "itens sem uri são ignorados");
    }

    #[test]
    fn diagnostics_parse() {
        let (uri, d) = diagnostics(&json!({"uri": "file:///x.rs", "diagnostics": [
            {"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
             "severity": 1, "message": "erro", "source": "rustc", "code": "E0308"},
            {"range": {"start": {"line": 2, "character": 0}, "end": {"line": 2, "character": 1}},
             "message": "sem severidade", "code": 42}
        ]}))
        .unwrap();
        assert_eq!(uri, "file:///x.rs");
        assert_eq!(d[0].severity, Severity::Error);
        assert_eq!(d[0].code.as_deref(), Some("E0308"));
        assert_eq!(d[1].severity, Severity::Warning);
        assert_eq!(d[1].code.as_deref(), Some("42"));
    }
}
