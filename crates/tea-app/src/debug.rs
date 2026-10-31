//! Sessão de debug (DAP) do app: uma por vez. O protocolo fica em `tea-dap`; aqui só o estado
//! que a interface precisa (onde parou, saída do programa) e a escolha do programa.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use iced::futures::channel::mpsc::UnboundedSender;
use tea_dap::{AdapterConfig, Event, Frame, Launch, Session};

pub struct Debugger {
    tx: UnboundedSender<(u64, Event)>,
    session: Option<Session>,
    /// Cada sessão tem um número: eventos atrasados de uma sessão velha são ignorados.
    generation: u64,
    /// Programa escolhido (vale até o app fechar).
    pub program: Option<PathBuf>,
    /// Onde o programa está parado, se estiver.
    pub stopped: Option<Vec<Frame>>,
    /// Compilando antes de iniciar (o resultado chega em `Message::DebugBuilt`).
    pub building: bool,
}

impl Debugger {
    pub fn new(tx: UnboundedSender<(u64, Event)>) -> Self {
        Self { tx, session: None, generation: 0, program: None, stopped: None, building: false }
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
        args: Vec<String>,
        cwd: PathBuf,
        breakpoints: HashMap<PathBuf, Vec<usize>>,
    ) -> Result<(), String> {
        let adapter = AdapterConfig::gdb();
        if tea_lsp::find_in_path(&adapter.command).is_none() {
            return Err(fl!("debug-no-gdb"));
        }
        if !program.is_file() {
            return Err(fl!("debug-program-not-found", path = program.display().to_string()));
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
        let launch = Launch { program: program.clone(), args, cwd };
        let session = Session::start(&adapter, launch, breakpoints, sink)
            .map_err(|e| fl!("debug-gdb-start-error", error = e.to_string()))?;
        self.session = Some(session);
        self.program = Some(program);
        self.stopped = None;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.building = false;
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

    /// Quadro no formato que o painel recebe.
    pub fn to_panel(frame: &Frame) -> tea_core::debug::Frame {
        tea_core::debug::Frame { id: frame.id, name: frame.name.clone(), path: frame.path.clone(), line: frame.line }
    }
}

/// Motivo da parada, no idioma da interface.
pub fn reason_label(reason: &str) -> String {
    match reason {
        "breakpoint" | "function breakpoint" | "data breakpoint" => fl!("debug-reason-breakpoint"),
        "step" => fl!("debug-reason-step"),
        "pause" => fl!("debug-reason-pause"),
        "exception" => fl!("debug-reason-exception"),
        "entry" => fl!("debug-reason-entry"),
        "goto" => fl!("debug-reason-goto"),
        other => other.to_string(),
    }
}
