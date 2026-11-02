//! `plugin.toml`: quem é o plugin, que arquivo `.wasm` carregar e se ele tem painel.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tea_core::Slot;

/// Versão da API (`package idetea:plugin@0.1.0` no WIT). Um plugin feito para outra não carrega.
pub const API: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api: String,
    /// Nome do arquivo `.wasm`, na mesma pasta do manifesto.
    pub wasm: String,
    #[serde(default)]
    pub panel: Option<PanelManifest>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelManifest {
    pub title: String,
    #[serde(default)]
    pub slot: Option<String>,
}

/// Id de plugin: como o nome de uma interface (minúsculas, números, `_` e `-`, até 64).
pub fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id.len() <= 64
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

impl Manifest {
    /// Lê e confere `<dir>/plugin.toml`. O id tem que ser o nome da pasta.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(dir.join("plugin.toml")).map_err(|e| e.to_string())?;
        let m: Manifest = toml::from_str(&text).map_err(|e| e.to_string())?;
        if !valid_id(&m.id) {
            return Err(fl!("plugin-bad-id", id = m.id.clone()));
        }
        let folder = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if folder != m.id {
            return Err(fl!("plugin-id-folder", id = m.id.clone(), folder = folder));
        }
        if m.api != API {
            return Err(fl!("plugin-api", found = m.api.clone(), expected = API));
        }
        if m.wasm.contains(['/', '\\']) || !m.wasm.ends_with(".wasm") {
            return Err(fl!("plugin-bad-wasm", wasm = m.wasm.clone()));
        }
        Ok(m)
    }

    pub fn wasm_path(&self, dir: &Path) -> PathBuf {
        dir.join(&self.wasm)
    }

    pub fn slot(&self) -> Slot {
        match self.panel.as_ref().and_then(|p| p.slot.as_deref()) {
            Some("left") => Slot::Left,
            Some("bottom") => Slot::Bottom,
            _ => Slot::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_manifest_is_valid() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins/contador");
        let m = Manifest::load(&dir).unwrap();
        assert_eq!((m.id.as_str(), m.api.as_str(), m.slot()), ("contador", API, Slot::Right));
    }

    #[test]
    fn rejects_bad_manifests() {
        let root = std::env::temp_dir().join(format!("idetea-manifest-{}", std::process::id()));
        let check = |folder: &str, text: &str| {
            let dir = root.join(folder);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("plugin.toml"), text).unwrap();
            Manifest::load(&dir)
        };
        let ok = "id = \"a\"\nname = \"A\"\nversion = \"1\"\napi = \"0.1\"\nwasm = \"a.wasm\"\n";
        assert!(check("a", ok).is_ok());
        assert!(check("b", ok).is_err(), "id diferente da pasta");
        assert!(check("a", &ok.replace("0.1", "9.0")).is_err(), "outra API");
        assert!(check("a", &ok.replace("a.wasm", "../x.wasm")).is_err());
        assert!(check("a", &format!("{ok}extra = 1\n")).is_err(), "campo desconhecido");
        let _ = std::fs::remove_dir_all(&root);
    }
}
