use std::time::Duration;

use iced::executor;

use iced::widget::image::Handle;

use iced::window;
use iced::window::Level;
use iced::window::Position;

use iced::Application;
use iced::Command;
use iced::ContentFit;
use iced::Font;
use iced::Length;
use iced::Pixels;
use iced::Settings;
use iced::Size;
use iced::Theme;

use crate::DEBUG_SPLASH_IMAGE;

#[derive(Debug)]
enum Message {
    Exit,
}

struct PorterSplash {
    title: String,
    image: Handle,
}

impl Application for PorterSplash {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = (String, Handle);

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                title: flags.0,
                image: flags.1,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Exit => window::close(iced::window::Id::MAIN),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        struct Exit;

        iced::subscription::unfold(std::any::TypeId::of::<Exit>(), 0, |state| async move {
            std::thread::sleep(Duration::from_millis(2500));

            (Message::Exit, state)
        })
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        iced::widget::image(self.image.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Fill)
            .into()
    }
}

pub fn show_splash<T: Into<String>>(title: T, splash: Option<&'static [u8]>) {
    let splash = splash
        .map(Handle::from_memory)
        .unwrap_or_else(|| Handle::from_memory(DEBUG_SPLASH_IMAGE));

    let settings = Settings {
        id: None,
        window: iced::window::Settings {
            size: Size::new(620.0, 350.0),
            position: Position::Centered,
            min_size: Some(Size::new(620.0, 350.0)),
            max_size: Some(Size::new(620.0, 350.0)),
            decorations: false,
            level: Level::AlwaysOnTop,
            ..Default::default()
        },
        fonts: Vec::new(),
        flags: (title.into(), splash),
        default_font: Font::DEFAULT,
        default_text_size: Pixels(16.0),
        antialiasing: true,
    };

    PorterSplash::run(settings).unwrap();
}
