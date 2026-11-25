//! O que o usuário mudou numa interface e ainda não foi substituído pelo agente.
//!
//! Três donos, cada um com o seu: o agente é dono do documento (estrutura e valores iniciais em `data`); a IDE
//! guarda aqui o que o usuário mudou; o iced guarda foco e cursor. Numa revisão nova, o agente vence só nos
//! caminhos em que **ele** mudou o valor; o resto do que o usuário digitou fica.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::document::{Attach, Component, Document, Kind, Range, Submit, Test};
use crate::response::AttachmentKind;

/// Um anexo esperando o clique: o arquivo (caminho absoluto) e de onde veio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub path: String,
    pub kind: AttachmentKind,
    /// Arquivo escolhido: o caminho de origem, com `~` no lugar da pasta pessoal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}
use crate::pointer;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    /// Revisão do documento aplicada por último.
    pub revision: u64,
    /// `data` do agente nessa revisão (para saber, na próxima, onde ele mudou).
    base: Value,
    /// Valores mudados pelo usuário: caminho → valor.
    local: BTreeMap<String, Value>,
    /// Revisão já respondida (com `submit: "once"`, os botões ficam desligados nela).
    answered: Option<u64>,
    /// O "comentário para o agente" sendo escrito (vai na próxima resposta e some).
    pub comment: String,
    /// Anexos prontos para ir na próxima resposta (RFC 0026, etapa 1): o arquivo já está em
    /// `responses/<nome>/anexos/`, com nome de pendente.
    pub attachments: Vec<Pending>,
    /// Só para ver (histórico): botões e comentário desligados.
    #[serde(skip)]
    frozen: bool,
}

impl State {
    pub fn new(doc: &Document) -> Self {
        Self { revision: doc.revision, base: doc.data.clone(), ..Self::default() }
    }

    /// Uma interface já respondida, como ficou: os valores da resposta por cima do `data`, tudo desligado.
    pub fn frozen(doc: &Document, values: &BTreeMap<String, Value>, comment: Option<&str>) -> Self {
        let local = values.iter().filter(|(_, v)| !v.is_null()).map(|(k, v)| (k.clone(), v.clone())).collect();
        Self {
            revision: doc.revision,
            base: doc.data.clone(),
            local,
            answered: Some(doc.revision),
            comment: comment.unwrap_or_default().to_string(),
            attachments: Vec::new(),
            frozen: true,
        }
    }

    /// O valor que o componente mostra: o do usuário, senão o do documento.
    pub fn value<'a>(&'a self, doc: &'a Document, path: &str) -> Option<&'a Value> {
        self.local.get(path).or_else(|| pointer::get(&doc.data, path))
    }

    pub fn set(&mut self, path: &str, value: Value) {
        self.local.insert(path.to_string(), value);
    }

    /// Há algo digitado ou escolhido que o agente ainda não viu.
    pub fn has_changes(&self) -> bool {
        !self.local.is_empty() || !self.comment.is_empty() || !self.attachments.is_empty()
    }

    /// Passa para uma versão nova do documento (ou a mesma, relida). Valores do usuário em caminhos que o
    /// agente mudou saem; os outros ficam. Também serve para um estado salvo antes de reiniciar a IDE.
    pub fn apply(&mut self, doc: &Document) {
        let base = std::mem::replace(&mut self.base, doc.data.clone());
        self.local.retain(|path, _| pointer::get(&base, path) == pointer::get(&doc.data, path));
        // Caminhos que nenhum componente usa mais não interessam.
        self.local.retain(|path, _| doc.components.iter().any(|c| c.kind.binding() == Some(path.as_str())));
        self.revision = doc.revision;
    }

    /// Ids marcados num `steps` (o valor no caminho é uma lista de ids).
    pub fn checked<'a>(&'a self, doc: &'a Document, path: &str) -> Vec<&'a str> {
        self.value(doc, path).and_then(Value::as_array).map_or(Vec::new(), |a| a.iter().filter_map(Value::as_str).collect())
    }

    /// Quantos passos obrigatórios do `steps` em `path` ainda faltam marcar.
    pub fn missing(&self, doc: &Document, path: &str) -> usize {
        let checked = self.checked(doc, path);
        doc.components
            .iter()
            .filter_map(|c| match &c.kind {
                Kind::Steps { items, value } if value == path => Some(items),
                _ => None,
            })
            .flatten()
            .filter(|s| !s.optional && !checked.contains(&s.id.as_str()))
            .count()
    }

    /// O componente aparece: sem `show_if`, ou com a comparação batendo.
    pub fn visible(&self, doc: &Document, c: &Component) -> bool {
        let Some(cond) = &c.show_if else { return true };
        let v = self.value(doc, &cond.path).unwrap_or(&Value::Null);
        match &cond.test {
            Test::Equals(x) => same(v, x),
            Test::NotEquals(x) => !same(v, x),
            Test::In(list) => list.iter().any(|x| same(v, x)),
            Test::Truthy(want) => self.filled(doc, &cond.path) == *want,
        }
    }

    /// O número de um `number`/`slider`: o valor, ou o texto digitado lido como número.
    pub fn number(&self, doc: &Document, path: &str) -> Option<f64> {
        match self.value(doc, path)? {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => parse_number(s),
            _ => None,
        }
    }

    /// O que há de errado no `number`/`slider` em `path` (`None`: vazio ou certo).
    pub fn number_problem(&self, doc: &Document, path: &str, range: &Range) -> Option<String> {
        match self.value(doc, path) {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) if s.trim().is_empty() => None,
            _ => match self.number(doc, path) {
                None => Some(fl!("ui-number-nan")),
                Some(n) if !range.contains(n) => Some(fl!("ui-number-out", range = describe(range))),
                Some(_) => None,
            },
        }
    }

    /// O campo em `path` tem valor: escolha feita, texto não vazio, caixa marcada, lista com algo; de um número,
    /// um número dentro dos limites.
    pub fn filled(&self, doc: &Document, path: &str) -> bool {
        let number = doc.components.iter().find_map(|c| match &c.kind {
            Kind::Number { value, range, .. } if value == path => Some(range),
            _ => None,
        });
        if let Some(range) = number {
            return self.number(doc, path).is_some_and(|n| range.contains(n));
        }
        match self.value(doc, path) {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(Value::String(s)) => !s.trim().is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
            Some(Value::Number(_)) => true,
        }
    }

    /// O que o botão ainda espera (`requires`): passos que faltam e os campos vazios (pelo rótulo).
    pub fn waiting(&self, doc: &Document, button: &Kind) -> Waiting {
        let mut out = Waiting::default();
        let Kind::Button { requires, attach, .. } = button else { return out };
        if let Some(attach) = attach {
            let has = self.attachments.iter().any(|a| match attach {
                Attach::Any => true,
                Attach::Screenshot => a.kind == AttachmentKind::Screenshot,
                Attach::File => a.kind == AttachmentKind::File,
            });
            if !has {
                out.fields.push(match attach {
                    Attach::Screenshot => fl!("ui-attach-needed-screenshot"),
                    Attach::File | Attach::Any => fl!("ui-attach-needed"),
                });
            }
        }
        for path in requires {
            let Some(c) = doc.components.iter().find(|c| c.kind.binding() == Some(path.as_str())) else { continue };
            // Um campo escondido (`show_if`) não segura o botão.
            if !self.visible(doc, c) {
                continue;
            }
            match &c.kind {
                Kind::Steps { .. } => out.steps += self.missing(doc, path),
                kind if !self.filled(doc, path) => out.fields.push(match kind {
                    Kind::Checkbox { label, .. } => label.clone(),
                    Kind::Select { label: Some(l), .. } | Kind::Input { label: Some(l), .. } | Kind::Number { label: Some(l), .. } => {
                        l.clone()
                    }
                    _ => c.id.clone(),
                }),
                _ => {}
            }
        }
        out
    }

    /// Os botões estão desligados: já respondeu esta revisão e ela só aceita uma resposta.
    pub fn locked(&self, doc: &Document) -> bool {
        self.frozen || (doc.submit == Submit::Once && self.answered == Some(doc.revision))
    }

    /// Valores dos caminhos pedidos pelo botão `source` (o `context` da ação). `None`: não é um botão.
    /// Marca a revisão como respondida.
    pub fn press(&mut self, doc: &Document, source: &str) -> Option<(String, BTreeMap<String, Value>)> {
        if self.locked(doc) {
            return None;
        }
        let button = doc.components.iter().find(|c| c.id == source)?;
        let Kind::Button { action, .. } = &button.kind else { return None };
        if !self.waiting(doc, &button.kind).is_empty() {
            return None;
        }
        // Os dados como o usuário vê: o `data` do agente com o que ele mudou por cima. Assim um caminho de objeto
        // (`/ajustar`) traz os campos de dentro já com as mudanças.
        let mut seen = doc.data.clone();
        let mut local: Vec<(&String, &Value)> = self.local.iter().collect();
        local.sort_by_key(|(k, _)| k.len()); // pais antes dos filhos
        for (path, value) in local {
            pointer::set(&mut seen, path, value.clone());
        }
        let values = action
            .context
            .iter()
            .map(|p| {
                let mut v = pointer::get(&seen, p).cloned().unwrap_or(Value::Null);
                // Um número digitado vai como número JSON (o texto, se não for número).
                let is_number = doc.components.iter().any(|c| matches!(&c.kind, Kind::Number { value, .. } if value == p));
                if is_number && let Value::String(s) = &v {
                    v = match parse_number(s) {
                        Some(n) => number_value(n),
                        None if s.trim().is_empty() => Value::Null,
                        None => v,
                    };
                }
                (p.clone(), v)
            })
            .collect();
        self.answered = Some(doc.revision);
        Some((action.name.clone(), values))
    }
}

/// Igualdade de valores do `show_if`: números pelo valor (`1` e `1.0` são iguais).
fn same(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// Um número digitado: aceita vírgula decimal (`1,5`).
pub fn parse_number(s: &str) -> Option<f64> {
    s.trim().replace(',', ".").parse::<f64>().ok().filter(|n| n.is_finite())
}

/// O número como JSON: inteiro quando não tem fração.
pub fn number_value(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9e15 {
        Value::from(n as i64)
    } else {
        serde_json::Number::from_f64(n).map_or(Value::Null, Value::Number)
    }
}

/// O número para mostrar (sem `.0` à toa).
pub fn show_number(n: f64) -> String {
    match number_value(n) {
        Value::Number(x) => x.to_string(),
        _ => String::new(),
    }
}

/// "1–100 usuários", "≥ 1", "≤ 10".
fn describe(range: &Range) -> String {
    let unit = range.unit.as_deref().map(|u| format!(" {u}")).unwrap_or_default();
    match (range.min, range.max) {
        (Some(a), Some(b)) => format!("{}–{}{unit}", show_number(a), show_number(b)),
        (Some(a), None) => format!("≥ {}{unit}", show_number(a)),
        (None, Some(b)) => format!("≤ {}{unit}", show_number(b)),
        (None, None) => String::new(),
    }
}

/// O que falta para um botão com `requires` ligar.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Waiting {
    /// Passos obrigatórios ainda sem marcar.
    pub steps: usize,
    /// Campos ainda vazios, pelo rótulo (ou id).
    pub fields: Vec<String>,
}

impl Waiting {
    pub fn is_empty(&self) -> bool {
        self.steps == 0 && self.fields.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Limits, parse};
    use serde_json::json;

    fn doc(revision: u64, data: Value) -> Document {
        let v = json!({
            "format": "idetea-ui", "version": 1, "revision": revision, "data": data,
            "components": [
                { "id": "a", "component": "input", "value": { "path": "/a" } },
                { "id": "b", "component": "input", "value": { "path": "/b" } },
                { "id": "ok", "component": "button", "label": "OK", "action": { "name": "ok", "context": ["/a", "/b", "/c"] } }
            ]
        });
        parse(v.to_string().as_bytes(), &Limits::default()).unwrap()
    }

    fn parsed(v: Value) -> Result<Document, String> {
        parse(v.to_string().as_bytes(), &Limits::default())
    }

    #[test]
    fn numbers_go_as_numbers_and_wait_inside_the_range() {
        let d = parsed(json!({
            "format": "idetea-ui", "version": 1, "revision": 1, "data": {"r": 3},
            "components": [
                { "id": "u", "component": "number", "label": "Usuários", "value": "/u", "min": 1, "max": 100, "unit": "usuários" },
                { "id": "r", "component": "slider", "label": "Réplicas", "value": "/r", "min": 1, "max": 12 },
                { "id": "ok", "component": "button", "label": "OK", "requires": ["/u", "/r"], "action": { "name": "ok", "context": ["/u", "/r"] } }
            ]
        }))
        .unwrap();
        let Kind::Number { range, .. } = &d.components[0].kind else { panic!() };
        let range = range.clone();
        let mut s = State::new(&d);
        assert_eq!(s.waiting(&d, &d.components[2].kind).fields, ["Usuários"], "vazio espera; o slider já tem 3");
        s.set("/u", json!("abc"));
        assert!(s.number_problem(&d, "/u", &range).is_some());
        s.set("/u", json!("500"));
        assert!(s.number_problem(&d, "/u", &range).unwrap().contains("1–100 usuários"));
        assert!(s.press(&d, "ok").is_none(), "fora do intervalo, o botão espera");
        s.set("/u", json!("12,5"));
        let (_, values) = s.press(&d, "ok").unwrap();
        assert_eq!(values["/u"], json!(12.5), "vírgula decimal, número JSON");
        assert_eq!(values["/r"], json!(3));
    }

    #[test]
    fn number_and_columns_are_checked_when_read() {
        let base = |extra: Value| {
            let mut c = vec![json!({ "id": "a", "component": "input", "value": "/a" }), json!({ "id": "b", "component": "input", "value": "/b" })];
            c.push(extra);
            parsed(json!({ "format": "idetea-ui", "version": 1, "revision": 1, "components": c }))
        };
        assert!(base(json!({ "id": "s", "component": "slider", "value": "/s", "min": 1 })).is_err(), "slider sem max");
        assert!(base(json!({ "id": "n", "component": "number", "value": "/n", "min": 5, "max": 1 })).is_err());
        assert!(base(json!({ "id": "c", "component": "columns", "children": ["a", "b"] })).is_ok());
        assert!(base(json!({ "id": "c", "component": "columns", "children": ["a", "x"] })).is_err(), "id que não existe");
        assert!(base(json!({ "id": "c", "component": "columns", "children": ["a", "b", "a", "b", "a"] })).is_err(), "mais de 4");
        assert!(base(json!({ "id": "c", "component": "columns", "children": ["c"] })).is_err(), "aninhado");
        let two = parsed(json!({ "format": "idetea-ui", "version": 1, "revision": 1, "components": [
            { "id": "a", "component": "input", "value": "/a" },
            { "id": "c1", "component": "columns", "children": ["a"] },
            { "id": "c2", "component": "columns", "children": ["a"] }]}));
        assert!(two.is_err(), "o mesmo id em dois columns");
    }

    #[test]
    fn show_if_hides_and_frees_the_button() {
        let d = parsed(json!({
            "format": "idetea-ui", "version": 1, "revision": 1, "data": {"modo": "main"},
            "components": [
                { "id": "m", "component": "select", "value": "/modo", "options": ["main", {"value": "branch", "recommended": true, "note": "isola"}] },
                { "id": "n", "component": "input", "label": "Branch", "value": "/branch", "show_if": {"path": "/modo", "equals": "branch"} },
                { "id": "x", "component": "text", "text": "x", "show_if": {"path": "/modo", "in": ["a", "main"]} },
                { "id": "y", "component": "text", "text": "y", "show_if": {"path": "/branch", "truthy": true} },
                { "id": "ok", "component": "button", "label": "OK", "requires": "/branch", "action": { "name": "ok", "context": ["/branch"] } }
            ]
        }))
        .unwrap();
        let Kind::Select { options, .. } = &d.components[0].kind else { panic!() };
        assert!(options[1].recommended && options[1].note.as_deref() == Some("isola"));
        let mut s = State::new(&d);
        let c = &d.components;
        assert!(!s.visible(&d, &c[1]) && s.visible(&d, &c[2]) && !s.visible(&d, &c[3]));
        assert!(s.waiting(&d, &c[4].kind).is_empty(), "escondido não segura o botão");
        s.set("/modo", json!("branch"));
        assert!(s.visible(&d, &c[1]) && !s.visible(&d, &c[2]));
        assert_eq!(s.waiting(&d, &c[4].kind).fields, ["Branch"]);
        s.set("/branch", json!("fix/x"));
        assert!(s.visible(&d, &c[3]));
        assert!(parsed(json!({ "format": "idetea-ui", "version": 1, "revision": 1, "components": [
            { "id": "t", "component": "text", "text": "t", "show_if": {"path": "/a", "equals": 1, "in": [1]} }]})).is_err(), "dois operadores");
    }

    #[test]
    fn a_button_with_attach_waits_for_the_right_attachment() {
        let v = json!({
            "format": "idetea-ui", "version": 1, "revision": 1,
            "components": [
                { "id": "foto", "component": "button", "label": "Enviar", "attach": "screenshot", "action": { "name": "ok" } },
                { "id": "qualquer", "component": "button", "label": "Outro", "attach": "any", "action": { "name": "ok" } },
                { "id": "livre", "component": "button", "label": "Sem", "action": { "name": "ok" } }
            ]
        });
        let d = parse(v.to_string().as_bytes(), &Limits::default()).unwrap();
        let mut s = State::new(&d);
        let wait = |s: &State, i: usize| s.waiting(&d, &d.components[i].kind).fields.len();
        assert_eq!((wait(&s, 0), wait(&s, 1), wait(&s, 2)), (1, 1, 0), "sem anexo, os dois esperam");
        assert!(s.press(&d, "foto").is_none());
        s.attachments.push(Pending { path: "/x/a.txt".into(), kind: AttachmentKind::File, from: None });
        assert_eq!((wait(&s, 0), wait(&s, 1)), (1, 0), "um arquivo não é uma captura");
        s.attachments.push(Pending { path: "/x/b.png".into(), kind: AttachmentKind::Screenshot, from: None });
        assert_eq!(wait(&s, 0), 0);
        assert!(s.press(&d, "foto").is_some());
    }

    #[test]
    fn an_unknown_attach_warns_and_accepts_any() {
        let v = json!({"format": "idetea-ui", "version": 1, "revision": 1, "components": [
            { "id": "b", "component": "button", "label": "B", "attach": "video", "action": { "name": "ok" } }]});
        let d = parse(v.to_string().as_bytes(), &Limits::default()).unwrap();
        assert!(matches!(d.components[0].kind, Kind::Button { attach: Some(Attach::Any), .. }));
        assert_eq!(d.warnings.len(), 1);
    }

    #[test]
    fn object_paths_carry_the_user_changes() {
        let v = json!({
            "format": "idetea-ui", "version": 1, "revision": 1,
            "data": { "ajustar": { "balao": false, "passaro": false } },
            "components": [
                { "id": "a", "component": "checkbox", "label": "Balão", "value": "/ajustar/balao" },
                { "id": "b", "component": "checkbox", "label": "Pássaro", "value": "/ajustar/passaro" },
                { "id": "ok", "component": "button", "label": "OK", "action": { "name": "ok", "context": ["/ajustar", "/ajustar/balao", "/nada"] } }
            ]
        });
        let d = parse(v.to_string().as_bytes(), &Limits::default()).unwrap();
        let mut s = State::new(&d);
        s.set("/ajustar/balao", json!(true));
        let (_, values) = s.press(&d, "ok").unwrap();
        assert_eq!(values["/ajustar"], json!({ "balao": true, "passaro": false }));
        assert_eq!(values["/ajustar/balao"], json!(true));
        assert_eq!(values["/nada"], Value::Null);
    }

    #[test]
    fn user_values_survive_where_the_agent_did_not_change() {
        let d1 = doc(1, json!({ "a": "x", "b": "y" }));
        let mut s = State::new(&d1);
        assert_eq!(s.value(&d1, "/a"), Some(&json!("x")));
        s.set("/a", json!("digitado"));
        s.set("/b", json!("também"));

        // Revisão 2: o agente mudou só o /b.
        let d2 = doc(2, json!({ "a": "x", "b": "novo" }));
        s.apply(&d2);
        assert_eq!(s.value(&d2, "/a"), Some(&json!("digitado")), "o agente não mexeu: fica o do usuário");
        assert_eq!(s.value(&d2, "/b"), Some(&json!("novo")), "o agente mudou: vence o agente");
        assert_eq!(s.revision, 2);
    }

    #[test]
    fn press_once_then_locked_until_a_new_revision() {
        let d1 = doc(1, json!({ "a": "x" }));
        let mut s = State::new(&d1);
        s.set("/b", json!("y"));
        let (name, values) = s.press(&d1, "ok").unwrap();
        assert_eq!(name, "ok");
        assert_eq!(values, BTreeMap::from([
            ("/a".to_string(), json!("x")),
            ("/b".to_string(), json!("y")),
            ("/c".to_string(), Value::Null),
        ]));
        assert!(s.locked(&d1));
        assert!(s.press(&d1, "ok").is_none(), "submit once");
        assert!(s.press(&d1, "a").is_none(), "não é botão");

        let d2 = doc(2, json!({ "a": "x" }));
        s.apply(&d2);
        assert!(!s.locked(&d2), "revisão nova reabre");
    }

    #[test]
    fn saved_state_round_trip() {
        let d1 = doc(1, json!({ "a": "x" }));
        let mut s = State::new(&d1);
        s.set("/a", json!("digitado"));
        let text = serde_json::to_string(&s).unwrap();
        let mut back: State = serde_json::from_str(&text).unwrap();
        back.apply(&d1);
        assert_eq!(back, s);
    }
}
