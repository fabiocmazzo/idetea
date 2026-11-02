//! Painel "Interfaces": um agente (ou qualquer script) escreve `.idetea/ui/<nome>.json` e a IDE desenha a
//! interface; quando o usuário clica num botão, a resposta vai para `.idetea/ui/responses/<nome>/`.
//!
//! O formato está em `docs/interfaces/formato.md`; as decisões, em `docs/rfc/0001-interfaces-declarativas.md`.
//! Uma thread olha a pasta (só `stat` a cada 250 ms) e lê os documentos fora da interface.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod player;
pub mod check;
pub mod store;
pub mod teach;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use iced::futures::channel::mpsc;
use iced::widget::{Column, button, column, row, scrollable, text};
use iced::{Element, Fill, Subscription, Task, Theme};
use serde::{Deserialize, Serialize};
use tea_core::icons::Icon;
use tea_core::{ButtonKind, Context, CoreEvent, Dialog, Extension, PanelId, Slot};
use tea_declarative_ui::{
    Document, Ide, Kind, Limits, Playback, Rendered, Response, Seen, State, UiMessage, timestamp,
};

use player::{Player, Playing};
use store::SurfaceFile;

const PANEL: PanelId = PanelId("ui");
const WATCH_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub enum Message {
    /// A pasta como a thread de observação leu.
    Snapshot(Arc<Vec<SurfaceFile>>),
    Select(String),
    Ui(String, UiMessage),
    /// Os arquivos da interface já foram salvos: grava a resposta do botão (`interface`, `botão`).
    Submit(String, String),
    /// Mostra o diálogo de "ensinar os agentes".
    AskTeach,
    /// Confirmado: grava as instruções.
    Teach,
    /// Olhar o tocador (enquanto um som toca).
    AudioTick,
    /// Relógio dos `sprite` (enquanto um deles anima na interface aberta).
    Frame,
}

/// O que a IDE guarda de uma interface (sobrevive a reiniciar a IDE).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Saved {
    state: State,
    /// Último `seq` usado nas respostas desta interface.
    seq: u64,
    /// Hash do arquivo na última leitura (0: estado gravado antes deste campo existir).
    hash: u64,
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
    /// Tema da IDE (cores do Markdown).
    theme: Option<Theme>,
    /// O executável da IDE mudou depois que ela abriu: as respostas saem com `ide.outdated`.
    outdated: bool,
    /// O Markdown de cada interface, já lido (refeito a cada leitura da pasta).
    rendered: BTreeMap<String, Rendered>,
    /// Para uma interface ainda sem o Markdown lido.
    no_markdown: Rendered,
    /// O som do componente `audio` tocando agora.
    player: Player,
    /// Começo do relógio dos `sprite`, e o tempo dele na última batida.
    clock: Option<Instant>,
    clock_ms: u64,
    /// `sprite` pausados, por interface: id → quadro mostrado. Uma revisão nova volta a animar.
    paused: BTreeMap<String, HashMap<String, u32>>,
    /// O que o usuário fez com as imagens e os sons, por interface (revisão, caminho → contagem). Vai na resposta
    /// (`media`) e recomeça numa revisão nova.
    seen: BTreeMap<String, (u64, BTreeMap<String, Seen>)>,
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
        let mut opens = Vec::new();
        for f in files.iter() {
            let Ok(doc) = &f.doc else { continue };
            if f.tracked {
                continue;
            }
            let before = self.file(&f.name).and_then(|old| old.doc.as_ref().ok()).map(|d| d.revision);
            // O agente apagou e recriou o documento entre duas leituras (a IDE não viu sumir): conteúdo novo com
            // revisão menor, ou na mesma revisão já respondida, é outra interface e começa do zero.
            let mut reborn = false;
            match self.saved.get_mut(&f.name) {
                Some(saved) => {
                    reborn = saved.hash != 0
                        && saved.hash != f.hash
                        && (doc.revision < saved.state.revision || saved.state.locked(doc));
                    if reborn {
                        saved.state = State::new(doc);
                    } else {
                        saved.state.apply(doc);
                    }
                    saved.hash = f.hash;
                }
                None => {
                    self.saved.insert(f.name.clone(), Saved { state: State::new(doc), seq: 0, hash: f.hash });
                }
            }
            // Revisão nova; ou, ao abrir a IDE, uma pergunta que ficou sem resposta (o agente pode ter escrito
            // antes de a IDE abrir).
            let waiting = first && f.pending.is_empty() && !self.saved[&f.name].state.locked(doc);
            if waiting || (!first && (reborn || before != Some(doc.revision))) {
                changed = Some((f.name.clone(), title(doc, &f.name).to_string(), before.is_none()));
                opens.extend(file_components(doc).filter(|f| f.3).map(|(path, line, column, _)| (path.to_string(), line, column)));
            }
        }
        // O som para quando a interface dele some ou muda de revisão.
        if let Some(p) = self.player.playing() {
            let same = files.iter().any(|f| f.name == p.name && f.doc.as_ref().is_ok_and(|d| d.revision == p.revision));
            if !same {
                self.player.stop();
            }
        }
        // Revisão nova (ou interface que sumiu): os `sprite` voltam a animar.
        self.paused.retain(|name, _| {
            let old = self.files.iter().find(|f| &f.name == name).and_then(|f| f.doc.as_ref().ok()).map(|d| d.revision);
            let new = files.iter().find(|f| &f.name == name).and_then(|f| f.doc.as_ref().ok()).map(|d| d.revision);
            new.is_some() && old == new
        });
        // Interface apagada pelo agente: acabou; o estado dela não serve mais.
        self.saved.retain(|name, _| files.iter().any(|f| &f.name == name));
        self.rendered = files.iter().filter_map(|f| Some((f.name.clone(), Rendered::new(f.doc.as_ref().ok()?)))).collect();
        self.files = files;
        if let Some((name, title, new)) = changed {
            ctx.notify(if new { fl!("ui-new", title = title) } else { fl!("ui-updated", title = title) });
            ctx.show_panel(PANEL);
            self.selected = Some(name);
        }
        for (path, line, column) in opens {
            let note = fl!("ui-open-later", path = path.clone());
            ctx.open_file_when_idle(self.root.join(path), line as usize, column as usize, note);
        }
        if self.selected.as_ref().is_none_or(|s| self.file(s).is_none()) {
            self.selected = self.files.first().map(|f| f.name.clone());
        }
        self.save();
    }

    /// Clique num botão. Se a interface aponta arquivos, a IDE salva os que estão com alterações antes de
    /// gravar a resposta (o agente lê o arquivo logo depois).
    fn press(&mut self, name: &str, source: &str, ctx: &mut Context) {
        let Some(f) = self.files.iter().find(|f| f.name == name && !f.tracked) else { return };
        let Ok(doc) = &f.doc else { return };
        if self.saved.get(name).is_none_or(|s| s.state.locked(doc)) {
            return;
        }
        let paths: Vec<PathBuf> = file_components(doc).map(|(path, ..)| self.root.join(path)).collect();
        if paths.is_empty() {
            self.submit(name, source, ctx);
        } else {
            ctx.save_files(paths, Message::Submit(name.to_string(), source.to_string()));
        }
    }

    fn submit(&mut self, name: &str, source: &str, ctx: &mut Context) {
        let Some(f) = self.files.iter().find(|f| f.name == name && !f.tracked) else { return };
        let Ok(doc) = &f.doc else { return };
        let media = self.media_seen(f, doc);
        let Some(saved) = self.saved.get_mut(name) else { return };
        let before = saved.state.clone();
        let Some((action, values)) = saved.state.press(doc, source) else { return };
        let seq = saved.seq.max(f.pending.iter().map(|p| p.0).max().unwrap_or(0)) + 1;
        self.counter += 1;
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
        let event = format!("{millis:x}-{:x}-{}", std::process::id(), self.counter);
        let mut response =
            Response::new(name, doc.revision, seq, event.clone(), action, source, timestamp(SystemTime::now()), values);
        let comment = saved.state.comment.trim();
        response.comment = (!comment.is_empty()).then(|| comment.to_string());
        response.media = media;
        response.ide =
            Some(Ide {
                pid: std::process::id(),
                version: env!("CARGO_PKG_VERSION").into(),
                outdated: self.outdated,
                instructions: Some(teach::version(teach::INSTRUCTIONS)),
            });
        match store::write_response(&self.root, &response) {
            Ok(_) => {
                saved.seq = seq;
                saved.state.comment.clear(); // foi junto com esta resposta
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

    /// Tocar o `audio` `id` da interface `name` (o que estiver tocando para).
    fn play(&mut self, name: &str, id: &str, ctx: &mut Context) {
        let Some(f) = self.files.iter().find(|f| f.name == name && !f.tracked) else { return };
        let Ok(doc) = &f.doc else { return };
        let Some((path, looped)) = doc.components.iter().find(|c| c.id == id).and_then(|c| match &c.kind {
            Kind::Audio { path, looped, .. } => Some((path, *looped)),
            _ => None,
        }) else {
            return;
        };
        // Só um arquivo que a leitura aceitou (dentro da raiz, sem symlink, extensão de som).
        if !matches!(f.media.sound(path), Some(Ok(_))) {
            return;
        }
        let what = Playing { name: name.to_string(), id: id.to_string(), revision: doc.revision, path: path.clone() };
        let path = path.clone();
        if let Some(before) = self.player.playing().cloned() {
            self.count(&before.name, &before.path, |s| s.interrupted += 1);
        }
        match self.player.play(what, self.root.join(&path), looped) {
            Ok(()) => self.count(name, &path, |s| s.played += 1),
            Err(e) => ctx.notify(fl!("ui-audio-failed", error = e.to_string())),
        }
    }

    /// Anota algo que o usuário fez com uma imagem ou um som (`path`) da interface `name`, na revisão atual.
    fn count(&mut self, name: &str, path: &str, f: impl FnOnce(&mut Seen)) {
        let Some(revision) = self.file(name).and_then(|f| f.doc.as_ref().ok()).map(|d| d.revision) else { return };
        let entry = self.seen.entry(name.to_string()).or_default();
        if entry.0 != revision {
            *entry = (revision, BTreeMap::new());
        }
        f(entry.1.entry(path.to_string()).or_default());
    }

    /// O `media` da resposta: cada imagem e som do documento, com a versão lida para a tela e as contagens.
    fn media_seen(&self, f: &SurfaceFile, doc: &Document) -> BTreeMap<String, Seen> {
        let counts = self.seen.get(&f.name).filter(|(r, _)| *r == doc.revision).map(|(_, c)| c);
        let media: Vec<&str> = doc.image_paths().into_iter().chain(doc.audio_paths()).collect();
        let mut out: BTreeMap<String, Seen> = media
            .iter()
            .map(|&path| {
                let mut seen = counts.and_then(|c| c.get(path)).cloned().unwrap_or_default();
                if let Some((size, modified)) = f.media.file(path) {
                    seen.size = Some(size);
                    seen.modified = modified.map(timestamp);
                }
                if let Some(Ok(sound)) = f.media.sound(path)
                    && let Some(info) = sound.info
                {
                    seen.seconds = Some((info.seconds() * 1000.0).round() / 1000.0);
                    // `+ 0.0` tira o `-0.0` (pico no máximo arredondado).
                    seen.peak_db = info.peak_db().map(|db| (f64::from(db) * 10.0).round() / 10.0 + 0.0);
                    seen.clips = info.clips();
                }
                (path.to_string(), seen)
            })
            .collect();
        // Arquivos abertos por um link `idetea:open/`: não são mídia do documento; a versão é a do disco agora.
        for (path, seen) in counts.into_iter().flatten().filter(|(p, _)| !media.contains(&p.as_str())) {
            let mut seen = seen.clone();
            if let Ok(m) = std::fs::symlink_metadata(self.root.join(path)) {
                seen.size = Some(m.len());
                seen.modified = m.modified().ok().map(timestamp);
            }
            out.insert(path.clone(), seen);
        }
        out
    }

    fn playback(&self, name: &str) -> Playback {
        let mut out = if self.player.available() { Playback::default() } else { Playback::no_player() };
        out.playing = self.player.playing().filter(|p| p.name == name).map(|p| p.id.clone());
        out.clock_ms = self.clock_ms;
        out.paused = self.paused.get(name).cloned().unwrap_or_default();
        out
    }

    /// Pausar (no quadro de agora) ou voltar a animar; `step`: pausado, o quadro seguinte.
    fn sprite(&mut self, name: &str, id: &str, step: bool) {
        let Some(f) = self.file(name) else { return };
        let Ok(doc) = &f.doc else { return };
        let Some((fps, path)) = doc.components.iter().find(|c| c.id == id).and_then(|c| match &c.kind {
            Kind::Sprite { fps, path, .. } => Some((*fps, path.clone())),
            _ => None,
        }) else {
            return;
        };
        let Some(Ok(frames)) = f.media.sprite(id) else { return };
        let count = frames.handles.len();
        let now = self.playback(name).frame(id, fps, count) as u32;
        let paused = self.paused.entry(name.to_string()).or_default();
        match (step, paused.contains_key(id)) {
            (true, true) => {
                paused.insert(id.to_string(), (now + 1) % count as u32);
                self.count(name, &path, |s| s.stepped += 1);
            }
            (true, false) => {}
            (false, true) => {
                paused.remove(id);
            }
            (false, false) => {
                paused.insert(id.to_string(), now);
            }
        }
    }

    /// Intervalo do relógio dos `sprite`, se algum anima na interface aberta: o do maior `fps`.
    fn sprite_interval(&self) -> Option<Duration> {
        let f = self.selected.as_deref().and_then(|s| self.file(s))?;
        let doc = f.doc.as_ref().ok()?;
        let paused = self.paused.get(&f.name);
        let fps = doc
            .components
            .iter()
            .filter_map(|c| match &c.kind {
                Kind::Sprite { fps, .. }
                    if paused.is_none_or(|p| !p.contains_key(&c.id))
                        && f.media.sprite(&c.id).is_some_and(|r| r.as_ref().is_ok_and(|fr| fr.handles.len() > 1)) =>
                {
                    Some(*fps)
                }
                _ => None,
            })
            .max()?;
        Some(Duration::from_millis(u64::from(1000 / fps.max(1)).max(16)))
    }

    fn teach_text() -> &'static str {
        teach::INSTRUCTIONS
    }

    fn ask_teach(&self, ctx: &mut Context) {
        match teach::plan(&self.root, Self::teach_text()) {
            Ok(plan) if plan.is_empty() => ctx.notify(fl!("ui-teach-up-to-date")),
            Ok(plan) => {
                let files: Vec<String> = plan
                    .iter()
                    .map(|c| if c.new { fl!("ui-teach-file-new", name = c.name.clone()) } else { c.name.clone() })
                    .collect();
                let mut detail = fl!("ui-teach-detail", files = files.join(", "));
                let newer: Vec<String> = plan
                    .iter()
                    .filter_map(|c| c.newer.as_ref().map(|v| fl!("ui-teach-newer-file", name = c.name.clone(), version = v.clone())))
                    .collect();
                // Um bloco de uma IDE mais nova: gravar volta as instruções atrás (o agente perde o que ela ensinou).
                let (label, kind) = if newer.is_empty() {
                    (fl!("ui-teach-write"), ButtonKind::Primary)
                } else {
                    detail = format!("{}\n\n{detail}", fl!("ui-teach-newer", files = newer.join(", "), ours = teach::version(teach::INSTRUCTIONS)));
                    (fl!("ui-teach-write-anyway"), ButtonKind::Danger)
                };
                ctx.show_dialog(
                    Dialog::new(fl!("ui-teach-title"), detail)
                        .button(label, kind, Message::Teach)
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
        let theme = self.theme.clone().unwrap_or(Theme::Dark);
        let rendered = self.rendered.get(&f.name).unwrap_or(&self.no_markdown);
        out = out.push(
            tea_declarative_ui::view(doc, &saved.state, rendered, &f.media, &self.playback(&f.name), &theme, true)
                .map(move |m| Message::Ui(name.clone(), m)),
        );
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

/// Os componentes `file`: (caminho, linha, coluna, abrir sozinho).
fn file_components(doc: &Document) -> impl Iterator<Item = (&str, u64, u64, bool)> {
    doc.components.iter().filter_map(|c| match &c.kind {
        Kind::File { path, line, column, open, .. } => Some((path.as_str(), *line, *column, *open)),
        _ => None,
    })
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
        self.player = Player::new(player::find());
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::Appearance { theme, .. } => self.theme = Some(theme.clone()),
            CoreEvent::BinaryChanged => self.outdated = true,
            _ => {}
        }
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
            Message::Ui(name, UiMessage::Comment(text)) => {
                if let Some(saved) = self.saved.get_mut(&name) {
                    saved.state.comment = text;
                    self.save();
                }
            }
            Message::Ui(name, UiMessage::Open { path, line, column }) => {
                let media = self.file(&name).and_then(|f| f.doc.as_ref().ok()).is_some_and(|d| d.image_paths().contains(&path.as_str()));
                if media {
                    self.count(&name, &path, |s| s.opened += 1);
                }
                tea_declarative_ui::open_file(&self.root, &path, line, column, ctx);
            }
            Message::Ui(name, UiMessage::Link(url)) => {
                if let tea_declarative_ui::Link::System(path) = tea_declarative_ui::link(&url) {
                    self.count(&name, &path, |s| s.opened += 1);
                }
                tea_declarative_ui::follow(&url, &self.root, ctx);
            }
            Message::Ui(_, UiMessage::Copy(code)) => return tea_declarative_ui::copy(code, ctx),
            Message::Ui(name, UiMessage::Play(id)) => self.play(&name, &id, ctx),
            Message::Ui(_, UiMessage::Stop) => {
                if let Some(p) = self.player.playing().cloned() {
                    self.count(&p.name, &p.path, |s| s.stopped += 1);
                }
                self.player.stop();
            }
            Message::Ui(name, UiMessage::SpritePause(id)) => self.sprite(&name, &id, false),
            Message::Ui(name, UiMessage::SpriteStep(id)) => self.sprite(&name, &id, true),
            Message::Frame => {
                let start = *self.clock.get_or_insert_with(Instant::now);
                self.clock_ms = start.elapsed().as_millis() as u64;
            }
            Message::AudioTick => {
                match self.player.tick() {
                    player::Tick::Failed(e) => ctx.notify(fl!("ui-audio-failed", error = e)),
                    player::Tick::Looped(p) => self.count(&p.name, &p.path, |s| {
                        s.completed += 1;
                        s.played += 1;
                    }),
                    player::Tick::Ended(p) => self.count(&p.name, &p.path, |s| s.completed += 1),
                    player::Tick::Same => {}
                }
            }
            Message::Submit(name, source) => self.submit(&name, &source, ctx),
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
        if self.outdated {
            out = out.push(text(fl!("ui-outdated")).size(13).style(text::warning).width(Fill));
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
        let mut subs = vec![Subscription::run_with(self.root.clone(), watch)];
        if self.player.playing().is_some() {
            subs.push(Subscription::run_with(AUDIO_TICK, audio_ticks));
        }
        if let Some(every) = self.sprite_interval() {
            subs.push(Subscription::run_with(SpriteClock(every), sprite_ticks));
        }
        Subscription::batch(subs)
    }
}

/// O relógio dos `sprite` (tipo próprio: não se confunde com o do tocador na mesma duração).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpriteClock(Duration);

fn sprite_ticks(clock: &SpriteClock) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let every = clock.0;
    std::thread::spawn(move || {
        while tx.unbounded_send(Message::Frame).is_ok() {
            std::thread::sleep(every);
        }
    });
    rx
}

/// De quanto em quanto tempo olhar o tocador (fim do som, `loop`).
const AUDIO_TICK: Duration = Duration::from_millis(200);

#[allow(clippy::trivially_copy_pass_by_ref)] // `Subscription::run_with` exige `fn(&D)`.
fn audio_ticks(every: &Duration) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let every = *every;
    std::thread::spawn(move || {
        while tx.unbounded_send(Message::AudioTick).is_ok() {
            std::thread::sleep(every);
        }
    });
    rx
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
        let mut files: Arc<Vec<store::SurfaceFile>> = Arc::new(Vec::new());
        loop {
            if tx.is_closed() {
                return;
            }
            let now = store::signature_with_media(&root, &files);
            if last.as_ref() != Some(&now) {
                files = Arc::new(store::scan(&root, &limits));
                // A assinatura depois da leitura: já com as imagens dos documentos novos.
                last = Some(store::signature_with_media(&root, &files));
                if tx.unbounded_send(Message::Snapshot(files.clone())).is_err() {
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

    /// Estado da IDE numa pasta dos testes. O mesmo valor em todos: eles rodam em paralelo e o ambiente é um só.
    fn test_state() {
        let dir = std::env::temp_dir().join(format!("idetea-ext-ui-state-{}", std::process::id()));
        // SAFETY: todos os testes gravam o mesmo valor; nenhum lê outra variável enquanto isso.
        unsafe { std::env::set_var("XDG_STATE_HOME", dir) };
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
        test_state();
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
        let ide = response.ide.expect("a resposta diz qual IDE respondeu");
        assert_eq!((ide.pid, ide.outdated), (std::process::id(), false));
        assert_eq!(ide.instructions, Some(teach::version(teach::INSTRUCTIONS)));

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

    #[test]
    fn recreated_with_the_same_name_starts_over() {
        let root = tmp_root("recriada");
        test_state();
        let mut ext = UiExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let path = store::ui_dir(&root).join("migracao.json");
        std::fs::write(&path, DOC).unwrap();
        let scan = |ext: &mut UiExtension| {
            let mut ctx = Context::new(&root, None);
            let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut ctx);
            ctx.into_requests()
        };
        scan(&mut ext);
        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::Submit("migracao".into(), "aplicar".into()), &mut ctx);
        let locked = |ext: &UiExtension| ext.saved["migracao"].state.locked(ext.files[0].doc.as_ref().unwrap());
        assert!(locked(&ext));

        // Mesmo conteúdo relido: continua respondida.
        scan(&mut ext);
        assert!(locked(&ext));

        // Apagado e recriado entre duas leituras, com a mesma revisão e outra pergunta: interface nova.
        std::fs::write(&path, DOC.replace("Revisar migração", "Revisar de novo")).unwrap();
        assert!(shows_panel(scan(&mut ext)), "avisa como pergunta nova");
        assert!(!locked(&ext), "botões ligados");
        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::Submit("migracao".into(), "aplicar".into()), &mut ctx);
        let seqs: Vec<u64> = store::pending(&store::ui_dir(&root), "migracao").iter().map(|p| p.0).collect();
        assert_eq!(seqs, [1, 2], "o seq continua subindo: a resposta nova não se confunde com a velha");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn files_open_on_arrival_and_are_saved_before_the_response() {
        let root = tmp_root("arquivos");
        test_state();
        let mut ext = UiExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        let doc = r#"{
            "format": "idetea-ui", "version": 1, "revision": 1, "title": "Mensagem do commit",
            "components": [
                { "id": "msg", "component": "file", "path": "rascunho.md", "line": 3, "column": 1, "open": true, "label": "Edite a mensagem" },
                { "id": "ver", "component": "file", "path": "src/lib.rs" },
                { "id": "ok", "component": "button", "label": "Pronto", "action": { "name": "pronto" } }
            ]
        }"#;
        std::fs::write(store::ui_dir(&root).join("commit.json"), doc).unwrap();

        // Chegou: abre o arquivo marcado com `open` (sem roubar o foco de quem digita: o app decide).
        let mut ctx = Context::new(&root, None);
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut ctx);
        let opens: Vec<_> = ctx
            .into_requests()
            .into_iter()
            .filter_map(|r| match r {
                tea_core::Request::OpenFileWhenIdle { path, line, column, .. } => Some((path, line, column)),
                _ => None,
            })
            .collect();
        assert_eq!(opens, [(root.join("rascunho.md"), 3, 1)], "só o arquivo com open");

        // Clique no link: abre direto.
        let mut ctx = Context::new(&root, None);
        let msgs: Vec<Message> = {
            let mut ui = simulator(ext.view(PANEL));
            ui.click("src/lib.rs").unwrap();
            ui.click("Pronto").unwrap();
            ui.into_messages().collect()
        };
        for m in msgs {
            let _ = ext.update(m, &mut ctx);
        }
        let requests = ctx.into_requests();
        assert!(requests.iter().any(|r| matches!(r, tea_core::Request::OpenFile(p) if *p == root.join("src/lib.rs"))));
        // O botão pede para salvar os arquivos antes; a resposta só sai quando o app devolve a mensagem.
        let then = requests
            .into_iter()
            .find_map(|r| match r {
                tea_core::Request::SaveFiles { paths, then } => {
                    assert_eq!(paths, [root.join("rascunho.md"), root.join("src/lib.rs")]);
                    Some(then)
                }
                _ => None,
            })
            .expect("pede para salvar");
        assert!(store::pending(&store::ui_dir(&root), "commit").is_empty(), "ainda sem resposta");
        let then = then.downcast_ref::<Message>().unwrap().clone();
        // Comentário escrito antes de a IDE salvar e gravar: vai junto com a resposta e o campo esvazia.
        let _ = ext.update(Message::Ui("commit".into(), UiMessage::Comment("  troque o título ".into())), &mut Context::new(&root, None));
        let _ = ext.update(then, &mut Context::new(&root, None));
        let pending = store::pending(&store::ui_dir(&root), "commit");
        assert_eq!(pending.len(), 1, "resposta gravada depois de salvar");
        let file = store::ui_dir(&root).join("responses/commit").join(tea_declarative_ui::file_name(pending[0].0, &pending[0].1));
        let response: Response = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
        assert_eq!(response.comment.as_deref(), Some("troque o título"));
        assert!(ext.saved["commit"].state.comment.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }


    #[test]
    fn sprite_clock_pause_and_step() {
        let root = tmp_root("sprite");
        test_state();
        let mut ext = UiExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let sheet = root.join("folha.png");
        // Folha 6×2 (o crate não depende de `image` para gerar uma).
        std::fs::write(&sheet, include_bytes!("../testdata/folha-6x2.png")).unwrap();
        let doc = r#"{"format": "idetea-ui", "version": 1, "revision": 1, "components": [
            {"id": "voo", "component": "sprite", "path": "folha.png", "frames": 3, "fps": 10}]}"#;
        std::fs::write(store::ui_dir(&root).join("anim.json"), doc).unwrap();
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        assert_eq!(ext.sprite_interval(), Some(Duration::from_millis(100)), "anima: o relógio roda no fps");

        let ui = |ext: &mut UiExtension, m: UiMessage| {
            let _ = ext.update(Message::Ui("anim".into(), m), &mut Context::new(&root, None));
        };
        ext.clock_ms = 250; // quadro 3 (índice 2)
        ui(&mut ext, UiMessage::SpritePause("voo".into()));
        assert_eq!(ext.paused["anim"]["voo"], 2, "pausa no quadro de agora");
        assert_eq!(ext.sprite_interval(), None, "pausado: sem relógio");
        ui(&mut ext, UiMessage::SpriteStep("voo".into()));
        assert_eq!(ext.paused["anim"]["voo"], 0, "depois do último, o primeiro");
        ui(&mut ext, UiMessage::SpritePause("voo".into()));
        assert!(ext.paused["anim"].is_empty(), "volta a animar");

        // Revisão nova: o que estava pausado volta a animar.
        ui(&mut ext, UiMessage::SpritePause("voo".into()));
        std::fs::write(store::ui_dir(&root).join("anim.json"), doc.replace(r#""revision": 1"#, r#""revision": 2"#)).unwrap();
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        assert!(!ext.paused.contains_key("anim"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn response_says_what_was_seen() {
        let root = tmp_root("visto");
        test_state();
        let mut ext = UiExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        std::fs::write(root.join("folha.png"), include_bytes!("../testdata/folha-6x2.png")).unwrap();
        let doc = r#"{"format": "idetea-ui", "version": 1, "revision": 1, "components": [
            {"id": "img", "component": "image", "path": "folha.png"},
            {"id": "voo", "component": "sprite", "path": "folha.png", "frames": 3},
            {"id": "som", "component": "audio", "path": "falta.wav"},
            {"id": "estouro", "component": "audio", "path": "estouro.wav"},
            {"id": "ok", "component": "button", "label": "Ok", "action": {"name": "ok"}}]}"#;
        std::fs::write(root.join("estouro.wav"), include_bytes!("../testdata/estouro.wav")).unwrap();
        std::fs::write(store::ui_dir(&root).join("ver.json"), doc).unwrap();
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        let ui = |ext: &mut UiExtension, m: UiMessage| {
            let _ = ext.update(Message::Ui("ver".into(), m), &mut Context::new(&root, None));
        };
        ui(&mut ext, UiMessage::Open { path: "folha.png".into(), line: 0, column: 0 });
        ui(&mut ext, UiMessage::SpritePause("voo".into()));
        ui(&mut ext, UiMessage::SpriteStep("voo".into()));
        ui(&mut ext, UiMessage::SpriteStep("voo".into()));
        // Um "tocador" que fica tocando (os testes não tocam som de verdade).
        let program = root.join("tocador");
        std::fs::write(&program, "#!/bin/sh\nsleep 5\n").unwrap();
        std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
        ext.player = Player::new(Some(program));
        ui(&mut ext, UiMessage::Play("estouro".into()));
        ui(&mut ext, UiMessage::Play("estouro".into())); // de novo, por cima
        ui(&mut ext, UiMessage::Stop);
        // Um clique num link `idetea:open/` (de um arquivo que falta: o `follow` só avisa, não abre o navegador).
        ui(&mut ext, UiMessage::Link("idetea:open/falta.html".into()));
        ui(&mut ext, UiMessage::Press("ok".into()));

        let (seq, event) = store::pending(&store::ui_dir(&root), "ver")[0].clone();
        let file = store::ui_dir(&root).join("responses/ver").join(tea_declarative_ui::file_name(seq, &event));
        let text = std::fs::read_to_string(file).unwrap();
        let response: Response = serde_json::from_str(&text).unwrap();
        let folha = &response.media["folha.png"];
        let meta = std::fs::metadata(root.join("folha.png")).unwrap();
        assert_eq!(folha.size, Some(meta.len()));
        assert_eq!(folha.modified, Some(timestamp(meta.modified().unwrap())), "a versão lida para a tela");
        assert_eq!((folha.opened, folha.stepped, folha.played), (1, 2, 0));
        assert_eq!(response.media["falta.wav"], Seen::default(), "som que não existe: sem tamanho nem contagem");
        let wav = &response.media["estouro.wav"];
        assert_eq!((wav.played, wav.interrupted, wav.stopped, wav.completed), (2, 1, 1, 0));
        assert_eq!((wav.seconds, wav.peak_db, wav.clips), (Some(0.1), Some(0.0), true), "o que o painel mostrou");
        assert!(text.contains("\"peak_db\": 0.0"), "sem -0.0: {text}");
        let html = response.media.get("falta.html").unwrap_or_else(|| panic!("o link entra no media: {text}"));
        assert_eq!((html.opened, html.size), (1, None), "aberto por idetea:open: entra com a versão do disco");
        assert!(!text.contains("\"stepped\": 0"), "contagens zeradas não aparecem: {text}");
        assert!(!text.contains("\"clips\": false"), "{text}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
