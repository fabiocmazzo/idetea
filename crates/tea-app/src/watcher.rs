//! Percebe arquivos criados, apagados, renomeados ou alterados no projeto por fora da IDE
//! (terminal, `cargo fmt`, outro editor, `git checkout`).
//!
//! Usa o `notify` (inotify no Linux) com um observador **por pasta**, só nas pastas que o
//! `.gitignore` não ignora. Um observador recursivo entraria em `target/` e `node_modules/`, que
//! mudam o tempo todo e gastam o limite de observadores do sistema. Pastas novas passam a ser
//! observadas assim que aparecem. Mudanças seguidas viram um aviso só (espera 150 ms de silêncio).

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::{Duration, Instant};

use iced::futures::channel::mpsc;
use notify::{EventKind, RecursiveMode, Watcher};

/// Silêncio que encerra uma rajada de mudanças.
const QUIET: Duration = Duration::from_millis(150);
/// Uma rajada contínua (ex.: `git checkout` grande) avisa pelo menos a cada 1 s.
const MAX_BATCH: Duration = Duration::from_secs(1);
/// Projetos com mais pastas que isso só têm as primeiras observadas (ex.: IDE aberta na home).
const MAX_DIRS: usize = 20_000;

#[derive(Debug, Clone)]
pub enum Change {
    /// Caminhos que mudaram (arquivos ou pastas, absolutos).
    Paths(Vec<PathBuf>),
    /// Algo impede de observar tudo (avisado uma vez).
    Problem(String),
}

/// Para a `Subscription`: começa a observar `root` numa thread, que termina quando a subscription some.
#[allow(clippy::ptr_arg)] // `Subscription::run_with` exige `fn(&D)` com o mesmo tipo dos dados.
pub fn watch(root: &PathBuf) -> mpsc::UnboundedReceiver<Change> {
    let (tx, rx) = mpsc::unbounded();
    let root = root.clone();
    std::thread::spawn(move || run(&root, &tx));
    rx
}

struct Dirs {
    watcher: notify::RecommendedWatcher,
    watched: HashSet<PathBuf>,
    /// Já avisou do limite (não repete).
    warned: bool,
}

impl Dirs {
    /// Observa `dir` e as subpastas que o `.gitignore` não ignora (nunca o `.git`).
    fn add_tree(&mut self, dir: &Path, tx: &mpsc::UnboundedSender<Change>) {
        let walk = ignore::WalkBuilder::new(dir).hidden(false).filter_entry(|e| e.file_name() != ".git").build();
        for entry in walk.flatten() {
            if !entry.file_type().is_some_and(|t| t.is_dir()) || self.watched.contains(entry.path()) {
                continue;
            }
            if self.watched.len() >= MAX_DIRS {
                self.warn(tx, fl!("watch-too-many", count = MAX_DIRS));
                return;
            }
            match self.watcher.watch(entry.path(), RecursiveMode::NonRecursive) {
                Ok(()) => {
                    self.watched.insert(entry.path().to_path_buf());
                }
                Err(e) => {
                    // Normalmente o limite de observadores do sistema (fs.inotify.max_user_watches).
                    self.warn(tx, fl!("watch-partial", error = e.to_string()));
                    return;
                }
            }
        }
    }

    fn warn(&mut self, tx: &mpsc::UnboundedSender<Change>, msg: String) {
        if !self.warned {
            self.warned = true;
            let _ = tx.unbounded_send(Change::Problem(fl!(
                "watch-problem", message = msg
            )));
        }
    }
}

/// Só o que muda conteúdo ou nomes. Abrir e ler arquivos (a própria IDE faz isso) não conta,
/// nem mudança só de permissão/data.
fn relevant(kind: &EventKind) -> bool {
    use notify::event::ModifyKind;
    match kind {
        EventKind::Create(_) | EventKind::Remove(_) => true,
        EventKind::Modify(ModifyKind::Metadata(_)) => false,
        EventKind::Modify(_) => true,
        EventKind::Access(_) | EventKind::Any | EventKind::Other => false,
    }
}

fn run(root: &Path, tx: &mpsc::UnboundedSender<Change>) {
    let (events_tx, events) = channel();
    let watcher = match notify::recommended_watcher(events_tx) {
        Ok(w) => w,
        Err(e) => {
            let _ = tx.unbounded_send(Change::Problem(fl!("watch-failed", error = e.to_string())));
            return;
        }
    };
    let mut dirs = Dirs { watcher, watched: HashSet::new(), warned: false };
    dirs.add_tree(root, tx);

    loop {
        // Espera a primeira mudança; de vez em quando confere se a IDE ainda quer os avisos.
        let first = match events.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => event,
            Err(RecvTimeoutError::Timeout) if tx.is_closed() => return,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        };
        let started = Instant::now();
        let mut batch = vec![first];
        while started.elapsed() < MAX_BATCH {
            match events.recv_timeout(QUIET) {
                Ok(event) => batch.push(event),
                Err(_) => break,
            }
        }

        let mut paths = BTreeSet::new();
        let mut removed = false;
        for event in batch.into_iter().flatten() {
            if !relevant(&event.kind) {
                continue;
            }
            removed |= matches!(event.kind, EventKind::Remove(_));
            for path in event.paths {
                if path.components().any(|c| c.as_os_str() == ".git") {
                    continue;
                }
                // Pasta nova (criada ou trazida por renomeação): observar ela e o que tiver dentro.
                if path.is_dir() && !dirs.watched.contains(&path) {
                    dirs.add_tree(&path, tx);
                }
                paths.insert(path);
            }
        }
        if removed {
            // O sistema já esqueceu os observadores das pastas apagadas; aqui só a lista.
            dirs.watched.retain(|d| d.is_dir());
        }
        if !paths.is_empty() && tx.unbounded_send(Change::Paths(paths.into_iter().collect())).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next(rx: &mut mpsc::UnboundedReceiver<Change>) -> Option<Change> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Ok(c) = rx.try_recv() {
                return Some(c);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    fn changed(rx: &mut mpsc::UnboundedReceiver<Change>) -> Vec<PathBuf> {
        match next(rx) {
            Some(Change::Paths(p)) => p,
            other => panic!("esperava mudanças, veio {other:?}"),
        }
    }

    #[test]
    fn sees_files_new_dirs_and_skips_ignored() {
        let root = std::env::temp_dir().join(format!("idetea-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::write(root.join(".gitignore"), "target/\n").unwrap();
        let root = root.canonicalize().unwrap();
        let mut rx = watch(&root);
        std::thread::sleep(Duration::from_millis(200)); // observadores no lugar

        std::fs::write(root.join("a.rs"), "1").unwrap();
        assert!(changed(&mut rx).contains(&root.join("a.rs")));

        // Pasta nova: o arquivo criado dentro dela depois também é visto.
        std::fs::create_dir(root.join("nova")).unwrap();
        assert!(changed(&mut rx).contains(&root.join("nova")));
        std::fs::write(root.join("nova/b.rs"), "2").unwrap();
        assert!(changed(&mut rx).contains(&root.join("nova/b.rs")));

        // Ignorada pelo .gitignore e dentro do .git: nada chega.
        std::fs::write(root.join("target/x.o"), "").unwrap();
        std::fs::write(root.join(".git/index"), "").unwrap();
        std::thread::sleep(Duration::from_millis(400));
        assert!(rx.try_recv().is_err(), "não observa target/ nem .git");

        // Várias mudanças seguidas viram um aviso só.
        for i in 0..20 {
            std::fs::write(root.join(format!("r{i}.rs")), "").unwrap();
        }
        assert_eq!(changed(&mut rx).iter().filter(|p| p.starts_with(&root)).count(), 20);
        drop(rx);
        let _ = std::fs::remove_dir_all(&root);
    }
}
