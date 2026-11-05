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
pub mod ask;
pub mod check;
pub mod fix;
pub mod history;
pub mod novo;
pub mod store;
pub mod teach;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use iced::futures::channel::{mpsc, oneshot};
use iced::widget::{Column, button, column, container, row, scrollable, space, text, text_input};
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
/// O histórico das interfaces (RFC 0007), à esquerda.
const HISTORY: PanelId = PanelId("ui-history");
const WATCH_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub enum Message {
    /// A pasta como a thread de observação leu.
    Snapshot(Arc<Vec<SurfaceFile>>),
    /// Os agentes que avisam que esperam (`<nome>.wait`), quando algo muda.
    Waits(BTreeMap<String, store::Wait>),
    Select(String),
    Ui(String, UiMessage),
    /// Os arquivos da interface já foram salvos: grava a resposta do botão (`interface`, `botão`).
    Submit(String, String),
    /// Mostra o diálogo de "ensinar os agentes".
    AskTeach,
    /// Confirmado: grava as instruções.
    Teach,
    /// Mostra o diálogo de "consertar este projeto".
    AskFix,
    /// Confirmado: conserta.
    Fix,
    /// Terminou de criar um projeto novo: a pasta, ou o erro (com os valores do formulário para mostrar de novo).
    NewProjectDone(Result<PathBuf, (String, serde_json::Value)>),
    /// Olhar o tocador (enquanto um som toca).
    AudioTick,
    /// Relógio dos `sprite` (enquanto um deles anima na interface aberta).
    Frame,
    /// O histórico, lido numa thread.
    HistoryLoaded(Arc<Vec<(PathBuf, history::Entry)>>),
    HistoryQuery(String),
    /// Abrir uma entrada do histórico (só para ver).
    HistoryOpen(PathBuf),
    HistoryBack,
    /// Algo dentro da entrada aberta: só links, abrir arquivo e copiar fazem efeito.
    HistoryUi(UiMessage),
    /// Mostra o painel do histórico.
    ShowHistory,
    /// Uma interface aberta, clicada no histórico: seleciona no painel Interfaces.
    ShowOpen(String),
    AskClearHistory,
    ClearHistory,
}

/// Uma entrada do histórico aberta para ver.
struct Opened {
    path: PathBuf,
    entry: history::Entry,
    doc: Result<Document, String>,
    state: State,
    rendered: Rendered,
    media: tea_declarative_ui::Media,
    /// Imagens e sons que mudaram (ou sumiram) depois da resposta.
    changed: Vec<String>,
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
    /// Revisão e hora em que a IDE a viu pela primeira vez (para o histórico).
    shown: Option<(u64, String)>,
    /// Última revisão respondida (sem isto, sumir é "fechada sem resposta").
    answered: Option<u64>,
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
    /// O Markdown de cada interface, com o hash do arquivo de onde veio (mantido enquanto o arquivo não muda,
    /// para não perder o modo de seleção).
    rendered: BTreeMap<String, (u64, Rendered)>,
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
    /// Quem espera a resposta, pelos `<nome>.wait` (sem o arquivo: o agente não avisa, e o painel não diz nada).
    waits: BTreeMap<String, store::Wait>,
    /// O histórico, da entrada mais nova para a mais velha.
    history: Arc<Vec<(PathBuf, history::Entry)>>,
    history_query: String,
    history_open: Option<Opened>,
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

    /// `true`: o histórico mudou.
    fn snapshot(&mut self, files: Arc<Vec<SurfaceFile>>, ctx: &mut Context) -> bool {
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
                    self.saved.insert(f.name.clone(), Saved { state: State::new(doc), hash: f.hash, ..Saved::default() });
                }
            }
            if let Some(saved) = self.saved.get_mut(&f.name)
                && saved.shown.as_ref().is_none_or(|(r, _)| *r != doc.revision || reborn)
            {
                saved.shown = Some((doc.revision, timestamp(SystemTime::now())));
                if reborn {
                    saved.answered = None;
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
        let recorded = self.record_history(&files);
        // Interface apagada pelo agente: acabou; o estado dela não serve mais.
        self.saved.retain(|name, _| files.iter().any(|f| &f.name == name));
        let mut before = std::mem::take(&mut self.rendered);
        self.rendered = files
            .iter()
            .filter_map(|f| {
                let doc = f.doc.as_ref().ok()?;
                let rendered = match before.remove(&f.name) {
                    Some((hash, r)) if hash == f.hash => r,
                    _ => Rendered::new(doc),
                };
                Some((f.name.clone(), (f.hash, rendered)))
            })
            .collect();
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
        recorded
    }

    /// Compara a leitura nova com a anterior: respostas que o agente apagou ("lida pelo agente") e documentos que
    /// ele apagou sem resposta ("fechada sem resposta"). `true`: o histórico mudou.
    fn record_history(&self, files: &[SurfaceFile]) -> bool {
        let now = SystemTime::now();
        let responses = store::ui_dir(&self.root).join("responses");
        let mut changed = false;
        for old in self.files.iter() {
            let new = files.iter().find(|f| f.name == old.name);
            for (seq, event) in &old.pending {
                let still = new.is_some_and(|n| n.pending.iter().any(|p| &p.1 == event));
                // A leitura pode ter começado antes de a resposta ser gravada: confere no disco.
                let on_disk = responses.join(&old.name).join(tea_declarative_ui::file_name(*seq, event)).exists();
                if !still && !on_disk {
                    changed |= history::mark_read(&self.root, event, now);
                }
            }
            if new.is_some() || old.tracked || old.name == novo::DOC_NAME {
                continue;
            }
            let (Ok(doc), Some(raw)) = (&old.doc, &old.raw) else { continue };
            let saved = self.saved.get(&old.name);
            if saved.is_some_and(|s| s.answered == Some(doc.revision)) {
                continue;
            }
            let shown = saved.and_then(|s| s.shown.as_ref()).filter(|(r, _)| *r == doc.revision).map(|(_, t)| t.clone());
            let entry = history::Entry::unanswered(&old.name, doc, raw, shown, now);
            changed |= history::record(&self.root, &entry).is_ok();
        }
        changed
    }

    /// Lê o histórico numa thread.
    fn reload_history(&self) -> Task<Message> {
        let root = self.root.clone();
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(Arc::new(history::list(&root)));
        });
        Task::future(async move { rx.await.ok() }).and_then(|h| Task::done(Message::HistoryLoaded(h)))
    }

    fn open_history(&mut self, path: PathBuf) {
        let Some((_, entry)) = self.history.iter().find(|(p, _)| *p == path) else { return };
        let entry = entry.clone();
        let bytes = serde_json::to_vec(&entry.document).unwrap_or_default();
        let doc = tea_declarative_ui::parse(&bytes, &Limits::default());
        let (state, rendered, media, changed) = match &doc {
            Ok(d) => {
                let values = entry.response.as_ref().map(|r| r.values.clone()).unwrap_or_default();
                let comment = entry.response.as_ref().and_then(|r| r.comment.as_deref());
                let media = tea_declarative_ui::Media::load(&self.root, d, &tea_declarative_ui::MediaLimits::default());
                // A versão de cada mídia na resposta contra a do disco agora.
                let changed = entry
                    .response
                    .iter()
                    .flat_map(|r| r.media.iter())
                    .filter(|(path, seen)| {
                        let now = std::fs::symlink_metadata(self.root.join(path)).ok();
                        let size = now.as_ref().map(std::fs::Metadata::len);
                        let modified = now.and_then(|m| m.modified().ok()).map(timestamp);
                        seen.size.is_some() && (size != seen.size || modified != seen.modified)
                    })
                    .map(|(path, _)| path.clone())
                    .collect();
                (State::frozen(d, &values, comment), Rendered::new(d), media, changed)
            }
            Err(_) => (State::default(), Rendered::default(), tea_declarative_ui::Media::default(), Vec::new()),
        };
        self.history_open = Some(Opened { path, entry, doc, state, rendered, media, changed });
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
                if let Some(raw) = &f.raw {
                    let shown = saved.shown.as_ref().filter(|(r, _)| *r == doc.revision).map(|(_, t)| t.clone());
                    let entry = history::Entry::answered(name, doc, raw, shown, response.clone());
                    if let Err(e) = history::record(&self.root, &entry) {
                        ctx.notify(fl!("ui-history-failed", error = e.to_string()));
                    }
                }
                saved.answered = Some(doc.revision);
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

    /// O que "consertar" faria, num diálogo. O que não se conserta daqui (respostas não lidas, instruções de uma IDE
    /// mais nova) aparece também, para o usuário saber.
    fn ask_fix(&self, ctx: &mut Context) {
        let found = fix::find(&self.root, std::time::SystemTime::now());
        if found.is_empty() {
            ctx.notify(fl!("ui-fix-nothing"));
            return;
        }
        let list = |fixable: bool| -> Vec<String> {
            found.iter().filter(|p| p.fixable() == fixable).map(|p| format!("• {}", p.describe())).collect()
        };
        let (will, wont) = (list(true), list(false));
        let mut detail = String::new();
        if !will.is_empty() {
            detail.push_str(&format!("{}\n{}", fl!("ui-fix-will"), will.join("\n")));
        }
        if !wont.is_empty() {
            if !detail.is_empty() {
                detail.push_str("\n\n");
            }
            detail.push_str(&format!("{}\n{}", fl!("ui-fix-wont"), wont.join("\n")));
        }
        let mut dialog = Dialog::new(fl!("ui-fix-title"), detail);
        if !will.is_empty() {
            dialog = dialog.button(fl!("ui-fix-button"), ButtonKind::Primary, Message::Fix);
        }
        ctx.show_dialog(dialog.cancel(fl!("ui-cancel")));
    }

    fn fix(&self, ctx: &mut Context) {
        let found = fix::find(&self.root, std::time::SystemTime::now());
        match fix::apply(&self.root, &found) {
            Ok(done) if done.is_empty() => ctx.notify(fl!("ui-fix-nothing")),
            Ok(done) => ctx.notify(fl!("ui-fix-done", count = done.len())),
            Err(e) => ctx.notify(fl!("ui-fix-failed", error = e.to_string())),
        }
    }

    /// Mostra o formulário do "Novo projeto" no painel (`error`: a tentativa anterior e os valores dela).
    fn ask_new_project(&mut self, error: Option<(String, serde_json::Value)>, ctx: &mut Context) {
        let revision = self.file(novo::DOC_NAME).and_then(|f| f.doc.as_ref().ok()).map_or(0, |d| d.revision) + 1;
        let (values, message) = match error {
            Some((e, values)) => (values, Some(e)),
            None => (novo::initial_values(&self.root), None),
        };
        match novo::write_form(&self.root, &novo::form(revision, &values, message.as_deref())) {
            Ok(()) => {
                self.selected = Some(novo::DOC_NAME.to_string());
                ctx.show_panel(PANEL);
            }
            Err(e) => ctx.notify(fl!("ui-write-failed", error = e.to_string())),
        }
    }

    /// Um botão do formulário do "Novo projeto": a IDE é o agente desta interface. Criar roda numa thread (as
    /// skills vêm pela rede).
    fn new_project(&mut self, source: &str, ctx: &mut Context) -> Task<Message> {
        let Some(f) = self.files.iter().find(|f| f.name == novo::DOC_NAME) else { return Task::none() };
        let Ok(doc) = &f.doc else { return Task::none() };
        let Some(saved) = self.saved.get_mut(novo::DOC_NAME) else { return Task::none() };
        let Some((action, values)) = saved.state.press(doc, source) else { return Task::none() };
        self.save();
        if action != "criar" {
            let _ = std::fs::remove_file(store::ui_dir(&self.root).join(format!("{}.json", novo::DOC_NAME)));
            return Task::none();
        }
        // Os valores sem a barra, como no `data` do formulário (para mostrar de novo se der errado).
        let shown: serde_json::Map<String, serde_json::Value> =
            values.iter().map(|(k, v)| (k.trim_start_matches('/').to_string(), v.clone())).collect();
        let shown = serde_json::Value::Object(shown);
        let request = match novo::request_from(&values) {
            Ok(r) => r,
            Err(e) => {
                self.ask_new_project(Some((e, shown)), ctx);
                return Task::none();
            }
        };
        let Some(cache) = tea_skills::cache_dir() else { return Task::none() };
        ctx.notify(fl!("new-creating", dir = request.dir.display().to_string()));
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = novo::create(&request, &cache).map(|_| request.dir.clone()).map_err(|e| (e, shown));
            let _ = tx.send(result);
        });
        Task::future(async move { rx.await.ok() }).and_then(|r| Task::done(Message::NewProjectDone(r)))
    }

    fn new_project_done(&mut self, result: Result<PathBuf, (String, serde_json::Value)>, ctx: &mut Context) {
        match result {
            Ok(dir) => {
                let _ = std::fs::remove_file(store::ui_dir(&self.root).join(format!("{}.json", novo::DOC_NAME)));
                match novo::open_window(&dir) {
                    Ok(()) => ctx.notify(fl!("new-created", dir = dir.display().to_string())),
                    Err(e) => ctx.notify(fl!("new-open-failed", dir = dir.display().to_string(), error = e.to_string())),
                }
            }
            Err((e, values)) => self.ask_new_project(Some((fl!("new-failed", error = e), values)), ctx),
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

    fn ask_clear_history(&self, ctx: &mut Context) {
        if self.history.is_empty() {
            ctx.notify(fl!("ui-history-empty"));
            return;
        }
        ctx.show_dialog(
            Dialog::new(fl!("ui-history-clear-title"), fl!("ui-history-clear-detail", count = self.history.len()))
                .button(fl!("ui-history-clear-button"), ButtonKind::Danger, Message::ClearHistory)
                .cancel(fl!("ui-cancel")),
        );
    }

    /// O painel do histórico: as abertas, as entradas por dia (com busca) ou uma entrada aberta.
    fn history_view(&self) -> Element<'_, Message> {
        if let Some(opened) = &self.history_open {
            return scrollable(self.history_entry_view(opened).padding(12)).height(Fill).into();
        }
        let mut out: Column<'_, Message> = column![].spacing(8).padding(12).width(Fill);
        out = out.push(
            text_input(&fl!("ui-history-search"), &self.history_query).on_input(Message::HistoryQuery).size(13).width(Fill),
        );
        let open: Vec<&SurfaceFile> = self.files.iter().filter(|f| !f.tracked && f.doc.is_ok()).collect();
        if !open.is_empty() {
            out = out.push(text(fl!("ui-history-open", count = open.len())).size(12).style(text::secondary));
            for f in open {
                let Ok(doc) = &f.doc else { continue };
                let answered = self.saved.get(&f.name).is_some_and(|s| s.state.locked(doc) || s.answered == Some(doc.revision));
                let line2 = match (&doc.source, answered) {
                    (Some(s), false) => format!("{s} · {}", fl!("ui-history-waiting-you")),
                    (None, false) => fl!("ui-history-waiting-you"),
                    (Some(s), true) => format!("{s} · {}", fl!("ui-history-answered-open")),
                    (None, true) => fl!("ui-history-answered-open"),
                };
                out = out.push(entry_button(title(doc, &f.name), String::new(), line2, false, Message::ShowOpen(f.name.clone())));
            }
        }
        let query = self.history_query.trim().to_lowercase();
        let (today, yesterday) = history::today_and_yesterday();
        let mut day = String::new();
        let mut shown = 0;
        for (path, e) in self.history.iter() {
            if !query.is_empty() && !e.haystack().contains(&query) {
                continue;
            }
            let (d, hm) = history::local(e.time()).unwrap_or_default();
            if d != day {
                let label = if d == today {
                    fl!("ui-history-today")
                } else if d == yesterday {
                    fl!("ui-history-yesterday")
                } else {
                    d.clone()
                };
                out = out.push(text(label).size(12).style(text::secondary));
                day = d;
            }
            let (line2, warn) = entry_summary(e);
            out = out.push(entry_button(e.title(), hm, line2, warn, Message::HistoryOpen(path.clone())));
            shown += 1;
        }
        if self.history.is_empty() {
            out = out.push(text(fl!("ui-history-none")).size(13).style(text::secondary).width(Fill));
        } else if shown == 0 {
            out = out.push(text(fl!("ui-history-no-match")).size(13).style(text::secondary).width(Fill));
        } else {
            out = out.push(
                button(text(fl!("ui-history-clear-button")).size(12)).style(button::text).on_press(Message::AskClearHistory),
            );
        }
        scrollable(out).height(Fill).into()
    }

    fn history_entry_view<'a>(&'a self, o: &'a Opened) -> Column<'a, Message> {
        let e = &o.entry;
        let back = button(text(fl!("ui-history-back")).size(12)).style(button::secondary).on_press(Message::HistoryBack);
        let mut out: Column<'a, Message> = column![back, text(e.title()).size(16)].spacing(8).width(Fill);
        let origin = match &e.source {
            Some(s) => format!("{} · r{} · {s}", e.name, e.revision),
            None => format!("{} · r{}", e.name, e.revision),
        };
        out = out.push(text(origin).size(12).style(text::secondary));
        let mut times = Vec::new();
        if let Some(t) = &e.shown {
            times.push(fl!("ui-history-shown", time = history::local_text(t)));
        }
        if let Some(t) = &e.answered {
            times.push(fl!("ui-history-answered", time = history::local_text(t)));
        }
        if let Some(t) = &e.closed {
            times.push(fl!("ui-history-closed", time = history::local_text(t)));
        }
        if let Some(t) = &e.read_by_agent {
            times.push(fl!("ui-history-read", time = history::local_text(t)));
        }
        for t in times {
            out = out.push(text(t).size(12).style(text::secondary));
        }
        if let Some(r) = &e.response {
            out = out.push(text(fl!("ui-history-button", action = r.action.clone())).size(13));
            if e.read_by_agent.is_none() {
                out = out.push(text(fl!("ui-history-not-read")).size(12).style(text::warning).width(Fill));
            }
        }
        for path in &o.changed {
            out = out.push(text(fl!("ui-history-media-changed", path = path.clone())).size(12).style(text::warning).width(Fill));
        }
        match &o.doc {
            Ok(doc) => {
                let theme = self.theme.clone().unwrap_or(Theme::Dark);
                let playback = Playback { unavailable: Some(fl!("ui-history-no-audio")), ..Playback::default() };
                let view = tea_declarative_ui::view(doc, &o.state, &o.rendered, &o.media, &playback, &theme, true);
                out = out.push(container(view.map(Message::HistoryUi)).padding(8).style(container::bordered_box).width(Fill));
            }
            Err(err) => out = out.push(text(fl!("ui-invalid", error = err.clone())).size(13).style(text::danger).width(Fill)),
        }
        out
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
        let rendered = self.rendered.get(&f.name).map_or(&self.no_markdown, |(_, r)| r);
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
        // Sem resposta lida ainda: dizer se o agente continua esperando.
        let open = !f.pending.is_empty() || !saved.state.locked(doc);
        match self.waits.get(&f.name) {
            Some(store::Wait::Waiting) if open => out = out.push(text(fl!("ui-agent-waiting")).size(12).style(text::secondary)),
            Some(store::Wait::Stopped { minutes }) if open => {
                out = out.push(text(fl!("ui-agent-stopped", minutes = usize::try_from(*minutes).unwrap_or(usize::MAX))).size(12).style(text::warning).width(Fill));
            }
            _ => {}
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

/// A segunda linha de uma entrada do histórico, e se merece atenção (resposta que o agente não leu).
fn entry_summary(e: &history::Entry) -> (String, bool) {
    let source = e.source.as_deref().map(|s| format!("{s} · ")).unwrap_or_default();
    match (&e.outcome, &e.response) {
        (history::Outcome::Answered, Some(r)) if e.read_by_agent.is_none() => {
            (format!("{source}→ {} · {}", r.action, fl!("ui-history-not-read-short")), true)
        }
        (history::Outcome::Answered, Some(r)) => (format!("{source}→ {}", r.action), false),
        _ => (format!("{source}{}", fl!("ui-history-unanswered")), false),
    }
}

/// Um cartão clicável do histórico: título e hora em cima, o resumo embaixo.
fn entry_button<'a>(title: &str, time: String, line2: String, warn: bool, on: Message) -> Element<'a, Message> {
    let top = row![text(title.to_string()).size(13).width(Fill), text(time).size(12).style(card_muted)].spacing(6);
    let bottom = text(line2).size(12).style(if warn { text::warning } else { card_muted });
    button(column![top, bottom].spacing(3)).style(card).padding([8, 10]).width(Fill).on_press(on).into()
}

/// O cartão: a superfície do tema com borda, mais clara sob o mouse.
fn card(theme: &Theme, status: button::Status) -> button::Style {
    let p = theme.extended_palette();
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => p.background.strong.color,
        _ => p.background.weak.color,
    };
    button::Style {
        background: Some(background.into()),
        text_color: p.background.weak.text,
        border: iced::Border { color: p.background.strong.color, width: 1.0, radius: 6.0.into() },
        ..button::Style::default()
    }
}

/// Texto secundário do cartão: a cor do texto do cartão, mais apagada (legível nos temas claros e escuros).
fn card_muted(theme: &Theme) -> text::Style {
    text::Style { color: Some(theme.extended_palette().background.weak.text.scale_alpha(0.72)) }
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
        ctx.panel_icon(PANEL, Icon::CommentDiscussion);
        ctx.register_panel_hidden(HISTORY, fl!("ui-history-panel"), Slot::Left);
        ctx.panel_icon(HISTORY, Icon::History);
        ctx.register_command("interfaces.history", fl!("cmd-ui-history"));
        ctx.register_command("interfaces.clear_history", fl!("cmd-ui-clear-history"));
        ctx.register_command("interfaces.show", fl!("cmd-ui-show"));
        ctx.register_command("interfaces.teach", fl!("cmd-ui-teach"));
        ctx.register_command("interfaces.fix", fl!("cmd-ui-fix"));
        ctx.register_command("interfaces.new_project", fl!("cmd-ui-new-project"));
        if let Err(e) = store::prepare(&self.root) {
            self.problem = Some(fl!("ui-prepare-failed", error = e.to_string()));
        }
        self.load();
        self.player = Player::new(player::find());
        self.reload_history()
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
            "interfaces.history" => ctx.show_panel(HISTORY),
            "interfaces.clear_history" => self.ask_clear_history(ctx),
            "interfaces.teach" => self.ask_teach(ctx),
            "interfaces.fix" => self.ask_fix(ctx),
            "interfaces.new_project" => self.ask_new_project(None, ctx),
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Snapshot(files) => {
                if self.snapshot(files, ctx) {
                    return self.reload_history();
                }
            }
            Message::Waits(waits) => self.waits = waits,
            Message::Select(name) => self.selected = Some(name),
            Message::Ui(name, UiMessage::Set { path, value }) => {
                if let Some(saved) = self.saved.get_mut(&name) {
                    saved.state.set(&path, value);
                    self.save();
                }
            }
            // O formulário do "Novo projeto" é da própria IDE: o clique não vira arquivo de resposta.
            Message::Ui(name, UiMessage::Press(source)) if name == novo::DOC_NAME => return self.new_project(&source, ctx),
            Message::Ui(name, UiMessage::Press(source)) => {
                self.press(&name, &source, ctx);
                return self.reload_history();
            }
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
            Message::Ui(name, UiMessage::ToggleSelect) => {
                if let Some((_, r)) = self.rendered.get_mut(&name) {
                    r.toggle_select();
                }
            }
            Message::Ui(name, UiMessage::SelectText(id, action)) => {
                if let Some((_, r)) = self.rendered.get_mut(&name) {
                    r.act(&id, action);
                }
            }
            Message::Ui(name, UiMessage::Shown { path, visible: true }) => self.count(&name, &path, |s| s.shown += 1),
            Message::Ui(_, UiMessage::Shown { visible: false, .. }) => {}
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
            Message::Submit(name, source) => {
                self.submit(&name, &source, ctx);
                return self.reload_history();
            }
            Message::HistoryLoaded(list) => {
                self.history = list;
                // A entrada aberta foi apagada (histórico limpo): volta para a lista.
                if self.history_open.as_ref().is_some_and(|o| !self.history.iter().any(|(p, _)| *p == o.path)) {
                    self.history_open = None;
                }
            }
            Message::HistoryQuery(q) => self.history_query = q,
            Message::HistoryOpen(path) => self.open_history(path),
            Message::HistoryBack => self.history_open = None,
            Message::HistoryUi(UiMessage::Open { path, line, column }) => {
                tea_declarative_ui::open_file(&self.root, &path, line, column, ctx);
            }
            Message::HistoryUi(UiMessage::Link(url)) => tea_declarative_ui::follow(&url, &self.root, ctx),
            Message::HistoryUi(UiMessage::Copy(code)) => return tea_declarative_ui::copy(code, ctx),
            Message::HistoryUi(UiMessage::ToggleSelect) => {
                if let Some(o) = &mut self.history_open {
                    o.rendered.toggle_select();
                }
            }
            Message::HistoryUi(UiMessage::SelectText(id, action)) => {
                if let Some(o) = &mut self.history_open {
                    o.rendered.act(&id, action);
                }
            }
            Message::HistoryUi(_) => {}
            Message::ShowHistory => ctx.show_panel(HISTORY),
            Message::ShowOpen(name) => {
                self.selected = Some(name);
                ctx.show_panel(PANEL);
            }
            Message::AskClearHistory => self.ask_clear_history(ctx),
            Message::ClearHistory => {
                match history::clear(&self.root) {
                    Ok(n) => ctx.notify(fl!("ui-history-cleared", count = n)),
                    Err(e) => ctx.notify(fl!("ui-history-failed", error = e.to_string())),
                }
                self.history_open = None;
                return self.reload_history();
            }
            Message::NewProjectDone(result) => self.new_project_done(result, ctx),
            Message::AskTeach => self.ask_teach(ctx),
            Message::Teach => self.teach(ctx),
            Message::AskFix => self.ask_fix(ctx),
            Message::Fix => self.fix(ctx),
        }
        Task::none()
    }

    fn view(&self, panel: PanelId) -> Element<'_, Message> {
        if panel == HISTORY {
            return self.history_view();
        }
        let mut out: Column<'_, Message> = column![].spacing(12).padding(12).width(Fill);
        if let Some(problem) = &self.problem {
            out = out.push(text(problem).size(13).style(text::danger).width(Fill));
        }
        if self.outdated {
            out = out.push(text(fl!("ui-outdated")).size(13).style(text::warning).width(Fill));
        }
        if !self.history.is_empty() {
            let link = button(text(fl!("ui-history-link", count = self.history.len())).size(12))
                .style(button::text)
                .padding(0)
                .on_press(Message::ShowHistory);
            out = out.push(row![space::horizontal(), link]);
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
        let mut last_waits = BTreeMap::new();
        loop {
            if tx.is_closed() {
                return;
            }
            // Só manda quando muda: "esperando", ou os minutos desde que parou.
            let waits = store::waits(&root, std::time::SystemTime::now());
            if waits != last_waits {
                last_waits = waits.clone();
                if tx.unbounded_send(Message::Waits(waits)).is_err() {
                    return;
                }
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
    fn history_keeps_answers_reads_and_closed_questions() {
        let root = tmp_root("historico");
        test_state();
        let scan = |ext: &mut UiExtension| {
            let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        };
        let mut ext = UiExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let ui = store::ui_dir(&root);
        std::fs::write(ui.join("migracao.json"), DOC).unwrap();
        std::fs::write(
            ui.join("aviso.json"),
            r#"{"format": "idetea-ui", "version": 1, "revision": 2, "title": "Só um aviso",
                "components": [{"id": "t", "component": "text", "text": "Pronto."}]}"#,
        )
        .unwrap();
        scan(&mut ext);

        // Responder: a entrada fica, com o documento e a resposta, ainda não lida pelo agente.
        ext.selected = Some("migracao".into());
        let msgs: Vec<Message> = {
            let mut sim = simulator(ext.view(PANEL));
            sim.click("Fiz backup").unwrap();
            sim.click("Aplicar").unwrap();
            sim.into_messages().collect()
        };
        for m in msgs {
            let _ = ext.update(m, &mut Context::new(&root, None));
        }
        let list = history::list(&root);
        assert_eq!(list.len(), 1);
        let e = &list[0].1;
        assert_eq!((e.name.as_str(), e.outcome, e.revision), ("migracao", history::Outcome::Answered, 1));
        assert_eq!(e.response.as_ref().unwrap().values["/ok"], serde_json::json!(true));
        assert!(e.shown.is_some() && e.read_by_agent.is_none());
        assert_eq!(e.document["title"], "Revisar migração");

        // O agente lê (apaga a resposta) e depois apaga as duas perguntas.
        std::fs::remove_dir_all(ui.join("responses/migracao")).unwrap();
        scan(&mut ext);
        assert!(history::list(&root)[0].1.read_by_agent.is_some(), "lida pelo agente");
        std::fs::remove_file(ui.join("migracao.json")).unwrap();
        std::fs::remove_file(ui.join("aviso.json")).unwrap();
        scan(&mut ext);
        let list = history::list(&root);
        assert_eq!(list.len(), 2, "a respondida não entra de novo; o aviso entra sem resposta");
        let aviso = list.iter().find(|(_, e)| e.name == "aviso").unwrap();
        assert_eq!((aviso.1.outcome, aviso.1.revision), (history::Outcome::Unanswered, 2));
        assert!(aviso.1.closed.is_some());

        // No painel do histórico: abrir a entrada mostra a interface como ficou, sem botão que responda.
        let _ = ext.update(Message::HistoryLoaded(Arc::new(list)), &mut Context::new(&root, None));
        let msgs: Vec<Message> = {
            let mut sim = simulator(ext.view(HISTORY));
            sim.click("Revisar migração").unwrap();
            sim.into_messages().collect()
        };
        for m in msgs {
            let _ = ext.update(m, &mut Context::new(&root, None));
        }
        let opened = ext.history_open.as_ref().expect("entrada aberta");
        assert_eq!(opened.entry.name, "migracao");
        let msgs: Vec<Message> = {
            let mut sim = simulator(ext.view(HISTORY));
            let _ = sim.click("Aplicar");
            sim.into_messages().collect()
        };
        assert!(!msgs.iter().any(|m| matches!(m, Message::HistoryUi(UiMessage::Press(_)))), "só para ver");
        let _ = std::fs::remove_dir_all(&root);
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
        ui(&mut ext, UiMessage::Shown { path: "estouro.wav".into(), visible: true });
        ui(&mut ext, UiMessage::Shown { path: "estouro.wav".into(), visible: false });
        ui(&mut ext, UiMessage::Shown { path: "estouro.wav".into(), visible: true });
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
        assert_eq!((wav.shown, folha.shown), (2, 0), "entrou na tela duas vezes; a folha nunca");
        assert_eq!((wav.seconds, wav.peak_db, wav.clips), (Some(0.1), Some(0.0), true), "o que o painel mostrou");
        assert!(text.contains("\"peak_db\": 0.0"), "sem -0.0: {text}");
        let html = response.media.get("falta.html").unwrap_or_else(|| panic!("o link entra no media: {text}"));
        assert_eq!((html.opened, html.size), (1, None), "aberto por idetea:open: entra com a versão do disco");
        assert!(!text.contains("\"stepped\": 0"), "contagens zeradas não aparecem: {text}");
        assert!(!text.contains("\"clips\": false"), "{text}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_new_project_form_is_handled_by_the_ide() {
        let root = tmp_root("novo-form");
        test_state();
        let mut ext = UiExtension::new();
        let _ = ext.activate(&mut Context::new(&root, None));
        let _ = ext.on_command("interfaces.new_project", &mut Context::new(&root, None));
        let doc = store::ui_dir(&root).join(format!("{}.json", novo::DOC_NAME));
        assert!(doc.is_file(), "o comando escreve o formulário");
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        // Um nome que não vale: o formulário volta com o erro, e nenhuma resposta é gravada.
        let ui = |ext: &mut UiExtension, m: UiMessage| {
            let _ = ext.update(Message::Ui(novo::DOC_NAME.into(), m), &mut Context::new(&root, None));
        };
        ui(&mut ext, UiMessage::Set { path: "/nome".into(), value: serde_json::json!("a/b") });
        ui(&mut ext, UiMessage::Press("criar".into()));
        assert!(store::pending(&store::ui_dir(&root), novo::DOC_NAME).is_empty(), "o clique não vira arquivo de resposta");
        let text = std::fs::read_to_string(&doc).unwrap();
        assert!(text.contains("\"revision\": 2") && text.contains("a/b"), "revisão nova com o erro: {text}");
        // Cancelar fecha o formulário.
        let _ = ext.update(Message::Snapshot(Arc::new(store::scan(&root, &Limits::default()))), &mut Context::new(&root, None));
        ui(&mut ext, UiMessage::Press("cancelar".into()));
        assert!(!doc.exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
