#![allow(unused)]
use std::ops::Sub;

use iced::advanced::text::Paragraph;
use iced::advanced::text::Renderer as TextRenderer;
use iced::alignment;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::Program;
use iced::widget::canvas::Text;
use iced::widget::text;
use iced::widget::text::LineHeight;
use iced::widget::text::Shaping;
use iced::widget::Canvas;
use iced::Color;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Theme;

use unicode_segmentation::UnicodeSegmentation;

/// Used to render better text wrapping.
#[derive(Debug)]
pub struct PorterText {
    content: String,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Font>,
    shaping: Shaping,
    style: text::Appearance,
    cache: canvas::Cache,
}

impl PorterText {
    /// Constructs a new instance of [`PorterText`].
    pub fn new<T: Into<String>>(content: T) -> Self {
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
            cache: canvas::Cache::new(),
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
    pub fn color(mut self, color: Color) -> Self {
        self.style = text::Appearance { color: Some(color) };
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

    /// Builds the final text element.
    pub fn build<Message>(self) -> Canvas<PorterText, Message> {
        let width = self.width;
        let height = self.height;

        canvas(self).width(width).height(height)
    }
}

impl<Message> Program<Message, Renderer> for PorterText {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let width = bounds.width;

            let size = self.size.unwrap_or_else(|| renderer.default_size());
            let font = self.font.unwrap_or_else(|| renderer.default_font());

            let text = iced::advanced::text::Text {
                content: &self.content,
                size,
                line_height: self.line_height,
                bounds: iced::Size::INFINITY,
                font,
                horizontal_alignment: Horizontal::Left,
                vertical_alignment: Vertical::Top,
                shaping: Shaping::Basic,
            };

            let paragraph =
                <Renderer as iced::advanced::text::Renderer>::Paragraph::with_text(text);
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
                self.content.clone()
            };

            let x = match self.horizontal_alignment {
                alignment::Horizontal::Left => 0.0,
                alignment::Horizontal::Center => bounds.center_x() - bounds.x,
                alignment::Horizontal::Right => bounds.width,
            };

            let y = match self.vertical_alignment {
                alignment::Vertical::Top => 0.0,
                alignment::Vertical::Center => bounds.center_y() - bounds.y,
                alignment::Vertical::Bottom => bounds.height,
            };

            let text = Text {
                content: render_str,
                size,
                line_height: self.line_height,
                position: Point { x, y },
                color: self.style.color.unwrap_or(Color::BLACK),
                font,
                horizontal_alignment: self.horizontal_alignment,
                vertical_alignment: self.vertical_alignment,
                shaping: self.shaping,
            };

            frame.fill_text(text);
        });

        vec![geometry]
    }
}
