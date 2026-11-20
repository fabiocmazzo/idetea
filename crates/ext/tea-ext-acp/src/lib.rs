//! Painel das sessões ACP (RFC 0022 e 0023): cada agente é um processo, a IDE mostra o texto
//! e espera uma permissão explícita antes de cada ação. Não raspa o terminal.
//!
//! Várias sessões convivem, uma aba cada. Abrir uma sessão nova não encerra as outras;
//! só fechar a aba (ou o processo cair) encerra aquela sessão. Uma aba encerrada pode recomeçar.
//!
//! A aba guarda a conversa: o que você mandou, a resposta (em Markdown), uma linha por ferramenta
//! (que abre o arquivo e expande a saída ou o diff) e o pedido de permissão. O estado (pensando,
//! qual ferramenta, fim do turno, erro) vai para o mascote, por [`AgentCue`]; com o mascote
//! desligado, para uma linha acima da mensagem. A conversa rola sozinha até o fim, a não ser que
//! você tenha subido para ler.
//!
//! Um turno por vez: o que você manda com um turno aberto espera na fila da aba.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod diff;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::channel::mpsc;
use iced::futures::channel::mpsc as iced_mpsc;
use iced::keyboard::{self, key::Named};
use iced::widget::{button, column, container, markdown, row, stack, text, text_editor, tooltip};
use iced::{Element, Fill, Font, Subscription, Task, Theme};
use tea_acp::permission::{Kind, ShownOption, UserChoice};
use tea_acp::session::{Command, Failure, LoginMethod, PromptPart, SessionEvent, ToolContent, ToolInfo, ToolLocation};
use tea_acp::target::{self, Launch, Missing, NodeReport, Target};
use tea_core::icons::{self, Icon};
use tea_core::pick::Pick;
use tea_core::{AgentCue, ButtonKind, Context, CoreEvent, Dialog, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("acp");
/// A conversa da aba à frente.
const TRANSCRIPT: &str = "acp-transcript";
/// A caixa de mensagem.
const INPUT: &str = "acp-input";
/// Até quantos pixels do fim ainda conta como "no fim" (segue o texto novo).
const FOLLOW_SLACK: f32 = 24.0;
/// Linhas de saída de uma ferramenta mostradas ao expandir.
const TOOL_TEXT_LINES: usize = 200;
/// Arquivos do projeto oferecidos no seletor de `@`.
const PICK_FILES: usize = 20_000;
/// O ponto que gira nas abas de trás que trabalham.
const SPINNER: [char; 4] = ['◐', '◓', '◑', '◒'];

/// Identifica uma sessão enquanto a extensão vive; nunca é reaproveitado.
pub type SessionId = u64;

#[derive(Debug, Clone)]
pub enum Message {
    Start(Target),
    /// A sessão (da geração dada) está ligada.
    Bound(SessionId, u32, mpsc::UnboundedSender<Command>),
    Event(SessionId, u32, Box<SessionEvent>),
    Select(SessionId),
    Close(SessionId),
    /// Abre de novo o mesmo agente numa aba encerrada, com a conversa acima.
    Restart(SessionId),
    Input(text_editor::Action),
    Send,
    /// ↑ na caixa vazia: a última mensagem enviada.
    HistoryUp,
    Unqueue(SessionId, usize),
    Cancel(SessionId),
    Choose(SessionId, u32),
    Login(SessionId, usize),
    /// A conversa rolou; `true` quando está no fim.
    Scrolled(bool),
    ToggleTool(SessionId, usize),
    ToggleDetail(SessionId),
    Open(PathBuf, Option<u32>),
    Copy(String),
    Link(markdown::Uri),
    /// Abre o seletor de arquivos para a mensagem.
    Attach,
    FilesListed(Arc<Vec<PathBuf>>),
    FilePicked(PathBuf),
    RemoveChip(usize),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SessionKey {
    id: SessionId,
    generation: u32,
    cwd: PathBuf,
    program: PathBuf,
    args: Vec<String>,
}

/// Um arquivo (ou umas linhas dele) que vai junto da próxima mensagem.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Chip {
    path: PathBuf,
    lines: Option<(u32, u32)>,
    /// O texto que você vê (seleção, ou o buffer com mudanças não salvas). `None`: o agente lê do disco.
    text: Option<String>,
}

impl Chip {
    fn label(&self, root: &Path) -> String {
        let shown = self.path.strip_prefix(root).unwrap_or(&self.path).display().to_string();
        match self.lines {
            Some((a, b)) if a == b => format!("{shown}:{a}"),
            Some((a, b)) => format!("{shown}:{a}-{b}"),
            None => shown,
        }
    }

    fn part(&self) -> PromptPart {
        PromptPart::File { path: self.path.clone(), lines: self.lines, text: self.text.clone() }
    }
}

/// Uma mensagem esperando o turno aberto acabar.
#[derive(Debug, Clone)]
struct Queued {
    text: String,
    files: Vec<String>,
    parts: Vec<PromptPart>,
}

/// Uma ferramenta, com tudo o que já se sabe dela (as atualizações só trazem o que mudou).
#[derive(Debug, Clone, Default)]
struct ToolState {
    id: String,
    title: String,
    status: String,
    kind: String,
    locations: Vec<ToolLocation>,
    content: Vec<ToolContent>,
    open: bool,
}

impl ToolState {
    fn merge(&mut self, info: ToolInfo) {
        if let Some(title) = info.title.filter(|t| !t.is_empty()) {
            self.title = title;
        }
        if let Some(status) = info.status {
            self.status = status;
        }
        if let Some(kind) = info.kind {
            self.kind = kind;
        }
        if let Some(locations) = info.locations {
            self.locations = locations;
        }
        if let Some(content) = info.content {
            self.content = content;
        }
    }

    fn running(&self) -> bool {
        self.status == "pending" || self.status == "in_progress"
    }
}

#[derive(Debug)]
enum Line {
    You { text: String, files: Vec<String> },
    /// A resposta: o texto original (para copiar) e o Markdown já interpretado.
    Assistant { text: String, md: Box<markdown::Content> },
    Tool(ToolState),
    /// Uma nota curta da IDE (a resposta parou no limite, o agente recusou).
    Note(String),
    /// A sessão recomeçou daqui para baixo.
    Restarted,
}

#[derive(Debug, Clone)]
struct Pending {
    title: String,
    tool: ToolInfo,
    command: Option<String>,
    url: Option<String>,
    detail: String,
    options: Vec<ShownOption>,
    show_detail: bool,
}

struct Session {
    id: SessionId,
    label: String,
    target: Target,
    generation: u32,
    /// `None` quando o processo acabou: a aba fica com a conversa até ser fechada ou recomeçar.
    key: Option<SessionKey>,
    cmd: Option<mpsc::UnboundedSender<Command>>,
    program: PathBuf,
    /// Nome e versão do agente (`agentInfo`).
    agent: Option<String>,
    version: Option<String>,
    lines: Vec<Line>,
    queue: Vec<Queued>,
    input: text_editor::Content,
    chips: Vec<Chip>,
    history: Vec<String>,
    pending: Option<Pending>,
    logins: Vec<LoginMethod>,
    notice: Option<String>,
    /// Um login rodou no terminal: o Recomeçar fica em destaque.
    logged_in: bool,
    /// Um turno está aberto (do envio até o fim do turno).
    busy: bool,
    /// Algo pede o usuário (permissão) numa aba que não está à frente.
    attention: bool,
    /// O último "trabalhando" contado ao mascote, para não repetir a cada pedaço de texto.
    doing: Option<String>,
    /// O mesmo estado, em texto, para a linha que aparece com o mascote desligado.
    status: Option<String>,
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
            self.status = Some(what.clone());
            ctx.agent_cue(self.label.clone(), AgentCue::Busy(Some(what)));
        }
    }

    /// Um marco que não é "trabalhando": o próximo "trabalhando" é contado de novo.
    fn cue(&mut self, cue: AgentCue, ctx: &mut Context) {
        self.doing = None;
        self.status = cue_text(&cue);
        ctx.agent_cue(self.label.clone(), cue);
    }

    /// Manda a primeira da fila, se não há turno aberto e a sessão está ligada.
    fn dequeue(&mut self, ctx: &mut Context) -> bool {
        if self.busy || self.cmd.is_none() || self.queue.is_empty() {
            return false;
        }
        let next = self.queue.remove(0);
        self.lines.push(Line::You { text: next.text, files: next.files });
        self.busy = true;
        self.send(Command::Prompt(next.parts));
        self.busy_with(fl!("acp-doing-thinking"), ctx);
        true
    }
}

fn cue_text(cue: &AgentCue) -> Option<String> {
    Some(match cue {
        AgentCue::Ready => fl!("acp-state-ready"),
        AgentCue::Busy(Some(what)) => what.clone(),
        AgentCue::Busy(None) => return None,
        AgentCue::Permission => fl!("acp-state-permission"),
        AgentCue::Done => fl!("acp-state-done"),
        AgentCue::Refused => fl!("acp-state-refused"),
        AgentCue::Failed => fl!("acp-state-failed"),
        AgentCue::Ended => fl!("acp-state-ended"),
    })
}

pub struct AcpExtension {
    root: PathBuf,
    next_id: SessionId,
    sessions: Vec<Session>,
    active: Option<SessionId>,
    /// Aviso fora de qualquer sessão (binário ausente, Node velho...).
    notice: Option<String>,
    /// Você subiu na conversa para ler: o texto novo não puxa a rolagem.
    reading_back: bool,
    theme: Theme,
    font_size: f32,
    /// `[mascot] enabled`: desligado, o estado aparece na aba.
    mascot: bool,
    /// Arquivos escolhidos sem sessão aberta: vão para a próxima que abrir.
    staged: Vec<Chip>,
    /// Pediu o contexto do editor e espera o [`CoreEvent::EditorContext`].
    asking: bool,
    spinner: usize,
}

impl Default for AcpExtension {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            next_id: 0,
            sessions: Vec::new(),
            active: None,
            notice: None,
            reading_back: false,
            theme: Theme::Dark,
            font_size: 13.0,
            mascot: true,
            staged: Vec::new(),
            asking: false,
            spinner: 0,
        }
    }
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
        let base = target_label(target);
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
            target,
            generation: 0,
            key: Some(SessionKey { id, generation: 0, cwd: self.root.clone(), program: launch.program.clone(), args: launch.args }),
            cmd: None,
            program: launch.program,
            agent: None,
            version: None,
            lines: Vec::new(),
            queue: Vec::new(),
            input: text_editor::Content::new(),
            chips: std::mem::take(&mut self.staged),
            history: Vec::new(),
            pending: None,
            logins: Vec::new(),
            notice: None,
            logged_in: false,
            busy: false,
            attention: false,
            doing: None,
            status: None,
        });
        self.active = Some(id);
        self.reading_back = false;
        self.notice = None;
        ctx.show_panel(PANEL);
        id
    }

    fn launch(&mut self, target: Target, ctx: &mut Context) -> Task<Message> {
        match resolve(target) {
            Ok(launch) => {
                self.start(target, launch, ctx);
                focus_input()
            }
            Err(missing) => {
                self.notice = Some(missing_text(&missing));
                ctx.show_panel(PANEL);
                Task::none()
            }
        }
    }

    /// A mesma aba, o mesmo agente, um processo novo. A conversa anterior fica acima de um separador.
    fn restart(&mut self, id: SessionId, ctx: &mut Context) -> Task<Message> {
        let root = self.root.clone();
        let Some(s) = self.session_mut(id) else { return Task::none() };
        if s.alive() {
            return Task::none();
        }
        match resolve(s.target) {
            Ok(launch) => {
                s.generation += 1;
                s.key = Some(SessionKey { id, generation: s.generation, cwd: root, program: launch.program.clone(), args: launch.args });
                s.program = launch.program;
                s.cmd = None;
                s.lines.push(Line::Restarted);
                s.notice = None;
                s.logins.clear();
                s.logged_in = false;
                s.pending = None;
                s.busy = false;
                s.status = None;
                s.doing = None;
                self.reading_back = false;
                Task::batch([follow(), focus_input()])
            }
            Err(missing) => {
                s.notice = Some(missing_text(&missing));
                let _ = ctx;
                Task::none()
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
        let root = self.root.clone();
        let Some(s) = self.session_mut(id) else { return Task::none() };
        let mut grew = false;
        match event {
            SessionEvent::Ready { agent, version } => {
                s.agent = agent;
                s.version = version;
                s.cue(AgentCue::Ready, ctx);
                grew = s.dequeue(ctx);
            }
            SessionEvent::Assistant { text } => {
                grew = append_text(&mut s.lines, &text);
                if grew {
                    s.busy_with(fl!("acp-doing-writing"), ctx);
                }
            }
            SessionEvent::Thought { text } => {
                if !text.is_empty() {
                    s.busy_with(fl!("acp-doing-thinking"), ctx);
                }
            }
            SessionEvent::Tool(info) => {
                let tool = info.id.clone();
                grew = upsert_tool(&mut s.lines, info);
                let running = s.lines.iter().rev().find_map(|l| match l {
                    Line::Tool(t) if t.id == tool && t.running() && !t.title.is_empty() => Some(t.title.clone()),
                    _ => None,
                });
                if let Some(title) = running {
                    s.busy_with(title, ctx);
                }
            }
            SessionEvent::Permission { title, tool, command, url, detail, options } => {
                let pending = Pending { title, tool, command, url, detail, options, show_detail: false };
                ctx.show_dialog(permission_dialog(id, &s.label, &pending, &root));
                s.pending = Some(pending);
                s.attention = !front;
                grew = true;
                s.cue(AgentCue::Permission, ctx);
            }
            // A fila é do painel: ele só manda uma mensagem por turno.
            SessionEvent::Sent => {}
            SessionEvent::Turn { stop } => {
                s.busy = false;
                s.pending = None;
                let note = match stop.as_str() {
                    "max_tokens" => Some(fl!("acp-stop-max-tokens")),
                    "max_turn_requests" => Some(fl!("acp-stop-max-turns")),
                    "refusal" => Some(fl!("acp-stop-refusal")),
                    _ => None,
                };
                if let Some(note) = note {
                    s.lines.push(Line::Note(note));
                    grew = true;
                }
                let cue = match stop.as_str() {
                    "end_turn" | "max_tokens" | "max_turn_requests" => AgentCue::Done,
                    "refusal" => AgentCue::Refused,
                    "error" => AgentCue::Failed,
                    _ => AgentCue::Busy(None),
                };
                s.cue(cue, ctx);
                grew |= s.dequeue(ctx);
            }
            SessionEvent::AuthRequired { message, logins } => {
                s.busy = false;
                s.cmd = None;
                s.logins = logins;
                s.notice = Some(fl!("acp-auth", message = message));
                s.key = None;
                s.cue(AgentCue::Busy(None), ctx);
            }
            SessionEvent::Failed(failure) => {
                s.busy = false;
                s.notice = Some(match failure {
                    Failure::Message(message) => fl!("acp-failed", error = message),
                    Failure::Panic => fl!("acp-failed-panic"),
                });
                s.cue(AgentCue::Failed, ctx);
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

    /// Manda `session/cancel`. O turno só fecha quando o agente responder (`cancelled`): até lá a
    /// próxima da fila espera, para não haver dois turnos ao mesmo tempo.
    fn cancel(&mut self, id: SessionId, ctx: &mut Context) {
        if let Some(s) = self.session_mut(id) {
            s.pending = None;
            s.attention = false;
            s.send(Command::Cancel);
            if s.alive() {
                s.cue(AgentCue::Busy(None), ctx);
            }
        }
    }

    /// Enviar: com um turno aberto (ou a sessão ainda ligando), a mensagem vai para a fila.
    fn submit(&mut self, ctx: &mut Context) -> Task<Message> {
        let root = self.root.clone();
        let Some(s) = self.active_mut() else { return Task::none() };
        if !s.alive() {
            return Task::none();
        }
        let typed = s.input.text().trim().to_string();
        if typed.is_empty() && s.chips.is_empty() {
            return Task::none();
        }
        let files: Vec<String> = s.chips.iter().map(|c| c.label(&root)).collect();
        let mut parts: Vec<PromptPart> = s.chips.drain(..).map(|c| c.part()).collect();
        if !typed.is_empty() {
            parts.insert(0, PromptPart::Text(typed.clone()));
            s.history.push(typed.clone());
        }
        s.input = text_editor::Content::new();
        s.queue.push(Queued { text: typed, files, parts });
        s.dequeue(ctx);
        self.reading_back = false;
        follow()
    }

    /// Junta um arquivo à próxima mensagem da aba à frente (ou da próxima sessão, sem nenhuma).
    fn add_chip(&mut self, chip: Chip, ctx: &mut Context) -> Task<Message> {
        ctx.show_panel(PANEL);
        match self.active_mut().filter(|s| s.alive()) {
            Some(s) => {
                if !s.chips.contains(&chip) {
                    s.chips.push(chip);
                }
                focus_input()
            }
            None => {
                if !self.staged.contains(&chip) {
                    self.staged.push(chip);
                }
                self.notice = Some(fl!("acp-ask-staged"));
                Task::none()
            }
        }
    }

    fn view_tabs(&self) -> Element<'_, Message> {
        let mut tabs = row![].spacing(4);
        for s in &self.sessions {
            let front = self.active == Some(s.id);
            let mut label = s.label.clone();
            if s.attention {
                label = format!("● {label}");
            } else if s.busy && !front {
                label = format!("{} {label}", SPINNER[self.spinner % SPINNER.len()]);
            } else if !s.alive() {
                label = fl!("acp-tab-ended", name = label);
            }
            let style = if front { button::primary } else { button::secondary };
            let close = button(icons::icon(Icon::Close).size(12)).style(button::text).padding(2).on_press(Message::Close(s.id));
            let tab = button(text(label).size(12)).style(style).padding([3, 8]).on_press(Message::Select(s.id));
            let tab: Element<'_, Message> = match (&s.agent, &s.version) {
                (Some(agent), version) => {
                    let tip = version.as_ref().map_or_else(|| agent.clone(), |v| format!("{agent} {v}"));
                    tooltip(tab, container(text(tip).size(12)).padding([3, 8]).style(container::bordered_box), tooltip::Position::Bottom).into()
                }
                _ => tab.into(),
            };
            tabs = tabs.push(row![tab, close].spacing(0).align_y(iced::Alignment::Center));
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
            body = body.push(text(fl!("acp-empty")).size(13).width(Fill));
            if !self.staged.is_empty() {
                body = body.push(chips_view(&self.staged, &self.root, false));
            }
            body = body.push(
                row![
                    button(text(fl!("acp-opencode")).size(13)).on_press(Message::Start(Target::OpenCode)),
                    button(text(fl!("acp-claude")).size(13)).style(button::secondary).on_press(Message::Start(Target::Claude)),
                ]
                .spacing(8),
            );
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
        if !s.alive() {
            let style = if s.logged_in { button::primary } else { button::secondary };
            body = body.push(button(text(fl!("acp-restart")).size(13)).style(style).on_press(Message::Restart(s.id)));
        }
        let mut transcript = column![].spacing(8).width(Fill);
        for (i, line) in s.lines.iter().enumerate() {
            transcript = transcript.push(self.line_view(s.id, i, line));
        }
        for (i, queued) in s.queue.iter().enumerate() {
            transcript = transcript.push(queued_view(s.id, i, queued));
        }
        let scroll = tea_core::style::scrollable(transcript)
            .id(TRANSCRIPT)
            .on_scroll(|v| Message::Scrolled(v.absolute_offset_reversed().y <= FOLLOW_SLACK))
            .height(Fill);
        let mut bottom = column![].spacing(6).width(Fill);
        if let Some(pending) = &s.pending {
            bottom = bottom.push(permission_card(s.id, pending, &self.root));
        }
        if !self.mascot
            && let Some(status) = &s.status
        {
            bottom = bottom.push(text(status.clone()).size(12).style(text::secondary));
        }
        if !s.chips.is_empty() {
            bottom = bottom.push(chips_view(&s.chips, &self.root, true));
        }
        let can_send = s.alive() && (!s.input.text().trim().is_empty() || !s.chips.is_empty());
        let send = button(text(fl!("acp-send")).size(13)).on_press_maybe(can_send.then_some(Message::Send));
        let input = text_editor(&s.input)
            .id(INPUT)
            .placeholder(fl!("acp-prompt"))
            .on_action(Message::Input)
            .key_binding(input_keys)
            .size(13)
            .min_height(32.0)
            .max_height(8.0 * 13.0 * 1.45 + 12.0);
        let attach = icons::icon_button(Icon::File, fl!("acp-attach"), s.alive().then_some(Message::Attach));
        let mut actions = row![input, column![send, attach].spacing(4).align_x(iced::Alignment::Center)].spacing(8);
        if s.busy {
            actions = actions.push(button(text(fl!("acp-cancel")).size(13)).style(button::secondary).on_press(Message::Cancel(s.id)));
        }
        bottom = bottom.push(actions).push(text(fl!("acp-prompt-hint")).size(11).style(text::secondary));
        column![body, scroll, container(bottom).padding([0, 12])].spacing(8).padding(iced::Padding { bottom: 12.0, ..iced::Padding::ZERO }).height(Fill).into()
    }

    fn markdown_settings(&self) -> markdown::Settings {
        let mut settings = markdown::Settings::with_text_size(self.font_size, &self.theme);
        settings.code_size = (self.font_size * 0.9).into();
        settings
    }

    fn line_view<'a>(&'a self, sid: SessionId, index: usize, line: &'a Line) -> Element<'a, Message> {
        match line {
            Line::You { text: said, files } => {
                let mut col = column![text(fl!("acp-you")).size(11).style(text::secondary)].spacing(2);
                if !said.is_empty() {
                    col = col.push(text(said.clone()).size(13).width(Fill));
                }
                if !files.is_empty() {
                    col = col.push(text(files.join("  ·  ")).size(11).style(text::secondary).font(Font::MONOSPACE));
                }
                container(col).padding([6, 10]).width(Fill).style(container::rounded_box).into()
            }
            Line::Assistant { text: raw, md } => {
                let body = markdown::view_with(md.items(), self.markdown_settings(), &Viewer);
                let copy = button(text(fl!("acp-copy-answer")).size(11)).style(button::text).padding([0, 4]).on_press(Message::Copy(raw.clone()));
                column![body, copy].spacing(2).width(Fill).into()
            }
            Line::Tool(tool) => tool_view(sid, index, tool, &self.root),
            Line::Note(note) => text(note.clone()).size(12).style(text::warning).width(Fill).into(),
            Line::Restarted => text(fl!("acp-restarted")).size(11).style(text::secondary).width(Fill).center().into(),
        }
    }
}

fn target_label(target: Target) -> String {
    match target {
        Target::OpenCode => fl!("acp-opencode"),
        Target::Claude => fl!("acp-claude"),
    }
}

fn resolve(target: Target) -> Result<Launch, Missing> {
    target::system_probe(claude_bin().as_deref()).resolve(target)
}

/// Enter envia; Shift+Enter quebra a linha; ↑ na caixa vazia traz a última mensagem.
fn input_keys(key: text_editor::KeyPress) -> Option<text_editor::Binding<Message>> {
    if !matches!(key.status, text_editor::Status::Focused { .. }) {
        return text_editor::Binding::from_key_press(key);
    }
    match key.key.as_ref() {
        keyboard::Key::Named(Named::Enter) if !key.modifiers.shift() && !key.modifiers.control() && !key.modifiers.alt() => {
            Some(text_editor::Binding::Custom(Message::Send))
        }
        // Vazia, o histórico; com texto, o cursor sobe (quem decide é o `HistoryUp`).
        keyboard::Key::Named(Named::ArrowUp) if key.modifiers.is_empty() => Some(text_editor::Binding::Custom(Message::HistoryUp)),
        _ => text_editor::Binding::from_key_press(key),
    }
}

/// Rola a conversa até o fim.
fn follow() -> Task<Message> {
    iced::widget::operation::snap_to_end(TRANSCRIPT)
}

fn focus_input() -> Task<Message> {
    iced::widget::operation::focus(INPUT)
}

/// Junta o pedaço de resposta ao último bloco dela. `true` se a conversa cresceu.
fn append_text(lines: &mut Vec<Line>, chunk: &str) -> bool {
    if chunk.is_empty() {
        return false;
    }
    match lines.last_mut() {
        Some(Line::Assistant { text, md }) => {
            text.push_str(chunk);
            md.push_str(chunk);
        }
        _ => lines.push(Line::Assistant { text: chunk.to_string(), md: Box::new(markdown::Content::parse(chunk)) }),
    }
    true
}

/// Uma linha por ferramenta: a atualização muda a linha que já existe e só os campos que vieram.
/// `true` se apareceu uma linha nova.
fn upsert_tool(lines: &mut Vec<Line>, info: ToolInfo) -> bool {
    let existing = lines.iter_mut().rev().find_map(|l| match l {
        Line::Tool(t) if t.id == info.id => Some(t),
        _ => None,
    });
    if let Some(tool) = existing {
        tool.merge(info);
        return false;
    }
    let mut tool = ToolState { id: info.id.clone(), ..Default::default() };
    tool.merge(info);
    lines.push(Line::Tool(tool));
    true
}

fn kind_icon(kind: &str) -> Icon {
    match kind {
        "read" => Icon::File,
        "edit" => Icon::Edit,
        "delete" => Icon::Trash,
        "move" => Icon::GoToFile,
        "search" => Icon::Search,
        "execute" => Icon::Terminal,
        "think" => Icon::CommentDiscussion,
        "fetch" => Icon::LinkExternal,
        _ => Icon::Info,
    }
}

fn shown_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn location_link<'a>(location: &ToolLocation, root: &Path) -> Element<'a, Message> {
    let label = match location.line {
        Some(line) => format!("{}:{line}", shown_path(&location.path, root)),
        None => shown_path(&location.path, root),
    };
    button(text(label).size(11).font(Font::MONOSPACE)).style(button::text).padding([0, 2]).on_press(Message::Open(location.path.clone(), location.line)).into()
}

fn tool_view<'a>(sid: SessionId, index: usize, tool: &'a ToolState, root: &Path) -> Element<'a, Message> {
    let chevron = if tool.open { Icon::ChevronDown } else { Icon::ChevronRight };
    let expandable = !tool.content.is_empty();
    let title = if tool.title.is_empty() { tool.id.clone() } else { tool.title.clone() };
    let head = row![
        icons::icon(if expandable { chevron } else { kind_icon(&tool.kind) }).size(12),
        icons::icon(kind_icon(&tool.kind)).size(12),
        text(title).size(12).style(text::secondary).width(Fill),
        text(status_text(&tool.status)).size(11).style(status_style(&tool.status)),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);
    let head: Element<'_, Message> = if expandable {
        button(head).style(button::text).padding(0).width(Fill).on_press(Message::ToggleTool(sid, index)).into()
    } else {
        head.into()
    };
    let mut col = column![head].spacing(4).width(Fill);
    if !tool.locations.is_empty() {
        let mut links = row![].spacing(6);
        for location in &tool.locations {
            links = links.push(location_link(location, root));
        }
        col = col.push(container(links).padding([0, 18]));
    }
    if tool.open {
        for content in &tool.content {
            col = col.push(container(content_view(content, root)).padding(iced::Padding { left: 18.0, ..iced::Padding::ZERO }));
        }
    }
    col.into()
}

fn content_view<'a>(content: &ToolContent, root: &Path) -> Element<'a, Message> {
    match content {
        ToolContent::Text(output) => code_box(clip_lines(output, TOOL_TEXT_LINES)),
        ToolContent::Diff { path, old, new } => diff::view(&shown_path(path, root), path, old.as_deref(), new),
        ToolContent::Terminal(id) => text(fl!("acp-terminal", id = id.clone())).size(11).style(text::secondary).into(),
    }
}

fn clip_lines(textual: &str, max: usize) -> String {
    let total = textual.lines().count();
    if total <= max {
        return textual.trim_end().to_string();
    }
    let mut kept: String = textual.lines().take(max).collect::<Vec<_>>().join("\n");
    kept.push('\n');
    let more: usize = total - max;
    kept.push_str(&fl!("acp-more-lines", count = more));
    kept
}

/// Texto em fonte de código, num quadro, com Copiar.
fn code_box<'a>(code: String) -> Element<'a, Message> {
    let copy = button(text(fl!("acp-copy")).size(11)).padding([2, 8]).style(button::secondary).on_press(Message::Copy(code.clone()));
    let block = container(text(code).size(12).font(Font::MONOSPACE).width(Fill)).padding(8).width(Fill).style(container::bordered_box);
    stack![block, container(copy).width(Fill).align_right(Fill).padding(4)].into()
}

fn queued_view<'a>(sid: SessionId, index: usize, queued: &'a Queued) -> Element<'a, Message> {
    let mut label = queued.text.clone();
    if !queued.files.is_empty() {
        label = if label.is_empty() { queued.files.join("  ·  ") } else { format!("{label}  ·  {}", queued.files.join("  ·  ")) };
    }
    let remove = icons::icon_button(Icon::Close, fl!("acp-unqueue"), Some(Message::Unqueue(sid, index)));
    container(
        row![
            column![text(fl!("acp-queued")).size(11).style(text::secondary), text(label).size(13).style(text::secondary).width(Fill)].spacing(2),
            remove,
        ]
        .spacing(6),
    )
    .padding([6, 10])
    .width(Fill)
    .style(container::bordered_box)
    .into()
}

fn chips_view<'a>(chips: &[Chip], root: &Path, removable: bool) -> Element<'a, Message> {
    let mut list = row![].spacing(6);
    for (i, chip) in chips.iter().enumerate() {
        let mut chip_row = row![icons::icon(Icon::File).size(11), text(chip.label(root)).size(11).font(Font::MONOSPACE)].spacing(4).align_y(iced::Alignment::Center);
        if removable {
            chip_row = chip_row.push(button(icons::icon(Icon::Close).size(10)).style(button::text).padding(1).on_press(Message::RemoveChip(i)));
        }
        list = list.push(container(chip_row).padding([2, 6]).style(container::rounded_box));
    }
    tea_core::style::scrollable(list).direction(iced::widget::scrollable::Direction::Horizontal(Default::default())).width(Fill).into()
}

fn permission_card<'a>(id: SessionId, pending: &'a Pending, root: &Path) -> Element<'a, Message> {
    let mut card = column![text(fl!("acp-permission-title")).size(13), text(pending.title.clone()).size(13).width(Fill)].spacing(6).padding(10).width(Fill);
    let mut shown = false;
    if let Some(command) = &pending.command {
        card = card.push(code_box(command.clone()));
        shown = true;
    }
    if let Some(url) = &pending.url {
        card = card.push(text(url.clone()).size(12).font(Font::MONOSPACE));
        shown = true;
    }
    for content in pending.tool.content.iter().flatten() {
        if matches!(content, ToolContent::Diff { .. }) {
            card = card.push(content_view(content, root));
            shown = true;
        }
    }
    if let Some(locations) = pending.tool.locations.as_ref().filter(|l| !l.is_empty()) {
        let mut links = row![].spacing(6);
        for location in locations {
            links = links.push(location_link(location, root));
        }
        card = card.push(links);
        shown = true;
    }
    if !pending.detail.is_empty() {
        // O JSON todo, recolhido quando já há algo legível acima.
        if pending.show_detail || !shown {
            card = card.push(code_box(pending.detail.clone()));
        }
        if shown {
            let label = if pending.show_detail { fl!("acp-hide-details") } else { fl!("acp-details") };
            card = card.push(button(text(label).size(11)).style(button::text).padding([0, 2]).on_press(Message::ToggleDetail(id)));
        }
    }
    card = card.push(text(fl!("acp-permission-note")).size(12).style(text::warning).width(Fill));
    let mut buttons = row![].spacing(6);
    for (i, option) in pending.options.iter().enumerate() {
        let index = u32::try_from(i).unwrap_or(u32::MAX);
        buttons = buttons.push(button(text(option.name.clone()).size(12)).style(option_style(option.kind)).on_press(Message::Choose(id, index)));
    }
    buttons = buttons.push(button(text(fl!("acp-cancel")).size(12)).style(button::secondary).on_press(Message::Cancel(id)));
    card = card.push(buttons);
    container(card).style(container::bordered_box).width(Fill).into()
}

fn option_style(kind: Kind) -> fn(&iced::Theme, button::Status) -> button::Style {
    if kind.is_allow() { button::primary } else { button::secondary }
}

/// O que o diálogo diz da ação, em texto: o comando, a URL, os arquivos que mudam ou são lidos.
fn permission_summary(pending: &Pending, root: &Path) -> String {
    let mut lines = Vec::new();
    if let Some(command) = &pending.command {
        lines.push(format!("$ {command}"));
    }
    if let Some(url) = &pending.url {
        lines.push(url.clone());
    }
    for content in pending.tool.content.iter().flatten() {
        if let ToolContent::Diff { path, .. } = content {
            lines.push(fl!("acp-permission-edit", path = shown_path(path, root)));
        }
    }
    if lines.is_empty()
        && let Some(locations) = &pending.tool.locations
    {
        lines.extend(locations.iter().map(|l| shown_path(&l.path, root)));
    }
    if lines.is_empty() && !pending.detail.is_empty() {
        lines.push(pending.detail.clone());
    }
    lines.join("\n")
}

fn permission_dialog(id: SessionId, session: &str, pending: &Pending, root: &Path) -> Dialog {
    let from = fl!("acp-permission-session", session = session.to_string());
    let note = fl!("acp-permission-note");
    let summary = permission_summary(pending, root);
    let title = &pending.title;
    let message = if summary.is_empty() { format!("{from}\n\n{title}\n\n{note}") } else { format!("{from}\n\n{title}\n{summary}\n\n{note}") };
    // Todos secundários: Enter e Esc fecham sem escolher. Permitir é um clique.
    let mut dialog = Dialog::new(fl!("acp-permission-title"), message);
    for (i, option) in pending.options.iter().enumerate() {
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
        "" => String::new(),
        other => other.to_string(),
    }
}

fn status_style(status: &str) -> fn(&Theme) -> text::Style {
    match status {
        "failed" => text::danger,
        "completed" => text::success,
        _ => text::secondary,
    }
}

/// Markdown do agente: imagens viram o texto alternativo (nada é carregado), e blocos de código
/// ganham Copiar.
struct Viewer;

impl<'a> markdown::Viewer<'a, Message> for Viewer {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::Link(url)
    }

    fn image(&self, settings: markdown::Settings, _url: &'a markdown::Uri, _title: &'a str, alt: &markdown::Text) -> Element<'a, Message> {
        let alt: String = alt.spans(settings.style).iter().map(|s| s.text.as_ref()).collect();
        container(text(alt).size(12)).padding(6).style(container::bordered_box).into()
    }

    fn code_block(&self, settings: markdown::Settings, _language: Option<&'a str>, code: &'a str, lines: &'a [markdown::Text]) -> Element<'a, Message> {
        let block = markdown::code_block(settings, lines, Message::Link);
        let copy = button(text(fl!("acp-copy")).size(11)).padding([2, 8]).style(button::secondary).on_press(Message::Copy(code.trim_end().to_string()));
        stack![block, container(copy).width(Fill).align_right(Fill).padding(4)].into()
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

fn mascot_enabled(root: &Path) -> bool {
    let global = tea_core::config::global_config_path();
    tea_core::config::Config::load(global.as_deref(), Some(&tea_core::config::project_config_path(root))).mascot.enabled
}

/// Os arquivos do projeto, respeitando o `.gitignore`, para o seletor de `@`.
fn list_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = ignore::WalkBuilder::new(root)
        .hidden(true)
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .map(|e| e.into_path())
        .take(PICK_FILES)
        .collect();
    files.sort();
    files
}

fn watch(key: &SessionKey) -> iced_mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced_mpsc::unbounded();
    let (cmd_tx, cmd_rx) = mpsc::unbounded();
    let (id, generation) = (key.id, key.generation);
    let launch = Launch {
        target: if key.args.first().is_some_and(|a| a == "acp") { Target::OpenCode } else { Target::Claude },
        program: key.program.clone(),
        args: key.args.clone(),
    };
    let cwd = key.cwd.clone();
    std::thread::spawn(move || {
        if tx.unbounded_send(Message::Bound(id, generation, cmd_tx)).is_err() {
            return;
        }
        tea_acp::session::run(&launch, &cwd, cmd_rx, move |event| {
            let _ = tx.unbounded_send(Message::Event(id, generation, Box::new(event)));
        });
    });
    rx
}

/// O relógio do ponto que gira nas abas de trás, só enquanto alguma trabalha.
fn ticks(_: &&'static str) -> iced_mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = iced_mpsc::unbounded();
    std::thread::spawn(move || {
        while tx.unbounded_send(Message::Tick).is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
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
        self.mascot = mascot_enabled(&self.root);
        ctx.register_panel_hidden(PANEL, fl!("acp-panel"), Slot::Right);
        ctx.panel_icon(PANEL, Icon::Hubot);
        ctx.register_command("acp.focus", fl!("cmd-acp-focus"));
        ctx.register_command("acp.opencode", fl!("cmd-acp-opencode"));
        ctx.register_command("acp.claude", fl!("cmd-acp-claude"));
        ctx.register_command("acp.cancel", fl!("cmd-acp-cancel"));
        ctx.register_command("acp.ask", fl!("cmd-acp-ask"));
        ctx.register_command("acp.attach", fl!("cmd-acp-attach"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::Appearance { theme, font_size, .. } => {
                self.theme = theme.clone();
                self.font_size = *font_size;
            }
            CoreEvent::ConfigReloaded => self.mascot = mascot_enabled(&self.root),
            CoreEvent::EditorContext { path, lines, text } if self.asking => {
                self.asking = false;
                let Some(path) = path.clone() else {
                    ctx.notify(fl!("acp-ask-nothing"));
                    return Task::none();
                };
                return self.add_chip(Chip { path, lines: *lines, text: text.clone() }, ctx);
            }
            _ => {}
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "acp.focus" => {
                ctx.show_panel(PANEL);
                return focus_input();
            }
            "acp.opencode" => return self.launch(Target::OpenCode, ctx),
            "acp.claude" => return self.launch(Target::Claude, ctx),
            "acp.cancel" => {
                if let Some(id) = self.active {
                    self.cancel(id, ctx);
                }
            }
            "acp.ask" => {
                self.asking = true;
                ctx.request_editor_context();
            }
            "acp.attach" => return self.update(Message::Attach, ctx),
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Start(target) => return self.launch(target, ctx),
            Message::Bound(id, generation, cmd) => {
                if let Some(s) = self.session_mut(id)
                    && s.generation == generation
                {
                    s.cmd = Some(cmd);
                }
            }
            Message::Event(id, generation, event) => {
                // Um evento atrasado do processo de antes de um Recomeçar não vale.
                if self.session(id).is_some_and(|s| s.generation == generation) {
                    return self.on_event_msg(id, *event, ctx);
                }
            }
            Message::Select(id) => return Task::batch([self.select(id), focus_input()]),
            Message::Close(id) => self.close(id, ctx),
            Message::Restart(id) => return self.restart(id, ctx),
            Message::Scrolled(at_end) => self.reading_back = !at_end,
            Message::Input(action) => {
                if let Some(s) = self.active_mut() {
                    // `@` no começo ou depois de um espaço abre o seletor de arquivos (não entra no texto).
                    if let text_editor::Action::Edit(text_editor::Edit::Insert('@')) = &action
                        && s.alive()
                        && at_word_start(&s.input)
                    {
                        return self.update(Message::Attach, ctx);
                    }
                    s.input.perform(action);
                }
            }
            Message::Send => return self.submit(ctx),
            Message::HistoryUp => {
                if let Some(s) = self.active_mut() {
                    match s.history.last() {
                        Some(last) if s.input.text().trim().is_empty() => {
                            s.input = text_editor::Content::with_text(last);
                            s.input.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
                        }
                        _ => s.input.perform(text_editor::Action::Move(text_editor::Motion::Up)),
                    }
                }
            }
            Message::Unqueue(id, index) => {
                if let Some(s) = self.session_mut(id)
                    && index < s.queue.len()
                {
                    s.queue.remove(index);
                }
            }
            Message::Cancel(id) => self.cancel(id, ctx),
            Message::Choose(id, index) => self.choose(id, index, ctx),
            Message::Login(id, index) => {
                let root = self.root.clone();
                if let Some(s) = self.session_mut(id)
                    && let Some(method) = s.logins.get(index)
                {
                    let command = target::login_command(&s.program, &method.args);
                    let done = fl!("acp-login-done", code = "{code}");
                    ctx.run_task(fl!("acp-login-task", name = method.name.clone()), command, root, done);
                    s.logged_in = true;
                }
            }
            Message::ToggleTool(id, index) => {
                if let Some(s) = self.session_mut(id)
                    && let Some(Line::Tool(tool)) = s.lines.get_mut(index)
                {
                    tool.open = !tool.open;
                }
            }
            Message::ToggleDetail(id) => {
                if let Some(p) = self.session_mut(id).and_then(|s| s.pending.as_mut()) {
                    p.show_detail = !p.show_detail;
                }
            }
            Message::Open(path, line) => match line {
                Some(line) => ctx.open_file_at(path, line as usize, 1),
                None => ctx.open_file(path),
            },
            Message::Copy(code) => return tea_declarative_ui::copy(code, ctx),
            Message::Link(url) => tea_declarative_ui::follow(url.as_str(), &self.root, ctx),
            Message::Attach => {
                let root = self.root.clone();
                let (tx, rx) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(list_files(&root));
                });
                return Task::future(async move { rx.await.ok() }).and_then(|files| Task::done(Message::FilesListed(Arc::new(files))));
            }
            Message::FilesListed(files) => {
                let mut pick = Pick::new(fl!("acp-pick-file"));
                for path in files.iter() {
                    pick = pick.item(Some(Icon::File), shown_path(path, &self.root), "", Message::FilePicked(path.clone()));
                }
                ctx.pick(pick);
            }
            Message::FilePicked(path) => return self.add_chip(Chip { path, lines: None, text: None }, ctx),
            Message::RemoveChip(index) => {
                if let Some(s) = self.active_mut()
                    && index < s.chips.len()
                {
                    s.chips.remove(index);
                }
            }
            Message::Tick => self.spinner = self.spinner.wrapping_add(1),
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        self.view_panel()
    }

    fn subscription(&self) -> Subscription<Message> {
        let sessions = self.sessions.iter().filter_map(|s| s.key.as_ref()).map(|key| Subscription::run_with(key.clone(), watch));
        let behind_busy = self.sessions.iter().any(|s| s.busy && self.active != Some(s.id));
        let spin = behind_busy.then(|| Subscription::run_with("acp-spinner", ticks));
        Subscription::batch(sessions.chain(spin))
    }
}

/// O cursor está no começo da caixa ou logo depois de um espaço (ali `@` é "anexar arquivo").
fn at_word_start(content: &text_editor::Content) -> bool {
    let position = content.cursor().position;
    let (line, column) = (position.line, position.column);
    if column == 0 {
        return true;
    }
    content.line(line).and_then(|l| l.text.chars().nth(column - 1)).is_some_and(char::is_whitespace)
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

    fn tool(id: &str, title: Option<&str>, status: Option<&str>) -> ToolInfo {
        ToolInfo { id: id.into(), title: title.map(Into::into), status: status.map(Into::into), ..Default::default() }
    }

    fn type_text(ext: &mut AcpExtension, said: &str, ctx: &mut Context) {
        for c in said.chars() {
            let _ = ext.update(Message::Input(text_editor::Action::Edit(text_editor::Edit::Insert(c))), ctx);
        }
    }

    fn prompts(rx: &mut mpsc::UnboundedReceiver<Command>) -> Vec<Vec<PromptPart>> {
        let mut out = Vec::new();
        while let Ok(command) = rx.try_recv() {
            if let Command::Prompt(parts) = command {
                out.push(parts);
            }
        }
        out
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
        let asked = SessionEvent::Permission { title: "echo oi".into(), tool: ToolInfo::default(), command: Some("echo oi".into()), url: None, detail: String::new(), options };
        let _ = ext.on_event_msg(ids[0], asked, &mut ctx);
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
        assert!(upsert_tool(&mut lines, ToolInfo { kind: Some("read".into()), ..tool("t1", Some("Read a.rs"), Some("pending")) }));
        assert!(!upsert_tool(&mut lines, tool("t1", None, Some("in_progress"))));
        assert!(!upsert_tool(&mut lines, ToolInfo { content: Some(vec![ToolContent::Text("ok".into())]), ..tool("t1", None, None) }));
        assert!(upsert_tool(&mut lines, tool("t2", Some("Edit b.rs"), Some("pending"))));
        assert!(!upsert_tool(&mut lines, tool("t1", None, Some("completed"))));
        assert_eq!(lines.len(), 2);
        let Line::Tool(first) = &lines[0] else { panic!() };
        assert_eq!((first.title.as_str(), first.status.as_str(), first.kind.as_str()), ("Read a.rs", "completed", "read"));
        assert_eq!(first.content, [ToolContent::Text("ok".into())], "uma atualização sem conteúdo não apaga o que veio");
    }

    #[test]
    fn state_goes_to_the_mascot_and_text_to_the_tab() {
        let (mut ext, ids) = ext_with(&[Target::Claude]);
        let mut ctx = Context::new(Path::new("/tmp"), None);
        let id = ids[0];
        for event in [
            SessionEvent::Ready { agent: Some("Claude".into()), version: Some("1.0".into()) },
            SessionEvent::Thought { text: "hmm".into() },
            SessionEvent::Thought { text: " more".into() },
            SessionEvent::Assistant { text: "Oi".into() },
            SessionEvent::Assistant { text: "!".into() },
            SessionEvent::Tool(tool("t", Some("Read a.rs"), Some("in_progress"))),
            SessionEvent::Tool(tool("t", None, Some("completed"))),
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
        assert!(matches!(&s.lines[0], Line::Assistant { text, .. } if text == "Oi!"));
        assert_eq!(s.agent.as_deref(), Some("Claude"));
    }

    #[test]
    fn a_message_sent_during_a_turn_waits_in_the_queue() {
        let (mut ext, ids) = ext_with(&[Target::Claude]);
        let id = ids[0];
        let (tx, mut rx) = mpsc::unbounded();
        ext.session_mut(id).unwrap().cmd = Some(tx);
        let mut ctx = Context::new(Path::new("/tmp"), None);
        type_text(&mut ext, "um", &mut ctx);
        let _ = ext.update(Message::Send, &mut ctx);
        type_text(&mut ext, "dois", &mut ctx);
        let _ = ext.update(Message::Send, &mut ctx);
        type_text(&mut ext, "três", &mut ctx);
        let _ = ext.update(Message::Send, &mut ctx);
        assert_eq!(prompts(&mut rx), [PromptPart::text("um")], "só a primeira saiu");
        assert_eq!(ext.session(id).unwrap().queue.len(), 2);
        // Tirar uma da fila; cancelar não manda a próxima antes do fim do turno.
        let _ = ext.update(Message::Unqueue(id, 1), &mut ctx);
        ext.cancel(id, &mut ctx);
        assert!(ext.session(id).unwrap().busy, "o turno só fecha quando o agente responde");
        assert!(prompts(&mut rx).is_empty());
        let _ = ext.on_event_msg(id, SessionEvent::Turn { stop: "cancelled".into() }, &mut ctx);
        assert_eq!(prompts(&mut rx), [PromptPart::text("dois")]);
        assert!(ext.session(id).unwrap().queue.is_empty());
        let you: Vec<_> = ext.session(id).unwrap().lines.iter().filter_map(|l| if let Line::You { text, .. } = l { Some(text.as_str()) } else { None }).collect();
        assert_eq!(you, ["um", "dois"]);
    }

    #[test]
    fn a_message_before_the_session_is_bound_goes_on_ready() {
        let (mut ext, ids) = ext_with(&[Target::OpenCode]);
        let id = ids[0];
        let mut ctx = Context::new(Path::new("/tmp"), None);
        type_text(&mut ext, "cedo", &mut ctx);
        let _ = ext.update(Message::Send, &mut ctx);
        assert_eq!(ext.session(id).unwrap().queue.len(), 1);
        let (tx, mut rx) = mpsc::unbounded();
        let _ = ext.update(Message::Bound(id, 0, tx), &mut ctx);
        let _ = ext.on_event_msg(id, SessionEvent::Ready { agent: None, version: None }, &mut ctx);
        assert_eq!(prompts(&mut rx), [PromptPart::text("cedo")]);
    }

    #[test]
    fn restart_reopens_the_same_tab_and_ignores_the_old_process() {
        let (mut ext, ids) = ext_with(&[Target::OpenCode]);
        let id = ids[0];
        let mut ctx = Context::new(Path::new("/tmp"), None);
        let _ = ext.on_event_msg(id, SessionEvent::Assistant { text: "antes".into() }, &mut ctx);
        let _ = ext.on_event_msg(id, SessionEvent::Closed, &mut ctx);
        assert!(!ext.session(id).unwrap().alive());
        // Sem o opencode no PATH do teste, o Recomeçar explica e não abre nada.
        let _ = ext.update(Message::Restart(id), &mut ctx);
        let s = ext.session_mut(id).unwrap();
        if s.alive() {
            assert_eq!(s.generation, 1);
            assert!(matches!(s.lines.last(), Some(Line::Restarted)));
        } else {
            assert!(s.notice.is_some());
            // Simula o recomeço para conferir a geração.
            s.generation = 1;
            s.key = Some(SessionKey { id, generation: 1, cwd: PathBuf::from("/tmp"), program: PathBuf::from("agente"), args: vec!["acp".into()] });
        }
        let _ = ext.update(Message::Event(id, 0, Box::new(SessionEvent::Assistant { text: "velho".into() })), &mut ctx);
        let s = ext.session(id).unwrap();
        assert!(!s.lines.iter().any(|l| matches!(l, Line::Assistant { text, .. } if text.contains("velho"))), "o processo de antes não escreve na aba");
    }

    #[test]
    fn stop_reasons_that_cut_the_answer_leave_a_note() {
        let (mut ext, ids) = ext_with(&[Target::Claude]);
        let mut ctx = Context::new(Path::new("/tmp"), None);
        let _ = ext.on_event_msg(ids[0], SessionEvent::Turn { stop: "max_tokens".into() }, &mut ctx);
        let _ = ext.on_event_msg(ids[0], SessionEvent::Turn { stop: "end_turn".into() }, &mut ctx);
        let notes = ext.session(ids[0]).unwrap().lines.iter().filter(|l| matches!(l, Line::Note(_))).count();
        assert_eq!(notes, 1);
    }

    #[test]
    fn chips_go_with_the_message_and_without_a_session_wait_for_one() {
        let mut ext = AcpExtension::new();
        ext.root = PathBuf::from("/proj");
        let mut ctx = Context::new(Path::new("/proj"), None);
        ext.asking = true;
        let event = CoreEvent::EditorContext { path: Some(PathBuf::from("/proj/src/a.rs")), lines: Some((3, 4)), text: Some("x\ny".into()) };
        let _ = ext.on_event(&event, &mut ctx);
        assert_eq!(ext.staged.len(), 1, "sem sessão, o arquivo espera");
        let id = ext.start(Target::Claude, launch(Target::Claude), &mut ctx);
        assert!(ext.staged.is_empty());
        let (tx, mut rx) = mpsc::unbounded();
        ext.session_mut(id).unwrap().cmd = Some(tx);
        assert_eq!(ext.session(id).unwrap().chips[0].label(&ext.root), "src/a.rs:3-4");
        type_text(&mut ext, "explica", &mut ctx);
        let _ = ext.update(Message::Send, &mut ctx);
        assert_eq!(
            prompts(&mut rx),
            [vec![
                PromptPart::Text("explica".into()),
                PromptPart::File { path: PathBuf::from("/proj/src/a.rs"), lines: Some((3, 4)), text: Some("x\ny".into()) },
            ]]
        );
        assert!(ext.session(id).unwrap().chips.is_empty());
        // Sem pedir, o contexto do editor não vira anexo.
        let _ = ext.on_event(&event, &mut ctx);
        assert!(ext.session(id).unwrap().chips.is_empty());
    }

    #[test]
    fn at_sign_opens_the_picker_only_at_a_word_start() {
        let (mut ext, _) = ext_with(&[Target::Claude]);
        let mut ctx = Context::new(Path::new("/tmp"), None);
        type_text(&mut ext, "a@b", &mut ctx);
        assert_eq!(ext.active_mut().unwrap().input.text(), "a@b", "num e-mail, o @ fica");
        type_text(&mut ext, " @", &mut ctx);
        assert_eq!(ext.active_mut().unwrap().input.text(), "a@b ", "depois de espaço, @ abre o seletor");
    }

    #[test]
    fn permission_summary_prefers_the_readable_parts() {
        let root = Path::new("/proj");
        let pending = |command: Option<&str>, content: Option<Vec<ToolContent>>, detail: &str| Pending {
            title: "t".into(),
            tool: ToolInfo { content, ..Default::default() },
            command: command.map(Into::into),
            url: None,
            detail: detail.into(),
            options: vec![],
            show_detail: false,
        };
        assert_eq!(permission_summary(&pending(Some("ls -la"), None, "{}"), root), "$ ls -la");
        let edit = vec![ToolContent::Diff { path: PathBuf::from("/proj/a.rs"), old: Some("a".into()), new: "b".into() }];
        assert_eq!(permission_summary(&pending(None, Some(edit), "{}"), root), fl!("acp-permission-edit", path = "a.rs"));
        assert_eq!(permission_summary(&pending(None, None, "{\n  \"x\": 1\n}"), root), "{\n  \"x\": 1\n}");
    }
}
