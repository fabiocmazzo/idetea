//! O que o usuário mudou numa interface e ainda não foi substituído pelo agente.
//!
//! Três donos, cada um com o seu: o agente é dono do documento (estrutura e valores iniciais em `data`); a IDE
//! guarda aqui o que o usuário mudou; o iced guarda foco e cursor. Numa revisão nova, o agente vence só nos
//! caminhos em que **ele** mudou o valor; o resto do que o usuário digitou fica.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::document::{Document, Kind, Submit};
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
        !self.local.is_empty() || !self.comment.is_empty()
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

    /// O campo em `path` tem valor: escolha feita, texto não vazio, caixa marcada, lista com algo.
    pub fn filled(&self, doc: &Document, path: &str) -> bool {
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
        let Kind::Button { requires, .. } = button else { return out };
        for path in requires {
            let Some(c) = doc.components.iter().find(|c| c.kind.binding() == Some(path.as_str())) else { continue };
            match &c.kind {
                Kind::Steps { .. } => out.steps += self.missing(doc, path),
                kind if !self.filled(doc, path) => out.fields.push(match kind {
                    Kind::Checkbox { label, .. } => label.clone(),
                    Kind::Select { label: Some(l), .. } | Kind::Input { label: Some(l), .. } => l.clone(),
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
        let values =
            action.context.iter().map(|p| (p.clone(), pointer::get(&seen, p).cloned().unwrap_or(Value::Null))).collect();
        self.answered = Some(doc.revision);
        Some((action.name.clone(), values))
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
