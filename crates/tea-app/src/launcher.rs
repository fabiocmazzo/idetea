//! "Agentes: iniciar…" (`Ctrl+Shift+A`, RFC 0016): abre um agente de código numa aba de terminal, na pasta do projeto
//! ou numa worktree nova. A lista junta os agentes conhecidos que estão no PATH com os lançadores do config do usuário
//! (`[agents.launchers]`); `[agents] default` vem primeiro. Quem abre o terminal é a extensão do terminal
//! (`CoreEvent::LaunchAgent`).
//!
//! Os lançadores só valem no config do usuário, como `[agents] allow`: o do projeto viria do repositório de outra
//! pessoa e abriria um comando qualquer com um atalho inocente.

use std::collections::BTreeMap;
use std::path::Path;

/// Agentes conhecidos: o binário (que é também o comando) e o nome mostrado.
pub const KNOWN: &[(&str, &str)] = &[
    ("claude", "Claude Code"),
    ("codex", "Codex"),
    ("gemini", "Gemini CLI"),
    ("opencode", "OpenCode"),
    ("aider", "Aider"),
    ("cursor-agent", "Cursor Agent"),
];

/// Um agente para abrir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launcher {
    /// Nome mostrado e da aba.
    pub name: String,
    pub command: String,
    /// O binário de um agente conhecido, ou o nome do lançador do config (para `[agents] default`).
    key: String,
}

/// O que a pessoa escolheu na lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// Na pasta do projeto.
    Here(Launcher),
    /// Numa worktree nova: falta o nome do branch.
    AskBranch(Launcher),
    /// Numa worktree com este branch.
    Worktree(Launcher, String),
}

impl Choice {
    pub fn launcher(&self) -> &Launcher {
        match self {
            Choice::Here(l) | Choice::AskBranch(l) | Choice::Worktree(l, _) => l,
        }
    }
}

/// Os agentes para a lista: os do config (na ordem do nome) e os conhecidos achados por `found`, com o `default`
/// primeiro. Um lançador do config com o mesmo nome de um conhecido fica no lugar dele.
pub fn list(launchers: &BTreeMap<String, String>, default: Option<&str>, found: impl Fn(&str) -> bool) -> Vec<Launcher> {
    let mut out: Vec<Launcher> = KNOWN
        .iter()
        .filter(|(bin, name)| found(bin) && !launchers.contains_key(*name))
        .map(|(bin, name)| Launcher { name: (*name).into(), command: (*bin).into(), key: (*bin).into() })
        .collect();
    out.extend(
        launchers
            .iter()
            .filter(|(_, cmd)| !cmd.trim().is_empty())
            .map(|(name, cmd)| Launcher { name: name.clone(), command: cmd.trim().into(), key: name.clone() }),
    );
    if let Some(d) = default
        && let Some(i) = out.iter().position(|l| l.key == d || l.name == d)
    {
        out[..=i].rotate_right(1);
    }
    out
}

/// O agente está no PATH (o mesmo PATH que o terminal vai herdar; no Windows, com as extensões do `PATHEXT`).
pub fn in_path(bin: &str) -> bool {
    tea_core::os::find_in_path(bin).is_some()
}

/// Marca do "não perguntar neste projeto" sobre as instruções dos agentes.
pub fn skip_teach_path(root: &Path) -> std::path::PathBuf {
    root.join(".idetea").join("launcher-skip-teach")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[Launcher]) -> Vec<&str> {
        list.iter().map(|l| l.name.as_str()).collect()
    }

    #[test]
    fn only_known_agents_in_the_path() {
        let list = list(&BTreeMap::new(), None, |bin| bin == "codex" || bin == "claude");
        assert_eq!(names(&list), ["Claude Code", "Codex"]);
        assert_eq!(list[1].command, "codex");
    }

    #[test]
    fn config_launchers_come_after_and_replace_a_known_one_with_the_same_name() {
        let config = BTreeMap::from([
            ("Codex".to_string(), "codex --profile fast".to_string()),
            ("Claude (Opus)".to_string(), "claude --model opus".to_string()),
            ("vazio".to_string(), "  ".to_string()),
        ]);
        let list = list(&config, None, |_| true);
        assert_eq!(
            names(&list),
            ["Claude Code", "Gemini CLI", "OpenCode", "Aider", "Cursor Agent", "Claude (Opus)", "Codex"]
        );
        assert_eq!(list.last().unwrap().command, "codex --profile fast");
    }

    #[test]
    fn the_default_comes_first_by_binary_or_name() {
        let config = BTreeMap::from([("Claude (Opus)".to_string(), "claude --model opus".to_string())]);
        let found = |bin: &str| bin == "claude" || bin == "codex";
        assert_eq!(names(&list(&config, Some("codex"), found)), ["Codex", "Claude Code", "Claude (Opus)"]);
        assert_eq!(names(&list(&config, Some("Claude (Opus)"), found)), ["Claude (Opus)", "Claude Code", "Codex"]);
        assert_eq!(names(&list(&config, Some("não existe"), found)), ["Claude Code", "Codex", "Claude (Opus)"]);
    }
}
