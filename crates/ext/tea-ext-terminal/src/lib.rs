//! Terminal integrado no painel inferior (alacritty via `iced_term`).
//!
//! O shell só é iniciado quando o painel é aberto pela primeira vez. Com o foco
//! no terminal, Ctrl+letra, F-teclas etc. vão para o shell; atalhos da IDE com
//! Ctrl+Shift continuam valendo (regra aplicada pelo app em todos os painéis).
//!
//! Tarefas (`CoreEvent::RunTask`) rodam num terminal só delas, ao lado do shell: não digitam por cima do que
//! estiver rodando nele. No fim, o código de saída aparece e o terminal espera o Enter para fechar; rodar outra
//! tarefa substitui a anterior. `Ctrl+C` para só o comando (o resto do script continua e mostra o código).
//!
//! Um agente pode pedir um terminal (`agents`): o pedido aparece inteiro no painel e só roda se o usuário permitir.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod agents;
pub mod cli;
mod worktree;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::widget::{button, column, container, keyed_column, row, scrollable, space, text};
use iced::{Color, Element, Fill, Font, Subscription, Task, Theme};
use iced_term::actions::Action;
use iced_term::settings::{BackendSettings, FontSettings, Settings, ThemeSettings};
use iced_term::{ColorPalette, Terminal, TerminalView};
use tea_core::icons::{self, Icon};
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("terminal");

/// Roda a tarefa (`$1`), mostra o fim (`$2` código `$3`) e espera o Enter. O `trap` faz o `Ctrl+C` parar só o
/// comando: o shell do script continua para mostrar o código. O comando vai como argumento (sem aspas para
/// escapar) e o `eval` deixa usar `&&`, pipes e variáveis.
#[cfg(unix)]
const TASK_SCRIPT: &str = r#"trap true INT
printf '\033[1m$ %s\033[0m\n' "$1"
( eval "$1" )
c=$?
printf '\n\033[2m%s%s%s\033[0m\n' "$2" "$c" "$3"
read _"#;

/// Como o `TASK_SCRIPT`, para o terminal de um agente; antes de mostrar o fim, grava o status `exited` com o código
/// (`$2` o arquivo, `$3` o pid da IDE) pelo mesmo caminho do temporário. `$4` código `$5` é o texto do fim.
#[cfg(unix)]
const AGENT_SCRIPT: &str = r#"trap true INT
printf '\033[1m$ %s\033[0m\n' "$1"
( eval "$1" )
c=$?
printf '{"format":"idetea-terminal-status","version":1,"state":"exited","ide":%s,"code":%s%s}\n' "$3" "$c" "$6" > "$2.tmp" && mv -f "$2.tmp" "$2"
printf '\n\033[2m%s%s%s\033[0m\n' "$4" "$c" "$5"
read _"#;

/// Equivalente do `TASK_SCRIPT` para Windows (PowerShell), lendo os parâmetros das variáveis `IDETEA_*`.
#[cfg(windows)]
const WINDOWS_TASK_SCRIPT: &str = r#"Write-Host "`e[1m$ $($env:IDETEA_CMD)`e[0m"
cmd.exe /C $env:IDETEA_CMD
$c = $LASTEXITCODE
Write-Host "`n`e[2m$($env:IDETEA_BEFORE)$c$($env:IDETEA_AFTER)`e[0m"
[void][Console]::ReadLine()"#;

/// Equivalente do `AGENT_SCRIPT` para Windows (PowerShell), gravando o status de forma atômica.
#[cfg(windows)]
const WINDOWS_AGENT_SCRIPT: &str = r#"Write-Host "`e[1m$ $($env:IDETEA_CMD)`e[0m"
cmd.exe /C $env:IDETEA_CMD
$c = $LASTEXITCODE
$json = "{`"format`":`"idetea-terminal-status`",`"version`":1,`"state`":`"exited`",`"ide`":$($env:IDETEA_PID),`"code`":$c$($env:IDETEA_TAIL)}`n"
[IO.File]::WriteAllText("$($env:IDETEA_STATUS).tmp", $json)
Move-Item -Force "$($env:IDETEA_STATUS).tmp" $env:IDETEA_STATUS
Write-Host "`n`e[2m$($env:IDETEA_BEFORE)$c$($env:IDETEA_AFTER)`e[0m"
[void][Console]::ReadLine()"#;

/// O shell interativo padrão do sistema operacional.
pub fn default_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "/bin/sh".into())
    }
    #[cfg(windows)]
    {
        tea_core::os::find_in_path("pwsh.exe")
            .or_else(|| tea_core::os::find_in_path("powershell.exe"))
            .map(|p| p.to_string_lossy().into_owned())
            .or_else(|| std::env::var("COMSPEC").ok().filter(|s| !s.is_empty()))
            .unwrap_or_else(|| "cmd.exe".into())
    }
}

/// De quanto em quanto tempo olhar a pasta dos pedidos (só `stat`).
const AGENTS_POLL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone)]
pub enum Message {
    Term(iced_term::Event),
    /// Abre um shell novo (numa aba nova).
    Start,
    /// Mostra o shell da aba `n` (posição na lista).
    ShowShell(usize),
    ShowTask,
    CloseTask,
    /// Fecha o terminal na tela (encerra o processo).
    CloseCurrent,
    /// Os pedidos de terminal na pasta dos agentes (vem quando algo muda).
    Requests(BTreeMap<String, Result<agents::Request, String>>),
    ShowAgent(usize),
    ShowRequest,
    /// O usuário permitiu este pedido (a cópia que estava na tela).
    Allow(Box<agents::Request>),
    Refuse(String),
}

struct TaskTerm {
    term: Terminal,
    name: String,
}

/// Um shell numa aba: o número fica o mesmo enquanto a aba existir ("Terminal 2").
struct Shell {
    term: Terminal,
    number: u32,
}

/// Um terminal pedido por um agente e permitido pelo usuário.
struct AgentTerm {
    term: Terminal,
    name: String,
    title: String,
    /// O comando terminou (status `exited`): apagar o pedido não fecha mais a aba, quem fecha é o usuário.
    ended: bool,
    /// O pedido sumiu com a aba ainda aberta: ela não pertence mais a nenhum pedido.
    detached: bool,
    /// Worktree a remover quando o usuário fechar a aba (o pedido já sumiu).
    worktree_on_close: bool,
}

/// O que está na tela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    Shell(usize),
    Task,
    Agent(usize),
    /// O pedido de um agente, esperando o usuário.
    Request,
}

pub struct TerminalExtension {
    root: PathBuf,
    shells: Vec<Shell>,
    task: Option<TaskTerm>,
    agents: Vec<AgentTerm>,
    /// Pedidos esperando o usuário, por nome: o que aparece e o que roda.
    pending: BTreeMap<String, agents::Request>,
    /// Pedidos já tratados nesta sessão (ou por outra IDE), com o conteúdo de quando foram tratados. Voltam a valer
    /// quando o arquivo some, muda (um `invalid` corrigido) ou quando o agente apaga o status.
    handled: BTreeMap<String, Result<agents::Request, String>>,
    shown: Shown,
    next_id: u64,
    next_number: u32,
    /// O tamanho da área do terminal, como o widget mediu por último.
    size: Option<iced::Size>,
    theme: Theme,
    font_size: f32,
    code_font: tea_core::fonts::CodeFont,
    /// De onde vem a lista de comandos que rodam sem confirmar (`[agents] allow`, só do config do usuário).
    allow_list: fn() -> Vec<String>,
}

/// O tamanho da área do terminal antes de o widget medir (um painel de baixo comum); o widget corrige depois.
const SIZE_GUESS: iced::Size = iced::Size::new(900.0, 260.0);

/// O `[agents] allow` do config do usuário, lido de novo a cada pedido novo (sem esperar recarregar o config).
#[cfg(not(test))]
fn user_allow_list() -> Vec<String> {
    let Some(global) = tea_core::config::global_config_path() else { return Vec::new() };
    tea_core::config::Config::load(Some(&global), None).agents.allow
}

/// Nos testes, nada é permitido sem confirmar (o config de quem roda os testes não entra).
#[cfg(test)]
fn user_allow_list() -> Vec<String> {
    Vec::new()
}

impl Default for TerminalExtension {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            shells: Vec::new(),
            task: None,
            agents: Vec::new(),
            pending: BTreeMap::new(),
            handled: BTreeMap::new(),
            shown: Shown::Shell(0),
            next_id: 0,
            next_number: 0,
            size: None,
            theme: Theme::TokyoNight,
            font_size: 14.0,
            code_font: tea_core::fonts::CodeFont::default(),
            allow_list: user_allow_list,
        }
    }
}

impl TerminalExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn settings(&self, program: String, args: Vec<String>, cwd: &Path) -> Settings {
        Settings {
            font: self.font(),
            theme: ThemeSettings::new(Box::new(palette_for(&self.theme))),
            backend: BackendSettings {
                program,
                args,
                env: [("TERM_PROGRAM".to_string(), "idetea".to_string())].into(),
                working_directory: Some(cwd.to_path_buf()),
            },
        }
    }

    fn font(&self) -> FontSettings {
        FontSettings {
            size: self.font_size,
            // A altura padrão do terminal (1.3) é mais justa que a do editor (1.45): acompanha a do
            // config na mesma proporção.
            scale_factor: self.code_font.line_height * 1.3 / tea_core::fonts::LINE_HEIGHT_DEFAULT,
            font_type: self.code_font.font,
        }
    }

    /// Abre um shell numa aba nova e mostra.
    fn start(&mut self, ctx: &mut Context) -> bool {
        self.next_id += 1;
        let shell = default_shell();
        match self.new_terminal(self.settings(shell, Vec::new(), &self.root.clone())) {
            Ok(term) => {
                // A numeração recomeça quando não sobra nenhum.
                if self.shells.is_empty() {
                    self.next_number = 0;
                }
                self.next_number += 1;
                self.shells.push(Shell { term, number: self.next_number });
                self.shown = Shown::Shell(self.shells.len() - 1);
                true
            }
            Err(e) => {
                ctx.notify(fl!("terminal-start-error", error = e.to_string()));
                false
            }
        }
    }

    /// Troca o shell na tela por um novo, na mesma aba (o processo anterior é encerrado).
    fn restart(&mut self, ctx: &mut Context) -> bool {
        let Shown::Shell(i) = self.shown else { return self.start(ctx) };
        if i >= self.shells.len() {
            return self.start(ctx);
        }
        let number = self.shells[i].number;
        self.shells.remove(i);
        if !self.start(ctx) {
            return false;
        }
        // O novo foi para o fim: volta para o lugar do antigo, com o mesmo número.
        let mut shell = self.shells.pop().expect("acabou de entrar");
        shell.number = number;
        self.shells.insert(i, shell);
        self.shown = Shown::Shell(i);
        true
    }

    /// Roda uma tarefa num terminal novo (a anterior, se ainda rodava, é encerrada).
    fn run_task(&mut self, name: &str, command: &str, cwd: &Path, done: &str, ctx: &mut Context) -> Task<Message> {
        self.next_id += 1;
        self.task = None;
        let (before, after) = done.split_once("{code}").unwrap_or((done, ""));
        #[cfg(unix)]
        let settings = {
            let args = ["-c", TASK_SCRIPT, "sh", command, before, after].map(str::to_string).to_vec();
            self.settings("/bin/sh".into(), args, cwd)
        };
        #[cfg(windows)]
        let settings = {
            let ps = tea_core::os::find_in_path("pwsh.exe")
                .or_else(|| tea_core::os::find_in_path("powershell.exe"))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "powershell.exe".into());
            let args = ["-NoLogo", "-NoProfile", "-Command", WINDOWS_TASK_SCRIPT].map(str::to_string).to_vec();
            let mut s = self.settings(ps, args, cwd);
            s.backend.env.insert("IDETEA_CMD".into(), command.to_string());
            s.backend.env.insert("IDETEA_BEFORE".into(), before.to_string());
            s.backend.env.insert("IDETEA_AFTER".into(), after.to_string());
            s
        };
        match self.new_terminal(settings) {
            Ok(term) => {
                self.task = Some(TaskTerm { term, name: name.to_string() });
                self.shown = Shown::Task;
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            Err(e) => {
                ctx.notify(fl!("terminal-task-error", error = e.to_string()));
                Task::none()
            }
        }
    }

    /// O terminal na tela agora.
    fn current(&self) -> Option<&Terminal> {
        match self.shown {
            Shown::Task => self.task.as_ref().map(|t| &t.term),
            Shown::Shell(i) => self.shells.get(i).map(|s| &s.term),
            Shown::Agent(i) => self.agents.get(i).map(|a| &a.term),
            Shown::Request => None,
        }
    }

    /// Há algo na tela (um terminal, ou um pedido esperando).
    fn showing(&self) -> bool {
        self.current().is_some() || (self.shown == Shown::Request && !self.pending.is_empty())
    }

    fn all_terms_mut(&mut self) -> impl Iterator<Item = &mut Terminal> + '_ {
        self.shells
            .iter_mut()
            .map(|s| &mut s.term)
            .chain(self.task.iter_mut().map(|t| &mut t.term))
            .chain(self.agents.iter_mut().map(|a| &mut a.term))
    }

    fn focus_current(&self) -> Task<Message> {
        match self.current() {
            Some(t) => TerminalView::focus(t.widget_id().clone()),
            None => Task::none(),
        }
    }

    /// Depois de um terminal sair: mostra o vizinho; sem nenhum, esconde o painel.
    fn after_removal(&mut self, ctx: &mut Context) -> Task<Message> {
        self.shown = match self.shown {
            Shown::Task if self.task.is_some() => Shown::Task,
            Shown::Request if !self.pending.is_empty() => Shown::Request,
            Shown::Shell(i) if !self.shells.is_empty() => Shown::Shell(i.min(self.shells.len() - 1)),
            Shown::Agent(i) if !self.agents.is_empty() => Shown::Agent(i.min(self.agents.len() - 1)),
            _ if !self.pending.is_empty() => Shown::Request,
            _ if !self.shells.is_empty() => Shown::Shell(self.shells.len() - 1),
            _ if !self.agents.is_empty() => Shown::Agent(self.agents.len() - 1),
            _ if self.task.is_some() => Shown::Task,
            _ => Shown::Shell(0),
        };
        if !self.showing() {
            ctx.hide_panel(PANEL);
            return Task::none();
        }
        self.focus_current()
    }

    fn close_current(&mut self, ctx: &mut Context) -> Task<Message> {
        // Soltar o terminal encerra o processo.
        match self.shown {
            Shown::Task => self.task = None,
            Shown::Shell(i) if i < self.shells.len() => {
                self.shells.remove(i);
            }
            Shown::Agent(i) if i < self.agents.len() => {
                let agent = self.agents.remove(i);
                // Fechado antes do fim: quem pediu fica sabendo.
                if !agent.detached && agents::read_status(&self.root, &agent.name).is_some_and(|d| d.state == "running") {
                    let _ = agents::write_status(&self.root, &agent.name, &agents::State::Closed);
                }
                if agent.worktree_on_close && let Err(e) = worktree::remove(&self.root, &agent.name) {
                    ctx.notify(fl!("agent-worktree-kept", path = worktree::path(&self.root, &agent.name).display().to_string(), error = e));
                }
            }
            Shown::Shell(_) | Shown::Agent(_) | Shown::Request => {}
        }
        self.after_removal(ctx)
    }

    /// Os pedidos mudaram na pasta: os que sumiram encerram o terminal (ou deixam de esperar); os novos aparecem.
    fn requests(&mut self, found: BTreeMap<String, Result<agents::Request, String>>, ctx: &mut Context) -> Task<Message> {
        agents::ensure_gitignore(&self.root);
        let ours = std::process::id();
        let mut removed = false;
        // Quem já terminou fica marcado antes de o pedido sumir (o `idetea run` apaga logo depois do `exited`).
        for agent in self.agents.iter_mut().filter(|a| !a.ended && !a.detached) {
            agent.ended = agents::read_status(&self.root, &agent.name).is_some_and(|d| d.state == "exited");
        }
        // Apagar o pedido é o jeito de o agente encerrar o terminal; se o comando já terminou, a aba fica até o
        // usuário fechar (para ver a saída e o código).
        let gone: Vec<String> = self.handled.keys().chain(self.pending.keys()).filter(|n| !found.contains_key(*n)).cloned().collect();
        for name in gone {
            let had_worktree = matches!(self.handled.get(&name), Some(Ok(r)) if r.worktree.is_some());
            self.handled.remove(&name);
            removed |= self.pending.remove(&name).is_some();
            if let Some(i) = self.agents.iter().position(|a| a.name == name && !a.detached) {
                if self.agents[i].ended {
                    let agent = &mut self.agents[i];
                    agent.detached = true;
                    agent.worktree_on_close = had_worktree;
                    continue;
                }
                self.agents.remove(i);
                if let Shown::Agent(s) = self.shown
                    && s > i
                {
                    self.shown = Shown::Agent(s - 1);
                }
                removed = true;
            }
            // O branch fica sempre; o worktree sai se estiver limpo (o git recusa se houver o que commitar).
            if had_worktree && let Err(e) = worktree::remove(&self.root, &name) {
                ctx.notify(fl!("agent-worktree-kept", path = worktree::path(&self.root, &name).display().to_string(), error = e));
            }
        }
        let mut asked = None;
        let mut allow: Option<Vec<String>> = None;
        let mut auto = Vec::new();
        let mut notes = Vec::new();
        for (name, request) in found {
            // Já tratado: só volta se o pedido mudou ou o status sumiu (e não está rodando aqui).
            let again = match self.handled.get(&name) {
                None => false,
                Some(before) => {
                    let running_here = self.agents.iter().any(|a| a.name == name && !a.detached);
                    let status_gone = agents::read_status(&self.root, &name).is_none();
                    if running_here || (*before == request && !status_gone) {
                        continue;
                    }
                    self.handled.remove(&name);
                    true
                }
            };
            let waiting = self.pending.contains_key(&name);
            if !waiting {
                let disk = agents::read_status(&self.root, &name);
                let decision = if again && !agents::owned_by_other(disk.as_ref(), ours) {
                    agents::Decision::Ask
                } else {
                    agents::decide(disk.as_ref(), ours)
                };
                match decision {
                    agents::Decision::Ask => {}
                    agents::Decision::Leave => {
                        self.handled.insert(name, request);
                        continue;
                    }
                    agents::Decision::Lost => {
                        let _ = agents::write_status(&self.root, &name, &agents::State::Lost);
                        self.handled.insert(name, request);
                        continue;
                    }
                }
            }
            match request {
                Err(error) => {
                    removed |= self.pending.remove(&name).is_some();
                    let _ = agents::write_status(&self.root, &name, &agents::State::Invalid(error.clone()));
                    self.handled.insert(name, Err(error));
                }
                // Regravado enquanto espera: a tela mostra a versão nova (é ela que roda).
                Ok(request) if waiting => {
                    if self.pending.get(&name) != Some(&request) {
                        self.pending.insert(name, request);
                    }
                }
                Ok(request) => {
                    let _ = agents::write_status(&self.root, &name, &agents::State::Pending);
                    let list = allow.get_or_insert_with(self.allow_list);
                    if let Some(entry) = agents::allowed(&request.command, list) {
                        // Na lista do usuário: roda sem perguntar, e o aviso diz por qual linha.
                        notes.push(fl!("agent-request-auto", title = request.title.as_str(), entry = entry));
                        auto.push(request.clone());
                    } else {
                        asked.get_or_insert_with(|| request.title.clone());
                    }
                    self.pending.insert(name, request);
                }
            }
        }
        let started: Vec<Task<Message>> = auto.into_iter().map(|r| self.allow(r, ctx)).collect();
        if let Some(title) = &asked {
            ctx.notify(fl!("agent-request-notify", title = title));
            self.shown = Shown::Request;
            ctx.show_panel(PANEL);
        }
        // Por último: o pedido que espera já tem a aba dele; o que rodou sozinho só tem este aviso.
        for note in notes {
            ctx.notify(note);
        }
        if asked.is_some() {
            return Task::batch(started);
        }
        if !started.is_empty() {
            return Task::batch(started);
        }
        if removed || (self.shown == Shown::Request && self.pending.is_empty()) {
            return self.after_removal(ctx);
        }
        Task::none()
    }

    /// Roda o pedido permitido, se a cópia na tela ainda é a que espera.
    fn allow(&mut self, request: agents::Request, ctx: &mut Context) -> Task<Message> {
        if self.pending.get(&request.name) != Some(&request) {
            return Task::none();
        }
        self.pending.remove(&request.name);
        self.handled.insert(request.name.clone(), Ok(request.clone()));
        self.next_id += 1;
        let done = fl!("agent-done", code = "{code}");
        let (before, after) = done.split_once("{code}").unwrap_or((&done, ""));
        let status = agents::status_path(&self.root, &request.name).to_string_lossy().into_owned();
        let pid = std::process::id().to_string();
        // Com worktree: a mesma pasta relativa, dentro da cópia do agente.
        let cwd = match &request.worktree {
            None => request.cwd.clone(),
            Some(branch) => match worktree::ensure(&self.root, &request.name, branch) {
                Ok(dir) => {
                    let base = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
                    let rel = request.cwd.strip_prefix(&base).map(Path::to_path_buf).unwrap_or_default();
                    dir.join(rel)
                }
                Err(e) => {
                    let _ = agents::write_status(&self.root, &request.name, &agents::State::Invalid(e.clone()));
                    ctx.notify(fl!("terminal-task-error", error = e));
                    return self.after_removal(ctx);
                }
            },
        };
        let extra = agents::extra(&request.name, request.worktree.as_deref());
        // O mesmo `extra` no `exited` que o script grava: `,"chave":valor,...` (JSON já escapado).
        let tail: String = extra.iter().map(|(k, v)| format!(",{}:{v}", serde_json::Value::from(k.as_str()))).collect();
        // `running` antes de subir: o script grava `exited` por cima no fim.
        let _ = agents::write_status_with(&self.root, &request.name, &agents::State::Running, &extra);
        #[cfg(unix)]
        let mut settings = {
            let args = ["-c", AGENT_SCRIPT, "sh", &request.command, &status, &pid, before, after, &tail].map(str::to_string).to_vec();
            self.settings("/bin/sh".into(), args, &cwd)
        };
        #[cfg(windows)]
        let mut settings = {
            let ps = tea_core::os::find_in_path("pwsh.exe")
                .or_else(|| tea_core::os::find_in_path("powershell.exe"))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "powershell.exe".into());
            let args = ["-NoLogo", "-NoProfile", "-Command", WINDOWS_AGENT_SCRIPT].map(str::to_string).to_vec();
            let mut s = self.settings(ps, args, &cwd);
            s.backend.env.insert("IDETEA_CMD".into(), request.command.clone());
            s.backend.env.insert("IDETEA_STATUS".into(), status);
            s.backend.env.insert("IDETEA_PID".into(), pid);
            s.backend.env.insert("IDETEA_BEFORE".into(), before.to_string());
            s.backend.env.insert("IDETEA_AFTER".into(), after.to_string());
            s.backend.env.insert("IDETEA_TAIL".into(), tail);
            s
        };
        let answer = agents::answer_path(&self.root, &request.name).to_string_lossy().into_owned();
        settings.backend.env.insert("IDETEA_ANSWER".into(), answer);
        match self.new_terminal(settings) {
            Ok(term) => {
                self.agents.push(AgentTerm { term, name: request.name, title: request.title, ended: false, detached: false, worktree_on_close: false });
                self.shown = Shown::Agent(self.agents.len() - 1);
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            Err(e) => {
                let _ = agents::write_status(&self.root, &request.name, &agents::State::Invalid(e.to_string()));
                ctx.notify(fl!("terminal-task-error", error = e.to_string()));
                self.after_removal(ctx)
            }
        }
    }

    fn refuse(&mut self, name: &str, ctx: &mut Context) -> Task<Message> {
        if let Some(request) = self.pending.remove(name) {
            let _ = agents::write_status(&self.root, name, &agents::State::Refused);
            self.handled.insert(name.to_string(), Ok(request));
        }
        self.after_removal(ctx)
    }

    /// A tela do pedido: tudo o que vai rodar, e os dois botões.
    fn request_view(&self) -> Element<'_, Message> {
        let Some(request) = self.pending.values().next() else { return space::vertical().into() };
        let who = request.source.as_deref().map_or_else(|| fl!("agent-request-someone"), str::to_string);
        let cwd = request
            .cwd
            .strip_prefix(self.root.canonicalize().unwrap_or_else(|_| self.root.clone()))
            .ok()
            .map(|p| if p.as_os_str().is_empty() { ".".to_string() } else { p.display().to_string() })
            .unwrap_or_else(|| request.cwd.display().to_string());
        let mut col = column![
            text(fl!("agent-request-title", who = who, title = request.title.as_str())).size(14),
            text(fl!("agent-request-command")).size(12).style(text::secondary),
            container(text(request.command.as_str()).font(Font::MONOSPACE).size(13))
                .padding(8)
                .width(Fill)
                .style(container::bordered_box),
            text(fl!("agent-request-cwd", cwd = cwd)).size(12),
        ]
        .spacing(8)
        .padding(8);
        if let Some(branch) = &request.worktree {
            let path = format!("{}/{}", worktree::DIR, request.name);
            col = col.push(text(fl!("agent-request-worktree", branch = branch.as_str(), path = path)).size(12));
        }
        col = col.push(column![
            text(fl!("agent-request-warning")).size(12).style(text::warning),
            row![
                button(text(fl!("agent-request-allow")).size(13))
                    .on_press(Message::Allow(Box::new(request.clone())))
                    .padding([3, 12])
                    .style(button::primary),
                button(text(fl!("agent-request-refuse")).size(13))
                    .on_press(Message::Refuse(request.name.clone()))
                    .padding([3, 12])
                    .style(button::secondary),
            ]
            .spacing(8),
        ]
        .spacing(8));
        if self.pending.len() > 1 {
            let count = self.pending.len() - 1;
            col = col.push(text(fl!("agent-request-more", count = count)).size(12).style(text::secondary));
        }
        scrollable(col).height(Fill).into()
    }

    /// Mostra um shell (abre um, se não houver) e pega o teclado.
    fn show_shell(&mut self, ctx: &mut Context) -> Task<Message> {
        if !matches!(self.shown, Shown::Shell(i) if i < self.shells.len()) {
            if self.shells.is_empty() && !self.start(ctx) {
                return Task::none();
            }
            self.shown = Shown::Shell(self.shells.len() - 1);
        }
        ctx.show_panel(PANEL);
        self.focus_current()
    }

    /// Cria um terminal já com o tamanho da área. Todo comando ao `iced_term` recalcula linhas e colunas com o
    /// último tamanho recebido; um terminal novo só tem o provisório (80×50 pixels) até o widget medir, e qualquer
    /// coisa antes disso (a saída do shell, um redesenho) o espremia em poucas colunas, quebrando o prompt.
    fn new_terminal(&mut self, settings: Settings) -> std::io::Result<Terminal> {
        let mut term = Terminal::new(self.next_id, settings)?;
        // Sem medida ainda (nenhum terminal apareceu), o `iced_term` usaria uma área de 80×50 px: umas 10 colunas e
        // 2 linhas, e a saída de um comando curto sumiria espremida antes de o widget medir. Uma estimativa até lá.
        let size = self.size.unwrap_or(SIZE_GUESS);
        term.handle(iced_term::Command::ProxyToBackend(iced_term::BackendCommand::Resize(Some(size), None)));
        Ok(term)
    }

    /// O que muda o lugar do terminal na tela: qual está à vista e se há barra de abas.
    fn layout(&self) -> (Shown, bool) {
        (self.shown, self.showing())
    }

    /// O `iced_term` guarda o desenho com a posição na tela dentro; quando a barra aparece ou some, ou outra aba
    /// volta, o desenho velho sairia no lugar antigo (por cima da barra). Reaplicar a fonte limpa esse cache.
    fn redraw_if_moved(&mut self, before: (Shown, bool)) {
        // Sem medida ainda, não há desenho velho; e reaplicar a fonte recalcularia o tamanho com o provisório.
        if self.layout() == before || self.size.is_none() {
            return;
        }
        let font = self.font();
        for term in self.all_terms_mut() {
            term.handle(iced_term::Command::ChangeFont(font.clone()));
        }
    }

    fn on_event_inner(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        // Aberto pela barra de ícones: já sobe o shell e pega o teclado.
        if let CoreEvent::PanelShown(_) = event {
            if !self.showing() {
                if !self.pending.is_empty() {
                    self.shown = Shown::Request;
                } else if !self.shells.is_empty() {
                    self.shown = Shown::Shell(self.shells.len() - 1);
                } else if self.task.is_some() {
                    self.shown = Shown::Task;
                } else if !self.start(ctx) {
                    return Task::none();
                }
            }
            return self.focus_current();
        }
        if let CoreEvent::RunTask { name, command, cwd, done } = event {
            return self.run_task(name, command, cwd, done, ctx);
        }
        if let CoreEvent::Appearance { theme, font_size, code_font } = event {
            self.theme = theme.clone();
            self.font_size = *font_size;
            self.code_font = *code_font;
            let font = self.font();
            for term in self.all_terms_mut() {
                term.handle(iced_term::Command::ChangeTheme(Box::new(palette_for(theme))));
                term.handle(iced_term::Command::ChangeFont(font.clone()));
            }
        }
        Task::none()
    }

    fn on_command_inner(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "terminal.focus" => self.show_shell(ctx),
            "terminal.new" => {
                if !self.start(ctx) {
                    return Task::none();
                }
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            "terminal.close" => self.close_current(ctx),
            "terminal.restart" => {
                if !self.restart(ctx) {
                    return Task::none();
                }
                ctx.show_panel(PANEL);
                self.focus_current()
            }
            _ => Task::none(),
        }
    }

    fn update_inner(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Start => {
                if self.start(ctx) {
                    return self.focus_current();
                }
            }
            Message::ShowShell(i) => {
                if i < self.shells.len() {
                    self.shown = Shown::Shell(i);
                }
                return self.focus_current();
            }
            Message::ShowTask => {
                if self.task.is_some() {
                    self.shown = Shown::Task;
                }
                return self.focus_current();
            }
            Message::CloseTask => {
                // Soltar o terminal encerra a tarefa, se ainda rodava.
                self.task = None;
                return self.after_removal(ctx);
            }
            Message::CloseCurrent => return self.close_current(ctx),
            Message::Requests(found) => return self.requests(found, ctx),
            Message::ShowAgent(i) => {
                if i < self.agents.len() {
                    self.shown = Shown::Agent(i);
                }
                return self.focus_current();
            }
            Message::ShowRequest => {
                if !self.pending.is_empty() {
                    self.shown = Shown::Request;
                }
            }
            Message::Allow(request) => return self.allow(*request, ctx),
            Message::Refuse(name) => return self.refuse(&name, ctx),
            Message::Term(iced_term::Event::BackendCall(id, cmd)) => {
                if let iced_term::BackendCommand::Resize(Some(size), _) = &cmd {
                    self.size = Some(*size);
                }
                if let Some(task) = self.task.as_mut().filter(|t| t.term.id == id) {
                    // A tarefa terminou e o Enter fechou: volta para um shell.
                    if task.term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                        self.task = None;
                        return self.after_removal(ctx);
                    }
                    return Task::none();
                }
                if let Some(i) = self.agents.iter().position(|a| a.term.id == id) {
                    // Terminou e o Enter fechou (o script já gravou o código no status).
                    if self.agents[i].term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                        self.agents.remove(i);
                        if let Shown::Agent(s) = self.shown
                            && s > i
                        {
                            self.shown = Shown::Agent(s - 1);
                        }
                        return self.after_removal(ctx);
                    }
                    return Task::none();
                }
                // Evento atrasado de um shell que já saiu: nenhum com esse id.
                let Some(i) = self.shells.iter().position(|s| s.term.id == id) else { return Task::none() };
                if self.shells[i].term.handle(iced_term::Command::ProxyToBackend(cmd)) == Action::Shutdown {
                    let number = self.shells[i].number;
                    self.shells.remove(i);
                    // A aba que estava na tela à direita desta escorrega uma posição.
                    if let Shown::Shell(s) = self.shown
                        && s > i
                    {
                        self.shown = Shown::Shell(s - 1);
                    }
                    ctx.notify(fl!("terminal-ended-n", number = number));
                    return self.after_removal(ctx);
                }
            }
        }
        Task::none()
    }
}

impl Extension for TerminalExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "terminal"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, "Terminal", Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::Terminal);
        ctx.register_command("terminal.focus", fl!("cmd-terminal-focus"));
        ctx.register_command("terminal.new", fl!("cmd-terminal-new"));
        ctx.register_command("terminal.close", fl!("cmd-terminal-close"));
        ctx.register_command("terminal.restart", fl!("cmd-terminal-restart"));
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        let before = self.layout();
        let task = self.on_event_inner(event, ctx);
        self.redraw_if_moved(before);
        task
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        let before = self.layout();
        let task = self.on_command_inner(command, ctx);
        self.redraw_if_moved(before);
        task
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        let before = self.layout();
        let task = self.update_inner(message, ctx);
        self.redraw_if_moved(before);
        task
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let body: Element<'_, Message> = match self.current() {
            None if self.showing() => self.request_view(),
            // Com chave: cada terminal tem o próprio estado no widget (o tamanho que já avisou ao shell). Sem ela,
            // o terminal que entra herdava o tamanho do anterior e nunca recebia o seu.
            Some(term) => keyed_column([(term.id, TerminalView::show(term).map(Message::Term))]).height(Fill).into(),
            None => container(
                column![
                    text(fl!("terminal-not-running")).size(13).style(text::secondary),
                    button(text(fl!("terminal-start")).size(13)).on_press(Message::Start).padding([3, 10]),
                ]
                .spacing(8)
                .align_x(iced::Center),
            )
            .center(Fill)
            .into(),
        };
        // Sem terminal: só o botão de iniciar. Com um ou mais, a barra (o `+` fica sempre à mão).
        if !self.showing() {
            return body;
        }
        let tab = |label: String, on: bool, msg: Message| -> Element<'_, Message> {
            button(text(label).size(12)).on_press(msg).padding([2, 10]).style(if on { button::secondary } else { button::text }).into()
        };
        let mut bar = row![].spacing(4).align_y(iced::Center);
        for (i, s) in self.shells.iter().enumerate() {
            bar = bar.push(tab(fl!("terminal-tab-n", number = s.number), self.shown == Shown::Shell(i), Message::ShowShell(i)));
        }
        if let Some(task) = &self.task {
            bar = bar.push(tab(fl!("terminal-tab-task", name = task.name.as_str()), self.shown == Shown::Task, Message::ShowTask));
        }
        for (i, a) in self.agents.iter().enumerate() {
            bar = bar.push(tab(fl!("terminal-tab-agent", title = a.title.as_str()), self.shown == Shown::Agent(i), Message::ShowAgent(i)));
        }
        if !self.pending.is_empty() {
            let on = self.shown == Shown::Request;
            bar = bar.push(
                button(text(fl!("terminal-tab-request", count = self.pending.len())).size(12))
                    .on_press(Message::ShowRequest)
                    .padding([2, 10])
                    .style(if on { button::primary } else { button::secondary }),
            );
        }
        bar = bar.push(icons::icon_button(Icon::Add, fl!("cmd-terminal-new"), Some(Message::Start)));
        bar = bar.push(space::horizontal());
        // O pedido se recusa pelo botão dele, não pelo ×.
        if self.shown != Shown::Request {
            let close = if self.shown == Shown::Task { fl!("terminal-close-task") } else { fl!("terminal-close") };
            bar = bar.push(icons::icon_button(Icon::Close, close, Some(Message::CloseCurrent)));
        }
        column![bar, body].spacing(2).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let terms = self.shells.iter().map(|s| &s.term).chain(self.task.iter().map(|t| &t.term)).chain(self.agents.iter().map(|a| &a.term));
        let mut subs: Vec<Subscription<Message>> = terms.map(|t| t.subscription().map(Message::Term)).collect();
        if !self.root.as_os_str().is_empty() {
            subs.push(Subscription::run_with(self.root.clone(), watch_requests));
        }
        Subscription::batch(subs)
    }
}

/// Olha a pasta dos pedidos (só `stat`); quando algo muda, lê tudo e manda. Termina quando a subscription some.
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
fn watch_requests(root: &PathBuf) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let root = root.clone();
    std::thread::spawn(move || {
        let mut last = None;
        loop {
            if tx.is_closed() {
                return;
            }
            let now = agents::signature(&root);
            if last.as_ref() != Some(&now) {
                last = Some(now);
                if tx.unbounded_send(Message::Requests(agents::scan(&root))).is_err() {
                    return;
                }
            }
            std::thread::sleep(AGENTS_POLL);
        }
    });
    rx
}

fn hex(c: Color) -> String {
    let [r, g, b, _] = c.into_rgba8();
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Cores do terminal a partir do tema da IDE: fundo, texto e as cores de destaque.
fn palette_for(theme: &Theme) -> ColorPalette {
    let p = theme.extended_palette();
    let base = ColorPalette::default();
    let mut out = ColorPalette {
        foreground: hex(p.background.base.text),
        background: hex(p.background.base.color),
        red: hex(p.danger.base.color),
        green: hex(p.success.base.color),
        yellow: hex(p.warning.base.color),
        blue: hex(p.primary.base.color),
        ..base
    };
    if !p.is_dark {
        // As cores padrão são pensadas para fundo escuro: "preto" e "branco" invertem.
        out.black = "#3b3b3b".into();
        out.white = "#7a7a7a".into();
        out.bright_black = "#6b6b6b".into();
        out.bright_white = "#2b2b2b".into();
        out.yellow = "#a06d00".into();
        out.bright_yellow = "#8a5d00".into();
        out.cyan = "#1f7f86".into();
        out.bright_cyan = "#176a70".into();
        out.dim_foreground = "#6b6b6b".into();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_follows_theme_background() {
        let dark = palette_for(&Theme::TokyoNight);
        let light = palette_for(&Theme::Light);
        assert_eq!(dark.background, hex(Theme::TokyoNight.extended_palette().background.base.color));
        assert_eq!(light.background, hex(Theme::Light.extended_palette().background.base.color));
        assert_ne!(dark.foreground, light.foreground);
        assert!(dark.background.starts_with('#') && dark.background.len() == 7);
    }

    /// O script da tarefa: o comando roda com `&&` e variáveis, e o fim mostra o código de saída.
    #[cfg(unix)]
    #[test]
    fn task_script_reports_exit_code() {
        let out = std::process::Command::new("/bin/sh")
            .args(["-c", TASK_SCRIPT, "sh", "X=oi; echo $X && exit 3", "[fim: ", "]"])
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("$ X=oi; echo $X && exit 3"), "{text}");
        assert!(text.contains("oi\n"), "{text}");
        assert!(text.contains("[fim: 3]"), "{text}");
    }

    fn status(root: &Path, name: &str) -> String {
        agents::read_status(root, name).map(|d| d.state).unwrap_or_default()
    }

    /// Pedido chega, espera o usuário (status `pending`), é recusado ou permitido; apagar o pedido encerra o terminal;
    /// um pedido regravado enquanto espera troca o que aparece, e a cópia velha na tela não roda mais.
    #[test]
    fn agent_requests_wait_for_the_user() {
        let root = std::env::temp_dir().join(format!("idetea-agent-flow-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(agents::DIR)).unwrap();
        let put = |name: &str, command: &str| {
            let doc = serde_json::json!({"format": agents::FORMAT, "version": 1, "title": name, "command": command});
            std::fs::write(root.join(agents::DIR).join(format!("{name}.json")), doc.to_string()).unwrap();
        };
        let mut ext = TerminalExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));

        put("um", "echo um");
        put("dois", "echo dois");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(ext.shown, Shown::Request);
        assert_eq!(ext.pending.len(), 2);
        assert_eq!((status(&root, "um"), status(&root, "dois")), ("pending".into(), "pending".into()));
        assert!(root.join(agents::DIR).join(".gitignore").exists());

        // Recusado: não volta a perguntar enquanto o arquivo existir.
        let _ = ext.update(Message::Refuse("dois".into()), &mut Context::new(&root, None));
        assert_eq!(status(&root, "dois"), "refused");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(ext.pending.keys().collect::<Vec<_>>(), ["um"]);

        // Regravado enquanto espera: a cópia antiga (a que estava na tela) não roda.
        let old = ext.pending["um"].clone();
        put("um", "sleep 30");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(ext.pending["um"].command, "sleep 30");
        let _ = ext.update(Message::Allow(Box::new(old)), &mut Context::new(&root, None));
        assert!(ext.agents.is_empty());
        assert_eq!(status(&root, "um"), "pending");

        let new = ext.pending["um"].clone();
        let _ = ext.update(Message::Allow(Box::new(new)), &mut Context::new(&root, None));
        assert_eq!(ext.agents.len(), 1);
        assert_eq!(ext.shown, Shown::Agent(0));
        assert!(ext.pending.is_empty());

        // Na lista do usuário: roda sem perguntar. Fora dela (mesmo começando igual), pergunta.
        ext.allow_list = || vec!["echo *".into()];
        put("livre", "echo livre");
        put("preso", "echo x; touch nao");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert!(ext.agents.iter().any(|a| a.name == "livre"), "permitido pela lista");
        assert_ne!(status(&root, "livre"), "pending");
        assert_eq!(ext.pending.keys().collect::<Vec<_>>(), ["preso"]);
        assert_eq!(ext.shown, Shown::Request);
        let _ = ext.update(Message::Refuse("preso".into()), &mut Context::new(&root, None));
        for name in ["livre", "preso"] {
            let _ = std::fs::remove_file(root.join(agents::DIR).join(format!("{name}.json")));
        }
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        ext.allow_list = user_allow_list;
        // `echo livre` pode ter terminado antes: aí a aba ficou e quem fecha é o usuário.
        while let Some(i) = ext.agents.iter().position(|a| a.name == "livre") {
            ext.shown = Shown::Agent(i);
            let _ = ext.close_current(&mut Context::new(&root, None));
        }

        // O agente apaga o pedido: o terminal fecha.
        std::fs::remove_file(root.join(agents::DIR).join("um.json")).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert!(ext.agents.is_empty());

        // Já terminou quando o pedido some (o `idetea run` limpa logo): a aba fica até o usuário fechar.
        put("cinco", "sleep 30");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        let req = ext.pending["cinco"].clone();
        let _ = ext.update(Message::Allow(Box::new(req)), &mut Context::new(&root, None));
        let doc = agents::status_json(&agents::State::Exited(0), std::process::id(), &serde_json::Map::new());
        std::fs::write(agents::status_path(&root, "cinco"), doc).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        std::fs::remove_file(root.join(agents::DIR).join("cinco.json")).unwrap();
        std::fs::remove_file(agents::status_path(&root, "cinco")).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(ext.agents.len(), 1, "a aba fica");
        assert!(ext.agents[0].detached);
        // O mesmo nome pedido de novo é outro pedido (a aba velha não o segura).
        put("cinco", "sleep 30");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert!(ext.pending.contains_key("cinco"));
        let _ = ext.update(Message::Refuse("cinco".into()), &mut Context::new(&root, None));
        ext.shown = Shown::Agent(0);
        let _ = ext.close_current(&mut Context::new(&root, None));
        assert!(ext.agents.is_empty());
        assert_eq!(status(&root, "cinco"), "refused", "fechar a aba velha não mexe no pedido novo");
        std::fs::remove_file(root.join(agents::DIR).join("cinco.json")).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));

        // Pedido recusado e regravado igual, sem a IDE ver o arquivo sumir: apagar o status basta para perguntar de novo.
        assert_eq!(status(&root, "dois"), "refused");
        std::fs::remove_file(agents::status_path(&root, "dois")).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert!(ext.pending.contains_key("dois"));
        assert_eq!(status(&root, "dois"), "pending");
        let _ = ext.update(Message::Refuse("dois".into()), &mut Context::new(&root, None));

        // Um `invalid` corrigido (regravado, sem apagar o status) é perguntado.
        put("quatro", "");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(status(&root, "quatro"), "invalid");
        put("quatro", "echo quatro");
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(status(&root, "quatro"), "pending");
        let _ = ext.update(Message::Refuse("quatro".into()), &mut Context::new(&root, None));

        // Outra IDE (esta, reaberta) vê um `running` de quem fechou: vira `lost`, sem perguntar.
        put("tres", "echo tres");
        let doc = agents::status_json(&agents::State::Running, 4_194_400, &serde_json::Map::new());
        std::fs::write(agents::status_path(&root, "tres"), doc).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(status(&root, "tres"), "lost");
        assert!(ext.pending.is_empty());
        // Sem mudança, uma nova olhada não pergunta nada de novo.
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert!(ext.pending.is_empty());
    }

    /// Clicar na tela do pedido: "Permitir" roda o pedido que está na tela, "Recusar" recusa esse mesmo.
    #[test]
    fn the_buttons_act_on_the_request_on_screen() {
        let root = std::env::temp_dir().join(format!("idetea-agent-click-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(agents::DIR)).unwrap();
        for name in ["ola", "recusar"] {
            let doc = serde_json::json!({"format": agents::FORMAT, "version": 1, "title": name, "command": "true"});
            std::fs::write(root.join(agents::DIR).join(format!("{name}.json")), doc.to_string()).unwrap();
        }
        let mut ext = TerminalExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));

        let click = |ext: &TerminalExtension, label: String| -> Vec<Message> {
            let mut ui = iced_test::simulator(ext.view(PANEL));
            ui.click(label.as_str()).unwrap();
            ui.into_messages().collect()
        };
        let msgs = click(&ext, fl!("agent-request-allow"));
        assert!(matches!(msgs.as_slice(), [Message::Allow(r)] if r.name == "ola"), "{msgs:?}");
        for m in msgs {
            let _ = ext.update(m, &mut Context::new(&root, None));
        }
        // `true` termina logo: o script pode já ter gravado `exited`.
        assert!(["running", "exited"].contains(&status(&root, "ola").as_str()));

        let _ = ext.update(Message::ShowRequest, &mut Context::new(&root, None));
        let msgs = click(&ext, fl!("agent-request-refuse"));
        assert!(matches!(msgs.as_slice(), [Message::Refuse(n)] if n == "recusar"), "{msgs:?}");
    }

    /// O script do agente grava o status `exited` com o código, num JSON que a IDE lê.
    #[cfg(unix)]
    #[test]
    fn a_helper_runs_in_its_worktree_and_answers_in_a_file() {
        let root = std::env::temp_dir().join(format!("idetea-agent-worktree-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(agents::DIR)).unwrap();
        let git = |args: &[&str]| {
            assert!(std::process::Command::new("git").args(args).current_dir(&root).output().unwrap().status.success(), "{args:?}");
        };
        git(&["init", "-q"]);
        git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "início"]);
        let doc = serde_json::json!({"format": agents::FORMAT, "version": 1, "title": "Ajudante", "worktree": "agente/ajudante",
                                     "command": "pwd > \"$IDETEA_ANSWER\""});
        std::fs::write(root.join(agents::DIR).join("ajudante.json"), doc.to_string()).unwrap();
        let mut ext = TerminalExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        let request = ext.pending["ajudante"].clone();
        assert_eq!(request.worktree.as_deref(), Some("agente/ajudante"));
        let _ = ext.update(Message::Allow(Box::new(request)), &mut Context::new(&root, None));
        let dir = worktree::path(&root, "ajudante");
        assert!(dir.join(".git").exists(), "a IDE criou o worktree");
        let on_disk: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(agents::status_path(&root, "ajudante")).unwrap()).unwrap();
        assert_eq!(on_disk["branch"], "agente/ajudante");
        assert_eq!(on_disk["worktree"], ".idetea/worktrees/ajudante");
        assert_eq!(on_disk["answer"], ".idetea/agents/ajudante.answer.md");
        // O comando roda no worktree e escreve a resposta na raiz do projeto.
        let answer = agents::answer_path(&root, "ajudante");
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !answer.exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        let pwd = std::fs::read_to_string(&answer).unwrap();
        assert_eq!(Path::new(pwd.trim()).canonicalize().unwrap(), dir.canonicalize().unwrap());
        // O agente apaga o pedido depois do fim: a aba fica para o usuário ver; ao fechá-la, o worktree (limpo) sai e
        // o branch fica.
        while status(&root, "ajudante") != "exited" && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        std::fs::remove_file(root.join(agents::DIR).join("ajudante.json")).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(ext.agents.len(), 1);
        assert!(dir.exists(), "o worktree espera a aba fechar");
        ext.shown = Shown::Agent(0);
        let _ = ext.close_current(&mut Context::new(&root, None));
        assert!(ext.agents.is_empty());
        assert!(!dir.exists());
        let branches = std::process::Command::new("git").args(["branch", "--list"]).current_dir(&root).output().unwrap();
        assert!(String::from_utf8_lossy(&branches.stdout).contains("agente/ajudante"));
        // Um branch que não vale é recusado ao ler.
        let bad = serde_json::json!({"format": agents::FORMAT, "version": 1, "worktree": "a;b", "command": "true"});
        std::fs::write(root.join(agents::DIR).join("ruim.json"), bad.to_string()).unwrap();
        let _ = ext.update(Message::Requests(agents::scan(&root)), &mut Context::new(&root, None));
        assert_eq!(status(&root, "ruim"), "invalid");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn agent_script_writes_the_exit_code() {
        let dir = std::env::temp_dir().join(format!("idetea-agent-script-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let status = dir.join("x.status.json");
        let out = std::process::Command::new("/bin/sh")
            .args(["-c", AGENT_SCRIPT, "sh", "echo oi; exit 7", status.to_str().unwrap(), "4321", "[fim: ", "]", r#","branch":"a/b""#])
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("oi\n") && text.contains("[fim: 7]"), "{text}");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&status).unwrap()).unwrap();
        assert_eq!(v["state"], "exited");
        assert_eq!(v["code"], 7);
        assert_eq!(v["ide"], 4321);
        assert_eq!(v["format"], agents::STATUS_FORMAT);
        assert_eq!(v["branch"], "a/b", "o script grava os campos extras do status");
        assert!(!dir.join("x.status.json.tmp").exists());
    }
}
