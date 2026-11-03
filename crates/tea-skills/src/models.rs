//! Os modelos do "Novo projeto" (`skills/modelos.toml`, embutido no binário): cada um é uma lista de skills e o
//! pedido inicial para o agente.

use serde::Deserialize;

const MODELS: &str = include_str!("../../../skills/modelos.toml");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Model {
    pub id: String,
    /// `nome` (da fonte padrão) ou `dono/repo:nome`.
    pub skills: Vec<String>,
    pub request: String,
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    source: String,
    #[serde(rename = "model")]
    models: Vec<Model>,
}

fn file() -> File {
    toml::from_str(MODELS).expect("skills/modelos.toml embutido é válido (conferido por teste)")
}

/// Os modelos, na ordem do arquivo.
pub fn all() -> Vec<Model> {
    file().models
}

pub fn get(id: &str) -> Option<Model> {
    all().into_iter().find(|m| m.id == id)
}

/// As skills de um modelo agrupadas por fonte, na ordem em que aparecem: `(fonte, [nomes])`.
pub fn by_source(model: &Model) -> Vec<(String, Vec<String>)> {
    let default = file().source;
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for s in &model.skills {
        let (source, name) = match s.rsplit_once(':') {
            Some((src, name)) => (src.to_string(), name.to_string()),
            None => (default.clone(), s.clone()),
        };
        match out.iter_mut().find(|(src, _)| *src == source) {
            Some((_, names)) => names.push(name),
            None => out.push((source, vec![name])),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_models_file_is_valid_and_its_skills_exist() {
        let models = all();
        assert_eq!(models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["vazio", "rust", "site", "python", "fastapi"]);
        let default = file().source;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
        for m in &models {
            for (source, names) in by_source(m) {
                for n in names {
                    assert!(crate::valid_name(&n), "{n}");
                    // As skills deste repositório: a pasta tem de existir, com o nome no SKILL.md.
                    if source == default {
                        let text = std::fs::read_to_string(root.join(&n).join("SKILL.md")).unwrap_or_else(|_| panic!("skills/{n}/SKILL.md"));
                        assert_eq!(crate::front_matter(&text).0.as_deref(), Some(n.as_str()));
                    }
                }
            }
        }
        let site = get("site").unwrap();
        assert_eq!(by_source(&site)[1], ("anthropics/skills".to_string(), vec!["frontend-design".to_string()]));
    }
}
