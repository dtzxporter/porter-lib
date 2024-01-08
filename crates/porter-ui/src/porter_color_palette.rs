use iced::Color;

/// Shared color palette values.
pub struct PorterColorPalette;

impl PorterColorPalette {
    /// Image asset color.
    pub fn asset_type_image() -> Color {
        Color::from_rgb8(202, 97, 195)
    }

    /// Model asset color.
    pub fn asset_type_model() -> Color {
        Color::from_rgb8(0, 157, 220)
    }

    /// Material asset color.
    pub fn asset_type_material() -> Color {
        Color::from_rgb8(27, 153, 139)
    }

    /// Animation asset color.
    pub fn asset_type_animation() -> Color {
        Color::from_rgb8(219, 80, 74)
    }

    /// Sound asset color.
    pub fn asset_type_sound() -> Color {
        Color::from_rgb8(216, 30, 91)
    }

    /// Raw file asset color.
    pub fn asset_type_raw_file() -> Color {
        Color::from_rgb8(255, 255, 0)
    }

    /// Info text color.
    pub fn asset_info() -> Color {
        Color::from_rgb8(0xC1, 0xC1, 0xC1)
    }

    /// Default text color.
    pub fn default_color() -> Color {
        Color::WHITE
    }
}
