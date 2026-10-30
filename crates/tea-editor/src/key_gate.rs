//! Envolve um widget e, quando ativo, intercepta algumas teclas antes dele.
//!
//! Usado para o menu de autocomplete: com ele aberto, ↑/↓/Enter/Tab/Esc navegam
//! no menu em vez de mover o cursor do editor.
//!
//! Também mostra a mãozinha com Ctrl sobre o texto (Ctrl+clique vai para a definição): o widget
//! de edição só sublinha a palavra, sem trocar o ponteiro.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, overlay, renderer};
use iced::keyboard::{self, key::Named};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

pub struct KeyGate<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    /// `None` = deixa a tecla passar.
    intercept: Option<Box<dyn Fn(Named) -> Option<Message> + 'a>>,
}

impl<'a, Message, Theme, Renderer> KeyGate<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self { content: content.into(), intercept: None }
    }

    /// Liga a interceptação (só quando `active`).
    pub fn intercept(mut self, active: bool, f: impl Fn(Named) -> Option<Message> + 'a) -> Self {
        if active {
            self.intercept = Some(Box::new(f));
        }
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for KeyGate<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<keyboard::Modifiers>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(keyboard::Modifiers::default())
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

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Keyboard(
            keyboard::Event::ModifiersChanged(m)
            | keyboard::Event::KeyPressed { modifiers: m, .. }
            | keyboard::Event::KeyReleased { modifiers: m, .. },
        ) = event
        {
            let held = tree.state.downcast_mut::<keyboard::Modifiers>();
            if held.control() != m.control() {
                // O ponteiro muda sem o mouse se mexer: precisa redesenhar.
                shell.request_redraw();
            }
            *held = *m;
        }
        // Alt+←/→ são da IDE (voltar/avançar na navegação): o editor não os vê, então o evento
        // chega ao app sem estar "capturado".
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(Named::ArrowLeft | Named::ArrowRight),
            modifiers,
            ..
        }) = event
            && modifiers.alt()
            && !modifiers.control()
            && !modifiers.shift()
        {
            return;
        }
        if let (Some(f), Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(named), .. })) =
            (&self.intercept, event)
            && let Some(message) = f(*named)
        {
            shell.publish(message);
            shell.capture_event();
            return;
        }
        self.content
            .as_widget_mut()
            .update(&mut tree.children[0], event, layout, cursor, renderer, clipboard, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let inner = self.content.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer);
        let ctrl = tree.state.downcast_ref::<keyboard::Modifiers>().control();
        if ctrl && inner == mouse::Interaction::Text { mouse::Interaction::Pointer } else { inner }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<KeyGate<'a, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(gate: KeyGate<'a, Message, Theme, Renderer>) -> Self {
        Element::new(gate)
    }
}
