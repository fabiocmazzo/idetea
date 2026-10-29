//! Marcas de debug sobre o editor: breakpoints (bolinha na margem) e a linha onde o programa
//! parou (seta + faixa). O widget de edição não tem API para isso, então é uma camada por cima dele.
//!
//! Como achar onde cada linha está na tela sem mexer no widget: para cada faixa visível
//! (altura de uma linha), perguntamos ao widget qual linha do arquivo está ali
//! (`lsp_position_at_point`, que já conhece dobras e quebra de linha). A rolagem vem de
//! `viewport_scroll`. É o mesmo cálculo que o próprio widget usa para posicionar o hover do LSP.
//!
//! A camada só reage ao mouse em cima dos números de linha: no resto ela é "transparente"
//! (não pega cliques nem o cursor), senão o editor embaixo deixaria de ver o mouse.

use std::collections::BTreeSet;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, renderer};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Theme};
use iced_code_editor::CodeEditor;

/// Largura da coluna dos números de linha no widget (`GUTTER_WIDTH`, privado lá).
/// Um teste confere que o widget ainda usa esses valores.
pub(crate) const LINE_NUMBERS_WIDTH: f32 = 45.0;
/// Números + coluna das dobras (`FOLD_MARGIN_WIDTH` = 14): onde o texto começa.
pub(crate) const GUTTER_WIDTH: f32 = LINE_NUMBERS_WIDTH + 14.0;

/// Uma faixa visível: onde ela está na tela (y relativo ao topo do editor) e qual linha do arquivo mostra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Row {
    pub y: f32,
    /// Linha do arquivo, começando em 0.
    pub line: usize,
}

/// Faixas visíveis, uma por linha do arquivo (as continuações de uma linha quebrada ficam de fora).
pub(crate) fn visible_rows(editor: &CodeEditor) -> Vec<Row> {
    let lh = editor.line_height();
    if lh <= 0.0 {
        return Vec::new();
    }
    let scroll = editor.viewport_scroll();
    let first = (scroll / lh).floor() as usize;
    let count = (editor.viewport_height() / lh).ceil() as usize + 1;
    let mut rows = Vec::with_capacity(count);
    let mut prev: Option<usize> = None;
    for r in first..first + count {
        let y_content = r as f32 * lh + lh / 2.0;
        // Um pouco depois da margem: na margem o widget não responde.
        let Some(pos) = editor.lsp_position_at_point(Point::new(GUTTER_WIDTH + 2.0, y_content)) else { continue };
        let line = pos.line as usize;
        // Mesma linha de antes: continuação de linha quebrada, ou além do fim do arquivo.
        if prev == Some(line) {
            continue;
        }
        prev = Some(line);
        rows.push(Row { y: r as f32 * lh - scroll, line });
    }
    rows
}

/// Última linha do arquivo (começando em 0), sem montar o texto todo: um ponto muito abaixo
/// do fim cai na última linha.
pub(crate) fn last_line(editor: &CodeEditor) -> usize {
    editor.lsp_position_at_point(Point::new(GUTTER_WIDTH + 2.0, f32::MAX / 4.0)).map_or(0, |p| p.line as usize)
}

/// Ajusta os breakpoints depois de uma edição que mudou o número de linhas em `delta`,
/// feita na linha `at`: os de baixo andam junto com o texto. Os que caíram dentro de linhas
/// apagadas vão para a linha da edição.
pub(crate) fn shift_breakpoints(breakpoints: &mut BTreeSet<usize>, at: usize, delta: isize) {
    if delta == 0 {
        return;
    }
    *breakpoints = breakpoints
        .iter()
        .map(|&b| {
            if b <= at {
                b
            } else if delta > 0 {
                b + delta as usize
            } else {
                b.saturating_sub(delta.unsigned_abs()).max(at)
            }
        })
        .collect();
}

pub(crate) struct Marks<'a, M> {
    rows: Vec<Row>,
    breakpoints: &'a BTreeSet<usize>,
    exec_line: Option<usize>,
    line_height: f32,
    on_toggle: Box<dyn Fn(usize) -> M + 'a>,
}

impl<'a, M> Marks<'a, M> {
    pub fn new(
        editor: &CodeEditor,
        breakpoints: &'a BTreeSet<usize>,
        exec_line: Option<usize>,
        on_toggle: impl Fn(usize) -> M + 'a,
    ) -> Self {
        Self {
            rows: visible_rows(editor),
            breakpoints,
            exec_line,
            line_height: editor.line_height(),
            on_toggle: Box::new(on_toggle),
        }
    }

    /// Linha do arquivo na altura `y` (relativo ao topo do editor).
    fn line_at(&self, y: f32) -> Option<usize> {
        self.rows.iter().rev().find(|r| r.y <= y).filter(|r| y < r.y + self.line_height).map(|r| r.line)
    }

    fn in_numbers(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Point> {
        let p = cursor.position_in(bounds)?;
        (p.x < LINE_NUMBERS_WIDTH).then_some(p)
    }
}

#[derive(Default)]
struct State;

impl<M, Renderer: renderer::Renderer> Widget<M, Theme, Renderer> for Marks<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State)
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(p) = self.in_numbers(layout.bounds(), cursor)
            && let Some(line) = self.line_at(p.y)
        {
            shell.publish((self.on_toggle)(line));
            shell.capture_event();
        }
        // Passar o mouse nos números mostra a bolinha fraca: precisa redesenhar.
        if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        // `None` fora dos números: o editor embaixo continua vendo o mouse.
        if self.in_numbers(layout.bounds(), cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let p = theme.extended_palette();
        let lh = self.line_height;
        let hovered = self.in_numbers(b, cursor).and_then(|pt| self.line_at(pt.y));
        renderer.with_layer(b, |renderer| {
            for row in &self.rows {
                let top = b.y + row.y;
                if top + lh < b.y || top > b.y + b.height {
                    continue;
                }
                if self.exec_line == Some(row.line) {
                    // Faixa por trás do texto (translúcida) e seta na margem.
                    let band = Rectangle { x: b.x + GUTTER_WIDTH, y: top, width: b.width - GUTTER_WIDTH, height: lh };
                    renderer.fill_quad(
                        renderer::Quad { bounds: band, ..renderer::Quad::default() },
                        Color { a: 0.22, ..p.warning.base.color },
                    );
                    let arrow = Rectangle { x: b.x + 2.0, y: top + lh * 0.3, width: lh * 0.55, height: lh * 0.4 };
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: arrow,
                            border: iced::Border { radius: (lh * 0.2).into(), ..iced::Border::default() },
                            ..renderer::Quad::default()
                        },
                        p.warning.base.color,
                    );
                }
                let has_bp = self.breakpoints.contains(&row.line);
                if has_bp || hovered == Some(row.line) {
                    let d = lh * 0.55;
                    let dot = Rectangle { x: b.x + 4.0, y: top + (lh - d) / 2.0, width: d, height: d };
                    let color = if has_bp { p.danger.base.color } else { Color { a: 0.35, ..p.danger.base.color } };
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: dot,
                            border: iced::Border { radius: (d / 2.0).into(), ..iced::Border::default() },
                            ..renderer::Quad::default()
                        },
                        color,
                    );
                }
            }
        });
    }
}

impl<'a, M: 'a, Renderer: renderer::Renderer + 'a> From<Marks<'a, M>> for Element<'a, M, Theme, Renderer> {
    fn from(m: Marks<'a, M>) -> Self {
        Element::new(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(text: &str) -> CodeEditor {
        let mut e = CodeEditor::new(text, "rs");
        e.set_font_size(14.0, true);
        e
    }

    /// Se o widget mudar a largura da margem, as marcas sairiam do lugar: este teste avisa.
    #[test]
    fn gutter_widths_match_widget() {
        let e = editor("fn main() {}\n");
        let lh = e.line_height();
        assert!(e.lsp_position_at_point(Point::new(GUTTER_WIDTH - 1.0, lh / 2.0)).is_none(), "ainda na margem");
        assert!(e.lsp_position_at_point(Point::new(GUTTER_WIDTH + 1.0, lh / 2.0)).is_some(), "já no texto");
    }

    #[test]
    fn rows_follow_lines_and_stop_at_the_end() {
        let e = editor("a\nb\nc");
        let rows = visible_rows(&e);
        let lines: Vec<usize> = rows.iter().map(|r| r.line).collect();
        assert_eq!(lines, [0, 1, 2], "uma faixa por linha, nada repetido depois do fim");
        assert_eq!(rows[1].y, e.line_height());
        assert_eq!(last_line(&e), 2);
    }

    #[test]
    fn breakpoints_move_with_the_text() {
        let mut bps: BTreeSet<usize> = [2, 5, 9].into();
        shift_breakpoints(&mut bps, 3, 2); // duas linhas novas depois da 3
        assert_eq!(bps, [2, 7, 11].into());
        shift_breakpoints(&mut bps, 6, -3); // apagou as linhas 7..9
        assert_eq!(bps, [2, 6, 8].into(), "o da linha apagada vai para a linha da edição");
    }
}
