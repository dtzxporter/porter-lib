use iced::widget::scrollable;

use iced::Rectangle;
use iced::Size;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

/// Wrapper around [`iced::widget::scrollable::Viewport`] to access bounds and content bounds.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct PorterViewport {
    offset_x: Offset,
    offset_y: Offset,
    pub bounds: Rectangle,
    pub content_bounds: Rectangle,
}

impl PorterViewport {
    pub fn zero() -> Self {
        Self {
            offset_x: Offset::Absolute(0.0),
            offset_y: Offset::Absolute(0.0),
            bounds: Rectangle::with_size(Size::ZERO),
            content_bounds: Rectangle::with_size(Size::ZERO),
        }
    }

    /// Converts from an iced viewport, incredibly unsafe, but the author hasn't merged my changes yet to expose these vars...
    pub fn from_viewport(viewport: scrollable::Viewport) -> Self {
        unsafe { std::mem::transmute(viewport) }
    }
}

impl std::ops::Deref for PorterViewport {
    type Target = scrollable::Viewport;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
