//! Barra de status: notificação, atividade em segundo plano, erros/avisos,
//! posição do cursor, linguagem, encoding e fim de linha.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::widget::{button, row, space, text};
use iced::{Element, Task, Theme};
use tea_core::icons::{Icon, icon};
use tea_core::{Context, CoreEvent, EditorStatus, Extension, PanelId, Severity, Slot};

const PANEL: PanelId = PanelId("bar");
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
        let modified = if s.modified { "● não salvo" } else { "" };
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

        row![
            text(&self.notification).size(size),
            space::horizontal(),
            text(activity).size(size).style(text::secondary),
            text(modified).size(size),
            problems,
            text(format!("Ln {}, Col {}", s.line, s.column)).size(size),
            text(&s.language).size(size),
            text(s.encoding).size(size),
            text(s.line_ending).size(size),
        ]
        .spacing(16)
        .align_y(iced::Center)
        .into()
    }
}
