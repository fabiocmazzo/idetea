//! Edição de um arquivo de config preservando comentários e formatação (`toml_edit`).
//! Usado pelo painel de configurações.

use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, value};

use crate::Slot;
use crate::config::DEFAULT_CONFIG;

pub struct ConfigFile {
    path: PathBuf,
    doc: DocumentMut,
}

impl ConfigFile {
    /// Abre o arquivo. Se não existir, começa de `initial` (ex.: o config padrão comentado, no global).
    pub fn open(path: &Path, initial: &str) -> Result<Self, String> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => initial.to_string(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let doc = text.parse::<DocumentMut>().map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self { path: path.to_path_buf(), doc })
    }

    /// Para o config global: um arquivo novo nasce com os padrões comentados.
    pub fn open_global(path: &Path) -> Result<Self, String> {
        Self::open(path, DEFAULT_CONFIG)
    }

    /// Para o config do projeto: um arquivo novo nasce vazio.
    pub fn open_project(path: &Path) -> Result<Self, String> {
        Self::open(path, "")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> String {
        self.doc.to_string()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&self.path, self.text())
    }

    fn section(&mut self, name: &str) -> &mut Item {
        let item = &mut self.doc[name];
        if item.is_none() {
            // Implícita: o cabeçalho `[nome]` só aparece se ela tiver valores próprios.
            let mut t = Table::new();
            t.set_implicit(true);
            *item = Item::Table(t);
        }
        item
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.section("ui")["theme"] = value(theme);
    }

    pub fn set_font_size(&mut self, size: f32) {
        // Inteiro quando der: `14` fica mais limpo que `14.0` no arquivo.
        if size.fract() == 0.0 {
            self.section("ui")["font_size"] = value(size as i64);
        } else {
            self.section("ui")["font_size"] = value(f64::from(size));
        }
    }

    pub fn set_splash(&mut self, on: bool) {
        self.section("ui")["splash"] = value(on);
    }

    pub fn set_extension_enabled(&mut self, id: &str, on: bool) {
        set(self.extension(id), "enabled", value(on));
    }

    pub fn set_extension_slot(&mut self, id: &str, slot: Slot) {
        let name = match slot {
            Slot::Left => "left",
            Slot::Right => "right",
            Slot::Bottom => "bottom",
            Slot::Status => "status",
        };
        set(self.extension(id), "slot", value(name));
    }

    /// Entrada da extensão; nasce como tabela inline (`id = { ... }`), igual ao config padrão.
    fn extension(&mut self, id: &str) -> &mut Item {
        let section = self.section("extensions");
        if section.get(id).is_none_or(Item::is_none) {
            set(section, id, value(toml_edit::InlineTable::new()));
        }
        &mut section[id]
    }

    pub fn set_lsp_enabled(&mut self, on: bool) {
        self.section("lsp")["enabled"] = value(on);
    }

    pub fn set_lsp_server_enabled(&mut self, name: &str, on: bool) {
        let lsp = self.section("lsp");
        if lsp.get("servers").is_none_or(Item::is_none) {
            // Fica como `[lsp.servers]` implícita: só aparecem as linhas dos servidores.
            let mut t = Table::new();
            t.set_implicit(true);
            set(lsp, "servers", Item::Table(t));
        }
        let servers = &mut lsp["servers"];
        if servers.get(name).is_none_or(Item::is_none) {
            set(servers, name, value(toml_edit::InlineTable::new()));
        }
        set(&mut servers[name], "enabled", value(on));
    }

    /// `chord` no formato do config (`"ctrl+shift+p"`). Comando vazio desliga o atalho.
    pub fn bind(&mut self, chord: &str, command: &str) {
        self.section("keys")[chord] = value(command);
    }

    /// Tira a linha do atalho deste arquivo. `true` se ela existia.
    pub fn remove_key(&mut self, chord: &str) -> bool {
        self.doc.get_mut("keys").and_then(Item::as_table_like_mut).and_then(|t| t.remove(chord)).is_some()
    }
}

/// Põe `key = v` numa tabela normal ou inline (indexar uma inline com chave nova dá pânico).
fn set(item: &mut Item, key: &str, v: Item) {
    if let Some(t) = item.as_table_like_mut() {
        t.insert(key, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn parse(text: &str) -> Config {
        Config::from_layers(&[("t".into(), text.into())])
    }

    #[test]
    fn edits_keep_comments_and_parse_back() {
        let dir = std::env::temp_dir().join(format!("idetea-cfg-{}", std::process::id()));
        let path = dir.join("sub").join("config.toml");
        let mut f = ConfigFile::open_global(&path).unwrap();
        f.set_theme("Nord");
        f.set_font_size(16.0);
        f.set_splash(false);
        f.set_extension_enabled("git", false);
        f.set_extension_slot("search", Slot::Right);
        f.set_extension_enabled("nova", true);
        f.set_lsp_server_enabled("clangd", false);
        f.bind("ctrl+alt+t", "terminal.focus");
        f.save().unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# Configuração da IDE TEA."), "comentários preservados");
        assert!(text.contains("font_size = 16\n"), "inteiro sem .0: {text}");
        let c = parse(&text);
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert_eq!(c.ui.theme, "Nord");
        assert!(!c.ui.splash);
        assert!(!c.extension("git").enabled);
        assert_eq!(c.extension("search").slot, Some(Slot::Right));
        assert!(c.extension("nova").enabled);
        assert_eq!(c.lsp.servers["clangd"].enabled, Some(false));
        assert!(c.keys.contains(&("ctrl+alt+t".into(), "terminal.focus".into())));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn project_file_starts_small_and_remove_key() {
        let path = std::env::temp_dir().join("idetea-nao-existe").join("config.toml");
        let mut f = ConfigFile::open_project(&path).unwrap();
        f.bind("ctrl+b", "");
        f.set_lsp_server_enabled("gopls", false);
        let text = f.text();
        assert!(!text.contains("[ui]"), "só o que mudou: {text}");
        assert!(!text.contains("[lsp]\n"), "sem cabeçalho vazio: {text}");
        assert!(parse(&text).errors.is_empty());
        assert!(f.remove_key("ctrl+b"));
        assert!(!f.remove_key("ctrl+b"));
    }
}
