//! Terminal integrado no painel inferior (alacritty via `iced_term`).
//!
//! O shell só é iniciado quando o painel é aberto pela primeira vez. Com o foco
//! no terminal, Ctrl+letra, F-teclas etc. vão para o shell; atalhos da IDE com
//! Ctrl+Shift continuam valendo (regra aplicada pelo app em todos os painéis).
//!
//! Tarefas (`CoreEvent::RunTask`) rodam num terminal só delas, ao lado do shell: não digitam por cima do que
//! estiver rodando nele. No fim, o código de saída aparece e o terminal espera o Enter para fechar; rodar outra
//! tarefa substitui a anterior. `Ctrl+C` para só o comando (o resto do script continua e mostra o código).

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::path::{Path, PathBuf};

use iced::widget::{button, column, container, row, space, text};
use iced::{Color, Element, Fill, Font, Subscription, Task, Theme};
use iced_term::actions::Action;
use iced_term::settings::{BackendSettings, FontSettings, Settings, ThemeSettings};
use iced_term::{ColorPalette, Terminal, TerminalView};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("terminal");

/// Roda a tarefa (`$1`), mostra o fim (`$2` código `$3`) e espera o Enter. O `trap` faz o `Ctrl+C` parar só o
/// comando: o shell do script continua para mostrar o código. O comando vai como argumento (sem aspas para
/// escapar) e o `eval` deixa usar `&&`, pipes e variáveis.
const TASK_SCRIPT: &str = r#"trap true INT
printf '\033[1m$ %s\033[0m\n' "$1"
( eval "$1" )
c=$?
printf '\n\033[2m%s%s%s\033[0m\n' "$2" "$c" "$3"
read _"#;

#[derive(Debug, Clone)]
pub enum Message {
    Term(iced_term::Event),
    Start,
    ShowShell,
    ShowTask,
    CloseTask,
}

struct TaskTerm {
    term: Terminal,
    name: String,
}

pub struct TerminalExtension {
    root: PathBuf,
    shell: Option<Terminal>,
    task: Option<TaskTerm>,
    /// O painel mostra a tarefa (senão, o shell).
    show_task: bool,
    next_id: u64,
    theme: Theme,
    font_size: f32,
}

impl Default for TerminalExtension {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            shell: None,
            task: None,
            show_task: false,
            next_id: 0,
            theme: Theme::TokyoNight,
            font_size: 14.0,
        }
    }
}

impl TerminalExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn settings(&self, program: String, args: Vec<String>, cwd: &Path) -> Settings {
        Settings {
            font: self.font(),
            theme: ThemeSettings::new(Box::new(palette_for(&self.theme))),
            backend: BackendSettings {
                program,
                args,
                env: [("TERM_PROGRAM".to_string(), "idetea".to_string())].into(),
                working_directory: Some(cwd.to_path_buf()),
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
        self.shell = None;
        let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/bin/sh".into());
        match Terminal::new(self.next_id, self.settings(shell, Vec::new(), &self.root.clone())) {
            Ok(term) => {
                self.shell = Some(term);
                self.show_task = false;
                true
            }
            Err(e) => {
                ctx.notify(fl!("terminal-start-error", error = e.to_string()));
                false
            }
        }
    }

    /// Roda uma tarefa num terminal novo (a anterior, se ainda rodava, é encerrada).
    fn run_task(&mut self, name: &str, command: &str, cwd: &Path, done: &str, ctx: &mut Context) -> Task<Message> {
        self.next_id += 1;
        self.task = None;
        let (before, after) = done.split_once("{code}").unwrap_or((done, ""));
        let args = ["-c", TASK_SCRIPT, "sh", command, before, after].map(str::to_string).to_vec();
        match Terminal::new(self.next_id, self.settings("/bin/sh".into(), args, cwd)) {
            Ok(term) => {
                self.task = Some(TaskTerm { term, name: name.to_string() });
                self.show_task = true;
                ctx.show_panel(PANEL);
                self.focus_task()
            }
            Err(e) => {
                ctx.notify(fl!("terminal-task-error", error = e.to_string()));
                Task::none()
            }
        }
    }

    /// O terminal na tela agora.
    fn current(&self) -> Option<&Terminal> {
        match (&self.task, self.show_task) {
            (Some(t), true) => Some(&t.term),
            _ => self.shell.as_ref(),
        }
    }

    fn focus_task(&self) -> Task<Message> {
        match self.current() {
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
            if self.current().is_none() && !self.start(ctx) {
                return Task::none();
            }
            return self.focus_task();
        }
        if let CoreEvent::RunTask { name, command, cwd, done } = event {
            return self.run_task(name, command, cwd, done, ctx);
        }
        if let CoreEvent::Appearance { theme, font_size } = event {
            self.theme = theme.clone();
            self.font_size = *font_size;
            let font = self.font();
            let terms = self.shell.iter_mut().chain(self.task.iter_mut().map(|t| &mut t.term));
            for term in terms {
                term.handle(iced_term::Command::ChangeTheme(Box::new(palette_for(theme))));
                term.handle(iced_term::Command::ChangeFont(font.clone()));
            }
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "terminal.focus" => {
                if self.shell.is_none() && !self.start(ctx) {
                    return Task::none();
                }
                self.show_task = false;
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
            Message::ShowShell => {
                if self.shell.is_none() && !self.start(ctx) {
                    return Task::none();
                }
                self.show_task = false;
                return self.focus_task();
            }
            Message::ShowTask => {
                self.show_task = self.task.is_some();
                return self.focus_task();
            }
            Message::CloseTask => {
                // Soltar o terminal encerra a tarefa, se ainda rodava.
                self.task = None;
                self.show_task = false;
                return self.focus_task();
            }
            Message::Term(iced_term::Event::BackendCall(id, cmd)) => {
                if let Some(task) = self.task.as_mut().filter(|t| t.term.id == id) {
                    // A tarefa terminou e o Enter fechou: volta para o shell.
                    if task.term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                        self.task = None;
                        self.show_task = false;
                        if self.shell.is_none() {
                            ctx.hide_panel(PANEL);
                        }
                        return self.focus_task();
                    }
                    return Task::none();
                }
                let Some(term) = self.shell.as_mut().filter(|t| t.id == id) else {
                    // Evento atrasado de um shell que já foi substituído.
                    return Task::none();
                };
                if term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                    self.shell = None;
                    if self.task.is_none() {
                        ctx.hide_panel(PANEL);
                    }
                    ctx.notify(fl!("terminal-ended"));
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let body: Element<'_, Message> = match self.current() {
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
        };
        let Some(task) = &self.task else { return body };
        // Com uma tarefa: abas "Terminal" e "Tarefa: nome" em cima.
        let tab = |label: String, on: bool, msg: Message| {
            button(text(label).size(12)).on_press(msg).padding([2, 10]).style(if on { button::secondary } else { button::text })
        };
        let bar = row![
            tab(fl!("terminal-tab-shell"), !self.show_task, Message::ShowShell),
            tab(fl!("terminal-tab-task", name = task.name.as_str()), self.show_task, Message::ShowTask),
            space::horizontal(),
            button(text(fl!("terminal-close-task")).size(12)).on_press(Message::CloseTask).padding([2, 10]).style(button::text),
        ]
        .spacing(4)
        .align_y(iced::Center);
        column![bar, body].spacing(2).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let terms = self.shell.iter().chain(self.task.iter().map(|t| &t.term));
        Subscription::batch(terms.map(|t| t.subscription().map(Message::Term)))
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

    /// O script da tarefa: o comando roda com `&&` e variáveis, e o fim mostra o código de saída.
    #[test]
    fn task_script_reports_exit_code() {
        let out = std::process::Command::new("/bin/sh")
            .args(["-c", TASK_SCRIPT, "sh", "X=oi; echo $X && exit 3", "[fim: ", "]"])
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("$ X=oi; echo $X && exit 3"), "{text}");
        assert!(text.contains("oi\n"), "{text}");
        assert!(text.contains("[fim: 3]"), "{text}");
    }
}
