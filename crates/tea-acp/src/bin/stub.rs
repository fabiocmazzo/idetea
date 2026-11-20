//! Agente ACP mínimo, só para os testes: stdio, uma mensagem JSON por linha.
//!
//! Não fala com a rede nem precisa de conta. O modo é o primeiro argumento:
//! `session` (texto e ferramenta), `permission` (pede permissão), `auth`
//! (`authRequired`) ou `queue` (responde cada prompt depois de 300 ms, grava no arquivo
//! um prompt por linha e `overlap` se chegar um com outro ainda aberto). O segundo argumento, se houver, é um arquivo onde o stub
//! grava o que o teste precisa ler (o stderr fica com o cliente).

use std::io::{BufRead, Write};
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "session".into());
    let log_path = args.next().map(PathBuf::from);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut prompt_id: Option<serde_json::Value> = None;
    let mut perm_id: Option<serde_json::Value> = None;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id").cloned();

        if method.is_empty() {
            if id.is_some() && msg.get("result").is_some() && id == perm_id {
                if let Some(path) = &log_path {
                    let _ = std::fs::write(path, serde_json::to_string(&msg["result"]).unwrap_or_default());
                }
                let stop = if msg.pointer("/result/outcome/outcome").and_then(|v| v.as_str()) == Some("cancelled") {
                    "cancelled"
                } else {
                    "end_turn"
                };
                if let Some(prompt) = prompt_id.clone() {
                    let _ = write_line(&mut stdout, &ok(&prompt, serde_json::json!({ "stopReason": stop })));
                }
            }
            continue;
        }

        let response = match method {
            "initialize" => {
                if let Some(path) = &log_path
                    && mode == "auth"
                {
                    let _ = std::fs::write(path, line.clone());
                }
                ok(
                    id.as_ref().unwrap_or(&serde_json::Value::Null),
                    serde_json::json!({
                        "protocolVersion": 1,
                        "agentCapabilities": {},
                        "agentInfo": { "name": "tea-acp-stub", "title": "Stub", "version": "0.0.1" },
                        "authMethods": if mode == "auth" { auth_methods() } else { serde_json::json!([]) }
                    }),
                )
            }
            "session/new" if mode == "auth" => err(id.as_ref().unwrap_or(&serde_json::Value::Null), -32000, "Authentication required"),
            "session/new" => ok(id.as_ref().unwrap_or(&serde_json::Value::Null), serde_json::json!({ "sessionId": "stub-1" })),
            "session/prompt" if mode == "queue" => {
                let open = std::sync::Arc::clone(&OPEN);
                if open.fetch_add(1, std::sync::atomic::Ordering::SeqCst) > 0 {
                    append(log_path.as_ref(), "overlap");
                }
                append(log_path.as_ref(), &serde_json::to_string(&msg["params"]["prompt"]).unwrap_or_default());
                let id = id.clone().unwrap_or(serde_json::Value::Null);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    open.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    let _ = write_line(&mut std::io::stdout(), &ok(&id, serde_json::json!({ "stopReason": "end_turn" })));
                });
                continue;
            }
            "session/prompt" => {
                prompt_id = id.clone();
                if mode == "permission" {
                    let _ = write_line(&mut stdout, &note("agent_message_chunk", serde_json::json!({
                        "content": { "type": "text", "text": "vou pedir" }
                    })));
                    perm_id = Some(serde_json::json!(9001));
                    let _ = write_line(&mut stdout, &serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 9001,
                        "method": "session/request_permission",
                        "params": {
                            "sessionId": "stub-1",
                            "toolCall": {
                                "toolCallId": "call-1",
                                "title": "echo oi",
                                "kind": "execute",
                                "status": "pending"
                            },
                            "options": [
                                { "optionId": "allow", "name": "Allow once", "kind": "allow_once" },
                                { "optionId": "reject", "name": "Reject once", "kind": "reject_once" }
                            ]
                        }
                    }));
                    continue;
                }
                let _ = write_line(&mut stdout, &note("agent_message_chunk", serde_json::json!({
                    "content": { "type": "text", "text": "olá do stub" }
                })));
                let _ = write_line(&mut stdout, &note("tool_call", serde_json::json!({
                    "toolCallId": "call-1",
                    "title": "ler arquivo",
                    "kind": "read",
                    "status": "in_progress"
                })));
                ok(id.as_ref().unwrap_or(&serde_json::Value::Null), serde_json::json!({ "stopReason": "end_turn" }))
            }
            "session/cancel" => continue,
            _ => {
                if let Some(id) = id.as_ref() {
                    err(id, -32601, "method not found")
                } else {
                    continue;
                }
            }
        };
        if write_line(&mut stdout, &response).is_err() {
            break;
        }
    }
}

static OPEN: std::sync::LazyLock<std::sync::Arc<std::sync::atomic::AtomicUsize>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)));

fn append(path: Option<&PathBuf>, line: &str) {
    if let Some(path) = path
        && let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path)
    {
        let _ = writeln!(file, "{line}");
    }
}

fn auth_methods() -> serde_json::Value {
    serde_json::json!([
        {
            "type": "terminal",
            "id": "claude-ai-login",
            "name": "Claude Subscription",
            "description": "Use Claude subscription",
            "args": ["--cli", "auth", "login", "--claudeai"]
        },
        {
            "type": "terminal",
            "id": "console-login",
            "name": "Anthropic Console",
            "description": "Use Anthropic Console",
            "args": ["--cli", "auth", "login", "--console"]
        }
    ])
}

fn note(kind: &str, mut extra: serde_json::Value) -> serde_json::Value {
    extra["sessionUpdate"] = serde_json::Value::String(kind.into());
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": { "sessionId": "stub-1", "update": extra }
    })
}

fn ok(id: &serde_json::Value, result: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: &serde_json::Value, code: i32, message: &str) -> serde_json::Value {
    serde_json::json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn write_line(stdout: &mut impl Write, value: &serde_json::Value) -> std::io::Result<()> {
    serde_json::to_writer(&mut *stdout, value)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}
