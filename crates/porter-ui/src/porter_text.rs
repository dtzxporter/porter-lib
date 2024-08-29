use std::borrow::Cow;
use std::marker::PhantomData;

use iced::advanced;
use iced::advanced::layout;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse::Cursor;
use iced::advanced::text::Paragraph;
use iced::advanced::widget::Tree;
use iced::advanced::Widget;

use iced::alignment;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::text;
use iced::widget::text::LineHeight;
use iced::widget::text::Shaping;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Size;

use unicode_segmentation::UnicodeSegmentation;

/// Used to render better text wrapping.
pub struct PorterText<'a, Message, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::text::Renderer,
{
    content: Cow<'a, str>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Font>,
    shaping: Shaping,
    style: <Theme as text::StyleSheet>::Style,
    _phantom: PhantomData<&'a (Message, Renderer)>,
}

impl<'a, Message, Theme, Renderer> PorterText<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: text::StyleSheet,
    Renderer: advanced::text::Renderer,
{
    /// Constructs a new instance of [`PorterText`].
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: content.into(),
            size: None,
            line_height: LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: Shaping::Basic,
            style: Default::default(),
            _phantom: PhantomData,
        }
    }

    /// Sets the size of the [`PorterText`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`PorterText`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the [`Font`] of the [`PorterText`].
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`PorterText`].
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the width of the [`PorterText`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`PorterText`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`alignment::Horizontal`] of the [`PorterText`].
    pub fn horizontal_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`PorterText`].
    pub fn vertical_alignment(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Sets the [`Shaping`] strategy of the [`PorterText`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PorterText<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: text::StyleSheet,
    Renderer: advanced::text::Renderer<Font = Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let width = bounds.width;

        let size = self.size.unwrap_or_else(|| renderer.default_size());
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let text = advanced::text::Text {
            content: &self.content,
            size,
            line_height: self.line_height,
            bounds: Size::INFINITY,
            font,
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Shaping::Basic,
        };

        let paragraph = <Renderer as advanced::text::Renderer>::Paragraph::with_text(text);
        let measure_full = paragraph.min_bounds().width;

        let render_str = if measure_full > width {
            let mut index = 0;

            while let Some(position) = paragraph.grapheme_position(0, index) {
                if position.x + size.0 > width {
                    break;
                }
                index += 1;
            }

            if index > 1 {
                index -= 1;
            }

            self.content
                .graphemes(true)
                .take(index)
                .chain(["…"])
                .collect::<Vec<_>>()
                .join("")
        } else {
            self.content.to_string()
        };

        let x = match self.horizontal_alignment {
            Horizontal::Left => bounds.x,
            Horizontal::Center => bounds.center_x(),
            Horizontal::Right => bounds.x + bounds.width,
        };

        let y = match self.vertical_alignment {
            Vertical::Top => bounds.y,
            Vertical::Center => bounds.center_y(),
            Vertical::Bottom => bounds.y + bounds.height,
        };

        if width > 0.0 {
            let text = iced::advanced::Text {
                content: &render_str,
                size,
                bounds: bounds.size(),
                line_height: self.line_height,
                font,
                horizontal_alignment: self.horizontal_alignment,
                vertical_alignment: self.vertical_alignment,
                shaping: self.shaping,
            };

            let color = theme
                .appearance(self.style.clone())
                .color
                .unwrap_or(style.text_color);

            renderer.fill_text(text, Point { x, y }, color, *viewport);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<PorterText<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'static,
    Theme: text::StyleSheet + 'a,
    Renderer: advanced::text::Renderer<Font = iced::Font> + 'a,
{
    fn from(
        text: PorterText<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}
