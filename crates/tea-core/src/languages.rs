use std::collections::BTreeMap;

use serde::Deserialize;

use crate::syntaxes::Source;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub id: String,
    pub name: String,
    pub files: Files,
    pub project: Option<Project>,
    pub lsp: Option<Lsp>,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Files {
    pub extensions: BTreeMap<String, String>,
    pub syntaxes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lsp {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinPack {
    pub manifest: Manifest,
    pub syntaxes: Vec<Source>,
}

pub fn builtins() -> (Vec<BuiltinPack>, Vec<String>) {
    let definitions = [
        (
            "hare",
            include_str!("../assets/languages/hare/pack.toml"),
            &[("hare.sublime-syntax", include_str!("../assets/languages/hare/hare.sublime-syntax"))][..],
        ),
        (
            "moonbit",
            include_str!("../assets/languages/moonbit/pack.toml"),
            &[("moonbit.sublime-syntax", include_str!("../assets/languages/moonbit/moonbit.sublime-syntax"))][..],
        ),
    ];
    let mut packs = Vec::new();
    let mut errors = Vec::new();
    for (id, text, resources) in definitions {
        match parse_builtin(id, text, resources) {
            Ok(pack) => packs.push(pack),
            Err(error) => errors.push(error),
        }
    }
    (packs, errors)
}

fn parse_builtin(id: &str, text: &str, resources: &[(&str, &str)]) -> Result<BuiltinPack, String> {
    let manifest: Manifest = toml::from_str(text).map_err(|e| format!("language pack {id}: {e}"))?;
    if manifest.format != 1 {
        return Err(format!("language pack {id}: formato {} desconhecido; esperado formato 1", manifest.format));
    }
    let mut syntaxes = Vec::new();
    for name in &manifest.files.syntaxes {
        let Some((_, text)) = resources.iter().find(|(resource, _)| resource == name) else {
            return Err(format!("language pack {id}: sintaxe não embutida: {name}"));
        };
        syntaxes.push(Source { label: format!("{id}/{name}"), text: (*text).to_string() });
    }
    Ok(BuiltinPack { manifest, syntaxes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_rejects_an_unknown_format() {
        let text = r#"
            format = 2
            id = "future"
            name = "Future"
            [files]
            extensions = { future = "future" }
            syntaxes = []
        "#;
        let error = parse_builtin("future", text, &[]).unwrap_err();
        assert!(error.contains("formato 1"), "{error}");
    }

    #[test]
    fn builtin_pack_ids_extensions_servers_and_tasks_are_unique() {
        let (packs, errors) = builtins();
        assert!(errors.is_empty(), "{errors:?}");
        let mut ids = std::collections::HashSet::new();
        let mut extensions = std::collections::HashSet::new();
        let mut servers = std::collections::HashSet::new();
        let mut tasks = std::collections::HashSet::new();
        for pack in packs {
            let manifest = pack.manifest;
            assert!(ids.insert(manifest.id));
            for extension in manifest.files.extensions.into_keys() {
                assert!(extensions.insert(extension));
            }
            if let Some(lsp) = manifest.lsp {
                assert!(servers.insert(lsp.name));
            }
            for task in manifest.tasks {
                assert!(tasks.insert(task.name));
            }
        }
    }

    #[test]
    fn builtin_syntaxes_produce_language_scopes() {
        use std::collections::HashSet;

        use syntect::parsing::{ParseState, ScopeStackOp, SyntaxDefinition, SyntaxSetBuilder};

        fn scopes(source: &Source, extension: &str, sample: &str) -> HashSet<String> {
            let mut builder = SyntaxSetBuilder::new();
            builder.add(SyntaxDefinition::load_from_str(&source.text, true, None).unwrap());
            let syntax_set = builder.build();
            let syntax = syntax_set.find_syntax_by_extension(extension).unwrap();
            let mut state = ParseState::new(syntax);
            let mut scopes = HashSet::new();
            for line in sample.split_inclusive('\n') {
                for (_, op) in state.parse_line(line, &syntax_set).unwrap() {
                    if let ScopeStackOp::Push(scope) = op {
                        scopes.insert(scope.build_string());
                    }
                }
            }
            scopes
        }

        let (packs, errors) = builtins();
        assert!(errors.is_empty(), "{errors:?}");
        let hare = packs.iter().find(|p| p.manifest.id == "hare").unwrap();
        let hare_scopes = scopes(
            &hare.syntaxes[0],
            "ha",
            "fn answer(): int { // resposta\n return 42;\n}\nlet greeting = \"hello\";\n",
        );
        for expected in [
            "keyword.control.hare",
            "support.type.primitive.hare",
            "constant.numeric.integer.decimal.hare",
            "string.quoted.double.hare",
            "comment.line.double-slash.hare",
        ] {
            assert!(hare_scopes.contains(expected), "escopo {expected} ausente: {hare_scopes:?}");
        }

        let moonbit = packs.iter().find(|p| p.manifest.id == "moonbit").unwrap();
        let moonbit_scopes = scopes(
            &moonbit.syntaxes[0],
            "mbt",
            "fn main() -> String { // resposta\n let answer = 42\n \"hello \\{answer}\"\n}\n",
        );
        for expected in [
            "keyword.declaration.function.moonbit",
            "support.type.builtin.moonbit",
            "constant.numeric.integer.decimal.moonbit",
            "string.quoted.double.moonbit",
            "meta.interpolation.moonbit",
            "comment.line.double-slash.moonbit",
        ] {
            assert!(moonbit_scopes.contains(expected), "escopo {expected} ausente: {moonbit_scopes:?}");
        }
    }

    #[test]
    fn every_pack_is_documented() {
        let languages = include_str!("../../../docs/languages.md");
        let (packs, errors) = builtins();
        assert!(errors.is_empty(), "{errors:?}");
        for pack in packs {
            let manifest = pack.manifest;
            let section = format!("\n## {}\n", manifest.name);
            assert!(languages.contains(&section), "docs/languages.md sem a seção de {}", manifest.name);
            for extension in manifest.files.extensions.keys() {
                assert!(languages.contains(&format!("`.{extension}`")), "docs/languages.md sem `.{extension}`");
            }
            let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/languages").join(&manifest.id);
            assert!(example.join("README.md").is_file(), "falta o exemplo {}", example.display());
        }
    }

    #[test]
    fn builtin_packs_describe_hare_and_moonbit() {
        let (packs, errors) = builtins();
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(packs.len(), 2);

        let hare = packs.iter().find(|p| p.manifest.id == "hare").unwrap();
        assert_eq!(hare.manifest.files.extensions.get("ha").map(String::as_str), Some("hare"));
        let lsp = hare.manifest.lsp.as_ref().unwrap();
        assert_eq!((lsp.command.as_str(), lsp.args.as_slice()), ("hare-lsp", ["-S".to_string()].as_slice()));
        assert!(hare.manifest.tasks.is_empty());
        assert_eq!(hare.syntaxes.len(), 1);

        let moonbit = packs.iter().find(|p| p.manifest.id == "moonbit").unwrap();
        assert_eq!(moonbit.manifest.files.extensions.get("mbt").map(String::as_str), Some("moonbit"));
        assert_eq!(moonbit.manifest.files.extensions.get("mbti").map(String::as_str), Some("moonbit"));
        assert_eq!(moonbit.manifest.project.as_ref().unwrap().markers, ["moon.mod"]);
        let lsp = moonbit.manifest.lsp.as_ref().unwrap();
        assert_eq!((lsp.command.as_str(), lsp.args.as_slice()), ("moon", ["lsp".to_string()].as_slice()));
        let commands: Vec<_> = moonbit.manifest.tasks.iter().map(|t| t.command.as_str()).collect();
        assert_eq!(commands, ["moon check", "moon build", "moon test", "moon fmt --check"]);
        assert_eq!(moonbit.syntaxes.len(), 1);
    }
}
