use iced::alignment::*;
use iced::widget::*;

use iced::Alignment;
use iced::Color;
use iced::Element;
use iced::Length;

use crate::Message;
use crate::PorterLabelStyle;
use crate::PorterMain;

impl PorterMain {
    /// Constructs the about view.
    pub fn about(&self) -> Element<Message> {
        container(column(vec![
            text("Thank you for using my tools, built for the community of modders and artists.")
            .size(20.0)
            .style(Color::from_rgb8(0xD4, 0xAF, 0x37))
            .into(),
            vertical_space(20.0)
            .into(),
            text("Please report all bugs or crashes to me on twitter @DTZxPorter.")
            .size(18.0)
            .style(PorterLabelStyle)
            .into(),
            vertical_space(20.0)
            .into(),
            text("As with all my tools, this program comes with no warranty what so ever. Use at your own risk.")
            .style(PorterLabelStyle)
            .into(),
            text(format!("\"{}\" Copyright © 2023 DTZxPorter.", self.name ))
            .style(PorterLabelStyle)
            .into()
        ])
        .spacing(8.0)
        .align_items(Alignment::Center))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
