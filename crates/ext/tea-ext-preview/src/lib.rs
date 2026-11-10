//! Painel "Prévia": Markdown formatado ou o desenho de um SVG, do arquivo ativo, atualizado enquanto
//! se digita. O texto vem do núcleo ([`CoreEvent::ActiveText`]), pedido na ativação.
//!
//! No Markdown, imagens locais aparecem (caminho relativo ao arquivo); links da web abrem no
//! navegador e links para arquivos do projeto abrem numa aba.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::path::{Path, PathBuf};

use iced::widget::{column, container, image, markdown, row, svg, text};
use iced::{ContentFit, Element, Fill, Length, Task, Theme};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Extension, FileAction, PanelId, Slot};

const PANEL: PanelId = PanelId("preview");
/// Tipos de arquivo com prévia.
const MARKDOWN: &[&str] = &["md", "markdown"];
const SVG: &[&str] = &["svg"];

#[derive(Debug, Clone)]
pub enum Message {
    Link(String),
}

/// O que o painel mostra.
enum Doc {
    /// Arquivo ativo sem prévia (ou nenhum).
    Nothing,
    Markdown(Box<markdown::Content>),
    Svg {
        /// Último desenho válido (continua na tela enquanto o texto está quebrado).
        handle: Option<svg::Handle>,
        size: (f32, f32),
        error: Option<String>,
    },
}

pub struct PreviewExtension {
    path: Option<PathBuf>,
    doc: Doc,
    theme: Theme,
    font_size: f32,
}

impl Default for PreviewExtension {
    fn default() -> Self {
        Self { path: None, doc: Doc::Nothing, theme: Theme::Dark, font_size: 14.0 }
    }
}

fn extension(path: &Path) -> String {
    path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

/// O que fazer com um link clicado no Markdown.
#[derive(Debug, PartialEq, Eq)]
enum LinkTarget {
    Web(String),
    File(PathBuf),
    /// Âncora dentro do próprio arquivo (`#titulo`): não faz nada.
    Anchor,
}

fn link_target(url: &str, base: Option<&Path>) -> LinkTarget {
    let lower = url.to_ascii_lowercase();
    if ["http://", "https://", "mailto:"].iter().any(|p| lower.starts_with(p)) {
        return LinkTarget::Web(url.to_string());
    }
    let file = url.split('#').next().unwrap_or_default();
    if file.is_empty() {
        return LinkTarget::Anchor;
    }
    let file = file.strip_prefix("file://").unwrap_or(file);
    let dir = base.and_then(Path::parent).unwrap_or(Path::new(""));
    LinkTarget::File(dir.join(file))
}

/// Desenha as imagens do Markdown a partir do disco (relativas ao arquivo).
struct Viewer<'a> {
    base: Option<&'a Path>,
}

impl<'a> markdown::Viewer<'a, Message> for Viewer<'a> {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::Link(url)
    }

    fn image(
        &self,
        settings: markdown::Settings,
        url: &'a markdown::Uri,
        title: &'a str,
        alt: &markdown::Text,
    ) -> Element<'a, Message> {
        if let LinkTarget::File(path) = link_target(url, self.base)
            && path.is_file()
        {
            return if extension(&path) == "svg" {
                svg(svg::Handle::from_path(path)).width(Length::Shrink).content_fit(ContentFit::ScaleDown).into()
            } else {
                image(image::Handle::from_path(path)).content_fit(ContentFit::ScaleDown).into()
            };
        }
        // Imagem da web ou que não existe: o texto alternativo, como o iced faz.
        let _ = title;
        container(text(alt.spans(settings.style).iter().map(|s| s.text.as_ref()).collect::<String>()).size(12))
            .padding(6)
            .style(container::bordered_box)
            .into()
    }
}

impl PreviewExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn set_text(&mut self, path: &Path, text: &str) {
        let ext = extension(path);
        if MARKDOWN.contains(&ext.as_str()) {
            self.doc = Doc::Markdown(Box::new(markdown::Content::parse(text)));
        } else if SVG.contains(&ext.as_str()) {
            let (old_handle, old_size) = match &self.doc {
                Doc::Svg { handle, size, .. } => (handle.clone(), *size),
                _ => (None, (0.0, 0.0)),
            };
            self.doc = match usvg::Tree::from_data(text.as_bytes(), &usvg::Options::default()) {
                Ok(tree) => Doc::Svg {
                    handle: Some(svg::Handle::from_memory(text.as_bytes().to_vec())),
                    size: (tree.size().width(), tree.size().height()),
                    error: None,
                },
                Err(e) => Doc::Svg { handle: old_handle, size: old_size, error: Some(e.to_string()) },
            };
        } else {
            self.doc = Doc::Nothing;
        }
    }

    fn settings(&self) -> markdown::Settings {
        let mut s = markdown::Settings::with_text_size(self.font_size, &self.theme);
        s.code_size = (self.font_size * 0.9).into();
        s
    }
}

impl Extension for PreviewExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "preview"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        ctx.register_panel_hidden(PANEL, fl!("preview-panel"), Slot::Right);
        ctx.panel_icon(PANEL, Icon::Preview);
        ctx.register_command("preview.toggle", fl!("cmd-preview-toggle"));
        ctx.watch_text(MARKDOWN.iter().chain(SVG).copied());
        ctx.file_action(FileAction {
            extensions: MARKDOWN.iter().chain(SVG).map(|e| e.to_string()).collect(),
            icon: Icon::Preview,
            tip: fl!("preview-panel"),
            command: "preview.toggle".into(),
            panel: Some(PANEL),
        });
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::ActiveFileChanged(path) => {
                if self.path != *path {
                    self.doc = Doc::Nothing;
                }
                self.path = path.clone();
            }
            CoreEvent::ActiveText { path, text } => {
                self.path = Some(path.clone());
                self.set_text(path, text);
            }
            CoreEvent::Appearance { theme, font_size, .. } => {
                self.theme = theme.clone();
                self.font_size = *font_size;
            }
            _ => {}
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if command == "preview.toggle" {
            ctx.toggle_panel(PANEL);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        let Message::Link(url) = message;
        match link_target(&url, self.path.as_deref()) {
            LinkTarget::Web(url) => {
                if let Err(e) = tea_core::os::open_url(&url) {
                    ctx.notify(fl!("preview-link-error", error = e.to_string()));
                }
            }
            LinkTarget::File(path) if path.is_file() => ctx.open_file(path),
            LinkTarget::File(path) => ctx.notify(fl!("preview-link-missing", path = path.display().to_string())),
            LinkTarget::Anchor => {}
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let note = |s: String| -> Element<'_, Message> { text(s).size(12).style(text::secondary).into() };
        match &self.doc {
            Doc::Nothing => note(fl!("preview-empty")),
            Doc::Markdown(content) => {
                let viewer = Viewer { base: self.path.as_deref() };
                let body = markdown::view_with(content.items(), self.settings(), &viewer);
                tea_core::style::scrollable(container(body).padding([8, 14]).width(Fill)).height(Fill).into()
            }
            Doc::Svg { handle, size, error } => {
                let mut info = row![text(format!("{} × {}", size.0.round(), size.1.round())).size(11).style(text::secondary)];
                if let Some(e) = error {
                    info = info.push(text(fl!("preview-svg-invalid", error = e.as_str())).size(11).style(text::danger));
                }
                let drawing: Element<'_, Message> = match handle {
                    Some(h) => svg(h.clone()).width(Fill).height(Fill).content_fit(ContentFit::Contain).into(),
                    None => note(fl!("preview-nothing")),
                };
                column![info, container(drawing).padding(8).width(Fill).height(Fill)].spacing(4).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_go_to_the_web_or_to_files() {
        let base = Path::new("/p/docs/guia.md");
        assert_eq!(link_target("https://x.org/a", Some(base)), LinkTarget::Web("https://x.org/a".into()));
        assert_eq!(link_target("#instalar", Some(base)), LinkTarget::Anchor);
        assert_eq!(link_target("../README.md#uso", Some(base)), LinkTarget::File("/p/docs/../README.md".into()));
        assert_eq!(link_target("img/logo.png", Some(base)), LinkTarget::File("/p/docs/img/logo.png".into()));
    }

    #[test]
    fn broken_svg_keeps_the_last_drawing() {
        let mut ext = PreviewExtension::new();
        let path = Path::new("/p/icone.svg");
        ext.set_text(path, r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="20"></svg>"#);
        assert!(matches!(ext.doc, Doc::Svg { handle: Some(_), size: (10.0, 20.0), error: None }));
        ext.set_text(path, "<svg");
        assert!(matches!(ext.doc, Doc::Svg { handle: Some(_), error: Some(_), .. }));
    }

    #[test]
    fn markdown_and_other_files() {
        let mut ext = PreviewExtension::new();
        ext.set_text(Path::new("/p/LEIA.MD"), "# Título\n\ntexto");
        assert!(matches!(&ext.doc, Doc::Markdown(c) if c.items().len() == 2));
        ext.set_text(Path::new("/p/main.rs"), "fn main() {}");
        assert!(matches!(ext.doc, Doc::Nothing));
    }
}
