//! Plugins WASM da IDE TEA (protótipo da Fase 4): componentes que falam com a IDE pelo WIT em `wit/plugin.wit`.
//! Decisões e medições em `docs/rfc/0002-plugins-wasm.md`.
//!
//! Plugins ficam em `$XDG_DATA_HOME/idetea/plugins/<id>/` (`plugin.toml` + o `.wasm`). Nunca na pasta do projeto:
//! um repositório clonado não instala código na IDE.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod extension;
pub mod manifest;
mod process;
pub mod runtime;

use std::path::{Path, PathBuf};

pub use extension::PluginExtension;
use manifest::Manifest;

/// Onde ficam os plugins instalados. `IDETEA_PLUGINS_DIR` troca (testes, desenvolvimento de um plugin).
pub fn plugins_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("IDETEA_PLUGINS_DIR").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))?;
    Some(base.join("idetea").join("plugins"))
}

/// Um plugin por subpasta, em ordem de nome. Ids já usados (`taken`: extensões de dentro e prefixos de comandos
/// do núcleo) não carregam: o plugin aparece com o erro. Nada é carregado aqui; o `.wasm` só é compilado na
/// ativação.
pub fn discover(dir: &Path, taken: &[&str]) -> Vec<PluginExtension> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .filter(|p| p.join("plugin.toml").is_file())
        .collect();
    dirs.sort();
    dirs.iter()
        .map(|d| {
            let manifest = Manifest::load(d).and_then(|m| {
                if taken.contains(&m.id.as_str()) { Err(fl!("plugin-id-taken", id = m.id.clone())) } else { Ok(m) }
            });
            PluginExtension::new(d, manifest)
        })
        .collect()
}
