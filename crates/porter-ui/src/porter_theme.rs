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
use iced::theme::Text;
use iced::theme::TextInput;

use iced::widget::button;
use iced::widget::checkbox;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::progress_bar;
use iced::widget::scrollable;
use iced::widget::text_input;

use iced::overlay::menu;

use iced::Background;
use iced::BorderRadius;
use iced::Color;
use iced::Font;
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
            border_radius: BorderRadius::from(4.0),
            ..Default::default()
        }
    }
}

impl From<PorterOverlayBackgroundStyle> for Container {
    fn from(value: PorterOverlayBackgroundStyle) -> Self {
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
            border_radius: BorderRadius::from(4.0),
            border_width: 1.5,
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
            text_color: Color::WHITE,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 1.0),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        let hovered = self.hovered(style);

        button::Appearance {
            border_width: 1.0,
            ..hovered
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            text_color: Color::from_rgb8(0x2C, 0x2C, 0x2C),
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.3),
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

    fn active(&self, _: &Self::Style) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: Some(Background::Color(Color::from_rgb8(0x1C, 0x1C, 0x1C))),
            border_radius: BorderRadius::from(0.0),
            border_width: 1.0,
            border_color: Color::from_rgb8(0x1C, 0x1C, 0x1C),
            scroller: scrollable::Scroller {
                color: Color::from_rgb8(0x2C, 0x2C, 0x2C),
                border_radius: BorderRadius::from(2.0),
                border_width: 1.0,
                border_color: Color::from_rgb8(0x3C, 0x3C, 0x3C),
            },
        }
    }

    fn hovered(&self, style: &Self::Style, _: bool) -> scrollable::Scrollbar {
        self.active(style)
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
            border_radius: BorderRadius::from(4.0),
            border_width: 1.5,
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
            icon_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let active = self.active(style);

        text_input::Appearance {
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 1.0),
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
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.3),
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

/// The style for the title font.
pub struct PorterTitleFont;

impl From<PorterTitleFont> for Font {
    fn from(_: PorterTitleFont) -> Self {
        Self {
            family: Family::SansSerif,
            weight: iced::font::Weight::Bold,
            stretch: Stretch::Normal,
            style: Style::Normal,
            monospaced: false,
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
            border_radius: BorderRadius::from([4.0, 4.0, 0.0, 0.0]),
            border_width: 1.0,
            border_color: Color::from_rgb8(0x1F, 0x1F, 0x1F),
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
            border_radius: BorderRadius::from(0.0),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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
            border_radius: BorderRadius::from([4.0, 4.0, 0.0, 0.0]),
            border_width: 1.0,
            border_color: Color::from_rgb8(0x1F, 0x1F, 0x1F),
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
            border_radius: BorderRadius::from(4.0),
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
            border_radius: BorderRadius::from(4.0),
            border_width: 1.5,
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
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
            border_radius: BorderRadius::from(2.0),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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
            border_radius: BorderRadius::from(4.0),
            border_width: 1.0,
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.5),
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
            border_radius: BorderRadius::from(4.0),
            border_width: 1.0,
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 0.75),
        }
    }

    fn hovered(&self, style: &Self::Style) -> pick_list::Appearance {
        let active = self.active(style);

        pick_list::Appearance {
            border_color: Color::from_rgba8(0x27, 0x9B, 0xD4, 1.0),
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
            border_width: 1.0,
            border_radius: BorderRadius::from(4.0),
            border_color: Color::from_rgb8(0x27, 0x9B, 0xD4),
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
            border_radius: BorderRadius::from(4.0),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl From<PorterDividerStyle> for Container {
    fn from(value: PorterDividerStyle) -> Self {
        Self::Custom(Box::new(value))
    }
}

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
