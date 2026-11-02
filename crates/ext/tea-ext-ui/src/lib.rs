//! Painel "Interfaces": um agente (ou qualquer script) escreve `.idetea/ui/<nome>.json` e a IDE desenha a
//! interface; quando o usuário clica num botão, a resposta vai para `.idetea/ui/responses/<nome>/`.
//!
//! O formato está em `docs/interfaces/formato.md`; as decisões, em `docs/rfc/0001-interfaces-declarativas.md`.
//! Uma thread olha a pasta (só `stat` a cada 250 ms) e lê os documentos fora da interface.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

pub mod store;
pub mod teach;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iced::futures::channel::mpsc;
use iced::widget::{Column, button, column, row, scrollable, text};
use iced::{Element, Fill, Subscription, Task};
use serde::{Deserialize, Serialize};
use tea_core::icons::Icon;
use tea_core::{ButtonKind, Context, Dialog, Extension, PanelId, Slot};
use tea_declarative_ui::{Document, Limits, Response, State, UiMessage, timestamp};

use store::SurfaceFile;

const PANEL: PanelId = PanelId("ui");
const WATCH_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub enum Message {
    /// A pasta como a thread de observação leu.
    Snapshot(Arc<Vec<SurfaceFile>>),
    Select(String),
    Ui(String, UiMessage),
    /// Mostra o diálogo de "ensinar os agentes".
    AskTeach,
    /// Confirmado: grava as instruções.
    Teach,
}

/// O que a IDE guarda de uma interface (sobrevive a reiniciar a IDE).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Saved {
    state: State,
    /// Último `seq` usado nas respostas desta interface.
    seq: u64,
}

#[derive(Default)]
pub struct UiExtension {
    root: PathBuf,
    files: Arc<Vec<SurfaceFile>>,
    saved: BTreeMap<String, Saved>,
    selected: Option<String>,
    /// Já chegou a primeira leitura (as que vêm depois avisam sobre interfaces novas).
    loaded: bool,
    /// Erro ao criar `.idetea/ui/`.
    problem: Option<String>,
    counter: u64,
}

impl UiExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn state_file(&self) -> Option<PathBuf> {
        let base = std::env::var_os("XDG_STATE_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("state")))?;
        Some(base.join("idetea").join("ui").join(project_key(&self.root)))
    }

    fn load(&mut self) {
        let Some(path) = self.state_file() else { return };
        if let Ok(text) = std::fs::read_to_string(path) {
            self.saved = serde_json::from_str(&text).unwrap_or_default();
        }
    }

    /// Grava o estado. Falha não atrapalha: no pior caso o que foi digitado se perde ao reiniciar.
    fn save(&self) {
        let Some(path) = self.state_file() else { return };
        if let Some(dir) = path.parent()
            && std::fs::create_dir_all(dir).is_ok()
            && let Ok(text) = serde_json::to_string(&self.saved)
        {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(tmp, path);
            }
        }
    }

    fn file(&self, name: &str) -> Option<&SurfaceFile> {
        self.files.iter().find(|f| f.name == name)
    }

    fn snapshot(&mut self, files: Arc<Vec<SurfaceFile>>, ctx: &mut Context) {
        let first = !self.loaded;
        self.loaded = true;
        let mut changed = None;
        for f in files.iter() {
            let Ok(doc) = &f.doc else { continue };
            if f.tracked {
                continue;
            }
            let before = self.file(&f.name).and_then(|old| old.doc.as_ref().ok()).map(|d| d.revision);
            match self.saved.get_mut(&f.name) {
                Some(saved) => saved.state.apply(doc),
                None => {
                    self.saved.insert(f.name.clone(), Saved { state: State::new(doc), seq: 0 });
                }
            }
            // Revisão nova; ou, ao abrir a IDE, uma pergunta que ficou sem resposta (o agente pode ter escrito
            // antes de a IDE abrir).
            let waiting = first && f.pending.is_empty() && !self.saved[&f.name].state.locked(doc);
            if waiting || (!first && before != Some(doc.revision)) {
                changed = Some((f.name.clone(), title(doc, &f.name).to_string(), before.is_none()));
            }
        }
        // Interface apagada pelo agente: acabou; o estado dela não serve mais.
        self.saved.retain(|name, _| files.iter().any(|f| &f.name == name));
        self.files = files;
        if let Some((name, title, new)) = changed {
            ctx.notify(if new { fl!("ui-new", title = title) } else { fl!("ui-updated", title = title) });
            ctx.show_panel(PANEL);
            self.selected = Some(name);
        }
        if self.selected.as_ref().is_none_or(|s| self.file(s).is_none()) {
            self.selected = self.files.first().map(|f| f.name.clone());
        }
        self.save();
    }

    fn press(&mut self, name: &str, source: &str, ctx: &mut Context) {
        let Some(f) = self.files.iter().find(|f| f.name == name && !f.tracked) else { return };
        let Ok(doc) = &f.doc else { return };
        let Some(saved) = self.saved.get_mut(name) else { return };
        let before = saved.state.clone();
        let Some((action, values)) = saved.state.press(doc, source) else { return };
        let seq = saved.seq.max(f.pending.iter().map(|p| p.0).max().unwrap_or(0)) + 1;
        self.counter += 1;
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
        let event = format!("{millis:x}-{:x}-{}", std::process::id(), self.counter);
        let response =
            Response::new(name, doc.revision, seq, event.clone(), action, source, timestamp(SystemTime::now()), values);
        match store::write_response(&self.root, &response) {
            Ok(_) => {
                saved.seq = seq;
                // Já conta como pendente, sem esperar a próxima leitura da pasta.
                if let Some(f) = Arc::make_mut(&mut self.files).iter_mut().find(|f| f.name == name) {
                    f.pending.push((seq, event));
                }
            }
            Err(e) => {
                saved.state = before;
                ctx.notify(fl!("ui-write-failed", error = e.to_string()));
            }
        }
        self.save();
    }

    fn teach_text() -> &'static str {
        teach::instructions(&fl!("ui-agents-lang"))
    }

    fn ask_teach(&self, ctx: &mut Context) {
        match teach::plan(&self.root, Self::teach_text()) {
            Ok(plan) if plan.is_empty() => ctx.notify(fl!("ui-teach-up-to-date")),
            Ok(plan) => {
                let files: Vec<String> = plan
                    .iter()
                    .map(|c| if c.new { fl!("ui-teach-file-new", name = c.name.clone()) } else { c.name.clone() })
                    .collect();
                ctx.show_dialog(
                    Dialog::new(fl!("ui-teach-title"), fl!("ui-teach-detail", files = files.join(", ")))
                        .button(fl!("ui-teach-write"), ButtonKind::Primary, Message::Teach)
                        .cancel(fl!("ui-cancel")),
                );
            }
            Err(e) => ctx.notify(fl!("ui-teach-failed", error = e.to_string())),
        }
    }

    fn teach(&self, ctx: &mut Context) {
        // Calcula de novo: o arquivo pode ter mudado enquanto o diálogo estava aberto.
        let result = teach::plan(&self.root, Self::teach_text()).and_then(|plan| teach::apply(&plan).map(|()| plan));
        match result {
            Ok(plan) => {
                for c in &plan {
                    ctx.file_changed(c.path.clone());
                }
                if let Some(first) = plan.first() {
                    ctx.open_file(first.path.clone());
                    let names: Vec<&str> = plan.iter().map(|c| c.name.as_str()).collect();
                    ctx.notify(fl!("ui-teach-done", files = names.join(", ")));
                }
            }
            Err(e) => ctx.notify(fl!("ui-teach-failed", error = e.to_string())),
        }
    }

    fn surface_view<'a>(&'a self, f: &'a SurfaceFile) -> Element<'a, Message> {
        let path = format!(".idetea/ui/{}.json", f.name);
        if f.tracked {
            return column![
                text(&f.name).size(16),
                text(fl!("ui-tracked", path = path)).size(13).style(text::warning).width(Fill),
            ]
            .spacing(8)
            .into();
        }
        let doc: &Document = match &f.doc {
            Ok(doc) => doc,
            Err(e) => {
                return column![
                    text(&f.name).size(16),
                    text(path).size(12).style(text::secondary),
                    text(fl!("ui-invalid", error = e.clone())).size(13).style(text::danger).width(Fill),
                ]
                .spacing(8)
                .into();
            }
        };
        let Some(saved) = self.saved.get(&f.name) else { return column![].into() };
        let origin = match &doc.source {
            Some(source) => fl!("ui-origin-source", path = path, source = source.clone()),
            None => fl!("ui-origin", path = path),
        };
        let mut out: Column<'a, Message> =
            column![text(title(doc, &f.name)).size(16), text(origin).size(12).style(text::secondary)].spacing(8);
        for w in &doc.warnings {
            out = out.push(text(w).size(12).style(text::warning).width(Fill));
        }
        let name = f.name.clone();
        out = out.push(tea_declarative_ui::view(doc, &saved.state).map(move |m| Message::Ui(name.clone(), m)));
        let status = if !f.pending.is_empty() {
            Some(fl!("ui-waiting", count = f.pending.len()))
        } else if saved.state.locked(doc) {
            Some(fl!("ui-received"))
        } else {
            None
        };
        if let Some(status) = status {
            out = out.push(text(status).size(12).style(text::secondary));
        }
        out.into()
    }
}

/// `<pasta>-<hash do caminho>.json`: legível à mão e sem misturar duas pastas com o mesmo nome.
fn project_key(root: &Path) -> String {
    let name: String = root
        .file_name()
        .map(|n| n.to_string_lossy().chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect())
        .unwrap_or_default();
    let hash = root
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3));
    format!("{name}-{hash:016x}.json")
}

fn title<'a>(doc: &'a Document, name: &'a str) -> &'a str {
    doc.title.as_deref().unwrap_or(name)
}

impl Extension for UiExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "interfaces"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("ui-panel"), Slot::Right);
        ctx.panel_icon(PANEL, Icon::Preview);
        ctx.register_command("interfaces.show", fl!("cmd-ui-show"));
        ctx.register_command("interfaces.teach", fl!("cmd-ui-teach"));
        if let Err(e) = store::prepare(&self.root) {
            self.problem = Some(fl!("ui-prepare-failed", error = e.to_string()));
        }
        self.load();
        Task::none()
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "interfaces.show" => ctx.show_panel(PANEL),
            "interfaces.teach" => self.ask_teach(ctx),
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Snapshot(files) => self.snapshot(files, ctx),
            Message::Select(name) => self.selected = Some(name),
            Message::Ui(name, UiMessage::Set { path, value }) => {
                if let Some(saved) = self.saved.get_mut(&name) {
                    saved.state.set(&path, value);
                    self.save();
                }
            }
            Message::Ui(name, UiMessage::Press(source)) => self.press(&name, &source, ctx),
            Message::AskTeach => self.ask_teach(ctx),
            Message::Teach => self.teach(ctx),
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let mut out: Column<'_, Message> = column![].spacing(12).padding(12).width(Fill);
        if let Some(problem) = &self.problem {
            out = out.push(text(problem).size(13).style(text::danger).width(Fill));
        }
        if self.files.is_empty() {
            out = out.push(text(fl!("ui-empty")).size(13).style(text::secondary).width(Fill));
            out = out.push(text(fl!("ui-teach-hint")).size(13).style(text::secondary).width(Fill));
            out = out.push(button(text(fl!("ui-teach-button")).size(13)).style(button::secondary).on_press(Message::AskTeach));
            return scrollable(out).height(Fill).into();
        }
        if self.files.len() > 1 {
            let tabs = row(self.files.iter().map(|f| {
                let label = f.doc.as_ref().ok().map_or(f.name.as_str(), |d| title(d, &f.name));
                let style = if Some(&f.name) == self.selected.as_ref() { button::primary } else { button::secondary };
                button(text(label).size(12)).style(style).on_press(Message::Select(f.name.clone())).into()
            }))
            .spacing(6)
            .wrap()
            .vertical_spacing(6);
            out = out.push(tabs);
        }
        if let Some(f) = self.selected.as_deref().and_then(|s| self.file(s)) {
            out = out.push(self.surface_view(f));
        }
        scrollable(out).height(Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.root.as_os_str().is_empty() {
            return Subscription::none();
        }
        Subscription::run_with(self.root.clone(), watch)
    }
}

/// Olha a pasta a cada 250 ms (só `stat`); quando algo muda, lê tudo e manda. Termina quando a subscription
/// some (o envio falha).
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
fn watch(root: &PathBuf) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let root = root.clone();
    std::thread::spawn(move || {
        let limits = Limits::default();
        let mut last = None;
        loop {
            if tx.is_closed() {
                return;
            }
            let now = store::signature(&root);
            if last.as_ref() != Some(&now) {
                last = Some(now);
                if tx.unbounded_send(Message::Snapshot(Arc::new(store::scan(&root, &limits)))).is_err() {
                    return;
                }
            }
            std::thread::sleep(WATCH_INTERVAL);
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::simulator;

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("idetea-ext-ui-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    const DOC: &str = r#"{
        "format": "idetea-ui", "version": 1, "revision": 1, "title": "Revisar migração",
        "data": { "ok": false },
        "components": [
            { "id": "c", "component": "checkbox", "label": "Fiz backup", "value": "/ok" },
            { "id": "aplicar", "component": "button", "label": "Aplicar", "action": { "name": "aplicar", "context": ["/ok"] } }
        ]
    }"#;

    fn shows_panel(requests: Vec<tea_core::Request>) -> bool {
        requests.iter().any(|r| matches!(r, tea_core::Request::PanelVisibility { id, visible: Some(true) } if *id == PANEL))
    }

    #[test]
    fn from_file_to_response() {
        let root = tmp_root("fluxo");
        // Estado da IDE numa pasta só deste teste.
        // SAFETY: o teste não roda outras threads que leiam o ambiente ao mesmo tempo.
        unsafe { std::env::set_var("XDG_STATE_HOME", root.join("state")) };
        let mut ext = UiExtension::new();
        let mut ctx = Context::new(&root, None);
        let _ = ext.activate(&mut ctx);
        std::fs::write(store::ui_dir(&root).join("migracao.json"), DOC).unwrap();

        // A pergunta já estava lá quando a IDE abriu: o painel aparece.
        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut ctx);
        assert!(shows_panel(ctx.into_requests()));

        let mut ctx = Context::new(&root, None);
        let msgs: Vec<Message> = {
            let mut ui = simulator(ext.view(PANEL));
            assert!(ui.find("Revisar migração").is_ok());
            ui.click("Fiz backup").unwrap();
            ui.click("Aplicar").unwrap();
            ui.into_messages().collect()
        };
        for m in msgs {
            let _ = ext.update(m, &mut ctx);
        }

        let pending = store::pending(&store::ui_dir(&root), "migracao");
        assert_eq!(pending.len(), 1);
        let (seq, event) = &pending[0];
        let file = store::ui_dir(&root).join("responses/migracao").join(tea_declarative_ui::file_name(*seq, event));
        let response: Response = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
        assert_eq!((response.action.as_str(), response.revision, response.seq), ("aplicar", 1, 1));
        assert_eq!(response.values["/ok"], serde_json::json!(true));

        // Reiniciar a IDE: a revisão continua respondida (o estado foi salvo).
        let mut again = UiExtension::new();
        let _ = again.activate(&mut Context::new(&root, None));
        let mut ctx = Context::new(&root, None);
        let _ = again.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut ctx);
        let doc = again.files[0].doc.as_ref().unwrap();
        assert!(again.saved["migracao"].state.locked(doc));
        assert!(!shows_panel(ctx.into_requests()), "já respondida: não abre o painel de novo");
        let _ = std::fs::remove_dir_all(&root);
    }
}
