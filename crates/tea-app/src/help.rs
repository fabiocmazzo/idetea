//! Janela de ajuda: atalhos (gerados do mapa de teclas), guia curto e "Sobre".
//!
//! O guia cita comandos como `{file.save}`; na tela isso vira o atalho atual
//! do usuário (ou "paleta" se o comando não tiver atalho).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use iced::widget::{
    Id, button, column, container, rich_text, row, rule, scrollable, space, span, text, text_input,
};
use iced::{Element, Fill, Font, Theme};
use tea_core::icons::{Icon, icon_button};
use tea_core::keymap::Keymap;

use crate::palette::CommandInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Shortcuts,
    Guide,
    About,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Tab(Tab),
    Query(String),
    Close,
}

/// Dados que só o app conhece, para a aba "Sobre".
pub struct AboutInfo<'a> {
    pub root: &'a Path,
    pub global_config: Option<&'a Path>,
    pub project_config: PathBuf,
}

pub struct Help {
    tab: Tab,
    query: String,
}

pub fn input_id() -> Id {
    Id::new("help-search")
}

/// Atalhos do widget de edição: fixos, não passam pelo mapa de teclas.
fn editor_keys() -> Vec<(String, String)> {
    [
        ("Ctrl+F".to_string(), fl!("help-key-find")),
        ("Ctrl+H".to_string(), fl!("help-key-replace")),
        ("Ctrl+G".to_string(), fl!("help-key-goto-line")),
        ("Ctrl+Z / Ctrl+Y".to_string(), fl!("help-key-undo")),
        ("Ctrl+D".to_string(), fl!("help-key-next-match")),
        ("Ctrl+K / Ctrl+J".to_string(), fl!("help-key-fold")),
        ("Ctrl+.".to_string(), fl!("help-key-quick-fix")),
        (fl!("help-key-ctrl-click"), fl!("help-key-definition")),
        ("Tab / Ctrl+Tab".to_string(), fl!("help-key-tab")),
    ]
    .into()
}

/// Guia: (título, parágrafos), ids no `.ftl`. `{ $palette-open }` no texto vira o atalho atual.
const GUIDE: &[(&str, &[&str])] = &[
    ("guide-start", &["guide-start-1", "guide-start-2", "guide-start-3", "guide-start-4"]),
    ("guide-files", &["guide-files-1", "guide-files-2", "guide-files-3", "guide-files-4", "guide-files-5", "guide-files-6", "guide-files-7", "guide-files-8", "guide-files-9", "guide-files-10", "guide-files-11", "guide-files-12"]),
    ("guide-tree", &["guide-tree-1", "guide-tree-2", "guide-tree-3"]),
    ("guide-bottom", &["guide-bottom-1", "guide-bottom-2"]),
    ("guide-terminal", &["guide-terminal-1", "guide-terminal-2"]),
    ("guide-debug", &["guide-debug-1", "guide-debug-2", "guide-debug-3"]),
    ("guide-git", &["guide-git-1", "guide-git-2", "guide-git-3", "guide-git-4"]),
    ("guide-lsp", &["guide-lsp-1", "guide-lsp-2", "guide-lsp-3", "guide-lsp-4", "guide-lsp-5", "guide-lsp-6", "guide-lsp-7", "guide-lsp-8"]),
    ("guide-config", &["guide-config-1", "guide-config-2", "guide-config-3", "guide-config-4"]),
];

impl Help {
    pub fn new(tab: Tab) -> Self {
        Self { tab, query: String::new() }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Tab(tab) => self.tab = tab,
            Msg::Query(q) => self.query = q,
            Msg::Close => {}
        }
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn view<'a>(&'a self, commands: &'a [CommandInfo], keymap: &'a Keymap, about: AboutInfo<'_>) -> Element<'a, Msg> {
        let tab_button = |label: String, tab: Tab| {
            let style = if self.tab == tab { button::secondary } else { button::text };
            button(text(label).size(13)).on_press(Msg::Tab(tab)).style(style).padding([4, 12])
        };
        let header = row![
            text(fl!("help-title")).size(18),
            space::horizontal().width(16),
            tab_button(fl!("help-tab-shortcuts"), Tab::Shortcuts),
            tab_button(fl!("help-tab-guide"), Tab::Guide),
            tab_button(fl!("help-tab-about"), Tab::About),
            space::horizontal(),
            icon_button(Icon::Close, fl!("help-close"), Some(Msg::Close)),
        ]
        .spacing(4)
        .align_y(iced::Center);

        let body: Element<'a, Msg> = match self.tab {
            Tab::Shortcuts => self.shortcuts(commands, keymap),
            Tab::Guide => scrollable(guide(keymap)).height(Fill).into(),
            Tab::About => scrollable(about_view(about)).height(Fill).into(),
        };

        crate::overlay::modal(column![header, rule::horizontal(1), body].spacing(10), 860.0, Msg::Close)
    }

    fn shortcuts<'a>(&'a self, commands: &'a [CommandInfo], keymap: &'a Keymap) -> Element<'a, Msg> {
        let search = text_input(&fl!("help-filter"), &self.query)
            .id(input_id())
            .on_input(Msg::Query)
            .size(14)
            .padding([6, 10]);

        let q = self.query.trim().to_lowercase();
        let matches = |fields: &[&str]| q.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(&q));

        // Agrupa pelo prefixo do título ("Arquivo: salvar" → "Arquivo").
        let general = fl!("help-group-general");
        let mut groups: BTreeMap<&str, Vec<(&CommandInfo, String)>> = BTreeMap::new();
        let mut seen = std::collections::HashSet::new();
        for c in commands {
            if !seen.insert(c.id.as_str()) {
                continue;
            }
            let keys = keymap.chords_for(&c.id).iter().map(|k| k.to_string()).collect::<Vec<_>>().join("  ");
            if !matches(&[&c.title, &c.id, &keys]) {
                continue;
            }
            let group = c.title.split_once(':').map_or(general.as_str(), |(g, _)| g.trim());
            groups.entry(group).or_default().push((c, keys));
        }

        let mut list = column![].spacing(2);
        for (group, mut items) in groups {
            items.sort_by(|a, b| a.0.title.cmp(&b.0.title));
            list = list.push(section_title(group));
            for (c, keys) in items {
                let title = c.title.split_once(':').map_or(c.title.as_str(), |(_, t)| t.trim());
                list = list.push(shortcut_row(title.to_string(), Some(&c.id), keys));
            }
        }
        let editor: Vec<_> = editor_keys().into_iter().filter(|(k, d)| matches(&[k, d])).collect();
        if !editor.is_empty() {
            list = list.push(section_title(&fl!("help-editor-keys")));
            for (keys, desc) in editor {
                list = list.push(shortcut_row(desc, None, keys));
            }
        }
        column![search, scrollable(list.padding([0, 12])).height(Fill)].spacing(8).into()
    }
}

fn section_title<'a>(title: &str) -> Element<'a, Msg> {
    container(text(title.to_uppercase()).size(11).style(text::secondary)).padding([10, 0]).into()
}

fn shortcut_row<'a>(title: String, id: Option<&'a str>, keys: String) -> Element<'a, Msg> {
    let keys: Element<'a, Msg> = if keys.is_empty() {
        text("—").size(12).style(text::secondary).into()
    } else {
        row(keys.split("  ").map(|k| key_cap(k.to_string()))).spacing(6).into()
    };
    row![
        text(title).size(13).width(Fill),
        text(id.unwrap_or_default()).size(11).font(Font::MONOSPACE).style(text::secondary),
        container(keys).align_right(170),
    ]
    .spacing(12)
    .align_y(iced::Center)
    .padding([3, 0])
    .into()
}

fn key_cap<'a>(key: String) -> Element<'a, Msg> {
    container(text(key).size(12).font(Font::MONOSPACE))
        .padding([1, 6])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(p.background.strong.color.into()),
                border: iced::Border { color: p.background.strongest.color, width: 1.0, radius: 4.0.into() },
                ..container::Style::default()
            }
        })
        .into()
}

/// Troca `{comando}` pelo atalho e `` `código` `` por fonte monoespaçada.
pub fn render_paragraph(paragraph: &str, keymap: &Keymap) -> Vec<(String, Chunk)> {
    let mut out = Vec::new();
    let mut rest = paragraph;
    while !rest.is_empty() {
        let next = rest.find(['{', '`']);
        let Some(start) = next else {
            out.push((rest.to_string(), Chunk::Text));
            break;
        };
        if start > 0 {
            out.push((rest[..start].to_string(), Chunk::Text));
        }
        let close = if rest[start..].starts_with('{') { '}' } else { '`' };
        let Some(len) = rest[start + 1..].find(close) else {
            out.push((rest[start..].to_string(), Chunk::Text));
            break;
        };
        let inner = &rest[start + 1..start + 1 + len];
        if close == '}' {
            match keymap.chord_for(inner) {
                Some(k) => out.push((k.to_string(), Chunk::Key)),
                None => out.push((fl!("help-in-palette"), Chunk::Code)),
            }
        } else {
            out.push((inner.to_string(), Chunk::Code));
        }
        rest = &rest[start + 1 + len + 1..];
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chunk {
    Text,
    Key,
    Code,
}

/// Parágrafo do guia no idioma da interface, com os comandos como `{palette.open}`.
fn paragraph(id: &str) -> String {
    tea_core::i18n::get_with_markers(&crate::LOADER, id)
}

fn guide<'a>(keymap: &Keymap) -> Element<'a, Msg> {
    let mut col = column![].spacing(6).padding([0, 12]);
    for (title, paragraphs) in GUIDE {
        col = col.push(container(text(crate::LOADER.get(title)).size(15)).padding([10, 0]));
        for p in *paragraphs {
            let spans: Vec<iced::widget::text::Span<'a, ()>> = render_paragraph(&paragraph(p), keymap)
                .into_iter()
                .map(|(t, kind)| match kind {
                    Chunk::Text => span(t),
                    Chunk::Key => span(t).font(Font { weight: iced::font::Weight::Bold, ..Font::MONOSPACE }),
                    Chunk::Code => span(t).font(Font::MONOSPACE),
                })
                .collect();
            col = col.push(rich_text(spans).size(13));
        }
    }
    col.into()
}

fn about_view<'a>(about: AboutInfo<'_>) -> Element<'a, Msg> {
    let line = |label: &str, value: String| {
        row![
            text(label.to_string()).size(13).width(160).style(text::secondary),
            text(value).size(13).font(Font::MONOSPACE)
        ]
        .spacing(8)
    };
    let global = about.global_config.map_or(fl!("about-no-home"), |p| p.display().to_string());
    column![
        text("IDE TEA").size(24),
        text("Just code. Add ice.").size(14).style(text::secondary),
        space::vertical().height(12),
        line(&fl!("about-version"), env!("CARGO_PKG_VERSION").to_string()),
        line(&fl!("about-license"), fl!("about-license-value", license = env!("CARGO_PKG_LICENSE"))),
        line(&fl!("about-project"), about.root.display().to_string()),
        line(&fl!("about-global-config"), global),
        line(&fl!("about-project-config"), about.project_config.display().to_string()),
        space::vertical().height(12),
        text(fl!("about-built-with")).size(13),
        text(fl!("about-icons")).size(13),
    ]
    .spacing(6)
    .padding([0, 12])
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_placeholders() {
        let (km, _) = Keymap::from_pairs([("ctrl+s", "file.save")]);
        let out = render_paragraph("Salvar: {file.save}, outro: {x.y} e `idetea`.", &km);
        assert_eq!(
            out,
            vec![
                ("Salvar: ".into(), Chunk::Text),
                ("Ctrl+S".into(), Chunk::Key),
                (", outro: ".into(), Chunk::Text),
                ("(paleta)".into(), Chunk::Code),
                (" e ".into(), Chunk::Text),
                ("idetea".into(), Chunk::Code),
                (".".into(), Chunk::Text),
            ]
        );
        assert_eq!(render_paragraph("sem fim {abc", &km).last().unwrap().0, "{abc");
    }

    #[test]
    fn guide_in_each_language() {
        use tea_core::i18n::{Language, get_with_markers, load_in};
        for lang in [Language::English, Language::Spanish] {
            let loader = load_in("tea-app", &crate::Localizations, lang);
            let text = get_with_markers(&loader, "guide-start-2");
            assert!(text.contains("{palette.open}"), "{lang:?}: {text}");
            assert!(!text.contains('$'), "{lang:?}: {text}");
        }
        assert_eq!(
            get_with_markers(&load_in("tea-app", &crate::Localizations, Language::English), "guide-start"),
            "Getting started"
        );
    }

    #[test]
    fn guide_commands_exist() {
        // Todo `{comando}` do guia precisa existir (núcleo ou extensões embutidas).
        let known: Vec<&str> = crate::CORE_COMMANDS.iter().map(|(id, _)| *id).collect();
        let ext = [
            "filetree.toggle",
            "filetree.reveal",
            "search.focus",
            "problems.focus",
            "problems.next",
            "terminal.focus",
            "git.focus",
            "git.fetch",
            "git.pull",
            "git.push",
            "references.focus",
            "outline.toggle",
            "calls.focus",
            "preview.toggle",
            "debug.focus",
        ];
        for (_, paragraphs) in GUIDE {
            for p in *paragraphs {
                let text = paragraph(p);
                assert!(!text.is_empty() && !text.contains('$'), "parágrafo sem texto: {p} = {text:?}");
                let mut rest = text.as_str();
                while let Some(i) = rest.find('{') {
                    let end = rest[i..].find('}').unwrap() + i;
                    let id = &rest[i + 1..end];
                    assert!(known.contains(&id) || ext.contains(&id), "comando desconhecido no guia: {id}");
                    rest = &rest[end + 1..];
                }
            }
        }
    }
}
