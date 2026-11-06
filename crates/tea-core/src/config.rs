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
# Splash screen ao abrir (mínimo de 2 s). Desligada por padrão; ligada, --no-splash ou IDETEA_NO_SPLASH=1
# desligam numa execução.
splash = false
# "dark", "light" ou o nome de um tema do iced (ex.: "Tokyo Night", "Nord", "Dracula").
theme = "dark"
font_size = 14
# Faixa de ícones na borda esquerda para mostrar/ocultar painéis.
activity_bar = true
# Ícone de cada arquivo pela linguagem ou pelo tipo (árvore, abas, Ctrl+P, breadcrumbs, busca, git):
# "seti" (o conjunto do VS Code, embutido) ou "none" (o ícone genérico).
file_icons = "seti"
# Idioma da interface: "auto" (o do sistema), "pt-BR", "en" ou "es". Vale ao reiniciar.
language = "auto"
# Reabrir as abas e os painéis de cada projeto como estavam (não vale para `idetea arquivo...`).
restore_session = true

# Editor. Um `.editorconfig` no projeto vale mais que isto (tamanho do recuo, tabs ou espaços, limpar ao
# salvar). Sem `.editorconfig`, um arquivo que já usa tabs para recuar continua com tabs.
[editor]
# Quebrar linhas longas na largura da janela.
wrap = true
# Mostrar espaços (·) e tabs (→).
show_whitespace = true
# Linhas verticais finas em cada nível de recuo.
indent_guides = true
# Tamanho do recuo (Tab insere espaços) e se usa espaços ou o caractere tab.
tab_size = 4
insert_spaces = true
# Ao salvar: tirar espaços no fim das linhas (menos em Markdown, onde dois espaços quebram a linha),
# garantir a quebra de linha no fim do arquivo, e formatar com o LSP (só no Ctrl+S).
trim_trailing_whitespace = false
insert_final_newline = false
format_on_save = false
# Acima do editor, o caminho do arquivo e o símbolo onde está o cursor (pasta › arquivo › impl › fn).
breadcrumbs = true
# Fonte do código (editor, números de linha e terminal). A JetBrains Mono vem com a IDE; vale qualquer fonte
# instalada pelo nome (ex.: "Fira Code", "Cascadia Code", "DejaVu Sans Mono") ou "monospace" (a do sistema).
# Ligaduras (=>, !=) aparecem quando a fonte tem.
font = "JetBrains Mono"
# thin, extra-light, light, normal, medium, semibold, bold, extra-bold, black.
font_weight = "normal"
# Altura da linha, em múltiplos do tamanho da fonte (1.0 a 2.5).
line_height = 1.45

# Tarefas (Ctrl+Shift+B): "nome" = "comando", rodam na raiz do projeto com a saída no terminal. Além destas,
# a lista traz o que achar no projeto (cargo, scripts do package.json, alvos do Makefile).
# Costumam ficar no config do projeto (.idetea/config.toml).
[tasks]
# "servidor" = "cargo run -- --porta 8080"

# Servidores de linguagem (LSP). Embutidos: rust-analyzer, clangd, pyright,
# typescript, gopls, lua, zls (Zig) e zxls (Ziex, `zx lsp`). Só iniciam se o comando existir no PATH.
[lsp]
enabled = true
# Exemplos:
# servers.rust-analyzer = { command = "/caminho/rust-analyzer" }
# servers.clangd = { enabled = false }
# servers.zls = { command = "/caminho/zls" }
# servers.svelte = { command = "svelteserver", args = ["--stdio"], languages = { svelte = "svelte" } }

# Debug (F5). Sem nada aqui: em projeto Rust compila com `cargo build` e acha o executável sozinho;
# nos outros, pergunta o executável. Costuma ficar no config do projeto (.idetea/config.toml).
[debug]
# program = "build/meu-app"        # relativo à raiz do projeto
# args = ["--verbose"]
# cwd = "."                        # pasta onde o programa roda (padrão: a raiz)
# build = "make"                   # roda antes de depurar; "" = não compilar

# Extensões: enabled = liga/desliga (precisa reiniciar); slot = "left", "right" ou "bottom".
[extensions]
filetree = { enabled = true, slot = "left" }
statusbar = { enabled = true }
search = { enabled = true, slot = "bottom" }
problems = { enabled = true, slot = "bottom" }
terminal = { enabled = true, slot = "bottom" }
git = { enabled = true, slot = "bottom" }
debug = { enabled = true, slot = "bottom" }
references = { enabled = true, slot = "bottom" }
outline = { enabled = true, slot = "right" }
calls = { enabled = true, slot = "bottom" }
preview = { enabled = true, slot = "right" }
# Interfaces que um agente escreve em .idetea/ui/ (docs/interfaces/formato.md).
interfaces = { enabled = true, slot = "right" }

# Pedidos de terminal dos agentes (.idetea/agents/, docs/terminais-de-agente.md). Cada pedido é confirmado,
# menos os desta lista, que rodam direto. Só vale neste arquivo (o do usuário): no config do projeto é ignorado,
# porque um agente pode escrever lá. "comando exato", ou "começo *": o começo e argumentos simples depois
# (sem ; & | $ ` < > ( ) { } \ nem quebra de linha).
[agents]
allow = []
# allow = ["npm run dev", "claude *"]

# Plugins (~/.local/share/idetea/plugins/<id>/): a config de cada um, no formato que o plugin pedir.
# [plugins.contador]
# comando = ["git", "status", "--short"]

# Atalhos: "combinação" = "comando". Comando vazio ("") desliga o atalho.
# Atalhos que o editor já usa (Ctrl+F, Ctrl+H, Ctrl+G, Ctrl+Z, Ctrl+D...) têm prioridade quando ele tem foco.
[keys]
"ctrl+shift+p" = "palette.open"
"f1" = "help.open"
"ctrl+s" = "file.save"
"ctrl+shift+s" = "file.save_as"
"ctrl+o" = "file.open"
"ctrl+alt+s" = "file.save_all"
"ctrl+r" = "file.open_recent"
"ctrl+p" = "file.quick_open"
"ctrl+shift+o" = "nav.file_symbol"
"ctrl+t" = "nav.project_symbol"
"ctrl+n" = "file.new"
"ctrl+w" = "file.close"
"ctrl+pagedown" = "tab.next"
"ctrl+pageup" = "tab.prev"
"ctrl+\\" = "editor.split"
"ctrl+1" = "editor.focus_left"
"ctrl+2" = "editor.focus_right"
"ctrl+b" = "filetree.toggle"
"ctrl+shift+e" = "filetree.reveal"
"ctrl+shift+f" = "search.focus"
"ctrl+shift+h" = "search.replace"
"ctrl+shift+j" = "view.toggle_bottom"
"ctrl+j" = "view.toggle_bottom"
"ctrl+shift+m" = "problems.focus"
"f8" = "problems.next"
"f12" = "lsp.definition"
"shift+f12" = "lsp.references"
"ctrl+f12" = "lsp.implementation"
"alt+shift+h" = "lsp.incoming_calls"
"alt+left" = "nav.back"
"alt+right" = "nav.forward"
"alt+7" = "outline.toggle"
"ctrl+shift+v" = "preview.toggle"
"f9" = "debug.toggle_breakpoint"
"f5" = "debug.continue"
"shift+f5" = "debug.stop"
"f6" = "debug.pause"
"f10" = "debug.next"
"f11" = "debug.step_in"
"shift+f11" = "debug.step_out"
"ctrl+shift+i" = "lsp.format"
"f2" = "lsp.rename"
"ctrl+'" = "terminal.focus"
"ctrl+`" = "terminal.focus"
"ctrl+shift+t" = "tab.reopen"
"ctrl+shift+[" = "editor.toggle_fold"
"ctrl+shift+]" = "editor.toggle_fold"
"ctrl+shift+b" = "tasks.run"
"ctrl+shift+r" = "tasks.rerun"
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
    /// Ícones de arquivo pela linguagem: `"seti"` (embutido) ou `"none"` (o genérico).
    pub file_icons: String,
    /// `"auto"` (o do sistema), `"pt-BR"`, `"en"` ou `"es"`. Vale ao reiniciar.
    pub language: String,
    /// Reabrir as abas e os painéis do projeto como estavam.
    pub restore_session: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            splash: false,
            theme: "dark".into(),
            font_size: 14.0,
            activity_bar: true,
            file_icons: "seti".into(),
            language: "auto".into(),
            restore_session: true,
        }
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

/// Editor (`[editor]`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub wrap: bool,
    pub show_whitespace: bool,
    pub indent_guides: bool,
    pub tab_size: u8,
    pub insert_spaces: bool,
    pub trim_trailing_whitespace: bool,
    pub insert_final_newline: bool,
    pub format_on_save: bool,
    /// Caminho do arquivo e símbolo do cursor acima do editor.
    pub breadcrumbs: bool,
    /// Fonte do código (editor e terminal). Vazio = a embutida (ver [`crate::fonts`]).
    pub font: String,
    pub font_weight: String,
    /// Altura da linha, em múltiplos do tamanho da fonte.
    pub line_height: f32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            wrap: true,
            show_whitespace: true,
            indent_guides: true,
            tab_size: 4,
            insert_spaces: true,
            trim_trailing_whitespace: false,
            insert_final_newline: false,
            format_on_save: false,
            breadcrumbs: true,
            font: crate::fonts::DEFAULT_FAMILY.into(),
            font_weight: "normal".into(),
            line_height: crate::fonts::LINE_HEIGHT_DEFAULT,
        }
    }
}

/// Pedidos de terminal dos agentes (`[agents]`). Só vem do config do usuário.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentsConfig {
    /// Comandos que rodam sem confirmar: o comando exato, ou `"começo *"`.
    pub allow: Vec<String>,
}

/// Como depurar o projeto (`[debug]`). Tudo opcional.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DebugConfig {
    /// Executável, relativo à raiz (ou absoluto).
    pub program: Option<String>,
    pub args: Vec<String>,
    /// Pasta onde o programa roda, relativa à raiz.
    pub cwd: Option<String>,
    /// Comando que compila antes de depurar. `Some("")` = não compilar; `None` = automático (cargo).
    pub build: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub ui: UiConfig,
    pub editor: EditorConfig,
    pub lsp: LspConfig,
    pub debug: DebugConfig,
    /// Tarefas do usuário: nome → comando.
    pub tasks: BTreeMap<String, String>,
    /// `[agents]`, só do config do usuário (o do projeto é ignorado, com aviso).
    pub agents: AgentsConfig,
    pub extensions: BTreeMap<String, ExtensionConfig>,
    /// Config de cada plugin (`[plugins.<id>]`): o plugin lê com `host.config` (formato livre).
    pub plugins: BTreeMap<String, toml::Table>,
    /// Atalhos na ordem das camadas (os últimos vencem).
    pub keys: Vec<(String, String)>,
    /// Problemas encontrados ao ler (o config segue com o que deu para ler).
    pub errors: Vec<String>,
}

/// Só o `[ui] language` dos arquivos (o do projeto vence), sem conferir o resto. Lido antes de
/// [`Config::load`] para os avisos do config já saírem no idioma escolhido.
pub fn peek_language(global: Option<&Path>, project: Option<&Path>) -> String {
    let mut language = String::from("auto");
    for path in [global, project].into_iter().flatten() {
        let Some(table) = std::fs::read_to_string(path).ok().and_then(|t| t.parse::<toml::Table>().ok()) else {
            continue;
        };
        if let Some(l) = table.get("ui").and_then(|u| u.get("language")).and_then(|l| l.as_str()) {
            language = l.to_string();
        }
    }
    language
}

/// Atalhos que já foram padrão e mudaram. O config do usuário é criado com uma cópia do padrão (as versões
/// até a 0.4 copiavam também o `[keys]`): uma linha igual a uma destas é dessa cópia, não escolha do usuário,
/// e fica de fora para valer o padrão de agora.
const RETIRED_DEFAULT_KEYS: &[(&str, &str)] = &[
    ("ctrl+shift+t", "theme.toggle"),
    ("ctrl+shift+t", "terminal.focus"),
    ("ctrl+shift+k", "lsp.hover"),
];

fn is_retired_default(chord: &str, command: &str) -> bool {
    let chord = chord.to_lowercase().replace(' ', "");
    RETIRED_DEFAULT_KEYS.iter().any(|(c, cmd)| *c == chord && *cmd == command)
}

/// Texto de um config de usuário novo: o padrão com o `[keys]` comentado, para que os atalhos padrão de versões
/// futuras cheguem a ele (só as linhas que o usuário descomentar ou escrever valem por cima).
pub fn user_config_template() -> String {
    let mut in_keys = false;
    DEFAULT_CONFIG
        .lines()
        .map(|line| {
            if line.starts_with('[') {
                in_keys = line == "[keys]";
            }
            if in_keys && !line.is_empty() && !line.starts_with('#') { format!("# {line}") } else { line.to_string() }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

impl Config {
    /// Carrega padrões + arquivos que existirem.
    pub fn load(global: Option<&Path>, project: Option<&Path>) -> Self {
        let mut layers = vec![("padrão".to_string(), DEFAULT_CONFIG.to_string())];
        let mut errors = Vec::new();
        let mut project_layer = None;
        for (path, is_project) in [(global, false), (project, true)].into_iter().filter_map(|(p, i)| p.map(|p| (p, i))) {
            match std::fs::read_to_string(path) {
                Ok(text) => {
                    if is_project {
                        project_layer = Some(layers.len());
                    }
                    layers.push((path.display().to_string(), text));
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }
        let mut config = Self::merge(&layers, project_layer);
        config.errors.splice(0..0, errors);
        config
    }

    /// Junta camadas de TOML `(nome, texto)`. As últimas sobrepõem as primeiras.
    pub fn from_layers(layers: &[(String, String)]) -> Self {
        Self::merge(layers, None)
    }

    /// Como [`Config::from_layers`]; `project` é o índice da camada do projeto, que não pode ter `[agents]`.
    fn merge(layers: &[(String, String)], project: Option<usize>) -> Self {
        let mut merged = toml::Table::new();
        let mut keys = Vec::new();
        let mut errors = Vec::new();
        for (i, (name, text)) in layers.iter().enumerate() {
            let mut table: toml::Table = match text.parse() {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("{name}: {e}"));
                    continue;
                }
            };
            // Um agente escreve na pasta do projeto: lá ele não pode se dar permissão.
            if project == Some(i) && table.remove("agents").is_some() {
                errors.push(fl!("config-agents-project", layer = name.as_str()));
            }
            // `keys` é juntado à parte para manter a ordem das camadas.
            if let Some(layer_keys) = table.remove("keys") {
                match layer_keys {
                    toml::Value::Table(t) => {
                        for (chord, cmd) in t {
                            match cmd {
                                // Cópia antiga do padrão num config do usuário: vale o padrão novo.
                                toml::Value::String(cmd) if i > 0 && is_retired_default(&chord, &cmd) => {}
                                toml::Value::String(cmd) => keys.push((chord, cmd)),
                                other => errors.push(fl!("config-key-not-text", layer = name.as_str(), chord = format!("{chord:?}"), value = other.to_string())),
                            }
                        }
                    }
                    _ => errors.push(fl!("config-keys-not-table", layer = name.as_str())),
                }
            }
            deep_merge(&mut merged, table);
        }

        let ui = take_section(&mut merged, "ui", &mut errors);
        let mut editor: EditorConfig = take_section(&mut merged, "editor", &mut errors);
        editor.tab_size = editor.tab_size.clamp(1, 16);
        let extensions = take_section(&mut merged, "extensions", &mut errors);
        let lsp = take_section(&mut merged, "lsp", &mut errors);
        let debug = take_section(&mut merged, "debug", &mut errors);
        let tasks = take_section(&mut merged, "tasks", &mut errors);
        let agents = take_section(&mut merged, "agents", &mut errors);
        let plugins = take_section(&mut merged, "plugins", &mut errors);
        for unknown in merged.keys() {
            errors.push(fl!("config-unknown-section", section = unknown.as_str()));
        }
        Self { ui, editor, lsp, debug, tasks, agents, extensions, plugins, keys, errors }
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
        assert!(!c.ui.splash, "a splash vem desligada");
        assert_eq!(c.editor, EditorConfig::default(), "o texto e o Default dizem o mesmo");
        assert_eq!(layers(&["[editor]\ntab_size = 0"]).editor.tab_size, 1);
        assert!(c.tasks.is_empty());
        assert_eq!(layers(&["[tasks]\nservidor = \"cargo run\""]).tasks["servidor"], "cargo run");
        assert_eq!(c.extension("filetree").slot, Some(Slot::Left));
        assert!(c.keys.iter().any(|(k, v)| k == "ctrl+s" && v == "file.save"));
    }

    #[test]
    fn agents_allow_only_from_the_user_config() {
        let dir = std::env::temp_dir().join(format!("idetea-config-agents-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (global, project) = (dir.join("global.toml"), dir.join("project.toml"));
        std::fs::write(&global, "[agents]\nallow = [\"npm run dev\"]\n").unwrap();
        std::fs::write(&project, "[agents]\nallow = [\"rm -rf ~\"]\n").unwrap();
        let c = Config::load(Some(&global), Some(&project));
        assert_eq!(c.agents.allow, ["npm run dev"], "o do projeto não soma nem troca");
        assert_eq!(c.errors.len(), 1, "{:?}", c.errors);
        let c = Config::load(None, Some(&project));
        assert!(c.agents.allow.is_empty());
        assert!(layers(&[DEFAULT_CONFIG]).agents.allow.is_empty());
        assert!(!layers(&["[agents]\nallows = []"]).errors.is_empty(), "campo desconhecido avisa");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn later_layers_win() {
        let c = layers(&[
            DEFAULT_CONFIG,
            "[ui]\nsplash = true\n[extensions]\nfiletree = { slot = \"right\" }",
            "[ui]\ntheme = \"light\"\n[keys]\n\"ctrl+s\" = \"\"",
        ]);
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert!(c.ui.splash);
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
        assert!(!c.ui.splash); // [ui] inválido cai no padrão
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

    #[test]
    fn retired_default_keys_in_a_user_file_give_way_to_the_new_defaults() {
        let user = "[keys]\n\"ctrl+shift+t\" = \"terminal.focus\"\n\"ctrl+shift+k\" = \"lsp.hover\"\n\"ctrl+e\" = \"terminal.focus\"";
        let c = layers(&[DEFAULT_CONFIG, user]);
        let last = |chord: &str| c.keys.iter().rev().find(|(k, _)| k == chord).map(|(_, v)| v.as_str());
        assert_eq!(last("ctrl+shift+t"), Some("tab.reopen"));
        assert_eq!(last("ctrl+shift+k"), None);
        assert_eq!(last("ctrl+e"), Some("terminal.focus"));
    }

    #[test]
    fn user_template_comments_out_the_keys() {
        let template = user_config_template();
        let c = layers(&[DEFAULT_CONFIG, &template]);
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert_eq!(c.keys.len(), layers(&[DEFAULT_CONFIG]).keys.len());
        assert!(template.contains("[keys]\n"));
        assert!(template.contains("# \"ctrl+s\" = \"file.save\""));
    }
}
