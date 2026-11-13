//! Tela inicial: o que aparece na ilha do editor quando não há arquivo aberto (só a aba vazia do começo). Atalhos
//! para começar, os recentes e as teclas principais. A aba vazia continua lá por baixo: começar a digitar ou
//! `Ctrl+N` a mostram.

use std::path::{Path, PathBuf};

use iced::widget::{button, column, container, image, row, space, text};
use iced::{Element, Fill, Theme};
use tea_core::icons::{Icon, icon};
use tea_core::keymap::Keymap;

use crate::{Message, recent};

/// Quantos recentes de cada tipo.
const RECENT_MAX: usize = 5;

/// Os recentes que a tela mostra, lidos uma vez (não a cada quadro).
#[derive(Debug, Clone, Default)]
pub struct Recents {
    pub files: Vec<PathBuf>,
    pub projects: Vec<PathBuf>,
}

impl Recents {
    /// Do arquivo de recentes: arquivos que ainda existem e outros projetos (este já está aberto).
    pub fn load(path: Option<&Path>, root: &Path) -> Self {
        let list = path.map(recent::Recent::load).unwrap_or_default();
        Self {
            files: list.files.into_iter().filter(|p| p.is_file()).take(RECENT_MAX).collect(),
            projects: list.projects.into_iter().filter(|p| p != root && p.is_dir()).take(RECENT_MAX).collect(),
        }
    }
}

pub fn view<'a>(root: &'a Path, recents: &'a Recents, keymap: &'a Keymap) -> Element<'a, Message> {
    let chord = |cmd: &str| keymap.chord_for(cmd).map(|c| c.to_string()).unwrap_or_default();
    let action = |glyph: Icon, label: String, cmd: &'static str| -> Element<'a, Message> {
        button(
            row![
                icon(glyph).size(15),
                text(label).size(13).width(Fill),
                text(chord(cmd)).size(11).style(text::secondary),
            ]
            .spacing(10)
            .align_y(iced::Center),
        )
        .on_press(Message::RunCommand(cmd.to_string()))
        .padding([5, 8])
        .width(Fill)
        .style(|t: &Theme, s| tea_core::style::list_item(t, s, false))
        .into()
    };
    let start = column![
        heading(fl!("welcome-start")),
        action(Icon::NewFile, fl!("welcome-new-file"), "file.new"),
        action(Icon::Terminal, fl!("welcome-launch-agent"), "agents.launch"),
        action(Icon::GoToFile, fl!("welcome-quick-open"), "file.quick_open"),
        action(Icon::File, fl!("welcome-open-file"), "file.open"),
        action(Icon::FolderOpened, fl!("welcome-open-folder"), "file.open_folder"),
        action(Icon::History, fl!("welcome-recent"), "file.open_recent"),
    ]
    .spacing(2)
    .width(Fill);

    let entry = |path: &'a PathBuf, project: bool| -> Element<'a, Message> {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string());
        let place = path.parent().map(recent::display).unwrap_or_default();
        let glyph: Element<'a, Message> = if project {
            icon(Icon::Folder).size(14).style(text::secondary).into()
        } else {
            tea_core::file_icons::icon(&name, 14.0, true).into()
        };
        button(
            row![
                glyph,
                column![text(name).size(13), text(place).size(11).style(text::secondary)].width(Fill),
            ]
            .spacing(10)
            .align_y(iced::Center),
        )
        .on_press(Message::OpenRecentPath(path.clone(), project))
        .padding([4, 8])
        .width(Fill)
        .style(|t: &Theme, s| tea_core::style::list_item(t, s, false))
        .into()
    };
    let mut recent_col = column![].spacing(2).width(Fill);
    if !recents.files.is_empty() {
        recent_col = recent_col.push(heading(fl!("welcome-recent-files")));
        recent_col = recent_col.extend(recents.files.iter().map(|p| entry(p, false)));
    }
    if !recents.projects.is_empty() {
        recent_col = recent_col.push(heading(fl!("welcome-recent-projects")));
        recent_col = recent_col.extend(recents.projects.iter().map(|p| entry(p, true)));
    }

    let project = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let logo = image(LOGO.clone()).width(56).height(56);
    let title = row![logo, column![text("IDE TEA").size(22), text(project).size(13).style(text::secondary)].spacing(2)]
        .spacing(14)
        .align_y(iced::Center);

    // Teclas que valem em qualquer lugar, para quem ainda não as conhece.
    let hints = [("palette.open", fl!("welcome-hint-palette")), ("search.focus", fl!("welcome-hint-search")), (
        "terminal.focus",
        fl!("welcome-hint-terminal"),
    )];
    let hints = row(hints.into_iter().filter_map(|(cmd, label)| {
        let c = keymap.chord_for(cmd)?;
        Some(
            row![key_cap(c.to_string()), text(label).size(12).style(text::secondary)]
                .spacing(6)
                .align_y(iced::Center)
                .into(),
        )
    }))
    .spacing(18)
    .wrap();

    let body = column![title, space().height(18), row![start, recent_col].spacing(28), space().height(18), hints]
        .spacing(4)
        .max_width(680);
    container(tea_core::style::scrollable(container(body).padding(24).center_x(Fill))).center(Fill).into()
}

/// Um handle só, criado uma vez: um handle novo a cada quadro tem outro id, e o iced decodifica a imagem de novo
/// (o logo piscava).
static LOGO: std::sync::LazyLock<image::Handle> =
    std::sync::LazyLock::new(|| image::Handle::from_bytes(include_bytes!("../../../assets/idetea.png").as_slice()));

fn heading<'a>(label: String) -> Element<'a, Message> {
    container(text(label.to_uppercase()).size(11).style(text::secondary)).padding([6, 8]).into()
}

fn key_cap<'a>(key: String) -> Element<'a, Message> {
    container(text(key).size(11).font(iced::Font::MONOSPACE))
        .padding([1, 6])
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(p.background.weak.color.into()),
                border: iced::Border {
                    color: p.background.strong.color,
                    width: 1.0,
                    radius: tea_core::style::RADIUS_SMALL.into(),
                },
                ..iced::widget::container::Style::default()
            }
        })
        .into()
}
