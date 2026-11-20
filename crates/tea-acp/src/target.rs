//! Os dois alvos do marco 1: `opencode acp` e o adaptador do Claude Code.
//!
//! A detecção acontece antes de abrir a sessão. Faltando o binário, o Node 22
//! ou o arquivo configurado, a IDE mostra a mensagem e não inicia nada.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Pacote npm do adaptador oficial, preso nesta versão.
pub const CLAUDE_ADAPTER_PACKAGE: &str = "@agentclientprotocol/claude-agent-acp";
pub const CLAUDE_ADAPTER_VERSION: &str = "0.109.0";
/// Nome do binário (`bin` do package.json e dos arquivos de release).
pub const CLAUDE_ADAPTER_BIN: &str = "claude-agent-acp";
/// Node mínimo do pacote npm (`engines.node` é `>=22`).
pub const CLAUDE_NODE_MAJOR: u32 = 22;

/// Qual agente a pessoa pediu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// `opencode acp`, ACP nativo.
    OpenCode,
    /// Claude Code atrás de `claude-agent-acp`.
    Claude,
}

impl Target {
    pub fn bin_name(self) -> &'static str {
        match self {
            Target::OpenCode => "opencode",
            Target::Claude => CLAUDE_ADAPTER_BIN,
        }
    }
}

/// O que vai ser executado. O ambiente do processo é o da IDE (não limpamos
/// `HOME` nem `PATH`): o login do Claude Code que já existe continua visível.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub target: Target,
    pub program: PathBuf,
    pub args: Vec<String>,
}

impl Launch {
    pub fn open_code(program: PathBuf) -> Self {
        Self { target: Target::OpenCode, program, args: vec!["acp".into()] }
    }

    pub fn claude(program: PathBuf) -> Self {
        // Sem `--hide-claude-auth`: o adaptador continua oferecendo a assinatura e o Console.
        Self { target: Target::Claude, program, args: Vec::new() }
    }
}

/// Por que não deu para iniciar. A interface traduz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Missing {
    /// `opencode` não está no PATH.
    OpenCode,
    /// Nem o binário, nem um arquivo configurado, e o Node não serve.
    Claude { node: NodeReport },
}

/// O que achamos do `node`, para a mensagem do Claude.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeReport {
    /// Não há `node` no PATH.
    Absent,
    /// Há `node`, mas a versão é anterior à 22.
    Old { version: String },
    /// Node 22 ou mais novo, mas o adaptador não está instalado.
    NewEnough { version: String },
    /// `node --version` não devolveu uma versão legível.
    Unreadable { output: String },
}

/// O que a detecção precisa do sistema, para os testes não dependerem do PATH real.
pub struct Probe<'a> {
    pub find: &'a dyn Fn(&str) -> Option<PathBuf>,
    /// Saída de `node --version`, se o binário existe. `None` se não há `node`.
    pub node_version: Option<&'a str>,
    /// `[agents] claude_acp` do config do usuário: arquivo standalone, sem Node.
    pub claude_bin: Option<&'a Path>,
}

pub fn resolve(target: Target, probe: &Probe<'_>) -> Result<Launch, Missing> {
    match target {
        Target::OpenCode => match (probe.find)("opencode") {
            Some(program) => Ok(Launch::open_code(program)),
            None => Err(Missing::OpenCode),
        },
        Target::Claude => resolve_claude(probe),
    }
}

fn resolve_claude(probe: &Probe<'_>) -> Result<Launch, Missing> {
    if let Some(path) = probe.claude_bin
        && path.is_file()
    {
        return Ok(Launch::claude(path.to_path_buf()));
    }
    if let Some(program) = (probe.find)(CLAUDE_ADAPTER_BIN) {
        return Ok(Launch::claude(program));
    }
    Err(Missing::Claude { node: node_report(probe.node_version) })
}

pub fn node_report(version_output: Option<&str>) -> NodeReport {
    let Some(raw) = version_output else { return NodeReport::Absent };
    let text = raw.trim();
    if text.is_empty() {
        return NodeReport::Unreadable { output: raw.trim().to_string() };
    }
    match parse_node_version(text) {
        Some((major, _, _)) if major >= CLAUDE_NODE_MAJOR => NodeReport::NewEnough { version: text.to_string() },
        Some(_) => NodeReport::Old { version: text.to_string() },
        None => NodeReport::Unreadable { output: text.to_string() },
    }
}

/// `v22.11.0`, `v22.0.0-pre`, `22.1.0`.
pub fn parse_node_version(text: &str) -> Option<(u32, u32, u32)> {
    let text = text.trim().trim_start_matches('v').trim_start_matches('V');
    let mut parts = text.split(['.', '-', '+']);
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

/// Procura no PATH de verdade e roda `node --version` se ele existir.
pub fn system_probe(claude_bin: Option<&Path>) -> ResolvedProbe {
    let node = tea_core_find("node").and_then(|_| read_node_version());
    ResolvedProbe { claude_bin: claude_bin.map(Path::to_path_buf), node }
}

/// Resultado já lido do sistema, para montar um [`Probe`] sem guardar o PATH.
pub struct ResolvedProbe {
    pub claude_bin: Option<PathBuf>,
    pub node: Option<String>,
}

impl ResolvedProbe {
    pub fn resolve(&self, target: Target) -> Result<Launch, Missing> {
        let find = |name: &str| find_in_path(name);
        resolve(target, &Probe { find: &find, node_version: self.node.as_deref(), claude_bin: self.claude_bin.as_deref() })
    }
}

fn tea_core_find(name: &str) -> Option<PathBuf> {
    find_in_path(name)
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    // A IDE TEA já tem isto em `tea_core::os`; o cliente não depende do núcleo
    // para os testes do protocolo rodarem sozinhos. A regra é a mesma: arquivo
    // executável no PATH.
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        if Path::new(name).extension().is_none() {
            let exts = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".into());
            for ext in exts.split(';').filter(|e| !e.is_empty()) {
                let with_ext = dir.join(format!("{name}{ext}"));
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn read_node_version() -> Option<String> {
    let output = Command::new("node").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Comando de login no terminal: o programa do agente mais os `args` do método
/// `terminal` (o adaptador do Claude rejeita `authenticate` para esses ids).
pub fn login_command(program: &Path, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(1 + args.len());
    parts.push(program.display().to_string());
    parts.extend(args.iter().cloned());
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_bins(name: &str) -> Option<PathBuf> {
        let _ = name;
        None
    }

    #[test]
    fn node_versions() {
        assert_eq!(parse_node_version("v22.11.0"), Some((22, 11, 0)));
        assert_eq!(parse_node_version("v18.20.4\n"), Some((18, 20, 4)));
        assert_eq!(parse_node_version("v22.0.0-nightly"), Some((22, 0, 0)));
        assert!(parse_node_version("not-a-version").is_none());
        assert!(matches!(node_report(None), NodeReport::Absent));
        assert!(matches!(node_report(Some("v20.1.0")), NodeReport::Old { .. }));
        assert!(matches!(node_report(Some("v22.0.0")), NodeReport::NewEnough { .. }));
        assert!(matches!(node_report(Some("")), NodeReport::Unreadable { .. }));
    }

    #[test]
    fn missing_tools_do_not_invent_a_launch() {
        let probe = Probe { find: &no_bins, node_version: None, claude_bin: None };
        assert!(matches!(resolve(Target::OpenCode, &probe), Err(Missing::OpenCode)));
        assert!(matches!(
            resolve(Target::Claude, &probe),
            Err(Missing::Claude { node: NodeReport::Absent })
        ));
        let probe = Probe { find: &no_bins, node_version: Some("v20.11.0"), claude_bin: None };
        assert!(matches!(
            resolve(Target::Claude, &probe),
            Err(Missing::Claude { node: NodeReport::Old { .. } })
        ));
        let probe = Probe { find: &no_bins, node_version: Some("v22.1.0"), claude_bin: None };
        assert!(matches!(
            resolve(Target::Claude, &probe),
            Err(Missing::Claude { node: NodeReport::NewEnough { .. } })
        ));
    }

    #[test]
    fn configured_file_and_path_hit() {
        let dir = std::env::temp_dir().join(format!("tea-acp-target-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("claude-agent-acp");
        std::fs::write(&bin, b"#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&bin).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&bin, perms).unwrap();
        }
        let probe = Probe { find: &no_bins, node_version: None, claude_bin: Some(&bin) };
        let launch = resolve(Target::Claude, &probe).unwrap();
        assert_eq!(launch.program, bin);
        assert!(launch.args.is_empty(), "não passamos --hide-claude-auth");

        let opencode = dir.join("opencode");
        std::fs::write(&opencode, b"").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&opencode).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&opencode, perms).unwrap();
        }
        let find = |name: &str| (name == "opencode").then(|| opencode.clone());
        let probe = Probe { find: &find, node_version: None, claude_bin: None };
        let launch = resolve(Target::OpenCode, &probe).unwrap();
        assert_eq!(launch.args, ["acp"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn login_command_appends_terminal_args() {
        let cmd = login_command(Path::new("claude-agent-acp"), &["--cli".into(), "auth".into(), "login".into(), "--claudeai".into()]);
        assert_eq!(cmd, "claude-agent-acp --cli auth login --claudeai");
    }
}
