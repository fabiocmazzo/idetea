//! Tela de configurações. Só desenha e guarda o estado da tela; gravar no
//! arquivo e recarregar fica com o app (`App::settings_update`).

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use iced::widget::{
    Id, button, column, container, pick_list, row, rule, scrollable, space, text, text_input, toggler,
};
use iced::{Element, Fill, Font, Theme};
use tea_core::Slot;
use tea_core::i18n::{self, Language};
use tea_core::config::Config;
use tea_core::icons::{Icon, icon_button};
use tea_core::keymap::Keymap;

use crate::palette::CommandInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Appearance,
    Extensions,
    Shortcuts,
    Lsp,
}

/// Em qual arquivo as mudanças são gravadas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Global,
    Project,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Tab(Tab),
    Target(Target),
    Theme(ThemeChoice),
    FontSize(f32),
    Splash(bool),
    RestoreSession(bool),
    ActivityBar(bool),
    Language(LanguageChoice),
    ExtEnabled(String, bool),
    ExtSlot(String, SlotChoice),
    LspEnabled(bool),
    LspServer(String, bool),
    RestartLsp,
    Filter(String),
    Record(String),
    CancelRecord,
    Unbind { chord: String, command: String },
    OpenFile,
    Close,
}

/// Extensão na lista: id, se está carregada agora e se tem painel que pode mudar de lado.
pub struct ExtRow {
    pub id: &'static str,
    pub loaded: bool,
    pub movable: bool,
}

/// Servidor de linguagem na lista.
pub struct ServerRow {
    pub name: String,
    pub command: String,
    pub found: bool,
}

pub struct Settings {
    pub tab: Tab,
    pub target: Target,
    pub filter: String,
    /// Comando esperando a nova tecla.
    pub recording: Option<String>,
    /// Aviso da gravação de tecla (ex.: "use Ctrl, Alt ou uma tecla de função").
    pub record_hint: Option<String>,
    pub servers: Vec<ServerRow>,
    pub extensions: Vec<ExtRow>,
    pub project_path: PathBuf,
}

pub fn filter_id() -> Id {
    Id::new("settings-filter")
}

pub const FONT_MIN: f32 = 8.0;
pub const FONT_MAX: f32 = 32.0;

impl Settings {
    pub fn new(servers: Vec<ServerRow>, extensions: Vec<ExtRow>, project_path: PathBuf) -> Self {
        Self {
            tab: Tab::Appearance,
            target: Target::Global,
            filter: String::new(),
            recording: None,
            record_hint: None,
            servers,
            extensions,
            project_path,
        }
    }

    pub fn view<'a>(
        &'a self,
        config: &'a Config,
        keymap: &'a Keymap,
        commands: &'a [CommandInfo],
        global_path: Option<&'a Path>,
    ) -> Element<'a, Msg> {
        let tab_button = |label: String, tab: Tab| {
            let style = if self.tab == tab { button::secondary } else { button::text };
            button(text(label).size(13)).on_press(Msg::Tab(tab)).style(style).padding([4, 12])
        };
        let header = row![
            text(fl!("settings-title")).size(18),
            space::horizontal().width(16),
            tab_button(fl!("settings-tab-appearance"), Tab::Appearance),
            tab_button(fl!("settings-tab-extensions"), Tab::Extensions),
            tab_button(fl!("settings-tab-shortcuts"), Tab::Shortcuts),
            tab_button("LSP".into(), Tab::Lsp),
            space::horizontal(),
            icon_button(Icon::Close, fl!("close-esc"), Some(Msg::Close)),
        ]
        .spacing(4)
        .align_y(iced::Center);

        let target_button = |label: String, t: Target| {
            let style = if self.target == t { button::primary } else { button::text };
            button(text(label).size(12)).on_press(Msg::Target(t)).style(style).padding([2, 10])
        };
        let path = match self.target {
            Target::Global => global_path.map_or(fl!("settings-no-home"), |p| p.display().to_string()),
            Target::Project => self.project_path.display().to_string(),
        };
        let target = row![
            text(fl!("settings-write-to")).size(12).style(text::secondary),
            target_button(fl!("settings-target-global"), Target::Global),
            target_button(fl!("settings-target-project"), Target::Project),
            text(path).size(11).font(Font::MONOSPACE).style(text::secondary),
            space::horizontal(),
            button(text(fl!("settings-open-file")).size(12)).on_press(Msg::OpenFile).style(button::text).padding([2, 8]),
        ]
        .spacing(8)
        .align_y(iced::Center);

        let body: Element<'a, Msg> = match self.tab {
            Tab::Appearance => appearance(config),
            Tab::Extensions => extensions_view(config, &self.extensions),
            Tab::Shortcuts => self.shortcuts(keymap, commands),
            Tab::Lsp => self.lsp(config),
        };
        let content = column![header, target, rule::horizontal(1), body].spacing(10);
        crate::overlay::modal(content, 860.0, Msg::Close)
    }

    fn shortcuts<'a>(&'a self, keymap: &'a Keymap, commands: &'a [CommandInfo]) -> Element<'a, Msg> {
        let filter = text_input(&fl!("keys-filter"), &self.filter)
            .id(filter_id())
            .on_input(Msg::Filter)
            .size(14)
            .padding([6, 10]);
        let q = self.filter.trim().to_lowercase();
        let mut rows: BTreeMap<&str, (&CommandInfo, Vec<String>)> = BTreeMap::new();
        for c in commands {
            let keys: Vec<String> = keymap.chords_for(&c.id).iter().map(|k| k.to_string()).collect();
            let hay = format!("{} {} {}", c.title, c.id, keys.join(" ")).to_lowercase();
            if q.is_empty() || hay.contains(&q) {
                rows.entry(c.title.as_str()).or_insert((c, keys));
            }
        }
        let mut list = column![].spacing(2);
        for (c, keys) in rows.values() {
            let recording = self.recording.as_deref() == Some(c.id.as_str());
            let mut caps = row![].spacing(6).align_y(iced::Center);
            for k in keys {
                caps = caps.push(key_cap(k.clone(), Msg::Unbind { chord: config_chord(k), command: c.id.clone() }));
            }
            let action: Element<'a, Msg> = if recording {
                row![
                    text(self.record_hint.clone().unwrap_or_else(|| fl!("keys-press")))
                        .size(12)
                        .style(text::primary),
                    icon_button(Icon::Close, fl!("cancel"), Some(Msg::CancelRecord)),
                ]
                .spacing(4)
                .align_y(iced::Center)
                .into()
            } else {
                icon_button(Icon::Add, fl!("keys-add"), Some(Msg::Record(c.id.clone())))
            };
            list = list.push(
                row![
                    column![
                        text(c.title.as_str()).size(13),
                        text(c.id.as_str()).size(11).font(Font::MONOSPACE).style(text::secondary)
                    ]
                    .width(Fill),
                    caps,
                    action,
                ]
                .spacing(10)
                .align_y(iced::Center)
                .padding([3, 0]),
            );
        }
        column![
            filter,
            text(fl!("keys-help"))
                .size(11)
                .style(text::secondary),
            scrollable(list.padding([0, 12])).height(Fill)
        ]
        .spacing(8)
        .into()
    }

    fn lsp<'a>(&'a self, config: &'a Config) -> Element<'a, Msg> {
        let mut list = column![setting_owned(
            fl!("settings-lsp"),
            fl!("settings-lsp-detail"),
            toggler(config.lsp.enabled).on_toggle(Msg::LspEnabled).into(),
        )]
        .spacing(4);
        list = list.push(section_title(fl!("settings-servers")));
        for s in &self.servers {
            let on = config.lsp.servers.get(&s.name).and_then(|o| o.enabled) != Some(false);
            let status = if s.found { fl!("settings-server-found") } else { fl!("settings-server-not-found") };
            let detail = format!("{} · {status}", s.command);
            let name = s.name.clone();
            list = list.push(setting_owned(
                s.name.clone(),
                detail,
                toggler(on).on_toggle(move |v| Msg::LspServer(name.clone(), v)).into(),
            ));
        }
        list = list.push(
            row![
                text(fl!("settings-lsp-note")).size(11).style(text::secondary),
                space::horizontal(),
                button(text(fl!("settings-restart-servers")).size(12)).on_press(Msg::RestartLsp).padding([3, 10]),
            ]
            .align_y(iced::Center)
            .padding([10, 0]),
        );
        scrollable(list.padding([0, 12])).height(Fill).into()
    }
}

/// Tema na lista: o valor gravado e o nome mostrado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeChoice(pub String);

impl fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_str() {
            "dark" => f.write_str(&fl!("theme-dark")),
            "light" => f.write_str(&fl!("theme-light")),
            other => f.write_str(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotChoice(pub Slot);

impl fmt::Display for SlotChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self.0 {
            Slot::Left => fl!("slot-left"),
            Slot::Right => fl!("slot-right"),
            Slot::Bottom => fl!("slot-bottom"),
            Slot::Status => fl!("slot-status"),
        })
    }
}

/// Opção da lista de idiomas: automático (o do sistema) ou um idioma.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageChoice {
    Auto,
    Fixed(Language),
}

impl LanguageChoice {
    /// Valor de `[ui] language`.
    pub fn tag(self) -> &'static str {
        match self {
            LanguageChoice::Auto => "auto",
            LanguageChoice::Fixed(l) => l.tag(),
        }
    }

    fn from_setting(setting: &str) -> Self {
        let s = setting.trim();
        if s.is_empty() || s.eq_ignore_ascii_case("auto") {
            return LanguageChoice::Auto;
        }
        Language::from_tag(s).map_or(LanguageChoice::Auto, LanguageChoice::Fixed)
    }
}

impl fmt::Display for LanguageChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageChoice::Auto => {
                let system = i18n::resolve("auto", i18n::system_locale().as_deref());
                f.write_str(&fl!("settings-language-auto", language = system.native_name()))
            }
            LanguageChoice::Fixed(l) => f.write_str(l.native_name()),
        }
    }
}

fn language_choices() -> Vec<LanguageChoice> {
    std::iter::once(LanguageChoice::Auto).chain(Language::ALL.map(LanguageChoice::Fixed)).collect()
}

const SLOTS: [SlotChoice; 3] = [SlotChoice(Slot::Left), SlotChoice(Slot::Right), SlotChoice(Slot::Bottom)];

fn theme_choices() -> Vec<ThemeChoice> {
    let mut v = vec![ThemeChoice("dark".into()), ThemeChoice("light".into())];
    v.extend(Theme::ALL.iter().map(|t| ThemeChoice(t.to_string())));
    v
}

fn appearance(config: &Config) -> Element<'_, Msg> {
    let size = config.ui.font_size;
    let font = row![
        button(text("−").size(14)).on_press_maybe((size > FONT_MIN).then_some(Msg::FontSize(size - 1.0))).padding([2, 10]),
        text(format!("{size}")).size(14).width(36).align_x(iced::Center),
        button(text("+").size(14)).on_press_maybe((size < FONT_MAX).then_some(Msg::FontSize(size + 1.0))).padding([2, 10]),
    ]
    .spacing(6)
    .align_y(iced::Center);
    let current = theme_choices().into_iter().find(|t| t.0.eq_ignore_ascii_case(config.ui.theme.trim()));
    // O idioma só muda ao reiniciar: avisa quando o escolhido não é o que está na tela.
    let chosen = i18n::resolve(&config.ui.language, i18n::system_locale().as_deref());
    let mut language_detail = fl!("settings-language-detail");
    if chosen != i18n::current() {
        language_detail = format!(
            "{language_detail} {}",
            fl!("settings-language-restart", language = i18n::current().native_name())
        );
    }
    let language = pick_list(language_choices(), Some(LanguageChoice::from_setting(&config.ui.language)), Msg::Language)
        .text_size(13)
        .width(220);
    column![
        setting_owned(
            fl!("settings-theme"),
            fl!("settings-theme-detail"),
            pick_list(theme_choices(), current, Msg::Theme).text_size(13).width(220).into()
        ),
        setting_owned(fl!("settings-language"), language_detail, language.into()),
        setting_owned(fl!("settings-font-size"), fl!("settings-font-size-detail"), font.into()),
        setting_owned(
            fl!("settings-activity-bar"),
            fl!("settings-activity-bar-detail"),
            toggler(config.ui.activity_bar).on_toggle(Msg::ActivityBar).into()
        ),
        setting_owned(
            fl!("settings-splash"),
            fl!("settings-splash-detail"),
            toggler(config.ui.splash).on_toggle(Msg::Splash).into()
        ),
        setting_owned(
            fl!("settings-restore-session"),
            fl!("settings-restore-session-detail"),
            toggler(config.ui.restore_session).on_toggle(Msg::RestoreSession).into()
        ),
    ]
    .spacing(4)
    .padding([0, 12])
    .into()
}

fn extensions_view<'a>(config: &'a Config, extensions: &'a [ExtRow]) -> Element<'a, Msg> {
    let mut list = column![].spacing(4);
    for e in extensions {
        let cfg = config.extension(e.id);
        let restart = if cfg.enabled != e.loaded { format!(" · {}", fl!("settings-on-restart")) } else { String::new() };
        let id = e.id.to_string();
        let enabled = toggler(cfg.enabled).on_toggle(move |v| Msg::ExtEnabled(id.clone(), v));
        let mut controls = row![].spacing(12).align_y(iced::Center);
        if e.movable {
            let id = e.id.to_string();
            let current = cfg.slot.map(SlotChoice);
            controls = controls.push(
                pick_list(SLOTS, current, move |s| Msg::ExtSlot(id.clone(), s))
                    .placeholder(fl!("default-capitalized"))
                    .text_size(13)
                    .width(130),
            );
        }
        controls = controls.push(enabled);
        list = list.push(setting_owned(e.id.to_string(), format!("{}{restart}", describe(e.id)), controls.into()));
    }
    list = list.push(
        text(fl!("settings-extensions-note"))
            .size(11)
            .style(text::secondary),
    );
    scrollable(list.padding([0, 12])).height(Fill).into()
}

fn describe(id: &str) -> String {
    match id {
        "filetree" => fl!("ext-filetree"),
        "statusbar" => fl!("ext-statusbar"),
        "search" => fl!("ext-search"),
        "problems" => fl!("ext-problems"),
        "terminal" => fl!("ext-terminal"),
        "git" => fl!("ext-git"),
        "debug" => fl!("ext-debug"),
        "references" => fl!("ext-references"),
        "outline" => fl!("ext-outline"),
        "preview" => fl!("ext-preview"),
        "calls" => fl!("ext-calls"),
        _ => String::new(),
    }
}

fn setting_owned<'a>(title: String, detail: String, control: Element<'a, Msg>) -> Element<'a, Msg> {
    row![column![text(title).size(14), text(detail).size(11).style(text::secondary)].spacing(2).width(Fill), control]
        .spacing(12)
        .align_y(iced::Center)
        .padding([8, 0])
        .into()
}

fn section_title<'a>(title: String) -> Element<'a, Msg> {
    container(text(title.to_uppercase()).size(11).style(text::secondary)).padding([10, 0]).into()
}

/// Tecla clicável (clique remove o atalho).
fn key_cap<'a>(key: String, on_press: Msg) -> Element<'a, Msg> {
    button(row![text(key).size(12).font(Font::MONOSPACE), text("×").size(11).style(text::secondary)].spacing(6))
        .on_press(on_press)
        .padding([1, 6])
        .style(|theme: &Theme, status| {
            let p = theme.extended_palette();
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => p.danger.weak.color,
                _ => p.background.strong.color,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: p.background.base.text,
                border: iced::Border { color: p.background.strongest.color, width: 1.0, radius: 4.0.into() },
                ..button::Style::default()
            }
        })
        .into()
}

/// "Ctrl+Shift+P" (como mostrado) → "ctrl+shift+p" (como no config).
fn config_chord(shown: &str) -> String {
    tea_core::keymap::Chord::parse(shown).map(|c| c.to_config()).unwrap_or_else(|_| shown.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shown_chord_back_to_config() {
        assert_eq!(config_chord("Ctrl+Shift+P"), "ctrl+shift+p");
        assert_eq!(config_chord("Ctrl+PageDown"), "ctrl+pagedown");
        assert_eq!(config_chord("F12"), "f12");
    }

    #[test]
    fn theme_list_has_defaults_first() {
        let t = theme_choices();
        assert_eq!(t[0].to_string(), "Escuro (padrão)");
        assert!(t.iter().any(|c| c.0 == "Nord"));
    }
}
