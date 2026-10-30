//! Servidores de linguagem: um por linguagem, iniciado no primeiro arquivo que precisar.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tea_core::config::LspConfig;
use tea_lsp::{Client, EventSink, ServerConfig};

pub struct LspManager {
    enabled: bool,
    servers: Vec<ServerConfig>,
    clients: HashMap<String, Client>,
    /// Servidores que falharam: não tentamos de novo sozinhos (evita loop). `lsp.restart` limpa.
    failed: HashMap<String, String>,
    root: PathBuf,
    sink: EventSink,
}

impl LspManager {
    pub fn new(root: &Path, config: &LspConfig, sink: EventSink) -> Self {
        Self {
            enabled: config.enabled,
            servers: servers_from_config(config),
            clients: HashMap::new(),
            failed: HashMap::new(),
            root: root.to_path_buf(),
            sink,
        }
    }

    /// Config recarregado: vale para servidores iniciados daqui em diante.
    pub fn update_config(&mut self, config: &LspConfig) {
        self.enabled = config.enabled;
        self.servers = servers_from_config(config);
    }

    /// Cliente e languageId para um arquivo, iniciando o servidor se preciso.
    /// `Err` traz uma mensagem para o usuário (só na primeira falha de cada servidor).
    pub fn client_for(&mut self, path: &Path) -> Result<Option<(Client, String)>, String> {
        if !self.enabled {
            return Ok(None);
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        let Some((server, language)) =
            self.servers.iter().find_map(|s| s.language_for(ext).map(|l| (s.clone(), l.to_string())))
        else {
            return Ok(None);
        };
        if self.failed.contains_key(&server.name) {
            return Ok(None);
        }
        if let Some(client) = self.clients.get(&server.name)
            && client.is_alive()
        {
            return Ok(Some((client.clone(), language)));
        }
        if tea_lsp::find_in_path(&server.command).is_none() {
            let msg = format!(
                "{}: comando \"{}\" não encontrado no PATH (instale ou ajuste [lsp.servers.{}] no config)",
                server.name, server.command, server.name
            );
            self.failed.insert(server.name.clone(), msg.clone());
            return Err(msg);
        }
        match Client::start(&server, &self.root, self.sink.clone()) {
            Ok(client) => {
                self.clients.insert(server.name.clone(), client.clone());
                Ok(Some((client, language)))
            }
            Err(e) => {
                let msg = format!("{}: não foi possível iniciar: {e}", server.name);
                self.failed.insert(server.name.clone(), msg.clone());
                Err(msg)
            }
        }
    }

    /// Por que um arquivo está sem servidor de linguagem (para avisar quando o usuário pede algo dele).
    pub fn why_missing(&self, path: &Path) -> String {
        if !self.enabled {
            return "LSP desligado no config ([lsp] enabled = false)".into();
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        let Some(server) = self.servers.iter().find(|s| s.language_for(ext).is_some()) else {
            return format!("nenhum servidor de linguagem configurado para arquivos .{ext}");
        };
        match self.failed.get(&server.name) {
            Some(reason) => format!("{} não está rodando: {}", server.name, with_hint(&server.name, reason)),
            None => format!("{} ainda não está pronto", server.name),
        }
    }

    pub fn on_exit(&mut self, server: &str, reason: &str) {
        self.clients.remove(server);
        self.failed.insert(server.to_string(), reason.to_string());
    }

    /// Encerra tudo e esquece falhas. Os editores devem ser religados depois.
    pub fn restart(&mut self) {
        self.shutdown_all();
        self.clients.clear();
        self.failed.clear();
    }

    pub fn shutdown_all(&self) {
        for client in self.clients.values() {
            client.shutdown();
        }
    }
}

/// Servidores embutidos + mudanças do config (por nome).
/// Acrescenta como resolver falhas conhecidas.
pub fn with_hint(server: &str, reason: &str) -> String {
    // O rustup instala um atalho `rust-analyzer` que só funciona com o componente instalado.
    if reason.contains("Unknown binary") && reason.contains("rust-analyzer") {
        return format!("{reason}. Instale com: rustup component add rust-analyzer");
    }
    if server == "rust-analyzer" && reason.contains("não encontrado no PATH") {
        return format!("{reason}. Com rustup: rustup component add rust-analyzer");
    }
    reason.to_string()
}

pub fn servers_from_config(config: &LspConfig) -> Vec<ServerConfig> {
    let mut servers = tea_lsp::default_servers();
    for (name, o) in &config.servers {
        if o.enabled == Some(false) {
            servers.retain(|s| &s.name != name);
            continue;
        }
        let i = match servers.iter().position(|s| &s.name == name) {
            Some(i) => i,
            None => {
                // Servidor novo: precisa ao menos de comando e linguagens.
                let (Some(command), Some(_)) = (&o.command, &o.languages) else { continue };
                servers.push(ServerConfig {
                    name: name.clone(),
                    command: command.clone(),
                    args: Vec::new(),
                    languages: Vec::new(),
                });
                servers.len() - 1
            }
        };
        let s = &mut servers[i];
        if let Some(c) = &o.command {
            s.command = c.clone();
        }
        if let Some(a) = &o.args {
            s.args = a.clone();
        }
        if let Some(l) = &o.languages {
            s.languages = l.iter().map(|(e, id)| (e.to_lowercase(), id.clone())).collect();
        }
    }
    // Servidores do config vêm primeiro: assim podem "roubar" uma extensão de um embutido.
    servers.sort_by_key(|s| !config.servers.contains_key(&s.name));
    servers
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tea_core::config::LspServerConfig;

    fn cfg(entries: &[(&str, LspServerConfig)]) -> LspConfig {
        LspConfig { enabled: true, servers: entries.iter().cloned().map(|(k, v)| (k.to_string(), v)).collect() }
    }

    #[test]
    fn overrides_disable_and_add() {
        let servers = servers_from_config(&cfg(&[
            ("clangd", LspServerConfig { enabled: Some(false), ..Default::default() }),
            ("rust-analyzer", LspServerConfig { command: Some("/opt/ra".into()), ..Default::default() }),
            (
                "zls",
                LspServerConfig {
                    command: Some("zls".into()),
                    languages: Some(BTreeMap::from([("zig".into(), "zig".into())])),
                    ..Default::default()
                },
            ),
            ("incompleto", LspServerConfig { command: Some("x".into()), ..Default::default() }),
        ]));
        assert!(!servers.iter().any(|s| s.name == "clangd"));
        assert!(!servers.iter().any(|s| s.name == "incompleto"));
        let ra = servers.iter().find(|s| s.name == "rust-analyzer").unwrap();
        assert_eq!(ra.command, "/opt/ra");
        assert_eq!(ra.language_for("rs"), Some("rust"), "mantém as linguagens padrão");
        assert_eq!(servers.iter().find(|s| s.name == "zls").unwrap().language_for("ZIG"), Some("zig"));
    }

    #[test]
    fn config_server_can_take_over_an_extension() {
        let servers = servers_from_config(&cfg(&[(
            "ccls",
            LspServerConfig {
                command: Some("ccls".into()),
                languages: Some(BTreeMap::from([("c".into(), "c".into())])),
                ..Default::default()
            },
        )]));
        let first_for_c = servers.iter().find(|s| s.language_for("c").is_some()).unwrap();
        assert_eq!(first_for_c.name, "ccls");
    }

    #[test]
    fn missing_command_is_reported_once() {
        let sink: EventSink = std::sync::Arc::new(|_| {});
        let config = cfg(&[(
            "fake",
            LspServerConfig {
                command: Some("idetea-comando-que-nao-existe".into()),
                languages: Some(BTreeMap::from([("fk".into(), "fake".into())])),
                ..Default::default()
            },
        )]);
        let mut m = LspManager::new(Path::new("/tmp"), &config, sink);
        assert!(m.client_for(Path::new("/tmp/a.fk")).is_err());
        assert!(matches!(m.client_for(Path::new("/tmp/b.fk")), Ok(None)), "não repete o aviso");
        assert!(matches!(m.client_for(Path::new("/tmp/sem-servidor.xyz")), Ok(None)));
    }
}

#[cfg(test)]
mod hint_tests {
    use super::with_hint;

    #[test]
    fn rustup_proxy_without_component_gets_install_hint() {
        let r = "error: Unknown binary 'rust-analyzer' in official toolchain 'stable-x86_64-unknown-linux-gnu'.";
        assert!(with_hint("rust-analyzer", r).ends_with("rustup component add rust-analyzer"));
        assert_eq!(with_hint("clangd", "saiu"), "saiu");
    }
}
