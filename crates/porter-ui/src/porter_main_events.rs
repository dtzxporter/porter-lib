use std::cmp::Ordering;
use std::path::PathBuf;
use std::time::Instant;

use iced::futures::channel::mpsc::UnboundedSender;

use iced::keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::keyboard::Modifiers;

use iced::mouse;
use iced::mouse::ScrollDelta;

use iced::widget::container;
use iced::widget::scrollable;
use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::scrollable::Viewport;
use iced::widget::text_input;

use iced::window;
use iced::Command;
use iced::Event;
use iced::Point;
use iced::Rectangle;

use rfd::FileDialog;
use rfd::MessageButtons;
use rfd::MessageDialog;
use rfd::MessageLevel;

use directories::ProjectDirs;

use porter_preview::PreviewKeyState;
use porter_preview::PreviewRenderer;

use porter_utils::StringCaseExt;

use crate::open_folder;
use crate::Message;
use crate::PorterMain;
use crate::PorterPreviewAsset;
use crate::PorterSearch;
use crate::PorterSettings;
use crate::PorterViewport;
use crate::PreviewControlScheme;

use crate::COLUMN_MAX;
use crate::COLUMN_MIN;
use crate::DOUBLE_CLICK_DURATION;
use crate::PORTER_DONATE_URL;
use crate::PORTER_SITE_URL;
use crate::ROW_HEIGHT;
use crate::ROW_OVERSCAN;
use crate::ROW_PADDING;
use crate::SEARCH_REALTIME_MAX;

impl PorterMain {
    pub fn on_ui_event(&mut self, event: Event) -> Command<Message> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => self.on_key_pressed(key),
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => self.on_key_released(key),
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.on_modifiers_changed(modifiers)
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => self.on_mouse_move(position),
            Event::Mouse(mouse::Event::ButtonPressed(button)) => self.on_mouse_button_press(button),
            Event::Mouse(mouse::Event::ButtonReleased(_)) => self.on_mouse_button_released(),
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => self.on_mouse_wheel(delta),
            Event::Window(
                _,
                window::Event::Resized {
                    width: _,
                    height: _,
                },
            ) => self.on_window_resize(),
            Event::Window(id, window::Event::FileDropped(file)) => self.on_file_dropped(id, file),
            Event::Window(id, window::Event::Opened { .. }) => self.on_window_opened(id),
            _ => Command::none(),
        }
    }

    pub fn on_key_pressed(&mut self, key: Key) -> Command<Message> {
        if self.loading || self.exporting || self.show_settings || self.show_about {
            return Command::none();
        }

        match key.as_ref() {
            Key::Character("c") => {
                if self.keyboard_modifiers.command() {
                    if let Some(buffer) = self.get_copy_text() {
                        return iced::clipboard::write(buffer);
                    }
                }
            }
            Key::Character("a") => {
                if self.keyboard_modifiers.command() {
                    return Command::batch([
                        text_input::focus(self.search_id.clone()),
                        text_input::select_all(self.search_id.clone()),
                    ]);
                }
            }
            Key::Character("v") => {
                if self.keyboard_modifiers.command() {
                    let read = iced::clipboard::read(|data| match data {
                        Some(data) => Message::SearchInput(data),
                        None => Message::Noop,
                    });

                    return Command::batch([text_input::focus(self.search_id.clone()), read]);
                }
            }
            _ => {
                // Not used.
            }
        }

        Command::none()
    }

    pub fn on_key_released(&mut self, key: Key) -> Command<Message> {
        match key.as_ref() {
            Key::Character("e") => {
                self.export_selected();
            }

            Key::Character("p") => {
                if !self.preview_enabled {
                    return Command::none();
                }

                if self.previewer.is_some() {
                    self.previewer = None;

                    return container::visible_bounds(self.scroll_container_id.clone())
                        .map(Message::ScrollResize);
                }

                self.previewer = Some(PreviewRenderer::new());
                self.request_preview_asset();

                return Command::batch([
                    container::visible_bounds(self.scroll_container_id.clone())
                        .map(Message::ScrollResize),
                    container::visible_bounds(self.previewer_container_id.clone())
                        .map(Message::PreviewResize),
                ]);
            }
            Key::Character("r") => {
                if let Some(previewer) = &mut self.previewer {
                    previewer.reset_view();
                }
            }
            Key::Character("b") => {
                if let Some(previewer) = &mut self.previewer {
                    previewer.toggle_bones();
                }
            }
            Key::Character("w") => {
                if let Some(previewer) = &mut self.previewer {
                    previewer.toggle_wireframe();
                }
            }
            Key::Character("m") => {
                if let Some(previewer) = &mut self.previewer {
                    previewer.toggle_shaded();
                }
            }
            Key::Character("g") => {
                if let Some(previewer) = &mut self.previewer {
                    previewer.toggle_grid();
                }
            }
            Key::Character("n") => {
                if let Some(previewer) = &mut self.previewer {
                    previewer.cycle_material();
                }
            }
            Key::Character("f") => {
                if self.keyboard_modifiers.command() {
                    return Command::batch([
                        text_input::focus(self.search_id.clone()),
                        text_input::select_all(self.search_id.clone()),
                    ]);
                }
            }
            Key::Named(Named::ArrowUp) => {
                if let Some(index) = self.item_selection.first().cloned() {
                    if index > 0 && self.item_selection.len() == 1 {
                        self.item_selection.clear();
                        self.item_selection.insert(index - 1);
                        self.request_preview_asset();
                    }
                }
            }
            Key::Named(Named::ArrowDown) => {
                if let Some(index) = self.item_selection.first().cloned() {
                    if !self.asset_manager.is_empty()
                        && index < self.asset_manager.len() - 1
                        && self.item_selection.len() == 1
                    {
                        self.item_selection.clear();
                        self.item_selection.insert(index + 1);
                        self.request_preview_asset();
                    }
                }
            }
            _ => {
                // Not used.
            }
        }

        Command::none()
    }

    pub fn on_modifiers_changed(&mut self, modifiers: Modifiers) -> Command<Message> {
        self.keyboard_modifiers = modifiers;

        Command::none()
    }

    pub fn on_mouse_move(&mut self, position: Point) -> Command<Message> {
        if !self.preview_viewport_size.contains(self.mouse_position) || self.previewer.is_none() {
            self.mouse_position = position;

            return Command::none();
        }

        let delta_position = self.mouse_position - position;

        if let Some(previewer) = &mut self.previewer {
            previewer.mouse_move(
                (delta_position.x, delta_position.y),
                PreviewKeyState {
                    maya: matches!(self.settings.preview_controls(), PreviewControlScheme::Maya),
                    left: matches!(self.mouse_button, Some(mouse::Button::Left)),
                    right: matches!(self.mouse_button, Some(mouse::Button::Right)),
                    middle: matches!(self.mouse_button, Some(mouse::Button::Middle)),
                    alt: self.keyboard_modifiers.alt() || self.keyboard_modifiers.command(),
                    shift: self.keyboard_modifiers.shift(),
                },
            );
        }

        self.mouse_position = position;

        Command::none()
    }

    pub fn on_mouse_wheel(&mut self, delta: ScrollDelta) -> Command<Message> {
        if !self.preview_viewport_size.contains(self.mouse_position) || self.previewer.is_none() {
            return Command::none();
        }

        let delta = match delta {
            ScrollDelta::Lines { x: _, y } => y,
            ScrollDelta::Pixels { x: _, y } => y,
        };

        if let Some(previewer) = &mut self.previewer {
            previewer.scroll_delta(delta);
        }

        Command::none()
    }

    pub fn on_mouse_button_press(&mut self, button: mouse::Button) -> Command<Message> {
        self.mouse_button = Some(button);

        Command::none()
    }

    pub fn on_mouse_button_released(&mut self) -> Command<Message> {
        self.row_press = None;
        self.mouse_button = None;

        Command::none()
    }

    pub fn on_window_resize(&mut self) -> Command<Message> {
        Command::batch([
            container::visible_bounds(self.scroll_container_id.clone()).map(Message::ScrollResize),
            container::visible_bounds(self.previewer_container_id.clone())
                .map(Message::PreviewResize),
        ])
    }

    pub fn on_file_dropped(&mut self, id: iced::window::Id, file: PathBuf) -> Command<Message> {
        if id != iced::window::Id::MAIN {
            return Command::none();
        }

        if self.exporting || self.loading {
            return Command::none();
        }

        if self.file_dropped.is_empty() {
            if let Some(channel) = self.channel.as_mut() {
                let result = channel.unbounded_send(Message::LoadFileDropped);

                debug_assert!(result.is_ok());
            }
        }

        self.file_dropped.push(file);

        Command::none()
    }

    pub fn on_window_opened(&mut self, id: iced::window::Id) -> Command<Message> {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::UI::WindowsAndMessaging::*;

            use raw_window_handle::RawWindowHandle;

            iced::window::run_with_handle(id, |handle| {
                let icon = crate::windows_icon();

                if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                    unsafe {
                        PostMessageW(
                            handle.hwnd.get() as _,
                            WM_SETICON,
                            ICON_BIG as WPARAM,
                            icon as LPARAM,
                        )
                    };
                    unsafe {
                        PostMessageW(
                            handle.hwnd.get() as _,
                            WM_SETICON,
                            ICON_SMALL as WPARAM,
                            icon as LPARAM,
                        )
                    };
                }

                Message::Noop
            })
        }

        #[cfg(not(target_os = "windows"))]
        let _ = id;

        #[cfg(not(target_os = "windows"))]
        Command::none()
    }

    pub fn on_ui_channel(&mut self, channel: UnboundedSender<Message>) -> Command<Message> {
        self.channel = Some(channel);

        Command::none()
    }

    pub fn on_scroll(&mut self, viewport: Viewport) -> Command<Message> {
        let viewport = PorterViewport::from_viewport(viewport);
        let size_of_item = ROW_HEIGHT + ROW_PADDING;

        let offsets = viewport.absolute_offset();
        let scroll_top = offsets.y;

        let item_start = (scroll_top / size_of_item).floor() as usize;
        let item_end = (item_start + ROW_OVERSCAN).min(self.asset_manager.len());

        self.item_range = item_start..item_end;

        self.scroll_viewport_state = viewport;

        scrollable::scroll_to(
            self.scroll_header_id.clone(),
            AbsoluteOffset {
                x: offsets.x,
                y: 0.0,
            },
        )
    }

    pub fn on_scroll_resize(&mut self, viewport: Option<Rectangle>) -> Command<Message> {
        let viewport = match viewport {
            Some(viewport) => viewport,
            None => return Command::none(),
        };

        let size_of_item = ROW_HEIGHT + ROW_PADDING;

        let offsets = self.scroll_viewport_state.absolute_offset();
        let scroll_top = offsets.y - (viewport.height - self.scroll_viewport_state.bounds.height);

        self.scroll_viewport_size = viewport;

        let item_start = (scroll_top / size_of_item).floor() as usize;
        let item_end = (item_start + ROW_OVERSCAN).min(self.asset_manager.len());

        self.item_range = item_start..item_end;

        scrollable::scroll_to(
            self.scroll_id.clone(),
            AbsoluteOffset {
                x: offsets.x,
                y: scroll_top,
            },
        )
    }

    pub fn on_preview(
        &mut self,
        asset: Option<PorterPreviewAsset>,
        request_id: u64,
    ) -> Command<Message> {
        if request_id != self.preview_request_id {
            return Command::none();
        }

        if let Some(previewer) = &mut self.previewer {
            if let Some(asset) = asset {
                match asset {
                    PorterPreviewAsset::Model(name, model, materials) => {
                        previewer.set_preview(name, (model, materials));
                    }
                    PorterPreviewAsset::Image(name, image) => {
                        previewer.set_preview(name, image);
                    }
                    PorterPreviewAsset::Material(name, images) => {
                        previewer.set_preview(name, images);
                    }
                }
            }
        }

        Command::none()
    }

    pub fn on_preview_resize(&mut self, viewport: Option<Rectangle>) -> Command<Message> {
        if let Some(viewport) = viewport {
            self.preview_viewport_size = viewport;

            if let Some(previewer) = &mut self.previewer {
                previewer.resize(
                    viewport.width,
                    viewport.height,
                    self.settings.far_clip() as f32,
                );
            }
        }

        Command::none()
    }

    pub fn on_close_preview(&mut self) -> Command<Message> {
        self.previewer = None;

        Command::none()
    }

    pub fn on_close_splash(&mut self) -> Command<Message> {
        if let Some(splash_id) = self.splash_id.take() {
            Command::batch([
                iced::window::close(splash_id),
                iced::window::change_mode(iced::window::Id::MAIN, window::Mode::Windowed),
            ])
        } else {
            Command::none()
        }
    }

    pub fn on_update_splash(&mut self, splash_animation: f32) -> Command<Message> {
        self.splash_animation = splash_animation;

        Command::none()
    }

    pub fn on_sync(&mut self, exporting: bool, progress: u32) -> Command<Message> {
        self.exporting = exporting;
        self.export_progress = progress;

        self.check_reload_required();

        Command::none()
    }

    pub fn on_row_press(&mut self, index: usize) -> Command<Message> {
        self.row_press = Some(index);

        Command::none()
    }

    pub fn on_row_release(&mut self, index: usize) -> Command<Message> {
        if let Some(press_index) = self.row_press.take() {
            if press_index == index {
                if self.row_press_last.elapsed() < DOUBLE_CLICK_DURATION && !self.exporting {
                    self.export_asset(index);
                } else if self.keyboard_modifiers.command() {
                    if self.item_selection.contains(&index) {
                        self.item_selection.remove(&index);
                    } else {
                        self.item_selection.insert(index);
                    }
                } else if self.keyboard_modifiers.shift() {
                    if let Some(first) = self.item_selection.first() {
                        match index.cmp(first) {
                            Ordering::Less => {
                                for i in index..*first {
                                    self.item_selection.insert(i);
                                }
                            }
                            Ordering::Greater => {
                                for i in *first..=index {
                                    self.item_selection.insert(i);
                                }
                            }
                            Ordering::Equal => {
                                self.item_selection.insert(index);
                            }
                        }
                    } else if self.item_selection.contains(&index) {
                        self.item_selection.remove(&index);
                    } else {
                        self.item_selection.insert(index);
                    }
                } else {
                    self.item_selection.clear();
                    self.item_selection.insert(index);
                    self.request_preview_asset();

                    self.row_press_last = Instant::now();
                }
            }
        }

        Command::none()
    }

    pub fn on_load_file(&mut self) -> Command<Message> {
        let mut file_dialog = FileDialog::new();

        for filter in &self.file_filters {
            file_dialog = file_dialog.add_filter(&filter.0, &filter.1);
        }

        let multi_file = self.multi_file;

        let Some(channel) = self.channel.clone() else {
            return Command::none();
        };

        iced::window::run_with_handle(iced::window::Id::MAIN, move |handle| {
            let file_dialog = file_dialog.set_parent(handle);

            let dialog = move || {
                if multi_file {
                    if let Some(files) = file_dialog.pick_files() {
                        let _ = channel.unbounded_send(Message::LoadFiles(files));
                    }
                } else if let Some(file) = file_dialog.pick_file() {
                    let _ = channel.unbounded_send(Message::LoadFiles(vec![file]));
                }
            };

            #[cfg(target_os = "windows")]
            std::thread::spawn(dialog);

            #[cfg(not(target_os = "windows"))]
            dialog();

            Message::Noop
        })
    }

    pub fn on_load_files(&mut self, files: Vec<PathBuf>) -> Command<Message> {
        self.load_files(files);

        Command::none()
    }

    pub fn on_load_file_dropped(&mut self) -> Command<Message> {
        if self.exporting || self.loading {
            return Command::none();
        }

        // We need to only take the files which match a filter in the application.
        // Then load those files only, to prevent issues with the asset manager.
        let mut target_extension: Option<String> = None;

        'outer: for filter in &self.file_filters {
            for file in &self.file_dropped {
                if let Some(extension) = file.extension() {
                    let extension = extension.to_string_lossy().to_string();

                    if filter.1.contains(&extension) {
                        target_extension = Some(extension);
                        break 'outer;
                    }
                }
            }
        }

        let Some(target_extension) = target_extension else {
            self.file_dropped.clear();
            self.file_dropped.shrink_to_fit();

            return Command::none();
        };

        let files_to_load: Vec<_> = self
            .file_dropped
            .drain(..)
            .filter(|file| {
                let Some(extension) = file.extension() else {
                    return false;
                };

                target_extension == extension.to_string_lossy()
            })
            .collect();

        if !files_to_load.is_empty() {
            self.load_files(files_to_load);
        }

        self.file_dropped.shrink_to_fit();

        Command::none()
    }

    pub fn on_load_game(&mut self) -> Command<Message> {
        self.load_game();

        Command::none()
    }

    pub fn on_load_result(&mut self, result: Result<(), String>) -> Command<Message> {
        self.loading = false;

        self.search_value = String::new();
        self.item_selection.clear();

        self.asset_manager.search_assets(None);

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        self.check_reload_required();

        if let Err(e) = result {
            let title = self.name.to_titlecase();

            Command::batch([
                iced::window::run_with_handle(iced::window::Id::MAIN, move |handle| {
                    let dialog = MessageDialog::new()
                        .set_title(title)
                        .set_description(e)
                        .set_level(MessageLevel::Warning)
                        .set_buttons(MessageButtons::Ok)
                        .set_parent(handle);

                    let dialog = move || {
                        dialog.show();
                    };

                    #[cfg(target_os = "windows")]
                    std::thread::spawn(dialog);

                    #[cfg(not(target_os = "windows"))]
                    dialog();

                    Message::Noop
                }),
                scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 }),
            ])
        } else {
            scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 })
        }
    }

    pub fn on_search_input(&mut self, input: String) -> Command<Message> {
        self.search_value = input;

        if self.asset_manager.loaded_len() > SEARCH_REALTIME_MAX && !self.search_value.is_empty() {
            Command::none()
        } else {
            self.on_search_submit()
        }
    }

    pub fn on_search_clear(&mut self) -> Command<Message> {
        self.search_value = String::new();
        self.item_selection.clear();

        self.asset_manager.search_assets(None);

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 })
    }

    pub fn on_search_submit(&mut self) -> Command<Message> {
        self.item_selection.clear();

        let search = PorterSearch::compile(self.search_value.clone());

        self.asset_manager.search_assets(Some(search));

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 })
    }

    pub fn on_cancel_export(&mut self) -> Command<Message> {
        self.export_cancel = true;

        self.asset_manager.cancel_export();

        Command::none()
    }

    pub fn on_donate(&mut self) -> Command<Message> {
        crate::open_url(PORTER_DONATE_URL);

        Command::none()
    }

    pub fn on_website(&mut self) -> Command<Message> {
        crate::open_url(PORTER_SITE_URL);

        Command::none()
    }

    pub fn on_toggle_settings(&mut self) -> Command<Message> {
        self.show_about = false;
        self.show_settings = !self.show_settings;

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        if !self.show_settings {
            self.check_reload_required();

            Command::batch([
                container::visible_bounds(self.scroll_container_id.clone())
                    .map(Message::ScrollResize),
                container::visible_bounds(self.previewer_container_id.clone())
                    .map(Message::PreviewResize),
            ])
        } else {
            Command::none()
        }
    }

    pub fn on_toggle_about(&mut self) -> Command<Message> {
        self.show_settings = false;
        self.show_about = !self.show_about;

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        if !self.show_about {
            Command::batch([
                container::visible_bounds(self.scroll_container_id.clone())
                    .map(Message::ScrollResize),
                container::visible_bounds(self.previewer_container_id.clone())
                    .map(Message::PreviewResize),
            ])
        } else {
            Command::none()
        }
    }

    pub fn on_export_selected(&mut self) -> Command<Message> {
        self.export_selected();

        Command::none()
    }

    pub fn on_export_all(&mut self) -> Command<Message> {
        self.export_all();

        Command::none()
    }

    pub fn on_save_settings(&mut self, settings: PorterSettings) -> Command<Message> {
        if !self.reload_required {
            self.reload_required = self.settings.reload_required(&settings);
        }

        self.settings = settings;
        self.settings.save(self.name);

        Command::none()
    }

    pub fn on_open_config_folder(&mut self) -> Command<Message> {
        let Some(project_directory) = ProjectDirs::from("com", "DTZxPorter", "GameTools") else {
            return Command::none();
        };

        open_folder(project_directory.config_dir());

        Command::none()
    }

    pub fn on_pick_export_folder(&mut self) -> Command<Message> {
        let settings = self.settings.clone();

        iced::window::run_with_handle(iced::window::Id::MAIN, move |handle| {
            let rfd = FileDialog::new()
                .set_directory(settings.output_directory())
                .set_parent(handle)
                .pick_folder();

            if let Some(path) = rfd {
                Message::SaveExportFolder(path)
            } else {
                Message::Noop
            }
        })
    }

    pub fn on_open_export_folder(&mut self) -> Command<Message> {
        open_folder(self.settings.output_directory());

        Command::none()
    }

    pub fn on_save_export_folder(&mut self, path: PathBuf) -> Command<Message> {
        self.settings.set_output_directory(path);
        self.settings.save(self.name);

        Command::none()
    }

    pub fn on_column_drag(&mut self, index: usize, offset: f32) -> Command<Message> {
        if let Some(column) = self.columns.get_mut(index) {
            column.width += offset;
        }

        Command::none()
    }

    pub fn on_column_drag_end(&mut self, index: usize) -> Command<Message> {
        if let Some(column) = self.columns.get_mut(index) {
            column.width = column.width.clamp(COLUMN_MIN, COLUMN_MAX);
        }

        Command::none()
    }

    pub fn on_noop(&mut self) -> Command<Message> {
        Command::none()
    }
}
