//! Árvore de arquivos: navegar, abrir, criar, renomear, apagar (vai para a lixeira)
//! e revelar o arquivo ativo.
//!
//! Usa só a API pública de `tea-core`.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use iced::widget::{Id, button, column, container, operation, row, scrollable, space, text, text_input};
use iced::{Element, Fill, Length, Task};
use tea_core::{Context, CoreEvent, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("tree");
const ROW_HEIGHT: f32 = 24.0;
const INDENT: f32 = 12.0;
const HIDDEN: &[&str] = &[".git"];

fn scroll_id() -> Id {
    Id::new("filetree-scroll")
}

fn input_id() -> Id {
    Id::new("filetree-input")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    path: PathBuf,
    is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EditKind {
    NewFile { dir: PathBuf },
    NewFolder { dir: PathBuf },
    Rename { path: PathBuf },
}

#[derive(Debug, Clone)]
struct Edit {
    kind: EditKind,
    value: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Clicked(PathBuf, bool),
    NewFile,
    NewFolder,
    Rename,
    Delete,
    ConfirmDelete,
    CancelDelete,
    EditChanged(String),
    EditSubmit,
    EditCancel,
    Refresh,
    CollapseAll,
}

#[derive(Default)]
pub struct FileTreeExtension {
    root: PathBuf,
    children: HashMap<PathBuf, Vec<Entry>>,
    expanded: HashSet<PathBuf>,
    selected: Option<PathBuf>,
    active: Option<PathBuf>,
    /// Último arquivo aberto por clique aqui: não precisa rolar até ele.
    clicked: Option<PathBuf>,
    edit: Option<Edit>,
    confirm_delete: Option<PathBuf>,
    error: Option<String>,
}

impl FileTreeExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn load(&mut self, dir: &Path) {
        self.children.insert(dir.to_path_buf(), read_dir_sorted(dir));
    }

    fn ensure_loaded(&mut self, dir: &Path) {
        if !self.children.contains_key(dir) {
            self.load(dir);
        }
    }

    /// Relê todas as pastas já carregadas e esquece as que sumiram.
    fn refresh_all(&mut self) {
        let dirs: Vec<PathBuf> = self.children.keys().cloned().collect();
        for dir in dirs {
            if dir.is_dir() {
                self.load(&dir);
            } else {
                self.children.remove(&dir);
                self.expanded.remove(&dir);
            }
        }
        if self.selected.as_deref().is_some_and(|p| !p.exists()) {
            self.selected = None;
        }
    }

    /// Pasta alvo para "novo arquivo/pasta": a selecionada, a pasta do arquivo selecionado, ou a raiz.
    fn target_dir(&self) -> PathBuf {
        match &self.selected {
            Some(p) if p.is_dir() => p.clone(),
            Some(p) => p.parent().map(Path::to_path_buf).unwrap_or_else(|| self.root.clone()),
            None => self.root.clone(),
        }
    }

    /// Linhas visíveis, na ordem da tela: (profundidade, entrada).
    fn visible_rows(&self) -> Vec<(usize, &Entry)> {
        let mut rows = Vec::new();
        self.collect_rows(&self.root, 0, &mut rows);
        rows
    }

    fn collect_rows<'a>(&'a self, dir: &Path, depth: usize, rows: &mut Vec<(usize, &'a Entry)>) {
        for entry in self.children.get(dir).into_iter().flatten() {
            rows.push((depth, entry));
            if entry.is_dir && self.expanded.contains(&entry.path) {
                self.collect_rows(&entry.path, depth + 1, rows);
            }
        }
    }

    /// Expande as pastas até `path`, seleciona e rola até ele.
    fn reveal(&mut self, path: &Path, scroll: bool) -> Task<Message> {
        let Ok(rel) = path.strip_prefix(&self.root) else { return Task::none() };
        let mut dir = self.root.clone();
        self.ensure_loaded(&dir.clone());
        let parts: Vec<_> = rel.components().collect();
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            dir.push(part);
            self.expanded.insert(dir.clone());
            self.ensure_loaded(&dir.clone());
        }
        self.selected = Some(path.to_path_buf());
        if !scroll {
            return Task::none();
        }
        let index = self.visible_rows().iter().position(|(_, e)| e.path == path);
        match index {
            Some(i) => {
                // Deixa algumas linhas de contexto acima.
                let y = ((i as f32) - 3.0).max(0.0) * ROW_HEIGHT;
                operation::scroll_to(scroll_id(), operation::AbsoluteOffset { x: None, y: Some(y) })
            }
            None => Task::none(),
        }
    }

    fn start_edit(&mut self, kind: EditKind) -> Task<Message> {
        let value = match &kind {
            EditKind::Rename { path } => path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
            _ => String::new(),
        };
        if let EditKind::NewFile { dir } | EditKind::NewFolder { dir } = &kind
            && dir != &self.root
        {
            self.expanded.insert(dir.clone());
            self.ensure_loaded(&dir.clone());
        }
        self.confirm_delete = None;
        self.error = None;
        self.edit = Some(Edit { kind, value });
        Task::batch([operation::focus(input_id()), operation::move_cursor_to_end(input_id())])
    }

    fn submit_edit(&mut self, ctx: &mut Context) {
        let Some(edit) = self.edit.take() else { return };
        let name = edit.value.trim();
        if name.is_empty() {
            return;
        }
        if let Err(e) = validate_name(name) {
            self.error = Some(e);
            self.edit = Some(edit);
            return;
        }
        let result = match &edit.kind {
            EditKind::NewFile { dir } => {
                let path = dir.join(name);
                create_file(&path).map(|()| {
                    ctx.open_file(&path);
                    path
                })
            }
            EditKind::NewFolder { dir } => {
                let path = dir.join(name);
                std::fs::create_dir_all(&path).map(|()| {
                    self.expanded.insert(path.clone());
                    path
                })
            }
            EditKind::Rename { path } => {
                let to = path.parent().unwrap_or(&self.root).join(name);
                if to.exists() && to != *path {
                    Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "já existe"))
                } else {
                    std::fs::rename(path, &to).map(|()| {
                        ctx.path_renamed(path, &to);
                        if self.expanded.remove(path) {
                            self.expanded.insert(to.clone());
                        }
                        to
                    })
                }
            }
        };
        match result {
            Ok(path) => {
                self.refresh_all();
                let _ = self.reveal(&path, false);
            }
            Err(e) => {
                self.error = Some(format!("{name}: {e}"));
                self.edit = Some(edit);
            }
        }
    }

    fn delete_confirmed(&mut self, ctx: &mut Context) {
        let Some(path) = self.confirm_delete.take() else { return };
        match trash::delete(&path) {
            Ok(()) => {
                ctx.path_deleted(&path);
                ctx.notify(format!("Enviado para a lixeira: {}", display_name(&path)));
                self.selected = None;
                self.refresh_all();
            }
            Err(e) => self.error = Some(format!("Não foi possível apagar: {e}")),
        }
    }

    fn edit_row(&self, depth: usize) -> Element<'_, Message> {
        let Some(edit) = &self.edit else { return space::horizontal().into() };
        let placeholder = match edit.kind {
            EditKind::NewFile { .. } => "nome do arquivo",
            EditKind::NewFolder { .. } => "nome da pasta",
            EditKind::Rename { .. } => "novo nome",
        };
        row![
            space::horizontal().width(depth as f32 * INDENT + 14.0),
            text_input(placeholder, &edit.value)
                .id(input_id())
                .on_input(Message::EditChanged)
                .on_submit(Message::EditSubmit)
                .size(13)
                .padding(2),
            button(text("✕").size(12)).on_press(Message::EditCancel).style(button::text).padding([2, 6]),
        ]
        .align_y(iced::Center)
        .into()
    }
}

impl Extension for FileTreeExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "filetree"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        self.root = ctx.root().to_path_buf();
        self.load(&self.root.clone());
        ctx.register_panel(PANEL, "Arquivos", Slot::Left);
        for (id, title) in [
            ("filetree.toggle", "Árvore: mostrar/ocultar"),
            ("filetree.reveal", "Árvore: revelar arquivo ativo"),
            ("filetree.refresh", "Árvore: recarregar"),
            ("filetree.new_file", "Árvore: novo arquivo"),
            ("filetree.new_folder", "Árvore: nova pasta"),
            ("filetree.rename", "Árvore: renomear selecionado"),
            ("filetree.delete", "Árvore: apagar selecionado (lixeira)"),
            ("filetree.collapse", "Árvore: recolher tudo"),
        ] {
            ctx.register_command(id, title);
        }
        match ctx.active_file().map(Path::to_path_buf) {
            Some(active) => {
                self.active = Some(active.clone());
                self.reveal(&active, true)
            }
            None => Task::none(),
        }
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        if self.root.as_os_str().is_empty() {
            return Task::none(); // ainda não ativada
        }
        match event {
            CoreEvent::ActiveFileChanged(path) => {
                self.active = path.clone();
                match path {
                    Some(p) => {
                        let from_click = self.clicked.as_deref() == Some(p.as_path());
                        self.clicked = None;
                        self.reveal(p, !from_click)
                    }
                    None => Task::none(),
                }
            }
            CoreEvent::FileSaved(path) => {
                // Um arquivo novo pode ter aparecido.
                if let Some(dir) = path.parent()
                    && self.children.contains_key(dir)
                {
                    self.load(dir);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn on_command(&mut self, command: &str, ctx: &mut Context) -> Task<Message> {
        match command {
            "filetree.toggle" => {
                ctx.toggle_panel(PANEL);
                Task::none()
            }
            "filetree.reveal" => {
                ctx.show_panel(PANEL);
                match self.active.clone() {
                    Some(p) => self.reveal(&p, true),
                    None => Task::none(),
                }
            }
            "filetree.refresh" => self.update(Message::Refresh, ctx),
            "filetree.new_file" => {
                ctx.show_panel(PANEL);
                self.update(Message::NewFile, ctx)
            }
            "filetree.new_folder" => {
                ctx.show_panel(PANEL);
                self.update(Message::NewFolder, ctx)
            }
            "filetree.rename" => {
                ctx.show_panel(PANEL);
                self.update(Message::Rename, ctx)
            }
            "filetree.delete" => {
                ctx.show_panel(PANEL);
                self.update(Message::Delete, ctx)
            }
            "filetree.collapse" => self.update(Message::CollapseAll, ctx),
            _ => Task::none(),
        }
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::Clicked(path, is_dir) => {
                self.selected = Some(path.clone());
                self.confirm_delete = None;
                self.error = None;
                if is_dir {
                    if !self.expanded.remove(&path) {
                        self.ensure_loaded(&path);
                        self.expanded.insert(path);
                    }
                } else {
                    self.clicked = Some(path.clone());
                    ctx.open_file(path);
                }
            }
            Message::NewFile => return self.start_edit(EditKind::NewFile { dir: self.target_dir() }),
            Message::NewFolder => return self.start_edit(EditKind::NewFolder { dir: self.target_dir() }),
            Message::Rename => {
                if let Some(path) = self.selected.clone() {
                    return self.start_edit(EditKind::Rename { path });
                }
                ctx.notify("Selecione um item na árvore para renomear");
            }
            Message::Delete => match self.selected.clone() {
                Some(path) => {
                    self.edit = None;
                    self.confirm_delete = Some(path);
                }
                None => ctx.notify("Selecione um item na árvore para apagar"),
            },
            Message::ConfirmDelete => self.delete_confirmed(ctx),
            Message::CancelDelete => self.confirm_delete = None,
            Message::EditChanged(value) => {
                if let Some(edit) = &mut self.edit {
                    edit.value = value;
                }
                self.error = None;
            }
            Message::EditSubmit => self.submit_edit(ctx),
            Message::EditCancel => {
                self.edit = None;
                self.error = None;
            }
            Message::Refresh => self.refresh_all(),
            Message::CollapseAll => self.expanded.clear(),
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let tool = |label: &'static str, msg: Option<Message>| {
            button(text(label).size(12)).on_press_maybe(msg).style(button::text).padding([2, 6])
        };
        let has_sel = self.selected.is_some();
        let toolbar = row![
            text(display_name(&self.root)).size(13),
            space::horizontal(),
            tool("+Arq", Some(Message::NewFile)),
            tool("+Pasta", Some(Message::NewFolder)),
            tool("Ren", has_sel.then_some(Message::Rename)),
            tool("Del", has_sel.then_some(Message::Delete)),
            tool("↻", Some(Message::Refresh)),
            tool("⊟", Some(Message::CollapseAll)),
        ]
        .align_y(iced::Center);

        let mut list = column![];
        let edit_parent = match self.edit.as_ref().map(|e| &e.kind) {
            Some(EditKind::NewFile { dir } | EditKind::NewFolder { dir }) => Some(dir.clone()),
            _ => None,
        };
        if edit_parent.as_deref() == Some(self.root.as_path()) {
            list = list.push(self.edit_row(0));
        }

        for (depth, entry) in self.visible_rows() {
            let renaming = matches!(
                self.edit.as_ref().map(|e| &e.kind),
                Some(EditKind::Rename { path }) if *path == entry.path
            );
            if renaming {
                list = list.push(self.edit_row(depth));
            } else {
                list = list.push(self.entry_row(depth, entry));
            }
            if self.confirm_delete.as_deref() == Some(entry.path.as_path()) {
                list = list.push(confirm_row(depth, entry));
            }
            if entry.is_dir && edit_parent.as_deref() == Some(entry.path.as_path()) {
                list = list.push(self.edit_row(depth + 1));
            }
        }

        let mut panel = column![toolbar].spacing(4);
        if let Some(err) = &self.error {
            panel = panel.push(text(err).size(12).style(text::danger));
        }
        panel.push(scrollable(list).id(scroll_id()).height(Fill)).into()
    }
}

impl FileTreeExtension {
    fn entry_row<'a>(&'a self, depth: usize, entry: &'a Entry) -> Element<'a, Message> {
        let name = display_name(&entry.path);
        let icon = match (entry.is_dir, self.expanded.contains(&entry.path)) {
            (true, true) => "▾",
            (true, false) => "▸",
            (false, _) => " ",
        };
        let is_active = self.active.as_deref() == Some(entry.path.as_path());
        let is_selected = self.selected.as_deref() == Some(entry.path.as_path());
        let style = if is_selected {
            button::secondary
        } else {
            button::text
        };
        let label = row![
            space::horizontal().width(depth as f32 * INDENT),
            text(icon).size(12).width(14),
            text(name).size(13).style(move |theme: &iced::Theme| {
                let palette = theme.extended_palette();
                text::Style { color: is_active.then_some(palette.primary.strong.color) }
            }),
        ]
        .align_y(iced::Center);
        button(label)
            .width(Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .padding([2, 4])
            .on_press(Message::Clicked(entry.path.clone(), entry.is_dir))
            .style(style)
            .into()
    }
}

fn confirm_row(depth: usize, entry: &Entry) -> Element<'_, Message> {
    let what = if entry.is_dir { "pasta e todo o conteúdo" } else { "arquivo" };
    container(
        row![
            space::horizontal().width(depth as f32 * INDENT + 14.0),
            text(format!("Mover {what} para a lixeira?")).size(12),
            space::horizontal(),
            button(text("Sim").size(12)).on_press(Message::ConfirmDelete).style(button::danger).padding([2, 8]),
            button(text("Não").size(12)).on_press(Message::CancelDelete).style(button::text).padding([2, 8]),
        ]
        .spacing(4)
        .align_y(iced::Center),
    )
    .padding([2, 0])
    .into()
}

fn display_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string())
}

fn read_dir_sorted(dir: &Path) -> Vec<Entry> {
    let mut entries: Vec<Entry> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !HIDDEN.contains(&e.file_name().to_string_lossy().as_ref()))
        .map(|e| {
            // Segue links simbólicos para saber se é pasta.
            let is_dir = e.path().is_dir();
            Entry { path: e.path(), is_dir }
        })
        .collect();
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| {
            let an = a.path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let bn = b.path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            an.cmp(&bn)
        })
    });
    entries
}

/// Nome relativo seguro: sem `..`, sem caminho absoluto. Subpastas (`a/b.rs`) são permitidas.
fn validate_name(name: &str) -> Result<(), String> {
    let path = Path::new(name);
    if path.is_absolute() {
        return Err("use um nome relativo".into());
    }
    if path.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("nome não pode conter \"..\" nem \".\"".into());
    }
    Ok(())
}

fn create_file(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::OpenOptions::new().write(true).create_new(true).open(path).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tea_core::Request;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tea-filetree-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/deep")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join("src/deep/x.rs"), "").unwrap();
        std::fs::write(dir.join("README.md"), "").unwrap();
        std::fs::write(dir.join("b.txt"), "").unwrap();
        dir
    }

    fn activated(root: &Path) -> FileTreeExtension {
        let mut ft = FileTreeExtension::new();
        let mut ctx = Context::new(root, None);
        let _ = ft.activate(&mut ctx);
        ft
    }

    fn names(ft: &FileTreeExtension) -> Vec<String> {
        ft.visible_rows().iter().map(|(d, e)| format!("{}{}", "  ".repeat(*d), display_name(&e.path))).collect()
    }

    #[test]
    fn lists_dirs_first_and_hides_git() {
        let root = temp_root("list");
        let ft = activated(&root);
        assert_eq!(names(&ft), ["src", "b.txt", "README.md"]);
    }

    #[test]
    fn reveal_expands_ancestors() {
        let root = temp_root("reveal");
        let mut ft = activated(&root);
        let _ = ft.reveal(&root.join("src/deep/x.rs"), false);
        assert_eq!(names(&ft), ["src", "  deep", "    x.rs", "b.txt", "README.md"]);
        assert_eq!(ft.selected, Some(root.join("src/deep/x.rs")));
    }

    #[test]
    fn create_rename_flow() {
        let root = temp_root("create");
        let mut ft = activated(&root);
        let mut ctx = Context::new(&root, None);

        let _ = ft.update(Message::NewFile, &mut ctx);
        let _ = ft.update(Message::EditChanged("novo/a.rs".into()), &mut ctx);
        let _ = ft.update(Message::EditSubmit, &mut ctx);
        assert!(root.join("novo/a.rs").is_file());
        assert!(ft.edit.is_none());

        let _ = ft.update(Message::Rename, &mut ctx);
        let _ = ft.update(Message::EditChanged("b.rs".into()), &mut ctx);
        let _ = ft.update(Message::EditSubmit, &mut ctx);
        assert!(root.join("novo/b.rs").is_file());
        assert!(!root.join("novo/a.rs").exists());

        let requests = ctx.into_requests();
        assert!(matches!(&requests[0], Request::OpenFile(p) if p.ends_with("novo/a.rs")));
        assert!(matches!(&requests[1], Request::PathRenamed { to, .. } if to.ends_with("novo/b.rs")));
    }

    #[test]
    fn rejects_unsafe_names_and_existing_files() {
        let root = temp_root("names");
        let mut ft = activated(&root);
        let mut ctx = Context::new(&root, None);
        let _ = ft.update(Message::NewFile, &mut ctx);
        let _ = ft.update(Message::EditChanged("../fora.rs".into()), &mut ctx);
        let _ = ft.update(Message::EditSubmit, &mut ctx);
        assert!(ft.error.is_some());
        assert!(ft.edit.is_some(), "input continua aberto para corrigir");

        let _ = ft.update(Message::EditChanged("b.txt".into()), &mut ctx);
        let _ = ft.update(Message::EditSubmit, &mut ctx);
        assert!(ft.error.is_some(), "não sobrescreve arquivo existente");
    }
}
