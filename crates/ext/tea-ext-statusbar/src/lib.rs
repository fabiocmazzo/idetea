//! Barra de status: linha/coluna, linguagem, encoding, fim de linha e notificações.

use std::time::{Duration, Instant};

use iced::widget::{row, space, text};
use iced::{Element, Task};
use tea_core::{Context, CoreEvent, EditorStatus, Extension, PanelId, Slot};

const PANEL: PanelId = PanelId("bar");
/// Depois disso, a notificação some no próximo movimento do editor.
const NOTIFICATION_TTL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct StatusBarExtension {
    status: EditorStatus,
    notification: String,
    notified_at: Option<Instant>,
}

impl StatusBarExtension {
    pub fn new() -> Self {
        Self::default()
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
            _ => {}
        }
        Task::none()
    }

    fn update(&mut self, message: Message, _ctx: &mut Context) -> Task<Message> {
        match message {}
    }

    fn view(&self, _panel: PanelId) -> Element<'_, Message> {
        let s = &self.status;
        let size = 12;
        let modified = if s.modified { "● não salvo" } else { "" };
        row![
            text(&self.notification).size(size),
            space::horizontal(),
            text(modified).size(size),
            space::horizontal(),
            text(format!("Ln {}, Col {}", s.line, s.column)).size(size),
            text(&s.language).size(size),
            text(s.encoding).size(size),
            text(s.line_ending).size(size),
        ]
        .spacing(16)
        .into()
    }
}
