//! Vista de tabela de um CSV, TSV ou JSONL (RFC 0020): outra vista do texto da aba.
//!
//! A tabela é lida do texto da aba ([`tea_table::Table`]) e relida quando ele muda. Editar uma célula não mexe na
//! tabela: devolve uma troca no texto ([`Action::Edit`]), que o editor aplica como qualquer edição (desfazer,
//! salvar, aviso de não salvo). Ordenar e filtrar mudam só a ordem em que as linhas aparecem.

use std::borrow::Cow;

use iced::keyboard::key::Named;
use iced::widget::{Id, button, column, container, mouse_area, operation, row, scrollable, space, text, text_input};
use tea_core::style;
use iced::{Alignment, Color, Element, Fill, Font, Length, Task, Theme};
use tea_table::{Edit, EditError, Format, JsonKind, RowRef, Table};

/// Linhas lidas para achar a largura de cada coluna.
const WIDTH_SAMPLE: usize = 200;
/// Largura de uma coluna, em caracteres.
const MIN_CHARS: usize = 3;
const MAX_CHARS: usize = 40;

#[derive(Debug, Clone)]
pub enum Msg {
    Scrolled { x: f32, y: f32, height: f32 },
    Select(RowRef, usize),
    StartEdit(RowRef, usize),
    EditInput(String),
    CommitEdit,
    Sort(usize),
    Filter(String),
    Key(Named),
    Copy,
    ShowText,
}

/// O que o editor faz depois de uma mensagem da tabela.
pub enum Action {
    None,
    Task(Task<Msg>),
    /// Trocar estes bytes do texto (já validado). O editor aplica, e a tabela é relida.
    Edit(Edit),
    /// Voltar para o texto, com o cursor neste byte (a célula escolhida), se houver.
    ShowText(Option<usize>),
    /// Mudar a seleção pede foco no editor (as setas só valem com a aba focada).
    Focus(Task<Msg>),
}

struct Editing {
    row: RowRef,
    col: usize,
    value: String,
}

pub struct TableView {
    format: Format,
    table: Table,
    /// O texto do qual a tabela foi lida, e a revisão dele no editor.
    text: String,
    revision: u64,
    /// A aba mostra a tabela (senão, o texto).
    pub showing: bool,
    sort: Option<(usize, bool)>,
    filter: String,
    /// As linhas que aparecem (índices em `table.rows`), na ordem da tela.
    order: Vec<usize>,
    selected: Option<(RowRef, usize)>,
    editing: Option<Editing>,
    /// Erro da última edição (JSON inválido...), mostrado na barra até a próxima ação.
    error: Option<String>,
    scroll: (f32, f32),
    viewport: f32,
    /// O tamanho da fonte do editor (o editor atualiza antes de cada mensagem), para saber a altura das linhas.
    pub font_size: f32,
    widths: Vec<f32>,
    body_id: Id,
    header_id: Id,
    input_id: Id,
}

impl TableView {
    pub fn new(format: Format, text: &str, revision: u64) -> Self {
        let mut view = Self {
            format,
            table: Table::parse("", format, true),
            text: String::new(),
            revision: u64::MAX,
            showing: true,
            sort: None,
            filter: String::new(),
            order: Vec::new(),
            selected: None,
            editing: None,
            error: None,
            scroll: (0.0, 0.0),
            viewport: 600.0,
            font_size: 14.0,
            widths: Vec::new(),
            body_id: Id::unique(),
            header_id: Id::unique(),
            input_id: Id::unique(),
        };
        view.reload(text, revision);
        view
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Relê a tabela do texto (o texto mudou: uma edição, outro programa).
    pub fn reload(&mut self, text: &str, revision: u64) {
        // Um CSV que agora tem outro separador continua com o de antes: o arquivo não muda de formato sozinho.
        self.table = Table::parse(text, self.format, true);
        self.text = text.to_string();
        self.revision = revision;
        self.widths = self.measure();
        if let Some((RowRef::Row(r), _)) = self.selected
            && r >= self.table.rows.len()
        {
            self.selected = None;
        }
        self.refresh_order();
    }

    fn measure(&self) -> Vec<f32> {
        (0..self.table.columns.len())
            .map(|c| {
                let header = self.table.columns[c].chars().count() + 2;
                let longest = self
                    .table
                    .rows
                    .iter()
                    .enumerate()
                    .take(WIDTH_SAMPLE)
                    .filter_map(|(i, _)| self.table.value(&self.text, RowRef::Row(i), c))
                    .map(|v| first_line(&v).chars().count())
                    .max()
                    .unwrap_or(0);
                header.max(longest).clamp(MIN_CHARS, MAX_CHARS) as f32
            })
            .collect()
    }

    pub(crate) fn value(&self, row: RowRef, col: usize) -> Option<Cow<'_, str>> {
        self.table.value(&self.text, row, col)
    }

    fn refresh_order(&mut self) {
        let needle = self.filter.to_lowercase();
        let mut order: Vec<usize> = (0..self.table.rows.len())
            .filter(|&r| {
                needle.is_empty()
                    || (0..self.table.columns.len()).any(|c| {
                        self.value(RowRef::Row(r), c).is_some_and(|v| v.to_lowercase().contains(&needle))
                    })
            })
            .collect();
        if let Some((col, ascending)) = self.sort {
            let key = |r: usize| self.value(RowRef::Row(r), col).map(Cow::into_owned);
            order.sort_by(|&a, &b| compare_cells(key(a).as_deref(), key(b).as_deref(), ascending));
        }
        self.order = order;
    }

    pub fn update(&mut self, msg: Msg) -> Action {
        if !matches!(msg, Msg::Scrolled { .. } | Msg::EditInput(_)) {
            self.error = None;
        }
        match msg {
            Msg::Scrolled { x, y, height } => {
                let moved_x = (x - self.scroll.0).abs() > 0.5;
                self.scroll = (x, y);
                self.viewport = height;
                if moved_x {
                    return Action::Task(operation::scroll_to(
                        self.header_id.clone(),
                        operation::AbsoluteOffset { x: Some(x), y: None },
                    ));
                }
                Action::None
            }
            Msg::Select(row, col) => {
                self.selected = Some((row, col));
                self.editing = None;
                Action::Focus(Task::none())
            }
            Msg::StartEdit(row, col) => self.start_edit(row, col),
            Msg::EditInput(value) => {
                if let Some(e) = &mut self.editing {
                    e.value = value;
                }
                Action::None
            }
            Msg::CommitEdit => {
                let Some(e) = self.editing.take() else { return Action::None };
                match self.table.edit(&self.text, e.row, e.col, &e.value) {
                    Ok(edit) if self.text[edit.range.clone()] == edit.text => Action::None,
                    Ok(edit) => Action::Edit(edit),
                    Err(err) => {
                        self.error = Some(match err {
                            EditError::InvalidJson(m) => fl!("table-invalid-json", error = m),
                            EditError::BrokenRow => fl!("table-broken-row"),
                            EditError::OutOfRange => fl!("table-out-of-range"),
                        });
                        // Continua editando, para corrigir.
                        self.editing = Some(e);
                        Action::Task(operation::focus(self.input_id.clone()))
                    }
                }
            }
            Msg::Sort(col) => {
                self.sort = match self.sort {
                    Some((c, true)) if c == col => Some((col, false)),
                    Some((c, false)) if c == col => None,
                    _ => Some((col, true)),
                };
                self.refresh_order();
                Action::None
            }
            Msg::Filter(filter) => {
                self.filter = filter;
                self.refresh_order();
                Action::None
            }
            Msg::Key(key) => self.key(key),
            Msg::Copy => match self.selected.and_then(|(r, c)| self.value(r, c)) {
                Some(v) => Action::Task(iced::clipboard::write(v.into_owned())),
                None => Action::None,
            },
            Msg::ShowText => {
                let at = self.selected.and_then(|(r, c)| {
                    let row = match r {
                        RowRef::Header => self.table.header.as_ref()?,
                        RowRef::Row(i) => self.table.rows.get(i)?,
                    };
                    Some(row.cells.get(c).cloned().flatten().map_or(row.end, |cell| cell.span.start))
                });
                Action::ShowText(at)
            }
        }
    }

    fn start_edit(&mut self, row: RowRef, col: usize) -> Action {
        if col >= self.table.columns.len() {
            return Action::None;
        }
        self.selected = Some((row, col));
        let value = self.table.raw(&self.text, row, col).map(Cow::into_owned).unwrap_or_default();
        self.editing = Some(Editing { row, col, value });
        Action::Focus(Task::batch([operation::focus(self.input_id.clone()), operation::select_all(self.input_id.clone())]))
    }

    /// Teclas com a aba focada e sem célula em edição (a edição tem as dela no campo).
    fn key(&mut self, key: Named) -> Action {
        if self.editing.is_some() {
            if key == Named::Escape {
                self.editing = None;
            }
            return Action::None;
        }
        let Some((row, col)) = self.selected else {
            if matches!(key, Named::ArrowDown | Named::ArrowUp | Named::ArrowLeft | Named::ArrowRight)
                && let Some(&first) = self.order.first()
            {
                self.selected = Some((RowRef::Row(first), 0));
            }
            return Action::None;
        };
        let pos = match row {
            RowRef::Header => None,
            RowRef::Row(r) => self.order.iter().position(|&o| o == r),
        };
        let last_col = self.table.columns.len().saturating_sub(1);
        let to_row = |p: Option<usize>| match p {
            None => RowRef::Header,
            Some(p) => RowRef::Row(self.order[p]),
        };
        let has_header = self.table.header.is_some();
        let new = match key {
            Named::ArrowLeft => (row, col.saturating_sub(1)),
            Named::ArrowRight => (row, (col + 1).min(last_col)),
            Named::ArrowUp => match pos {
                Some(0) if has_header => (RowRef::Header, col),
                Some(p) => (to_row(Some(p.saturating_sub(1))), col),
                None => (row, col),
            },
            Named::ArrowDown => match pos {
                None if !self.order.is_empty() => (to_row(Some(0)), col),
                Some(p) if p + 1 < self.order.len() => (to_row(Some(p + 1)), col),
                _ => (row, col),
            },
            Named::Enter | Named::F2 => return self.start_edit(row, col),
            Named::Escape => {
                self.selected = None;
                return Action::None;
            }
            _ => return Action::None,
        };
        self.selected = Some(new);
        self.scroll_into_view(new.0)
    }

    /// Rola o corpo até a linha escolhida, se ela saiu da tela.
    fn scroll_into_view(&self, row: RowRef) -> Action {
        let RowRef::Row(r) = row else { return Action::None };
        let Some(p) = self.order.iter().position(|&o| o == r) else { return Action::None };
        let h = row_height(self.font_size);
        let top = p as f32 * h;
        let y = if top < self.scroll.1 {
            top
        } else if top + h > self.scroll.1 + self.viewport {
            top + h - self.viewport
        } else {
            return Action::None;
        };
        Action::Task(operation::scroll_to(self.body_id.clone(), operation::AbsoluteOffset { x: None, y: Some(y) }))
    }

    /// As teclas que a tabela quer quando a aba está focada.
    pub fn wants(&self, key: Named) -> bool {
        match key {
            Named::Escape => true,
            Named::ArrowUp | Named::ArrowDown | Named::ArrowLeft | Named::ArrowRight | Named::Enter | Named::F2 => {
                self.editing.is_none()
            }
            _ => false,
        }
    }

    pub fn view(&self, font: Font, size: f32, theme: &Theme) -> Element<'_, Msg> {
        let palette = theme.extended_palette();
        let char_w = size * 0.6;
        let h = row_height(size);
        let number_w = (self.table.rows.last().map_or(1, |r| r.line + 1).to_string().len() as f32 + 1.0) * char_w + 12.0;
        let col_w = |c: usize| self.widths.get(c).copied().unwrap_or(10.0) * char_w + 14.0;
        let total_w = number_w + (0..self.table.columns.len()).map(col_w).sum::<f32>();
        let muted = palette.background.base.text.scale_alpha(0.5);
        let selected_bg = palette.primary.weak.color.scale_alpha(0.35);
        let border = palette.background.strong.color;

        // Barra: voltar ao texto, filtro, contagem, erro.
        let count = if self.order.len() == self.table.rows.len() {
            fl!("table-rows", count = self.table.rows.len())
        } else {
            fl!("table-rows-filtered", shown = self.order.len(), count = self.table.rows.len())
        };
        let mut info = count;
        if let Some((c, asc)) = self.sort {
            info.push_str(&format!(
                " · {}",
                fl!("table-sorted", column = self.table.columns.get(c).cloned().unwrap_or_default(), arrow = if asc { "↑" } else { "↓" })
            ));
        }
        let mut bar = row![
            button(text(fl!("table-show-text")).size(size * 0.9)).style(button::secondary).on_press(Msg::ShowText),
            text_input(&fl!("table-filter"), &self.filter).on_input(Msg::Filter).size(size * 0.9).width(Length::Fixed(220.0)),
            text(info).size(size * 0.85).color(muted),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        if let Some(e) = &self.error {
            bar = bar.push(text(e.clone()).size(size * 0.85).color(palette.danger.base.color));
        }

        // Cabeçalho: os nomes; clicar ordena. Num CSV com cabeçalho, dois cliques editam o nome.
        let mut header = row![container(text("#").size(size * 0.85).color(muted)).width(number_w).padding([4, 6])];
        for (c, name) in self.table.columns.iter().enumerate() {
            let arrow = match self.sort {
                Some((s, true)) if s == c => " ↑",
                Some((s, false)) if s == c => " ↓",
                _ => "",
            };
            let is_selected = self.selected == Some((RowRef::Header, c));
            let cell: Element<'_, Msg> = if let Some(e) = self.editing.as_ref().filter(|e| e.row == RowRef::Header && e.col == c) {
                self.input(e, font, size, col_w(c))
            } else {
                let label = container(text(format!("{name}{arrow}")).size(size).font(Font { weight: iced::font::Weight::Bold, ..font }).wrapping(text::Wrapping::None))
                    .width(col_w(c))
                    .height(h)
                    .padding([2, 6])
                    .clip(true)
                    .style(move |_| cell_style(is_selected.then_some(selected_bg), border));
                let mut area = mouse_area(label).on_press(Msg::Sort(c));
                if self.table.header.is_some() {
                    area = area.on_double_click(Msg::StartEdit(RowRef::Header, c)).on_right_press(Msg::Select(RowRef::Header, c));
                }
                area.into()
            };
            header = header.push(cell);
        }
        let header = style::scrollable(header.width(total_w))
            .id(self.header_id.clone())
            .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::hidden()));

        // Corpo: só as linhas visíveis; espaços em cima e embaixo fazem a rolagem ter o tamanho todo.
        let first = ((self.scroll.1 / h).floor() as usize).min(self.order.len());
        let count = ((self.viewport / h).ceil() as usize + 2).min(self.order.len() - first);
        let mut body = column![space().height(first as f32 * h)];
        for &r in &self.order[first..first + count] {
            body = body.push(self.row_view(r, font, size, h, number_w, &col_w, total_w, selected_bg, border, muted, palette.danger.base.color));
        }
        body = body.push(space().height((self.order.len() - first - count) as f32 * h));
        let body = style::scrollable(body.width(total_w))
            .id(self.body_id.clone())
            .direction(scrollable::Direction::Both { vertical: style::scrollbar(), horizontal: style::scrollbar() })
            .on_scroll(|v| Msg::Scrolled { x: v.absolute_offset().x, y: v.absolute_offset().y, height: v.bounds().height })
            .width(Fill)
            .height(Fill);

        column![container(bar).padding([6, 8]), header, body].width(Fill).height(Fill).into()
    }

    #[allow(clippy::too_many_arguments)]
    fn row_view<'a>(
        &'a self,
        r: usize,
        font: Font,
        size: f32,
        h: f32,
        number_w: f32,
        col_w: &dyn Fn(usize) -> f32,
        total_w: f32,
        selected_bg: Color,
        border: Color,
        muted: Color,
        danger: Color,
    ) -> Element<'a, Msg> {
        let data = &self.table.rows[r];
        let number = container(text((data.line + 1).to_string()).size(size * 0.85).color(muted))
            .width(number_w)
            .height(h)
            .padding([2, 6])
            .align_x(Alignment::End);
        let mut line = row![number];
        if let Some(err) = &data.error {
            // JSONL que não é um objeto: o texto da linha inteira, marcado.
            let raw = self.text.split('\n').nth(data.line).unwrap_or_default();
            let tip = format!("⚠ {}   {raw}", fl!("table-invalid-json", error = err.clone()));
            let label = text(tip).size(size).font(font).color(danger).wrapping(text::Wrapping::None);
            return line.push(container(label).width(total_w - number_w).height(h).padding([2, 6]).clip(true)).into();
        }
        for c in 0..self.table.columns.len() {
            let w = col_w(c);
            if let Some(e) = self.editing.as_ref().filter(|e| e.row == RowRef::Row(r) && e.col == c) {
                line = line.push(self.input(e, font, size, w));
                continue;
            }
            let value = self.value(RowRef::Row(r), c);
            let kind = self.table.json_kind(&self.text, RowRef::Row(r), c);
            let shown = value.as_deref().map(first_line).unwrap_or_default().to_string();
            let numeric = match kind {
                Some(k) => k == JsonKind::Number,
                None => !shown.is_empty() && shown.trim().parse::<f64>().is_ok(),
            };
            let mut label = text(shown).size(size).font(font).wrapping(text::Wrapping::None);
            if kind == Some(JsonKind::Null) {
                label = label.color(muted);
            }
            let is_selected = self.selected == Some((RowRef::Row(r), c));
            let cell = container(label)
                .width(w)
                .height(h)
                .padding([2, 6])
                .clip(true)
                .align_x(if numeric { Alignment::End } else { Alignment::Start })
                .style(move |_| cell_style(is_selected.then_some(selected_bg), border));
            line = line.push(
                mouse_area(cell).on_press(Msg::Select(RowRef::Row(r), c)).on_double_click(Msg::StartEdit(RowRef::Row(r), c)),
            );
        }
        line.into()
    }

    fn input<'a>(&self, e: &'a Editing, font: Font, size: f32, width: f32) -> Element<'a, Msg> {
        text_input("", &e.value)
            .id(self.input_id.clone())
            .on_input(Msg::EditInput)
            .on_submit(Msg::CommitEdit)
            .font(font)
            .size(size)
            .padding([1, 5])
            .width(width.max(160.0))
            .into()
    }
}

fn row_height(size: f32) -> f32 {
    (size * 1.7).round()
}

fn first_line(v: &str) -> &str {
    v.split('\n').next().unwrap_or_default()
}

/// Números como números; o resto sem diferenciar maiúsculas; vazio por último nas duas ordens.
fn compare_cells(a: Option<&str>, b: Option<&str>, ascending: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.filter(|s| !s.is_empty()), b.filter(|s| !s.is_empty())) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => {
            let ord = match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
                (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
                _ => a.to_lowercase().cmp(&b.to_lowercase()),
            };
            if ascending { ord } else { ord.reverse() }
        }
    }
}

fn cell_style(background: Option<Color>, border: Color) -> container::Style {
    container::Style {
        background: background.map(Into::into),
        border: iced::Border { color: border.scale_alpha(0.35), width: 0.5, radius: 0.0.into() },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(text: &str) -> TableView {
        TableView::new(Format::Csv(b','), text, 1)
    }

    #[test]
    fn sort_and_filter_change_only_the_order() {
        let text = "nome,n\nbia,10\nana,9\nzé,\n";
        let mut t = view(text);
        assert_eq!(t.order, [0, 1, 2]);
        let _ = t.update(Msg::Sort(1));
        assert_eq!(t.order, [1, 0, 2], "9 antes de 10 (número), vazio por último");
        let _ = t.update(Msg::Sort(1));
        assert_eq!(t.order, [0, 1, 2], "10 antes de 9, vazio ainda por último");
        let _ = t.update(Msg::Sort(0));
        assert_eq!(t.order, [1, 0, 2]);
        let _ = t.update(Msg::Filter("AN".into()));
        assert_eq!(t.order, [1], "só ana tem \"an\" (sem diferenciar maiúsculas)");
        assert_eq!(t.text, text, "o texto não muda");
    }

    #[test]
    fn commit_returns_the_edit_and_reload_follows() {
        let text = "nome,n\nana,1\n";
        let mut t = view(text);
        let _ = t.update(Msg::StartEdit(RowRef::Row(0), 0));
        let _ = t.update(Msg::EditInput("ana, b".into()));
        let Action::Edit(edit) = t.update(Msg::CommitEdit) else { panic!("sem edição") };
        let new = tea_table::apply(text, &edit);
        assert_eq!(new, "nome,n\n\"ana, b\",1\n");
        t.reload(&new, 2);
        assert_eq!(t.value(RowRef::Row(0), 0).as_deref(), Some("ana, b"));
        // Sem mudança: nada a aplicar.
        let _ = t.update(Msg::StartEdit(RowRef::Row(0), 1));
        assert!(matches!(t.update(Msg::CommitEdit), Action::None));
    }

    #[test]
    fn invalid_json_keeps_editing_with_an_error() {
        let text = "{\"id\": 1}\n";
        let mut t = TableView::new(Format::Jsonl, text, 1);
        let _ = t.update(Msg::StartEdit(RowRef::Row(0), 0));
        let _ = t.update(Msg::EditInput("um".into()));
        assert!(matches!(t.update(Msg::CommitEdit), Action::Task(_)));
        assert!(t.error.is_some());
        assert!(t.editing.is_some());
    }

    #[test]
    fn arrows_move_in_screen_order() {
        let mut t = view("n\n3\n1\n2\n");
        let _ = t.update(Msg::Sort(0));
        let _ = t.update(Msg::Key(Named::ArrowDown));
        assert_eq!(t.selected, Some((RowRef::Row(1), 0)), "a primeira na tela é o 1");
        let _ = t.update(Msg::Key(Named::ArrowDown));
        assert_eq!(t.selected, Some((RowRef::Row(2), 0)));
        let _ = t.update(Msg::Key(Named::ArrowUp));
        let _ = t.update(Msg::Key(Named::ArrowUp));
        assert_eq!(t.selected, Some((RowRef::Header, 0)));
        let Action::ShowText(Some(at)) = t.update(Msg::ShowText) else { panic!() };
        assert_eq!(at, 0);
    }
}
