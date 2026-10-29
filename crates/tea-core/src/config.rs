//! Configuração em camadas: padrões embutidos → global → projeto.
//!
//! Global: `$XDG_CONFIG_HOME/idetea/config.toml` (ou `~/.config/idetea/config.toml`).
//! Projeto: `<raiz>/.idetea/config.toml`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Slot;

/// Config padrão. Também é o conteúdo inicial do arquivo global quando ele é criado.
pub const DEFAULT_CONFIG: &str = r#"# Configuração da IDE TEA.
# Este arquivo é recarregado ao salvar.
# Config de projeto: <projeto>/.idetea/config.toml (sobrepõe este).

[ui]
# Splash screen ao abrir (mínimo de 2 s). Também desliga com --no-splash ou IDETEA_NO_SPLASH=1.
splash = true
# "dark", "light" ou o nome de um tema do iced (ex.: "Tokyo Night", "Nord", "Dracula").
theme = "dark"
font_size = 14
# Faixa de ícones na borda esquerda para mostrar/ocultar painéis.
activity_bar = true

# Servidores de linguagem (LSP). Embutidos: rust-analyzer, clangd, pyright,
# typescript, gopls, lua. Só iniciam se o comando existir no PATH.
[lsp]
enabled = true
# Exemplos:
# servers.rust-analyzer = { command = "/caminho/rust-analyzer" }
# servers.clangd = { enabled = false }
# servers.zls = { command = "zls", languages = { zig = "zig" } }

# Extensões: enabled = liga/desliga (precisa reiniciar); slot = "left", "right" ou "bottom".
[extensions]
filetree = { enabled = true, slot = "left" }
statusbar = { enabled = true }
search = { enabled = true, slot = "bottom" }
problems = { enabled = true, slot = "bottom" }
terminal = { enabled = true, slot = "bottom" }
git = { enabled = true, slot = "bottom" }

# Atalhos: "combinação" = "comando". Comando vazio ("") desliga o atalho.
# Atalhos que o editor já usa (Ctrl+F, Ctrl+H, Ctrl+G, Ctrl+Z, Ctrl+D...) têm prioridade quando ele tem foco.
[keys]
"ctrl+shift+p" = "palette.open"
"f1" = "help.open"
"ctrl+s" = "file.save"
"ctrl+shift+s" = "file.save_as"
"ctrl+o" = "file.open"
"ctrl+p" = "file.quick_open"
"ctrl+n" = "file.new"
"ctrl+w" = "file.close"
"ctrl+pagedown" = "tab.next"
"ctrl+pageup" = "tab.prev"
"ctrl+b" = "filetree.toggle"
"ctrl+shift+e" = "filetree.reveal"
"ctrl+shift+f" = "search.focus"
"ctrl+shift+j" = "view.toggle_bottom"
"ctrl+shift+m" = "problems.focus"
"f8" = "problems.next"
"f12" = "lsp.definition"
"f9" = "debug.toggle_breakpoint"
"f5" = "debug.continue"
"shift+f5" = "debug.stop"
"f6" = "debug.pause"
"f10" = "debug.next"
"f11" = "debug.step_in"
"shift+f11" = "debug.step_out"
"ctrl+shift+k" = "lsp.hover"
"ctrl+shift+i" = "lsp.format"
"ctrl+shift+t" = "terminal.focus"
"ctrl+shift+g" = "git.focus"
"ctrl+," = "settings.open"
"ctrl+q" = "app.quit"
"#;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub splash: bool,
    pub theme: String,
    pub font_size: f32,
    pub activity_bar: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { splash: true, theme: "dark".into(), font_size: 14.0, activity_bar: true }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ExtensionConfig {
    pub enabled: bool,
    pub slot: Option<Slot>,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self { enabled: true, slot: None }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LspConfig {
    pub enabled: bool,
    /// Sobrescreve ou acrescenta servidores, por nome.
    pub servers: BTreeMap<String, LspServerConfig>,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self { enabled: true, servers: BTreeMap::new() }
    }
}

/// Campos ausentes mantêm o padrão do servidor embutido de mesmo nome.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LspServerConfig {
    pub enabled: Option<bool>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    /// Extensão de arquivo → languageId. Ex.: `{ rs = "rust" }`.
    pub languages: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub ui: UiConfig,
    pub lsp: LspConfig,
    pub extensions: BTreeMap<String, ExtensionConfig>,
    /// Atalhos na ordem das camadas (os últimos vencem).
    pub keys: Vec<(String, String)>,
    /// Problemas encontrados ao ler (o config segue com o que deu para ler).
    pub errors: Vec<String>,
}

impl Config {
    /// Carrega padrões + arquivos que existirem.
    pub fn load(global: Option<&Path>, project: Option<&Path>) -> Self {
        let mut layers = vec![("padrão".to_string(), DEFAULT_CONFIG.to_string())];
        let mut errors = Vec::new();
        for path in [global, project].into_iter().flatten() {
            match std::fs::read_to_string(path) {
                Ok(text) => layers.push((path.display().to_string(), text)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }
        let mut config = Self::from_layers(&layers);
        config.errors.splice(0..0, errors);
        config
    }

    /// Junta camadas de TOML `(nome, texto)`. As últimas sobrepõem as primeiras.
    pub fn from_layers(layers: &[(String, String)]) -> Self {
        let mut merged = toml::Table::new();
        let mut keys = Vec::new();
        let mut errors = Vec::new();
        for (name, text) in layers {
            let mut table: toml::Table = match text.parse() {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("{name}: {e}"));
                    continue;
                }
            };
            // `keys` é juntado à parte para manter a ordem das camadas.
            if let Some(layer_keys) = table.remove("keys") {
                match layer_keys {
                    toml::Value::Table(t) => {
                        for (chord, cmd) in t {
                            match cmd {
                                toml::Value::String(cmd) => keys.push((chord, cmd)),
                                other => errors.push(format!("{name}: [keys] {chord:?} deve ser texto, veio {other}")),
                            }
                        }
                    }
                    _ => errors.push(format!("{name}: [keys] deve ser uma tabela")),
                }
            }
            deep_merge(&mut merged, table);
        }

        let ui = take_section(&mut merged, "ui", &mut errors);
        let extensions = take_section(&mut merged, "extensions", &mut errors);
        let lsp = take_section(&mut merged, "lsp", &mut errors);
        for unknown in merged.keys() {
            errors.push(format!("seção desconhecida: [{unknown}]"));
        }
        Self { ui, lsp, extensions, keys, errors }
    }

    /// Config de uma extensão (padrão: ligada, slot escolhido pela extensão).
    pub fn extension(&self, id: &str) -> ExtensionConfig {
        self.extensions.get(id).cloned().unwrap_or_default()
    }
}

fn take_section<T: for<'de> Deserialize<'de> + Default>(
    table: &mut toml::Table,
    name: &str,
    errors: &mut Vec<String>,
) -> T {
    match table.remove(name) {
        None => T::default(),
        Some(value) => value.try_into().unwrap_or_else(|e| {
            errors.push(format!("[{name}]: {e}"));
            T::default()
        }),
    }
}

fn deep_merge(base: &mut toml::Table, over: toml::Table) {
    for (k, v) in over {
        match (base.get_mut(&k), v) {
            (Some(toml::Value::Table(b)), toml::Value::Table(o)) => deep_merge(b, o),
            (_, v) => {
                base.insert(k, v);
            }
        }
    }
}

/// Caminho do config global.
pub fn global_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("idetea").join("config.toml"))
}

/// Caminho do config do projeto.
pub fn project_config_path(root: &Path) -> PathBuf {
    root.join(".idetea").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layers(texts: &[&str]) -> Config {
        let l: Vec<_> = texts.iter().enumerate().map(|(i, t)| (format!("camada{i}"), t.to_string())).collect();
        Config::from_layers(&l)
    }

    #[test]
    fn defaults_parse_cleanly() {
        let c = layers(&[DEFAULT_CONFIG]);
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert!(c.ui.splash);
        assert_eq!(c.extension("filetree").slot, Some(Slot::Left));
        assert!(c.keys.iter().any(|(k, v)| k == "ctrl+s" && v == "file.save"));
    }

    #[test]
    fn later_layers_win() {
        let c = layers(&[
            DEFAULT_CONFIG,
            "[ui]\nsplash = false\n[extensions]\nfiletree = { slot = \"right\" }",
            "[ui]\ntheme = \"light\"\n[keys]\n\"ctrl+s\" = \"\"",
        ]);
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert!(!c.ui.splash);
        assert_eq!(c.ui.theme, "light");
        // Merge profundo: `enabled` continua true, só o slot mudou.
        let ft = c.extension("filetree");
        assert!(ft.enabled);
        assert_eq!(ft.slot, Some(Slot::Right));
        assert_eq!(c.keys.last().unwrap(), &("ctrl+s".to_string(), String::new()));
    }

    #[test]
    fn broken_layer_is_reported_not_fatal() {
        let c = layers(&[DEFAULT_CONFIG, "[ui\nsplash = ", "[ui]\nsplash = \"sim\"\n[outra]\nx = 1"]);
        assert_eq!(c.errors.len(), 3, "{:?}", c.errors);
        assert!(c.ui.splash); // [ui] inválido cai no padrão
    }

    #[test]
    fn lsp_overrides() {
        let c = layers(&[
            DEFAULT_CONFIG,
            "[lsp.servers.clangd]\nenabled = false\n[lsp.servers.zls]\ncommand = \"zls\"\nlanguages = { zig = \"zig\" }",
        ]);
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert!(c.lsp.enabled);
        assert_eq!(c.lsp.servers["clangd"].enabled, Some(false));
        assert_eq!(c.lsp.servers["zls"].languages.as_ref().unwrap()["zig"], "zig");
    }

    #[test]
    fn unknown_extension_defaults_enabled() {
        let c = layers(&[DEFAULT_CONFIG]);
        assert!(c.extension("nao-existe").enabled);
    }
}
