use std::cmp::Ordering;
use std::path::PathBuf;
use std::time::Instant;

use iced::futures::channel::mpsc::UnboundedSender;

use iced::keyboard;
use iced::keyboard::KeyCode;
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

use crate::Message;
use crate::PorterMain;
use crate::PorterPreviewAsset;
use crate::PorterSetParentWindows;
use crate::PorterSettings;
use crate::PorterViewport;
use crate::PreviewControlScheme;

use crate::COLUMN_MAX;
use crate::COLUMN_MIN;
use crate::DOUBLE_CLICK_DURATION;
use crate::ROW_HEIGHT;
use crate::ROW_OVERSCAN;
use crate::ROW_PADDING;
use crate::SEARCH_REALTIME_MAX;

impl PorterMain {
    pub fn on_ui_event(&mut self, event: Event) -> Command<Message> {
        match event {
            Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {
                self.on_key_released(key_code)
            }
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
            Event::Window(_, window::Event::FileDropped(file)) => self.on_file_dropped(file),
            _ => Command::none(),
        }
    }

    pub fn on_key_released(&mut self, key_code: KeyCode) -> Command<Message> {
        if key_code == KeyCode::E {
            self.export_selected();
        } else if key_code == KeyCode::Up {
            if let Some(index) = self.item_selection.first().cloned() {
                if index > 0 && self.item_selection.len() == 1 {
                    self.item_selection.clear();
                    self.item_selection.insert(index - 1);
                    self.request_preview_asset();
                }
            }
        } else if key_code == KeyCode::Down {
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
        } else if key_code == KeyCode::P {
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
        } else if key_code == KeyCode::R {
            if let Some(previewer) = &mut self.previewer {
                previewer.reset_view();
            }
        } else if key_code == KeyCode::B {
            if let Some(previewer) = &mut self.previewer {
                previewer.toggle_bones();
            }
        } else if key_code == KeyCode::W {
            if let Some(previewer) = &mut self.previewer {
                previewer.toggle_wireframe();
            }
        } else if key_code == KeyCode::M {
            if let Some(previewer) = &mut self.previewer {
                previewer.toggle_shaded();
            }
        } else if key_code == KeyCode::N {
            if let Some(previewer) = &mut self.previewer {
                previewer.cycle_material();
            }
        } else if key_code == KeyCode::F && self.keyboard_modifiers.command() {
            return Command::batch([
                text_input::focus(self.search_id.clone()),
                text_input::select_all(self.search_id.clone()),
            ]);
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

    pub fn on_file_dropped(&mut self, file: PathBuf) -> Command<Message> {
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

        Command::none()
    }

    pub fn on_scroll_resize(&mut self, viewport: Option<Rectangle>) -> Command<Message> {
        let viewport = match viewport {
            Some(viewport) => viewport,
            None => return Command::none(),
        };

        let size_of_item = ROW_HEIGHT + ROW_PADDING;

        let scroll_top = self.scroll_viewport_state.absolute_offset().y
            - (viewport.height - self.scroll_viewport_state.bounds.height);

        self.scroll_viewport_size = viewport;

        let item_start = (scroll_top / size_of_item).floor() as usize;
        let item_end = (item_start + ROW_OVERSCAN).min(self.asset_manager.len());

        self.item_range = item_start..item_end;

        scrollable::scroll_to(
            self.scroll_id.clone(),
            AbsoluteOffset {
                x: 0.0,
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
                previewer.resize(viewport.width, viewport.height);
            }
        }

        Command::none()
    }

    pub fn on_close_preview(&mut self) -> Command<Message> {
        self.previewer = None;

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
        let mut file_dialog = FileDialog::new().set_parent_windows();

        for filter in &self.file_filters {
            file_dialog = file_dialog.add_filter(&filter.0, &filter.1);
        }

        if self.multi_file {
            if let Some(files) = file_dialog.pick_files() {
                self.load_files(files);
            }
        } else if let Some(file) = file_dialog.pick_file() {
            self.load_files(vec![file]);
        }

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
        if let Err(e) = result {
            MessageDialog::new()
                .set_title(&self.title)
                .set_description(e)
                .set_level(MessageLevel::Warning)
                .set_buttons(MessageButtons::Ok)
                .set_parent_windows()
                .show();
        }

        self.loading = false;

        self.search_value = String::new();
        self.item_selection.clear();

        self.asset_manager.search_assets(String::new());

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        self.check_reload_required();

        scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 })
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

        self.asset_manager.search_assets(String::new());

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 })
    }

    pub fn on_search_submit(&mut self) -> Command<Message> {
        self.item_selection.clear();

        self.asset_manager.search_assets(self.search_value.clone());

        self.item_range = 0..ROW_OVERSCAN.min(self.asset_manager.len());
        self.scroll_viewport_state = PorterViewport::zero();

        scrollable::scroll_to(self.scroll_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 })
    }

    pub fn on_cancel_export(&mut self) -> Command<Message> {
        self.asset_manager.cancel_export();

        Command::none()
    }

    pub fn on_donate(&mut self) -> Command<Message> {
        crate::open_url(&self.donate_url);

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
        self.settings.save(self.name.clone());

        Command::none()
    }

    pub fn on_open_config_folder(&mut self) -> Command<Message> {
        let Some(project_directory) = ProjectDirs::from("com", "DTZxPorter", "GameTools") else {
            return Command::none();
        };

        let dirs = std::fs::create_dir_all(project_directory.config_dir());

        debug_assert!(dirs.is_ok());

        let config_folder = project_directory.config_dir().to_string_lossy().to_string();

        let mut command = std::process::Command::new(if cfg!(target_os = "windows") {
            "explorer.exe"
        } else if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        });

        command.arg(if cfg!(target_os = "windows") {
            String::from("/start,")
        } else {
            config_folder.clone()
        });

        if cfg!(target_os = "windows") {
            command.arg(config_folder);
        }

        let result = command.output();

        debug_assert!(result.is_ok());

        Command::none()
    }

    pub fn on_pick_export_folder(&mut self) -> Command<Message> {
        let rfd = FileDialog::new()
            .set_directory(self.settings.output_directory())
            .set_parent_windows()
            .pick_folder();

        if let Some(path) = rfd {
            self.settings.set_output_directory(path);
            self.settings.save(self.name.clone());
        }

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
            column.width = column.width.min(COLUMN_MAX).max(COLUMN_MIN);
        }

        Command::none()
    }

    pub fn on_noop(&mut self) -> Command<Message> {
        Command::none()
    }
}
