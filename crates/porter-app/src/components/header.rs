use iced::border::Radius;

use iced::widget::Image;
use iced::widget::container;
use iced::widget::image::Handle;
use iced::widget::space;
use iced::widget::stack;
use iced::widget::text;

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::ContentFit;
use iced::Element;
use iced::Length;
use iced::Task;
use iced::Theme;

use crate::AppState;
use crate::Icon;
use crate::Message;
use crate::fonts;
use crate::palette;
use crate::system;
use crate::widgets;

/// Header component handler.
pub struct Header {
    /// Whether or not to show the about view.
    pub show_about: bool,
    /// Whether or not to show the settings view.
    pub show_settings: bool,
    /// An optional icon to display.
    icon: Option<Handle>,
}

/// Messages produced by the header component.
#[derive(Debug, Clone)]
pub enum HeaderMessage {
    Donate,
    About,
    Settings,
    UpdateIcon(Option<Icon>),
}

impl Header {
    /// Creates a new header component.
    pub fn new() -> Self {
        Self {
            show_about: false,
            show_settings: false,
            icon: None,
        }
    }

    /// Handles updates for the header component.
    pub fn update(&mut self, state: &mut AppState, message: HeaderMessage) -> Task<Message> {
        use HeaderMessage::*;

        match message {
            Donate => self.on_donate(state),
            About => self.on_about(state),
            Settings => self.on_settings(state),
            UpdateIcon(icon) => self.on_update_icon(icon),
        }
    }

    /// Handles rendering the header component.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        let about_button = (
            "About",
            Some(Message::from(HeaderMessage::About)),
            self.show_about,
        );

        let settings_button = (
            "Settings",
            Some(Message::from(HeaderMessage::Settings)),
            self.show_settings,
        );

        let mut row: Vec<_> = Vec::with_capacity(8);

        row.extend([
            widgets::button("Donate")
                .on_press(Message::from(HeaderMessage::Donate))
                .into(),
            container(
                widgets::row([
                    text(state.name.to_uppercase())
                        .font(fonts::TITLE_FONT)
                        .size(32.0)
                        .into(),
                    text("by").size(12.0).into(),
                    text(state.author)
                        .color(palette::TEXT_COLOR_PORTER)
                        .size(14.0)
                        .into(),
                ])
                .width(Length::Shrink)
                .height(Length::Shrink)
                .spacing(4.0)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into(),
            widgets::switch_button([about_button, settings_button]).into(),
        ]);

        if let Some(handle) = &self.icon {
            row.extend([
                space().width(4.0).into(),
                stack([
                    widgets::laser()
                        .width(36.0)
                        .height(36.0)
                        .radius(8.0)
                        .padding(2.0)
                        .into(),
                    container(
                        Image::new(handle)
                            .width(32.0)
                            .height(32.0)
                            .content_fit(ContentFit::Fill),
                    )
                    .width(36.0)
                    .height(36.0)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into(),
                ])
                .into(),
            ]);
        }

        container(
            container(
                widgets::row(row)
                    .width(Length::Fill)
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding([4.0, 8.0])
            .style(header_background_style),
        )
        .padding([0.0, 8.0])
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
    }

    /// Opens the donation url.
    fn on_donate(&mut self, state: &AppState) -> Task<Message> {
        system::open_url(state.donate_url);

        Task::none()
    }

    /// Toggles the about view.
    fn on_about(&mut self, _: &AppState) -> Task<Message> {
        self.show_about = !self.show_about;
        self.show_settings = false;

        Task::done(Message::CheckReload)
    }

    /// Toggles the settings view.
    fn on_settings(&mut self, _: &AppState) -> Task<Message> {
        self.show_about = false;
        self.show_settings = !self.show_settings;

        if !self.show_settings {
            Task::done(Message::CheckReload)
        } else {
            Task::none()
        }
    }

    /// Updates the display icon.
    fn on_update_icon(&mut self, icon: Option<Icon>) -> Task<Message> {
        self.icon = icon.map(|icon| Handle::from(icon.rounded(12.0)));

        Task::none()
    }
}

/// Style for the header background.
fn header_background_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
        border: Border {
            width: 1.0,
            color: palette::BACKGROUND_COLOR_LIGHT_100,
            radius: Radius::new(0.0)
                .bottom_left(4.0)
                .bottom_right(4.0),
        },
        ..Default::default()
    }
}
