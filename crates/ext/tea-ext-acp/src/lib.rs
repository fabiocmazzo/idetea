//! Painel das sessões ACP (RFC 0022): cada agente é um processo, a IDE mostra o texto
//! e espera uma permissão explícita antes de cada ação. Não raspa o terminal.
//!
//! Várias sessões convivem, uma aba cada. Abrir uma sessão nova não encerra as outras;
//! só fechar a aba (ou o processo cair) encerra aquela sessão.

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
use tea_core::{ButtonKind, Context, Dialog, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("acp");

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
}

#[derive(Default)]
pub struct AcpExtension {
    root: PathBuf,
    next_id: SessionId,
    sessions: Vec<Session>,
    active: Option<SessionId>,
    /// Aviso fora de qualquer sessão (binário ausente, Node velho...).
    notice: Option<String>,
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
        });
        self.active = Some(id);
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

    fn select(&mut self, id: SessionId) {
        if let Some(session) = self.session_mut(id) {
            session.attention = false;
            self.active = Some(id);
        }
    }

    /// Fecha a aba e encerra só aquela sessão.
    fn close(&mut self, id: SessionId) {
        let Some(index) = self.sessions.iter().position(|s| s.id == id) else { return };
        let session = self.sessions.remove(index);
        session.send(Command::Shutdown);
        if self.active == Some(id) {
            let next = index.min(self.sessions.len().saturating_sub(1));
            self.active = self.sessions.get(next).map(|s| s.id);
        }
    }

    /// O mascote continua pelos hooks (`idetea mascot setup-hooks`). `mascot_cue` é o gancho
    /// para um marco seguinte, sem gravar o estado daqui e duplicar o que o hook já faz.
    fn on_event_msg(&mut self, id: SessionId, event: SessionEvent, ctx: &mut Context) {
        let front = self.active == Some(id);
        let Some(s) = self.session_mut(id) else { return };
        match event {
            SessionEvent::Ready { agent } => {
                s.lines.push(Line::Note(fl!("acp-ready", id = agent)));
            }
            SessionEvent::Assistant { text } => append_text(&mut s.lines, true, text),
            SessionEvent::Thought { text } => append_text(&mut s.lines, false, text),
            SessionEvent::Tool { title, status, .. } => {
                s.lines.push(Line::Tool { title, status });
            }
            SessionEvent::Permission { title, detail, options } => {
                s.pending = Some(Pending { title: title.clone(), detail: detail.clone(), options: options.clone() });
                s.attention = !front;
                ctx.show_dialog(permission_dialog(id, &s.label, &title, &detail, &options));
            }
            SessionEvent::Turn { stop } => {
                s.busy = false;
                s.lines.push(Line::Note(fl!("acp-turn", stop = stop_text(&stop))));
            }
            SessionEvent::AuthRequired { message, logins } => {
                s.busy = false;
                s.cmd = None;
                s.logins = logins;
                s.notice = Some(fl!("acp-auth", message = message));
                s.key = None;
            }
            SessionEvent::Failed { message } => {
                s.busy = false;
                if message != "shutdown" {
                    s.notice = Some(fl!("acp-failed", error = message));
                }
            }
            SessionEvent::Closed => {
                s.busy = false;
                s.cmd = None;
                s.key = None;
                s.pending = None;
                if s.notice.is_none() && s.logins.is_empty() {
                    s.lines.push(Line::Note(fl!("acp-closed")));
                }
            }
        }
    }

    fn choose(&mut self, id: SessionId, index: u32) {
        let Some(s) = self.session_mut(id) else { return };
        if s.pending.as_ref().is_some_and(|p| (index as usize) < p.options.len()) {
            s.send(Command::Permission(UserChoice::Option(index)));
            s.pending = None;
            s.attention = false;
        }
    }

    fn cancel(&mut self, id: SessionId) {
        if let Some(s) = self.session_mut(id) {
            s.busy = false;
            s.pending = None;
            s.attention = false;
            s.send(Command::Cancel);
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
        let scroll = tea_core::style::scrollable(transcript).height(Fill);
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
                if let Some(id) = self.active {
                    self.cancel(id);
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
            Message::Event(id, event) => self.on_event_msg(id, event, ctx),
            Message::Select(id) => self.select(id),
            Message::Close(id) => self.close(id),
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
                    }
                }
            }
            Message::Cancel(id) => self.cancel(id),
            Message::Choose(id, index) => self.choose(id, index),
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
        ext.select(ids[1]);
        ext.close(ids[1]);
        assert_eq!(ext.sessions.iter().map(|s| s.id).collect::<Vec<_>>(), [ids[0], ids[2]]);
        assert_eq!(ext.active, Some(ids[2]), "a aba ao lado fica à frente");
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)), "a outra sessão não recebeu nada");
        ext.close(ids[2]);
        ext.close(ids[0]);
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
        ext.on_event_msg(ids[0], SessionEvent::Permission { title: "echo oi".into(), detail: String::new(), options }, &mut ctx);
        let first = ext.session(ids[0]).unwrap();
        assert!(first.pending.is_some());
        assert!(first.attention, "a aba de trás fica marcada");
        assert!(ext.session(ids[1]).unwrap().pending.is_none());
        ext.on_event_msg(ids[1], SessionEvent::Closed, &mut ctx);
        assert!(!ext.session(ids[1]).unwrap().alive());
        assert!(ext.session(ids[0]).unwrap().alive());
        ext.choose(ids[0], 0);
        assert!(matches!(rx0.try_recv(), Ok(Command::Permission(UserChoice::Option(0)))));
        assert!(matches!(rx1.try_recv(), Err(TryRecvError::Closed)), "a sessão encerrada não recebe a escolha");
        assert!(!ext.session(ids[0]).unwrap().attention);
    }
}
