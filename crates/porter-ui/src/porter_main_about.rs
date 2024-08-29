use iced::alignment::*;
use iced::widget::*;

use iced::Alignment;
use iced::Color;
use iced::Element;
use iced::Length;

use crate::Message;
use crate::PorterLabelStyle;
use crate::PorterMain;
use crate::PORTER_COPYRIGHT;
use crate::PORTER_DISCLAIMER;

impl PorterMain {
    /// Constructs the about view.
    pub fn about(&self) -> Element<Message> {
        container(
            column([
                text(
                    "Thank you for using my tools, built for the community of modders and artists.",
                )
                .size(20.0)
                .style(Color::from_rgb8(0xD4, 0xAF, 0x37))
                .into(),
                vertical_space().height(20.0).into(),
                text("Please report all bugs or crashes to me on twitter @DTZxPorter.")
                    .size(18.0)
                    .style(PorterLabelStyle)
                    .into(),
                vertical_space().height(20.0).into(),
                text(PORTER_DISCLAIMER).style(PorterLabelStyle).into(),
                text(format!(
                    "\"{}\" {}.",
                    self.name.to_uppercase(),
                    PORTER_COPYRIGHT
                ))
                .style(PorterLabelStyle)
                .into(),
            ])
            .spacing(8.0)
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}
