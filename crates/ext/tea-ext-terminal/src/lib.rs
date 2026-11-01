//! Terminal integrado no painel inferior (alacritty via `iced_term`).
//!
//! O shell só é iniciado quando o painel é aberto pela primeira vez. Com o foco
//! no terminal, Ctrl+letra, F-teclas etc. vão para o shell; atalhos da IDE com
//! Ctrl+Shift continuam valendo (regra aplicada pelo app em todos os painéis).

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::path::PathBuf;

use iced::widget::{button, column, container, text};
use iced::{Color, Element, Fill, Font, Subscription, Task, Theme};
use iced_term::actions::Action;
use iced_term::settings::{BackendSettings, FontSettings, Settings, ThemeSettings};
use iced_term::{ColorPalette, Terminal, TerminalView};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("terminal");

#[derive(Debug, Clone)]
pub enum Message {
    Term(iced_term::Event),
    Start,
}

pub struct TerminalExtension {
    root: PathBuf,
    term: Option<Terminal>,
    next_id: u64,
    theme: Theme,
    font_size: f32,
}

impl Default for TerminalExtension {
    fn default() -> Self {
        Self { root: PathBuf::new(), term: None, next_id: 0, theme: Theme::TokyoNight, font_size: 14.0 }
    }
}

impl TerminalExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn settings(&self) -> Settings {
        let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/bin/sh".into());
        Settings {
            font: self.font(),
            theme: ThemeSettings::new(Box::new(palette_for(&self.theme))),
            backend: BackendSettings {
                program: shell,
                args: Vec::new(),
                env: [("TERM_PROGRAM".to_string(), "idetea".to_string())].into(),
                working_directory: Some(self.root.clone()),
            },
        }
    }

    fn font(&self) -> FontSettings {
        FontSettings { size: self.font_size, font_type: Font::MONOSPACE, ..FontSettings::default() }
    }

    /// Inicia o shell (substituindo um anterior, se houver).
    fn start(&mut self, ctx: &mut Context) -> bool {
        self.next_id += 1;
        // Soltar o anterior encerra o processo dele.
        self.term = None;
        match Terminal::new(self.next_id, self.settings()) {
            Ok(term) => {
                self.term = Some(term);
                true
            }
            Err(e) => {
                ctx.notify(fl!("terminal-start-error", error = e.to_string()));
                false
            }
        }
    }

    fn focus_task(&self) -> Task<Message> {
        match &self.term {
            Some(t) => TerminalView::focus(t.widget_id().clone()),
            None => Task::none(),
        }
    }
}

impl Extension for TerminalExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "terminal"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, "Terminal", Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Terminal);
        ctx.register_command("terminal.focus", fl!("cmd-terminal-focus"));
        ctx.register_command("terminal.new", fl!("cmd-terminal-new"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        // Aberto pela barra de ícones: já sobe o shell e pega o teclado.
        if let CoreEvent::PanelShown(_) = event {
            if self.term.is_none() && !self.start(ctx) {
                return Task::none();
            }
            return self.focus_task();
        }
        if let CoreEvent::Appearance { theme, font_size } = event {
            self.theme = theme.clone();
            self.font_size = *font_size;
            let font = self.font();
            if let Some(term) = &mut self.term {
                term.handle(iced_term::Command::ChangeTheme(Box::new(palette_for(theme))));
                term.handle(iced_term::Command::ChangeFont(font));
            }
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "terminal.focus" => {
                if self.term.is_none() && !self.start(ctx) {
                    return Task::none();
                }
                ctx.show_panel(PANEL);
                self.focus_task()
            }
            "terminal.new" => {
                if !self.start(ctx) {
                    return Task::none();
                }
                ctx.show_panel(PANEL);
                self.focus_task()
            }
            _ => Task::none(),
        }
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Start => {
                if self.start(ctx) {
                    return self.focus_task();
                }
            }
            Message::Term(iced_term::Event::BackendCall(id, cmd)) => {
                let Some(term) = self.term.as_mut().filter(|t| t.id == id) else {
                    // Evento atrasado de um shell que já foi substituído.
                    return Task::none();
                };
                if term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                    self.term = None;
                    ctx.hide_panel(PANEL);
                    ctx.notify(fl!("terminal-ended"));
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        match &self.term {
            Some(term) => TerminalView::show(term).map(Message::Term),
            None => container(
                column![
                    text(fl!("terminal-not-running")).size(13).style(text::secondary),
                    button(text(fl!("terminal-start")).size(13)).on_press(Message::Start).padding([3, 10]),
                ]
                .spacing(8)
                .align_x(iced::Center),
            )
            .center(Fill)
            .into(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.term {
            Some(term) => term.subscription().map(Message::Term),
            None => Subscription::none(),
        }
    }
}

fn hex(c: Color) -> String {
    let [r, g, b, _] = c.into_rgba8();
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Cores do terminal a partir do tema da IDE: fundo, texto e as cores de destaque.
fn palette_for(theme: &Theme) -> ColorPalette {
    let p = theme.extended_palette();
    let base = ColorPalette::default();
    let mut out = ColorPalette {
        foreground: hex(p.background.base.text),
        background: hex(p.background.base.color),
        red: hex(p.danger.base.color),
        green: hex(p.success.base.color),
        yellow: hex(p.warning.base.color),
        blue: hex(p.primary.base.color),
        ..base
    };
    if !p.is_dark {
        // As cores padrão são pensadas para fundo escuro: "preto" e "branco" invertem.
        out.black = "#3b3b3b".into();
        out.white = "#7a7a7a".into();
        out.bright_black = "#6b6b6b".into();
        out.bright_white = "#2b2b2b".into();
        out.yellow = "#a06d00".into();
        out.bright_yellow = "#8a5d00".into();
        out.cyan = "#1f7f86".into();
        out.bright_cyan = "#176a70".into();
        out.dim_foreground = "#6b6b6b".into();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_follows_theme_background() {
        let dark = palette_for(&Theme::TokyoNight);
        let light = palette_for(&Theme::Light);
        assert_eq!(dark.background, hex(Theme::TokyoNight.extended_palette().background.base.color));
        assert_eq!(light.background, hex(Theme::Light.extended_palette().background.base.color));
        assert_ne!(dark.foreground, light.foreground);
        assert!(dark.background.starts_with('#') && dark.background.len() == 7);
    }
}
