//! O diff de uma edição do agente (`ToolCallContent::Diff`), em linhas `+`/`−` com um pouco de contexto.

use std::path::Path;

use iced::widget::{button, column, container, text};
use iced::{Element, Fill, Font};
use similar::{Algorithm, DiffOp, capture_diff_slices};

use crate::Message;

/// Linhas iguais mostradas em volta de cada mudança.
const CONTEXT: usize = 2;
/// Linhas de diff mostradas no máximo; o resto vira "… mais N linhas".
const MAX_LINES: usize = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Same,
    Added,
    Removed,
    /// Linhas iguais escondidas.
    Gap,
}

/// As linhas do diff, já com o contexto cortado. Arquivo novo (`old: None`): tudo `+`.
pub fn lines(old: Option<&str>, new: &str) -> Vec<(Mark, String)> {
    let old: Vec<&str> = old.map(|o| o.lines().collect()).unwrap_or_default();
    let new: Vec<&str> = new.lines().collect();
    let mut out = Vec::new();
    let ops = capture_diff_slices(Algorithm::Myers, &old, &new);
    let last = ops.len().saturating_sub(1);
    for (i, op) in ops.iter().enumerate() {
        match *op {
            DiffOp::Equal { old_index, len, .. } => {
                let run = &old[old_index..old_index + len];
                let head = if i == 0 { 0 } else { CONTEXT.min(run.len()) };
                let tail = if i == last { 0 } else { CONTEXT.min(run.len() - head) };
                out.extend(run[..head].iter().map(|l| (Mark::Same, (*l).to_string())));
                if run.len() > head + tail {
                    out.push((Mark::Gap, String::new()));
                }
                out.extend(run[run.len() - tail..].iter().map(|l| (Mark::Same, (*l).to_string())));
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                out.extend(old[old_index..old_index + old_len].iter().map(|l| (Mark::Removed, (*l).to_string())));
            }
            DiffOp::Insert { new_index, new_len, .. } => {
                out.extend(new[new_index..new_index + new_len].iter().map(|l| (Mark::Added, (*l).to_string())));
            }
            DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                out.extend(old[old_index..old_index + old_len].iter().map(|l| (Mark::Removed, (*l).to_string())));
                out.extend(new[new_index..new_index + new_len].iter().map(|l| (Mark::Added, (*l).to_string())));
            }
        }
    }
    out
}

pub fn view<'a>(shown: &str, path: &Path, old: Option<&str>, new: &str) -> Element<'a, Message> {
    let all = lines(old, new);
    let mut col = column![
        button(text(shown.to_string()).size(11).font(Font::MONOSPACE)).style(button::text).padding([0, 2]).on_press(Message::Open(path.to_path_buf(), None))
    ]
    .spacing(0)
    .width(Fill);
    for (mark, line) in all.iter().take(MAX_LINES) {
        let row = match mark {
            Mark::Added => text(format!("+ {line}")).style(text::success),
            Mark::Removed => text(format!("− {line}")).style(text::danger),
            Mark::Same => text(format!("  {line}")).style(text::secondary),
            Mark::Gap => text("  ⋯").style(text::secondary),
        };
        col = col.push(row.size(12).font(Font::MONOSPACE));
    }
    if all.len() > MAX_LINES {
        let more: usize = all.len() - MAX_LINES;
        col = col.push(text(fl!("acp-more-lines", count = more)).size(11).style(text::secondary));
    }
    container(col).padding(8).width(Fill).style(container::bordered_box).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_a_little_context_and_hides_the_rest() {
        let old = "a\nb\nc\nd\ne\nf\ng\nh\n";
        let new = "a\nb\nc\nd\nE\nf\ng\nh\n";
        let marks: Vec<Mark> = lines(Some(old), new).into_iter().map(|(m, _)| m).collect();
        assert_eq!(marks, [Mark::Gap, Mark::Same, Mark::Same, Mark::Removed, Mark::Added, Mark::Same, Mark::Same, Mark::Gap]);
    }

    #[test]
    fn a_new_file_is_all_added() {
        let out = lines(None, "x\ny");
        assert_eq!(out, [(Mark::Added, "x".into()), (Mark::Added, "y".into())]);
    }
}
