//! Um plugin visto pela IDE: uma extensão como as de dentro, com o painel desenhado pelo `tea-declarative-ui`.

use std::path::{Path, PathBuf};

use iced::widget::{column, scrollable, text};
use iced::{Element, Fill, Task};
use tea_core::icons::Icon;
use tea_core::{Context, CoreEvent, Extension, PanelId};
use tea_declarative_ui::{Document, Limits, State, UiMessage};

use crate::manifest::Manifest;
use crate::runtime::{Event, HostRequest, Instance, UiAction};

#[derive(Debug, Clone)]
pub enum Message {
    Ui(UiMessage),
}

pub struct PluginExtension {
    /// `&'static` como o das extensões de dentro: o texto vive até o fim do programa (um por plugin).
    id: &'static str,
    panel: Option<PanelId>,
    manifest: Option<Manifest>,
    dir: PathBuf,
    root: PathBuf,
    instance: Option<Instance>,
    doc: Option<Result<Document, String>>,
    state: State,
    /// Por que o plugin está desligado (manifesto ruim, erro ao carregar, trap, prazo estourado).
    failed: Option<String>,
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl PluginExtension {
    /// Plugin na pasta `dir` (o manifesto já lido, ou o erro dele).
    pub fn new(dir: &Path, manifest: Result<Manifest, String>) -> Self {
        let folder = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let (id, manifest, failed) = match manifest {
            Ok(m) => (m.id.clone(), Some(m), None),
            Err(e) => (folder, None, Some(e)),
        };
        let panel = manifest.as_ref().and_then(|m| m.panel.as_ref()).map(|_| PanelId(leak(format!("plugin.{id}"))));
        Self {
            id: leak(id),
            panel,
            manifest,
            dir: dir.to_path_buf(),
            root: PathBuf::new(),
            instance: None,
            doc: None,
            state: State::default(),
            failed,
        }
    }

    fn name(&self) -> &str {
        self.manifest.as_ref().map_or(self.id, |m| m.name.as_str())
    }

    /// O plugin falhou: desliga e avisa. A instância não é mais usada (pode ter ficado num estado ruim).
    fn fail(&mut self, error: String, ctx: &mut Context) {
        ctx.notify(fl!("plugin-failed", name = self.name().to_string(), error = error.clone()));
        self.instance = None;
        self.failed = Some(error);
    }

    fn apply(&mut self, result: Result<Vec<HostRequest>, String>, ctx: &mut Context) {
        let requests = match result {
            Ok(r) => r,
            Err(e) => return self.fail(e, ctx),
        };
        for r in requests {
            match r {
                HostRequest::Notify(t) => ctx.notify(format!("{}: {t}", self.name())),
                HostRequest::RegisterCommand { id, title } => {
                    if id.strip_prefix(self.id).is_some_and(|rest| rest.starts_with('.')) {
                        ctx.register_command(id, title);
                    } else {
                        log(self.id, &fl!("plugin-command-prefix", id = id, prefix = self.id));
                    }
                }
                HostRequest::OpenFile { path, line, column } => {
                    open(ctx, self.root.join(path), line.into(), column.into());
                }
                HostRequest::SetPanel(json) => {
                    let doc = tea_declarative_ui::parse(json.as_bytes(), &Limits::default());
                    if let Ok(d) = &doc {
                        match &self.doc {
                            Some(Ok(_)) => self.state.apply(d),
                            _ => self.state = State::new(d),
                        }
                    }
                    self.doc = Some(doc);
                }
                HostRequest::ShowPanel => {
                    if let Some(panel) = self.panel {
                        ctx.show_panel(panel);
                    }
                }
                HostRequest::Log(t) => log(self.id, &t),
            }
        }
    }

    fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy().into_owned()
    }
}

/// Abre na linha e coluna (começando em 1; linha 0 = só abre).
fn open(ctx: &mut Context, path: PathBuf, line: u64, column: u64) {
    if line == 0 {
        ctx.open_file(path);
    } else {
        ctx.open_file_at(path, line as usize, column.max(1) as usize);
    }
}

fn log(id: &str, text: &str) {
    if std::env::var_os("IDETEA_PLUGIN_LOG").is_some() {
        eprintln!("[plugin {id}] {text}");
    }
}

impl Extension for PluginExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        self.id
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        if let (Some(panel), Some(m)) = (self.panel, &self.manifest) {
            let title = m.panel.as_ref().map(|p| p.title.clone()).unwrap_or_default();
            ctx.register_panel_hidden(panel, title, m.slot());
            ctx.panel_icon(panel, Icon::Preview);
        }
        if let Some(e) = self.failed.clone() {
            ctx.notify(fl!("plugin-failed", name = self.name().to_string(), error = e));
            return Task::none();
        }
        let Some(m) = &self.manifest else { return Task::none() };
        match Instance::load(&m.wasm_path(&self.dir), &self.root) {
            Ok(mut instance) => {
                log(self.id, &format!("carregado em {:?}", instance.load_time));
                let result = instance.activate();
                self.instance = Some(instance);
                self.apply(result, ctx);
            }
            Err(e) => self.fail(e, ctx),
        }
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if let Some(instance) = &mut self.instance {
            let result = instance.on_command(command);
            self.apply(result, ctx);
        }
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, ctx: &mut Context) -> Task<Message> {
        let event = match event {
            CoreEvent::FileSaved(p) => Event::FileSaved(self.relative(p)),
            CoreEvent::ActiveFileChanged(p) => Event::ActiveFile(p.as_deref().map(|p| self.relative(p))),
            _ => return Task::none(),
        };
        if let Some(instance) = &mut self.instance {
            let result = instance.on_event(&event);
            self.apply(result, ctx);
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Ui(UiMessage::Set { path, value }) => self.state.set(&path, value),
            Message::Ui(UiMessage::Open { path, line, column }) => open(ctx, self.root.join(path), line, column),
            Message::Ui(UiMessage::Press(source)) => {
                let Some(Ok(doc)) = &self.doc else { return Task::none() };
                let Some((name, values)) = self.state.press(doc, &source) else { return Task::none() };
                let values = serde_json::to_string(&values).unwrap_or_else(|_| "{}".into());
                if let Some(instance) = &mut self.instance {
                    let result = instance.on_action(&UiAction { name, source, values });
                    self.apply(result, ctx);
                }
            }
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let mut out = column![].spacing(12).padding(12).width(Fill);
        if let Some(e) = &self.failed {
            out = out.push(text(fl!("plugin-off", error = e.clone())).size(13).style(text::danger).width(Fill));
        }
        match &self.doc {
            Some(Ok(doc)) => {
                if let Some(title) = &doc.title {
                    out = out.push(text(title).size(16));
                }
                for w in &doc.warnings {
                    out = out.push(text(w).size(12).style(text::warning).width(Fill));
                }
                out = out.push(tea_declarative_ui::view(doc, &self.state).map(Message::Ui));
            }
            Some(Err(e)) => {
                out = out.push(text(fl!("plugin-bad-panel", error = e.clone())).size(13).style(text::danger).width(Fill));
            }
            None => {}
        }
        scrollable(out).height(Fill).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::simulator;

    fn messages(ext: &PluginExtension, click: &str) -> Vec<Message> {
        let mut ui = simulator(ext.view(ext.panel.unwrap()));
        ui.click(click).unwrap();
        ui.into_messages().collect()
    }

    fn shows(ext: &PluginExtension, label: &str) -> bool {
        simulator(ext.view(ext.panel.unwrap())).find(label).is_ok()
    }

    #[test]
    fn installed_plugin_works_as_an_extension() {
        let Some(wasm) = crate::runtime::tests::example_wasm() else { return };
        let base = std::env::temp_dir().join(format!("idetea-plugins-{}", std::process::id()));
        let dir = base.join("plugins/contador");
        let project = base.join("projeto");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins/contador/plugin.toml");
        std::fs::copy(example, dir.join("plugin.toml")).unwrap();
        std::fs::copy(&wasm, dir.join("contador.wasm")).unwrap();

        assert!(crate::discover(&base.join("plugins"), &["contador"])[0].failed.is_some(), "id já usado");
        let mut ext = crate::discover(&base.join("plugins"), &["git"]).remove(0);
        let mut ctx = Context::new(&project, None);
        let _ = ext.activate(&mut ctx);
        assert!(ext.failed.is_none(), "{:?}", ext.failed);
        let requests = ctx.into_requests();
        assert!(requests.iter().any(|r| matches!(r, tea_core::Request::RegisterCommand { id, .. } if id == "contador.zerar")));
        assert!(shows(&ext, "Valor: 0"));

        for m in messages(&ext, "+1") {
            let _ = ext.update(m, &mut Context::new(&project, None));
        }
        assert!(shows(&ext, "Valor: 1"), "o clique foi ao plugin e o painel voltou");

        // Plugin preso: desligado, com o motivo no painel; a IDE segue.
        let mut ctx = Context::new(&project, None);
        let _ = ext.on_command("contador.travar", &mut ctx);
        assert!(ext.failed.is_some() && ext.instance.is_none());
        assert!(ctx.into_requests().iter().any(|r| matches!(r, tea_core::Request::Notify(_))));
        let _ = std::fs::remove_dir_all(&base);
    }
}
