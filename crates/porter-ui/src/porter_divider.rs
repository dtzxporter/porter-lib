use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::gradient::Linear;
use iced::widget::canvas::Fill;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::Program;
use iced::widget::container;
use iced::widget::Canvas;
use iced::Background;
use iced::Color;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Theme;

/// State of a divider.
#[derive(Default, Clone, Copy)]
pub struct PorterDividerState {
    pub is_hovered: bool,
    pub drag_origin: Option<Point>,
}

/// A column header divider that supports resize operations.
pub struct PorterDivider<Message> {
    width: Length,
    height: Length,
    style: container::Appearance,
    on_drag: Box<dyn Fn(f32) -> Message>,
    on_release: Message,
    cache: canvas::Cache,
}

impl<Message> PorterDivider<Message>
where
    Message: Clone,
{
    /// Constructs a new instance of [`PorterDivider`].
    pub fn new(on_drag: impl Fn(f32) -> Message + 'static, on_release: Message) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            style: Default::default(),
            on_drag: Box::new(on_drag),
            on_release,
            cache: canvas::Cache::new(),
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
    pub fn style(mut self, style: impl container::StyleSheet) -> Self {
        self.style = style.appearance(&Default::default());
        self
    }

    /// Builds the final divider.
    pub fn build(self) -> Canvas<Self, Message> {
        let width = self.width;
        let height = self.height;

        canvas(self).width(width).height(height)
    }
}

impl<Message> Program<Message, Renderer> for PorterDivider<Message>
where
    Message: Clone,
{
    type State = PorterDividerState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let bounds = Rectangle {
            x: bounds.x - 5.0,
            y: bounds.y,
            width: bounds.width + 10.0,
            height: bounds.height,
        };

        state.is_hovered = cursor.is_over(bounds);

        if let iced::widget::canvas::Event::Mouse(event) = event {
            match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(origin) = cursor.position_over(bounds) {
                        state.drag_origin = Some(origin);
                        return (canvas::event::Status::Captured, None);
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.drag_origin.take().is_some() {
                        return (
                            canvas::event::Status::Captured,
                            Some(self.on_release.clone()),
                        );
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor.position() {
                        if let Some(origin) = &mut state.drag_origin {
                            let shift = (position - *origin).x;

                            *origin = position;

                            return (canvas::event::Status::Captured, Some((self.on_drag)(shift)));
                        }
                    }
                }
                _ => {
                    // Nothing.
                }
            }
        }

        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let background = self
            .style
            .background
            .unwrap_or(Background::Color(Color::TRANSPARENT));

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                match background {
                    Background::Color(color) => Fill::from(color),
                    Background::Gradient(iced::Gradient::Linear(linear)) => {
                        let result = Linear::new(
                            Point::ORIGIN,
                            Point {
                                x: bounds.width,
                                y: bounds.height,
                            },
                        );

                        for stop in linear.stops.into_iter().flatten() {
                            result.add_stop(stop.offset, stop.color);
                        }

                        Fill::from(result)
                    }
                },
            )
        });

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        if _state.drag_origin.is_some() || _state.is_hovered {
            iced::advanced::mouse::Interaction::ResizingHorizontally
        } else {
            iced::advanced::mouse::Interaction::Idle
        }
    }
}
