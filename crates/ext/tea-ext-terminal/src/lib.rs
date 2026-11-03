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

use iced::widget::{button, column, container, keyed_column, row, space, text};
use iced::{Color, Element, Fill, Font, Subscription, Task, Theme};
use iced_term::actions::Action;
use iced_term::settings::{BackendSettings, FontSettings, Settings, ThemeSettings};
use iced_term::{ColorPalette, Terminal, TerminalView};
use tea_core::icons::{self, Icon};
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
    /// Abre um shell novo (numa aba nova).
    Start,
    /// Mostra o shell da aba `n` (posição na lista).
    ShowShell(usize),
    ShowTask,
    CloseTask,
    /// Fecha o terminal na tela (encerra o processo).
    CloseCurrent,
}

struct TaskTerm {
    term: Terminal,
    name: String,
}

/// Um shell numa aba: o número fica o mesmo enquanto a aba existir ("Terminal 2").
struct Shell {
    term: Terminal,
    number: u32,
}

/// O que está na tela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    Shell(usize),
    Task,
}

pub struct TerminalExtension {
    root: PathBuf,
    shells: Vec<Shell>,
    task: Option<TaskTerm>,
    shown: Shown,
    next_id: u64,
    next_number: u32,
    /// O tamanho da área do terminal, como o widget mediu por último.
    size: Option<iced::Size>,
    theme: Theme,
    font_size: f32,
}

impl Default for TerminalExtension {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            shells: Vec::new(),
            task: None,
            shown: Shown::Shell(0),
            next_id: 0,
            next_number: 0,
            size: None,
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

    /// Abre um shell numa aba nova e mostra.
    fn start(&mut self, ctx: &mut Context) -> bool {
        self.next_id += 1;
        let shell = std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/bin/sh".into());
        match self.new_terminal(self.settings(shell, Vec::new(), &self.root.clone())) {
            Ok(term) => {
                // A numeração recomeça quando não sobra nenhum.
                if self.shells.is_empty() {
                    self.next_number = 0;
                }
                self.next_number += 1;
                self.shells.push(Shell { term, number: self.next_number });
                self.shown = Shown::Shell(self.shells.len() - 1);
                true
            }
            Err(e) => {
                ctx.notify(fl!("terminal-start-error", error = e.to_string()));
                false
            }
        }
    }

    /// Troca o shell na tela por um novo, na mesma aba (o processo anterior é encerrado).
    fn restart(&mut self, ctx: &mut Context) -> bool {
        let Shown::Shell(i) = self.shown else { return self.start(ctx) };
        if i >= self.shells.len() {
            return self.start(ctx);
        }
        let number = self.shells[i].number;
        self.shells.remove(i);
        if !self.start(ctx) {
            return false;
        }
        // O novo foi para o fim: volta para o lugar do antigo, com o mesmo número.
        let mut shell = self.shells.pop().expect("acabou de entrar");
        shell.number = number;
        self.shells.insert(i, shell);
        self.shown = Shown::Shell(i);
        true
    }

    /// Roda uma tarefa num terminal novo (a anterior, se ainda rodava, é encerrada).
    fn run_task(&mut self, name: &str, command: &str, cwd: &Path, done: &str, ctx: &mut Context) -> Task<Message> {
        self.next_id += 1;
        self.task = None;
        let (before, after) = done.split_once("{code}").unwrap_or((done, ""));
        let args = ["-c", TASK_SCRIPT, "sh", command, before, after].map(str::to_string).to_vec();
        match self.new_terminal(self.settings("/bin/sh".into(), args, cwd)) {
            Ok(term) => {
                self.task = Some(TaskTerm { term, name: name.to_string() });
                self.shown = Shown::Task;
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            Err(e) => {
                ctx.notify(fl!("terminal-task-error", error = e.to_string()));
                Task::none()
            }
        }
    }

    /// O terminal na tela agora.
    fn current(&self) -> Option<&Terminal> {
        match self.shown {
            Shown::Task => self.task.as_ref().map(|t| &t.term),
            Shown::Shell(i) => self.shells.get(i).map(|s| &s.term),
        }
    }

    fn focus_current(&self) -> Task<Message> {
        match self.current() {
            Some(t) => TerminalView::focus(t.widget_id().clone()),
            None => Task::none(),
        }
    }

    /// Depois de um terminal sair: mostra o vizinho; sem nenhum, esconde o painel.
    fn after_removal(&mut self, ctx: &mut Context) -> Task<Message> {
        self.shown = match self.shown {
            Shown::Task if self.task.is_some() => Shown::Task,
            Shown::Shell(i) if !self.shells.is_empty() => Shown::Shell(i.min(self.shells.len() - 1)),
            _ if !self.shells.is_empty() => Shown::Shell(self.shells.len() - 1),
            _ if self.task.is_some() => Shown::Task,
            _ => Shown::Shell(0),
        };
        if self.current().is_none() {
            ctx.hide_panel(PANEL);
            return Task::none();
        }
        self.focus_current()
    }

    fn close_current(&mut self, ctx: &mut Context) -> Task<Message> {
        // Soltar o terminal encerra o processo.
        match self.shown {
            Shown::Task => self.task = None,
            Shown::Shell(i) if i < self.shells.len() => {
                self.shells.remove(i);
            }
            Shown::Shell(_) => {}
        }
        self.after_removal(ctx)
    }

    /// Mostra um shell (abre um, se não houver) e pega o teclado.
    fn show_shell(&mut self, ctx: &mut Context) -> Task<Message> {
        if !matches!(self.shown, Shown::Shell(i) if i < self.shells.len()) {
            if self.shells.is_empty() && !self.start(ctx) {
                return Task::none();
            }
            self.shown = Shown::Shell(self.shells.len() - 1);
        }
        ctx.show_panel(PANEL);
        self.focus_current()
    }

    /// Cria um terminal já com o tamanho da área. Todo comando ao `iced_term` recalcula linhas e colunas com o
    /// último tamanho recebido; um terminal novo só tem o provisório (80×50 pixels) até o widget medir, e qualquer
    /// coisa antes disso (a saída do shell, um redesenho) o espremia em poucas colunas, quebrando o prompt.
    fn new_terminal(&mut self, settings: Settings) -> std::io::Result<Terminal> {
        let mut term = Terminal::new(self.next_id, settings)?;
        if let Some(size) = self.size {
            term.handle(iced_term::Command::ProxyToBackend(iced_term::BackendCommand::Resize(Some(size), None)));
        }
        Ok(term)
    }

    /// O que muda o lugar do terminal na tela: qual está à vista e se há barra de abas.
    fn layout(&self) -> (Shown, bool) {
        (self.shown, self.current().is_some())
    }

    /// O `iced_term` guarda o desenho com a posição na tela dentro; quando a barra aparece ou some, ou outra aba
    /// volta, o desenho velho sairia no lugar antigo (por cima da barra). Reaplicar a fonte limpa esse cache.
    fn redraw_if_moved(&mut self, before: (Shown, bool)) {
        // Sem medida ainda, não há desenho velho; e reaplicar a fonte recalcularia o tamanho com o provisório.
        if self.layout() == before || self.size.is_none() {
            return;
        }
        let font = self.font();
        let terms = self.shells.iter_mut().map(|s| &mut s.term).chain(self.task.iter_mut().map(|t| &mut t.term));
        for term in terms {
            term.handle(iced_term::Command::ChangeFont(font.clone()));
        }
    }

    fn on_event_inner(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        // Aberto pela barra de ícones: já sobe o shell e pega o teclado.
        if let CoreEvent::PanelShown(_) = event {
            if self.current().is_none() {
                if !self.shells.is_empty() {
                    self.shown = Shown::Shell(self.shells.len() - 1);
                } else if self.task.is_some() {
                    self.shown = Shown::Task;
                } else if !self.start(ctx) {
                    return Task::none();
                }
            }
            return self.focus_current();
        }
        if let CoreEvent::RunTask { name, command, cwd, done } = event {
            return self.run_task(name, command, cwd, done, ctx);
        }
        if let CoreEvent::Appearance { theme, font_size } = event {
            self.theme = theme.clone();
            self.font_size = *font_size;
            let font = self.font();
            let terms = self.shells.iter_mut().map(|s| &mut s.term).chain(self.task.iter_mut().map(|t| &mut t.term));
            for term in terms {
                term.handle(iced_term::Command::ChangeTheme(Box::new(palette_for(theme))));
                term.handle(iced_term::Command::ChangeFont(font.clone()));
            }
        }
        Task::none()
    }

    fn on_command_inner(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "terminal.focus" => self.show_shell(ctx),
            "terminal.new" => {
                if !self.start(ctx) {
                    return Task::none();
                }
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            "terminal.close" => self.close_current(ctx),
            "terminal.restart" => {
                if !self.restart(ctx) {
                    return Task::none();
                }
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            _ => Task::none(),
        }
    }

    fn update_inner(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Start => {
                if self.start(ctx) {
                    return self.focus_current();
                }
            }
            Message::ShowShell(i) => {
                if i < self.shells.len() {
                    self.shown = Shown::Shell(i);
                }
                return self.focus_current();
            }
            Message::ShowTask => {
                if self.task.is_some() {
                    self.shown = Shown::Task;
                }
                return self.focus_current();
            }
            Message::CloseTask => {
                // Soltar o terminal encerra a tarefa, se ainda rodava.
                self.task = None;
                return self.after_removal(ctx);
            }
            Message::CloseCurrent => return self.close_current(ctx),
            Message::Term(iced_term::Event::BackendCall(id, cmd)) => {
                if let iced_term::BackendCommand::Resize(Some(size), _) = &cmd {
                    self.size = Some(*size);
                }
                if let Some(task) = self.task.as_mut().filter(|t| t.term.id == id) {
                    // A tarefa terminou e o Enter fechou: volta para um shell.
                    if task.term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                        self.task = None;
                        return self.after_removal(ctx);
                    }
                    return Task::none();
                }
                // Evento atrasado de um shell que já saiu: nenhum com esse id.
                let Some(i) = self.shells.iter().position(|s| s.term.id == id) else { return Task::none() };
                if self.shells[i].term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                    let number = self.shells[i].number;
                    self.shells.remove(i);
                    // A aba que estava na tela à direita desta escorrega uma posição.
                    if let Shown::Shell(s) = self.shown
                        && s > i
                    {
                        self.shown = Shown::Shell(s - 1);
                    }
                    ctx.notify(fl!("terminal-ended-n", number = number));
                    return self.after_removal(ctx);
                }
            }
        }
        Task::none()
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
        ctx.register_command("terminal.close", fl!("cmd-terminal-close"));
        ctx.register_command("terminal.restart", fl!("cmd-terminal-restart"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        let before = self.layout();
        let task = self.on_event_inner(event, ctx);
        self.redraw_if_moved(before);
        task
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        let before = self.layout();
        let task = self.on_command_inner(command, ctx);
        self.redraw_if_moved(before);
        task
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        let before = self.layout();
        let task = self.update_inner(message, ctx);
        self.redraw_if_moved(before);
        task
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let body: Element<'_, Message> = match self.current() {
            // Com chave: cada terminal tem o próprio estado no widget (o tamanho que já avisou ao shell). Sem ela,
            // o terminal que entra herdava o tamanho do anterior e nunca recebia o seu.
            Some(term) => keyed_column([(term.id, TerminalView::show(term).map(Message::Term))]).height(Fill).into(),
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
        // Sem terminal: só o botão de iniciar. Com um ou mais, a barra (o `+` fica sempre à mão).
        if self.current().is_none() {
            return body;
        }
        let tab = |label: String, on: bool, msg: Message| -> Element<'_, Message> {
            button(text(label).size(12)).on_press(msg).padding([2, 10]).style(if on { button::secondary } else { button::text }).into()
        };
        let mut bar = row![].spacing(4).align_y(iced::Center);
        for (i, s) in self.shells.iter().enumerate() {
            bar = bar.push(tab(fl!("terminal-tab-n", number = s.number), self.shown == Shown::Shell(i), Message::ShowShell(i)));
        }
        if let Some(task) = &self.task {
            bar = bar.push(tab(fl!("terminal-tab-task", name = task.name.as_str()), self.shown == Shown::Task, Message::ShowTask));
        }
        bar = bar.push(icons::icon_button(Icon::Add, fl!("cmd-terminal-new"), Some(Message::Start)));
        bar = bar.push(space::horizontal());
        let close = if self.shown == Shown::Task { fl!("terminal-close-task") } else { fl!("terminal-close") };
        bar = bar.push(icons::icon_button(Icon::Close, close, Some(Message::CloseCurrent)));
        column![bar, body].spacing(2).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let terms = self.shells.iter().map(|s| &s.term).chain(self.task.iter().map(|t| &t.term));
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
