use std::rc::Rc;

use iced::font::Family;
use iced::font::Stretch;
use iced::font::Style;

use iced::theme::Button;
use iced::theme::Checkbox;
use iced::theme::Container;
use iced::theme::PickList;
use iced::theme::ProgressBar;
use iced::theme::Scrollable;
use iced::theme::Slider;
use iced::theme::Text;
use iced::theme::TextInput;

use iced::widget::button;
use iced::widget::checkbox;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::progress_bar;
use iced::widget::scrollable;
use iced::widget::slider;
use iced::widget::text_input;

use iced::overlay::menu;

use iced::Background;
use iced::Border;
use iced::Color;
use iced::Font;
use iced::Shadow;
use iced::Theme;

use crate::porter_spinner;
use crate::porter_spinner::StyleSheet;

/// The style of a row in the list view.
pub struct PorterRowStyle(usize, bool);

impl PorterRowStyle {
    pub fn new(index: usize, selected: bool) -> Self {
        Self(index, selected)
    }
}

impl container::StyleSheet for PorterRowStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        let color = if self.1 {
            Color::from_rgb8(0x27, 0x9B, 0xD4)
        } else if self.0 % 2 > 0 {
            Color::from_rgb8(0x17, 0x17, 0x17)
        } else {
            Color::from_rgb8(0x1C, 0x1C, 0x1C)
        };

        container::Appearance {
            text_color: None,
            background: Some(Background::Color(color)),
            ..Default::default()
        }
    }
}

impl From<PorterRowStyle> for Container {
    fn from(value: PorterRowStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for a standard background color.
pub struct PorterBackgroundStyle;

impl container::StyleSheet for PorterBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x11, 0x11, 0x11))),
            ..Default::default()
        }
    }
}

impl From<PorterBackgroundStyle> for Container {
    fn from(value: PorterBackgroundStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the overlay background.
pub struct PorterOverlayBackgroundStyle;

impl container::StyleSheet for PorterOverlayBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgba8(0x11, 0x11, 0x11, 0.75))),
            border: Border::with_radius(4.0),
            ..Default::default()
        }
    }
}

impl From<PorterOverlayBackgroundStyle> for Container {
    fn from(value: PorterOverlayBackgroundStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for debugging.
#[allow(unused)]
pub struct PorterDebugBackgroundStyle;

impl container::StyleSheet for PorterDebugBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgba8(0xFF, 0x0, 0x0, 1.0))),
            ..Default::default()
        }
    }
}

impl From<PorterDebugBackgroundStyle> for Container {
    fn from(value: PorterDebugBackgroundStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the bottom bar.
pub struct PorterHeaderBackgroundStyle;

impl container::StyleSheet for PorterHeaderBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C))),
            ..Default::default()
        }
    }
}

impl From<PorterHeaderBackgroundStyle> for Container {
    fn from(value: PorterHeaderBackgroundStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for all buttons.
pub struct PorterButtonStyle;

impl button::StyleSheet for PorterButtonStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: None,
            border: Border {
                width: 1.0,
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
                ..Border::with_radius(4.0)
            },
            shadow: Default::default(),
            text_color: Color::WHITE,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            border: Border {
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 1.0),
                ..active.border
            },
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            text_color: Color::from_rgb8(0x2C, 0x2C, 0x2C),
            border: Border {
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.3),
                ..active.border
            },
            ..active
        }
    }
}

impl From<PorterButtonStyle> for Button {
    fn from(value: PorterButtonStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style of the scrollable scrollbar.
pub struct PorterScrollStyle;

impl scrollable::StyleSheet for PorterScrollStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: Default::default(),
            scrollbar: scrollable::Scrollbar {
                background: Some(Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C))),
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb8(0x1C, 0x1C, 0x1C),
                    ..Border::with_radius(0.0)
                },
                scroller: scrollable::Scroller {
                    color: Color::from_rgb8(0x2C, 0x2C, 0x2C),
                    border: Border {
                        width: 1.0,
                        color: Color::from_rgb8(0x3C, 0x3C, 0x3C),
                        ..Border::with_radius(2.0)
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, style: &Self::Style, _: bool) -> scrollable::Appearance {
        self.active(style)
    }

    fn disabled(&self, style: &Self::Style) -> scrollable::Appearance {
        let active = self.active(style);

        scrollable::Appearance {
            scrollbar: scrollable::Scrollbar {
                scroller: scrollable::Scroller {
                    color: Color::from_rgb8(0x1C, 0x1C, 0x1C),
                    border: Border {
                        width: 1.0,
                        color: Color::from_rgb8(0x2C, 0x2C, 0x2C),
                        ..Border::with_radius(2.0)
                    },
                },
                ..active.scrollbar
            },
            ..active
        }
    }
}

impl From<PorterScrollStyle> for Scrollable {
    fn from(value: PorterScrollStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style of a text input field.
pub struct PorterTextInputStyle;

impl text_input::StyleSheet for PorterTextInputStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb8(0x11, 0x11, 0x11)),
            border: Border {
                width: 1.0,
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
                ..Border::with_radius(4.0)
            },
            icon_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let active = self.active(style);

        text_input::Appearance {
            border: Border {
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 1.0),
                ..active.border
            },
            ..active
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        Color::from_rgb8(0xC1, 0xC1, 0xC1)
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        Color::WHITE
    }

    fn disabled_color(&self, _: &Self::Style) -> Color {
        Color::from_rgb8(0x2C, 0x2C, 0x2C)
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        Color::from_rgb8(0x27, 0x9B, 0xD4)
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        let active = self.active(style);

        text_input::Appearance {
            border: Border {
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.3),
                ..active.border
            },
            ..active
        }
    }
}

impl From<PorterTextInputStyle> for TextInput {
    fn from(value: PorterTextInputStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for a label.
pub struct PorterLabelStyle;

impl From<PorterLabelStyle> for Text {
    fn from(_: PorterLabelStyle) -> Self {
        Self::Color(Color::from_rgb8(0xC1, 0xC1, 0xC1))
    }
}

/// The style for warning.
pub struct PorterLabelWarningStyle;

impl From<PorterLabelWarningStyle> for Text {
    fn from(_: PorterLabelWarningStyle) -> Self {
        Self::Color(Color::from_rgb8(0xD4, 0xAF, 0x37))
    }
}

/// The style for success.
pub struct PorterLabelSuccessStyle;

impl From<PorterLabelSuccessStyle> for Text {
    fn from(_: PorterLabelSuccessStyle) -> Self {
        Self::Color(Color::from_rgb8(35, 206, 107))
    }
}

/// The style for the title font.
pub struct PorterTitleFont;

impl From<PorterTitleFont> for Font {
    fn from(_: PorterTitleFont) -> Self {
        Self {
            family: Family::SansSerif,
            weight: iced::font::Weight::Bold,
            stretch: Stretch::Normal,
            style: Style::Normal,
        }
    }
}

/// The style for the preview container.
pub struct PorterPreviewStyle;

impl container::StyleSheet for PorterPreviewStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1F, 0x1F, 0x1F))),
            border: Border {
                width: 1.0,
                color: Color::from_rgb8(0x1F, 0x1F, 0x1F),
                ..Border::with_radius([4.0, 4.0, 0.0, 0.0])
            },
            shadow: Default::default(),
        }
    }
}

impl From<PorterPreviewStyle> for Container {
    fn from(value: PorterPreviewStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the preview close button.
pub struct PorterPreviewButtonStyle;

impl button::StyleSheet for PorterPreviewButtonStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: None,
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                ..Border::with_radius(0.0)
            },
            shadow: Default::default(),
            text_color: Color::from_rgb8(0xC1, 0xC1, 0xC1),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            text_color: Color::WHITE,
            ..active
        }
    }
}

impl From<PorterPreviewButtonStyle> for Button {
    fn from(value: PorterPreviewButtonStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the column headers.
pub struct PorterColumnHeader;

impl container::StyleSheet for PorterColumnHeader {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1F, 0x1F, 0x1F))),
            border: Border {
                width: 1.0,
                color: Color::from_rgb8(0x1F, 0x1F, 0x1F),
                ..Border::with_radius([4.0, 4.0, 0.0, 0.0])
            },
            shadow: Default::default(),
        }
    }
}

impl From<PorterColumnHeader> for Container {
    fn from(value: PorterColumnHeader) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// Style used for progress bars.
pub struct PorterProgressStyle;

impl progress_bar::StyleSheet for PorterProgressStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C)),
            bar: Background::Color(Color::from_rgb8(0x27, 0x9B, 0xD4)),
            border_radius: Border::with_radius(4.0).radius,
        }
    }
}

impl From<PorterProgressStyle> for ProgressBar {
    fn from(value: PorterProgressStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the background of a switch button.
pub struct PorterSwitchButtonBackgroundStyle;

impl container::StyleSheet for PorterSwitchButtonBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: None,
            border: Border {
                width: 1.0,
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
                ..Border::with_radius(4.0)
            },
            shadow: Default::default(),
        }
    }
}

impl From<PorterSwitchButtonBackgroundStyle> for Container {
    fn from(value: PorterSwitchButtonBackgroundStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// Style for the switch buttons.
pub struct PorterSwitchButtonStyle(pub bool);

impl button::StyleSheet for PorterSwitchButtonStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: if self.0 {
                Some(Background::Color(Color::from_rgb8(0x27, 0x9B, 0xD4)))
            } else {
                None
            },
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                ..Border::with_radius(2.0)
            },
            shadow: Default::default(),
            text_color: Color::WHITE,
        }
    }
}

impl From<PorterSwitchButtonStyle> for Button {
    fn from(value: PorterSwitchButtonStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for checkbox controls.
pub struct PorterCheckboxStyle;

impl checkbox::StyleSheet for PorterCheckboxStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style, _: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: Background::Color(Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75)),
            icon_color: Color::WHITE,
            border: Border {
                width: 1.0,
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.5),
                ..Border::with_radius(4.0)
            },
            text_color: Some(Color::WHITE),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let active = self.active(style, is_checked);

        checkbox::Appearance {
            background: Background::Color(Color::from_rgb8(0x27, 0x9B, 0xD4)),
            ..active
        }
    }

    fn disabled(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let active = self.active(style, is_checked);

        checkbox::Appearance {
            text_color: Some(Color::from_rgb8(0x2C, 0x2C, 0x2C)),
            border: Border {
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.3),
                ..active.border
            },
            ..active
        }
    }
}

impl From<PorterCheckboxStyle> for Checkbox {
    fn from(value: PorterCheckboxStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for a pick list control.
pub struct PorterPickListStyle;

impl pick_list::StyleSheet for PorterPickListStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: Color::WHITE,
            placeholder_color: Color::WHITE,
            handle_color: Color::from_rgb8(0x27, 0x9B, 0xD4),
            background: Background::Color(Color::from_rgb8(0x11, 0x11, 0x11)),
            border: Border {
                width: 1.0,
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
                ..Border::with_radius(4.0)
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> pick_list::Appearance {
        let active = self.active(style);

        pick_list::Appearance {
            border: Border {
                color: Color::from_rgba8(0x27, 0x9B, 0xD4, 1.0),
                ..active.border
            },
            ..active
        }
    }
}

impl menu::StyleSheet for PorterPickListStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> menu::Appearance {
        menu::Appearance {
            text_color: Color::from_rgb8(0xC1, 0xC1, 0xC1),
            background: Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C)),
            border: Border {
                width: 1.0,
                color: Color::from_rgb8(0x27, 0x9B, 0xD4),
                ..Border::with_radius(4.0)
            },
            selected_text_color: Color::WHITE,
            selected_background: Background::Color(Color::from_rgb8(0x27, 0x9B, 0xD4)),
        }
    }
}

impl From<PorterPickListStyle> for PickList {
    fn from(value: PorterPickListStyle) -> Self {
        let rc = Rc::new(value);

        Self::Custom(rc.clone(), rc)
    }
}

/// The theme for dividers.
pub struct PorterDividerStyle;

impl container::StyleSheet for PorterDividerStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x11, 0x11, 0x11))),
            border: Border {
                width: 2.0,
                color: Color::from_rgb8(0x11, 0x11, 0x11),
                ..Border::with_radius(4.0)
            },
            shadow: Shadow::default(),
        }
    }
}

impl From<PorterDividerStyle> for Container {
    fn from(value: PorterDividerStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the loading spinner.
pub struct PorterSpinnerStyle;

impl porter_spinner::StyleSheet for PorterSpinnerStyle {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> porter_spinner::Appearance {
        porter_spinner::Appearance {
            background: None,
            track_color: Color::from_rgb8(0x11, 0x11, 0x11),
            bar_color: Color::from_rgb8(0x27, 0x9B, 0xD4),
        }
    }
}

impl From<PorterSpinnerStyle> for porter_spinner::Appearance {
    fn from(value: PorterSpinnerStyle) -> Self {
        value.appearance(&())
    }
}

/// The style for the left hand side of the splash screen.
pub struct PorterSplashLeftStyle;

impl container::StyleSheet for PorterSplashLeftStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C))),
            ..Default::default()
        }
    }
}

impl From<PorterSplashLeftStyle> for Container {
    fn from(value: PorterSplashLeftStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for the background of the splash screen.
pub struct PorterSplashBackgroundStyle;

impl container::StyleSheet for PorterSplashBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            border: Border {
                color: Color::from_rgb8(0x27, 0x9B, 0xD4),
                width: 1.0,
                ..Default::default()
            },
            background: Some(Background::Color(Color::from_rgb8(0x11, 0x11, 0x11))),
            ..Default::default()
        }
    }
}

impl From<PorterSplashBackgroundStyle> for Container {
    fn from(value: PorterSplashBackgroundStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for a link.
pub struct PorterLinkStyle;

impl button::StyleSheet for PorterLinkStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: Color::from_rgb8(0x27, 0x9B, 0xD4),
            background: None,
            ..Default::default()
        }
    }

    fn hovered(&self, _: &Self::Style) -> button::Appearance {
        button::Appearance {
            text_color: Color::from_rgb8(0x3A, 0xB4, 0xE8),
            background: None,
            ..Default::default()
        }
    }
}

impl From<PorterLinkStyle> for Button {
    fn from(value: PorterLinkStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

/// The style for a slider.
pub struct PorterSliderStyle;

impl slider::StyleSheet for PorterSliderStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> slider::Appearance {
        let handle = slider::Handle {
            shape: slider::HandleShape::Rectangle {
                width: 8,
                border_radius: 4.0.into(),
            },
            color: Color::WHITE,
            border_color: Color::WHITE,
            border_width: 1.0,
        };

        slider::Appearance {
            rail: slider::Rail {
                colors: (Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75), Color::WHITE),
                width: 4.0,
                border_radius: 2.0.into(),
            },
            handle: slider::Handle {
                color: Color::from_rgb8(0x27, 0x9B, 0xD4),
                border_color: Color::from_rgb8(0x27, 0x9B, 0xD4),
                ..handle
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        let active = self.active(style);

        slider::Appearance {
            handle: slider::Handle {
                color: Color::from_rgb8(0x34, 0xA8, 0xE8),
                border_color: Color::from_rgb8(0x34, 0xA8, 0xE8),
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        self.hovered(style)
    }
}

impl From<PorterSliderStyle> for Slider {
    fn from(value: PorterSliderStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}
