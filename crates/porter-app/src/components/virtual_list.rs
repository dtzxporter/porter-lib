use std::cmp::Ordering;

use iced::border::Radius;
use iced::border::rounded;

use iced::widget::Id;
use iced::widget::container;
use iced::widget::mouse_area;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;

use iced::widget::text::Ellipsis;
use iced::widget::text::Wrapping;

use iced::widget::operation::scroll_by;
use iced::widget::operation::scroll_to;

use iced::Alignment;
use iced::Background;
use iced::Border;
use iced::Element;
use iced::Length;
use iced::Padding;
use iced::Task;
use iced::Theme;

use crate::AppState;
use crate::Message;
use crate::Sort;
use crate::fonts;
use crate::palette;
use crate::widgets;

use super::PreviewMessage;

/// Size of a row in pixels.
const ROW_HEIGHT: f32 = 28.0;
/// Number of rows to render.
const ROW_OVERSCAN: usize = 50;

/// Size of the header in pixels.
const HEADER_HEIGHT: f32 = 30.0;

/// The minimum width of a column.
pub const COLUMN_MIN: f32 = 50.0;

/// Size of the scrollbars in pixels.
const SCROLLBAR_WIDTH: f32 = 16.0;

/// Virtual list component handler.
pub struct VirtualList {
    viewport: Option<scrollable::Viewport>,
    header_id: Id,
    scroll_id: Id,
    scroll_overflow: bool,
    dragging: bool,
    dragging_min_width: Option<f32>,
}

/// Messages produced by the virtual list component.
#[derive(Debug, Clone)]
pub enum VirtualListMessage {
    Noop,
    Scroll(scrollable::Viewport),
    ScrollReset,
    Click(usize),
    DoubleClick(usize),
    HeaderDrag(usize, f32),
    HeaderDragStart(usize),
    HeaderDragEnd(usize),
    Sort(usize),
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
}

impl VirtualList {
    /// Creates a new virtual list component.
    pub fn new() -> Self {
        Self {
            viewport: None,
            header_id: Id::unique(),
            scroll_id: Id::unique(),
            scroll_overflow: false,
            dragging: false,
            dragging_min_width: None,
        }
    }

    /// Handles updates for the virtual list component.
    pub fn update(&mut self, state: &mut AppState, message: VirtualListMessage) -> Task<Message> {
        use VirtualListMessage::*;

        match message {
            Noop => self.on_noop(),
            Scroll(viewport) => self.on_scroll(state, viewport),
            ScrollReset => self.on_scroll_reset(state),
            Click(index) => self.on_click(state, index),
            DoubleClick(index) => self.on_double_click(state, index),
            HeaderDrag(index, offset) => self.on_header_drag(state, index, offset),
            HeaderDragStart(index) => self.on_header_drag_start(state, index),
            HeaderDragEnd(index) => self.on_header_drag_end(state, index),
            Sort(index) => self.on_sort(state, index),
            MoveUp => self.on_move_up(state),
            MoveDown => self.on_move_down(state),
            PageUp => self.on_page_up(state),
            PageDown => self.on_page_down(state),
        }
    }

    /// Handles rendering for the virtual list component.
    pub fn view(&self, state: &AppState) -> Element<'_, Message> {
        let content_width = self.content_width(state);
        let content: Element<Message> = if state.loading || state.asset_manager.assets_empty() {
            if state.loading {
                widgets::spinner().into()
            } else {
                let supports_files = state.asset_manager.supports_files();
                let supports_games = state.asset_manager.supports_games();

                let middle_text = if state.asset_manager.assets_total() == 0 {
                    match (supports_files, supports_games) {
                        (true, true) => {
                            "Load a running game or supported game file to view and export assets."
                        }
                        (false, true) => "Load a running game to view and export assets.",
                        (true, false) => "Load a supported game file to view and export assets.",
                        (false, false) => "No supported loading mechanisms available.",
                    }
                } else {
                    "No assets were found. Try adjusting your search term."
                };

                let mut rows: Vec<_> = Vec::with_capacity(2);

                rows.push(
                    text(middle_text)
                        .color(palette::TEXT_COLOR_SECONDARY)
                        .into(),
                );

                if state.settings.has_last_files() && supports_files {
                    rows.push(
                        widgets::link("Load Last File")
                            .on_press_maybe(if state.is_busy() {
                                None
                            } else {
                                Some(Message::LoadLastFiles)
                            })
                            .into(),
                    );
                }

                widgets::column(rows)
                    .align_x(Alignment::Center)
                    .into()
            }
        } else {
            let item_range = state.item_range.clone();

            let top_gap = (item_range.start as f32) * ROW_HEIGHT;
            let bottom_gap = (((state
                .asset_manager
                .assets_visible()
                .saturating_sub(item_range.start + item_range.len()))
                as f32)
                * ROW_HEIGHT)
                + ((self.scroll_overflow as u32) as f32 * SCROLLBAR_WIDTH);

            let mut rows: Vec<_> = Vec::with_capacity(ROW_OVERSCAN + 2);

            rows.push(space().height(top_gap).into());

            for index in item_range {
                let mut columns: Vec<_> = Vec::with_capacity(state.asset_columns.len());

                let selected = state.assets_selected.contains(&index);

                for (column, (value, color)) in state
                    .asset_columns
                    .iter()
                    .zip(state.asset_manager.assets_info(index))
                {
                    let color = if selected {
                        palette::TEXT_COLOR_DEFAULT
                    } else {
                        color.unwrap_or_else(|| {
                            column
                                .color
                                .unwrap_or(palette::TEXT_COLOR_DEFAULT)
                        })
                    };

                    columns.push(
                        text(value)
                            .width(column.width.max(COLUMN_MIN))
                            .height(Length::Fill)
                            .ellipsis(Ellipsis::End)
                            .wrapping(Wrapping::None)
                            .align_y(Alignment::Center)
                            .color(color)
                            .into(),
                    );
                }

                let list_item = widgets::list_item(
                    mouse_area(
                        widgets::row(columns)
                            .clip(true)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(Padding::ZERO.left(4.0))
                            .spacing(4.0)
                            .align_y(Alignment::Center),
                    )
                    .on_press(Message::from(VirtualListMessage::Click(index)))
                    .on_double_click(Message::from(VirtualListMessage::DoubleClick(index))),
                    index,
                    selected,
                )
                // We want events from the mouse area, not the list item.
                // We want the list item to still respond to events.
                // Only if we are not dragging (looks janky).
                .on_press_maybe(if self.dragging {
                    None
                } else {
                    Some(Message::from(VirtualListMessage::Noop))
                })
                .width(Length::Fill)
                .height(Length::Fixed(ROW_HEIGHT));

                let mut copy_menu = widgets::menu_item("Copy");

                for (column_index, column) in state.asset_columns.iter().enumerate() {
                    copy_menu = copy_menu.child(
                        widgets::menu_item(column.header)
                            // Copy the value of this column for this asset.
                            .on_press(Message::CopyColumn(index, column_index)),
                    );
                }

                let row = widgets::context(list_item)
                    .item(
                        widgets::menu_item("Export")
                            // Only allow export when state is not busy.
                            .on_press_maybe(if state.is_busy() {
                                None
                            } else {
                                Some(Message::ExportOne(index))
                            }),
                    )
                    .item(widgets::menu_item_divider())
                    .item(
                        widgets::menu_item("Preview")
                            .child(
                                widgets::menu_item("Asset")
                                    .on_press(Message::PreviewAsset(index, false)),
                            )
                            .child(
                                widgets::menu_item("Asset (Raw)")
                                    .on_press(Message::PreviewAsset(index, true)),
                            ),
                    )
                    .item(widgets::menu_item_divider())
                    .item(copy_menu);

                rows.push(row.into());
            }

            rows.push(space().height(bottom_gap).into());

            widgets::scrollable(widgets::row([
                widgets::column(rows)
                    .width(Length::Fixed(content_width))
                    .into(),
                space().width(SCROLLBAR_WIDTH).into(),
            ]))
            .id(self.scroll_id.clone())
            .on_scroll(|viewport| Message::from(VirtualListMessage::Scroll(viewport)))
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::new()
                    .width(SCROLLBAR_WIDTH)
                    .scroller_width(SCROLLBAR_WIDTH)
                    .spacing(0.0),
                horizontal: scrollable::Scrollbar::new()
                    .width(SCROLLBAR_WIDTH)
                    .scroller_width(SCROLLBAR_WIDTH)
                    .spacing(0.0),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        let mut headers: Vec<_> = Vec::with_capacity(state.asset_columns.len() * 2);

        for (index, column) in state.asset_columns.iter().enumerate() {
            let mut heading: Vec<_> = Vec::with_capacity(4);

            heading.extend([
                space().width(4.0).into(),
                text(column.header)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .ellipsis(Ellipsis::End)
                    .wrapping(Wrapping::None)
                    .align_y(Alignment::Center)
                    .into(),
            ]);

            if matches!(column.sort, Some(Sort::Ascending) | Some(Sort::Descending)) {
                heading.extend([
                    text(if matches!(column.sort, Some(Sort::Ascending)) {
                        "\u{F1CB}"
                    } else {
                        "\u{F1CC}"
                    })
                    .height(Length::Fill)
                    .font(fonts::ICON_FONT)
                    .align_y(Alignment::Center)
                    .into(),
                    space().width(8.0).into(),
                ]);
            }

            headers.extend([
                mouse_area(
                    widgets::row(heading)
                        .width(column.width.max(COLUMN_MIN))
                        .height(Length::Fill),
                )
                .on_press(Message::from(VirtualListMessage::Sort(index)))
                .into(),
                widgets::header_divider(
                    move |offset| Message::from(VirtualListMessage::HeaderDrag(index, offset)),
                    Message::from(VirtualListMessage::HeaderDragStart(index)),
                    Message::from(VirtualListMessage::HeaderDragEnd(index)),
                )
                .width(Length::Fixed(4.0))
                .height(Length::Fixed(HEADER_HEIGHT - 4.0))
                .style(list_header_divider_style)
                .into(),
            ]);
        }

        let header = container(
            scrollable::Scrollable::with_direction(
                widgets::row(headers)
                    .width(Length::Fixed(content_width + SCROLLBAR_WIDTH))
                    .height(Length::Fill)
                    .align_y(Alignment::Center),
                scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::new()
                        .width(0.0)
                        .scroller_width(0.0)
                        .margin(0.0),
                ),
            )
            .id(self.header_id.clone())
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fixed(HEADER_HEIGHT))
        .style(list_header_style);

        widgets::column([
            header.into(),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(list_container_style)
                .into(),
        ])
        .align_x(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Occurs when no operation should occur.
    fn on_noop(&mut self) -> Task<Message> {
        Task::none()
    }

    /// Occurs when the user scrolls the virtual list.
    fn on_scroll(&mut self, state: &mut AppState, viewport: scrollable::Viewport) -> Task<Message> {
        let item_size = ROW_HEIGHT;

        let offsets = viewport.absolute_offset();
        let bounds = viewport.bounds();
        let content_bounds = viewport.content_bounds();

        let items_visible = state.asset_manager.assets_visible();

        let mut item_start = (offsets.y / item_size).floor() as usize;

        if item_start + ROW_OVERSCAN > items_visible {
            item_start = item_start.saturating_sub((item_start + ROW_OVERSCAN) - items_visible);
        }

        let item_end = (item_start + ROW_OVERSCAN).min(items_visible);

        state.item_range = item_start..item_end;

        self.scroll_overflow = content_bounds.width > bounds.width;

        self.viewport = Some(viewport);

        scroll_to(
            self.header_id.clone(),
            scrollable::AbsoluteOffset {
                x: offsets.x,
                y: 0.0,
            },
        )
    }

    /// Occurs when the scroll should reset.
    fn on_scroll_reset(&mut self, _: &mut AppState) -> Task<Message> {
        let offset = scrollable::AbsoluteOffset { x: 0.0, y: 0.0 };

        Task::batch([
            scroll_to(self.header_id.clone(), offset),
            scroll_to(self.scroll_id.clone(), offset),
        ])
    }

    /// Occurs when a row has been clicked.
    fn on_click(&mut self, state: &mut AppState, index: usize) -> Task<Message> {
        if state.modifier_keys.command() {
            if state.assets_selected.contains(&index) {
                state.assets_selected.remove(&index);
            } else {
                state.assets_selected.insert(index);
            }
        } else if state.modifier_keys.shift() {
            if let Some(first) = state.assets_selected.first() {
                match index.cmp(first) {
                    Ordering::Less => {
                        for i in index..*first {
                            state.assets_selected.insert(i);
                        }
                    }
                    Ordering::Greater => {
                        for i in *first..=index {
                            state.assets_selected.insert(i);
                        }
                    }
                    Ordering::Equal => {
                        state.assets_selected.insert(index);
                    }
                }
            } else if state.assets_selected.contains(&index) {
                state.assets_selected.remove(&index);
            } else {
                state.assets_selected.insert(index);
            }
        } else if state.assets_selected.len() == 1 && state.assets_selected.contains(&index) {
            return Task::none();
        } else {
            state.assets_selected.clear();
            state.assets_selected.insert(index);

            return Task::done(Message::from(PreviewMessage::Request));
        }

        Task::none()
    }

    /// Occurs when a row has been double clicked.
    fn on_double_click(&mut self, state: &mut AppState, index: usize) -> Task<Message> {
        state.assets_selected.clear();
        state.assets_selected.insert(index);

        Task::done(Message::ExportSelected)
    }

    /// Occurs when a column header is dragged.
    fn on_header_drag(&mut self, state: &mut AppState, index: usize, offset: f32) -> Task<Message> {
        if let Some(column) = state.asset_columns.get_mut(index) {
            column.width += offset;
        }

        if let Some(viewport) = &self.viewport {
            let offsets = viewport.absolute_offset();

            Task::batch([
                scroll_to(
                    self.header_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: offsets.x,
                        y: 0.0,
                    },
                ),
                scroll_to(
                    self.scroll_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: offsets.x,
                        y: offsets.y,
                    },
                ),
            ])
        } else {
            Task::none()
        }
    }

    /// Occurs when a column header has started dragging.
    fn on_header_drag_start(&mut self, state: &mut AppState, _index: usize) -> Task<Message> {
        self.dragging = true;
        self.dragging_min_width = Some(self.content_width(state));

        Task::none()
    }

    /// Occurs when a column header has finished dragging.
    fn on_header_drag_end(&mut self, state: &mut AppState, index: usize) -> Task<Message> {
        self.dragging = false;
        self.dragging_min_width = None;

        if let Some(column) = state.asset_columns.get_mut(index) {
            column.width = column.width.max(COLUMN_MIN);
        }

        Task::none()
    }

    /// Occurs when a column header should be sorted.
    fn on_sort(&mut self, state: &mut AppState, index: usize) -> Task<Message> {
        let Some(column) = state.asset_columns.get(index) else {
            return Task::none();
        };

        if column.sort.is_none() {
            return Task::none();
        }

        Task::done(Message::Sort(Some(index)))
    }

    /// Occurs when the up arrow is pressed.
    fn on_move_up(&mut self, state: &mut AppState) -> Task<Message> {
        let Some(index) = state.assets_selected.first().copied() else {
            return Task::none();
        };

        if index > 0 && state.assets_selected.len() == 1 {
            state.assets_selected.clear();
            state.assets_selected.insert(index - 1);

            let Some(viewport) = &self.viewport else {
                return Task::done(Message::from(PreviewMessage::Request));
            };

            return Task::batch([
                Task::done(Message::from(PreviewMessage::Request)),
                self.on_scroll_into_view(viewport, index - 1),
            ]);
        }

        Task::none()
    }

    /// Occurs when the down arrow is pressed.
    fn on_move_down(&mut self, state: &mut AppState) -> Task<Message> {
        let Some(index) = state.assets_selected.first().copied() else {
            return Task::none();
        };

        if !state.asset_manager.assets_empty()
            && index < state.asset_manager.assets_visible() - 1
            && state.assets_selected.len() == 1
        {
            state.assets_selected.clear();
            state.assets_selected.insert(index + 1);

            let Some(viewport) = &self.viewport else {
                return Task::done(Message::from(PreviewMessage::Request));
            };

            return Task::batch([
                Task::done(Message::from(PreviewMessage::Request)),
                self.on_scroll_into_view(viewport, index + 1),
            ]);
        }

        Task::none()
    }

    /// Occurs when the page up button is pressed.
    fn on_page_up(&mut self, _: &mut AppState) -> Task<Message> {
        let Some(viewport) = &self.viewport else {
            return Task::none();
        };

        let shift = (viewport.bounds().height / ROW_HEIGHT).floor() * ROW_HEIGHT;

        scroll_by(
            self.scroll_id.clone(),
            scrollable::AbsoluteOffset { x: 0.0, y: -shift },
        )
    }

    /// Occurs when the page down button is pressed.
    fn on_page_down(&mut self, _: &mut AppState) -> Task<Message> {
        let Some(viewport) = &self.viewport else {
            return Task::none();
        };

        let shift = (viewport.bounds().height / ROW_HEIGHT).floor() * ROW_HEIGHT;

        scroll_by(
            self.scroll_id.clone(),
            scrollable::AbsoluteOffset { x: 0.0, y: shift },
        )
    }

    /// Occurs when we want to scroll an item into view.
    fn on_scroll_into_view(&self, viewport: &scrollable::Viewport, index: usize) -> Task<Message> {
        let viewport_offset = viewport.absolute_offset();
        let viewport_bounds = viewport.bounds();

        let item_top = ROW_HEIGHT * index as f32;
        let item_bottom = ROW_HEIGHT * (index + 1) as f32;

        let viewport_top = viewport_offset.y;
        let viewport_bottom = viewport_offset.y + viewport_bounds.height;

        let scroll_top = if item_top < viewport_top {
            item_top
        } else if item_bottom > viewport_bottom {
            item_bottom - viewport_bounds.height
        } else {
            viewport_offset.y
        };

        scroll_to(
            self.scroll_id.clone(),
            scrollable::AbsoluteOffset {
                x: viewport_bounds.x,
                y: scroll_top,
            },
        )
    }

    /// Computes the width of all list view content.
    fn content_width(&self, state: &AppState) -> f32 {
        let column_width: f32 = state
            .asset_columns
            .iter()
            .map(|x| x.width.max(COLUMN_MIN) + 4.0)
            .sum();

        let column_width = column_width.max(
            self.viewport
                .as_ref()
                .map(|x| x.bounds().width)
                .unwrap_or_default()
                - SCROLLBAR_WIDTH,
        );

        column_width.max(
            self.dragging_min_width
                .unwrap_or_default(),
        )
    }
}

/// Style for the list container.
fn list_container_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
        ..Default::default()
    }
}

/// Style for the list header.
fn list_header_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette::BACKGROUND_COLOR_LIGHT_100)),
        border: Border {
            width: 1.0,
            color: palette::BACKGROUND_COLOR_LIGHT_100,
            radius: Radius::new(0.0)
                .top_left(4.0)
                .top_right(4.0),
        },
        ..Default::default()
    }
}

/// Style for the header dividers.
fn list_header_divider_style(_: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        background: Some(Background::Color(palette::BACKGROUND_COLOR_DEFAULT)),
        border: Border {
            width: 2.0,
            color: palette::BACKGROUND_COLOR_DEFAULT,
            ..rounded(4.0)
        },
        ..Default::default()
    }
}
