//! "Agentes: iniciar…" (`Ctrl+Shift+A`, RFC 0016): abre um agente de código numa aba de terminal, na pasta do projeto
//! ou numa worktree nova. A lista junta os agentes conhecidos que estão no PATH com os lançadores do config do usuário
//! (`[agents.launchers]`); `[agents] default` vem primeiro. Quem abre o terminal é a extensão do terminal
//! (`CoreEvent::LaunchAgent`).
//!
//! Os lançadores só valem no config do usuário, como `[agents] allow`: o do projeto viria do repositório de outra
//! pessoa e abriria um comando qualquer com um atalho inocente.

use std::collections::BTreeMap;
use std::path::Path;

pub use tea_core::coding_agents::KNOWN;

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

/// `idetea agent [nome] [--worktree <branch>]` (`--list` mostra os que há): pede à IDE uma aba com o agente, como o
/// `Ctrl+Shift+A`. Vai pelo pedido de terminal (`idetea run --no-wait`), então a IDE mostra o comando e pergunta, a
/// não ser que ele esteja no `[agents] allow`: um agente pode chamar isto também. Imprime o nome do pedido.
pub fn run_cli(args: &[String], cwd: &Path) -> i32 {
    let config = tea_core::config::global_config_path()
        .map(|g| tea_core::config::Config::load(Some(&g), None).agents)
        .unwrap_or_default();
    let agents = list(&config.launchers, config.default.as_deref(), in_path);
    let mut wanted = None;
    let mut worktree = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--list" => {
                for l in &agents {
                    println!("{}\t{}\t{}", l.key, l.name, l.command);
                }
                return 0;
            }
            "--worktree" => match it.next() {
                Some(b) => worktree = Some(b.clone()),
                None => return usage_error(arg),
            },
            "-h" | "--help" => {
                println!("{}", fl!("agent-usage"));
                return 0;
            }
            a if a.starts_with('-') || wanted.is_some() => return usage_error(a),
            a => wanted = Some(a.to_string()),
        }
    }
    let found = match &wanted {
        Some(w) => agents.iter().find(|l| l.key.eq_ignore_ascii_case(w) || l.name.eq_ignore_ascii_case(w)),
        None => agents.first(),
    };
    let Some(agent) = found else {
        eprintln!("{}", fl!("agent-not-found", name = wanted.unwrap_or_default()));
        return 2;
    };
    let title = match &worktree {
        Some(b) => format!("{} · {b}", agent.name),
        None => agent.name.clone(),
    };
    let mut run = vec!["--no-wait".to_string(), "--title".into(), title, "--source".into(), "idetea agent".into()];
    if let Some(b) = worktree {
        run.extend(["--worktree".into(), b]);
    }
    run.extend(["--".into(), agent.command.clone()]);
    run_request(&run, cwd)
}

fn usage_error(arg: &str) -> i32 {
    eprintln!("{}", fl!("agent-bad-arg", arg = arg.to_string()));
    eprintln!("{}", fl!("agent-usage"));
    2
}

#[cfg(feature = "ext-terminal")]
fn run_request(args: &[String], cwd: &Path) -> i32 {
    tea_ext_terminal::cli::run(args, cwd)
}

#[cfg(not(feature = "ext-terminal"))]
fn run_request(_args: &[String], _cwd: &Path) -> i32 {
    eprintln!("idetea: compilada sem a extensão do terminal (feature ext-terminal)");
    2
}

/// Acima disto, `Ctrl+L` manda só o lugar da seleção, sem o trecho.
pub const MAX_SNIPPET: usize = 32 * 1024;

/// Um diagnóstico para mandar junto: linha e coluna começando em 1.
pub struct Problem<'a> {
    pub line: usize,
    pub column: usize,
    pub severity: &'a str,
    pub message: &'a str,
    pub source: Option<&'a str>,
}

/// O que `Ctrl+L` manda ao agente: o lugar (`caminho:linha` ou `caminho:primeira-última`), o trecho selecionado num
/// bloco de código e os diagnósticos dessas linhas no formato dos compiladores (`caminho:linha:coluna: erro: …`).
/// `path` é relativo ao projeto (`None` num arquivo sem nome). `selection`: primeira e última linha e o texto.
pub fn context_text(path: Option<&str>, line: usize, selection: Option<(usize, usize, &str)>, problems: &[Problem]) -> String {
    let place = |l: &str| path.map(|p| format!("{p}:{l}"));
    let mut out = String::new();
    match selection {
        Some((first, last, text)) => {
            let lines = if first == last { first.to_string() } else { format!("{first}-{last}") };
            if let Some(p) = place(&lines) {
                out.push_str(&p);
                out.push('\n');
            }
            if text.len() <= MAX_SNIPPET {
                let fence = if text.contains("```") { "````" } else { "```" };
                let lang = path.and_then(|p| Path::new(p).extension()).map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
                out.push_str(&format!("{fence}{lang}\n{}\n{fence}\n", text.trim_end_matches('\n')));
            }
        }
        None => {
            if let Some(p) = place(&line.to_string()) {
                out.push_str(&p);
                out.push('\n');
            }
        }
    }
    for d in problems {
        let at = path.map(|p| format!("{p}:")).unwrap_or_default();
        let source = d.source.map(|s| format!(" ({s})")).unwrap_or_default();
        let message = d.message.lines().next().unwrap_or_default();
        out.push_str(&format!("{at}{}:{}: {}: {message}{source}\n", d.line, d.column, d.severity));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_with_a_selection_has_the_place_the_snippet_and_the_problems() {
        let problems = [Problem { line: 12, column: 5, severity: "error", message: "mismatched types\nmais", source: Some("rustc") }];
        let text = context_text(Some("src/main.rs"), 12, Some((10, 14, "fn main() {\n}\n")), &problems);
        assert_eq!(text, "src/main.rs:10-14\n```rs\nfn main() {\n}\n```\nsrc/main.rs:12:5: error: mismatched types (rustc)\n");
    }

    #[test]
    fn context_without_a_selection_is_the_place_and_one_line_selection_has_no_range() {
        assert_eq!(context_text(Some("a.py"), 7, None, &[]), "a.py:7\n");
        assert_eq!(context_text(Some("a.md"), 3, Some((3, 3, "x ``` y")), &[]), "a.md:3\n````md\nx ``` y\n````\n");
        assert_eq!(context_text(None, 1, Some((1, 1, "x")), &[]), "```\nx\n```\n");
        let big = "x".repeat(MAX_SNIPPET + 1);
        assert_eq!(context_text(Some("a"), 1, Some((1, 900, &big)), &[]), "a:1-900\n");
    }

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
