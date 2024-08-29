use std::marker::PhantomData;

use iced::advanced;
use iced::advanced::graphics::core::event::Status;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::mouse::Interaction;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::Tree;
use iced::advanced::Widget;

use iced::mouse;
use iced::widget::container;
use iced::Background;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Size;

/// State of a divider.
#[derive(Default, Clone, Copy)]
pub struct PorterDividerState {
    pub is_hovered: bool,
    pub drag_origin: Option<Point>,
}

/// A column header divider that supports resize operations.
pub struct PorterDivider<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet,
    Renderer: advanced::Renderer,
{
    width: Length,
    height: Length,
    style: <Theme as container::StyleSheet>::Style,
    on_drag: Box<dyn Fn(f32) -> Message>,
    on_release: Message,
    _phantom: PhantomData<&'a Renderer>,
}

impl<'a, Message, Theme, Renderer> PorterDivider<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: container::StyleSheet,
    Renderer: advanced::Renderer,
{
    /// Constructs a new instance of [`PorterDivider`].
    pub fn new(on_drag: impl Fn(f32) -> Message + 'static, on_release: Message) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            style: Default::default(),
            on_drag: Box::new(on_drag),
            on_release,
            _phantom: PhantomData,
        }
    }

    /// Sets the width of the [`PorterDivider`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`PorterDivider`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the style of the [`PorterDivider`].
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PorterDivider<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: container::StyleSheet,
    Renderer: advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn state(&self) -> State {
        State::new(PorterDividerState::default())
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: advanced::Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let state: &mut PorterDividerState = tree.state.downcast_mut();

        let bounds = layout.bounds();
        let bounds = Rectangle {
            x: bounds.x - 5.0,
            y: bounds.y,
            width: bounds.width + 10.0,
            height: bounds.height,
        };

        state.is_hovered = cursor.is_over(bounds);

        if let Event::Mouse(event) = event {
            match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(origin) = cursor.position_over(bounds) {
                        state.drag_origin = Some(origin);
                        return Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.drag_origin.take().is_some() {
                        shell.publish(self.on_release.clone());
                        return Status::Captured;
                    }
                }
                mouse::Event::CursorMoved { position } => {
                    if let Some(origin) = &mut state.drag_origin {
                        let shift = (position - *origin).x;

                        *origin = position;

                        shell.publish((self.on_drag)(shift));
                        return Status::Captured;
                    }
                }
                _ => {
                    // Nothing.
                }
            }
        }

        Status::Ignored
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let style = theme.appearance(&self.style);

        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds: layout.bounds(),
                border: style.border,
                shadow: style.shadow,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: advanced::Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> Interaction {
        let state: &PorterDividerState = tree.state.downcast_ref();

        if state.drag_origin.is_some() || state.is_hovered {
            Interaction::ResizingHorizontally
        } else {
            Interaction::Idle
        }
    }
}

impl<'a, Message, Theme, Renderer> From<PorterDivider<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'static,
    Theme: container::StyleSheet + 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(
        divider: PorterDivider<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(divider)
    }
}
