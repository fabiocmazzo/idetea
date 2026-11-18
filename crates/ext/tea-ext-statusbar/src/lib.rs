//! Barra de status: notificação, atividade em segundo plano, erros/avisos,
//! posição do cursor, linguagem, encoding e fim de linha.

tea_core::i18n_loader!();
macro_rules! fl {
    ($($t:tt)*) => { tea_core::i18n::fl!(crate::LOADER, $($t)*) };
}

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::widget::{button, container, row, text, tooltip};
use iced::{Element, Task, Theme};
use tea_core::icons::{Icon, icon};
use tea_core::{Context, CoreEvent, EditorStatus, Extension, PanelId, Severity, Slot};

const PANEL: PanelId = PanelId("bar");
/// Linhas de uma notificação longa no tooltip (o resto fica de fora).
const TOOLTIP_LINES: usize = 40;

/// A primeira linha com texto da notificação, com `…` quando há mais; e se há mais.
fn summary(msg: &str) -> (String, bool) {
    let mut lines = msg.lines().map(str::trim_end).filter(|l| !l.trim().is_empty());
    let first = lines.next().unwrap_or_default();
    let more = lines.next().is_some();
    (if more { format!("{first} …") } else { first.to_string() }, more)
}

/// Depois disso, a notificação some no próximo movimento do editor.
const NOTIFICATION_TTL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub enum Message {
    OpenProblems,
}

#[derive(Default)]
pub struct StatusBarExtension {
    status: EditorStatus,
    notification: String,
    notified_at: Option<Instant>,
    /// (arquivo, fonte) → (erros, avisos).
    counts: HashMap<(PathBuf, String), (usize, usize)>,
    /// fonte → texto da atividade.
    activity: BTreeMap<String, String>,
}

impl StatusBarExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn totals(&self) -> (usize, usize) {
        self.counts.values().fold((0, 0), |(e, w), (de, dw)| (e + de, w + dw))
    }
}

impl Extension for StatusBarExtension {
    type Message = Message;

    fn id(&self) -> &'static str {
        "statusbar"
    }

    fn activate(&mut self, ctx: &mut Context) -> Task<Message> {
        ctx.register_panel(PANEL, "Status", Slot::Status);
        Task::none()
    }

    fn on_event(&mut self, event: &CoreEvent, _ctx: &mut Context) -> Task<Message> {
        match event {
            CoreEvent::EditorStatus(status) => {
                self.status = status.clone();
                // Sem timer: a notificação expira quando o usuário volta a mexer no editor.
                if self.notified_at.is_some_and(|t| t.elapsed() > NOTIFICATION_TTL) {
                    self.notification.clear();
                    self.notified_at = None;
                }
            }
            CoreEvent::Notification(msg) => {
                self.notification = msg.clone();
                self.notified_at = Some(Instant::now());
            }
            CoreEvent::Diagnostics { source, path, items } => {
                let e = items.iter().filter(|d| d.severity == Severity::Error).count();
                let w = items.iter().filter(|d| d.severity == Severity::Warning).count();
                let key = (path.clone(), source.clone());
                if e + w == 0 {
                    self.counts.remove(&key);
                } else {
                    self.counts.insert(key, (e, w));
                }
            }
            CoreEvent::Activity { source, text } => match text {
                Some(t) => {
                    self.activity.insert(source.clone(), t.clone());
                }
                None => {
                    self.activity.remove(source);
                }
            },
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, ctx: &mut Context) -> Task<Message> {
        match message {
            Message::OpenProblems => ctx.run_command("problems.focus"),
        }
        Task::none()
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let s = &self.status;
        let size = 12;
        let modified = if s.modified { fl!("status-unsaved") } else { String::new() };
        let activity = self.activity.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(" · ");
        let (errors, warnings) = self.totals();
        let problems = button(
            row![
                icon(Icon::Error).size(13).style(move |t: &Theme| text::Style {
                    color: (errors > 0).then(|| t.extended_palette().danger.base.color),
                }),
                text(errors).size(size),
                icon(Icon::Warning).size(13).style(move |t: &Theme| text::Style {
                    color: (warnings > 0).then(|| t.extended_palette().warning.base.color),
                }),
                text(warnings).size(size),
            ]
            .spacing(4)
            .align_y(iced::Center),
        )
        .on_press(Message::OpenProblems)
        .style(button::text)
        .padding([0, 4]);

        // Só a primeira linha, cortada se não couber: um erro de várias linhas (o do rust-analyzer quando o
        // `cargo metadata` falha) faria a barra crescer até caber tudo. O texto inteiro fica no tooltip.
        let (first, more) = summary(&self.notification);
        let line = container(text(first).size(size).wrapping(text::Wrapping::None)).width(iced::Fill).clip(true);
        let notification: Element<'_, Message> = if more {
            let full = self.notification.lines().take(TOOLTIP_LINES).collect::<Vec<_>>().join("\n");
            tooltip(line, container(text(full).size(size)).padding(8).max_width(900).style(container::bordered_box), tooltip::Position::Top)
                .into()
        } else {
            line.into()
        };
        row![
            notification,
            text(activity).size(size).style(text::secondary),
            text(modified).size(size),
            problems,
            text(fl!("status-position", line = s.line, column = s.column)).size(size),
            text(&s.language).size(size),
            text(s.encoding).size(size),
            text(s.line_ending).size(size),
        ]
        .spacing(16)
        .align_y(iced::Center)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_message_shows_one_line() {
        let msg = "rust-analyzer: warning: Failed to run build scripts\n\nPlease refer to the logs\nerror: can't find lib";
        assert_eq!(summary(msg), ("rust-analyzer: warning: Failed to run build scripts …".to_string(), true));
        assert_eq!(summary("Salvo."), ("Salvo.".to_string(), false));
        assert_eq!(summary("\n  \nsó esta\n"), ("só esta".to_string(), false));
        assert_eq!(summary(""), (String::new(), false));
    }
}
