//! Entrada suave das janelas sobre a IDE (RFC 0015, etapa 3): o conteúdo desliza alguns pixels até o lugar e, se
//! houver, o fundo escurecido aparece aos poucos. O iced 0.14 não tem opacidade para um bloco inteiro; o deslize e o
//! fundo bastam para a janela não "piscar" na tela.
//!
//! A animação começa no primeiro quadro em que o widget aparece e roda uma vez: enquanto a janela continua aberta,
//! o estado (no `Tree`) fica, e ela não se repete.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, overlay, renderer};
use iced::time::Instant;
use iced::{Color, Element, Event, Length, Rectangle, Size, Theme, Vector, window};

/// Duração da entrada: curta o bastante para não atrasar quem usa o teclado.
pub const DURATION: Duration = Duration::from_millis(140);

pub struct Appear<'a, M> {
    content: Element<'a, M>,
    /// Deslocamento vertical no começo (px); negativo vem de cima.
    slide: f32,
    backdrop: Option<Color>,
    enabled: bool,
}

/// `[ui] animations`: vale para todas as janelas; o app atualiza ao ler a config.
static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Com `[ui] animations = false`, mostra direto, sem animação.
pub fn appear<'a, M>(content: impl Into<Element<'a, M>>) -> Appear<'a, M> {
    Appear { content: content.into(), slide: 6.0, backdrop: None, enabled: enabled() }
}

impl<M> Appear<'_, M> {
    pub fn slide(mut self, px: f32) -> Self {
        self.slide = px;
        self
    }

    /// Fundo que cobre a área toda e chega a esta cor aos poucos.
    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = Some(color);
        self
    }

    /// De 0 a 1, já com a curva (sai rápido e assenta devagar).
    fn progress(&self, state: &State, now: Instant) -> f32 {
        if !self.enabled {
            return 1.0;
        }
        let Some(start) = state.start else { return 0.0 };
        let t = (now.saturating_duration_since(start).as_secs_f32() / DURATION.as_secs_f32()).clamp(0.0, 1.0);
        ease_out(t)
    }
}

/// Cúbica de saída: `1 − (1 − t)³`.
pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

#[derive(Default)]
struct State {
    start: Option<Instant>,
}

impl<M> Widget<M, Theme, iced::Renderer> for Appear<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        if self.enabled
            && let Event::Window(window::Event::RedrawRequested(now)) = event
        {
            let state = tree.state.downcast_mut::<State>();
            let start = *state.start.get_or_insert(*now);
            if now.saturating_duration_since(start) < DURATION {
                shell.request_redraw();
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let t = self.progress(tree.state.downcast_ref::<State>(), Instant::now());
        if let Some(color) = self.backdrop {
            renderer.fill_quad(
                renderer::Quad { bounds: layout.bounds(), ..renderer::Quad::default() },
                color.scale_alpha(t),
            );
        }
        let shift = Vector::new(0.0, self.slide * (1.0 - t));
        renderer.with_translation(shift, |renderer| {
            self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a, M: 'a> From<Appear<'a, M>> for Element<'a, M> {
    fn from(a: Appear<'a, M>) -> Self {
        Element::new(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_starts_fast_and_settles() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.5) > 0.8, "metade do tempo já anda mais de 80%");
    }

    #[test]
    fn without_animations_it_shows_at_once() {
        let a = Appear::<()> { content: iced::widget::space().into(), slide: 6.0, backdrop: None, enabled: false };
        assert_eq!(a.progress(&State::default(), Instant::now()), 1.0);
    }
}
