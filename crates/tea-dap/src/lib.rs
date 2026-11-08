//! Cliente DAP (Debug Adapter Protocol) da IDE TEA. Não depende de UI.
//!
//! - Uma [`Session`] por programa sendo depurado: inicia o adaptador (ex.: `gdb -i dap`),
//!   faz o handshake, manda os breakpoints e roda o programa.
//! - Tudo que vem do adaptador chega como [`Event`] num callback, de uma thread do cliente.
//! - Linhas são **1-based**, como o DAP (pedimos `linesStartAt1`).
//!
//! Sequência (conferida com o gdb 18): `initialize` → evento `initialized` → `launch` →
//! `setBreakpoints` (um por arquivo) → `configurationDone`, que é quando o programa começa a rodar.
//! Quando ele para (`stopped`), o cliente já pede a pilha e entrega um [`Event::Stopped`] com ela.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{Value, json};
use tea_lsp::transport::{read_message, write_message};

/// Como iniciar o adaptador de debug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConfig {
    /// Nome curto, ex.: `"gdb"`.
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl AdapterConfig {
    /// `gdb` 14 ou mais novo fala DAP sozinho.
    pub fn gdb() -> Self {
        Self { name: "gdb".into(), command: "gdb".into(), args: vec!["-q".into(), "-i".into(), "dap".into()] }
    }

    /// `lldb-dap` (macOS Xcode Command Line Tools / LLVM): fala DAP direto na entrada/saída padrão.
    pub fn lldb_dap() -> Self {
        Self { name: "lldb-dap".into(), command: "lldb-dap".into(), args: Vec::new() }
    }

    /// Escolhe o adaptador de debug instalado no sistema:
    /// - No macOS, prefere `lldb-dap` (no `PATH` ou no Xcode Command Line Tools) e cai para `gdb`.
    /// - No Linux e Windows, prefere `gdb` e cai para `lldb-dap`.
    pub fn detect() -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            if let Some(path) = tea_lsp::find_in_path("lldb-dap") {
                return Some(Self { command: path.to_string_lossy().into_owned(), ..Self::lldb_dap() });
            }
            for xcode_path in [
                "/Library/Developer/CommandLineTools/usr/bin/lldb-dap",
                "/Applications/Xcode.app/Contents/Developer/usr/bin/lldb-dap",
            ] {
                if Path::new(xcode_path).is_file() {
                    return Some(Self { command: xcode_path.into(), ..Self::lldb_dap() });
                }
            }
            if let Some(path) = tea_lsp::find_in_path("gdb") {
                return Some(Self { command: path.to_string_lossy().into_owned(), ..Self::gdb() });
            }
            None
        }
        #[cfg(not(target_os = "macos"))]
        {
            if let Some(path) = tea_lsp::find_in_path("gdb") {
                return Some(Self { command: path.to_string_lossy().into_owned(), ..Self::gdb() });
            }
            if let Some(path) = tea_lsp::find_in_path("lldb-dap") {
                return Some(Self { command: path.to_string_lossy().into_owned(), ..Self::lldb_dap() });
            }
            None
        }
    }
}

/// O programa a depurar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

/// Um quadro da pilha de chamadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: i64,
    /// Nome da função.
    pub name: String,
    /// Arquivo, se o adaptador souber (código sem símbolos de debug não tem).
    pub path: Option<PathBuf>,
    /// 1-based.
    pub line: usize,
}

/// Um grupo de variáveis de um quadro (`Locals`, `Registers`...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub name: String,
    pub reference: i64,
    /// Caro de buscar: não abrir sozinho.
    pub expensive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    /// > 0: tem filhos, pedidos com [`Session::variables`].
    pub reference: i64,
}

/// Algo que veio do adaptador.
#[derive(Debug, Clone)]
pub enum Event {
    /// O programa parou. `frames[0]` é onde ele está (pode vir vazio se a pilha falhar).
    Stopped { thread_id: i64, reason: String, frames: Vec<Frame> },
    /// O programa voltou a rodar.
    Continued,
    /// Resposta de [`Session::scopes`] (vazia se falhou).
    Scopes { frame_id: i64, scopes: Vec<Scope> },
    /// Resposta de [`Session::variables`] (vazia se falhou).
    Variables { reference: i64, variables: Vec<Variable> },
    /// Saída do programa ou do adaptador. `category`: `stdout`, `stderr`, `console`...
    Output { category: String, text: String },
    /// O programa terminou (`code` quando o adaptador informa).
    Exited { code: Option<i64> },
    /// A sessão de debug acabou (programa terminou ou foi parado).
    Terminated,
    /// Um pedido falhou (ex.: `launch` de um programa que não existe).
    Error { command: String, message: String },
    /// O processo do adaptador saiu.
    AdapterExited { reason: String },
}

/// Recebe os eventos do adaptador. Chamado de uma thread do cliente: deve ser rápido.
pub type EventSink = Arc<dyn Fn(Event) + Send + Sync>;

#[derive(Clone)]
pub struct Session {
    inner: Arc<Inner>,
}

struct Inner {
    name: String,
    sink: EventSink,
    state: Mutex<State>,
    child: Mutex<Option<Child>>,
    /// Últimas linhas do stderr: explicam por que o adaptador morreu.
    last_stderr: Mutex<Vec<String>>,
}

struct State {
    stdin: Option<ChildStdin>,
    seq: i64,
    /// Pedidos esperando resposta: seq → o que fazer com ela.
    pending: HashMap<i64, Pending>,
    /// O que mandar quando o adaptador avisar `initialized`.
    launch: Option<Launch>,
    breakpoints: HashMap<PathBuf, Vec<usize>>,
    /// Thread que parou por último (os comandos de passo agem nela).
    thread_id: Option<i64>,
    alive: bool,
}

#[derive(Debug)]
enum Pending {
    Other(&'static str),
    /// Pilha pedida depois de um `stopped`.
    StackTrace { thread_id: i64, reason: String },
    Scopes(i64),
    Variables(i64),
}

impl Session {
    /// Inicia o adaptador e o programa. `breakpoints`: arquivo → linhas (1-based).
    pub fn start(
        adapter: &AdapterConfig,
        launch: Launch,
        breakpoints: HashMap<PathBuf, Vec<usize>>,
        sink: EventSink,
    ) -> std::io::Result<Self> {
        let mut child = Command::new(&adapter.command)
            .args(&adapter.args)
            .current_dir(&launch.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");

        let inner = Arc::new(Inner {
            name: adapter.name.clone(),
            sink,
            state: Mutex::new(State {
                stdin,
                seq: 0,
                pending: HashMap::new(),
                launch: Some(launch),
                breakpoints,
                thread_id: None,
                alive: true,
            }),
            child: Mutex::new(Some(child)),
            last_stderr: Mutex::new(Vec::new()),
        });

        let reader = inner.clone();
        std::thread::Builder::new().name(format!("dap-{}-out", adapter.name)).spawn(move || {
            let mut out = BufReader::new(stdout);
            let reason = loop {
                match read_message(&mut out) {
                    Ok(Some(msg)) => reader.handle(msg),
                    Ok(None) => break "o adaptador encerrou".to_string(),
                    Err(e) => break format!("erro de protocolo: {e}"),
                }
            };
            reader.lock().alive = false;
            std::thread::sleep(Duration::from_millis(50));
            let stderr = reader.last_stderr.lock().unwrap_or_else(|e| e.into_inner()).join(" | ");
            let detail = if stderr.is_empty() { String::new() } else { format!(": {stderr}") };
            (reader.sink)(Event::AdapterExited { reason: format!("{reason}{detail}") });
        })?;

        let logger = inner.clone();
        std::thread::Builder::new().name(format!("dap-{}-err", adapter.name)).spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let mut last = logger.last_stderr.lock().unwrap_or_else(|e| e.into_inner());
                if last.len() == 3 {
                    last.remove(0);
                }
                last.push(line.trim().to_string());
            }
        })?;

        let session = Session { inner };
        session.inner.lock().request(
            "initialize",
            json!({
                "clientID": "idetea",
                "clientName": "IDE TEA",
                "adapterID": adapter.name,
                "locale": "pt-BR",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "pathFormat": "path",
                "supportsRunInTerminalRequest": false,
            }),
            Pending::Other("initialize"),
        );
        Ok(session)
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn is_alive(&self) -> bool {
        self.inner.lock().alive
    }

    /// Troca os breakpoints de um arquivo (linhas 1-based). Vale na hora, mesmo com o programa rodando.
    pub fn set_breakpoints(&self, path: &Path, lines: Vec<usize>) {
        let mut st = self.inner.lock();
        st.breakpoints.insert(path.to_path_buf(), lines.clone());
        // Antes do `initialized`, os breakpoints vão junto com o launch.
        if st.launch.is_none() {
            st.send_breakpoints(path, &lines);
        }
    }

    pub fn continue_(&self) {
        self.step("continue");
    }

    /// Próxima linha, sem entrar em funções.
    pub fn next(&self) {
        self.step("next");
    }

    pub fn step_in(&self) {
        self.step("stepIn");
    }

    pub fn step_out(&self) {
        self.step("stepOut");
    }

    pub fn pause(&self) {
        let mut st = self.inner.lock();
        let thread_id = st.thread_id.unwrap_or(1);
        st.request("pause", json!({ "threadId": thread_id }), Pending::Other("pause"));
    }

    /// Grupos de variáveis de um quadro (resposta em [`Event::Scopes`]). Só vale com o programa parado.
    pub fn scopes(&self, frame_id: i64) {
        self.inner.lock().request("scopes", json!({ "frameId": frame_id }), Pending::Scopes(frame_id));
    }

    /// Variáveis de um grupo, ou filhos de uma variável (resposta em [`Event::Variables`]).
    pub fn variables(&self, reference: i64) {
        let args = json!({ "variablesReference": reference });
        self.inner.lock().request("variables", args, Pending::Variables(reference));
    }

    fn step(&self, command: &'static str) {
        let mut st = self.inner.lock();
        let Some(thread_id) = st.thread_id else { return };
        st.request(command, json!({ "threadId": thread_id }), Pending::Other(command));
        (self.inner.sink)(Event::Continued);
    }

    /// Para o programa e o adaptador. Se ele não sair sozinho em 2 s, é morto.
    pub fn stop(&self) {
        self.inner.lock().request("disconnect", json!({ "terminateDebuggee": true }), Pending::Other("disconnect"));
        let inner = self.inner.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            if let Some(child) = inner.child.lock().unwrap_or_else(|e| e.into_inner()).as_mut()
                && child.try_wait().ok().flatten().is_none()
            {
                let _ = child.kill();
                let _ = child.wait();
            }
        });
    }
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn handle(&self, msg: Value) {
        match msg["type"].as_str() {
            Some("response") => self.on_response(&msg),
            Some("event") => self.on_event(&msg),
            Some("request") => {
                // Pedidos do adaptador (ex.: runInTerminal): não suportados, responde com erro.
                let mut st = self.lock();
                let seq = st.next_seq();
                st.write(json!({
                    "seq": seq, "type": "response", "request_seq": msg["seq"], "success": false,
                    "command": msg["command"], "message": "não suportado pela IDE TEA",
                }));
            }
            _ => {}
        }
    }

    fn on_response(&self, msg: &Value) {
        let Some(seq) = msg["request_seq"].as_i64() else { return };
        let Some(pending) = self.lock().pending.remove(&seq) else { return };
        let ok = msg["success"].as_bool().unwrap_or(false);
        if !ok {
            let message = msg["message"].as_str().unwrap_or("falhou").to_string();
            let body_error = msg["body"]["error"]["format"].as_str();
            let message = body_error.map_or(message.clone(), |b| format!("{message}: {b}"));
            match pending {
                // Falhar ao pegar a pilha não impede de mostrar que parou.
                Pending::StackTrace { thread_id, reason } => {
                    (self.sink)(Event::Stopped { thread_id, reason, frames: Vec::new() })
                }
                // Variáveis que não vieram: mostra vazio, sem alarde.
                Pending::Scopes(frame_id) => (self.sink)(Event::Scopes { frame_id, scopes: Vec::new() }),
                Pending::Variables(reference) => (self.sink)(Event::Variables { reference, variables: Vec::new() }),
                Pending::Other(command) => (self.sink)(Event::Error { command: command.into(), message }),
            }
            return;
        }
        let body = &msg["body"];
        match pending {
            Pending::StackTrace { thread_id, reason } => {
                let frames = parse_frames(&body["stackFrames"]);
                (self.sink)(Event::Stopped { thread_id, reason, frames });
            }
            Pending::Scopes(frame_id) => (self.sink)(Event::Scopes { frame_id, scopes: parse_scopes(&body["scopes"]) }),
            Pending::Variables(reference) => {
                (self.sink)(Event::Variables { reference, variables: parse_variables(&body["variables"]) })
            }
            Pending::Other(_) => {}
        }
    }

    fn on_event(&self, msg: &Value) {
        let body = &msg["body"];
        match msg["event"].as_str().unwrap_or_default() {
            "initialized" => {
                let mut st = self.lock();
                let Some(launch) = st.launch.take() else { return };
                st.request(
                    "launch",
                    json!({
                        "program": launch.program,
                        "args": launch.args,
                        "cwd": launch.cwd,
                    }),
                    Pending::Other("launch"),
                );
                let files: Vec<(PathBuf, Vec<usize>)> =
                    st.breakpoints.iter().map(|(p, l)| (p.clone(), l.clone())).collect();
                for (path, lines) in files {
                    st.send_breakpoints(&path, &lines);
                }
                st.request("configurationDone", json!({}), Pending::Other("configurationDone"));
            }
            "stopped" => {
                let thread_id = body["threadId"].as_i64().unwrap_or(1);
                let reason = body["reason"].as_str().unwrap_or("pause").to_string();
                let mut st = self.lock();
                st.thread_id = Some(thread_id);
                st.request(
                    "stackTrace",
                    json!({ "threadId": thread_id, "startFrame": 0, "levels": 50 }),
                    Pending::StackTrace { thread_id, reason },
                );
            }
            "continued" => (self.sink)(Event::Continued),
            "output" => {
                let category = body["category"].as_str().unwrap_or("console").to_string();
                let text = body["output"].as_str().unwrap_or_default().to_string();
                // Telemetria do adaptador não interessa ao usuário.
                if category != "telemetry" && !text.is_empty() {
                    (self.sink)(Event::Output { category, text });
                }
            }
            "exited" => (self.sink)(Event::Exited { code: body["exitCode"].as_i64() }),
            "terminated" => {
                self.lock().thread_id = None;
                (self.sink)(Event::Terminated);
            }
            _ => {}
        }
    }
}

impl State {
    fn next_seq(&mut self) -> i64 {
        self.seq += 1;
        self.seq
    }

    fn write(&mut self, msg: Value) {
        let Some(stdin) = self.stdin.as_mut() else { return };
        if write_message(stdin, &msg).is_err() {
            self.alive = false;
            self.stdin = None;
        }
    }

    fn request(&mut self, command: &str, arguments: Value, pending: Pending) {
        let seq = self.next_seq();
        self.pending.insert(seq, pending);
        self.write(json!({ "seq": seq, "type": "request", "command": command, "arguments": arguments }));
    }

    fn send_breakpoints(&mut self, path: &Path, lines: &[usize]) {
        let breakpoints: Vec<Value> = lines.iter().map(|l| json!({ "line": l })).collect();
        self.request(
            "setBreakpoints",
            json!({
                "source": { "path": path, "name": path.file_name().map(|n| n.to_string_lossy()) },
                "breakpoints": breakpoints,
                "lines": lines,
            }),
            Pending::Other("setBreakpoints"),
        );
    }
}

fn parse_frames(frames: &Value) -> Vec<Frame> {
    frames
        .as_array()
        .into_iter()
        .flatten()
        .map(|f| Frame {
            id: f["id"].as_i64().unwrap_or(0),
            name: f["name"].as_str().unwrap_or("?").to_string(),
            path: f["source"]["path"].as_str().map(PathBuf::from),
            line: f["line"].as_u64().unwrap_or(0) as usize,
        })
        .collect()
}

fn parse_scopes(scopes: &Value) -> Vec<Scope> {
    scopes
        .as_array()
        .into_iter()
        .flatten()
        .map(|s| Scope {
            name: s["name"].as_str().unwrap_or("?").to_string(),
            reference: s["variablesReference"].as_i64().unwrap_or(0),
            expensive: s["expensive"].as_bool().unwrap_or(false),
        })
        .collect()
}

fn parse_variables(vars: &Value) -> Vec<Variable> {
    vars.as_array()
        .into_iter()
        .flatten()
        .map(|v| Variable {
            name: v["name"].as_str().unwrap_or("?").to_string(),
            value: v["value"].as_str().unwrap_or_default().to_string(),
            type_name: v["type"].as_str().filter(|t| !t.is_empty()).map(str::to_string),
            reference: v["variablesReference"].as_i64().unwrap_or(0),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn frames_without_source() {
        let frames = parse_frames(&json!([
            { "id": 0, "name": "soma", "line": 2, "source": { "path": "/p/t.rs" } },
            { "id": 1, "name": "??", "line": 0 },
        ]));
        assert_eq!(frames[0].path.as_deref(), Some(Path::new("/p/t.rs")));
        assert_eq!(frames[1].path, None);
    }

    /// Fluxo completo com o gdb de verdade (pula se não houver gdb ≥ 14 ou rustc).
    #[test]
    fn gdb_breakpoint_step_and_exit() {
        let dir = std::env::temp_dir().join(format!("idetea-dap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("t.rs");
        std::fs::write(
            &src,
            "fn soma(a: i32, b: i32) -> i32 {\n    a + b\n}\nfn main() {\n    let x = 10;\n    let y = soma(x, 5);\n    println!(\"y = {y}\");\n}\n",
        )
        .unwrap();
        let built = Command::new("rustc").args(["-g", "-o"]).arg(dir.join("t")).arg(&src).status();
        let has_dap = Command::new("gdb")
            .args(["-q", "-batch", "-ex", "python import gdb.dap"])
            .output()
            .is_ok_and(|o| o.status.success());
        if !built.is_ok_and(|s| s.success()) || !has_dap {
            eprintln!("sem rustc ou gdb com DAP: teste pulado");
            return;
        }

        let (tx, rx) = mpsc::channel();
        let sink: EventSink = Arc::new(move |e| {
            let _ = tx.send(e);
        });
        let launch = Launch { program: dir.join("t"), args: vec![], cwd: dir.clone() };
        let bps = HashMap::from([(src.clone(), vec![2])]);
        let session = Session::start(&AdapterConfig::gdb(), launch, bps, sink).unwrap();
        let wait = |pred: &dyn Fn(&Event) -> bool| loop {
            match rx.recv_timeout(Duration::from_secs(20)) {
                Ok(e) if pred(&e) => return e,
                Ok(_) => continue,
                Err(_) => panic!("tempo esgotado esperando o gdb"),
            }
        };

        let Event::Stopped { frames, reason, .. } = wait(&|e| matches!(e, Event::Stopped { .. })) else { unreachable!() };
        assert_eq!(reason, "breakpoint");
        assert_eq!((frames[0].name.contains("soma"), frames[0].line), (true, 2));
        assert_eq!(frames[0].path.as_deref(), Some(src.as_path()));
        assert!(frames.len() >= 2, "a pilha tem quem chamou");

        session.scopes(frames[0].id);
        let Event::Scopes { scopes, .. } = wait(&|e| matches!(e, Event::Scopes { .. })) else { unreachable!() };
        let mut names = Vec::new();
        for scope in scopes.iter().filter(|s| !s.expensive && s.reference > 0) {
            session.variables(scope.reference);
            let Event::Variables { variables, .. } = wait(&|e| matches!(e, Event::Variables { .. })) else {
                unreachable!()
            };
            names.extend(variables.into_iter().map(|v| format!("{}={}", v.name, v.value)));
        }
        assert!(names.contains(&"a=10".to_string()) && names.contains(&"b=5".to_string()), "{scopes:?} {names:?}");

        session.step_out();
        let Event::Stopped { frames, .. } = wait(&|e| matches!(e, Event::Stopped { .. })) else { unreachable!() };
        assert_eq!(frames[0].line, 6, "voltou para o main");

        session.continue_();
        let Event::Output { text, .. } = wait(&|e| matches!(e, Event::Output { text, .. } if text.contains("y ="))) else {
            unreachable!()
        };
        assert_eq!(text.trim(), "y = 15");
        let Event::Exited { code } = wait(&|e| matches!(e, Event::Exited { .. })) else { unreachable!() };
        assert_eq!(code, Some(0));
        wait(&|e| matches!(e, Event::Terminated));
        session.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
