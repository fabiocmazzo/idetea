//! Git: branch na barra de status e um painel com as mudanças, stage/unstage,
//! commit e diff do arquivo selecionado. Usa o binário `git` do sistema.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

mod git;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use iced::futures::channel::{mpsc, oneshot};
use iced::widget::{Id, button, column, container, operation, row, rule, scrollable, space, text, text_input};
use iced::{Element, Fill, Font, Point, Subscription, Task, Theme};
use tea_core::icons::{Icon, icon, icon_button};
use tea_core::menu::{Menu, context_area};
use tea_core::{ButtonKind, Context, CoreEvent, Dialog, Extension, PanelId, Slot};

pub use git::{Entry, Repo, Status};
use git::{DiffLine, LineKind};

const PANEL: PanelId = PanelId("git");
const BAR: PanelId = PanelId("git-bar");
/// Diffs maiores são cortados (a tela não precisa de mais que isso).
const MAX_DIFF_LINES: usize = 3000;
/// De quanto em quanto tempo olhar se o `.git` mudou (commit no terminal, checkout...).
const WATCH_INTERVAL: Duration = Duration::from_millis(1000);

/// O que fazer quando uma ação dá certo (além de avisar e atualizar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum After {
    Nothing,
    /// Commit feito: limpa a caixa da mensagem.
    ClearMessage,
    /// Alterações descartadas: o diff na tela já não vale.
    ClearSelection,
}

#[derive(Debug, Clone)]
pub enum Message {
    Discovered(Option<Repo>),
    Refresh,
    Refreshed(u64, Arc<Result<Status, String>>),
    Select { path: String, staged: bool },
    DiffLoaded { path: String, staged: bool, diff: Arc<Result<String, String>> },
    Open(String),
    OpenAt(String, usize),
    Stage(String),
    Unstage(String),
    StageAll,
    UnstageAll,
    CommitMessage(String),
    Commit,
    /// Uma ação terminou: `Ok(Some(msg))` vira notificação.
    Done(After, Arc<Result<Option<String>, String>>),
    GitDirChanged,
    /// Botão direito num arquivo da lista.
    EntryMenu { path: String, staged: bool, at: Point },
    /// Pede confirmação para descartar.
    Discard(String),
    DiscardConfirmed(String),
    CopyPath(String),
    Fetch,
    Pull,
    Push,
    /// Push de uma branch sem upstream, depois da confirmação.
    PushConfirmed { remote: String, branch: String },
    /// Fetch/pull/push terminou. `bool`: era um pull (arquivos podem ter mudado no disco).
    RemoteDone(bool, Arc<Result<Option<String>, String>>),
    /// Versões do último commit dos arquivos abertos, para as marcas na margem.
    Bases(Arc<Vec<(PathBuf, Option<String>)>>),
    /// Abre a lista de branches (carrega e mostra na paleta).
    Checkout,
    Branches(Arc<Result<Vec<git::Branch>, String>>),
    SwitchTo { name: String, remote: bool },
    CreateBranch(String),
    /// Troca terminou: nome da branch e o resultado.
    Switched(String, Arc<Result<String, String>>),
    /// Blame de um arquivo chegou.
    Blamed(PathBuf, Arc<Result<git::Blame, String>>),
    /// Clique no autor da linha, na barra de status.
    BlameDetails,
}

/// Arquivo cujo diff está na tela.
struct Selected {
    path: String,
    staged: bool,
    lines: Vec<DiffLine>,
    truncated: bool,
    error: Option<String>,
}

#[derive(Default)]
pub struct GitExtension {
    root: PathBuf,
    repo: Option<Repo>,
    status: Option<Status>,
    error: Option<String>,
    generation: u64,
    selected: Option<Selected>,
    message: String,
    busy: bool,
    /// Arquivo que a ação em andamento muda no disco (as abas abertas dele precisam recarregar).
    touched: Option<PathBuf>,
    /// Fetch/pull/push em andamento ("Enviando…"). Um por vez; não bloqueia stage e commit.
    remote: Option<String>,
    /// Último status enviado à árvore (só reenvia se mudar).
    published: Option<Arc<HashMap<PathBuf, tea_core::FileStatus>>>,
    /// Arquivos abertos alguma vez (para mandar a versão do último commit de novo quando o HEAD mudar).
    opened: HashSet<PathBuf>,
    /// Commit do HEAD na última atualização.
    head_oid: Option<String>,
    /// Arquivo, linha (começa em 1) e se a aba tem alterações, do editor ativo.
    cursor: Option<(PathBuf, usize, bool)>,
    /// Blame do arquivo ativo (do disco), e o que está sendo carregado.
    blame: Option<(PathBuf, Arc<git::Blame>)>,
    blame_loading: Option<PathBuf>,
}

/// "há 3 dias": quanto tempo desde `time` (segundos desde 1970).
fn ago(time: i64) -> String {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    let s = (now - time).max(0);
    let (m, h, d) = (s / 60, s / 3600, s / 86_400);
    if m < 1 {
        fl!("git-ago-now")
    } else if h < 1 {
        fl!("git-ago-minutes", count = m)
    } else if d < 1 {
        fl!("git-ago-hours", count = h)
    } else if d < 30 {
        fl!("git-ago-days", count = d)
    } else if d < 365 {
        let months: i64 = d / 30;
        fl!("git-ago-months", count = months)
    } else {
        let years: i64 = d / 365;
        fl!("git-ago-years", count = years)
    }
}

/// Roda `f` numa thread e entrega o resultado como mensagem.
fn spawn<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
    done: impl Fn(T) -> Message + Send + 'static,
) -> Task<Message> {
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    Task::future(async move { rx.await.ok() }).and_then(move |r| Task::done(done(r)))
}

fn commit_input_id() -> Id {
    Id::new("git-commit-message")
}

impl GitExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn refresh(&mut self) -> Task<Message> {
        let Some(repo) = self.repo.clone() else { return Task::none() };
        self.generation += 1;
        let generation = self.generation;
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(git::status(&repo));
        });
        Task::future(async move { rx.await.ok() })
            .and_then(move |r| Task::done(Message::Refreshed(generation, Arc::new(r))))
    }

    /// Busca a versão do último commit dos arquivos (numa thread) para as marcas na margem.
    fn load_bases(&self, paths: Vec<PathBuf>) -> Task<Message> {
        let Some(repo) = self.repo.clone() else { return Task::none() };
        if paths.is_empty() {
            return Task::none();
        }
        spawn(
            move || paths.into_iter().map(|p| { let t = git::head_text(&repo, &p); (p, t) }).collect(),
            |bases| Message::Bases(Arc::new(bases)),
        )
    }

    /// Carrega o blame do arquivo ativo (numa thread). `force`: de novo mesmo se já tem (salvou, HEAD mudou).
    fn load_blame(&mut self, force: bool) -> Task<Message> {
        let (Some(repo), Some((path, _, _))) = (self.repo.clone(), self.cursor.clone()) else { return Task::none() };
        if !path.starts_with(&repo.root) || self.blame_loading.as_ref() == Some(&path) {
            return Task::none();
        }
        if !force && self.blame.as_ref().is_some_and(|(p, _)| *p == path) {
            return Task::none();
        }
        self.blame_loading = Some(path.clone());
        let p = path.clone();
        spawn(move || git::blame(&repo, &p), move |r| Message::Blamed(path.clone(), Arc::new(r)))
    }

    /// Commit da linha do cursor (só com a aba sem alterações: senão as linhas podem não bater com o disco).
    fn line_commit(&self) -> Option<&git::BlameCommit> {
        let (path, line, modified) = self.cursor.as_ref()?;
        let (bpath, blame) = self.blame.as_ref()?;
        if *modified || bpath != path {
            return None;
        }
        blame.at(*line)
    }

    fn entry(&self, path: &str) -> Option<&Entry> {
        self.status.as_ref()?.entries.iter().find(|e| e.path == path)
    }

    fn load_diff(&self, path: String, staged: bool) -> Task<Message> {
        let (Some(repo), Some(entry)) = (self.repo.clone(), self.entry(&path).cloned()) else {
            return Task::none();
        };
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(git::diff(&repo, &entry, staged));
        });
        Task::future(async move { rx.await.ok() })
            .and_then(move |d| Task::done(Message::DiffLoaded { path: path.clone(), staged, diff: Arc::new(d) }))
    }

    /// Roda uma ação do git e depois atualiza a lista.
    fn action(&mut self, after: After, f: impl FnOnce(&Repo) -> Result<Option<String>, String> + Send + 'static) -> Task<Message> {
        let Some(repo) = self.repo.clone() else { return Task::none() };
        self.busy = true;
        spawn(move || f(&repo), move |r| Message::Done(after, Arc::new(r)))
    }

    /// Caminho e, numa renomeação, o nome antigo (os dois lados entram e saem do stage juntos).
    fn paths_of(&self, path: &str) -> Vec<String> {
        let mut paths = vec![path.to_string()];
        if let Some(orig) = self.entry(path).and_then(|e| e.orig.clone()) {
            paths.push(orig);
        }
        paths
    }

    /// Roda fetch/pull/push numa thread (a rede pode demorar).
    fn remote_op(
        &mut self,
        label: String,
        is_pull: bool,
        ctx: &mut Context,
        f: impl FnOnce(&Repo) -> Result<String, String> + Send + 'static,
    ) -> Task<Message> {
        let Some(repo) = self.repo.clone() else { return Task::none() };
        if let Some(running) = &self.remote {
            ctx.notify(fl!("git-wait", running = running.to_lowercase()));
            return Task::none();
        }
        self.remote = Some(label);
        spawn(move || f(&repo).map(Some), move |r| Message::RemoteDone(is_pull, Arc::new(r)))
    }

    fn abs(&self, path: &str) -> PathBuf {
        self.repo.as_ref().map_or_else(|| self.root.join(path), |r| r.root.join(path))
    }
}

impl Extension for GitExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "git"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        ctx.register_panel_hidden(PANEL, fl!("git-panel"), Slot::Bottom);
        ctx.panel_icon(PANEL, Icon::SourceControl);
        ctx.register_panel(BAR, "Git", Slot::Status);
        ctx.register_command("git.focus", fl!("cmd-git-focus"));
        ctx.register_command("git.refresh", fl!("cmd-git-refresh"));
        ctx.register_command("git.stage_all", fl!("cmd-git-stage-all"));
        ctx.register_command("git.unstage_all", fl!("cmd-git-unstage-all"));
        ctx.register_command("git.commit", fl!("cmd-git-commit"));
        ctx.register_command("git.pull", fl!("cmd-git-pull"));
        ctx.register_command("git.push", fl!("cmd-git-push"));
        ctx.register_command("git.fetch", fl!("cmd-git-fetch"));
        ctx.register_command("git.checkout", fl!("cmd-git-checkout"));
        let root = self.root.clone();
        spawn(move || git::discover(&root), Message::Discovered)
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::PanelShown(_) => self.refresh(),
            // Arquivos mudaram por fora (pull, checkout, outro programa): o blame do disco também.
            CoreEvent::FilesChanged => Task::batch([self.refresh(), self.load_blame(true)]),
            CoreEvent::FileSaved(path) if self.repo.as_ref().is_some_and(|r| path.starts_with(&r.root)) => {
                let blame = if self.cursor.as_ref().is_some_and(|(p, _, _)| p == path) { self.load_blame(true) } else { Task::none() };
                Task::batch([self.refresh(), blame])
            }
            CoreEvent::EditorStatus(st) => {
                self.cursor = st.path.clone().map(|p| (p, st.line, st.modified));
                self.load_blame(false)
            }
            CoreEvent::FileOpened(path) | CoreEvent::ActiveFileChanged(Some(path)) => {
                if self.opened.insert(path.clone()) { self.load_bases(vec![path.clone()]) } else { Task::none() }
            }
            _ => Task::none(),
        }
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        if self.repo.is_none() {
            ctx.notify(fl!("git-no-repo-notice"));
            return Task::none();
        }
        match command {
            "git.focus" => {
                ctx.show_panel(PANEL);
                self.refresh()
            }
            "git.refresh" => self.refresh(),
            "git.stage_all" => self.update(Message::StageAll, ctx),
            "git.unstage_all" => self.update(Message::UnstageAll, ctx),
            "git.pull" => self.update(Message::Pull, ctx),
            "git.push" => self.update(Message::Push, ctx),
            "git.fetch" => self.update(Message::Fetch, ctx),
            "git.checkout" => self.update(Message::Checkout, ctx),
            "git.commit" => {
                ctx.show_panel(PANEL);
                operation::focus(commit_input_id())
            }
            _ => Task::none(),
        }
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Discovered(repo) => {
                self.repo = repo;
                return self.refresh();
            }
            Message::Refresh | Message::GitDirChanged => return self.refresh(),
            Message::Refreshed(generation, result) => {
                if generation != self.generation {
                    return Task::none();
                }
                // HEAD mudou (commit, checkout, pull) ou é a primeira vez: as marcas comparam com outra versão.
                let mut bases = Task::none();
                if let Ok(st) = &*result
                    && (st.oid != self.head_oid || self.status.is_none())
                {
                    self.head_oid = st.oid.clone();
                    bases = Task::batch([self.load_bases(self.opened.iter().cloned().collect()), self.load_blame(true)]);
                }
                match &*result {
                    Ok(st) => {
                        self.status = Some(st.clone());
                        self.error = None;
                        if let Some(repo) = &self.repo {
                            let map = git::file_statuses(repo, st);
                            if self.published.as_deref() != Some(&map) {
                                let map = Arc::new(map);
                                self.published = Some(map.clone());
                                ctx.file_statuses(map);
                            }
                        }
                    }
                    Err(e) => self.error = Some(e.clone()),
                }
                // O diff na tela pode ter mudado (arquivo salvo, stage...).
                if let Some(sel) = &self.selected {
                    let still = self.entry(&sel.path).is_some_and(|e| {
                        if sel.staged { e.staged.is_some() } else { e.unstaged.is_some() }
                    });
                    if still {
                        return Task::batch([bases, self.load_diff(sel.path.clone(), sel.staged)]);
                    }
                    self.selected = None;
                }
                return bases;
            }
            Message::Checkout => {
                let Some(repo) = self.repo.clone() else { return Task::none() };
                return spawn(move || git::branches(&repo), |r| Message::Branches(Arc::new(r)));
            }
            Message::Branches(result) => match &*result {
                Ok(list) => {
                    let mut pick = tea_core::pick::Pick::new(fl!("git-checkout-placeholder"))
                        .create(fl!("git-create-branch"), Message::CreateBranch);
                    for b in list {
                        let detail = if b.current { fl!("git-branch-current", date = b.date.as_str()) } else { b.date.clone() };
                        let icon = if b.remote { Icon::RepoFetch } else { Icon::SourceControl };
                        let msg = Message::SwitchTo { name: b.name.clone(), remote: b.remote };
                        pick = pick.item(Some(icon), b.name.clone(), detail, msg);
                    }
                    ctx.pick(pick);
                }
                Err(e) => ctx.notify(fl!("git-notice", message = e.as_str())),
            },
            Message::SwitchTo { name, remote } => {
                if self.status.as_ref().and_then(|s| s.branch.as_deref()) == Some(name.as_str()) {
                    return Task::none();
                }
                let Some(repo) = self.repo.clone() else { return Task::none() };
                let shown = if remote { name.split_once('/').map_or(name.clone(), |(_, n)| n.to_string()) } else { name.clone() };
                return spawn(move || git::switch(&repo, &name, false, remote), move |r| Message::Switched(shown.clone(), Arc::new(r)));
            }
            Message::CreateBranch(name) => {
                let Some(repo) = self.repo.clone() else { return Task::none() };
                let shown = name.clone();
                return spawn(move || git::switch(&repo, &name, true, false), move |r| Message::Switched(shown.clone(), Arc::new(r)));
            }
            Message::Switched(name, result) => {
                match &*result {
                    Ok(_) => {
                        ctx.notify(fl!("git-switched", branch = name.as_str()));
                        // Os arquivos no disco mudaram: abas sem alterações recarregam, a árvore relê.
                        ctx.files_changed();
                    }
                    Err(e) => ctx.notify(fl!("git-switch-failed", branch = name.as_str(), error = e.as_str())),
                }
                return self.refresh();
            }
            Message::Blamed(path, result) => {
                if self.blame_loading.as_ref() == Some(&path) {
                    self.blame_loading = None;
                }
                // Erro (arquivo novo, fora do commit): guarda vazio para não pedir de novo a cada tecla.
                let blame = match &*result {
                    Ok(b) => b.clone(),
                    Err(_) => git::Blame::default(),
                };
                self.blame = Some((path, Arc::new(blame)));
            }
            Message::BlameDetails => {
                let Some(c) = self.line_commit().cloned() else { return Task::none() };
                if c.uncommitted {
                    return Task::none();
                }
                let short: String = c.hash.chars().take(10).collect();
                let message = fl!("git-blame-detail", author = c.author.as_str(), ago = ago(c.time), hash = short.as_str());
                ctx.show_dialog(
                    Dialog::new(c.summary.clone(), message)
                        .button(fl!("git-blame-copy"), ButtonKind::Primary, Message::CopyPath(c.hash.clone()))
                        .cancel(fl!("git-close")),
                );
            }
            Message::Bases(bases) => {
                for (path, text) in bases.iter() {
                    ctx.diff_base(path.clone(), text.as_deref().map(Arc::from));
                }
            }
            Message::Select { path, staged } => return self.load_diff(path, staged),
            Message::DiffLoaded { path, staged, diff } => {
                let (lines, truncated, error) = match &*diff {
                    Ok(out) => {
                        let (lines, truncated) = git::parse_diff(out, MAX_DIFF_LINES);
                        (lines, truncated, None)
                    }
                    Err(e) => (Vec::new(), false, Some(e.clone())),
                };
                self.selected = Some(Selected { path, staged, lines, truncated, error });
            }
            Message::Open(path) => ctx.open_file(self.abs(&path)),
            Message::OpenAt(path, line) => ctx.open_file_at(self.abs(&path), line, 1),
            Message::Stage(path) => {
                let paths = self.paths_of(&path);
                return self.action(After::Nothing, move |r| git::stage(r, &paths).map(|_| None));
            }
            Message::Unstage(path) => {
                let paths = self.paths_of(&path);
                return self.action(After::Nothing, move |r| git::unstage(r, &paths).map(|_| None));
            }
            Message::StageAll => return self.action(After::Nothing, |r| git::stage_all(r).map(|_| None)),
            Message::UnstageAll => return self.action(After::Nothing, |r| git::unstage_all(r).map(|_| None)),
            Message::CommitMessage(m) => self.message = m,
            Message::Commit => {
                let msg = self.message.trim().to_string();
                if msg.is_empty() {
                    ctx.notify(fl!("git-need-message"));
                } else if self.status.as_ref().is_none_or(Status::is_staged_empty) {
                    ctx.notify(fl!("git-nothing-staged"));
                } else if !self.busy {
                    return self.action(After::ClearMessage, move |r| git::commit(r, &msg).map(Some));
                }
            }
            Message::EntryMenu { path, staged, at } => {
                let Some(entry) = self.entry(&path).cloned() else { return Task::none() };
                ctx.show_menu(self.entry_menu(&entry, staged, at));
            }
            Message::Discard(path) => {
                let Some(entry) = self.entry(&path) else { return Task::none() };
                let what = if entry.untracked { fl!("git-discard-untracked") } else { fl!("git-discard-tracked") };
                ctx.show_dialog(
                    Dialog::new(fl!("git-discard-title", path = path.as_str()), what)
                        .button(fl!("git-discard"), ButtonKind::Danger, Message::DiscardConfirmed(path.clone()))
                        .cancel(fl!("cancel")),
                );
            }
            Message::DiscardConfirmed(path) => {
                let Some(entry) = self.entry(&path).cloned() else { return Task::none() };
                self.touched = Some(self.abs(&path));
                return self.action(After::ClearSelection, move |r| git::discard(r, &entry).map(|_| Some(fl!("git-discarded", path = path))));
            }
            Message::Fetch => return self.remote_op(fl!("git-fetching"), false, ctx, git::fetch),
            Message::Pull => {
                let Some(st) = &self.status else { return Task::none() };
                if st.branch.is_none() {
                    ctx.notify(fl!("git-detached-pull"));
                } else if st.upstream.is_none() {
                    ctx.notify(fl!("git-no-upstream"));
                } else {
                    return self.remote_op(fl!("git-pulling"), true, ctx, git::pull);
                }
            }
            Message::Push => {
                let (Some(st), Some(repo)) = (&self.status, &self.repo) else { return Task::none() };
                let Some(branch) = st.branch.clone() else {
                    ctx.notify(fl!("git-detached-push"));
                    return Task::none();
                };
                if st.upstream.is_some() {
                    return self.remote_op(fl!("git-pushing"), false, ctx, |r| git::push(r, None));
                }
                // Branch nova: pergunta antes de criar ela no remoto.
                let remotes = git::remotes(repo);
                let Some(remote) = remotes.iter().find(|r| *r == "origin").or(remotes.first()).cloned() else {
                    ctx.notify(fl!("git-no-remote"));
                    return Task::none();
                };
                ctx.show_dialog(
                    Dialog::new(
                        fl!("git-publish-title", branch = branch.as_str()),
                        fl!("git-publish-detail", branch = branch.as_str(), remote = remote.as_str()),
                    )
                    .button(fl!("git-publish"), ButtonKind::Primary, Message::PushConfirmed { remote, branch })
                    .cancel(fl!("cancel")),
                );
            }
            Message::PushConfirmed { remote, branch } => {
                return self.remote_op(fl!("git-publishing"), false, ctx, move |r| git::push(r, Some((&remote, &branch))));
            }
            Message::RemoteDone(is_pull, result) => {
                self.remote = None;
                match &*result {
                    Ok(Some(msg)) => ctx.notify(fl!("git-notice", message = msg.as_str())),
                    Ok(None) => {}
                    Err(e) => ctx.notify(fl!("git-notice", message = e.as_str())),
                }
                // Mesmo com erro (ex.: conflito) o pull pode ter mexido nos arquivos.
                if is_pull {
                    ctx.files_changed();
                }
                return self.refresh();
            }
            Message::CopyPath(text) => {
                ctx.notify(fl!("copied", text = text.as_str()));
                return iced::clipboard::write(text);
            }
            Message::Done(after, result) => {
                self.busy = false;
                if let Some(path) = self.touched.take()
                    && result.is_ok()
                {
                    ctx.file_changed(path);
                }
                match &*result {
                    Ok(Some(msg)) => {
                        match after {
                            After::ClearMessage => self.message.clear(),
                            After::ClearSelection => self.selected = None,
                            After::Nothing => {}
                        }
                        ctx.notify(fl!("git-notice", message = msg.as_str()));
                    }
                    Ok(None) => {}
                    Err(e) => ctx.notify(fl!("git-notice", message = e.as_str())),
                }
                return self.refresh();
            }
        }
        Task::none()
    }

    fn view(&self, panel: PanelId) -> Element<'_, Message> {
        if panel == BAR {
            return self.bar_view();
        }
        if self.repo.is_none() {
            return text(fl!("git-no-repo")).size(13).style(text::secondary).into();
        }
        row![container(self.changes_view()).width(340).height(Fill), rule::vertical(1), self.diff_view()]
            .spacing(6)
            .height(Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.repo {
            Some(repo) => Subscription::run_with(repo.git_dir.clone(), watch_git_dir),
            None => Subscription::none(),
        }
    }
}

// ---- Telas -------------------------------------------------------------------

fn status_color(c: char) -> fn(&Theme) -> text::Style {
    match c {
        'A' | '?' => |t| text::Style { color: Some(t.extended_palette().success.base.color) },
        'D' | 'U' => |t| text::Style { color: Some(t.extended_palette().danger.base.color) },
        'R' | 'C' => |t| text::Style { color: Some(t.extended_palette().primary.base.color) },
        _ => |t| text::Style { color: Some(t.extended_palette().warning.base.color) },
    }
}


impl GitExtension {
    fn bar_view(&self) -> Element<'_, Message> {
        let Some(st) = &self.status else { return space::horizontal().width(0).into() };
        let mut label = row![icon(Icon::SourceControl).size(13), text(st.head_label()).size(12)].spacing(4);
        if st.ahead > 0 {
            label = label.push(row![icon(Icon::ArrowUp).size(12), text(st.ahead).size(12)]);
        }
        if st.behind > 0 {
            label = label.push(row![icon(Icon::ArrowDown).size(12), text(st.behind).size(12)]);
        }
        if let Some(running) = &self.remote {
            label = label.push(text(running).size(12).style(text::secondary));
        }
        if !st.entries.is_empty() {
            label = label.push(text(format!("●{}", st.entries.len())).size(12));
        }
        let branch = button(label.align_y(iced::Center)).on_press(Message::Checkout).style(button::text).padding([0, 4]);
        let Some(c) = self.line_commit() else { return branch.into() };
        let who = if c.uncommitted {
            fl!("git-blame-uncommitted")
        } else {
            let line = format!("{}, {} · {}", c.author, ago(c.time), c.summary);
            let short: String = line.chars().take(70).collect();
            if line.chars().count() > 70 { format!("{short}…") } else { short }
        };
        let blame = button(row![icon(Icon::GitCommit).size(12), text(who).size(12).style(text::secondary)].spacing(4).align_y(iced::Center))
            .on_press_maybe((!c.uncommitted).then_some(Message::BlameDetails))
            .style(button::text)
            .padding([0, 4]);
        row![branch, blame].spacing(4).align_y(iced::Center).into()
    }

    fn changes_view(&self) -> Element<'_, Message> {
        let head = self.status.as_ref().map(Status::head_label).unwrap_or_default();
        let (ahead, behind) = self.status.as_ref().map_or((0, 0), |s| (s.ahead, s.behind));
        let idle = self.remote.is_none();
        let count = |n: u32, arrow: &str| if n > 0 { format!(" ({arrow}{n})") } else { String::new() };
        let branch = button(row![icon(Icon::SourceControl).size(14), text(head).size(13)].spacing(4).align_y(iced::Center))
            .on_press(Message::Checkout)
            .style(button::text)
            .padding([1, 4]);
        let header = row![
            branch,
            text(self.remote.clone().unwrap_or_default()).size(12).style(text::secondary),
            space::horizontal(),
            icon_button(Icon::RepoFetch, fl!("git-fetch-tip"), idle.then_some(Message::Fetch)),
            icon_button(Icon::RepoPull, fl!("git-pull-tip", count = count(behind, "↓")), idle.then_some(Message::Pull)),
            icon_button(Icon::RepoPush, fl!("git-push-tip", count = count(ahead, "↑")), idle.then_some(Message::Push)),
            icon_button(Icon::Refresh, fl!("git-refresh"), Some(Message::Refresh)),
        ]
        .spacing(4)
        .align_y(iced::Center);

        let mut commit = text_input(&fl!("git-commit-placeholder"), &self.message)
            .id(commit_input_id())
            .size(13)
            .padding([3, 6]);
        if !self.busy {
            commit = commit.on_input(Message::CommitMessage).on_submit(Message::Commit);
        }

        let mut list = column![].spacing(1);
        if let Some(e) = &self.error {
            list = list.push(text(e).size(12).style(text::danger));
        }
        if let Some(st) = &self.status {
            let staged: Vec<&Entry> = st.entries.iter().filter(|e| e.staged.is_some()).collect();
            let unstaged: Vec<&Entry> = st.entries.iter().filter(|e| e.unstaged.is_some()).collect();
            if !staged.is_empty() {
                list = list.push(self.section(fl!("git-staged"), staged.len(), icon_button(Icon::Remove, fl!("git-unstage-all"), Some(Message::UnstageAll))));
                for e in staged {
                    list = list.push(self.entry_row(e, true));
                }
            }
            if !unstaged.is_empty() {
                list = list.push(self.section(fl!("git-changes"), unstaged.len(), icon_button(Icon::Add, fl!("git-stage-all"), Some(Message::StageAll))));
                for e in unstaged {
                    list = list.push(self.entry_row(e, false));
                }
            }
            if st.entries.is_empty() {
                list = list.push(text(fl!("git-nothing-to-commit")).size(12).style(text::secondary));
            }
        }
        column![header, commit, tea_core::style::scrollable(list).height(Fill)].spacing(4).into()
    }

    fn section<'a>(&self, title: String, n: usize, action: Element<'a, Message>) -> Element<'a, Message> {
        row![text(format!("{} ({n})", title.to_uppercase())).size(11).style(text::secondary), space::horizontal(), action]
            .align_y(iced::Center)
            .padding([2, 0])
            .into()
    }

    fn entry_row<'a>(&self, e: &'a Entry, staged: bool) -> Element<'a, Message> {
        let letter = if staged { e.staged } else { e.unstaged }.unwrap_or(' ');
        let letter = if e.untracked { 'U' } else { letter };
        let color = if e.untracked { status_color('?') } else { status_color(letter) };
        let (name, dir) = match e.path.rsplit_once('/') {
            Some((dir, name)) => (name, dir),
            None => (e.path.as_str(), ""),
        };
        let selected = self.selected.as_ref().is_some_and(|s| s.path == e.path && s.staged == staged);
        let label = row![
            tea_core::file_icons::icon(name, 12.0, true),
            text(name).size(12),
            text(dir).size(11).style(text::secondary),
            space::horizontal(),
            text(letter.to_string()).size(12).font(Font::MONOSPACE).style(color),
        ]
        .spacing(6)
        .align_y(iced::Center);
        let toggle = if staged {
            icon_button(Icon::Remove, fl!("git-unstage"), Some(Message::Unstage(e.path.clone())))
        } else {
            icon_button(Icon::Add, fl!("git-stage"), Some(Message::Stage(e.path.clone())))
        };
        let line = row![
            button(label)
                .on_press(Message::Select { path: e.path.clone(), staged })
                .style(if selected { button::secondary } else { button::text })
                .padding([1, 4])
                .width(Fill),
            icon_button(Icon::GoToFile, fl!("git-open-file"), Some(Message::Open(e.path.clone()))),
            toggle,
        ]
        .align_y(iced::Center);
        let path = e.path.clone();
        context_area(line, move |at| Message::EntryMenu { path: path.clone(), staged, at })
    }

    fn entry_menu(&self, e: &Entry, staged: bool, at: Point) -> Menu {
        let path = e.path.clone();
        let abs = self.abs(&path).display().to_string();
        let mut menu = Menu::new(at)
            .item(Some(Icon::GoToFile), fl!("git-open-file"), Message::Open(path.clone()))
            .item(None, fl!("git-view-diff"), Message::Select { path: path.clone(), staged })
            .separator();
        menu = if staged {
            menu.item(Some(Icon::Remove), fl!("git-unstage"), Message::Unstage(path.clone()))
        } else {
            menu.item(Some(Icon::Add), fl!("git-stage"), Message::Stage(path.clone()))
                .danger(Some(Icon::Discard), fl!("git-discard-menu"), Message::Discard(path.clone()))
        };
        menu.separator()
            .item(Some(Icon::Copy), fl!("git-copy-path"), Message::CopyPath(abs))
            .item(None, fl!("git-copy-relative"), Message::CopyPath(path))
    }

    fn diff_view(&self) -> Element<'_, Message> {
        let Some(sel) = &self.selected else {
            return container(text(fl!("git-select-file")).size(12).style(text::secondary))
                .center(Fill)
                .into();
        };
        let title = row![
            text(&sel.path).size(12),
            text(if sel.staged { "  (stage)" } else { "" }).size(11).style(text::secondary),
            space::horizontal(),
            icon_button(Icon::GoToFile, fl!("git-open-file"), Some(Message::Open(sel.path.clone()))),
        ]
        .align_y(iced::Center);
        if let Some(e) = &sel.error {
            return column![title, text(e).size(12).style(text::danger)].spacing(4).into();
        }
        let mut lines = column![];
        for l in &sel.lines {
            let style: fn(&Theme) -> text::Style = match l.kind {
                LineKind::Added => |t| text::Style { color: Some(t.extended_palette().success.base.color) },
                LineKind::Removed => |t| text::Style { color: Some(t.extended_palette().danger.base.color) },
                LineKind::Hunk => |t| text::Style { color: Some(t.extended_palette().primary.base.color) },
                LineKind::Header => text::secondary,
                LineKind::Context => text::default,
            };
            let line = text(if l.text.is_empty() { " " } else { &l.text }).size(12).font(Font::MONOSPACE).style(style);
            lines = lines.push(match l.new_line {
                Some(n) => button(line)
                    .on_press(Message::OpenAt(sel.path.clone(), n))
                    .style(button::text)
                    .padding(0)
                    .width(Fill)
                    .into(),
                None => Element::from(line),
            });
        }
        if sel.truncated {
            lines = lines.push(text(fl!("git-diff-truncated", count = MAX_DIFF_LINES)).size(12).style(text::secondary));
        }
        if sel.lines.is_empty() {
            lines = lines.push(text(fl!("git-no-diff")).size(12).style(text::secondary));
        }
        column![
            title,
            tea_core::style::scrollable(lines).direction(scrollable::Direction::Both {
                vertical: tea_core::style::scrollbar(),
                horizontal: tea_core::style::scrollbar(),
            })
        ]
        .spacing(4)
        .width(Fill)
        .into()
    }
}

// ---- Observador do .git --------------------------------------------------------

/// Arquivos que mudam em commit, stage, checkout, reset, fetch.
fn watched(git_dir: &Path) -> [PathBuf; 4] {
    ["index", "HEAD", "logs/HEAD", "FETCH_HEAD"].map(|f| git_dir.join(f))
}

fn stamp(files: &[PathBuf]) -> Vec<Option<SystemTime>> {
    files.iter().map(|f| std::fs::metadata(f).and_then(|m| m.modified()).ok()).collect()
}

/// Uma thread olha as datas de alguns arquivos do `.git` a cada segundo (só `stat`, barato).
/// Termina sozinha quando a subscription some (o envio falha).
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
fn watch_git_dir(git_dir: &PathBuf) -> mpsc::UnboundedReceiver<Message> {
    let (tx, rx) = mpsc::unbounded();
    let files = watched(git_dir);
    std::thread::spawn(move || {
        let mut last = stamp(&files);
        loop {
            std::thread::sleep(WATCH_INTERVAL);
            if tx.is_closed() {
                return;
            }
            let now = stamp(&files);
            if now != last {
                last = now;
                if tx.unbounded_send(Message::GitDirChanged).is_err() {
                    return;
                }
            }
        }
    });
    rx
}
