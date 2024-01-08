use std::collections::BTreeSet;
use std::ops::Add;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use iced::alignment::Horizontal;
use iced::alignment::Vertical;

use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::UnboundedSender;
use iced::futures::SinkExt;
use iced::futures::StreamExt;

use iced::keyboard::Modifiers;

use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::image;
use iced::widget::mouse_area;
use iced::widget::progress_bar;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::text_input;
use iced::widget::vertical_space;

use iced::Alignment;
use iced::Application;
use iced::Color;
use iced::Command;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Size;
use iced::Theme;

use porter_preview::PreviewRenderer;

use crate::porter_overlay;
use crate::porter_spinner;
use crate::PorterAssetManager;
use crate::PorterBackgroundStyle;
use crate::PorterButtonStyle;
use crate::PorterColumnHeader;
use crate::PorterDivider;
use crate::PorterDividerStyle;
use crate::PorterExecutor;
use crate::PorterHeaderBackgroundStyle;
use crate::PorterLabelStyle;
use crate::PorterMainBuilder;
use crate::PorterMainColumn;
use crate::PorterOverlayBackgroundStyle;
use crate::PorterPreviewAsset;
use crate::PorterPreviewButtonStyle;
use crate::PorterPreviewStyle;
use crate::PorterProgressStyle;
use crate::PorterRowStyle;
use crate::PorterScrollStyle;
use crate::PorterSettings;
use crate::PorterSpinnerStyle;
use crate::PorterSwitchButtonBackgroundStyle;
use crate::PorterSwitchButtonStyle;
use crate::PorterText;
use crate::PorterTextInputStyle;
use crate::PorterTitleFont;
use crate::PorterViewport;

/// The height of each row in px.
pub const ROW_HEIGHT: f32 = 26.0;
/// The padding in between each row in px.
pub const ROW_PADDING: f32 = 0.0;

/// Number of rows to render in addition to the starting row.
pub const ROW_OVERSCAN: usize = 50;

/// The minimum width of a column.
pub const COLUMN_MIN: f32 = 50.0;
/// The maximum width of a column.
pub const COLUMN_MAX: f32 = 1000.0;

/// The maximum number of assets before search isn't realtime.
pub const SEARCH_REALTIME_MAX: usize = 250000;

/// Time in which a double click is registered.
pub const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(250);

/// A list of preview controls to render over the previewer.
pub const PREVIEW_CONTROLS: &[(&str, &str)] = &[
    ("Toggle Bones:", "[B]"),
    ("Toggle Wireframe:", "[W]"),
    ("Toggle Shaded:", "[M]"),
    ("Reset View:", "[R]"),
    ("Cycle Image:", "[N]"),
];

/// Main window of the porter ui application.
pub struct PorterMain {
    pub(crate) title: String,
    pub(crate) name: String,
    pub(crate) donate_url: String,
    pub(crate) item_range: Range<usize>,
    pub(crate) item_selection: BTreeSet<usize>,
    pub(crate) asset_manager: Arc<dyn PorterAssetManager>,
    pub(crate) file_filters: Vec<(String, Vec<String>)>,
    pub(crate) multi_file: bool,
    pub(crate) preview_enabled: bool,
    pub(crate) materials_enabled: bool,
    pub(crate) sounds_enabled: bool,
    pub(crate) raw_files_enabled: bool,
    pub(crate) raw_files_forcable: bool,
    pub(crate) row_press: Option<usize>,
    pub(crate) row_press_last: Instant,
    pub(crate) loading: bool,
    pub(crate) exporting: bool,
    pub(crate) show_settings: bool,
    pub(crate) show_about: bool,
    pub(crate) export_progress: u32,
    pub(crate) keyboard_modifiers: Modifiers,
    pub(crate) search_id: text_input::Id,
    pub(crate) search_value: String,
    pub(crate) scroll_id: scrollable::Id,
    pub(crate) scroll_container_id: container::Id,
    pub(crate) scroll_viewport_size: Rectangle,
    pub(crate) scroll_viewport_state: PorterViewport,
    pub(crate) previewer: Option<PreviewRenderer>,
    pub(crate) previewer_container_id: container::Id,
    pub(crate) preview_viewport_size: Rectangle,
    pub(crate) preview_request_id: u64,
    pub(crate) mouse_position: Point,
    pub(crate) mouse_button: Option<iced::mouse::Button>,
    pub(crate) columns: Vec<PorterMainColumn>,
    pub(crate) channel: Option<UnboundedSender<Message>>,
    pub(crate) last_load: Option<Vec<PathBuf>>,
    pub(crate) file_dropped: Vec<PathBuf>,
    pub(crate) reload_required: bool,
    pub(crate) settings: PorterSettings,
}

/// Messages for the porter ui application.
#[derive(Debug, Clone)]
pub enum Message {
    UIEvent(Event),
    UIChannel(UnboundedSender<Message>),
    Scroll(scrollable::Viewport),
    ScrollResize(Option<Rectangle>),
    Preview(Option<PorterPreviewAsset>, u64),
    PreviewResize(Option<Rectangle>),
    ClosePreview,
    Sync(bool, u32),
    RowPress(usize),
    RowRelease(usize),
    LoadFile,
    LoadFileDropped,
    LoadGame,
    LoadResult(Result<(), String>),
    SearchInput(String),
    SearchClear,
    SearchSubmit,
    CancelExport,
    Donate,
    ToggleAbout,
    ToggleSettings,
    ExportSelected,
    ExportAll,
    SaveSettings(PorterSettings),
    OpenConfigFolder,
    PickExportFolder,
    ColumnDrag(usize, f32),
    ColumnDragEnd(usize),
    Noop,
}

impl Application for PorterMain {
    type Executor = PorterExecutor;

    type Message = Message;

    type Theme = Theme;

    type Flags = PorterMainBuilder;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut settings = PorterSettings::load(flags.name.clone());

        if !flags.materials_enabled {
            settings.set_load_materials(false);
        }

        if !flags.sounds_enabled {
            settings.set_load_sounds(false);
        }

        if !flags.raw_files_enabled {
            settings.set_load_raw_files(false);
        }

        if !flags.raw_files_forcable {
            settings.set_force_raw_files(false);
        }

        (
            Self {
                title: flags.title,
                name: flags.name,
                donate_url: flags.donate_url,
                item_range: 0..0,
                item_selection: BTreeSet::new(),
                asset_manager: flags.asset_manager,
                file_filters: flags.file_filters,
                multi_file: flags.multi_file,
                preview_enabled: flags.preview,
                materials_enabled: flags.materials_enabled,
                sounds_enabled: flags.sounds_enabled,
                raw_files_enabled: flags.raw_files_enabled,
                raw_files_forcable: flags.raw_files_forcable,
                row_press: None,
                row_press_last: Instant::now(),
                loading: false,
                exporting: false,
                show_settings: false,
                show_about: false,
                export_progress: 0,
                keyboard_modifiers: Modifiers::empty(),
                search_id: text_input::Id::unique(),
                search_value: String::new(),
                scroll_id: scrollable::Id::unique(),
                scroll_container_id: container::Id::unique(),
                scroll_viewport_size: Rectangle::with_size(Size::ZERO),
                scroll_viewport_state: PorterViewport::zero(),
                previewer: None,
                previewer_container_id: container::Id::unique(),
                preview_viewport_size: Rectangle::with_size(Size::ZERO),
                preview_request_id: 0,
                mouse_position: Point::ORIGIN,
                mouse_button: None,
                columns: flags.columns,
                channel: None,
                last_load: None,
                file_dropped: Vec::new(),
                reload_required: false,
                settings,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::UIEvent(event) => self.on_ui_event(event),
            Message::UIChannel(channel) => self.on_ui_channel(channel),
            Message::Scroll(viewport) => self.on_scroll(viewport),
            Message::ScrollResize(viewport) => self.on_scroll_resize(viewport),
            Message::Preview(asset, request_id) => self.on_preview(asset, request_id),
            Message::PreviewResize(viewport) => self.on_preview_resize(viewport),
            Message::ClosePreview => self.on_close_preview(),
            Message::Sync(exporting, progress) => self.on_sync(exporting, progress),
            Message::RowPress(index) => self.on_row_press(index),
            Message::RowRelease(index) => self.on_row_release(index),
            Message::LoadFile => self.on_load_file(),
            Message::LoadFileDropped => self.on_load_file_dropped(),
            Message::LoadGame => self.on_load_game(),
            Message::LoadResult(result) => self.on_load_result(result),
            Message::SearchInput(input) => self.on_search_input(input),
            Message::SearchClear => self.on_search_clear(),
            Message::SearchSubmit => self.on_search_submit(),
            Message::CancelExport => self.on_cancel_export(),
            Message::Donate => self.on_donate(),
            Message::ToggleSettings => self.on_toggle_settings(),
            Message::ToggleAbout => self.on_toggle_about(),
            Message::ExportSelected => self.on_export_selected(),
            Message::ExportAll => self.on_export_all(),
            Message::SaveSettings(settings) => self.on_save_settings(settings),
            Message::OpenConfigFolder => self.on_open_config_folder(),
            Message::PickExportFolder => self.on_pick_export_folder(),
            Message::ColumnDrag(index, offset) => self.on_column_drag(index, offset),
            Message::ColumnDragEnd(index) => self.on_column_drag_end(index),
            Message::Noop => self.on_noop(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let events = iced::event::listen().map(Message::UIEvent);

        let channel = iced::subscription::channel("main", 100, |mut output| async move {
            let (tx, mut rx) = mpsc::unbounded::<Message>();

            let result = output.send(Message::UIChannel(tx)).await;

            debug_assert!(result.is_ok());

            loop {
                while let Some(msg) = rx.next().await {
                    let result = output.send(msg).await;

                    debug_assert!(result.is_ok());
                }
            }
        });

        iced::Subscription::batch([events, channel])
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let panels = if self.show_about {
            vec![self.header(), self.about()]
        } else if self.show_settings {
            vec![self.header(), self.settings()]
        } else if let Some(preview) = &self.previewer {
            vec![
                self.header(),
                self.search(),
                row(vec![self.list(), self.preview(preview)])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .spacing(4.0)
                    .padding([0.0, 8.0])
                    .into(),
                self.controls(),
            ]
        } else {
            vec![
                self.header(),
                self.search(),
                row(vec![self.list()])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .padding([0.0, 8.0])
                    .into(),
                self.controls(),
            ]
        };

        container(column(panels))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(PorterBackgroundStyle)
            .into()
    }
}

impl PorterMain {
    /// Constructs the preview element and header.
    pub fn preview(&self, preview: &PreviewRenderer) -> Element<Message> {
        let (width, height, pixels) = preview.render();
        let handle = image::Handle::from_pixels(width, height, pixels);

        let mut columns = column(Vec::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(4.0);

        for (stat_header, stat_value) in preview.statistics() {
            columns = columns.push(
                row(vec![
                    text(stat_header)
                        .size(16.0)
                        .width(75.0)
                        .style(Color::from_rgb8(0x27, 0x9B, 0xD4))
                        .into(),
                    text(":")
                        .size(16.0)
                        .style(Color::from_rgb8(0x27, 0x9B, 0xD4))
                        .into(),
                    text(stat_value).size(16.0).style(Color::WHITE).into(),
                ])
                .width(Length::Shrink)
                .spacing(8.0),
            );
        }

        let columns = container(
            container(columns)
                .width(Length::Shrink)
                .padding(4.0)
                .style(PorterOverlayBackgroundStyle),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .padding(4.0);

        let mut controls = column(Vec::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(4.0);

        for (control_name, control) in PREVIEW_CONTROLS {
            controls = controls.push(
                row(vec![
                    text(control_name)
                        .size(16.0)
                        .style(Color::from_rgb8(0x27, 0x9B, 0xD4))
                        .into(),
                    text(control).size(16.0).style(Color::WHITE).into(),
                ])
                .width(Length::Shrink)
                .spacing(8.0),
            );
        }

        let controls = container(
            container(controls)
                .width(Length::Shrink)
                .padding(4.0)
                .style(PorterOverlayBackgroundStyle),
        )
        .align_y(Vertical::Bottom)
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .padding(4.0);

        container(
            column(vec![
                container(
                    row(vec![
                        text("Asset Preview")
                            .width(Length::Fill)
                            .style(Color::WHITE)
                            .into(),
                        button(text("\u{2715}").size(20.0).shaping(text::Shaping::Advanced))
                            .on_press(Message::ClosePreview)
                            .padding(0.0)
                            .style(PorterPreviewButtonStyle)
                            .into(),
                    ])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(Alignment::Center),
                )
                .width(Length::Fill)
                .height(30.0)
                .padding([0.0, 8.0, 0.0, 4.0])
                .align_y(Vertical::Center)
                .style(PorterColumnHeader)
                .into(),
                container(porter_overlay(
                    image(handle)
                        .content_fit(iced::ContentFit::Cover)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    if self.settings.preview_overlay() {
                        column(vec![columns.into(), controls.into()])
                            .width(Length::Fill)
                            .height(Length::Fill)
                    } else {
                        column(vec![columns.into()])
                            .width(Length::Fill)
                            .height(Length::Fill)
                    },
                ))
                .id(self.previewer_container_id.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(1.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(PorterPreviewStyle)
        .into()
    }

    /// Constructs the header view element, with app info, version, about and settings.
    pub fn header(&self) -> Element<Message> {
        container(row(vec![
            container(
                button("Donate")
                    .on_press(Message::Donate)
                    .style(PorterButtonStyle),
            )
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .into(),
            container(
                row(vec![
                    text(self.name.clone())
                        .style(Color::WHITE)
                        .font(PorterTitleFont)
                        .size(32.0)
                        .into(),
                    text("by").style(Color::WHITE).size(12.0).into(),
                    text("DTZxPorter")
                        .style(Color::from_rgb8(236, 52, 202))
                        .size(12.0)
                        .into(),
                ])
                .height(Length::Fill)
                .spacing(4.0)
                .align_items(Alignment::Center),
            )
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into(),
            container(
                container(
                    row(vec![
                        button("About")
                            .on_press(Message::ToggleAbout)
                            .style(PorterSwitchButtonStyle(self.show_about))
                            .into(),
                        button("Settings")
                            .on_press(Message::ToggleSettings)
                            .style(PorterSwitchButtonStyle(self.show_settings))
                            .into(),
                    ])
                    .spacing(8.0)
                    .align_items(Alignment::Center),
                )
                .padding(3.0)
                .align_y(Vertical::Center)
                .style(PorterSwitchButtonBackgroundStyle),
            )
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center)
            .into(),
        ]))
        .width(Length::Fill)
        .height(52.0)
        .padding([0.0, 8.0])
        .style(PorterHeaderBackgroundStyle)
        .into()
    }

    /// Constructs the search view element with text input, clear button, and assets loaded info.
    pub fn search(&self) -> Element<Message> {
        let mut search = vec![if self.loading || self.exporting {
            text_input("Search for assets...", self.search_value.as_str())
                .style(PorterTextInputStyle)
                .width(Length::Fixed(350.0))
                .into()
        } else {
            text_input("Search for assets...", self.search_value.as_str())
                .id(self.search_id.clone())
                .on_input(Message::SearchInput)
                .on_submit(Message::SearchSubmit)
                .style(PorterTextInputStyle)
                .width(Length::Fixed(350.0))
                .into()
        }];

        if self.asset_manager.loaded_len() > SEARCH_REALTIME_MAX {
            search.push(
                button("Search")
                    .padding([5.0, 8.0])
                    .style(PorterButtonStyle)
                    .on_press_maybe(
                        if self.search_value.is_empty() || self.loading || self.exporting {
                            None
                        } else {
                            Some(Message::SearchSubmit)
                        },
                    )
                    .into(),
            );
        }

        search.extend([
            button("Clear")
                .padding([5.0, 8.0])
                .style(PorterButtonStyle)
                .on_press_maybe(
                    if self.search_value.is_empty() || self.loading || self.exporting {
                        None
                    } else {
                        Some(Message::SearchClear)
                    },
                )
                .into(),
            container(
                text(if self.loading {
                    "Loading...".to_string()
                } else if self.search_value.is_empty() {
                    format!("{} assets loaded", self.asset_manager.len())
                } else {
                    format!(
                        "Showing {} assets out of {} loaded",
                        self.asset_manager.len(),
                        self.asset_manager.loaded_len()
                    )
                })
                .style(PorterLabelStyle),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center)
            .into(),
        ]);

        container(
            row(search)
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(4.0)
                .padding(8.0)
                .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(52.0)
        .into()
    }

    /// Constructs the controls view element with load and export buttons.
    pub fn controls(&self) -> Element<Message> {
        let mut row = row(Vec::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8.0)
            .padding(8.0)
            .align_items(Alignment::Center);

        if self.asset_manager.supports_load_game() {
            row = row.push(
                button("Load Game")
                    .padding([5.0, 8.0])
                    .style(PorterButtonStyle)
                    .on_press_maybe(if self.loading || self.exporting {
                        None
                    } else {
                        Some(Message::LoadGame)
                    }),
            );
        }

        if self.asset_manager.supports_load_files() {
            row = row.push(
                button("Load File")
                    .padding([5.0, 8.0])
                    .style(PorterButtonStyle)
                    .on_press_maybe(if self.loading || self.exporting {
                        None
                    } else {
                        Some(Message::LoadFile)
                    }),
            );
        }

        row = row
            .push(
                button("Export Selected")
                    .padding([5.0, 8.0])
                    .style(PorterButtonStyle)
                    .on_press_maybe(
                        if self.item_selection.is_empty() || self.loading || self.exporting {
                            None
                        } else {
                            Some(Message::ExportSelected)
                        },
                    ),
            )
            .push(
                button("Export All")
                    .padding([5.0, 8.0])
                    .style(PorterButtonStyle)
                    .on_press_maybe(
                        if self.asset_manager.is_empty() || self.loading || self.exporting {
                            None
                        } else {
                            Some(Message::ExportAll)
                        },
                    ),
            );

        if self.exporting {
            row = row
                .push(
                    button("Cancel")
                        .padding([5.0, 8.0])
                        .style(PorterButtonStyle)
                        .on_press(Message::CancelExport),
                )
                .push(
                    container(porter_overlay(
                        progress_bar(0.0..=100.0, self.export_progress.min(100).max(0) as f32)
                            .width(200.0)
                            .height(32.0)
                            .style(PorterProgressStyle),
                        container(
                            text(format!("{}%", self.export_progress.min(100).max(0)))
                                .size(16.0)
                                .style(Color::WHITE),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Right)
                    .align_y(Vertical::Center),
                )
        }

        container(row).width(Length::Fill).height(52.0).into()
    }

    /// Constructs the list view element with it's headers, rows, and columns.
    pub fn list(&self) -> Element<Message> {
        let item_size = ROW_HEIGHT + ROW_PADDING;
        let item_range = self.item_range.clone();

        let top_gap = vertical_space((item_range.start as f32) * item_size);
        let bottom_gap = vertical_space(
            ((self.asset_manager.len() - (item_range.start + item_range.len())) as f32) * item_size,
        );

        let mut rows: Vec<Element<_, _>> = Vec::with_capacity(ROW_OVERSCAN + 2);

        rows.push(top_gap.into());

        for row_index in item_range {
            let mut columns: Vec<Element<_, _>> = Vec::with_capacity(self.columns.len());

            let selected = self.item_selection.contains(&row_index);

            for (column, (value, color)) in self
                .columns
                .iter()
                .zip(self.asset_manager.asset_info(row_index, self.columns.len()))
            {
                columns.push(
                    PorterText::new(value)
                        .width(column.width.min(COLUMN_MAX).max(COLUMN_MIN).add(5.0))
                        .height(Length::Fill)
                        .vertical_alignment(Vertical::Center)
                        .color(selected.then_some(Color::WHITE).unwrap_or_else(|| {
                            color.unwrap_or_else(|| column.color.unwrap_or(Color::WHITE))
                        }))
                        .build()
                        .into(),
                );
            }

            let row = container(
                row(columns)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .spacing(4.0)
                    .padding([0.0, 4.0])
                    .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(ROW_HEIGHT)
            .style(PorterRowStyle::new(row_index, selected));

            rows.push(if let Some(row_press) = self.row_press {
                if row_index == row_press {
                    mouse_area(row)
                        .on_release(Message::RowRelease(row_index))
                        .into()
                } else {
                    mouse_area(row)
                        .on_press(Message::RowPress(row_index))
                        .into()
                }
            } else {
                mouse_area(row)
                    .on_press(Message::RowPress(row_index))
                    .into()
            });
        }

        rows.push(bottom_gap.into());

        let scroller = scrollable(
            column(rows)
                .spacing(ROW_PADDING)
                .padding([0.0, 17.0, 0.0, 0.0])
                .align_items(Alignment::Center),
        )
        .id(self.scroll_id.clone())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(PorterScrollStyle)
        .direction(scrollable::Direction::Vertical(
            scrollable::Properties::new()
                .width(16.0)
                .scroller_width(16.0),
        ))
        .on_scroll(Message::Scroll);

        let scroller = container(scroller)
            .id(self.scroll_container_id.clone())
            .width(Length::Fill)
            .height(Length::Fill);

        let mut columns: Vec<Element<_, _>> = Vec::with_capacity(self.columns.len());

        for (index, column) in self.columns.iter().enumerate() {
            columns.push(
                PorterText::new(column.header.clone())
                    .width(column.width.min(COLUMN_MAX).max(COLUMN_MIN))
                    .height(Length::Fill)
                    .vertical_alignment(Vertical::Center)
                    .color(Color::WHITE)
                    .build()
                    .into(),
            );

            columns.push(
                PorterDivider::new(
                    move |offset| Message::ColumnDrag(index, offset),
                    Message::ColumnDragEnd(index),
                )
                .height(Length::Fixed(28.0))
                .width(3.0)
                .style(PorterDividerStyle)
                .build()
                .into(),
            );
        }

        let header = container(
            row(columns)
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(4.0)
                .padding([0.0, 4.0])
                .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(30.0)
        .style(PorterColumnHeader);

        let empty_element = if self.loading {
            Element::from(
                porter_spinner::Circular::new()
                    .size(48.0)
                    .style(PorterSpinnerStyle.into())
                    .cycle_duration(Duration::from_secs(2)),
            )
        } else {
            let middle_text = if self.asset_manager.loaded_len() == 0 {
                match (
                self.asset_manager.supports_load_files(),
                self.asset_manager.supports_load_game(),
            ) {
                (true, true) => {
                    "Either load a running instance of a supported game or one of the supported game files to view and export assets."
                }
                (false, true) => {
                    "Load a running instance of a supported game to view and export assets."
                }
                (true, false) => {
                    "Load one of the supported game files to view and export assets."
                }
                (false, false) => "No supported loading mechanisms available.",
            }
            } else {
                "No assets were found. Try adjusting your search term."
            };

            Element::from(text(middle_text).style(PorterLabelStyle))
        };

        let list = container(
            container(if self.asset_manager.is_empty() {
                Element::from(
                    container(empty_element)
                        .id(self.scroll_container_id.clone())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
            } else {
                Element::from(scroller)
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .style(PorterBackgroundStyle),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(1.0)
        .style(PorterHeaderBackgroundStyle);

        column(vec![header.into(), list.into()])
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
