//! Uma sessão ACP v1: `initialize`, `session/new`, `session/prompt`,
//! `session/update`, `session/request_permission` e `session/cancel`.
//!
//! O processo é stdio (uma mensagem JSON-RPC por linha). O stderr do agente
//! fica com ele. Não há HTTP. O protocolo é o v1 estável: o OpenCode fala v1,
//! e o adaptador do Claude só serve v2 quando o cliente pede a versão 2.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::v1::schema::{
    AuthCapabilities, AuthMethod, CancelNotification, ClientCapabilities, ContentBlock, EmbeddedResource,
    EmbeddedResourceResource, Implementation, InitializeRequest, PromptResponse, RequestPermissionRequest,
    RequestPermissionResponse, ResourceLink, SessionNotification, SessionUpdate, StopReason, TextContent,
    TextResourceContents, ToolCallContent, ToolCallLocation, ToolCallStatus, ToolKind,
};
use agent_client_protocol::v1::{SessionMessage, SessionSetup};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Client, Error, ErrorCode};
use futures::channel::mpsc;
use futures::{FutureExt, StreamExt};

use crate::permission::{self, PermissionAnswer, ShownOption, UserChoice};
use crate::target::Launch;

/// O que a interface manda para a sessão que já está no ar.
#[derive(Debug, Clone)]
pub enum Command {
    /// Uma mensagem. Se um turno estiver aberto, ela espera o fim dele: nunca há dois
    /// `session/prompt` ao mesmo tempo na mesma sessão.
    Prompt(Vec<PromptPart>),
    /// Para o turno (`session/cancel`). Se houver um pedido de permissão aberto,
    /// a resposta é `cancelled`, nunca uma permissão.
    Cancel,
    /// Escolha explícita num pedido de permissão. `Dismiss` não responde.
    Permission(UserChoice),
    Shutdown,
}

/// Um pedaço da mensagem: texto, ou um arquivo do projeto (inteiro ou umas linhas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptPart {
    Text(String),
    /// `lines` começa em 1, inclusivo. `text` é o conteúdo que a pessoa vê (o buffer, com o que
    /// não foi salvo); sem ele, vai só o link e o agente lê do disco.
    File { path: PathBuf, lines: Option<(u32, u32)>, text: Option<String> },
}

impl PromptPart {
    pub fn text(text: impl Into<String>) -> Vec<PromptPart> {
        vec![PromptPart::Text(text.into())]
    }
}

/// Uma ferramenta, como a interface mostra. Num `tool_call_update` só vêm os campos que
/// mudaram; os outros ficam `None` e a interface guarda o que já sabia.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolInfo {
    pub id: String,
    pub title: Option<String>,
    /// `pending`, `in_progress`, `completed`, `failed`.
    pub status: Option<String>,
    /// `read`, `edit`, `delete`, `move`, `search`, `execute`, `think`, `fetch`, `other`.
    pub kind: Option<String>,
    pub locations: Option<Vec<ToolLocation>>,
    pub content: Option<Vec<ToolContent>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolLocation {
    pub path: PathBuf,
    /// Começa em 1.
    pub line: Option<u32>,
}

/// O que a ferramenta produziu (ou vai produzir, num pedido de permissão).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolContent {
    Text(String),
    Diff { path: PathBuf, old: Option<String>, new: String },
    /// Um terminal do cliente; este cliente não anuncia `terminal`, então só o id.
    Terminal(String),
}

/// Por que a sessão falhou, para a interface traduzir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// Erro do protocolo ou do processo, com a mensagem dele (e o fim do stderr, quando o processo saiu).
    Message(String),
    /// A thread da sessão entrou em pânico.
    Panic,
}

/// O que a sessão conta para a interface, na ordem em que chega.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    /// `initialize` e `session/new` passaram. `agent` e `version` vêm do `agentInfo`, quando o agente manda.
    Ready { agent: Option<String>, version: Option<String> },
    /// Texto do assistente, um pedaço de cada vez.
    Assistant { text: String },
    /// Pensamento do agente (não é a resposta).
    Thought { text: String },
    /// Estado de uma ferramenta (`tool_call` / `tool_call_update`).
    Tool(ToolInfo),
    /// O agente pediu permissão. A interface espera uma escolha explícita. `detail` é o
    /// `rawInput` em JSON indentado (vazio sem ele); `command` e `url`, quando o `rawInput` os traz.
    Permission { title: String, tool: ToolInfo, command: Option<String>, url: Option<String>, detail: String, options: Vec<ShownOption> },
    /// Uma mensagem da fila saiu para o agente (o turno anterior acabou).
    Sent,
    /// O turno acabou.
    Turn { stop: String },
    /// O agente exigiu login (`authRequired`) ou anunciou métodos de terminal.
    /// A sessão não fica pendurada: isto encerra a tentativa.
    AuthRequired { message: String, logins: Vec<LoginMethod> },
    /// Falha que não é de autenticação.
    Failed(Failure),
    /// O processo saiu.
    Closed,
}

/// Um login que a pessoa roda num terminal (o adaptador não aceita `authenticate`
/// para método `terminal`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginMethod {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Argumentos para acrescentar ao programa do agente.
    pub args: Vec<String>,
}

/// Liga o mascote aos eventos da sessão, sem ser o protocolo. Os hooks
/// (`idetea mascot setup-hooks`) continuam o caminho do Claude Code; isto é o
/// gancho para um marco seguinte usar os mesmos estados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotCue {
    /// O agente espera uma resposta (`attention`).
    Attention,
    /// Uma ferramenta está rodando (`working`).
    Working,
    /// O turno terminou sem erro (`celebrating`).
    Celebrating,
    /// O turno ou a sessão falhou (`error`).
    Error,
}

pub fn mascot_cue(event: &SessionEvent) -> Option<MascotCue> {
    match event {
        SessionEvent::Permission { .. } | SessionEvent::AuthRequired { .. } => Some(MascotCue::Attention),
        SessionEvent::Tool(ToolInfo { status: Some(status), .. }) if status == "in_progress" || status == "pending" => {
            Some(MascotCue::Working)
        }
        SessionEvent::Turn { stop } if stop != "error" && stop != "cancelled" => Some(MascotCue::Celebrating),
        SessionEvent::Failed(_) => Some(MascotCue::Error),
        _ => None,
    }
}

/// Roda a sessão até `Shutdown`, o processo cair, ou um erro. Bloqueia a thread.
pub fn run(launch: &Launch, cwd: &Path, commands: mpsc::UnboundedReceiver<Command>, mut emit: impl FnMut(SessionEvent) + Send) {
    let logins = Arc::new(Mutex::new(Vec::new()));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        futures::executor::block_on(drive(launch, cwd, commands, &mut emit, Arc::clone(&logins)))
    }));
    match result {
        Ok(Ok(())) => emit(SessionEvent::Closed),
        Ok(Err(error)) if error.code == ErrorCode::AUTH_REQUIRED => {
            let methods = logins.lock().unwrap_or_else(|p| p.into_inner()).clone();
            emit(SessionEvent::AuthRequired { message: error.message, logins: methods });
            emit(SessionEvent::Closed);
        }
        // O encerramento pedido pela aba (fechar) não é falha.
        Ok(Err(error)) if error.message == "shutdown" => emit(SessionEvent::Closed),
        Ok(Err(error)) => {
            emit(SessionEvent::Failed(Failure::Message(error.to_string())));
            emit(SessionEvent::Closed);
        }
        Err(_) => {
            emit(SessionEvent::Failed(Failure::Panic));
            emit(SessionEvent::Closed);
        }
    }
}

async fn drive(
    launch: &Launch,
    cwd: &Path,
    mut commands: mpsc::UnboundedReceiver<Command>,
    emit: &mut (dyn FnMut(SessionEvent) + Send),
    logins: Arc<Mutex<Vec<LoginMethod>>>,
) -> Result<(), Error> {
    let agent = AcpAgent::new(AcpAgentConfig::new(&launch.program).args(launch.args.clone()));
    let cwd = absolute_cwd(cwd);
    Client
        .v1()
        .name("IDE TEA")
        .connect_with(agent, async move |cx| {
            let request = InitializeRequest::new(ProtocolVersion::V1)
                .client_capabilities(ClientCapabilities::new().auth(AuthCapabilities::new().terminal(true)))
                .client_info(Implementation::new("IDE TEA", env!("CARGO_PKG_VERSION")));
            let init = cx.send_request(request).block_task().await?;
            *logins.lock().unwrap_or_else(|p| p.into_inner()) = logins_from(&init.auth_methods);
            let caps = &init.agent_capabilities.prompt_capabilities;
            let info = Agent {
                name: init.agent_info.as_ref().map(|i| i.title.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| i.name.clone())),
                version: init.agent_info.as_ref().map(|i| i.version.clone()).filter(|v| !v.is_empty()),
                embedded_context: caps.embedded_context,
            };
            let SessionSetup { mut session, .. } = cx.build_session(&cwd).block_task().start_session().await?;
            run_session(&mut session, &info, &mut commands, emit).await
        })
        .await
}

/// O que o `initialize` disse do agente e serve à sessão.
struct Agent {
    name: Option<String>,
    version: Option<String>,
    /// `promptCapabilities.embeddedContext`: aceita o texto do arquivo dentro da mensagem.
    embedded_context: bool,
}

fn absolute_cwd(cwd: &Path) -> PathBuf {
    if cwd.is_absolute() {
        return cwd.to_path_buf();
    }
    std::env::current_dir().map(|base| base.join(cwd)).unwrap_or_else(|_| cwd.to_path_buf())
}

async fn run_session<Link>(
    session: &mut agent_client_protocol::v1::ActiveSession<'static, Link>,
    agent: &Agent,
    commands: &mut mpsc::UnboundedReceiver<Command>,
    emit: &mut (dyn FnMut(SessionEvent) + Send),
) -> Result<(), Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    emit(SessionEvent::Ready { agent: agent.name.clone(), version: agent.version.clone() });
    let mut turn = Turn { open: false, queued: Vec::new(), embedded: agent.embedded_context };
    loop {
        let update = futures::select! {
            update = session.read_update().fuse() => update?,
            command = commands.next().fuse() => {
                if apply_command(session, command, &mut turn)? == Step::Stop {
                    return Ok(());
                }
                continue;
            }
        };
        match update {
            SessionMessage::PromptResponse(response) => {
                turn.open = false;
                emit(SessionEvent::Turn { stop: stop_label(&response) });
                response?;
                // Uma de cada vez: a próxima só sai quando esta acabar.
                if !turn.queued.is_empty() {
                    let next = turn.queued.remove(0);
                    turn.send(session, next)?;
                    emit(SessionEvent::Sent);
                }
            }
            SessionMessage::Message(dispatch) => {
                handle_dispatch(session, dispatch, commands, emit, &mut turn).await?;
            }
            _ => {}
        }
    }
}

/// O turno aberto e o que espera por ele.
struct Turn {
    open: bool,
    queued: Vec<Vec<PromptPart>>,
    embedded: bool,
}

impl Turn {
    fn send<Link>(&mut self, session: &mut agent_client_protocol::v1::ActiveSession<'static, Link>, parts: Vec<PromptPart>) -> Result<(), Error>
    where
        Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
    {
        self.open = true;
        session.send_prompt_blocks(prompt_blocks(parts, self.embedded))
    }

    fn prompt<Link>(&mut self, session: &mut agent_client_protocol::v1::ActiveSession<'static, Link>, parts: Vec<PromptPart>) -> Result<(), Error>
    where
        Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
    {
        if self.open {
            self.queued.push(parts);
            Ok(())
        } else {
            self.send(session, parts)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Continue,
    Stop,
}

fn apply_command<Link>(
    session: &mut agent_client_protocol::v1::ActiveSession<'static, Link>,
    command: Option<Command>,
    turn: &mut Turn,
) -> Result<Step, Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    match command {
        Some(Command::Prompt(parts)) => turn.prompt(session, parts)?,
        Some(Command::Cancel) => {
            session.connection().send_notification_to(agent_client_protocol::Agent, CancelNotification::new(session.session_id().clone()))?;
        }
        Some(Command::Shutdown) | None => return Ok(Step::Stop),
        // Permissão sem pedido aberto não aprova nada.
        Some(Command::Permission(_)) => {}
    }
    Ok(Step::Continue)
}

async fn handle_dispatch<Link>(
    session: &mut agent_client_protocol::v1::ActiveSession<'static, Link>,
    dispatch: agent_client_protocol::v1::Dispatch,
    commands: &mut mpsc::UnboundedReceiver<Command>,
    emit: &mut (dyn FnMut(SessionEvent) + Send),
    turn: &mut Turn,
) -> Result<(), Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    // O pedido de permissão é respondido aqui, esperando a pessoa. Enquanto
    // espera, o laço de fora não corre: o agente também está parado neste pedido.
    let mut note: Option<SessionNotification> = None;
    agent_client_protocol::util::MatchDispatch::new(dispatch.into_raw())
        .if_request(async |req: RequestPermissionRequest, responder| {
            emit(permission_event(&req));
            let outcome = wait_permission(session, &req, commands, turn).await?;
            responder.respond(RequestPermissionResponse::new(outcome))
        })
        .await
        .if_notification(async |notif: SessionNotification| {
            note = Some(notif);
            Ok(())
        })
        .await
        .otherwise_ignore()?;
    if let Some(notif) = note {
        apply_update(&notif.update, emit);
    }
    Ok(())
}

async fn wait_permission<Link>(
    session: &agent_client_protocol::v1::ActiveSession<'static, Link>,
    req: &RequestPermissionRequest,
    commands: &mut mpsc::UnboundedReceiver<Command>,
    turn: &mut Turn,
) -> Result<agent_client_protocol::v1::schema::RequestPermissionOutcome, Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    loop {
        match commands.next().await {
            Some(Command::Permission(UserChoice::Dismiss)) => {
                // Esc, clique fora, fechar: o pedido continua aberto.
            }
            Some(Command::Permission(choice)) => match permission::answer(&req.options, choice) {
                PermissionAnswer::Pending => {}
                PermissionAnswer::Selected(outcome) => return Ok(outcome),
                PermissionAnswer::Cancel => {
                    session.connection().send_notification_to(agent_client_protocol::Agent, CancelNotification::new(session.session_id().clone()))?;
                    return Ok(permission::cancelled_outcome());
                }
            },
            Some(Command::Cancel) | Some(Command::Shutdown) | None => {
                session.connection().send_notification_to(agent_client_protocol::Agent, CancelNotification::new(session.session_id().clone()))?;
                return Ok(permission::cancelled_outcome());
            }
            // O turno está aberto (é ele que pede): a mensagem espera.
            Some(Command::Prompt(parts)) => turn.queued.push(parts),
        }
    }
}

fn permission_event(req: &RequestPermissionRequest) -> SessionEvent {
    let fields = &req.tool_call.fields;
    let mut tool = update_info(&req.tool_call.tool_call_id.0, fields);
    let title = tool.title.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| tool.id.clone());
    tool.title = Some(title.clone());
    let input = fields.raw_input.as_ref();
    let field = |name: &str| input.and_then(|v| v.get(name)).and_then(|v| v.as_str()).map(str::to_string).filter(|s| !s.is_empty());
    let detail = input.map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())).unwrap_or_default();
    SessionEvent::Permission { title, command: field("command"), url: field("url"), detail, tool, options: permission::shown_options(&req.options) }
}

fn apply_update(update: &SessionUpdate, emit: &mut dyn FnMut(SessionEvent)) {
    match update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            if let Some(text) = block_text(&chunk.content) {
                emit(SessionEvent::Assistant { text });
            }
        }
        SessionUpdate::AgentThoughtChunk(chunk) => {
            if let Some(text) = block_text(&chunk.content) {
                emit(SessionEvent::Thought { text });
            }
        }
        SessionUpdate::ToolCall(call) => emit(SessionEvent::Tool(ToolInfo {
            id: call.tool_call_id.0.to_string(),
            title: Some(call.title.clone()),
            status: Some(status_label(call.status)),
            kind: Some(kind_label(call.kind)),
            locations: Some(locations(&call.locations)),
            content: Some(contents(&call.content)),
        })),
        SessionUpdate::ToolCallUpdate(update) => emit(SessionEvent::Tool(update_info(&update.tool_call_id.0, &update.fields))),
        _ => {}
    }
}

fn update_info(id: &str, fields: &agent_client_protocol::v1::schema::ToolCallUpdateFields) -> ToolInfo {
    ToolInfo {
        id: id.to_string(),
        title: fields.title.clone().filter(|t| !t.is_empty()),
        status: fields.status.map(status_label),
        kind: fields.kind.map(kind_label),
        locations: fields.locations.as_deref().map(locations),
        content: fields.content.as_deref().map(contents),
    }
}

fn locations(list: &[ToolCallLocation]) -> Vec<ToolLocation> {
    list.iter().map(|l| ToolLocation { path: l.path.clone(), line: l.line }).collect()
}

fn contents(list: &[ToolCallContent]) -> Vec<ToolContent> {
    list.iter()
        .filter_map(|c| match c {
            ToolCallContent::Content(c) => block_text(&c.content).map(ToolContent::Text),
            ToolCallContent::Diff(d) => Some(ToolContent::Diff { path: d.path.clone(), old: d.old_text.clone(), new: d.new_text.clone() }),
            ToolCallContent::Terminal(t) => Some(ToolContent::Terminal(t.terminal_id.0.to_string())),
            _ => None,
        })
        .collect()
}

fn kind_label(kind: ToolKind) -> String {
    match kind {
        ToolKind::Read => "read",
        ToolKind::Edit => "edit",
        ToolKind::Delete => "delete",
        ToolKind::Move => "move",
        ToolKind::Search => "search",
        ToolKind::Execute => "execute",
        ToolKind::Think => "think",
        ToolKind::Fetch => "fetch",
        _ => "other",
    }
    .into()
}

/// A mensagem em blocos ACP. Todo agente v1 aceita texto e `resource_link`; o texto do arquivo
/// dentro da mensagem (`resource`) só vai quando o agente anunciou `embeddedContext`. Sem isso,
/// as linhas escolhidas vão como texto, depois do link.
fn prompt_blocks(parts: Vec<PromptPart>, embedded: bool) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    for part in parts {
        match part {
            PromptPart::Text(text) => blocks.push(ContentBlock::Text(TextContent::new(text))),
            PromptPart::File { path, lines, text } => {
                let uri = file_uri(&path, lines);
                match text {
                    Some(text) if embedded => blocks.push(ContentBlock::Resource(EmbeddedResource::new(
                        EmbeddedResourceResource::TextResourceContents(TextResourceContents::new(text, uri)),
                    ))),
                    Some(text) => {
                        blocks.push(ContentBlock::ResourceLink(ResourceLink::new(file_name(&path), uri)));
                        let fence = if text.contains("```") { "````" } else { "```" };
                        blocks.push(ContentBlock::Text(TextContent::new(format!("{fence}\n{}\n{fence}", text.trim_end_matches('\n')))));
                    }
                    None => blocks.push(ContentBlock::ResourceLink(ResourceLink::new(file_name(&path), uri))),
                }
            }
        }
    }
    blocks
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string())
}

/// `file:///caminho#L3-L9`, com o caminho em percent-encoding (espaços, acentos).
pub fn file_uri(path: &Path, lines: Option<(u32, u32)>) -> String {
    let mut uri = String::from("file://");
    for byte in path.to_string_lossy().as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => uri.push(*byte as char),
            other => uri.push_str(&format!("%{other:02X}")),
        }
    }
    match lines {
        Some((a, b)) if a == b => uri.push_str(&format!("#L{a}")),
        Some((a, b)) => uri.push_str(&format!("#L{a}-L{b}")),
        None => {}
    }
    uri
}

fn block_text(block: &ContentBlock) -> Option<String> {
    match block {
        ContentBlock::Text(TextContent { text, .. }) => Some(text.clone()),
        _ => None,
    }
}

fn status_label(status: ToolCallStatus) -> String {
    match status {
        ToolCallStatus::Pending => "pending".into(),
        ToolCallStatus::InProgress => "in_progress".into(),
        ToolCallStatus::Completed => "completed".into(),
        ToolCallStatus::Failed => "failed".into(),
        _ => "other".into(),
    }
}

fn stop_label(response: &Result<PromptResponse, Error>) -> String {
    match response {
        Ok(response) => match response.stop_reason {
            StopReason::EndTurn => "end_turn".into(),
            StopReason::MaxTokens => "max_tokens".into(),
            StopReason::MaxTurnRequests => "max_turn_requests".into(),
            StopReason::Refusal => "refusal".into(),
            StopReason::Cancelled => "cancelled".into(),
            _ => "other".into(),
        },
        Err(_) => "error".into(),
    }
}

fn logins_from(methods: &[AuthMethod]) -> Vec<LoginMethod> {
    methods
        .iter()
        .filter_map(|method| match method {
            AuthMethod::Terminal(t) => Some(LoginMethod {
                id: t.id.0.to_string(),
                name: t.name.clone(),
                description: t.description.clone().unwrap_or_default(),
                args: t.args.clone(),
            }),
            AuthMethod::Agent(_) => None,
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mascot_cues_follow_the_session_and_not_the_hooks_file() {
        assert_eq!(
            mascot_cue(&SessionEvent::Permission {
                title: "t".into(),
                tool: ToolInfo::default(),
                command: None,
                url: None,
                detail: String::new(),
                options: vec![]
            }),
            Some(MascotCue::Attention)
        );
        assert_eq!(
            mascot_cue(&SessionEvent::Tool(ToolInfo { id: "1".into(), status: Some("in_progress".into()), ..Default::default() })),
            Some(MascotCue::Working)
        );
        assert_eq!(mascot_cue(&SessionEvent::Tool(ToolInfo { id: "1".into(), status: Some("completed".into()), ..Default::default() })), None);
        assert_eq!(mascot_cue(&SessionEvent::Turn { stop: "end_turn".into() }), Some(MascotCue::Celebrating));
        assert_eq!(mascot_cue(&SessionEvent::Turn { stop: "cancelled".into() }), None);
        assert_eq!(mascot_cue(&SessionEvent::Failed(Failure::Panic)), Some(MascotCue::Error));
        assert_eq!(mascot_cue(&SessionEvent::Assistant { text: "oi".into() }), None);
    }

    #[test]
    fn embedded_context_only_when_the_agent_accepts_it() {
        let parts = || vec![PromptPart::File { path: PathBuf::from("/p/a.rs"), lines: Some((2, 2)), text: Some("x".into()) }];
        let with = prompt_blocks(parts(), true);
        assert!(matches!(with.as_slice(), [ContentBlock::Resource(_)]));
        let without = prompt_blocks(parts(), false);
        assert!(matches!(without.as_slice(), [ContentBlock::ResourceLink(_), ContentBlock::Text(_)]));
        let link_only = prompt_blocks(vec![PromptPart::File { path: PathBuf::from("/p/a.rs"), lines: None, text: None }], true);
        assert!(matches!(link_only.as_slice(), [ContentBlock::ResourceLink(_)]));
        assert_eq!(file_uri(Path::new("/p/ação.rs"), Some((2, 2))), "file:///p/a%C3%A7%C3%A3o.rs#L2");
    }
}
