use iced::Color;

/// A column in the list view of the main window.
pub struct PorterMainColumn {
    pub(crate) header: String,
    pub(crate) width: f32,
    pub(crate) color: Option<Color>,
}

impl PorterMainColumn {
    /// Constructs a new porter main column.
    pub fn new<H: Into<String>>(header: H, width: usize, color: Option<Color>) -> Self {
        Self {
            header: header.into(),
            width: width as f32,
            color,
        }
    }
}
