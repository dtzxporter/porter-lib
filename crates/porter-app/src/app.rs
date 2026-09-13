use std::path::PathBuf;

use iced::window;

use iced::Element;
use iced::Event;
use iced::Subscription;
use iced::Task;

use iced::theme::Theme;
use iced::theme::palette::Seed;

use crate::AppState;
use crate::AssetPreview;
use crate::AssetPreviewRequest;
use crate::ColumnStatus;
use crate::Controller;
use crate::MainMessage;
use crate::MainWindow;
use crate::Message;
use crate::PreviewWindow;
use crate::PreviewWindowMessage;
use crate::SplashMessage;
use crate::SplashWindow;
use crate::components::ContentMessage;
use crate::components::HeaderMessage;
use crate::components::PreviewMessage;
use crate::components::SearchBarMessage;
use crate::components::SettingsMessage;
use crate::palette;
use crate::strings;

/// Entry point for the iced application.
pub struct App {
    state: AppState,
    main_window: MainWindow,
    splash_window: SplashWindow,
    preview_window: Option<PreviewWindow>,
}

impl App {
    /// Constructs a new app entry point.
    pub fn new(state: AppState) -> (Self, Task<Message>) {
        let (main_window, main_window_task) = MainWindow::create();
        let (splash_window, splash_window_task) = SplashWindow::create();

        let task = Task::batch([main_window_task, splash_window_task]).discard();

        let ui = Self {
            state,
            main_window,
            splash_window,
            preview_window: None,
        };

        (ui, task)
    }

    /// Provides the title for the given window.
    pub fn title(&self, id: window::Id) -> String {
        if id == self.main_window.id {
            self.main_window.title(&self.state)
        } else if id == self.splash_window.id {
            self.splash_window.title(&self.state)
        } else if let Some(preview_window) = &self.preview_window
            && preview_window.id == id
        {
            preview_window.title(&self.state)
        } else {
            String::new()
        }
    }

    /// Handles updating the app state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        use Message::*;

        match message {
            Noop => self.on_noop(),
            UI(event, id) => self.on_ui(event, id),
            WindowOpened(id) => self.on_window_opened(id),
            Controller(controller) => self.on_controller(controller),
            Splash(message) => self
                .splash_window
                .update(&mut self.state, message),
            Main(message) => self
                .main_window
                .update(&mut self.state, message),
            PreviewProxy(message) => self.on_preview_proxy(message),
            PreviewWindow(message) => self
                .preview_window
                .as_mut()
                .map(|window| window.update(&mut self.state, message))
                .unwrap_or(Task::none()),
            PreviewWindowCreate => self.on_preview_window_create(),
            PreviewWindowClosed => self.on_preview_window_closed(),
            PreviewToggle => self.on_preview_toggle(),
            PreviewRequest => self.on_preview_request(),
            PreviewAsset(index, raw) => self.on_preview_asset(index, raw),
            LoadUpdate(result, files) => self.on_load_update(result, files),
            ProgressUpdate(finished, progress) => self.on_progress_update(finished, progress),
            PreviewUpdate(request_id, asset) => self.on_preview_update(request_id, asset),
            ExportOne(index) => self.on_export_one(index),
            ExportSelected => self.on_export_selected(),
            ExportAll => self.on_export_all(),
            ExportCancel => self.on_export_cancel(),
            CopyColumn(index, column_index) => self.on_copy_column(index, column_index),
            LoadFiles(files) => self.on_load_files(files),
            LoadFilesDropped => self.on_load_files_dropped(),
            LoadLastFiles => self.on_load_last_files(),
            LoadGame => self.on_load_game(),
            Sort(index) => self.on_sort(index),
            CheckReload => self.on_check_reload(),
        }
    }

    /// Custom theme defaults.
    pub fn theme(&self, _: window::Id) -> Theme {
        Theme::custom(
            String::from("Porter"),
            Seed {
                background: palette::BACKGROUND_COLOR_DEFAULT,
                text: palette::TEXT_COLOR_DEFAULT,
                primary: palette::PRIMARY_COLOR,
                ..Seed::LIGHT
            },
        )
    }

    /// Handles global and controller events.
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::event;
        use iced::event::Status;
        use iced::stream;

        use iced::futures::SinkExt;
        use iced::futures::StreamExt;
        use iced::futures::channel::mpsc;

        /// Filters out events that aren't necessary for the global listener.
        #[inline(always)]
        fn filter_event(event: Event, id: window::Id) -> Option<Message> {
            // Whenever we need to listen to a new global event, add it here.
            // This prevents thrashing the main message queue with unnecessary events.
            if matches!(event, Event::Keyboard(_))
                || matches!(event, Event::Window(window::Event::Closed))
                || matches!(event, Event::Window(window::Event::Opened { .. }))
                || matches!(event, Event::Window(window::Event::FileDropped(_)))
            {
                return Some(Message::UI(event, id));
            }

            None
        }

        let events = event::listen_with(|event, status, id| match status {
            Status::Ignored => filter_event(event, id),
            Status::Captured => None,
        });

        let controller = Subscription::run(|| {
            stream::channel(100, |mut output: mpsc::Sender<Message>| async move {
                let (tx, mut rx) = mpsc::unbounded::<Message>();

                output
                    .send(Message::Controller(Controller::with_channel(tx)))
                    .await
                    .expect("Failed to initialize controller!");

                loop {
                    while let Some(message) = rx.next().await {
                        let result = output.send(message).await;

                        debug_assert!(result.is_ok());
                    }
                }
            })
        });

        Subscription::batch([events, controller])
    }

    /// Handles rendering a given window.
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if id == self.main_window.id {
            self.main_window.view(&self.state)
        } else if id == self.splash_window.id {
            self.splash_window.view(&self.state)
        } else if let Some(preview_window) = &self.preview_window
            && preview_window.id == id
        {
            preview_window.view(&self.state)
        } else {
            iced::widget::space().into()
        }
    }

    /// Occurs when nothing should happen.
    fn on_noop(&mut self) -> Task<Message> {
        Task::none()
    }

    /// Occurs when a ui event has triggered for a given window.
    fn on_ui(&mut self, event: Event, id: window::Id) -> Task<Message> {
        if id == self.main_window.id {
            self.main_window
                .update(&mut self.state, MainMessage::UI(event))
        } else if id == self.splash_window.id {
            self.splash_window
                .update(&mut self.state, SplashMessage::UI(event))
        } else if let Some(preview_window) = &mut self.preview_window
            && preview_window.id == id
        {
            preview_window.update(&mut self.state, PreviewWindowMessage::UI(event))
        } else {
            Task::none()
        }
    }

    /// Occurs when a window opens.
    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::*;
            use windows_sys::Win32::UI::WindowsAndMessaging::*;

            use raw_window_handle::RawWindowHandle;

            use crate::icon_windows;

            iced::window::run(id, |handle| {
                let icon = icon_windows::windows_icon();

                if let Ok(RawWindowHandle::Win32(handle)) = handle
                    .window_handle()
                    .map(|x| x.as_raw())
                {
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
        {
            let _ = id;

            Task::none()
        }
    }

    /// Occurs when the global controller is initialized.
    fn on_controller(&mut self, controller: Controller) -> Task<Message> {
        self.state.controller = controller;

        Task::none()
    }

    /// Occurs when progress has been made by the asset manager.
    fn on_progress_update(&mut self, finished: bool, progress: u32) -> Task<Message> {
        if finished {
            self.state.loading = false;
            self.state.exporting = false;
            self.state.export_canceled = false;
            self.state.progress = 0;

            return self.on_check_reload();
        } else {
            self.state.progress = progress.clamp(0, 100);
        }

        Task::none()
    }

    /// Occurs when a preview request has been completed by the asset manager.
    fn on_preview_update(&mut self, request_id: u64, asset: AssetPreview) -> Task<Message> {
        match self.state.asset_preview {
            None => Task::none(),
            Some(request) if request.request_id() != request_id => {
                if self.state.asset_manager.assets_empty() {
                    self.state.asset_preview = None;
                    return Task::none();
                }

                let manager = self.state.asset_manager.clone();
                let controller = self.state.controller.clone();
                let settings = self.state.settings.clone();

                let index = request.index();
                let raw = request.raw();
                let request_id = request.request_id();

                porter_threads::spawn(move || {
                    manager.preview(settings, controller, index, raw, request_id);
                });

                Task::none()
            }
            Some(_) => {
                self.state.asset_preview = None;

                self.on_preview_proxy(PreviewMessage::Update(asset))
            }
        }
    }

    /// Occurs when a load request has been completed by the asset manager.
    fn on_load_update(
        &mut self,
        result: Result<(), String>,
        files: Option<Vec<PathBuf>>,
    ) -> Task<Message> {
        self.state.loading = false;
        self.state.progress = 0;
        self.state.reset_item_range();

        let icon = self.state.asset_manager.display_icon();

        if let Err(e) = result {
            self.state.last_load = None;

            Task::batch([
                Task::done(Message::from(MainMessage::Warning(e))),
                Task::done(Message::from(HeaderMessage::UpdateIcon(icon))),
            ])
        } else {
            Task::batch(
                [
                    Task::done(Message::from(SearchBarMessage::Submit)),
                    Task::done(Message::from(HeaderMessage::UpdateIcon(icon))),
                    Task::done(Message::Sort(None)),
                    self.on_check_reload(),
                ]
                .into_iter()
                .chain(files.map(|files| {
                    let settings = self
                        .state
                        .settings
                        .update(|settings| settings.set_last_files(Some(files)));

                    Task::done(Message::from(SettingsMessage::Save(settings)))
                })),
            )
        }
    }

    /// Occurs when a preview message needs to be proxied to the specific window it exists on.
    fn on_preview_proxy(&mut self, message: PreviewMessage) -> Task<Message> {
        if let Some(preview_window) = &mut self.preview_window {
            use ContentMessage::*;
            use PreviewWindowMessage::*;

            preview_window.update(&mut self.state, Content(Preview(message)))
        } else {
            use ContentMessage::*;
            use MainMessage::*;

            self.main_window
                .update(&mut self.state, Content(Preview(message)))
        }
    }

    /// Occurs when the user wants to expand the preview to a new window.
    fn on_preview_window_create(&mut self) -> Task<Message> {
        let (preview_window, preview_window_task) = PreviewWindow::create();

        self.preview_window = Some(preview_window);

        Task::batch([
            preview_window_task.discard(),
            Task::done(Message::PreviewRequest),
        ])
    }

    /// Occurs when the user closes the preview window.
    fn on_preview_window_closed(&mut self) -> Task<Message> {
        self.preview_window = None;

        Task::none()
    }

    /// Occurs when the user wants to toggle the preview window.
    fn on_preview_toggle(&mut self) -> Task<Message> {
        if self.preview_window.is_some() {
            return Task::none();
        }

        if self.state.settings.preview_window() {
            self.on_preview_window_create()
        } else {
            Task::done(Message::from(ContentMessage::PreviewToggle))
        }
    }

    /// Occurs when we need to request to preview the selected asset.
    fn on_preview_request(&mut self) -> Task<Message> {
        let Some(index) = self
            .state
            .assets_selected
            .first()
            .cloned()
        else {
            return Task::none();
        };

        self.on_preview_asset_request(index, self.state.modifier_keys.alt())
    }

    /// Occurs when we need to request to preview a specific asset.
    fn on_preview_asset(&mut self, index: usize, raw: bool) -> Task<Message> {
        if self.state.settings.preview_window() && self.preview_window.is_none() {
            let (preview_window, preview_window_task) = PreviewWindow::create();

            self.preview_window = Some(preview_window);

            Task::batch([
                preview_window_task.discard(),
                self.on_preview_asset_request(index, raw),
            ])
        } else if self.preview_window.is_some() {
            self.on_preview_asset_request(index, raw)
        } else {
            Task::batch([
                Task::done(Message::from(ContentMessage::PreviewOpen)),
                self.on_preview_asset_request(index, raw),
            ])
        }
    }

    /// Occurs when we need to request a preview asset from the asset manager.
    fn on_preview_asset_request(&mut self, index: usize, raw: bool) -> Task<Message> {
        if self.state.asset_manager.assets_empty() {
            return Task::none();
        }

        match &mut self.state.asset_preview {
            None => {
                let request = AssetPreviewRequest::new(index, raw);
                let request_id = request.request_id();

                let manager = self.state.asset_manager.clone();
                let controller = self.state.controller.clone();
                let settings = self.state.settings.clone();

                porter_threads::spawn(move || {
                    manager.preview(settings, controller, index, raw, request_id);
                });

                self.state.asset_preview = Some(request);
            }
            Some(request) => {
                request.next_request(index, raw);
            }
        }

        Task::none()
    }

    /// Occurs when the user requests to export one asset.
    fn on_export_one(&mut self, index: usize) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        self.on_export(vec![index])
    }

    /// Occurs when the user requests to export selected assets.
    fn on_export_selected(&mut self) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let assets: Vec<usize> = self
            .state
            .assets_selected
            .iter()
            .copied()
            .collect();

        self.on_export(assets)
    }

    /// Occurs when the user requests to export all assets.
    fn on_export_all(&mut self) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let assets: Vec<usize> = (0..self
            .state
            .asset_manager
            .assets_visible())
            .collect();

        self.on_export(assets)
    }

    /// Occurs when the user requests to cancel an export.
    fn on_export_cancel(&mut self) -> Task<Message> {
        if self.state.export_canceled {
            return Task::none();
        }

        self.state.export_canceled = true;
        self.state.asset_manager.export_cancel();

        Task::none()
    }

    /// Occurs when the user wants to export the provided assets.
    fn on_export(&mut self, assets: Vec<usize>) -> Task<Message> {
        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();

        self.state.exporting = true;
        self.state.export_canceled = false;
        self.state.progress = 0;

        porter_threads::spawn(move || {
            manager.export(settings, controller, assets);
        });

        Task::none()
    }

    /// Occurs when the user requests to copy the value of an asset column.
    fn on_copy_column(&mut self, index: usize, column_index: usize) -> Task<Message> {
        if self.state.loading {
            return Task::none();
        }

        let Some(value) = self
            .state
            .asset_manager
            .assets_info(index)
            .into_iter()
            .nth(column_index)
        else {
            return Task::none();
        };

        iced::clipboard::write(value.0).discard()
    }

    /// Occurs when the user requests to load some files.
    fn on_load_files(&mut self, files: Vec<PathBuf>) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();

        self.state.loading = true;
        self.state.progress = 0;
        self.state.last_load = Some(files.clone());
        self.state.assets_selected.clear();

        porter_threads::spawn(move || {
            controller.load_update(manager.load_files(settings, files.clone()), Some(files));
        });

        Task::none()
    }

    /// Occurs when the last file has been received from the drop queue.
    fn on_load_files_dropped(&mut self) -> Task<Message> {
        let mut files = std::mem::take(&mut self.state.files_dropped);

        // We need to only take the files which match a filter in the application.
        // Then load those files only, to prevent issues with the asset manager.
        let mut target_extension: Option<String> = None;

        'outer: for filter in &self.state.file_filters {
            for file in &files {
                if let Some(extension) = file.extension() {
                    let extension = extension.to_string_lossy().into_owned();

                    if filter.1.iter().any(|&x| x == extension) {
                        target_extension = Some(extension);
                        break 'outer;
                    }
                }
            }
        }

        let Some(target_extension) = target_extension else {
            return Task::none();
        };

        let files_to_load: Vec<_> = files
            .drain(..)
            .filter(|file| {
                let Some(extension) = file.extension() else {
                    return false;
                };

                target_extension == extension.to_string_lossy()
            })
            .take(if cfg!(feature = "multi-file") {
                usize::MAX
            } else {
                1
            })
            .collect();

        if !files_to_load.is_empty() {
            return self.on_load_files(files_to_load);
        }

        Task::none()
    }

    /// Occurs when the user requests to load the last loaded files.
    fn on_load_last_files(&mut self) -> Task<Message> {
        let Some(files) = self.state.settings.last_files() else {
            return Task::none();
        };

        let missing = files.iter().any(|file| !file.exists());

        if missing {
            let settings = self
                .state
                .settings
                .update(|settings| settings.set_last_files(None));

            return Task::batch([
                Task::done(Message::from(SettingsMessage::Save(settings))),
                Task::done(Message::from(MainMessage::Warning(
                    if cfg!(feature = "multi-file") {
                        String::from(strings::ONE_OR_MORE_MISSING)
                    } else {
                        String::from(strings::ONE_MISSING)
                    },
                ))),
            ]);
        }

        self.on_load_files(files)
    }

    /// Occurs when the user requests to load a game.
    fn on_load_game(&mut self) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let manager = self.state.asset_manager.clone();
        let controller = self.state.controller.clone();
        let settings = self.state.settings.clone();

        self.state.loading = true;
        self.state.progress = 0;
        self.state.last_load = Some(Vec::new());
        self.state.assets_selected.clear();

        porter_threads::spawn(move || {
            controller.load_update(manager.load_game(settings), None);
        });

        Task::none()
    }

    /// Occurs when assets should be sorted, or a column has changed.
    fn on_sort(&mut self, index: Option<usize>) -> Task<Message> {
        if self.state.is_busy() {
            return Task::none();
        }

        let statuses: Vec<_> = self
            .state
            .asset_columns
            .iter()
            .enumerate()
            .map(|(index, column)| ColumnStatus::new(index, column.sort.unwrap_or_default()))
            .collect();

        for status in self
            .state
            .asset_manager
            .sort(index, statuses)
        {
            if let Some(column) = self
                .state
                .asset_columns
                .get_mut(status.index)
                && column.sort.is_some()
            {
                column.sort = Some(status.sort);
            }
        }

        Task::none()
    }

    /// Occurs when we need to check if a reload is required.
    fn on_check_reload(&mut self) -> Task<Message> {
        if !self.state.reload_required
            || self.state.is_busy()
            || self.main_window.header.show_settings
        {
            return Task::none();
        }

        self.state.reload_required = false;

        if let Some(last_load) = self.state.last_load.take() {
            if last_load.is_empty() {
                return self.on_load_game();
            } else {
                return self.on_load_files(last_load);
            }
        }

        Task::none()
    }
}
