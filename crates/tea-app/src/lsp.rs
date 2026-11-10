//! Servidores de linguagem: um por linguagem, iniciado no primeiro arquivo que precisar.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tea_core::config::LspConfig;
use tea_lsp::{Client, EventSink, ServerConfig};

pub struct LspManager {
    enabled: bool,
    servers: Vec<ServerConfig>,
    clients: HashMap<String, Client>,
    /// Servidores que falharam (no início ou com quedas demais): não tentamos de novo sozinhos (evita loop).
    /// `lsp.restart` limpa.
    failed: HashMap<String, String>,
    /// Servidores que caíram depois de prontos: religam na próxima edição (ver [`LspManager::on_exit`]).
    crashed: HashMap<String, Vec<Instant>>,
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
            crashed: HashMap::new(),
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
        let server = ziex_fallback(server, &self.root);
        if tea_lsp::find_in_path(&server.command).is_none() {
            let msg = fl!("lsp-command-not-found", server = server.name.clone(), command = server.command.clone());
            let hinted = with_hint(&server.name, &msg);
            self.failed.insert(server.name.clone(), msg);
            return Err(hinted);
        }
        match Client::start(&server, &self.root, self.sink.clone()) {
            Ok(client) => {
                self.clients.insert(server.name.clone(), client.clone());
                Ok(Some((client, language)))
            }
            Err(e) => {
                let msg = fl!("lsp-start-error", server = server.name.clone(), error = e.to_string());
                let hinted = with_hint(&server.name, &msg);
                self.failed.insert(server.name.clone(), msg);
                Err(hinted)
            }
        }
    }

    /// Um servidor rodando, qualquer um (para buscas no projeto quando a aba ativa não tem servidor).
    pub fn any_running(&self) -> Option<Client> {
        self.clients.values().find(|c| c.is_alive()).cloned()
    }

    /// Por que um arquivo está sem servidor de linguagem (para avisar quando o usuário pede algo dele).
    pub fn why_missing(&self, path: &Path) -> String {
        if !self.enabled {
            return fl!("lsp-disabled");
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        let Some(server) = self.servers.iter().find(|s| s.language_for(ext).is_some()) else {
            return fl!("lsp-no-server-for", ext = ext);
        };
        match self.failed.get(&server.name) {
            Some(reason) => fl!("lsp-not-running", server = server.name.clone(), reason = with_hint(&server.name, reason)),
            None => fl!("lsp-not-ready", server = server.name.clone()),
        }
    }

    /// Um processo de servidor terminou. Só conta se for o cliente atual desse servidor: o antigo, trocado por
    /// `lsp.restart`, também avisa quando sai. Caiu depois de pronto: religa quando a digitação pausar (e não na
    /// hora, que reabriria o mesmo texto que o derrubou), até [`MAX_CRASHES`] vezes em [`CRASH_WINDOW`]. Falha no
    /// início ou quedas demais: fica parado até `lsp.restart`.
    pub fn on_exit(&mut self, server: &str, client: u64, reason: &str) -> Exit {
        self.on_exit_at(server, client, reason, Instant::now())
    }

    fn on_exit_at(&mut self, server: &str, client: u64, reason: &str, now: Instant) -> Exit {
        let Some(current) = self.clients.get(server).filter(|c| c.id() == client) else { return Exit::Stale };
        let ready = current.is_ready();
        self.clients.remove(server);
        if ready {
            let crashes = self.crashed.entry(server.to_string()).or_default();
            if count_crash(crashes, now) {
                return Exit::WillRestart { first: crashes.len() == 1 };
            }
            self.crashed.remove(server);
            self.failed.insert(server.to_string(), reason.to_string());
            return Exit::TooManyCrashes;
        }
        self.failed.insert(server.to_string(), reason.to_string());
        Exit::Failed
    }

    /// O servidor do arquivo caiu e espera uma edição para voltar.
    pub fn awaits_restart(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        self.servers
            .iter()
            .find(|s| s.language_for(ext).is_some())
            .is_some_and(|s| self.crashed.contains_key(&s.name) && !self.clients.contains_key(&s.name))
    }

    /// Encerra tudo e esquece falhas. Os editores devem ser religados depois.
    pub fn restart(&mut self) {
        self.shutdown_all();
        self.clients.clear();
        self.failed.clear();
        self.crashed.clear();
    }

    pub fn shutdown_all(&self) {
        for client in self.clients.values() {
            client.shutdown();
        }
    }
}

/// Servidores embutidos + mudanças do config (por nome).
/// Acrescenta como resolver falhas conhecidas.
/// Quedas que ainda religam sozinhas dentro de [`CRASH_WINDOW`]. Cada religação espera uma edição e a pausa na
/// digitação, então não há laço sem o usuário; o teto só segura um servidor quebrado de vez.
pub const MAX_CRASHES: usize = 10;
pub const CRASH_WINDOW: Duration = Duration::from_secs(5 * 60);

/// O que fazer depois que um servidor terminou.
#[derive(Debug, PartialEq, Eq)]
pub enum Exit {
    /// Era um cliente antigo (já trocado): nada a fazer.
    Stale,
    /// Caiu depois de pronto: as abas religam quando a digitação pausar. `first`: primeira queda da janela (só
    /// ela é avisada).
    WillRestart { first: bool },
    /// Caiu mais de [`MAX_CRASHES`] vezes em pouco tempo: parado até `lsp.restart`.
    TooManyCrashes,
    /// Não chegou a ficar pronto: parado até `lsp.restart`.
    Failed,
}

/// Anota uma queda em `now`. `true`: ainda cabe religar sozinho.
fn count_crash(crashes: &mut Vec<Instant>, now: Instant) -> bool {
    crashes.retain(|t| now.duration_since(*t) < CRASH_WINDOW);
    crashes.push(now);
    crashes.len() <= MAX_CRASHES
}

pub fn with_hint(server: &str, reason: &str) -> String {
    // O rustup instala um atalho `rust-analyzer` que só funciona com o componente instalado.
    if reason.contains("Unknown binary") && reason.contains("rust-analyzer") {
        return fl!("lsp-hint-rustup-install", reason = reason);
    }
    if server == "rust-analyzer" && tea_lsp::find_in_path("rust-analyzer").is_none() {
        return fl!("lsp-hint-rustup", reason = reason);
    }
    if server == "zxls" && tea_lsp::find_in_path("zx").is_none() {
        return fl!("lsp-hint-ziex", reason = reason);
    }
    // Language packs: o README de cada pack fica dentro do binário, então a dica de instalação vem aqui.
    if server == "moonbit" && tea_lsp::find_in_path("moon").is_none() {
        return fl!("lsp-hint-moonbit", reason = reason);
    }
    if server == "hare-lsp" && tea_lsp::find_in_path("hare-lsp").is_none() {
        return fl!("lsp-hint-hare", reason = reason);
    }
    reason.to_string()
}

/// Sem o CLI `zx` no PATH, um projeto Ziex compila o próprio servidor, como fazem as extensões do Ziex para o
/// Zed e o VS Code: `zig build zx -Dziex-lsp=true --release=fast -- lsp` na raiz. Só para o `zxls` padrão (um
/// comando escrito no config vale como está).
pub fn ziex_fallback(server: ServerConfig, root: &Path) -> ServerConfig {
    if server.name != "zxls" || server.command != "zx" || tea_lsp::find_in_path("zx").is_some() || !depends_on_ziex(root) {
        return server;
    }
    let args = ["build", "zx", "-Dziex-lsp=true", "--release=fast", "--", "lsp"];
    ServerConfig { command: "zig".into(), args: args.map(String::from).to_vec(), ..server }
}

/// O `build.zig.zon` da raiz depende do `ziex`.
fn depends_on_ziex(root: &Path) -> bool {
    std::fs::read_to_string(root.join("build.zig.zon"))
        .is_ok_and(|zon| zon.chars().filter(|c| !c.is_whitespace()).collect::<String>().contains(".ziex=.{"))
}

pub fn servers_from_config(config: &LspConfig) -> Vec<ServerConfig> {
    let mut servers = tea_lsp::default_servers();
    let (packs, _) = tea_core::languages::builtins();
    for pack in packs {
        let manifest = pack.manifest;
        let Some(lsp) = manifest.lsp else { continue };
        servers.push(ServerConfig {
            name: lsp.name,
            command: lsp.command,
            args: lsp.args,
            languages: manifest.files.extensions.into_iter().map(|(e, id)| (e.to_lowercase(), id)).collect(),
        });
    }
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
    fn language_pack_servers_are_included() {
        let servers = servers_from_config(&cfg(&[]));
        let moonbit = servers.iter().find(|s| s.name == "moonbit").unwrap();
        assert_eq!(moonbit.command, "moon");
        assert_eq!(moonbit.args, ["lsp"]);
        assert_eq!(moonbit.language_for("mbt"), Some("moonbit"));
        assert_eq!(moonbit.language_for("mbti"), Some("moonbit"));
        let hare = servers.iter().find(|s| s.name == "hare-lsp").unwrap();
        assert_eq!(hare.command, "hare-lsp");
        assert_eq!(hare.args, ["-S"]);
        assert_eq!(hare.language_for("ha"), Some("hare"));
    }

    #[test]
    fn pack_server_can_be_disabled() {
        let servers = servers_from_config(&cfg(&[("moonbit", LspServerConfig { enabled: Some(false), ..Default::default() })]));
        assert!(!servers.iter().any(|s| s.name == "moonbit"));
        assert!(servers.iter().any(|s| s.name == "hare-lsp"));
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
    use super::{ServerConfig, with_hint, ziex_fallback};

    #[test]
    fn rustup_proxy_without_component_gets_install_hint() {
        let r = "error: Unknown binary 'rust-analyzer' in official toolchain 'stable-x86_64-unknown-linux-gnu'.";
        assert!(with_hint("rust-analyzer", r).ends_with("rustup component add rust-analyzer"));
        assert_eq!(with_hint("clangd", "saiu"), "saiu");
    }

    #[test]
    fn language_pack_servers_get_install_hints_when_missing() {
        if tea_lsp::find_in_path("moon").is_none() {
            assert!(with_hint("moonbit", "x").contains("moonbitlang.com"));
        }
        if tea_lsp::find_in_path("hare-lsp").is_none() {
            assert!(with_hint("hare-lsp", "x").contains("git.sr.ht/~whynothugo/hare-lsp"));
        }
    }

    #[test]
    fn ziex_project_without_the_cli_builds_its_own_server() {
        let dir = std::env::temp_dir().join(format!("idetea-ziex-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let zxls = || tea_lsp::default_servers().into_iter().find(|s| s.name == "zxls").unwrap();
        assert_eq!(zxls().language_for("zx"), Some("zx"));
        // Sem `ziex` no build.zig.zon: fica o `zx lsp`.
        std::fs::write(dir.join("build.zig.zon"), ".{ .name = .app, .dependencies = .{} }").unwrap();
        assert_eq!(ziex_fallback(zxls(), &dir).command, "zx");
        std::fs::write(
            dir.join("build.zig.zon"),
            ".{\n    .name = .app,\n    .dependencies = .{\n        .ziex = .{ .url = \"x\", .hash = \"y\" },\n    },\n}",
        )
        .unwrap();
        let s = ziex_fallback(zxls(), &dir);
        if tea_lsp::find_in_path("zx").is_none() {
            assert_eq!(s.command, "zig");
            assert_eq!(s.args, ["build", "zx", "-Dziex-lsp=true", "--release=fast", "--", "lsp"]);
        }
        // Outro servidor, ou um comando do config: como está.
        let custom = ServerConfig { command: "/opt/zx".into(), ..zxls() };
        assert_eq!(ziex_fallback(custom, &dir).command, "/opt/zx");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod crash_tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use tea_core::config::LspConfig;
    use tea_lsp::{Client, ServerConfig};

    use super::{CRASH_WINDOW, Exit, LspManager, MAX_CRASHES, count_crash};

    #[test]
    fn crashes_restart_until_too_many_in_the_window() {
        let start = Instant::now();
        let mut crashes = Vec::new();
        for n in 0..MAX_CRASHES {
            assert!(count_crash(&mut crashes, start + Duration::from_secs(n as u64)), "queda {n}");
        }
        assert!(!count_crash(&mut crashes, start + Duration::from_secs(10)));
        // Passada a janela, as quedas antigas não contam mais.
        let mut crashes = vec![start; MAX_CRASHES];
        assert!(count_crash(&mut crashes, start + CRASH_WINDOW));
        assert_eq!(crashes.len(), 1);
    }

    #[test]
    fn exit_of_an_old_client_is_ignored_and_a_failed_start_stays_failed() {
        let root = std::env::temp_dir();
        let config = LspConfig { enabled: true, servers: Default::default() };
        let mut manager = LspManager::new(&root, &config, Arc::new(|_| {}));
        // O próprio binário de teste faz as vezes do servidor: lista os testes e sai, sem handshake.
        let exe = std::env::current_exe().unwrap();
        let server = ServerConfig {
            name: "falso".into(),
            command: exe.to_string_lossy().into_owned(),
            args: vec!["--list".into()],
            languages: vec![("falso".into(), "falso".into())],
        };
        let client = Client::start(&server, &root, Arc::new(|_| {})).unwrap();
        manager.clients.insert("falso".into(), client.clone());

        assert_eq!(manager.on_exit("falso", client.id() + 1000, "saiu"), Exit::Stale);
        assert!(manager.clients.contains_key("falso"), "o cliente atual continua");
        assert_eq!(manager.on_exit("falso", client.id(), "saiu"), Exit::Failed);
        assert!(!manager.clients.contains_key("falso") && manager.failed.contains_key("falso"));
        assert_eq!(manager.on_exit("falso", client.id(), "saiu"), Exit::Stale, "aviso repetido não conta");
        client.kill_now();
    }
}

