//! Sessão mínima contra um agente ACP de mentira, em stdio. Sem rede e sem conta.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use futures::channel::mpsc;
use tea_acp::permission::UserChoice;
use tea_acp::session::{self, Command, SessionEvent};
use tea_acp::target::{Launch, Target};

struct Dir {
    path: PathBuf,
}

impl Dir {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("tea-acp-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

struct Harness {
    cmd: mpsc::UnboundedSender<Command>,
    events: std::sync::mpsc::Receiver<SessionEvent>,
    dir: Dir,
}

fn start(mode: &str) -> Harness {
    let dir = Dir::new();
    let log = dir.path.join("log.json");
    let launch = Launch {
        target: Target::OpenCode,
        program: PathBuf::from(env!("CARGO_BIN_EXE_tea-acp-stub")),
        args: vec![mode.into(), log.display().to_string()],
    };
    let cwd = dir.path.clone();
    let (cmd, cmd_rx) = mpsc::unbounded();
    let (ev_tx, events) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        session::run(&launch, &cwd, cmd_rx, move |event| {
            let _ = ev_tx.send(event);
        });
    });
    Harness { cmd, events, dir }
}

impl Harness {
    fn log(&self) -> String {
        let path = self.dir.path.join("log.json");
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if let Ok(text) = std::fs::read_to_string(&path)
                && !text.is_empty()
            {
                return text;
            }
            if Instant::now() > deadline {
                panic!("o stub não gravou {}", path.display());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn next(&self) -> SessionEvent {
        self.events.recv_timeout(Duration::from_secs(8)).unwrap_or_else(|e| panic!("sem evento: {e}"))
    }

    fn until(&self, pred: impl Fn(&SessionEvent) -> bool) -> SessionEvent {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let event = self.events.recv_timeout(left).unwrap_or_else(|e| panic!("sem evento: {e}"));
            if pred(&event) {
                return event;
            }
            if matches!(event, SessionEvent::Failed { .. } | SessionEvent::Closed) {
                panic!("a sessão acabou antes: {event:?}");
            }
        }
    }
}

#[test]
fn stub_session_streams_text_and_a_tool_call() {
    let h = start("session");
    let ready = h.next();
    assert!(matches!(ready, SessionEvent::Ready { .. }), "{ready:?}");
    h.cmd.unbounded_send(Command::Prompt("oi".into())).unwrap();
    let text = h.until(|e| matches!(e, SessionEvent::Assistant { .. }));
    assert!(matches!(text, SessionEvent::Assistant { text } if text == "olá do stub"));
    let tool = h.until(|e| matches!(e, SessionEvent::Tool { .. }));
    assert!(matches!(tool, SessionEvent::Tool { title, status, .. } if title == "ler arquivo" && status == "in_progress"));
    let turn = h.until(|e| matches!(e, SessionEvent::Turn { .. }));
    assert!(matches!(turn, SessionEvent::Turn { stop } if stop == "end_turn"));
    h.cmd.unbounded_send(Command::Shutdown).unwrap();
    let _ = h.until(|e| matches!(e, SessionEvent::Closed));
}

#[test]
fn permission_allow_reject_and_cancel() {
    assert_outcome(Command::Permission(UserChoice::Allow), "allow");
    assert_outcome(Command::Permission(UserChoice::Reject), "reject");
    assert_outcome(Command::Cancel, "cancelled");
}

fn assert_outcome(command: Command, needle: &str) {
    let h = start("permission");
    let _ = h.until(|e| matches!(e, SessionEvent::Ready { .. }));
    h.cmd.unbounded_send(Command::Prompt("faz".into())).unwrap();
    let asked = h.until(|e| matches!(e, SessionEvent::Permission { .. }));
    let SessionEvent::Permission { title, options, .. } = asked else { unreachable!() };
    assert_eq!(title, "echo oi");
    assert!(options.iter().any(|o| o.id == "allow" && o.kind.is_allow()));
    assert!(options.iter().any(|o| o.id == "reject" && o.kind.is_reject()));
    // Dispensar não responde: a próxima escolha é que vale, e não é uma permissão.
    h.cmd.unbounded_send(Command::Permission(UserChoice::Dismiss)).unwrap();
    h.cmd.unbounded_send(command).unwrap();
    let log = h.log();
    assert!(log.contains(needle), "{log}");
    if needle != "allow" {
        assert!(!log.contains("allow"), "dispensar, recusar ou cancelar não pode permitir: {log}");
    }
    let turn = h.until(|e| matches!(e, SessionEvent::Turn { .. }));
    assert!(matches!(turn, SessionEvent::Turn { .. }), "{turn:?}");
}

#[test]
fn unauthenticated_stub_does_not_hang() {
    let h = start("auth");
    let event = h.until(|e| matches!(e, SessionEvent::AuthRequired { .. }));
    let SessionEvent::AuthRequired { logins, .. } = event else { unreachable!() };
    let ids: Vec<_> = logins.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["claude-ai-login", "console-login"]);
    assert_eq!(logins[0].args, ["--cli", "auth", "login", "--claudeai"]);
    let closed = h.next();
    assert!(matches!(closed, SessionEvent::Closed), "{closed:?}");
    let init = h.log();
    assert!(init.contains("\"terminal\":true") || init.contains("\"terminal\": true"), "o cliente anuncia auth.terminal: {init}");
}
