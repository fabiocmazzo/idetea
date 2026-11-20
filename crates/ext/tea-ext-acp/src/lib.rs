//! Painel da sessão ACP (RFC 0022): o agente é um processo, a IDE mostra o texto
//! e espera uma permissão explícita antes de cada ação. Não raspa o terminal.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::path::PathBuf;

use futures::channel::mpsc;
use iced::futures::channel::mpsc as iced_mpsc;
use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Fill, Subscription, Task};
use tea_acp::permission::{Kind, ShownOption, UserChoice};
use tea_acp::session::{Command, LoginMethod, SessionEvent};
use tea_acp::target::{self, Launch, Missing, NodeReport, Target};
use tea_core::icons::Icon;
use tea_core::{ButtonKind, Context, Dialog, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("acp");

#[derive(Debug, Clone)]
pub enum Message {
    Start(Target),
    Bound(mpsc::UnboundedSender<Command>),
    Event(SessionEvent),
    Prompt(String),
    Send,
    Cancel,
    Choose(u32),
    Login(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SessionKey {
    generation: u64,
    cwd: PathBuf,
    program: PathBuf,
    args: Vec<String>,
}

#[derive(Debug, Clone)]
enum Line {
    You(String),
    Assistant(String),
    Thought(String),
    Tool { title: String, status: String },
    Note(String),
}

#[derive(Debug, Clone)]
struct Pending {
    title: String,
    detail: String,
    options: Vec<ShownOption>,
}

#[derive(Default)]
pub struct AcpExtension {
    root: PathBuf,
    key: Option<SessionKey>,
    generation: u64,
    cmd: Option<mpsc::UnboundedSender<Command>>,
    program: Option<PathBuf>,
    lines: Vec<Line>,
    prompt: String,
    pending: Option<Pending>,
    logins: Vec<LoginMethod>,
    notice: Option<String>,
    busy: bool,
}

impl AcpExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn send(&self, command: Command) {
        if let Some(cmd) = &self.cmd {
            let _ = cmd.unbounded_send(command);
        }
    }

    fn start(&mut self, launch: Launch, ctx: &mut Context) {
        self.send(Command::Shutdown);
        self.cmd = None;
        self.generation += 1;
        self.lines.clear();
        self.pending = None;
        self.logins.clear();
        self.notice = None;
        self.busy = false;
        self.program = Some(launch.program.clone());
        self.key = Some(SessionKey {
            generation: self.generation,
            cwd: self.root.clone(),
            program: launch.program,
            args: launch.args,
        });
        ctx.show_panel(PANEL);
    }

    fn launch(&mut self, target: Target, ctx: &mut Context) {
        let configured = claude_bin();
        let found = target::system_probe(configured.as_deref()).resolve(target);
        match found {
            Ok(launch) => self.start(launch, ctx),
            Err(missing) => {
                self.notice = Some(missing_text(&missing));
                ctx.show_panel(PANEL);
            }
        }
    }

    /// O mascote continua pelos hooks (`idetea mascot setup-hooks`). `mascot_cue` é o gancho
    /// para um marco seguinte, sem gravar o estado daqui e duplicar o que o hook já faz.
    fn on_event_msg(&mut self, event: SessionEvent, ctx: &mut Context) {
        match event {
            SessionEvent::Ready { agent } => {
                self.lines.push(Line::Note(fl!("acp-ready", id = agent)));
            }
            SessionEvent::Assistant { text } => append_text(&mut self.lines, true, text),
            SessionEvent::Thought { text } => append_text(&mut self.lines, false, text),
            SessionEvent::Tool { title, status, .. } => {
                self.lines.push(Line::Tool { title, status });
            }
            SessionEvent::Permission { title, detail, options } => {
                self.pending = Some(Pending { title: title.clone(), detail: detail.clone(), options: options.clone() });
                ctx.show_dialog(permission_dialog(&title, &detail, &options));
            }
            SessionEvent::Turn { stop } => {
                self.busy = false;
                self.lines.push(Line::Note(fl!("acp-turn", stop = stop_text(&stop))));
            }
            SessionEvent::AuthRequired { message, logins } => {
                self.busy = false;
                self.cmd = None;
                self.logins = logins;
                self.notice = Some(fl!("acp-auth", message = message));
                self.key = None;
            }
            SessionEvent::Failed { message } => {
                self.busy = false;
                if message != "shutdown" {
                    self.notice = Some(fl!("acp-failed", error = message));
                }
            }
            SessionEvent::Closed => {
                self.busy = false;
                self.cmd = None;
                self.key = None;
                if self.notice.is_none() && self.logins.is_empty() {
                    self.lines.push(Line::Note(fl!("acp-closed")));
                }
            }
        }
    }

    fn choose(&mut self, index: u32) {
        if self.pending.as_ref().is_some_and(|p| (index as usize) < p.options.len()) {
            self.send(Command::Permission(UserChoice::Option(index)));
            self.pending = None;
        }
    }

    fn view_panel(&self) -> Element<'_, Message> {
        let mut body = column![].spacing(8).padding(12).width(Fill);
        if let Some(notice) = &self.notice {
            body = body.push(text(notice).size(13).style(text::warning).width(Fill));
        }
        for (i, login) in self.logins.iter().enumerate() {
            let label = fl!("acp-login", name = login.name.clone());
            body = body.push(button(text(label).size(13)).style(button::secondary).on_press(Message::Login(i)));
            if !login.description.is_empty() {
                body = body.push(text(login.description.clone()).size(12).style(text::secondary).width(Fill));
            }
        }
        if self.key.is_none() && self.lines.is_empty() {
            body = body
                .push(text(fl!("acp-empty")).size(13).width(Fill))
                .push(row![
                    button(text(fl!("acp-opencode")).size(13)).on_press(Message::Start(Target::OpenCode)),
                    button(text(fl!("acp-claude")).size(13)).style(button::secondary).on_press(Message::Start(Target::Claude)),
                ]
                .spacing(8));
        }
        let mut transcript = column![].spacing(8).width(Fill);
        for line in &self.lines {
            transcript = transcript.push(line_view(line));
        }
        let scroll = tea_core::style::scrollable(transcript).height(Fill);
        let mut bottom = column![].spacing(8).width(Fill);
        if let Some(pending) = &self.pending {
            bottom = bottom.push(permission_card(pending));
        }
        let send = if self.prompt.trim().is_empty() || self.cmd.is_none() {
            button(text(fl!("acp-send")).size(13))
        } else {
            button(text(fl!("acp-send")).size(13)).on_press(Message::Send)
        };
        let mut actions = row![
            text_input(&fl!("acp-prompt"), &self.prompt).on_input(Message::Prompt).on_submit(Message::Send).size(13).width(Fill),
            send,
        ]
        .spacing(8);
        if self.busy {
            actions = actions.push(button(text(fl!("acp-cancel")).size(13)).style(button::secondary).on_press(Message::Cancel));
        }
        bottom = bottom.push(actions);
        column![body, scroll, bottom].spacing(8).height(Fill).into()
    }
}

fn append_text(lines: &mut Vec<Line>, assistant: bool, text: String) {
    if text.is_empty() {
        return;
    }
    match lines.last_mut() {
        Some(Line::Assistant(prev)) if assistant => prev.push_str(&text),
        Some(Line::Thought(prev)) if !assistant => prev.push_str(&text),
        _ if assistant => lines.push(Line::Assistant(text)),
        _ => lines.push(Line::Thought(text)),
    }
}

fn line_view(line: &Line) -> Element<'_, Message> {
    match line {
        Line::You(text_value) => text(format!("{}  {text_value}", fl!("acp-you"))).size(13).width(Fill).into(),
        Line::Assistant(text_value) => text(text_value.clone()).size(13).width(Fill).into(),
        Line::Thought(text_value) => text(format!("{}  {text_value}", fl!("acp-thought"))).size(12).style(text::secondary).width(Fill).into(),
        Line::Tool { title, status } => text(format!("{title}  {}", status_text(status))).size(12).style(text::secondary).width(Fill).into(),
        Line::Note(text_value) => text(text_value.clone()).size(12).style(text::secondary).width(Fill).into(),
    }
}

fn permission_card(pending: &Pending) -> Element<'_, Message> {
    let mut card = column![
        text(fl!("acp-permission-title")).size(13),
        text(pending.title.clone()).size(13).width(Fill),
        text(fl!("acp-permission-note")).size(12).style(text::warning).width(Fill),
    ]
    .spacing(4)
    .padding(10)
    .width(Fill);
    if !pending.detail.is_empty() {
        card = card.push(text(pending.detail.clone()).size(12).style(text::secondary).width(Fill));
    }
    let mut buttons = row![].spacing(6);
    for (i, option) in pending.options.iter().enumerate() {
        let index = u32::try_from(i).unwrap_or(u32::MAX);
        buttons = buttons.push(button(text(option.name.clone()).size(12)).style(option_style(option.kind)).on_press(Message::Choose(index)));
    }
    buttons = buttons.push(button(text(fl!("acp-cancel")).size(12)).style(button::secondary).on_press(Message::Cancel));
    card = card.push(buttons);
    card.into()
}

fn option_style(kind: Kind) -> fn(&iced::Theme, button::Status) -> button::Style {
    if kind.is_allow() { button::primary } else { button::secondary }
}

fn permission_dialog(title: &str, detail: &str, options: &[ShownOption]) -> Dialog {
    let mut message = format!("{title}\n\n{}", fl!("acp-permission-note"));
    if !detail.is_empty() {
        message = format!("{title}\n{detail}\n\n{}", fl!("acp-permission-note"));
    }
    // Todos secundários: Enter e Esc fecham sem escolher. Permitir é um clique.
    let mut dialog = Dialog::new(fl!("acp-permission-title"), message);
    for (i, option) in options.iter().enumerate() {
        let index = u32::try_from(i).unwrap_or(u32::MAX);
        dialog = dialog.button(option.name.clone(), ButtonKind::Secondary, Message::Choose(index));
    }
    dialog.button(fl!("acp-cancel"), ButtonKind::Secondary, Message::Cancel)
}

fn status_text(status: &str) -> String {
    match status {
        "pending" => fl!("acp-status-pending"),
        "in_progress" => fl!("acp-status-in-progress"),
        "completed" => fl!("acp-status-completed"),
        "failed" => fl!("acp-status-failed"),
        "update" => fl!("acp-status-update"),
        other => other.to_string(),
    }
}

fn stop_text(stop: &str) -> String {
    match stop {
        "end_turn" => fl!("acp-stop-end"),
        "cancelled" => fl!("acp-stop-cancelled"),
        "refusal" => fl!("acp-stop-refusal"),
        "error" => fl!("acp-stop-error"),
        other => other.to_string(),
    }
}

fn missing_text(missing: &Missing) -> String {
    match missing {
        Missing::OpenCode => fl!("acp-missing-opencode"),
        Missing::Claude { node } => match node {
            NodeReport::Absent => fl!("acp-missing-claude-absent"),
            NodeReport::Old { version } => fl!("acp-missing-claude-old", version = version.clone()),
            NodeReport::NewEnough { version } => fl!("acp-missing-claude-node", version = version.clone()),
            NodeReport::Unreadable { output } => fl!("acp-missing-claude-unread", output = output.clone()),
        },
    }
}

fn claude_bin() -> Option<PathBuf> {
    let global = tea_core::config::global_config_path()?;
    tea_core::config::Config::load(Some(&global), None).agents.claude_acp.filter(|s| !s.is_empty()).map(PathBuf::from)
}

fn watch(key: &SessionKey) -> iced_mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced_mpsc::unbounded();
    let (cmd_tx, cmd_rx) = mpsc::unbounded();
    let launch = Launch {
        target: if key.args.first().is_some_and(|a| a == "acp") { Target::OpenCode } else { Target::Claude },
        program: key.program.clone(),
        args: key.args.clone(),
    };
    let cwd = key.cwd.clone();
    std::thread::spawn(move || {
        if tx.unbounded_send(Message::Bound(cmd_tx)).is_err() {
            return;
        }
        tea_acp::session::run(&launch, &cwd, cmd_rx, move |event| {
            let _ = tx.unbounded_send(Message::Event(event));
        });
    });
    rx
}

impl Extension for AcpExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "acp"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("acp-panel"), Slot::Right);
        ctx.panel_icon(PANEL, Icon::CommentDiscussion);
        ctx.register_command("acp.focus", fl!("cmd-acp-focus"));
        ctx.register_command("acp.opencode", fl!("cmd-acp-opencode"));
        ctx.register_command("acp.claude", fl!("cmd-acp-claude"));
        ctx.register_command("acp.cancel", fl!("cmd-acp-cancel"));
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "acp.focus" => ctx.show_panel(PANEL),
            "acp.opencode" => self.launch(Target::OpenCode, ctx),
            "acp.claude" => self.launch(Target::Claude, ctx),
            "acp.cancel" => {
                self.busy = false;
                self.send(Command::Cancel);
            }
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Start(target) => self.launch(target, ctx),
            Message::Bound(cmd) => self.cmd = Some(cmd),
            Message::Event(event) => self.on_event_msg(event, ctx),
            Message::Prompt(text) => self.prompt = text,
            Message::Send => {
                let text = self.prompt.trim().to_string();
                if !text.is_empty() && self.cmd.is_some() {
                    self.lines.push(Line::You(text.clone()));
                    self.prompt.clear();
                    self.busy = true;
                    self.send(Command::Prompt(text));
                }
            }
            Message::Cancel => {
                self.busy = false;
                self.pending = None;
                self.send(Command::Cancel);
            }
            Message::Choose(index) => self.choose(index),
            Message::Login(index) => {
                if let (Some(program), Some(method)) = (self.program.clone(), self.logins.get(index)) {
                    let command = target::login_command(&program, &method.args);
                    let done = fl!("acp-login-done", code = "{code}");
                    ctx.run_task(fl!("acp-login-task", name = method.name.clone()), command, self.root.clone(), done);
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        self.view_panel()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.key {
            Some(key) => Subscription::run_with(key.clone(), watch),
            None => Subscription::none(),
        }
    }
}
