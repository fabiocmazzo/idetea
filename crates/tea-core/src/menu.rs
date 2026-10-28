//! Menu de contexto (botão direito).
//!
//! 1. Envolva o conteúdo em [`context_area`]: o clique direito vira uma mensagem com a posição.
//! 2. Ao receber essa mensagem, monte um [`Menu`] e chame `ctx.show_menu(menu)`.
//! 3. O app desenha o menu e devolve a mensagem do item escolhido (ou roda o comando dele).

use std::any::Any;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, overlay, renderer};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::ExtMessage;
use crate::icons::Icon;

/// O que um item faz.
#[derive(Debug, Clone)]
pub enum MenuAction {
    /// Mensagem para a extensão que abriu o menu (do tipo `Message` dela).
    Message(ExtMessage),
    /// Roda um comando (o menu mostra o atalho dele, se houver).
    Command(String),
}

#[derive(Debug, Clone)]
pub enum MenuItem {
    Action {
        label: String,
        icon: Option<Icon>,
        /// `None` = item desabilitado.
        action: Option<MenuAction>,
        danger: bool,
    },
    Separator,
}

#[derive(Debug, Clone)]
pub struct Menu {
    /// Posição na janela (canto de cima à esquerda do menu).
    pub position: Point,
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(position: Point) -> Self {
        Self { position, items: Vec::new() }
    }

    /// Item que manda `on_press` (do tipo `Message` da própria extensão) de volta.
    pub fn item<M: Any + Send + Sync>(self, icon: Option<Icon>, label: impl Into<String>, on_press: M) -> Self {
        self.push(icon, label, Some(MenuAction::Message(ExtMessage::new(on_press))), false)
    }

    /// Item que roda um comando.
    pub fn command(self, icon: Option<Icon>, label: impl Into<String>, command: impl Into<String>) -> Self {
        self.push(icon, label, Some(MenuAction::Command(command.into())), false)
    }

    /// Item destrutivo (vermelho), que manda `on_press`.
    pub fn danger<M: Any + Send + Sync>(self, icon: Option<Icon>, label: impl Into<String>, on_press: M) -> Self {
        self.push(icon, label, Some(MenuAction::Message(ExtMessage::new(on_press))), true)
    }

    /// Item visível mas desabilitado.
    pub fn disabled(self, icon: Option<Icon>, label: impl Into<String>) -> Self {
        self.push(icon, label, None, false)
    }

    pub fn separator(mut self) -> Self {
        // Sem separadores repetidos, no começo ou no fim.
        if matches!(self.items.last(), Some(MenuItem::Action { .. })) {
            self.items.push(MenuItem::Separator);
        }
        self
    }

    fn push(mut self, icon: Option<Icon>, label: impl Into<String>, action: Option<MenuAction>, danger: bool) -> Self {
        self.items.push(MenuItem::Action { label: label.into(), icon, action, danger });
        self
    }

    /// Itens que dá para escolher (para navegar com ↑/↓).
    pub fn enabled(&self, i: usize) -> bool {
        matches!(self.items.get(i), Some(MenuItem::Action { action: Some(_), .. }))
    }
}

// ---- Widget: área que reage ao botão direito -----------------------------------

/// Envolve `content`; um clique direito dentro dele manda `on_right_click(posição na janela)`.
/// Áreas aninhadas: vale a mais interna (ela captura o clique).
pub fn context_area<'a, M: 'a, Theme: 'a, Renderer: renderer::Renderer + 'a>(
    content: impl Into<Element<'a, M, Theme, Renderer>>,
    on_right_click: impl Fn(Point) -> M + 'a,
) -> Element<'a, M, Theme, Renderer> {
    Element::new(ContextArea { content: content.into(), on_right_click: Box::new(on_right_click) })
}

type OnRightClick<'a, M> = Box<dyn Fn(Point) -> M + 'a>;

struct ContextArea<'a, M, Theme, Renderer> {
    content: Element<'a, M, Theme, Renderer>,
    on_right_click: OnRightClick<'a, M>,
}

impl<M, Theme, Renderer> Widget<M, Theme, Renderer> for ContextArea<'_, M, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
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
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        // Filhos primeiro: uma área mais interna tem prioridade.
        self.content
            .as_widget_mut()
            .update(&mut tree.children[0], event, layout, cursor, renderer, clipboard, shell, viewport);
        if shell.is_event_captured() {
            return;
        }
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event
            && let Some(position) = cursor.position_over(layout.bounds())
        {
            shell.publish((self.on_right_click)(position));
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
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
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separators_are_tidy() {
        let m = Menu::new(Point::ORIGIN).separator().item(None, "a", 1u8).separator().separator().disabled(None, "b");
        assert_eq!(m.items.len(), 3, "sem separador no começo nem repetido");
        assert!(m.enabled(0));
        assert!(!m.enabled(1), "separador");
        assert!(!m.enabled(2), "desabilitado");
    }
}
