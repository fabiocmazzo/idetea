//! Command palette: lista todos os comandos (núcleo + extensões) com busca aproximada.
//! Também serve de campo de texto simples para "Salvar como".

use iced::widget::{Id, button, column, container, row, scrollable, space, text, text_input};
use iced::{Element, Fill, Length, Theme};
use tea_core::keymap::Keymap;

pub const MAX_ROWS: usize = 12;

pub const ROW_HEIGHT: f32 = 30.0;

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
}

#[derive(Debug, Clone)]
pub enum Msg {
    Query(String),
    Submit,
    Pick(usize),
    Close,
}

pub struct Palette {
    pub mode: Mode,
    pub query: String,
    pub selected: usize,
}

impl Palette {
    pub fn commands() -> Self {
        Self { mode: Mode::Commands, query: String::new(), selected: 0 }
    }

    pub fn save_as(initial: String) -> Self {
        Self { mode: Mode::SaveAs, query: initial, selected: 0 }
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.selected = 0;
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
    let query: Vec<char> = query.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
    if query.is_empty() {
        return Some(0);
    }
    let text: Vec<char> = text.to_lowercase().chars().collect();
    let mut score = 0;
    let mut qi = 0;
    let mut prev_match: Option<usize> = None;
    for (ti, &c) in text.iter().enumerate() {
        if qi < query.len() && c == query[qi] {
            score += 1;
            if prev_match == Some(ti.wrapping_sub(1)) {
                score += 5; // seguidas
            }
            let word_start = ti == 0 || !text[ti - 1].is_alphanumeric();
            if word_start {
                score += 8;
            }
            prev_match = Some(ti);
            qi += 1;
        }
    }
    (qi == query.len()).then(|| score - text.len() as i32 / 4)
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

    #[test]
    fn selection_wraps() {
        let mut p = Palette::commands();
        p.move_selection(-1, 3);
        assert_eq!(p.selected, 2);
        p.move_selection(1, 3);
        assert_eq!(p.selected, 0);
    }
}
