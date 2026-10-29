//! Command palette: lista todos os comandos (núcleo + extensões) com busca aproximada.
//! Também abre arquivo pelo nome (`Ctrl+P`) e serve de campo de texto para "Salvar como".

use std::path::Path;
use std::sync::{Arc, Mutex};

use iced::widget::{Id, button, column, container, row, scrollable, space, text, text_input};
use iced::{Element, Fill, Length, Theme};
use tea_core::icons::{Icon, icon};
use tea_core::keymap::Keymap;

pub const MAX_ROWS: usize = 12;

pub const ROW_HEIGHT: f32 = 30.0;

/// Quantos arquivos a lista mostra (o resto aparece digitando mais).
const MAX_FILE_RESULTS: usize = 200;
/// Projetos maiores que isso só listam os primeiros (evita travar numa pasta gigante, como a home).
const MAX_FILES: usize = 300_000;

pub fn input_id() -> Id {
    Id::new("palette-input")
}

fn list_id() -> Id {
    Id::new("palette-list")
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Commands,
    /// Pede um caminho para salvar a aba ativa.
    SaveAs,
    /// Abre um arquivo do projeto pelo nome.
    Files,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Query(String),
    Submit,
    Pick(usize),
    /// Clique num arquivo (índice na lista de arquivos).
    PickFile(usize),
    Close,
}

pub struct Palette {
    pub mode: Mode,
    pub query: String,
    pub selected: usize,
    /// Modo `Files`: caminhos relativos à raiz, com `/`.
    pub files: Arc<Vec<String>>,
    /// Modo `Files`: abas abertas (relativos), aparecem primeiro sem busca.
    recent: Vec<String>,
    /// Modo `Files`: a lista ainda está sendo lida do disco.
    pub loading: bool,
    /// Modo `Files`: índices em `files` que combinam, do melhor para o pior.
    pub found: Vec<usize>,
}

impl Palette {
    fn new(mode: Mode, query: String) -> Self {
        Self { mode, query, selected: 0, files: Arc::default(), recent: Vec::new(), loading: false, found: Vec::new() }
    }

    pub fn commands() -> Self {
        Self::new(Mode::Commands, String::new())
    }

    pub fn save_as(initial: String) -> Self {
        Self::new(Mode::SaveAs, initial)
    }

    /// `files` é a última lista conhecida (pode estar velha ou vazia: `loading` diz que vem outra).
    pub fn files(files: Arc<Vec<String>>, recent: Vec<String>, loading: bool) -> Self {
        let mut p = Self { files, recent, loading, ..Self::new(Mode::Files, String::new()) };
        p.found = match_files(&p.query, &p.files, &p.recent);
        p
    }

    pub fn set_files(&mut self, files: Arc<Vec<String>>) {
        self.files = files;
        self.loading = false;
        self.found = match_files(&self.query, &self.files, &self.recent);
        self.selected = self.selected.min(self.found.len().saturating_sub(1));
    }

    pub fn set_query(&mut self, query: String) {
        // `>` no começo troca para os comandos (como no VS Code).
        if self.mode == Mode::Files
            && let Some(rest) = query.strip_prefix('>')
        {
            *self = Self::new(Mode::Commands, rest.trim_start().to_string());
            return;
        }
        self.query = query;
        self.selected = 0;
        if self.mode == Mode::Files {
            self.found = match_files(&self.query, &self.files, &self.recent);
        }
    }

    /// Quantos itens a lista mostra agora (para as setas).
    pub fn total(&self, commands: &[CommandInfo]) -> usize {
        match self.mode {
            Mode::Commands => self.matches(commands).len(),
            Mode::Files => self.found.len(),
            Mode::SaveAs => 0,
        }
    }

    /// Modo `Files`: o arquivo escolhido (relativo) e a linha, se a busca terminar em `:42`.
    pub fn chosen_file(&self, index: Option<usize>) -> Option<(String, Option<usize>)> {
        let i = index.or_else(|| self.found.get(self.selected).copied())?;
        Some((self.files.get(i)?.clone(), split_line(&self.query).1))
    }

    pub fn move_selection(&mut self, delta: i32, total: usize) {
        if total == 0 {
            return;
        }
        let total = total as i32;
        self.selected = (self.selected as i32 + delta).rem_euclid(total) as usize;
    }

    /// Rola a lista para o item selecionado ficar visível (na última linha, se preciso).
    pub fn scroll_to_selected<T>(&self) -> iced::Task<T> {
        let first_visible = self.selected.saturating_sub(MAX_ROWS - 1);
        iced::widget::operation::scroll_to(
            list_id(),
            iced::widget::operation::AbsoluteOffset { x: None, y: Some(first_visible as f32 * ROW_HEIGHT) },
        )
    }

    /// Índices de `commands` que combinam com a busca, do melhor para o pior.
    pub fn matches(&self, commands: &[CommandInfo]) -> Vec<usize> {
        let mut scored: Vec<(i32, usize)> = commands
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                let by_title = fuzzy_score(&self.query, &c.title);
                let by_id = fuzzy_score(&self.query, &c.id).map(|s| s - 5);
                by_title.max(by_id).map(|s| (s, i))
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| commands[a.1].title.cmp(&commands[b.1].title)));
        scored.into_iter().map(|(_, i)| i).collect()
    }

    pub fn view<'a>(&'a self, commands: &'a [CommandInfo], keymap: &'a Keymap) -> Element<'a, Msg> {
        let placeholder = match self.mode {
            Mode::Commands => "Digite um comando…",
            Mode::SaveAs => "Caminho do arquivo (relativo à raiz do projeto)",
            Mode::Files => "Nome do arquivo (arquivo:linha vai para a linha, > para comandos)",
        };
        let input = text_input(placeholder, &self.query)
            .id(input_id())
            .on_input(Msg::Query)
            .on_submit(Msg::Submit)
            .padding(8)
            .size(15);

        let mut body = column![input].spacing(4);
        match self.mode {
            Mode::Commands => {
                let found = self.matches(commands);
                if found.is_empty() {
                    body = body.push(text("Nenhum comando encontrado").size(13));
                }
                let mut list = column![];
                for (pos, &i) in found.iter().enumerate() {
                    let cmd = &commands[i];
                    let hint = keymap.chord_for(&cmd.id).map(|c| c.to_string()).unwrap_or_default();
                    let style = if pos == self.selected { button::primary } else { button::text };
                    list = list.push(
                        button(
                            row![
                                text(&cmd.title).size(14),
                                space::horizontal(),
                                text(hint).size(12).style(text::secondary),
                            ]
                            .align_y(iced::Center),
                        )
                        .width(Fill)
                        .height(ROW_HEIGHT)
                        .padding([4, 8])
                        .on_press(Msg::Pick(i))
                        .style(style),
                    );
                }
                let rows = found.len().min(MAX_ROWS) as f32;
                body = body.push(scrollable(list).id(list_id()).height(Length::Fixed(rows * ROW_HEIGHT)));
            }
            Mode::SaveAs => {
                body = body.push(text("Enter salva · Esc cancela").size(12).style(text::secondary));
            }
            Mode::Files => {
                if self.found.is_empty() {
                    let msg = if self.loading { "Lendo os arquivos do projeto…" } else { "Nenhum arquivo encontrado" };
                    body = body.push(text(msg).size(13).style(text::secondary));
                }
                let mut list = column![];
                for (pos, &i) in self.found.iter().enumerate() {
                    let path = &self.files[i];
                    let (dir, name) = match path.rsplit_once('/') {
                        Some((d, n)) => (d, n),
                        None => ("", path.as_str()),
                    };
                    let style = if pos == self.selected { button::primary } else { button::text };
                    list = list.push(
                        button(
                            row![
                                icon(Icon::File).size(14),
                                text(name).size(14),
                                text(dir).size(12).style(text::secondary),
                            ]
                            .spacing(8)
                            .align_y(iced::Center),
                        )
                        .width(Fill)
                        .height(ROW_HEIGHT)
                        .padding([4, 8])
                        .on_press(Msg::PickFile(i))
                        .style(style),
                    );
                }
                let rows = self.found.len().min(MAX_ROWS) as f32;
                body = body.push(scrollable(list).id(list_id()).height(Length::Fixed(rows * ROW_HEIGHT)));
            }
        }

        container(body)
            .width(560)
            .padding(8)
            .style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.weak.color.into()),
                    border: iced::Border { color: p.background.strong.color, width: 1.0, radius: 8.0.into() },
                    shadow: iced::Shadow {
                        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                        offset: iced::Vector::new(0.0, 8.0),
                        blur_radius: 24.0,
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }
}

/// Busca aproximada: as letras da busca aparecem em ordem no texto.
/// Pontua mais letras seguidas e começo de palavra. `None` = não combina.
pub fn fuzzy_score(query: &str, text: &str) -> Option<i32> {
    fuzzy_score_chars(&query_chars(query), text)
}

fn query_chars(query: &str) -> Vec<char> {
    query.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect()
}

/// Igual a [`fuzzy_score`], com a busca já em minúsculas (não aloca: roda em cada arquivo do projeto).
fn fuzzy_score_chars(query: &[char], text: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let (mut score, mut qi, mut len) = (0, 0, 0);
    let (mut prev_matched, mut prev_char) = (false, None::<char>);
    for c in text.chars().flat_map(char::to_lowercase) {
        let matched = qi < query.len() && c == query[qi];
        if matched {
            score += 1;
            if prev_matched {
                score += 5; // seguidas
            }
            if prev_char.is_none_or(|p| !p.is_alphanumeric()) {
                score += 8; // começo de palavra
            }
            qi += 1;
        }
        prev_matched = matched;
        prev_char = Some(c);
        len += 1;
    }
    (qi == query.len()).then(|| score - len / 4)
}

/// `main.rs:42` → (`main.rs`, Some(42)).
fn split_line(query: &str) -> (&str, Option<usize>) {
    match query.trim().rsplit_once(':') {
        Some((file, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => {
            (file, line.parse().ok().filter(|&n| n > 0))
        }
        _ => (query.trim(), None),
    }
}

/// Arquivos que combinam com a busca, do melhor para o pior (no máximo [`MAX_FILE_RESULTS`]).
/// O nome do arquivo vale mais que o caminho. Sem busca: as abas abertas primeiro, depois a ordem alfabética.
fn match_files(query: &str, files: &[String], recent: &[String]) -> Vec<usize> {
    let q = query_chars(split_line(query).0);
    if q.is_empty() {
        let first: Vec<usize> = recent.iter().filter_map(|r| files.iter().position(|f| f == r)).collect();
        let rest = (0..files.len()).filter(|i| !first.contains(i));
        return first.iter().copied().chain(rest).take(MAX_FILE_RESULTS).collect();
    }
    let mut scored: Vec<(i32, usize)> = files
        .iter()
        .enumerate()
        .filter_map(|(i, path)| {
            let name = path.rsplit('/').next().unwrap_or(path);
            let by_name = fuzzy_score_chars(&q, name).map(|s| s + 20);
            let by_path = fuzzy_score_chars(&q, path);
            by_name.max(by_path).map(|s| (s, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| files[a.1].len().cmp(&files[b.1].len())).then(a.1.cmp(&b.1)));
    scored.truncate(MAX_FILE_RESULTS);
    scored.into_iter().map(|(_, i)| i).collect()
}

/// Lista os arquivos do projeto (relativos, com `/`, em ordem), respeitando o `.gitignore`.
/// Mostra arquivos ocultos (`.github`...), mas nunca entra no `.git`.
pub fn list_files(root: &Path) -> Vec<String> {
    let files = Mutex::new(Vec::new());
    ignore::WalkBuilder::new(root)
        .hidden(false)
        .filter_entry(|e| e.file_name() != ".git")
        .build_parallel()
        .run(|| {
            Box::new(|entry| {
                let Ok(entry) = entry else { return ignore::WalkState::Continue };
                if !entry.file_type().is_some_and(|t| t.is_file()) {
                    return ignore::WalkState::Continue;
                }
                let Ok(rel) = entry.path().strip_prefix(root) else { return ignore::WalkState::Continue };
                let rel: Vec<_> = rel.components().map(|c| c.as_os_str().to_string_lossy()).collect();
                let mut files = files.lock().unwrap_or_else(|e| e.into_inner());
                if files.len() >= MAX_FILES {
                    return ignore::WalkState::Quit;
                }
                files.push(rel.join("/"));
                ignore::WalkState::Continue
            })
        });
    let mut files = files.into_inner().unwrap_or_else(|e| e.into_inner());
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmds(titles: &[(&str, &str)]) -> Vec<CommandInfo> {
        titles.iter().map(|(id, t)| CommandInfo { id: id.to_string(), title: t.to_string() }).collect()
    }

    #[test]
    fn fuzzy_basics() {
        assert!(fuzzy_score("slv", "Arquivo: salvar").is_some());
        assert!(fuzzy_score("xyz", "Arquivo: salvar").is_none());
        assert_eq!(fuzzy_score("", "qualquer"), Some(0));
        // Começo de palavra e letras seguidas pontuam mais.
        assert!(fuzzy_score("sal", "Arquivo: salvar") > fuzzy_score("sal", "Arquivo: isso alterna"));
    }

    #[test]
    fn ranking_and_id_search() {
        let list = cmds(&[
            ("file.save_as", "Arquivo: salvar como…"),
            ("file.save", "Arquivo: salvar"),
            ("theme.toggle", "Tema: alternar claro/escuro"),
        ]);
        let mut p = Palette::commands();
        p.set_query("salvar".into());
        let found = p.matches(&list);
        assert_eq!(found.len(), 2);
        assert_eq!(list[found[0]].id, "file.save", "o mais curto vence o empate");

        p.set_query("theme".into());
        assert_eq!(p.matches(&list), [2], "busca também pelo id");
    }

    fn file_list(paths: &[&str]) -> Arc<Vec<String>> {
        Arc::new(paths.iter().map(|p| p.to_string()).collect())
    }

    fn found(p: &Palette) -> Vec<&str> {
        p.found.iter().map(|&i| p.files[i].as_str()).collect()
    }

    #[test]
    fn files_rank_name_over_path() {
        let files = file_list(&["crates/main_window/lib.rs", "src/main.rs", "docs/domain.md", "src/app/main.rs"]);
        let mut p = Palette::files(files, Vec::new(), false);
        p.set_query("main".into());
        // Nome que combina vence caminho que combina; empate: caminho mais curto.
        assert_eq!(found(&p)[..2], ["src/main.rs", "src/app/main.rs"]);
        assert!(found(&p).contains(&"crates/main_window/lib.rs"), "também acha pelo caminho");

        p.set_query("app/main".into());
        assert_eq!(found(&p)[0], "src/app/main.rs", "com barra, busca no caminho");

        p.set_query("zzz".into());
        assert!(p.found.is_empty());
    }

    #[test]
    fn files_line_suffix_and_command_switch() {
        let mut p = Palette::files(file_list(&["src/main.rs", "README.md"]), Vec::new(), false);
        p.set_query("main.rs:42".into());
        assert_eq!(found(&p), ["src/main.rs"], "o :42 não atrapalha a busca");
        assert_eq!(p.chosen_file(None), Some(("src/main.rs".into(), Some(42))));
        assert_eq!(split_line("a.rs:"), ("a.rs:", None));
        assert_eq!(split_line("c:x"), ("c:x", None));

        p.set_query(">salvar".into());
        assert_eq!(p.mode, Mode::Commands, "> troca para os comandos");
        assert_eq!(p.query, "salvar");
    }

    #[test]
    fn files_empty_query_shows_open_tabs_first() {
        let files = file_list(&["a.rs", "b.rs", "c.rs"]);
        let mut p = Palette::files(Arc::default(), vec!["c.rs".into()], true);
        assert!(p.found.is_empty() && p.loading, "abre antes de ler o disco");
        p.set_files(files);
        assert_eq!(found(&p), ["c.rs", "a.rs", "b.rs"]);
        assert!(!p.loading);
    }

    #[test]
    fn list_files_respects_gitignore_and_skips_git() {
        let root = std::env::temp_dir().join(format!("idetea-quickopen-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for f in ["src/main.rs", ".github/ci.yml", "target/debug/x", ".git/HEAD", ".gitignore"] {
            let path = root.join(f);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "").unwrap();
        }
        std::fs::write(root.join(".gitignore"), "target/\n").unwrap();
        // O `ignore` só lê o .gitignore dentro de um repositório.
        std::fs::create_dir_all(root.join(".git")).unwrap();
        assert_eq!(list_files(&root), [".github/ci.yml", ".gitignore", "src/main.rs"]);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Digitar não pode engasgar: 100 mil caminhos precisam ser filtrados bem rápido.
    /// O tempo só é cobrado em release (`cargo test --release`); em debug oscila demais.
    #[test]
    fn files_match_is_fast() {
        let files: Vec<String> =
            (0..100_000).map(|i| format!("crates/modulo_{}/src/sub_{}/arquivo_{i}.rs", i % 97, i % 13)).collect();
        let started = std::time::Instant::now();
        let found = match_files("arq123", &files, &[]);
        let took = started.elapsed();
        assert!(!found.is_empty());
        eprintln!("100 mil caminhos: {took:?}");
        // Em release fica perto de 10 ms; a folga cobre máquinas lentas e testes rodando em paralelo.
        #[cfg(not(debug_assertions))]
        assert!(took < std::time::Duration::from_millis(150), "{took:?}");
        let _ = took;
    }

    #[test]
    fn selection_wraps() {
        let mut p = Palette::commands();
        p.move_selection(-1, 3);
        assert_eq!(p.selected, 2);
        p.move_selection(1, 3);
        assert_eq!(p.selected, 0);
    }
}
