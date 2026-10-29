//! Sessão de debug (DAP) do app: uma por vez. O protocolo fica em `tea-dap`; aqui só o estado
//! que a interface precisa (onde parou, saída do programa) e a escolha do programa.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use iced::futures::channel::mpsc::UnboundedSender;
use tea_dap::{AdapterConfig, Event, Frame, Launch, Session};

/// Linhas de saída guardadas (o painel de debug mostra; o resto é descartado).
const MAX_OUTPUT_LINES: usize = 5000;

pub struct Debugger {
    tx: UnboundedSender<(u64, Event)>,
    session: Option<Session>,
    /// Cada sessão tem um número: eventos atrasados de uma sessão velha são ignorados.
    generation: u64,
    /// Programa escolhido (vale até o app fechar).
    pub program: Option<PathBuf>,
    /// Onde o programa está parado, se estiver.
    pub stopped: Option<Vec<Frame>>,
    pub output: Vec<String>,
}

impl Debugger {
    pub fn new(tx: UnboundedSender<(u64, Event)>) -> Self {
        Self { tx, session: None, generation: 0, program: None, stopped: None, output: Vec::new() }
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn is_running(&self) -> bool {
        self.session.is_some()
    }

    /// O evento é da sessão atual?
    pub fn is_current(&self, generation: u64) -> bool {
        self.session.is_some() && generation == self.generation
    }

    /// Inicia o `gdb` com o programa. `breakpoints`: arquivo → linhas (1-based).
    pub fn start(
        &mut self,
        program: PathBuf,
        cwd: PathBuf,
        breakpoints: HashMap<PathBuf, Vec<usize>>,
    ) -> Result<(), String> {
        let adapter = AdapterConfig::gdb();
        if tea_lsp::find_in_path(&adapter.command).is_none() {
            return Err("gdb não encontrado no PATH (o debug usa o gdb 14 ou mais novo)".into());
        }
        if !program.is_file() {
            return Err(format!("programa não encontrado: {}", program.display()));
        }
        self.stop();
        self.generation += 1;
        let (generation, tx) = (self.generation, self.tx.clone());
        let sink: tea_dap::EventSink = Arc::new(move |e| {
            if std::env::var_os("IDETEA_DAP_LOG").is_some() {
                eprintln!("[dap] {e:?}");
            }
            let _ = tx.unbounded_send((generation, e));
        });
        let launch = Launch { program: program.clone(), args: Vec::new(), cwd };
        let session = Session::start(&adapter, launch, breakpoints, sink)
            .map_err(|e| format!("não foi possível iniciar o gdb: {e}"))?;
        self.session = Some(session);
        self.program = Some(program);
        self.stopped = None;
        self.output.clear();
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(s) = self.session.take() {
            s.stop();
        }
        self.stopped = None;
    }

    /// A sessão acabou sozinha (programa terminou, gdb saiu).
    pub fn ended(&mut self) {
        self.session = None;
        self.stopped = None;
    }

    pub fn push_output(&mut self, text: &str) {
        self.output.extend(text.lines().map(str::to_string));
        if self.output.len() > MAX_OUTPUT_LINES {
            let extra = self.output.len() - MAX_OUTPUT_LINES;
            self.output.drain(..extra);
        }
    }
}

/// Motivo da parada em português.
pub fn reason_label(reason: &str) -> &str {
    match reason {
        "breakpoint" | "function breakpoint" | "data breakpoint" => "breakpoint",
        "step" => "passo",
        "pause" => "pausado",
        "exception" => "exceção",
        "entry" => "início",
        "goto" => "salto",
        other => other,
    }
}
