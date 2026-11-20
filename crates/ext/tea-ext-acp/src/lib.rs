//! Painel das sessões ACP (RFC 0022): cada agente é um processo, a IDE mostra o texto
//! e espera uma permissão explícita antes de cada ação. Não raspa o terminal.
//!
//! Várias sessões convivem, uma aba cada. Abrir uma sessão nova não encerra as outras;
//! só fechar a aba (ou o processo cair) encerra aquela sessão.
//!
//! A aba guarda a conversa: o que você mandou, a resposta, uma linha por ferramenta e o pedido de
//! permissão. O estado (pensando, qual ferramenta, fim do turno, erro) vai para o mascote, por
//! [`AgentCue`]. A conversa rola sozinha até o fim, a não ser que você tenha subido para ler.

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
use tea_core::icons::{self, Icon};
use tea_core::{AgentCue, ButtonKind, Context, Dialog, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("acp");
/// A conversa da aba à frente.
const TRANSCRIPT: &str = "acp-transcript";
/// Até quantos pixels do fim ainda conta como "no fim" (segue o texto novo).
const FOLLOW_SLACK: f32 = 24.0;

/// Identifica uma sessão enquanto a extensão vive; nunca é reaproveitado.
pub type SessionId = u64;

#[derive(Debug, Clone)]
pub enum Message {
    Start(Target),
    Bound(SessionId, mpsc::UnboundedSender<Command>),
    Event(SessionId, SessionEvent),
    Select(SessionId),
    Close(SessionId),
    Prompt(String),
    Send,
    Cancel(SessionId),
    Choose(SessionId, u32),
    Login(SessionId, usize),
    /// A conversa rolou; `true` quando está no fim.
    Scrolled(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SessionKey {
    id: SessionId,
    cwd: PathBuf,
    program: PathBuf,
    args: Vec<String>,
}

#[derive(Debug, Clone)]
enum Line {
    You(String),
    Assistant(String),
    /// Uma ferramenta: as atualizações mudam a mesma linha (pelo `id`).
    Tool { id: String, title: String, status: String },
}

#[derive(Debug, Clone)]
struct Pending {
    title: String,
    detail: String,
    options: Vec<ShownOption>,
}

struct Session {
    id: SessionId,
    label: String,
    /// `None` quando o processo acabou: a aba fica com a conversa até ser fechada.
    key: Option<SessionKey>,
    cmd: Option<mpsc::UnboundedSender<Command>>,
    program: PathBuf,
    lines: Vec<Line>,
    prompt: String,
    pending: Option<Pending>,
    logins: Vec<LoginMethod>,
    notice: Option<String>,
    busy: bool,
    /// Algo pede o usuário (permissão) numa aba que não está à frente.
    attention: bool,
    /// O último "trabalhando" contado ao mascote, para não repetir a cada pedaço de texto.
    doing: Option<String>,
}

impl Session {
    fn send(&self, command: Command) {
        if let Some(cmd) = &self.cmd {
            let _ = cmd.unbounded_send(command);
        }
    }

    fn alive(&self) -> bool {
        self.key.is_some()
    }

    /// Conta ao mascote o que a sessão está fazendo, só quando muda.
    fn busy_with(&mut self, what: String, ctx: &mut Context) {
        if self.doing.as_deref() != Some(what.as_str()) {
            self.doing = Some(what.clone());
            ctx.agent_cue(self.label.clone(), AgentCue::Busy(Some(what)));
        }
    }

    /// Um marco que não é "trabalhando": o próximo "trabalhando" é contado de novo.
    fn cue(&mut self, cue: AgentCue, ctx: &mut Context) {
        self.doing = None;
        ctx.agent_cue(self.label.clone(), cue);
    }
}

#[derive(Default)]
pub struct AcpExtension {
    root: PathBuf,
    next_id: SessionId,
    sessions: Vec<Session>,
    active: Option<SessionId>,
    /// Aviso fora de qualquer sessão (binário ausente, Node velho...).
    notice: Option<String>,
    /// Você subiu na conversa para ler: o texto novo não puxa a rolagem.
    reading_back: bool,
}

impl AcpExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    fn active_mut(&mut self) -> Option<&mut Session> {
        let id = self.active?;
        self.session_mut(id)
    }

    fn label_for(&self, target: Target) -> String {
        let base = match target {
            Target::OpenCode => fl!("acp-opencode"),
            Target::Claude => fl!("acp-claude"),
        };
        let taken: Vec<&str> = self.sessions.iter().map(|s| s.label.as_str()).collect();
        if !taken.contains(&base.as_str()) {
            return base;
        }
        (2..).map(|n| format!("{base} {n}")).find(|l| !taken.contains(&l.as_str())).unwrap_or(base)
    }

    /// Abre uma sessão nova numa aba nova. As outras continuam como estão.
    fn start(&mut self, target: Target, launch: Launch, ctx: &mut Context) -> SessionId {
        self.next_id += 1;
        let id = self.next_id;
        let label = self.label_for(target);
        self.sessions.push(Session {
            id,
            label,
            key: Some(SessionKey { id, cwd: self.root.clone(), program: launch.program.clone(), args: launch.args }),
            cmd: None,
            program: launch.program,
            lines: Vec::new(),
            prompt: String::new(),
            pending: None,
            logins: Vec::new(),
            notice: None,
            busy: false,
            attention: false,
            doing: None,
        });
        self.active = Some(id);
        self.reading_back = false;
        self.notice = None;
        ctx.show_panel(PANEL);
        id
    }

    fn launch(&mut self, target: Target, ctx: &mut Context) {
        let configured = claude_bin();
        let found = target::system_probe(configured.as_deref()).resolve(target);
        match found {
            Ok(launch) => {
                self.start(target, launch, ctx);
            }
            Err(missing) => {
                self.notice = Some(missing_text(&missing));
                ctx.show_panel(PANEL);
            }
        }
    }

    fn select(&mut self, id: SessionId) -> Task<Message> {
        let Some(session) = self.session_mut(id) else { return Task::none() };
        session.attention = false;
        self.active = Some(id);
        self.reading_back = false;
        follow()
    }

    /// Fecha a aba e encerra só aquela sessão. O mascote esquece o que ela fazia.
    fn close(&mut self, id: SessionId, ctx: &mut Context) {
        let Some(index) = self.sessions.iter().position(|s| s.id == id) else { return };
        let session = self.sessions.remove(index);
        session.send(Command::Shutdown);
        if session.alive() {
            ctx.agent_cue(session.label, AgentCue::Busy(None));
        }
        if self.active == Some(id) {
            let next = index.min(self.sessions.len().saturating_sub(1));
            self.active = self.sessions.get(next).map(|s| s.id);
        }
    }

    /// A conversa vai para a aba; o estado, para o mascote. Devolve a rolagem até o fim quando
    /// a aba é a da frente, chegou texto novo e você não subiu para ler.
    fn on_event_msg(&mut self, id: SessionId, event: SessionEvent, ctx: &mut Context) -> Task<Message> {
        let front = self.active == Some(id);
        let Some(s) = self.session_mut(id) else { return Task::none() };
        let mut grew = false;
        match event {
            SessionEvent::Ready { .. } => s.cue(AgentCue::Ready, ctx),
            SessionEvent::Assistant { text } => {
                grew = append_text(&mut s.lines, text);
                if grew {
                    s.busy_with(fl!("acp-doing-writing"), ctx);
                }
            }
            SessionEvent::Thought { text } => {
                if !text.is_empty() {
                    s.busy_with(fl!("acp-doing-thinking"), ctx);
                }
            }
            SessionEvent::Tool { id: tool, title, status } => {
                grew = upsert_tool(&mut s.lines, tool.clone(), title, status);
                let running = s.lines.iter().rev().find_map(|l| match l {
                    Line::Tool { id, title, status } if *id == tool && (status == "pending" || status == "in_progress") && !title.is_empty() => {
                        Some(title.clone())
                    }
                    _ => None,
                });
                if let Some(title) = running {
                    s.busy_with(title, ctx);
                }
            }
            SessionEvent::Permission { title, detail, options } => {
                s.pending = Some(Pending { title: title.clone(), detail: detail.clone(), options: options.clone() });
                s.attention = !front;
                grew = true;
                s.cue(AgentCue::Permission, ctx);
                ctx.show_dialog(permission_dialog(id, &s.label, &title, &detail, &options));
            }
            SessionEvent::Turn { stop } => {
                s.busy = false;
                let cue = match stop.as_str() {
                    "end_turn" | "max_tokens" | "max_turn_requests" => AgentCue::Done,
                    "refusal" => AgentCue::Refused,
                    "error" => AgentCue::Failed,
                    _ => AgentCue::Busy(None),
                };
                s.cue(cue, ctx);
            }
            SessionEvent::AuthRequired { message, logins } => {
                s.busy = false;
                s.cmd = None;
                s.logins = logins;
                s.notice = Some(fl!("acp-auth", message = message));
                s.key = None;
                s.cue(AgentCue::Busy(None), ctx);
            }
            SessionEvent::Failed { message } => {
                s.busy = false;
                if message != "shutdown" {
                    s.notice = Some(fl!("acp-failed", error = message));
                    s.cue(AgentCue::Failed, ctx);
                }
            }
            SessionEvent::Closed => {
                let quiet_end = s.notice.is_some() || !s.logins.is_empty();
                s.busy = false;
                s.cmd = None;
                s.pending = None;
                if s.key.take().is_some() && !quiet_end {
                    s.cue(AgentCue::Ended, ctx);
                }
            }
        }
        if grew && front && !self.reading_back { follow() } else { Task::none() }
    }

    fn choose(&mut self, id: SessionId, index: u32, ctx: &mut Context) {
        let Some(s) = self.session_mut(id) else { return };
        if s.pending.as_ref().is_some_and(|p| (index as usize) < p.options.len()) {
            s.send(Command::Permission(UserChoice::Option(index)));
            s.pending = None;
            s.attention = false;
            s.cue(AgentCue::Busy(None), ctx);
            s.busy_with(fl!("acp-doing-working"), ctx);
        }
    }

    fn cancel(&mut self, id: SessionId, ctx: &mut Context) {
        if let Some(s) = self.session_mut(id) {
            s.busy = false;
            s.pending = None;
            s.attention = false;
            s.send(Command::Cancel);
            if s.alive() {
                s.cue(AgentCue::Busy(None), ctx);
            }
        }
    }

    fn view_tabs(&self) -> Element<'_, Message> {
        let mut tabs = row![].spacing(4);
        for s in &self.sessions {
            let mut label = s.label.clone();
            if s.attention {
                label = format!("● {label}");
            } else if !s.alive() {
                label = fl!("acp-tab-ended", name = label);
            }
            let style = if self.active == Some(s.id) { button::primary } else { button::secondary };
            let close = button(icons::icon(Icon::Close).size(12)).style(button::text).padding(2).on_press(Message::Close(s.id));
            tabs = tabs.push(
                row![button(text(label).size(12)).style(style).padding([3, 8]).on_press(Message::Select(s.id)), close]
                    .spacing(0)
                    .align_y(iced::Alignment::Center),
            );
        }
        let new = |label: String, target| {
            button(row![icons::icon(Icon::Add).size(12), text(label).size(12)].spacing(4)).style(button::text).padding([3, 6]).on_press(Message::Start(target))
        };
        tabs = tabs.push(new(fl!("acp-opencode"), Target::OpenCode)).push(new(fl!("acp-claude"), Target::Claude));
        tea_core::style::scrollable(tabs).direction(iced::widget::scrollable::Direction::Horizontal(Default::default())).width(Fill).into()
    }

    fn view_panel(&self) -> Element<'_, Message> {
        let mut body = column![].spacing(8).padding(12).width(Fill);
        if let Some(notice) = &self.notice {
            body = body.push(text(notice).size(13).style(text::warning).width(Fill));
        }
        let Some(s) = self.active.and_then(|id| self.session(id)) else {
            body = body
                .push(text(fl!("acp-empty")).size(13).width(Fill))
                .push(row![
                    button(text(fl!("acp-opencode")).size(13)).on_press(Message::Start(Target::OpenCode)),
                    button(text(fl!("acp-claude")).size(13)).style(button::secondary).on_press(Message::Start(Target::Claude)),
                ]
                .spacing(8));
            return body.into();
        };
        body = body.push(self.view_tabs());
        if let Some(notice) = &s.notice {
            body = body.push(text(notice).size(13).style(text::warning).width(Fill));
        }
        for (i, login) in s.logins.iter().enumerate() {
            let label = fl!("acp-login", name = login.name.clone());
            body = body.push(button(text(label).size(13)).style(button::secondary).on_press(Message::Login(s.id, i)));
            if !login.description.is_empty() {
                body = body.push(text(login.description.clone()).size(12).style(text::secondary).width(Fill));
            }
        }
        let mut transcript = column![].spacing(8).width(Fill);
        for line in &s.lines {
            transcript = transcript.push(line_view(line));
        }
        let scroll = tea_core::style::scrollable(transcript)
            .id(TRANSCRIPT)
            .on_scroll(|v| Message::Scrolled(v.absolute_offset_reversed().y <= FOLLOW_SLACK))
            .height(Fill);
        let mut bottom = column![].spacing(8).width(Fill);
        if let Some(pending) = &s.pending {
            bottom = bottom.push(permission_card(s.id, pending));
        }
        let send = if s.prompt.trim().is_empty() || s.cmd.is_none() {
            button(text(fl!("acp-send")).size(13))
        } else {
            button(text(fl!("acp-send")).size(13)).on_press(Message::Send)
        };
        let mut actions = row![
            text_input(&fl!("acp-prompt"), &s.prompt).on_input(Message::Prompt).on_submit(Message::Send).size(13).width(Fill),
            send,
        ]
        .spacing(8);
        if s.busy {
            actions = actions.push(button(text(fl!("acp-cancel")).size(13)).style(button::secondary).on_press(Message::Cancel(s.id)));
        }
        bottom = bottom.push(actions);
        column![body, scroll, bottom].spacing(8).height(Fill).into()
    }
}

/// Rola a conversa até o fim.
fn follow() -> Task<Message> {
    iced::widget::operation::snap_to_end(TRANSCRIPT)
}

/// Junta o pedaço de resposta ao último bloco dela. `true` se a conversa cresceu.
fn append_text(lines: &mut Vec<Line>, text: String) -> bool {
    if text.is_empty() {
        return false;
    }
    match lines.last_mut() {
        Some(Line::Assistant(prev)) => prev.push_str(&text),
        _ => lines.push(Line::Assistant(text)),
    }
    true
}

/// Uma linha por ferramenta: a atualização muda a linha que já existe. Título vazio ou status
/// "update" não apagam o que já se sabia. `true` se apareceu uma linha nova.
fn upsert_tool(lines: &mut Vec<Line>, id: String, title: String, status: String) -> bool {
    let existing = lines.iter_mut().rev().find_map(|l| match l {
        Line::Tool { id: known, title, status } if *known == id => Some((title, status)),
        _ => None,
    });
    if let Some((known_title, known_status)) = existing {
        if !title.is_empty() {
            *known_title = title;
        }
        if status != "update" {
            *known_status = status;
        }
        return false;
    }
    lines.push(Line::Tool { id, title, status });
    true
}

fn line_view(line: &Line) -> Element<'_, Message> {
    match line {
        Line::You(text_value) => text(format!("{}  {text_value}", fl!("acp-you"))).size(13).width(Fill).into(),
        Line::Assistant(text_value) => text(text_value.clone()).size(13).width(Fill).into(),
        Line::Tool { title, status, .. } => text(format!("{title}  {}", status_text(status))).size(12).style(text::secondary).width(Fill).into(),
    }
}

fn permission_card(id: SessionId, pending: &Pending) -> Element<'_, Message> {
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
        buttons = buttons.push(button(text(option.name.clone()).size(12)).style(option_style(option.kind)).on_press(Message::Choose(id, index)));
    }
    buttons = buttons.push(button(text(fl!("acp-cancel")).size(12)).style(button::secondary).on_press(Message::Cancel(id)));
    card = card.push(buttons);
    card.into()
}

fn option_style(kind: Kind) -> fn(&iced::Theme, button::Status) -> button::Style {
    if kind.is_allow() { button::primary } else { button::secondary }
}

fn permission_dialog(id: SessionId, session: &str, title: &str, detail: &str, options: &[ShownOption]) -> Dialog {
    let from = fl!("acp-permission-session", session = session.to_string());
    let note = fl!("acp-permission-note");
    let message = if detail.is_empty() { format!("{from}\n\n{title}\n\n{note}") } else { format!("{from}\n\n{title}\n{detail}\n\n{note}") };
    // Todos secundários: Enter e Esc fecham sem escolher. Permitir é um clique.
    let mut dialog = Dialog::new(fl!("acp-permission-title"), message);
    for (i, option) in options.iter().enumerate() {
        let index = u32::try_from(i).unwrap_or(u32::MAX);
        dialog = dialog.button(option.name.clone(), ButtonKind::Secondary, Message::Choose(id, index));
    }
    dialog.button(fl!("acp-cancel"), ButtonKind::Secondary, Message::Cancel(id))
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
    let id = key.id;
    let launch = Launch {
        target: if key.args.first().is_some_and(|a| a == "acp") { Target::OpenCode } else { Target::Claude },
        program: key.program.clone(),
        args: key.args.clone(),
    };
    let cwd = key.cwd.clone();
    std::thread::spawn(move || {
        if tx.unbounded_send(Message::Bound(id, cmd_tx)).is_err() {
            return;
        }
        tea_acp::session::run(&launch, &cwd, cmd_rx, move |event| {
            let _ = tx.unbounded_send(Message::Event(id, event));
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
        ctx.panel_icon(PANEL, Icon::Hubot);
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
                if let Some(id) = self.active {
                    self.cancel(id, ctx);
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Start(target) => self.launch(target, ctx),
            Message::Bound(id, cmd) => {
                if let Some(s) = self.session_mut(id) {
                    s.cmd = Some(cmd);
                }
            }
            Message::Event(id, event) => return self.on_event_msg(id, event, ctx),
            Message::Select(id) => return self.select(id),
            Message::Close(id) => self.close(id, ctx),
            Message::Scrolled(at_end) => self.reading_back = !at_end,
            Message::Prompt(text) => {
                if let Some(s) = self.active_mut() {
                    s.prompt = text;
                }
            }
            Message::Send => {
                if let Some(s) = self.active_mut() {
                    let text = s.prompt.trim().to_string();
                    if !text.is_empty() && s.cmd.is_some() {
                        s.lines.push(Line::You(text.clone()));
                        s.prompt.clear();
                        s.busy = true;
                        s.send(Command::Prompt(text));
                        s.busy_with(fl!("acp-doing-thinking"), ctx);
                        self.reading_back = false;
                        return follow();
                    }
                }
            }
            Message::Cancel(id) => self.cancel(id, ctx),
            Message::Choose(id, index) => self.choose(id, index, ctx),
            Message::Login(id, index) => {
                if let Some(s) = self.session(id)
                    && let Some(method) = s.logins.get(index)
                {
                    let command = target::login_command(&s.program, &method.args);
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
        Subscription::batch(self.sessions.iter().filter_map(|s| s.key.as_ref()).map(|key| Subscription::run_with(key.clone(), watch)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc::TryRecvError;
    use std::path::Path;

    fn launch(target: Target) -> Launch {
        let args = match target {
            Target::OpenCode => vec!["acp".to_string()],
            Target::Claude => Vec::new(),
        };
        Launch { target, program: PathBuf::from("agente"), args }
    }

    fn ext_with(targets: &[Target]) -> (AcpExtension, Vec<SessionId>) {
        let mut ext = AcpExtension::new();
        let root = Path::new("/tmp");
        let mut ctx = Context::new(root, None);
        let ids = targets.iter().map(|t| ext.start(*t, launch(*t), &mut ctx)).collect();
        (ext, ids)
    }

    #[test]
    fn a_new_session_keeps_the_previous_one() {
        let (ext, ids) = ext_with(&[Target::OpenCode, Target::Claude]);
        assert_eq!(ext.sessions.len(), 2);
        assert!(ext.sessions.iter().all(Session::alive));
        assert_eq!(ext.active, Some(ids[1]));
        let keys: Vec<_> = ext.sessions.iter().filter_map(|s| s.key.as_ref()).collect();
        assert_ne!(keys[0], keys[1], "cada sessão tem a sua assinatura");
    }

    #[test]
    fn labels_are_numbered_per_target() {
        let (ext, _) = ext_with(&[Target::Claude, Target::OpenCode, Target::Claude, Target::Claude]);
        let labels: Vec<_> = ext.sessions.iter().map(|s| s.label.as_str()).collect();
        let claude = fl!("acp-claude");
        assert_eq!(labels[0], claude);
        assert_eq!(labels[2], format!("{claude} 2"));
        assert_eq!(labels[3], format!("{claude} 3"));
    }

    #[test]
    fn closing_a_tab_ends_only_that_session() {
        let (mut ext, ids) = ext_with(&[Target::OpenCode, Target::Claude, Target::OpenCode]);
        let (tx, mut rx) = mpsc::unbounded();
        ext.session_mut(ids[0]).unwrap().cmd = Some(tx);
        let mut ctx = Context::new(Path::new("/tmp"), None);
        let _ = ext.select(ids[1]);
        ext.close(ids[1], &mut ctx);
        assert_eq!(ext.sessions.iter().map(|s| s.id).collect::<Vec<_>>(), [ids[0], ids[2]]);
        assert_eq!(ext.active, Some(ids[2]), "a aba ao lado fica à frente");
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)), "a outra sessão não recebeu nada");
        ext.close(ids[2], &mut ctx);
        ext.close(ids[0], &mut ctx);
        assert!(matches!(rx.try_recv(), Ok(Command::Shutdown)));
        assert_eq!(ext.active, None);
    }

    #[test]
    fn events_and_choices_go_to_their_own_session() {
        let (mut ext, ids) = ext_with(&[Target::OpenCode, Target::Claude]);
        let (tx0, mut rx0) = mpsc::unbounded();
        let (tx1, mut rx1) = mpsc::unbounded();
        ext.session_mut(ids[0]).unwrap().cmd = Some(tx0);
        ext.session_mut(ids[1]).unwrap().cmd = Some(tx1);
        let root = Path::new("/tmp");
        let mut ctx = Context::new(root, None);
        let options = vec![ShownOption { id: "allow".into(), name: "Permitir".into(), kind: Kind::AllowOnce }];
        let _ = ext.on_event_msg(ids[0], SessionEvent::Permission { title: "echo oi".into(), detail: String::new(), options }, &mut ctx);
        let first = ext.session(ids[0]).unwrap();
        assert!(first.pending.is_some());
        assert!(first.attention, "a aba de trás fica marcada");
        assert!(ext.session(ids[1]).unwrap().pending.is_none());
        let _ = ext.on_event_msg(ids[1], SessionEvent::Closed, &mut ctx);
        assert!(!ext.session(ids[1]).unwrap().alive());
        assert!(ext.session(ids[0]).unwrap().alive());
        ext.choose(ids[0], 0, &mut ctx);
        assert!(matches!(rx0.try_recv(), Ok(Command::Permission(UserChoice::Option(0)))));
        assert!(matches!(rx1.try_recv(), Err(TryRecvError::Closed)), "a sessão encerrada não recebe a escolha");
        assert!(!ext.session(ids[0]).unwrap().attention);
    }

    fn cues(ctx: Context) -> Vec<AgentCue> {
        ctx.into_requests()
            .into_iter()
            .filter_map(|r| match r {
                tea_core::Request::AgentCue { cue, .. } => Some(cue),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn tool_updates_change_the_same_line() {
        let mut lines = Vec::new();
        assert!(upsert_tool(&mut lines, "t1".into(), "Read a.rs".into(), "pending".into()));
        assert!(!upsert_tool(&mut lines, "t1".into(), String::new(), "in_progress".into()));
        assert!(!upsert_tool(&mut lines, "t1".into(), String::new(), "update".into()));
        assert!(upsert_tool(&mut lines, "t2".into(), "Edit b.rs".into(), "pending".into()));
        assert!(!upsert_tool(&mut lines, "t1".into(), String::new(), "completed".into()));
        assert_eq!(lines.len(), 2);
        assert!(matches!(&lines[0], Line::Tool { title, status, .. } if title == "Read a.rs" && status == "completed"));
    }

    #[test]
    fn state_goes_to_the_mascot_and_text_to_the_tab() {
        let (mut ext, ids) = ext_with(&[Target::Claude]);
        let mut ctx = Context::new(Path::new("/tmp"), None);
        let id = ids[0];
        for event in [
            SessionEvent::Ready { agent: "claude".into() },
            SessionEvent::Thought { text: "hmm".into() },
            SessionEvent::Thought { text: " more".into() },
            SessionEvent::Assistant { text: "Oi".into() },
            SessionEvent::Assistant { text: "!".into() },
            SessionEvent::Tool { id: "t".into(), title: "Read a.rs".into(), status: "in_progress".into() },
            SessionEvent::Tool { id: "t".into(), title: String::new(), status: "completed".into() },
            SessionEvent::Turn { stop: "end_turn".into() },
            SessionEvent::Closed,
        ] {
            let _ = ext.on_event_msg(id, event, &mut ctx);
        }
        let thinking = fl!("acp-doing-thinking");
        let writing = fl!("acp-doing-writing");
        assert_eq!(
            cues(ctx),
            [
                AgentCue::Ready,
                AgentCue::Busy(Some(thinking)),
                AgentCue::Busy(Some(writing)),
                AgentCue::Busy(Some("Read a.rs".into())),
                AgentCue::Done,
                AgentCue::Ended,
            ],
            "um aviso por mudança, não um por pedaço de texto"
        );
        let s = ext.session(id).unwrap();
        assert_eq!(s.lines.len(), 2, "a resposta e a ferramenta; pensamento e fim de turno não entram na aba");
        assert!(matches!(&s.lines[0], Line::Assistant(t) if t == "Oi!"));
    }
}
