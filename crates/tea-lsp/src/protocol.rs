//! Interpretação das respostas e notificações do servidor (sem I/O).
//! As posições aqui ainda estão na unidade do servidor.

use serde_json::Value;

use crate::text::{Position, Range};
use crate::{CompletionItem, DocumentSymbol, Severity, SignatureHelp};

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

/// Dica de parâmetros. Parâmetro em texto é achado no `label` em ordem, depois do `(`: em `fn a(a: i32)`,
/// o `a` é o de dentro dos parênteses. Em posições `[início, fim]`, elas contam UTF-16.
pub fn signature_help(result: &Value) -> Option<SignatureHelp> {
    let signatures = result.get("signatures")?.as_array()?;
    if signatures.is_empty() {
        return None;
    }
    let as_index = |v: Option<&Value>| v.and_then(Value::as_u64).map(|n| n as usize);
    let index = as_index(result.get("activeSignature")).unwrap_or(0).min(signatures.len() - 1);
    let sig = &signatures[index];
    let label = sig.get("label")?.as_str()?.to_string();
    let params = sig.get("parameters").and_then(Value::as_array).cloned().unwrap_or_default();
    let active = as_index(sig.get("activeParameter")).or(as_index(result.get("activeParameter")));

    let mut from = label.find('(').map_or(0, |i| i + 1);
    let mut spans = Vec::new();
    for p in &params {
        let found = match p.get("label") {
            Some(Value::String(s)) if !s.is_empty() => label[from..].find(s.as_str()).map(|i| (from + i, from + i + s.len())),
            Some(Value::Array(a)) => match (a.first().and_then(Value::as_u64), a.get(1).and_then(Value::as_u64)) {
                (Some(s), Some(e)) => utf16_to_byte(&label, s as usize).zip(utf16_to_byte(&label, e as usize)),
                _ => None,
            },
            _ => None,
        };
        if let Some((_, end)) = found {
            from = end;
        }
        spans.push(found);
    }
    let chars = |byte: usize| label[..byte].chars().count();
    let current = active.and_then(|i| params.get(i).zip(spans.get(i).copied().flatten()));
    let doc = current.and_then(|(p, _)| p.get("documentation")).and_then(doc_text);
    Some(SignatureHelp {
        active: current.map(|(_, (s, e))| (chars(s), chars(e))),
        label,
        doc,
        index,
        count: signatures.len(),
    })
}

/// Caracteres da dica de parâmetros que o servidor anuncia (`triggerCharacters` e `retriggerCharacters`).
/// Servidor com a dica mas sem lista: `(` e `,`.
pub fn signature_triggers(caps: &Value) -> Vec<char> {
    let Some(provider) = caps.get("signatureHelpProvider").filter(|p| !p.is_null()) else { return Vec::new() };
    let mut chars: Vec<char> = ["triggerCharacters", "retriggerCharacters"]
        .iter()
        .filter_map(|k| provider.get(*k).and_then(Value::as_array))
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|s| s.chars().next())
        .collect();
    if chars.is_empty() {
        chars = vec!['(', ','];
    }
    chars.sort_unstable();
    chars.dedup();
    chars
}

/// Posição em UTF-16 → byte do texto. `None` se passar do fim.
fn utf16_to_byte(text: &str, units: usize) -> Option<usize> {
    let mut count = 0;
    for (byte, c) in text.char_indices() {
        if count >= units {
            return Some(byte);
        }
        count += c.len_utf16();
    }
    (count >= units).then_some(text.len())
}

/// Documentação (texto ou MarkupContent): o primeiro parágrafo numa linha, até 200 caracteres.
fn doc_text(v: &Value) -> Option<String> {
    let text = match v {
        Value::String(s) => s.as_str(),
        Value::Object(o) => o.get("value")?.as_str()?,
        _ => return None,
    };
    let first = text.trim().split("\n\n").next()?.split_whitespace().collect::<Vec<_>>().join(" ");
    if first.is_empty() {
        return None;
    }
    let short: String = first.chars().take(200).collect();
    Some(if first.chars().count() > 200 { format!("{short}…") } else { short })
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

/// Resposta de `textDocument/documentSymbol`: em árvore (`DocumentSymbol[]`) ou no formato antigo,
/// lista simples (`SymbolInformation[]`), que é aninhada aqui pelo trecho que cada um ocupa.
/// Posições ainda no formato do servidor.
pub fn document_symbols(result: &Value) -> Vec<DocumentSymbol> {
    let Some(items) = result.as_array() else { return Vec::new() };
    if items.iter().any(|i| i.get("location").is_some()) {
        let mut flat: Vec<DocumentSymbol> = items
            .iter()
            .filter_map(|i| {
                let r = range(i.get("location")?.get("range")?)?;
                Some(DocumentSymbol {
                    name: i.get("name")?.as_str()?.to_string(),
                    kind: i.get("kind").and_then(Value::as_u64).unwrap_or(0) as u32,
                    detail: None,
                    range: r,
                    selection: r,
                    children: Vec::new(),
                })
            })
            .collect();
        flat.sort_by_key(|s| (s.range.start, std::cmp::Reverse(s.range.end)));
        return nest(flat);
    }
    items.iter().filter_map(tree_symbol).collect()
}

fn tree_symbol(v: &Value) -> Option<DocumentSymbol> {
    let r = range(v.get("range")?)?;
    Some(DocumentSymbol {
        name: v.get("name")?.as_str()?.to_string(),
        kind: v.get("kind").and_then(Value::as_u64).unwrap_or(0) as u32,
        detail: v.get("detail").and_then(Value::as_str).filter(|d| !d.is_empty()).map(str::to_string),
        range: r,
        selection: v.get("selectionRange").and_then(range).unwrap_or(r),
        children: v.get("children").and_then(Value::as_array).map(|c| c.iter().filter_map(tree_symbol).collect()).unwrap_or_default(),
    })
}

/// Aninha uma lista ordenada pelo começo (e, no empate, o maior antes): cada um vira filho
/// do último que o contém.
fn nest(sorted: Vec<DocumentSymbol>) -> Vec<DocumentSymbol> {
    fn insert(list: &mut Vec<DocumentSymbol>, s: DocumentSymbol) {
        match list.last_mut() {
            Some(last) if last.range.start <= s.range.start && s.range.end <= last.range.end => {
                insert(&mut last.children, s)
            }
            _ => list.push(s),
        }
    }
    let mut roots = Vec::new();
    for s in sorted {
        insert(&mut roots, s);
    }
    roots
}

/// Resposta de `workspace/symbol` (`SymbolInformation[]` ou `WorkspaceSymbol[]`):
/// (nome, kind, container, uri, posição no formato do servidor). `WorkspaceSymbol` sem trecho
/// (só a uri) aponta para o começo do arquivo.
pub fn workspace_symbols(result: &Value) -> Vec<(String, u32, Option<String>, String, Position)> {
    let Some(items) = result.as_array() else { return Vec::new() };
    items
        .iter()
        .filter_map(|i| {
            let loc = i.get("location")?;
            let uri = loc.get("uri")?.as_str()?.to_string();
            let at = loc.get("range").and_then(range).map_or(Position::default(), |r| r.start);
            Some((
                i.get("name")?.as_str()?.to_string(),
                i.get("kind").and_then(Value::as_u64).unwrap_or(0) as u32,
                i.get("containerName").and_then(Value::as_str).filter(|c| !c.is_empty()).map(str::to_string),
                uri,
                at,
            ))
        })
        .collect()
}

/// Um `CallHierarchyItem` cru: (nome, tipo, detalhe, uri, range, selectionRange). Posições do servidor.
pub type RawCallItem = (String, u32, Option<String>, String, Range, Range);

pub fn call_item(v: &Value) -> Option<RawCallItem> {
    Some((
        v.get("name")?.as_str()?.to_string(),
        v.get("kind").and_then(Value::as_u64).unwrap_or(0) as u32,
        v.get("detail").and_then(Value::as_str).filter(|d| !d.is_empty()).map(str::to_string),
        v.get("uri")?.as_str()?.to_string(),
        range(v.get("range")?)?,
        range(v.get("selectionRange")?)?,
    ))
}

/// Resposta de `callHierarchy/incomingCalls` (`from`) ou `outgoingCalls` (`to`): o item (cru, para
/// os próximos pedidos) e os trechos das chamadas (`fromRanges`).
pub fn calls(result: &Value, field: &str) -> Vec<(Value, Vec<Range>)> {
    let Some(items) = result.as_array() else { return Vec::new() };
    items
        .iter()
        .filter_map(|c| {
            let item = c.get(field)?.clone();
            let ranges = c.get("fromRanges").and_then(Value::as_array).map_or(Vec::new(), |rs| {
                rs.iter().filter_map(range).collect()
            });
            Some((item, ranges))
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

/// `WorkspaceEdit` → `(uri, edições)` por arquivo. Aceita `changes` e `documentChanges` (só as edições
/// de texto: criar, renomear ou apagar arquivos fica de fora). Posições ainda no formato do servidor.
pub fn workspace_edit(result: &Value) -> Vec<(String, Vec<(Range, String)>)> {
    let mut out: Vec<(String, Vec<(Range, String)>)> = Vec::new();
    if let Some(changes) = result.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            let (Some(uri), Some(edits)) = (
                change.get("textDocument").and_then(|d| d.get("uri")).and_then(Value::as_str),
                change.get("edits"),
            ) else {
                continue;
            };
            out.push((uri.to_string(), text_edits(edits)));
        }
    } else if let Some(changes) = result.get("changes").and_then(Value::as_object) {
        for (uri, edits) in changes {
            out.push((uri.clone(), text_edits(edits)));
        }
    }
    out.retain(|(_, e)| !e.is_empty());
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
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

    #[test]
    fn signature_help_shapes() {
        use serde_json::json;
        let active = |h: &SignatureHelp| h.active.map(|(s, e)| h.label.chars().skip(s).take(e - s).collect::<String>());
        // Parâmetros em texto: `a` acha o de dentro dos parênteses, não o nome da função.
        let r = json!({ "signatures": [{ "label": "fn a(a: i32, b: ç) -> i32",
            "parameters": [{ "label": "a: i32" }, { "label": "b: ç", "documentation": { "kind": "markdown", "value": "O segundo.\n\nMais texto." } }] }],
            "activeParameter": 1 });
        let h = signature_help(&r).unwrap();
        assert_eq!(active(&h).as_deref(), Some("b: ç"));
        assert_eq!(h.doc.as_deref(), Some("O segundo."));
        assert_eq!((h.index, h.count), (0, 1));
        // Posições em UTF-16, `activeParameter` da assinatura vence o de fora; sobrecargas.
        let r = json!({ "activeSignature": 1, "activeParameter": 0, "signatures": [
            { "label": "f()" },
            { "label": "f(ç: int, x: int)", "activeParameter": 1, "parameters": [{ "label": [2, 8] }, { "label": [10, 16] }] }] });
        let h = signature_help(&r).unwrap();
        assert_eq!(active(&h).as_deref(), Some("x: int"));
        assert_eq!((h.index, h.count), (1, 2));
        assert_eq!(signature_help(&Value::Null), None);
        assert_eq!(signature_help(&json!({ "signatures": [] })), None);

        assert_eq!(signature_triggers(&json!({ "signatureHelpProvider": { "triggerCharacters": ["(", ","], "retriggerCharacters": [")"] } })), ['(', ')', ',']);
        assert_eq!(signature_triggers(&json!({ "signatureHelpProvider": {} })), ['(', ',']);
        assert!(signature_triggers(&json!({})).is_empty());
    }

    #[test]
    fn workspace_edit_both_shapes() {
        let edit = |l: u32, c: u32, t: &str| json!({ "range": { "start": { "line": l, "character": c }, "end": { "line": l, "character": c + 3 } }, "newText": t });
        let changes = json!({ "changes": {
            "file:///b.rs": [edit(1, 4, "novo")],
            "file:///a.rs": [edit(0, 0, "novo"), edit(5, 2, "novo")],
            "file:///vazio.rs": []
        }});
        let got = workspace_edit(&changes);
        assert_eq!(got.iter().map(|(u, e)| (u.as_str(), e.len())).collect::<Vec<_>>(), [("file:///a.rs", 2), ("file:///b.rs", 1)]);

        let doc = json!({ "documentChanges": [
            { "textDocument": { "uri": "file:///a.rs", "version": 3 }, "edits": [edit(2, 1, "x")] },
            { "kind": "rename", "oldUri": "file:///c.rs", "newUri": "file:///d.rs" }
        ]});
        let got = workspace_edit(&doc);
        assert_eq!(got.len(), 1, "operações de arquivo ficam de fora");
        assert_eq!(got[0].1[0].1, "x");
        assert!(workspace_edit(&Value::Null).is_empty());
    }

    fn first(result: &Value) -> Option<(String, Range)> {
        super::locations(result).into_iter().next()
    }
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
        assert_eq!(first(&json!({"uri": "file:///a", "range": r})).unwrap().0, "file:///a");
        let link = json!([{"targetUri": "file:///b", "targetRange": r, "targetSelectionRange": r}]);
        let (uri, range) = first(&link).unwrap();
        assert_eq!((uri.as_str(), range.start.line), ("file:///b", 1));
        assert!(first(&json!([])).is_none());
        assert!(first(&Value::Null).is_none());
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

    #[test]
    fn document_symbols_tree_and_flat() {
        let r = |a: u32, b: u32| json!({"start": {"line": a, "character": 0}, "end": {"line": b, "character": 1}});
        let tree = json!([{ "name": "Conta", "kind": 23, "range": r(0, 5), "selectionRange": r(0, 0),
            "children": [{ "name": "depositar", "kind": 6, "detail": "fn(&mut self)", "range": r(2, 2), "selectionRange": r(2, 2) }] }]);
        let s = document_symbols(&tree);
        assert_eq!((s[0].name.as_str(), s[0].children[0].detail.as_deref()), ("Conta", Some("fn(&mut self)")));

        // Formato antigo: lista simples, aninhada pelo trecho.
        let loc = |a, b| json!({"uri": "file:///a", "range": r(a, b)});
        let flat = json!([
            { "name": "b", "kind": 6, "location": loc(2, 3) },
            { "name": "A", "kind": 5, "location": loc(0, 9) },
            { "name": "c", "kind": 6, "location": loc(5, 6) },
            { "name": "solta", "kind": 12, "location": loc(10, 12) },
        ]);
        let s = document_symbols(&flat);
        let names: Vec<_> = s.iter().map(|x| (x.name.as_str(), x.children.len())).collect();
        assert_eq!(names, [("A", 2), ("solta", 0)]);
        assert!(document_symbols(&Value::Null).is_empty());
    }

}
