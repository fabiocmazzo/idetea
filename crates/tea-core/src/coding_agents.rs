//! Os agentes de código que a IDE conhece (RFC 0016): o lançador os procura no PATH, e o `Ctrl+L` sabe que eles
//! aceitam colagem entre marcadores (*bracketed paste*).

/// O binário (que é também o comando) e o nome mostrado.
pub const KNOWN: &[(&str, &str)] = &[
    ("claude", "Claude Code"),
    ("codex", "Codex"),
    ("gemini", "Gemini CLI"),
    ("opencode", "OpenCode"),
    ("aider", "Aider"),
    ("cursor-agent", "Cursor Agent"),
];

/// O comando abre um agente conhecido (pela primeira palavra, sem a pasta e sem `.exe`).
pub fn is_known(command: &str) -> bool {
    let first = command.split_whitespace().next().unwrap_or_default();
    // As duas barras: o comando pode ser de Windows num config lido em outro sistema.
    let file = first.rsplit(['/', '\\']).next().unwrap_or_default();
    let bin = file.strip_suffix(".exe").or_else(|| file.strip_suffix(".EXE")).unwrap_or(file);
    KNOWN.iter().any(|(b, _)| *b == bin)
}

/// O binário de um comando (a primeira palavra, sem a pasta e sem `.exe`).
fn binary(command: &str) -> &str {
    let first = command.split_whitespace().next().unwrap_or_default();
    let file = first.rsplit(['/', '\\']).next().unwrap_or_default();
    file.strip_suffix(".exe").or_else(|| file.strip_suffix(".EXE")).unwrap_or(file)
}

/// O comando já diz qual conversa abrir (ou roda sem conversa): a IDE não põe outra.
fn picks_conversation(command: &str) -> bool {
    command.split_whitespace().skip(1).any(|w| {
        matches!(w, "-r" | "-c" | "-p" | "--resume" | "--continue" | "--print" | "--session-id" | "resume")
            || w.starts_with("--resume=")
            || w.starts_with("--session-id=")
    })
}

/// Abre o agente numa conversa com id escolhido pela IDE, para depois retomar exatamente ela (RFC 0024, M2). Só o
/// Claude Code aceita isso (`--session-id <uuid>`); para os outros, `None` e o comando fica como está.
pub fn with_new_session(command: &str) -> Option<(String, String)> {
    if binary(command) != "claude" || picks_conversation(command) {
        return None;
    }
    let id = uuid::Uuid::new_v4().to_string();
    Some((format!("{} --session-id {id}", command.trim()), id))
}

/// O comando que retoma a conversa de um agente aberto com `command` (na mesma pasta). `session`: o id que a IDE
/// escolheu, quando escolheu. `None`: o agente não tem como retomar; abre uma conversa nova.
pub fn resume_command(command: &str, session: Option<&str>) -> Option<String> {
    let command = command.trim();
    match binary(command) {
        "claude" => {
            // Tira o `--session-id <id>` que a IDE pôs: o `--resume` é que diz a conversa.
            let mut words: Vec<&str> = Vec::new();
            let mut skip = false;
            for w in command.split_whitespace() {
                if skip {
                    skip = false;
                    continue;
                }
                if w == "--session-id" {
                    skip = true;
                    continue;
                }
                if w.starts_with("--session-id=") {
                    continue;
                }
                words.push(w);
            }
            let rest = words[1..].join(" ");
            let first = words[0];
            let how = match session {
                Some(id) => format!("--resume {id}"),
                // Sem id: a conversa mais recente nesta pasta.
                None => "--continue".to_string(),
            };
            Some(if rest.is_empty() { format!("{first} {how}") } else { format!("{first} {how} {rest}") })
        }
        // O `codex resume --last` pega a conversa mais recente desta pasta (filtra pela pasta).
        "codex" => {
            let first = command.split_whitespace().next().unwrap_or("codex");
            Some(format!("{first} resume --last"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_gets_a_session_id_and_resumes_it() {
        let (cmd, id) = with_new_session("claude --model opus").unwrap();
        assert_eq!(cmd, format!("claude --model opus --session-id {id}"));
        assert_eq!(uuid::Uuid::parse_str(&id).map(|u| u.get_version_num()), Ok(4));
        assert_eq!(resume_command(&cmd, Some(&id)).unwrap(), format!("claude --resume {id} --model opus"));
        assert_eq!(resume_command("claude", None).unwrap(), "claude --continue");
        assert_eq!(resume_command("/usr/bin/claude --session-id=abc", Some("abc")).unwrap(), "/usr/bin/claude --resume abc");
    }

    #[test]
    fn only_claude_without_a_chosen_conversation_gets_an_id() {
        assert!(with_new_session("codex").is_none());
        assert!(with_new_session("claude --resume x").is_none());
        assert!(with_new_session("claude -c").is_none());
        assert!(with_new_session("my-agent claude").is_none());
    }

    #[test]
    fn codex_resumes_the_last_one_here_and_others_cannot() {
        assert_eq!(resume_command("codex --model o3", None).unwrap(), "codex resume --last");
        assert_eq!(resume_command("gemini", None), None);
        assert_eq!(resume_command("aider --yes", None), None);
    }

    #[test]
    fn known_by_the_first_word() {
        assert!(is_known("claude --model opus"));
        assert!(is_known("/usr/local/bin/codex"));
        assert!(is_known(r"C:\Tools\gemini.exe -m x"));
        assert!(!is_known("my-agent claude"));
        assert!(!is_known(""));
    }
}
