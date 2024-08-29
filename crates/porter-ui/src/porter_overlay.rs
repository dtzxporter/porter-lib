use iced::advanced::*;
use iced::Renderer;
use iced::Vector;
use iced::{event, mouse, Event, Length, Point, Rectangle};
use iced::{Element, Size};

/// Overlays the second element ontop of the first.
pub fn porter_overlay<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    overlay: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    PorterOverlay {
        base: base.into(),
        overlay: overlay.into(),
    }
    .into()
}

struct PorterOverlay<'a, Message, Theme> {
    base: Element<'a, Message, Theme>,
    overlay: Element<'a, Message, Theme>,
}

impl<'a, Message, Theme> Widget<Message, Theme, Renderer> for PorterOverlay<'a, Message, Theme> {
    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.base),
            widget::Tree::new(&self.overlay),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.base, &self.overlay])
    }

    fn operate(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.base
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.base.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        Some(overlay::Element::new(Box::new(PorterOverlayChild {
            position: layout.position() + translation,
            content: &mut self.overlay,
            tree: &mut tree.children[1],
            size: layout.bounds().size(),
        })))
    }
}

impl<'a, Message, Theme: 'a> From<PorterOverlay<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: 'a,
{
    fn from(double_pass: PorterOverlay<'a, Message, Theme>) -> Self {
        Element::new(double_pass)
    }
}

struct PorterOverlayChild<'a, 'b, Message, Theme> {
    position: Point,
    content: &'b mut Element<'a, Message, Theme>,
    tree: &'b mut widget::Tree,
    size: Size,
}

impl<'a, 'b, Message, Theme> overlay::Overlay<Message, Theme, Renderer>
    for PorterOverlayChild<'a, 'b, Message, Theme>
{
    fn layout(&mut self, renderer: &Renderer, _bounds: iced::Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, self.size)
            .width(Length::Fill)
            .height(Length::Fill);

        let child = self
            .content
            .as_widget()
            .layout(self.tree, renderer, &limits);

        let node = layout::Node::with_children(self.size, vec![child]);

        node.move_to(self.position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            &layout.bounds(),
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.content.as_widget_mut().operate(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            self.tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            Vector::ZERO,
        )
    }

    fn is_over(&self, _layout: Layout<'_>, _renderer: &Renderer, _cursor_position: Point) -> bool {
        false
    }
}
