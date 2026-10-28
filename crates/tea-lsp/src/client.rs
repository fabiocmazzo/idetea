//! Processo do servidor LSP, handshake, pedidos e notificações.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{Value, json};

use crate::protocol;
use crate::text::{DocText, Encoding, Position, Range};
use crate::transport::{read_message, write_message};
use crate::uri::{path_to_uri, uri_to_path};
use crate::{Diagnostic, Event, MessageLevel, ServerConfig, TextEdit};

/// Recebe os eventos do servidor. Chamado de uma thread do cliente: deve ser rápido.
pub type EventSink = Arc<dyn Fn(Event) + Send + Sync>;

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    name: String,
    sink: EventSink,
    state: Mutex<State>,
    child: Mutex<Option<Child>>,
    /// Últimas linhas do stderr: explicam por que o servidor morreu.
    last_stderr: Mutex<Vec<String>>,
}

struct State {
    stdin: Option<ChildStdin>,
    next_id: i64,
    pending: HashMap<i64, Pending>,
    docs: HashMap<PathBuf, DocText>,
    encoding: Encoding,
    /// `true` = o servidor aceita mudanças parciais; `false` = manda o texto inteiro.
    incremental: bool,
    ready: bool,
    /// Mensagens enviadas antes do handshake terminar.
    outbox: Vec<Value>,
    progress_titles: HashMap<String, String>,
    alive: bool,
}

enum Pending {
    Initialize,
    Hover(PathBuf),
    Completion(PathBuf),
    Definition,
    Formatting(PathBuf),
    Shutdown,
}

impl Client {
    /// Inicia o servidor na pasta `root`. O handshake acontece em segundo plano;
    /// o que for enviado antes disso fica na fila.
    pub fn start(config: &ServerConfig, root: &Path, sink: EventSink) -> std::io::Result<Self> {
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");

        let inner = Arc::new(Inner {
            name: config.name.clone(),
            sink,
            state: Mutex::new(State {
                stdin,
                next_id: 1,
                pending: HashMap::new(),
                docs: HashMap::new(),
                encoding: Encoding::Utf16,
                incremental: true,
                ready: false,
                outbox: Vec::new(),
                progress_titles: HashMap::new(),
                alive: true,
            }),
            child: Mutex::new(Some(child)),
            last_stderr: Mutex::new(Vec::new()),
        });

        let reader = inner.clone();
        std::thread::Builder::new().name(format!("lsp-{}-out", config.name)).spawn(move || {
            let mut out = BufReader::new(stdout);
            let reason = loop {
                match read_message(&mut out) {
                    Ok(Some(msg)) => reader.handle(msg),
                    Ok(None) => break "o servidor encerrou".to_string(),
                    Err(e) => break format!("erro de protocolo: {e}"),
                }
            };
            reader.lock().alive = false;
            let status = reader
                .child
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_mut()
                .and_then(|c| c.try_wait().ok().flatten())
                .map(|s| format!(" ({s})"))
                .unwrap_or_default();
            // Dá um instante para o stderr chegar antes de montar a explicação.
            std::thread::sleep(Duration::from_millis(50));
            let stderr = reader.last_stderr.lock().unwrap_or_else(|e| e.into_inner()).join(" | ");
            let detail = if stderr.is_empty() { String::new() } else { format!(": {stderr}") };
            (reader.sink)(Event::Exited { server: reader.name.clone(), reason: format!("{reason}{status}{detail}") });
        })?;

        let logger = inner.clone();
        std::thread::Builder::new().name(format!("lsp-{}-err", config.name)).spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                {
                    let mut last = logger.last_stderr.lock().unwrap_or_else(|e| e.into_inner());
                    if last.len() == 3 {
                        last.remove(0);
                    }
                    last.push(line.trim().to_string());
                }
                (logger.sink)(Event::Log { server: logger.name.clone(), text: line });
            }
        })?;

        let client = Client { inner };
        client.initialize(root);
        Ok(client)
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn is_alive(&self) -> bool {
        self.inner.lock().alive
    }

    fn initialize(&self, root: &Path) {
        let root_uri = path_to_uri(root);
        let folder_name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let params = json!({
            "processId": std::process::id(),
            "clientInfo": { "name": "idetea", "version": env!("CARGO_PKG_VERSION") },
            "rootUri": root_uri,
            "rootPath": root.to_string_lossy(),
            "workspaceFolders": [{ "uri": root_uri, "name": folder_name }],
            "capabilities": {
                "general": { "positionEncodings": ["utf-32", "utf-16"] },
                "workspace": { "workspaceFolders": true, "configuration": true },
                "window": { "workDoneProgress": true },
                "textDocument": {
                    "synchronization": { "didSave": true, "dynamicRegistration": false },
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "completion": { "completionItem": { "snippetSupport": false } },
                    "definition": { "linkSupport": true },
                    "formatting": {},
                    "publishDiagnostics": { "relatedInformation": false }
                }
            }
        });
        let mut st = self.inner.lock();
        let id = st.next_id;
        st.next_id += 1;
        st.pending.insert(id, Pending::Initialize);
        // O initialize passa na frente da fila.
        st.write(json!({ "jsonrpc": "2.0", "id": id, "method": "initialize", "params": params }));
    }

    // ---- Documentos --------------------------------------------------------

    pub fn did_open(&self, path: &Path, language_id: &str, version: i32, text: &str) {
        let mut st = self.inner.lock();
        st.docs.insert(path.to_path_buf(), DocText::new(text));
        st.send(notification(
            "textDocument/didOpen",
            json!({ "textDocument": {
                "uri": path_to_uri(path), "languageId": language_id, "version": version, "text": text
            }}),
        ));
    }

    /// Mudanças em sequência, cada uma relativa ao texto depois da anterior (posições em caracteres).
    pub fn did_change(&self, path: &Path, version: i32, changes: &[(Range, String)]) {
        let mut st = self.inner.lock();
        let enc = st.encoding;
        let incremental = st.incremental;
        let Some(doc) = st.docs.get_mut(path) else { return };
        let mut content_changes = Vec::with_capacity(changes.len());
        for (range, text) in changes {
            if incremental {
                let r = doc.range_to_server(*range, enc);
                content_changes.push(json!({ "range": protocol::range_json(r), "text": text }));
            }
            doc.apply(*range, text);
        }
        if !incremental {
            content_changes.push(json!({ "text": doc.text() }));
        }
        st.send(notification(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": path_to_uri(path), "version": version },
                "contentChanges": content_changes
            }),
        ));
    }

    pub fn did_save(&self, path: &Path, text: &str) {
        let mut st = self.inner.lock();
        if let Some(doc) = st.docs.get_mut(path) {
            // Ressincroniza a cópia: se algo se perdeu, o save conserta.
            *doc = DocText::new(text);
        }
        st.send(notification("textDocument/didSave", json!({ "textDocument": { "uri": path_to_uri(path) }, "text": text })));
    }

    pub fn did_close(&self, path: &Path) {
        let mut st = self.inner.lock();
        if st.docs.remove(path).is_some() {
            st.send(notification("textDocument/didClose", json!({ "textDocument": { "uri": path_to_uri(path) } })));
        }
    }

    /// Linha do documento aberto (cópia mantida pelo cliente). Útil para achar a palavra no cursor.
    pub fn line_text(&self, path: &Path, line: u32) -> Option<String> {
        self.inner.lock().docs.get(path)?.line(line).map(str::to_string)
    }

    // ---- Pedidos -----------------------------------------------------------

    pub fn hover(&self, path: &Path, position: Position) {
        self.position_request("textDocument/hover", path, position, Pending::Hover(path.to_path_buf()));
    }

    pub fn completion(&self, path: &Path, position: Position) {
        self.position_request("textDocument/completion", path, position, Pending::Completion(path.to_path_buf()));
    }

    pub fn definition(&self, path: &Path, position: Position) {
        self.position_request("textDocument/definition", path, position, Pending::Definition);
    }

    pub fn formatting(&self, path: &Path, tab_size: u32, insert_spaces: bool) {
        let params = json!({
            "textDocument": { "uri": path_to_uri(path) },
            "options": { "tabSize": tab_size, "insertSpaces": insert_spaces }
        });
        self.inner.lock().request("textDocument/formatting", params, Pending::Formatting(path.to_path_buf()));
    }

    fn position_request(&self, method: &str, path: &Path, position: Position, pending: Pending) {
        let mut st = self.inner.lock();
        let enc = st.encoding;
        let Some(doc) = st.docs.get(path) else { return };
        let p = doc.to_server(position, enc);
        let params = json!({ "textDocument": { "uri": path_to_uri(path) }, "position": protocol::position_json(p) });
        st.request(method, params, pending);
    }

    /// Pede para o servidor encerrar. Se ele não sair em 2 s, é morto.
    pub fn shutdown(&self) {
        {
            let mut st = self.inner.lock();
            if !st.alive {
                return;
            }
            st.request("shutdown", Value::Null, Pending::Shutdown);
        }
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

fn notification(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "method": method, "params": params })
}

impl State {
    fn write(&mut self, msg: Value) {
        let Some(stdin) = self.stdin.as_mut() else { return };
        if write_message(stdin, &msg).is_err() {
            self.alive = false;
            self.stdin = None;
        }
    }

    /// Envia, ou guarda na fila se o handshake ainda não terminou.
    fn send(&mut self, msg: Value) {
        if self.ready {
            self.write(msg);
        } else {
            self.outbox.push(msg);
        }
    }

    fn request(&mut self, method: &str, params: Value, pending: Pending) {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, pending);
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
    }

    /// Converte uma posição do servidor para caracteres. Lê o arquivo do disco se ele não estiver aberto.
    fn to_chars(&self, path: &Path, r: Range, disk: &mut HashMap<PathBuf, Option<DocText>>) -> Range {
        if self.encoding == Encoding::Utf32 {
            return r;
        }
        if let Some(doc) = self.docs.get(path) {
            return doc.range_from_server(r, self.encoding);
        }
        match disk.entry(path.to_path_buf()).or_insert_with(|| DocText::from_file(path)) {
            Some(doc) => doc.range_from_server(r, self.encoding),
            None => r,
        }
    }
}

impl Inner {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Trata uma mensagem do servidor. Os eventos são emitidos depois de soltar o lock.
    fn handle(&self, msg: Value) {
        let events = {
            let mut st = self.lock();
            let method = msg.get("method").and_then(Value::as_str).map(str::to_string);
            match (method, msg.get("id").cloned()) {
                (Some(method), Some(id)) => {
                    self.reply_to_server(&mut st, &method, id, msg.get("params"));
                    Vec::new()
                }
                (Some(method), None) => self.notification(&mut st, &method, msg.get("params").unwrap_or(&Value::Null)),
                (None, Some(id)) => self.response(&mut st, &id, &msg),
                (None, None) => Vec::new(),
            }
        };
        for e in events {
            (self.sink)(e);
        }
    }

    /// Pedidos que o servidor faz para o cliente.
    fn reply_to_server(&self, st: &mut State, method: &str, id: Value, params: Option<&Value>) {
        let result = match method {
            // Um `null` por item pedido: "use seus padrões".
            "workspace/configuration" => {
                let n = params.and_then(|p| p.get("items")).and_then(Value::as_array).map_or(0, Vec::len);
                Value::Array(vec![Value::Null; n])
            }
            "window/workDoneProgress/create" | "client/registerCapability" | "client/unregisterCapability" => Value::Null,
            "workspace/workspaceFolders" => Value::Null,
            _ => {
                st.write(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "método não suportado" } }));
                return;
            }
        };
        st.write(json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }

    fn notification(&self, st: &mut State, method: &str, params: &Value) -> Vec<Event> {
        let server = self.name.clone();
        match method {
            "textDocument/publishDiagnostics" => {
                let Some((uri, raw)) = protocol::diagnostics(params) else { return Vec::new() };
                let Some(path) = uri_to_path(&uri) else { return Vec::new() };
                let mut disk = HashMap::new();
                let diagnostics = raw
                    .into_iter()
                    .map(|d| Diagnostic {
                        range: st.to_chars(&path, d.range, &mut disk),
                        severity: d.severity,
                        message: d.message,
                        source: d.source,
                        code: d.code,
                    })
                    .collect();
                vec![Event::Diagnostics { server, path, diagnostics }]
            }
            "$/progress" => {
                let token = match params.get("token") {
                    Some(Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => return Vec::new(),
                };
                let Some(value) = params.get("value") else { return Vec::new() };
                let kind = value.get("kind").and_then(Value::as_str).unwrap_or_default();
                let message = value.get("message").and_then(Value::as_str).map(str::to_string);
                let percentage = value.get("percentage").and_then(Value::as_u64).map(|p| p as u32);
                let title = match kind {
                    "begin" => {
                        let t = value.get("title").and_then(Value::as_str).unwrap_or_default().to_string();
                        st.progress_titles.insert(token.clone(), t.clone());
                        t
                    }
                    "end" => st.progress_titles.remove(&token).unwrap_or_default(),
                    _ => st.progress_titles.get(&token).cloned().unwrap_or_default(),
                };
                vec![Event::Progress { server, title, message, percentage, done: kind == "end" }]
            }
            "window/showMessage" => {
                let level = match params.get("type").and_then(Value::as_u64) {
                    Some(1) => MessageLevel::Error,
                    Some(2) => MessageLevel::Warning,
                    _ => MessageLevel::Info,
                };
                let text = params.get("message").and_then(Value::as_str).unwrap_or_default().to_string();
                vec![Event::Message { server, level, text }]
            }
            "window/logMessage" => {
                let text = params.get("message").and_then(Value::as_str).unwrap_or_default().to_string();
                vec![Event::Log { server, text }]
            }
            _ => Vec::new(),
        }
    }

    fn response(&self, st: &mut State, id: &Value, msg: &Value) -> Vec<Event> {
        let Some(pending) = id.as_i64().and_then(|id| st.pending.remove(&id)) else { return Vec::new() };
        let server = self.name.clone();
        if let Some(err) = msg.get("error") {
            let text = err.get("message").and_then(Value::as_str).unwrap_or("erro").to_string();
            // Pedido cancelado ou conteúdo mudou: normal, não incomoda o usuário.
            let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
            if matches!(pending, Pending::Initialize) {
                return vec![Event::Exited { server, reason: format!("falha no initialize: {text}") }];
            }
            if code == -32800 || code == -32801 {
                return Vec::new();
            }
            return vec![Event::Log { server, text: format!("erro na resposta: {text}") }];
        }
        let result = msg.get("result").unwrap_or(&Value::Null);
        match pending {
            Pending::Initialize => {
                let caps = result.get("capabilities").unwrap_or(&Value::Null);
                let enc = caps
                    .get("positionEncoding")
                    .or_else(|| result.get("offsetEncoding"))
                    .and_then(Value::as_str)
                    .unwrap_or("utf-16");
                st.encoding = if enc == "utf-32" { Encoding::Utf32 } else { Encoding::Utf16 };
                let change = match caps.get("textDocumentSync") {
                    Some(Value::Number(n)) => n.as_u64(),
                    Some(obj) => obj.get("change").and_then(Value::as_u64),
                    None => None,
                };
                st.incremental = change != Some(1);
                st.ready = true;
                st.write(notification("initialized", json!({})));
                for queued in std::mem::take(&mut st.outbox) {
                    st.write(queued);
                }
                vec![Event::Ready { server }]
            }
            Pending::Hover(path) => match protocol::hover_text(result) {
                Some(text) => vec![Event::Hover { path, text }],
                None => Vec::new(),
            },
            Pending::Completion(path) => {
                let items = protocol::completion_items(result);
                vec![Event::Completion { path, items }]
            }
            Pending::Definition => match protocol::first_location(result) {
                Some((uri, range)) => match uri_to_path(&uri) {
                    Some(path) => {
                        let r = st.to_chars(&path, range, &mut HashMap::new());
                        vec![Event::Definition { path, position: r.start }]
                    }
                    None => vec![Event::DefinitionNotFound],
                },
                None => vec![Event::DefinitionNotFound],
            },
            Pending::Formatting(path) => {
                let mut disk = HashMap::new();
                let edits = protocol::text_edits(result)
                    .into_iter()
                    .map(|(r, new_text)| TextEdit { range: st.to_chars(&path, r, &mut disk), new_text })
                    .collect();
                vec![Event::Formatting { path, edits }]
            }
            Pending::Shutdown => {
                st.write(notification("exit", Value::Null));
                Vec::new()
            }
        }
    }
}
