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
    AuthCapabilities, AuthMethod, CancelNotification, ClientCapabilities, ContentBlock, Implementation,
    InitializeRequest, PromptResponse, RequestPermissionRequest, RequestPermissionResponse, SessionNotification,
    SessionUpdate, StopReason, TextContent, ToolCallStatus,
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
    Prompt(String),
    /// Para o turno (`session/cancel`). Se houver um pedido de permissão aberto,
    /// a resposta é `cancelled`, nunca uma permissão.
    Cancel,
    /// Escolha explícita num pedido de permissão. `Dismiss` não responde.
    Permission(UserChoice),
    Shutdown,
}

/// O que a sessão conta para a interface, na ordem em que chega.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    /// `initialize` e `session/new` passaram.
    Ready { agent: String },
    /// Texto do assistente, um pedaço de cada vez.
    Assistant { text: String },
    /// Pensamento do agente (não é a resposta).
    Thought { text: String },
    /// Estado de uma ferramenta (`tool_call` / `tool_call_update`).
    Tool { id: String, title: String, status: String },
    /// O agente pediu permissão. A interface espera uma escolha explícita.
    Permission { title: String, detail: String, options: Vec<ShownOption> },
    /// O turno acabou.
    Turn { stop: String },
    /// O agente exigiu login (`authRequired`) ou anunciou métodos de terminal.
    /// A sessão não fica pendurada: isto encerra a tentativa.
    AuthRequired { message: String, logins: Vec<LoginMethod> },
    /// Falha que não é de autenticação.
    Failed { message: String },
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
        SessionEvent::Tool { status, .. } if status == "in_progress" || status == "pending" => Some(MascotCue::Working),
        SessionEvent::Turn { stop } if stop != "error" && stop != "cancelled" => Some(MascotCue::Celebrating),
        SessionEvent::Failed { .. } => Some(MascotCue::Error),
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
        Ok(Err(error)) => {
            emit(SessionEvent::Failed { message: error.to_string() });
            emit(SessionEvent::Closed);
        }
        Err(_) => {
            emit(SessionEvent::Failed { message: "a sessão ACP entrou em pânico".into() });
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
            let SessionSetup { mut session, .. } = cx.build_session(&cwd).block_task().start_session().await?;
            run_session(&mut session, &mut commands, emit).await
        })
        .await
}

fn absolute_cwd(cwd: &Path) -> PathBuf {
    if cwd.is_absolute() {
        return cwd.to_path_buf();
    }
    std::env::current_dir().map(|base| base.join(cwd)).unwrap_or_else(|_| cwd.to_path_buf())
}

async fn run_session<Link>(
    session: &mut agent_client_protocol::v1::ActiveSession<'static, Link>,
    commands: &mut mpsc::UnboundedReceiver<Command>,
    emit: &mut (dyn FnMut(SessionEvent) + Send),
) -> Result<(), Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    let agent_name = session.session_id().0.to_string();
    emit(SessionEvent::Ready { agent: agent_name });
    let mut queued: Vec<String> = Vec::new();
    loop {
        let update = futures::select! {
            update = session.read_update().fuse() => update?,
            command = commands.next().fuse() => {
                if apply_command(session, command)? == Step::Stop {
                    return Ok(());
                }
                continue;
            }
        };
        match update {
            SessionMessage::PromptResponse(response) => {
                emit(SessionEvent::Turn { stop: stop_label(&response) });
                response?;
                for text in queued.drain(..) {
                    session.send_prompt(text)?;
                }
            }
            SessionMessage::Message(dispatch) => {
                handle_dispatch(session, dispatch, commands, emit, &mut queued).await?;
            }
            _ => {}
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
) -> Result<Step, Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    match command {
        Some(Command::Prompt(text)) => session.send_prompt(text)?,
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
    queued: &mut Vec<String>,
) -> Result<(), Error>
where
    Link: agent_client_protocol::role::HasPeer<agent_client_protocol::Agent> + Clone,
{
    // O pedido de permissão é respondido aqui, esperando a pessoa. Enquanto
    // espera, o laço de fora não corre: o agente também está parado neste pedido.
    let mut note: Option<SessionNotification> = None;
    agent_client_protocol::util::MatchDispatch::new(dispatch.into_raw())
        .if_request(async |req: RequestPermissionRequest, responder| {
            let (title, detail) = permission_text(&req);
            emit(SessionEvent::Permission { title, detail, options: permission::shown_options(&req.options) });
            let outcome = wait_permission(session, &req, commands, queued).await?;
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
    queued: &mut Vec<String>,
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
            Some(Command::Prompt(text)) => queued.push(text),
        }
    }
}

fn permission_text(req: &RequestPermissionRequest) -> (String, String) {
    let title = req
        .tool_call
        .fields
        .title
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| req.tool_call.tool_call_id.0.to_string());
    let detail = req.tool_call.fields.raw_input.as_ref().map(|v| v.to_string()).unwrap_or_default();
    (title, detail)
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
        SessionUpdate::ToolCall(call) => {
            emit(SessionEvent::Tool { id: call.tool_call_id.0.to_string(), title: call.title.clone(), status: status_label(call.status) });
        }
        SessionUpdate::ToolCallUpdate(update) => {
            let title = update.fields.title.clone().unwrap_or_default();
            let status = update.fields.status.map(status_label).unwrap_or_else(|| "update".into());
            emit(SessionEvent::Tool { id: update.tool_call_id.0.to_string(), title, status });
        }
        _ => {}
    }
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
            mascot_cue(&SessionEvent::Permission { title: "t".into(), detail: String::new(), options: vec![] }),
            Some(MascotCue::Attention)
        );
        assert_eq!(
            mascot_cue(&SessionEvent::Tool { id: "1".into(), title: "bash".into(), status: "in_progress".into() }),
            Some(MascotCue::Working)
        );
        assert_eq!(mascot_cue(&SessionEvent::Tool { id: "1".into(), title: "bash".into(), status: "completed".into() }), None);
        assert_eq!(mascot_cue(&SessionEvent::Turn { stop: "end_turn".into() }), Some(MascotCue::Celebrating));
        assert_eq!(mascot_cue(&SessionEvent::Turn { stop: "cancelled".into() }), None);
        assert_eq!(mascot_cue(&SessionEvent::Failed { message: "x".into() }), Some(MascotCue::Error));
        assert_eq!(mascot_cue(&SessionEvent::Assistant { text: "oi".into() }), None);
    }
}
