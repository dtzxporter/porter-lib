mod binary;
mod context;
mod header_divider;
mod laser;
mod resizable;
mod spinner;
mod viewport;
mod waveform;

pub use viewport::ViewportAction;
pub use viewport::ViewportState;

use std::borrow::Borrow;
use std::ops::RangeInclusive;
use std::time::Duration;

use iced::border::Radius;
use iced::border::rounded;

use iced::widget;
use iced::widget::Button;
use iced::widget::Checkbox;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::PickList;
use iced::widget::ProgressBar;
use iced::widget::Row;
use iced::widget::Rule;
use iced::widget::Scrollable;
use iced::widget::Slider;
use iced::widget::TextInput;
use iced::widget::Tooltip;
use iced::widget::container;
use iced::widget::text;

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Rectangle;

use crate::fonts;
use crate::palette;

/// Styled button widget.
pub fn button<'a, Message>(content: impl Into<Element<'a, Message>>) -> Button<'a, Message> {
    use widget::button;

    Button::new(content).style(|_, status| {
        let active = button::Style {
            background: Some(Background::Color(
                palette::BACKGROUND_COLOR_SEMI_TRANSPARENT,
            )),
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            text_color: palette::TEXT_COLOR_DEFAULT,
            ..Default::default()
        };

        match status {
            button::Status::Active => active,
            button::Status::Hovered => button::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_LIGHT_250,
                    ..active.border
                },
                ..active
            },
            button::Status::Pressed => button::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_DARK_250,
                    ..active.border
                },
                ..active
            },
            button::Status::Disabled => button::Style {
                background: None,
                border: Border {
                    color: palette::PRIMARY_COLOR.scale_alpha(0.3),
                    ..active.border
                },
                text_color: palette::TEXT_COLOR_DISABLED,
                ..active
            },
        }
    })
}

/// Styled settings button widget.
pub fn settings_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    selected: bool,
) -> Button<'a, Message>
where
    Message: 'a,
{
    use widget::button;

    Button::new(
        container(content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(48.0),
    )
    .style(move |_, status| {
        let active = if selected {
            button::Style {
                background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_100)),
                border: Border {
                    width: 1.0,
                    color: palette::PRIMARY_COLOR,
                    ..rounded(4.0)
                },
                text_color: palette::TEXT_COLOR_DEFAULT,
                ..Default::default()
            }
        } else {
            button::Style {
                background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                border: Border {
                    width: 1.0,
                    color: palette::BACKGROUND_COLOR_LIGHT_050,
                    ..rounded(4.0)
                },
                text_color: palette::TEXT_COLOR_DEFAULT,
                ..Default::default()
            }
        };

        match status {
            button::Status::Active => active,
            button::Status::Hovered => button::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_LIGHT_250,
                    ..active.border
                },
                ..active
            },
            button::Status::Pressed => button::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_DARK_250,
                    ..active.border
                },
                ..active
            },
            button::Status::Disabled => active,
        }
    })
    .padding(0.0)
    .width(Length::Fill)
    .height(Length::Shrink)
}

/// Styled icon button.
pub fn icon_button<'a, Message>(content: impl Into<Element<'a, Message>>) -> Button<'a, Message> {
    use widget::button;

    Button::new(content)
        .padding(0.0)
        .style(|_, status| {
            let active = button::Style {
                text_color: palette::TEXT_COLOR_MUTED,
                ..Default::default()
            };

            match status {
                button::Status::Active => active,
                button::Status::Hovered | button::Status::Pressed => button::Style {
                    text_color: palette::TEXT_COLOR_DEFAULT,
                    ..active
                },
                _ => active,
            }
        })
}

/// Styled horizontal rule widget.
pub fn horizontal_rule<'a>() -> Rule<'a> {
    use widget::rule;

    rule::horizontal(1.0).style(|theme| rule::Style {
        color: palette::BACKGROUND_COLOR_LIGHT_150,
        radius: Radius::new(0.0),
        fill_mode: rule::FillMode::Full,
        ..rule::default(theme)
    })
}

/// Styled checkbox widget.
pub fn checkbox<'a, Message>(
    label: impl text::IntoFragment<'a>,
    is_checked: bool,
) -> Checkbox<'a, Message> {
    use widget::checkbox;

    Checkbox::new(is_checked)
        .label(label)
        .size(20.0)
        .style(|_, status| {
            let active = checkbox::Style {
                background: Background::Color(palette::PRIMARY_COLOR),
                icon_color: palette::TEXT_COLOR_DEFAULT,
                border: rounded(4.0),
                text_color: Some(palette::TEXT_COLOR_DEFAULT),
            };

            match status {
                checkbox::Status::Active { .. } => active,
                checkbox::Status::Hovered { .. } => checkbox::Style {
                    background: Background::Color(palette::PRIMARY_COLOR_LIGHT_250),
                    ..active
                },
                checkbox::Status::Disabled { .. } => checkbox::Style {
                    background: Background::Color(palette::PRIMARY_COLOR_DARK_250),
                    text_color: Some(palette::TEXT_COLOR_DISABLED),
                    ..active
                },
            }
        })
}

/// Styled tooltip widget.
#[allow(dead_code)]
pub fn tooltip<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    tooltip: impl Into<Element<'a, Message>>,
    position: widget::tooltip::Position,
) -> Tooltip<'a, Message> {
    use widget::container;

    Tooltip::new(content, tooltip, position)
        .gap(8.0)
        .snap_within_viewport(true)
        .style(|_| container::Style {
            text_color: Some(palette::TEXT_COLOR_DEFAULT),
            background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_100)),
            border: Border {
                width: 1.0,
                color: palette::BACKGROUND_COLOR_LIGHT_150,
                ..rounded(4.0)
            },
            shadow: Default::default(),
            ..Default::default()
        })
}

/// Styled pick list widget.
pub fn pick_list<'a, T, L, V, Message>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
{
    use widget::overlay::menu;
    use widget::pick_list;

    PickList::new(selected, options, |x| x.to_string())
        .on_select(on_select)
        .style(|_, status| {
            let active = pick_list::Style {
                text_color: palette::TEXT_COLOR_DEFAULT,
                placeholder_color: palette::TEXT_COLOR_SECONDARY,
                handle_color: palette::PRIMARY_COLOR,
                background: Background::Color(palette::BACKGROUND_COLOR_SEMI_TRANSPARENT),
                border: Border {
                    width: 1.0,
                    color: palette::PRIMARY_COLOR,
                    ..rounded(4.0)
                },
            };

            match status {
                pick_list::Status::Active => active,
                pick_list::Status::Disabled => active,
                pick_list::Status::Hovered => pick_list::Style {
                    handle_color: palette::PRIMARY_COLOR_LIGHT_250,
                    border: Border {
                        color: palette::PRIMARY_COLOR_LIGHT_250,
                        ..active.border
                    },
                    ..active
                },
                pick_list::Status::Opened { .. } => pick_list::Style {
                    handle_color: palette::PRIMARY_COLOR_DARK_250,
                    border: Border {
                        color: palette::PRIMARY_COLOR_DARK_250,
                        ..active.border
                    },
                    ..active
                },
            }
        })
        .menu_style(|theme| menu::Style {
            background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            text_color: palette::TEXT_COLOR_SECONDARY,
            selected_text_color: palette::TEXT_COLOR_DEFAULT,
            selected_background: Background::Color(palette::PRIMARY_COLOR),
            ..menu::default(theme)
        })
}

/// Styled slider widget.
pub fn slider<'a, T, Message, F>(
    range: RangeInclusive<T>,
    value: T,
    on_change: F,
) -> Slider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
    Message: Clone,
    F: Fn(T) -> Message + 'a,
{
    use widget::slider;

    Slider::new(range, value, on_change)
        .height(22.0)
        .style(|_, status| {
            let active_handle = slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 8,
                    border_radius: Radius::new(4.0),
                },
                background: Background::Color(palette::PRIMARY_COLOR),
                border_color: palette::PRIMARY_COLOR,
                border_width: 1.0,
            };

            let active = slider::Style {
                rail: slider::Rail {
                    backgrounds: (
                        Background::Color(palette::PRIMARY_COLOR),
                        Background::Color(Color::WHITE),
                    ),
                    width: 4.0,
                    border: rounded(2.0),
                },
                handle: active_handle,
            };

            match status {
                slider::Status::Active => active,
                slider::Status::Hovered => slider::Style {
                    handle: slider::Handle {
                        background: Background::Color(palette::PRIMARY_COLOR_LIGHT_250),
                        border_color: palette::PRIMARY_COLOR_LIGHT_250,
                        ..active.handle
                    },
                    ..active
                },
                slider::Status::Dragged => slider::Style {
                    handle: slider::Handle {
                        background: Background::Color(palette::PRIMARY_COLOR_DARK_250),
                        border_color: palette::PRIMARY_COLOR_DARK_250,
                        ..active.handle
                    },
                    ..active
                },
            }
        })
}

/// Styled list item widget.
pub fn list_item<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    item_index: usize,
    selected: bool,
) -> Button<'a, Message> {
    use widget::button;

    let background_default = if selected {
        palette::PRIMARY_COLOR
    } else if item_index.is_multiple_of(2) {
        palette::BACKGROUND_COLOR_LIGHT_050
    } else {
        palette::BACKGROUND_COLOR_LIGHT_025
    };

    Button::new(content)
        .style(move |_, status| {
            let active = button::Style {
                background: Some(Background::Color(background_default)),
                border: Border {
                    width: 1.0,
                    color: background_default,
                    ..rounded(4.0)
                },
                text_color: palette::TEXT_COLOR_DEFAULT,
                ..Default::default()
            };

            match status {
                button::Status::Active => active,
                button::Status::Hovered | button::Status::Pressed => {
                    if selected {
                        active
                    } else {
                        button::Style {
                            background: Some(Background::Color(
                                palette::PRIMARY_COLOR_DARK_250.scale_alpha(0.5),
                            )),
                            border: Border {
                                color: palette::PRIMARY_COLOR_LIGHT_250.scale_alpha(0.75),
                                ..active.border
                            },
                            ..active
                        }
                    }
                }
                _ => active,
            }
        })
        .padding(0.0)
}

/// Styled text input widget.
pub fn text_input<'a, Message>(placeholder: &str, value: &str) -> TextInput<'a, Message>
where
    Message: Clone + 'a,
{
    use widget::text_input;

    TextInput::new(placeholder, value).style(|_, status| {
        let active = text_input::Style {
            background: Background::Color(palette::BACKGROUND_COLOR_SEMI_TRANSPARENT),
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            icon: palette::TEXT_COLOR_DEFAULT,
            placeholder: palette::TEXT_COLOR_SECONDARY,
            value: palette::TEXT_COLOR_DEFAULT,
            selection: palette::PRIMARY_COLOR,
        };

        match status {
            text_input::Status::Active => active,
            text_input::Status::Hovered | text_input::Status::Focused { .. } => text_input::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR_LIGHT_250,
                    ..active.border
                },
                ..active
            },
            text_input::Status::Disabled => text_input::Style {
                border: Border {
                    color: palette::PRIMARY_COLOR.scale_alpha(0.3),
                    ..active.border
                },
                icon: palette::TEXT_COLOR_DISABLED,
                placeholder: palette::TEXT_COLOR_DISABLED,
                value: palette::TEXT_COLOR_DISABLED,
                ..active
            },
        }
    })
}

/// Styled options as a switch button.
pub fn switch_button<'a, Message>(
    options: impl IntoIterator<Item = (&'a str, Option<Message>, bool)>,
) -> Container<'a, Message>
where
    Message: Clone + 'a,
{
    use widget::button;
    use widget::container;
    use widget::space;

    let mut options: Vec<_> = options
        .into_iter()
        .map(|(text, on_press, selected)| {
            Element::from(
                button(text)
                    .on_press_maybe(on_press)
                    .style(move |_, status| {
                        let active = button::Style {
                            background: if selected {
                                Some(Background::Color(palette::PRIMARY_COLOR))
                            } else {
                                Some(Background::Color(
                                    palette::BACKGROUND_COLOR_SEMI_TRANSPARENT,
                                ))
                            },
                            border: Border {
                                width: 1.0,
                                color: Color::TRANSPARENT,
                                ..rounded(2.0)
                            },
                            text_color: palette::TEXT_COLOR_DEFAULT,
                            ..Default::default()
                        };

                        match status {
                            button::Status::Active => active,
                            button::Status::Hovered | button::Status::Pressed => button::Style {
                                background: if selected {
                                    Some(Background::Color(palette::PRIMARY_COLOR))
                                } else {
                                    Some(Background::Color(palette::PRIMARY_COLOR.scale_alpha(0.1)))
                                },
                                ..active
                            },
                            _ => active,
                        }
                    }),
            )
        })
        .collect();

    for i in (0..options.len().max(1) - 1).rev() {
        options.insert(
            i + 1,
            Element::from(
                container(
                    space()
                        // Ideally, this should be padded to the height of the parent container.
                        .width(1.0)
                        .height(20.0),
                )
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::WHITE.scale_alpha(0.1))),
                    ..Default::default()
                }),
            ),
        );
    }

    container(row(options).align_y(Alignment::Center))
        .style(|_| container::Style {
            background: None,
            border: Border {
                width: 1.0,
                color: palette::PRIMARY_COLOR,
                ..rounded(4.0)
            },
            ..Default::default()
        })
        .padding(2.0)
        .width(Length::Shrink)
        .height(Length::Shrink)
}

/// Styled button as a link widget.
pub fn link<'a, Message>(content: impl Into<Element<'a, Message>>) -> Button<'a, Message> {
    use widget::button;

    Button::new(content)
        .padding(0)
        .style(|_, status| match status {
            button::Status::Active | button::Status::Disabled => button::Style {
                text_color: palette::TEXT_COLOR_LINK,
                background: None,
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                text_color: palette::TEXT_COLOR_LINK_HOVER,
                background: None,
                ..Default::default()
            },
        })
}

/// Styled bytton as a tab widget.
pub fn tab<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    selected: bool,
) -> Button<'a, Message> {
    use widget::button;

    Button::new(content)
        .padding([4.0, 8.0])
        .style(move |_, status| match status {
            button::Status::Active | button::Status::Disabled => {
                if selected {
                    button::Style {
                        text_color: palette::TEXT_COLOR_DEFAULT,
                        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                        border: Border {
                            width: 1.0,
                            color: palette::BACKGROUND_COLOR_LIGHT_050,
                            radius: Radius::new(0.0)
                                .bottom_left(8.0)
                                .bottom_right(8.0),
                        },
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        text_color: palette::TEXT_COLOR_MUTED,
                        background: None,
                        ..Default::default()
                    }
                }
            }
            button::Status::Hovered | button::Status::Pressed => {
                if selected {
                    button::Style {
                        text_color: palette::TEXT_COLOR_DEFAULT,
                        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                        border: Border {
                            width: 1.0,
                            color: palette::BACKGROUND_COLOR_LIGHT_050,
                            radius: Radius::new(0.0)
                                .bottom_left(8.0)
                                .bottom_right(8.0),
                        },
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        text_color: palette::TEXT_COLOR_SECONDARY,
                        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                        border: Border {
                            width: 1.0,
                            color: palette::BACKGROUND_COLOR_DEFAULT,
                            radius: Radius::new(0.0)
                                .bottom_left(8.0)
                                .bottom_right(8.0),
                        },
                        ..Default::default()
                    }
                }
            }
        })
}

/// Styled scrollable widget.
pub fn scrollable<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    use widget::scrollable;

    Scrollable::new(content)
        .auto_scroll(false)
        .style(|_, status| {
            let active_rail = scrollable::Rail {
                background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
                border: Border {
                    width: 1.0,
                    color: palette::BACKGROUND_COLOR_LIGHT_050,
                    ..rounded(0.0)
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_100),
                    border: Border {
                        width: 1.0,
                        color: palette::BACKGROUND_COLOR_LIGHT_150,
                        ..rounded(2.0)
                    },
                },
            };

            let disabled_rail = scrollable::Rail {
                border: Border {
                    width: 1.0,
                    color: palette::BACKGROUND_COLOR_LIGHT_100,
                    ..rounded(2.0)
                },
                ..active_rail
            };

            let auto_scroll = scrollable::AutoScroll {
                background: Background::Color(Color::TRANSPARENT),
                border: Default::default(),
                icon: Color::WHITE,
                shadow: Default::default(),
            };

            match status {
                scrollable::Status::Active {
                    is_horizontal_scrollbar_disabled,
                    is_vertical_scrollbar_disabled,
                } => scrollable::Style {
                    container: Default::default(),
                    vertical_rail: if is_vertical_scrollbar_disabled {
                        disabled_rail
                    } else {
                        active_rail
                    },
                    horizontal_rail: if is_horizontal_scrollbar_disabled {
                        disabled_rail
                    } else {
                        active_rail
                    },
                    gap: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_025)),
                    auto_scroll,
                },
                scrollable::Status::Hovered {
                    is_horizontal_scrollbar_disabled,
                    is_vertical_scrollbar_disabled,
                    ..
                } => scrollable::Style {
                    container: Default::default(),
                    vertical_rail: if is_vertical_scrollbar_disabled {
                        disabled_rail
                    } else {
                        active_rail
                    },
                    horizontal_rail: if is_horizontal_scrollbar_disabled {
                        disabled_rail
                    } else {
                        active_rail
                    },
                    gap: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_025)),
                    auto_scroll,
                },
                scrollable::Status::Dragged {
                    is_horizontal_scrollbar_disabled,
                    is_vertical_scrollbar_disabled,
                    ..
                } => scrollable::Style {
                    container: Default::default(),
                    vertical_rail: if is_vertical_scrollbar_disabled {
                        disabled_rail
                    } else {
                        active_rail
                    },
                    horizontal_rail: if is_horizontal_scrollbar_disabled {
                        disabled_rail
                    } else {
                        active_rail
                    },
                    gap: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_025)),
                    auto_scroll,
                },
            }
        })
}

/// Styled progress bar widget.
pub fn progress_bar<'a>(range: RangeInclusive<f32>, value: f32) -> ProgressBar<'a> {
    use widget::progress_bar;

    ProgressBar::new(range, value).style(|_| progress_bar::Style {
        background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
        bar: Background::Color(palette::PRIMARY_COLOR),
        border: rounded(4.0),
    })
}

/// Spinner progress indicator widget.
pub fn spinner<'a>() -> spinner::Spinner<'a> {
    spinner::Spinner::new()
        .size(48.0)
        .bar_height(4.0)
        .style(spinner::Style {
            track_color: palette::BACKGROUND_COLOR_DEFAULT,
            bar_color: palette::PRIMARY_COLOR,
        })
        .cycle_duration(Duration::from_secs(2))
        .rotation_duration(Duration::from_secs(2))
}

/// Context menu widget.
pub fn context<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> context::Context<'a, Message, Theme, Renderer>
where
    Message: Clone,
{
    context::Context::new(content)
}

/// Menu item widget.
pub fn menu_item<N: Into<String>, Message>(name: N) -> context::MenuItem<Message>
where
    Message: Clone,
{
    context::MenuItem::new(name.into())
}

/// Menu item divider widget.
pub fn menu_item_divider<Message>() -> context::MenuItem<Message>
where
    Message: Clone,
{
    context::MenuItem::divider()
}

/// Preview viewport widget.
pub fn viewport<'a, Message, Theme, Renderer, A>(
    state: &'a ViewportState,
    on_action: A,
) -> viewport::Viewport<'a, Message, Theme, Renderer, A>
where
    Message: Clone,
    Renderer: iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>,
    A: Fn(ViewportAction) -> Message + 'a,
{
    viewport::Viewport::new(state, on_action)
}

/// Binary hex viewer widget.
pub fn binary<Message, Theme, Renderer>(
    buffer: &[u8],
) -> binary::Binary<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
{
    binary::Binary::new(buffer)
        .font(fonts::BINARY_FONT)
        .style(binary::Style {
            background: Background::Color(palette::BACKGROUND_COLOR_LIGHT_050),
            hex_color: palette::TEXT_COLOR_DEFAULT,
            text_color: palette::TEXT_COLOR_DEFAULT,
            offset_color: palette::TEXT_COLOR_INFO,
            header_color: palette::TEXT_COLOR_INFO,
        })
}

/// Laser animated background widget.
pub fn laser<'a, Message, Theme, Renderer>() -> laser::Laser<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    laser::Laser::new()
}

/// Waveform animated background widget.
#[allow(dead_code)]
pub fn waveform<'a, Message, Theme, Renderer>(
    is_playing: bool,
    is_loading: bool,
    seed: u64,
    on_update: Message,
) -> waveform::Waveform<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    waveform::Waveform::new(is_playing, is_loading, seed, on_update)
}

/// Resizable widget.
#[allow(dead_code)]
pub fn resizable<'a, Message, Theme, Renderer, R>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_resize: R,
) -> resizable::Resizable<'a, Message, Theme, Renderer, R>
where
    R: Fn(Rectangle<f32>) -> Message,
{
    resizable::Resizable::new(content, on_resize)
}

/// Header divider widget.
pub fn header_divider<'a, Message, Theme, Renderer, D>(
    on_drag: D,
    on_press: Message,
    on_release: Message,
) -> header_divider::HeaderDivider<'a, Message, Theme, Renderer, D>
where
    Message: Clone,
    Theme: container::Catalog,
    D: Fn(f32) -> Message,
{
    header_divider::HeaderDivider::new(on_drag, on_press, on_release)
}

/// Optimized row creation.
#[inline]
pub fn row<'a, Message, Theme, Renderer>(
    content: impl Into<Vec<Element<'a, Message, Theme, Renderer>>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Row::from_vec(content.into())
}

/// Optimized column creation.
#[inline]
pub fn column<'a, Message, Theme, Renderer>(
    content: impl Into<Vec<Element<'a, Message, Theme, Renderer>>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    Column::from_vec(content.into())
}
