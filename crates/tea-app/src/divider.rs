//! Divisor arrastável entre um painel e o editor. Desenha uma linha fina (ou, entre ilhas, nada: o vão já separa);
//! o cursor vira "redimensionar" em cima dele. Duplo clique volta ao padrão.

use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, renderer};
use iced::{Element, Event, Length, Rectangle, Size, Theme};

/// Largura da área que pega o mouse (a linha desenhada tem 1 px).
const GRAB: f32 = 5.0;
const DOUBLE_CLICK: Duration = Duration::from_millis(400);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Linha em pé, arrasta para os lados (painéis laterais).
    Vertical,
    /// Linha deitada, arrasta para cima/baixo (painel inferior).
    Horizontal,
}

pub struct Divider<'a, M> {
    axis: Axis,
    on_drag: Box<dyn Fn(f32) -> M + 'a>,
    on_release: M,
    on_reset: M,
    islands: bool,
    animated: bool,
}

/// Quanto a barrinha leva para aparecer entre ilhas.
const FADE: Duration = Duration::from_millis(120);

/// `on_drag` recebe quanto o mouse andou (px) no eixo do arraste.
pub fn divider<'a, M: Clone>(axis: Axis, on_drag: impl Fn(f32) -> M + 'a, on_release: M, on_reset: M) -> Divider<'a, M> {
    Divider { axis, on_drag: Box::new(on_drag), on_release, on_reset, islands: false, animated: false }
}

impl<M> Divider<'_, M> {
    /// Entre ilhas (RFC 0015): ocupa o vão inteiro e só aparece com o mouse em cima.
    pub fn islands(mut self, on: bool) -> Self {
        self.islands = on;
        self
    }

    /// Entre ilhas, a barrinha surge aos poucos em vez de piscar (`[ui] animations`).
    pub fn animated(mut self, on: bool) -> Self {
        self.animated = on;
        self
    }

    /// Opacidade da barrinha entre ilhas: 0 sem o mouse, 1 arrastando ou depois de `FADE`.
    fn reveal(&self, state: &State, now: Instant) -> f32 {
        match (state.dragging, state.hover_since) {
            (Some(_), _) => 1.0,
            (None, None) => 0.0,
            (None, Some(_)) if !self.animated => 1.0,
            (None, Some(since)) => {
                crate::appear::ease_out((now.saturating_duration_since(since).as_secs_f32() / FADE.as_secs_f32()).min(1.0))
            }
        }
    }

    fn grab(&self) -> f32 {
        if self.islands { tea_core::style::GAP } else { GRAB }
    }
}

#[derive(Default)]
struct State {
    /// Posição do mouse no último evento do arraste.
    dragging: Option<f32>,
    last_press: Option<Instant>,
    /// Desde quando o mouse está em cima (para a barrinha surgir aos poucos).
    hover_since: Option<Instant>,
}

impl<M: Clone, Renderer: renderer::Renderer> Widget<M, Theme, Renderer> for Divider<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        match self.axis {
            Axis::Vertical => Size::new(Length::Fixed(self.grab()), Length::Fill),
            Axis::Horizontal => Size::new(Length::Fill, Length::Fixed(self.grab())),
        }
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = Widget::<M, Theme, Renderer>::size(self);
        layout::Node::new(limits.resolve(size.width, size.height, Size::ZERO))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let along = |p: iced::Point| match self.axis {
            Axis::Vertical => p.x,
            Axis::Horizontal => p.y,
        };
        let over = cursor.is_over(layout.bounds());
        match (over, state.hover_since) {
            (true, None) => {
                state.hover_since = Some(Instant::now());
                shell.request_redraw();
            }
            (false, Some(_)) => {
                state.hover_since = None;
                shell.request_redraw();
            }
            _ => {}
        }
        if self.islands
            && let Event::Window(iced::window::Event::RedrawRequested(now)) = event
            && state.hover_since.is_some_and(|t| now.saturating_duration_since(t) < FADE)
        {
            shell.request_redraw();
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(p) = cursor.position_over(layout.bounds()) {
                    let now = Instant::now();
                    if state.last_press.is_some_and(|t| now.duration_since(t) < DOUBLE_CLICK) {
                        state.last_press = None;
                        state.dragging = None;
                        shell.publish(self.on_reset.clone());
                    } else {
                        state.last_press = Some(now);
                        state.dragging = Some(along(p));
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(last) = state.dragging {
                    let now = along(*position);
                    if now != last {
                        state.dragging = Some(now);
                        shell.publish((self.on_drag)(now - last));
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging.is_some() => {
                state.dragging = None;
                shell.publish(self.on_release.clone());
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let dragging = tree.state.downcast_ref::<State>().dragging.is_some();
        if dragging || cursor.is_over(layout.bounds()) {
            match self.axis {
                Axis::Vertical => mouse::Interaction::ResizingHorizontally,
                Axis::Horizontal => mouse::Interaction::ResizingVertically,
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let active = state.dragging.is_some() || cursor.is_over(b);
        let p = theme.extended_palette();
        if self.islands {
            // Entre ilhas o vão é a fronteira: só uma barrinha arredondada quando dá para arrastar, sem passar
            // dos cantos das ilhas.
            let alpha = self.reveal(state, Instant::now());
            if alpha <= 0.0 {
                return;
            }
            let (thick, end) = (3.0, tea_core::style::RADIUS_ISLAND);
            let bar = match self.axis {
                Axis::Vertical => Rectangle {
                    x: b.center_x() - thick / 2.0,
                    y: b.y + end,
                    width: thick,
                    height: (b.height - 2.0 * end).max(0.0),
                },
                Axis::Horizontal => Rectangle {
                    x: b.x + end,
                    y: b.center_y() - thick / 2.0,
                    width: (b.width - 2.0 * end).max(0.0),
                    height: thick,
                },
            };
            let quad = renderer::Quad {
                bounds: bar,
                border: iced::Border { radius: (thick / 2.0).into(), ..iced::Border::default() },
                ..renderer::Quad::default()
            };
            renderer.fill_quad(quad, p.primary.base.color.scale_alpha(alpha));
            return;
        }
        // Linha fina normalmente; mais grossa e colorida quando dá para arrastar.
        let (thick, color) = if active { (3.0, p.primary.base.color) } else { (1.0, p.background.strong.color) };
        let line = match self.axis {
            Axis::Vertical => Rectangle { x: b.center_x() - thick / 2.0, width: thick, ..b },
            Axis::Horizontal => Rectangle { y: b.center_y() - thick / 2.0, height: thick, ..b },
        };
        renderer.fill_quad(renderer::Quad { bounds: line, ..renderer::Quad::default() }, color);
    }
}

impl<'a, M: Clone + 'a, Renderer: renderer::Renderer + 'a> From<Divider<'a, M>> for Element<'a, M, Theme, Renderer> {
    fn from(d: Divider<'a, M>) -> Self {
        Element::new(d)
    }
}
