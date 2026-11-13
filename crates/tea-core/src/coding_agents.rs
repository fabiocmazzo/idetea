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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_by_the_first_word() {
        assert!(is_known("claude --model opus"));
        assert!(is_known("/usr/local/bin/codex"));
        assert!(is_known(r"C:\Tools\gemini.exe -m x"));
        assert!(!is_known("my-agent claude"));
        assert!(!is_known(""));
    }
}
