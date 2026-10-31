//! Splash screen: aparece com fade, fica no mínimo 2 s e some com fade
//! quando tudo já carregou. A IDE carrega por trás dela.

use std::time::{Duration, Instant};

use iced::widget::{Space, center, column, container, text};
use iced::{Color, Element, Fill, Font, Theme, font};

pub const MIN_VISIBLE: Duration = Duration::from_secs(2);
const FADE_IN: Duration = Duration::from_millis(350);
const FADE_OUT: Duration = Duration::from_millis(450);
const BAR_WIDTH: f32 = 180.0;

pub struct Splash {
    started: Instant,
    now: Instant,
    /// Quando a IDE terminou de carregar.
    ready_at: Option<Instant>,
}

impl Splash {
    pub fn new(now: Instant) -> Self {
        Self { started: now, now, ready_at: None }
    }

    pub fn mark_ready(&mut self, now: Instant) {
        self.ready_at.get_or_insert(now);
    }

    pub fn tick(&mut self, now: Instant) {
        self.now = now;
    }

    /// Momento em que o fade de saída começa: 2 s depois do início ou quando carregar, o que vier por último.
    fn fade_out_start(&self) -> Option<Instant> {
        self.ready_at.map(|ready| ready.max(self.started + MIN_VISIBLE))
    }

    pub fn is_finished(&self) -> bool {
        self.fade_out_start().is_some_and(|start| self.now >= start + FADE_OUT)
    }

    /// Opacidade do conjunto (0 a 1).
    pub fn opacity(&self) -> f32 {
        let since_start = self.now.saturating_duration_since(self.started);
        let fade_in = (since_start.as_secs_f32() / FADE_IN.as_secs_f32()).min(1.0);
        let fade_out = match self.fade_out_start() {
            Some(start) if self.now > start => {
                1.0 - (self.now.duration_since(start).as_secs_f32() / FADE_OUT.as_secs_f32()).min(1.0)
            }
            _ => 1.0,
        };
        ease(fade_in.min(fade_out))
    }

    /// Progresso da barra: anda com o tempo mínimo e só completa quando carregou.
    fn progress(&self) -> f32 {
        let by_time = self.now.saturating_duration_since(self.started).as_secs_f32() / MIN_VISIBLE.as_secs_f32();
        let cap = if self.ready_at.is_some() { 1.0 } else { 0.9 };
        ease(by_time.min(cap))
    }

    pub fn view<'a, M: 'a>(&self) -> Element<'a, M> {
        let alpha = self.opacity();
        // A logo sobe levemente durante o fade de entrada.
        let lift = (1.0 - alpha) * 12.0;
        let progress = self.progress();
        let status = if self.ready_at.is_some() { fl!("splash-ready") } else { fl!("splash-loading") };

        let title = text("TEA")
            .size(84)
            .font(Font { weight: font::Weight::Light, ..Font::DEFAULT })
            .style(move |t: &Theme| text::Style { color: Some(fade(t.extended_palette().primary.strong.color, alpha)) });
        let subtitle = text("Just code. Add ice.").size(15).style(move |t: &Theme| text::Style {
            color: Some(fade(t.extended_palette().background.base.text, alpha * 0.7)),
        });
        let bar = container(
            container(Space::new().width(BAR_WIDTH * progress).height(2)).style(move |t: &Theme| {
                container::Style::default().background(fade(t.extended_palette().primary.base.color, alpha))
            }),
        )
        .width(BAR_WIDTH)
        .style(move |t: &Theme| {
            container::Style::default().background(fade(t.extended_palette().background.strong.color, alpha * 0.5))
        });
        let footer = text(format!("IDE TEA v{} · {status}", env!("CARGO_PKG_VERSION"))).size(12).style(
            move |t: &Theme| text::Style { color: Some(fade(t.extended_palette().background.base.text, alpha * 0.45)) },
        );

        let content = column![Space::new().height(lift), title, subtitle, Space::new().height(28), bar, Space::new().height(10), footer]
            .align_x(iced::Center)
            .spacing(6);

        container(center(content))
            .width(Fill)
            .height(Fill)
            .style(move |t: &Theme| container::Style::default().background(fade(t.extended_palette().background.base.color, alpha)))
            .into()
    }
}

fn fade(color: Color, alpha: f32) -> Color {
    Color { a: color.a * alpha, ..color }
}

/// Suaviza o começo e o fim da animação.
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn stays_at_least_two_seconds_when_ready_early() {
        let t0 = Instant::now();
        let mut s = Splash::new(t0);
        s.mark_ready(t0 + MS(10));
        s.tick(t0 + MS(1999));
        assert!(!s.is_finished());
        assert!((s.opacity() - 1.0).abs() < 1e-6);
        s.tick(t0 + MIN_VISIBLE + FADE_OUT);
        assert!(s.is_finished());
    }

    #[test]
    fn waits_for_slow_loading() {
        let t0 = Instant::now();
        let mut s = Splash::new(t0);
        s.tick(t0 + MS(5000));
        assert!(!s.is_finished(), "não some antes de carregar");
        assert!(s.progress() < 1.0);
        s.mark_ready(t0 + MS(5000));
        s.tick(t0 + MS(5000) + FADE_OUT);
        assert!(s.is_finished());
    }

    #[test]
    fn fades_in_and_out() {
        let t0 = Instant::now();
        let mut s = Splash::new(t0);
        s.mark_ready(t0);
        s.tick(t0);
        assert!(s.opacity() < 0.01);
        s.tick(t0 + MIN_VISIBLE + FADE_OUT / 2);
        assert!(s.opacity() > 0.1 && s.opacity() < 0.9);
    }
}
