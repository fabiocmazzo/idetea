//! Ponteiros JSON (RFC 6901), como `/usuario/nome`: o lugar de um valor nos dados da interface.

use serde_json::{Map, Value};

/// Tamanho máximo de um ponteiro.
const MAX_LEN: usize = 256;

/// Um ponteiro que dá para usar: começa com `/`, tem ao menos um nome e não é gigante.
pub fn is_valid(path: &str) -> bool {
    path.len() > 1 && path.len() <= MAX_LEN && path.starts_with('/')
}

/// Os nomes do caminho, com `~1` → `/` e `~0` → `~`.
fn tokens(path: &str) -> impl Iterator<Item = String> + '_ {
    path.split('/').skip(1).map(|t| t.replace("~1", "/").replace("~0", "~"))
}

pub fn get<'a>(data: &'a Value, path: &str) -> Option<&'a Value> {
    data.pointer(path)
}

/// Grava `value` em `path`, criando os objetos do caminho que faltarem. Um pedaço do caminho que não é objeto
/// (um número no meio, por exemplo) vira objeto. Índices de listas existentes são respeitados.
pub fn set(data: &mut Value, path: &str, value: Value) {
    let names: Vec<String> = tokens(path).collect();
    let Some((last, parents)) = names.split_last() else { return };
    let mut here = data;
    for name in parents {
        here = match slot(here, name) {
            Some(Slot::Index(i)) => match here {
                Value::Array(items) => &mut items[i],
                _ => return,
            },
            _ => match here {
                Value::Object(map) => map.entry(name.clone()).or_insert_with(|| Value::Object(Map::new())),
                _ => return,
            },
        };
    }
    match (slot(here, last), here) {
        (Some(Slot::Index(i)), Value::Array(items)) => items[i] = value,
        (_, Value::Object(map)) => {
            map.insert(last.clone(), value);
        }
        _ => {}
    }
}

enum Slot {
    Index(usize),
    Key,
}

/// Onde `name` fica dentro de `here`: posição de uma lista que já existe, ou chave de objeto (o que não é
/// objeto vira um vazio).
fn slot(here: &mut Value, name: &str) -> Option<Slot> {
    if let Value::Array(items) = here
        && let Ok(i) = name.parse::<usize>()
        && i < items.len()
    {
        return Some(Slot::Index(i));
    }
    if !here.is_object() {
        *here = Value::Object(Map::new());
    }
    Some(Slot::Key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_and_set() {
        let mut d = json!({ "a": { "b": 1 }, "lista": [10, 20], "x/y": true });
        assert_eq!(get(&d, "/a/b"), Some(&json!(1)));
        assert_eq!(get(&d, "/x~1y"), Some(&json!(true)));
        set(&mut d, "/a/b", json!(2));
        set(&mut d, "/novo/fundo", json!("oi"));
        set(&mut d, "/lista/1", json!(21));
        set(&mut d, "/a/b/c", json!(3)); // número no caminho vira objeto
        assert_eq!(d, json!({ "a": { "b": { "c": 3 } }, "lista": [10, 21], "x/y": true, "novo": { "fundo": "oi" } }));
        assert!(is_valid("/a") && !is_valid("/") && !is_valid("a") && !is_valid(""));
    }
}
