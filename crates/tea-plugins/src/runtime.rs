//! Rodar um plugin: carregar o componente, ligar o WASI e o `host` do WIT, chamar as funções exportadas.
//!
//! Cada chamada tem um prazo (interrupção por época): um plugin preso num laço não trava a IDE; ele é desligado.
//! Os pedidos do plugin (`notify`, `set-panel`...) são juntados durante a chamada e devolvidos no fim.

use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{FsPerms, WasiCtx, WasiCtxView, WasiView};

wasmtime::component::bindgen!({ world: "plugin", path: "wit", additional_derives: [PartialEq] });

// `Event` e `UiAction` (do `types` do WIT) já ficam neste módulo pelo `bindgen!`.
pub use idetea::plugin::types::{Command, RunOutput};

/// Um "tique" da época. O prazo de uma chamada é contado em tiques.
const TICK: Duration = Duration::from_millis(10);
/// Prazo de `activate` (pode ler arquivos, montar estado).
pub const ACTIVATE_BUDGET: Duration = Duration::from_secs(2);
/// Prazo das outras chamadas (respondem a um clique: têm que ser rápidas).
pub const CALL_BUDGET: Duration = Duration::from_millis(500);

/// O que o plugin pediu durante uma chamada.
#[derive(Debug, Clone, PartialEq)]
pub enum HostRequest {
    Notify(String),
    RegisterCommand { id: String, title: String },
    OpenFile { path: String, line: u32, column: u32 },
    SetPanel(String),
    ShowPanel,
    Log(String),
    Run { id: u32, command: Command },
    /// `host.run` com programa, argumentos ou entrada grandes demais: recusado sem rodar.
    RunTooBig(u32),
}

pub struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    requests: Vec<HostRequest>,
    /// `[plugins.<id>]` do config, em JSON (o que `host.config` devolve).
    config: String,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.wasi, table: &mut self.table }
    }
}

/// Texto que vem do plugin: tamanho limitado (um plugin com defeito não enche a memória da IDE).
fn cap(mut s: String, max: usize) -> String {
    if s.len() > max {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
    s
}

impl idetea::plugin::host::Host for HostState {
    fn notify(&mut self, text: String) {
        self.requests.push(HostRequest::Notify(cap(text, 1024)));
    }
    fn register_command(&mut self, id: String, title: String) {
        self.requests.push(HostRequest::RegisterCommand { id: cap(id, 128), title: cap(title, 256) });
    }
    fn open_file(&mut self, path: String, line: u32, column: u32) {
        self.requests.push(HostRequest::OpenFile { path: cap(path, 4096), line, column });
    }
    fn set_panel(&mut self, document: String) {
        // O documento passa pelos limites do formato na leitura (256 KB); aqui só não deixa passar muito disso.
        self.requests.push(HostRequest::SetPanel(cap(document, 512 * 1024)));
    }
    fn show_panel(&mut self) {
        self.requests.push(HostRequest::ShowPanel);
    }
    fn log(&mut self, text: String) {
        self.requests.push(HostRequest::Log(cap(text, 4096)));
    }
    fn config(&mut self) -> String {
        self.config.clone()
    }
    fn run(&mut self, id: u32, command: Command) {
        // Pedidos grandes demais não chegam ao sistema; o plugin recebe o motivo em `run-finished`.
        let too_big = command.program.len() > 4096
            || command.args.len() > 1024
            || command.args.iter().map(String::len).sum::<usize>() > 256 * 1024
            || command.stdin.as_ref().is_some_and(|s| s.len() > 4 * 1024 * 1024);
        self.requests.push(if too_big { HostRequest::RunTooBig(id) } else { HostRequest::Run { id, command } });
    }
}

impl idetea::plugin::types::Host for HostState {}

/// Um motor para todos os plugins, com uma thread que avança a época a cada [`TICK`].
fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut config = Config::new();
        config.epoch_interruption(true);
        let engine = Engine::new(&config).expect("configuração do wasmtime");
        let ticker = engine.clone();
        std::thread::Builder::new()
            .name("plugins-epoch".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(TICK);
                    ticker.increment_epoch();
                }
            })
            .expect("thread da época");
        engine
    })
}

/// Um plugin carregado.
pub struct Instance {
    store: Store<HostState>,
    bindings: Plugin,
    /// Quanto custou compilar e instanciar (para medir).
    pub load_time: Duration,
}

impl Instance {
    /// Compila e instancia o componente `wasm`, com a pasta `project` visível em `/project`.
    pub fn load(wasm: &Path, project: &Path) -> Result<Self, String> {
        let start = Instant::now();
        let component = Component::from_file(engine(), wasm).map_err(|e| format!("{e:#}"))?;
        Self::instantiate(component, project, start)
    }

    /// Como [`Instance::load`], com o resultado de [`precompile`] (sem compilar de novo).
    ///
    /// # Safety
    /// `bytes` tem que ter vindo de [`precompile`] desta mesma build (o wasmtime confia no código nativo).
    pub unsafe fn load_precompiled(bytes: &[u8], project: &Path) -> Result<Self, String> {
        let start = Instant::now();
        // SAFETY: repassado a quem chama.
        let component = unsafe { Component::deserialize(engine(), bytes) }.map_err(|e| format!("{e:#}"))?;
        Self::instantiate(component, project, start)
    }

    fn instantiate(component: Component, project: &Path, start: Instant) -> Result<Self, String> {
        let engine = engine();
        let mut linker = Linker::<HostState>::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(|e| format!("{e:#}"))?;
        Plugin::add_to_linker::<HostState, HasSelf<HostState>>(&mut linker, |s| s).map_err(|e| format!("{e:#}"))?;
        let mut wasi = WasiCtx::builder();
        if std::env::var_os("IDETEA_PLUGIN_LOG").is_some() {
            wasi.inherit_stderr();
        }
        if project.is_dir() {
            wasi.preopened_dir(project, "/project", FsPerms::ReadWrite).map_err(|e| format!("{e:#}"))?;
        }
        let state =
            HostState { wasi: wasi.build(), table: ResourceTable::new(), requests: Vec::new(), config: "{}".into() };
        let mut store = Store::new(engine, state);
        store.set_epoch_deadline(ticks(ACTIVATE_BUDGET));
        let bindings = Plugin::instantiate(&mut store, &component, &linker).map_err(|e| format!("{e:#}"))?;
        Ok(Self { store, bindings, load_time: start.elapsed() })
    }

    /// Roda uma chamada com prazo e devolve os pedidos feitos nela. `Err`: o plugin falhou (trap, prazo
    /// estourado); depois disso a instância não deve mais ser usada.
    fn call<R>(
        &mut self,
        budget: Duration,
        f: impl FnOnce(&Plugin, &mut Store<HostState>) -> wasmtime::Result<R>,
    ) -> Result<(R, Vec<HostRequest>), String> {
        self.store.set_epoch_deadline(ticks(budget));
        let result = f(&self.bindings, &mut self.store);
        let requests = std::mem::take(&mut self.store.data_mut().requests);
        match result {
            Ok(r) => Ok((r, requests)),
            Err(e) if e.downcast_ref::<wasmtime::Trap>() == Some(&wasmtime::Trap::Interrupt) => {
                Err(fl!("plugin-timeout", ms = (budget.as_millis() as u64)))
            }
            Err(e) => Err(format!("{e:#}")),
        }
    }

    /// O que `host.config` devolve (objeto JSON).
    pub fn set_config(&mut self, json: String) {
        self.store.data_mut().config = json;
    }

    pub fn activate(&mut self) -> Result<Vec<HostRequest>, String> {
        let (result, requests) = self.call(ACTIVATE_BUDGET, |p, s| p.call_activate(s))?;
        result.map(|()| requests)
    }

    pub fn on_command(&mut self, id: &str) -> Result<Vec<HostRequest>, String> {
        self.call(CALL_BUDGET, |p, s| p.call_on_command(s, id)).map(|((), r)| r)
    }

    pub fn on_event(&mut self, event: &Event) -> Result<Vec<HostRequest>, String> {
        self.call(CALL_BUDGET, |p, s| p.call_on_event(s, event)).map(|((), r)| r)
    }

    pub fn on_action(&mut self, action: &UiAction) -> Result<Vec<HostRequest>, String> {
        self.call(CALL_BUDGET, |p, s| p.call_on_action(s, action)).map(|((), r)| r)
    }
}

/// Compila o `.wasm` para código nativo desta máquina (para guardar e abrir rápido depois).
pub fn precompile(wasm: &Path) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(wasm).map_err(|e| e.to_string())?;
    engine().precompile_component(&bytes).map_err(|e| format!("{e:#}"))
}

fn ticks(d: Duration) -> u64 {
    (d.as_millis() / TICK.as_millis()).max(1) as u64
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::OnceLock;

    /// Compila o plugin de exemplo (uma vez). `None` sem o alvo `wasm32-wasip2`: o teste pula.
    pub(crate) fn example_wasm() -> Option<PathBuf> {
        static WASM: OnceLock<Option<PathBuf>> = OnceLock::new();
        WASM.get_or_init(|| {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins/contador");
            let ok = Command::new(env!("CARGO"))
                .args(["build", "--release", "--quiet", "--target", "wasm32-wasip2"])
                .current_dir(&dir)
                .status()
                .is_ok_and(|s| s.success());
            let wasm = dir.join("target/wasm32-wasip2/release/contador.wasm");
            if !ok || !wasm.exists() {
                eprintln!("sem wasm32-wasip2: teste do plugin pulado");
                return None;
            }
            Some(wasm)
        })
        .clone()
    }

    fn panel(requests: &[HostRequest]) -> &str {
        requests.iter().rev().find_map(|r| if let HostRequest::SetPanel(d) = r { Some(d.as_str()) } else { None }).unwrap()
    }

    #[test]
    fn example_plugin_round_trip() {
        let Some(wasm) = example_wasm() else { return };
        let project = std::env::temp_dir().join(format!("idetea-plugin-{}", std::process::id()));
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join("a.txt"), "x").unwrap();
        std::fs::write(project.join("b.txt"), "y").unwrap();

        let mut p = Instance::load(&wasm, &project).unwrap();
        eprintln!("carregar: {:?}", p.load_time);
        let r = p.activate().unwrap();
        assert!(r.contains(&HostRequest::RegisterCommand { id: "contador.zerar".into(), title: "Contador: zerar".into() }));
        assert!(panel(&r).contains("Valor: 0"));
        assert!(panel(&r).contains("2 itens na raiz"), "o plugin vê o projeto pelo WASI: {}", panel(&r));

        let click = UiAction { name: "mais".into(), source: "mais".into(), values: "{}".into() };
        let t = Instant::now();
        let r = p.on_action(&click).unwrap();
        eprintln!("um clique: {:?}", t.elapsed());
        assert!(panel(&r).contains("Valor: 1"));
        let r = p.on_command("contador.zerar").unwrap();
        assert!(r.contains(&HostRequest::Notify("Contador zerado.".into())));
        assert!(panel(&r).contains("Valor: 0"));
        let r = p.on_event(&Event::FileSaved("a.txt".into())).unwrap();
        assert!(panel(&r).contains("1 arquivos salvos"));

        // Config e `host.run`: o pedido sai com o comando da config; o resultado volta como evento.
        p.set_config(r#"{"comando": ["echo", "olá"]}"#.into());
        let r = p.on_command("contador.rodar").unwrap();
        let run = r.iter().find_map(|r| if let HostRequest::Run { id, command } = r { Some((*id, command)) } else { None });
        let (id, command) = run.expect("pediu para rodar");
        assert_eq!((command.program.as_str(), command.args.as_slice()), ("echo", ["olá".to_string()].as_slice()));
        assert!(panel(&r).contains("Rodando echo"));
        let out = crate::process::run(&project, id, command);
        let r = p.on_event(&Event::RunFinished(out)).unwrap();
        assert!(panel(&r).contains("olá"), "{}", panel(&r));
        let r = p.on_event(&Event::ConfigChanged).unwrap();
        assert!(r.contains(&HostRequest::Log("config mudou".into())));

        // Laço infinito: interrompido no prazo.
        let t = Instant::now();
        let err = p.on_command("contador.travar").unwrap_err();
        assert!(err.contains("prazo"), "{err}");
        assert!(t.elapsed() < CALL_BUDGET * 3, "{:?}", t.elapsed());
        let _ = std::fs::remove_dir_all(&project);
    }
}
