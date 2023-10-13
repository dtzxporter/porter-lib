use std::path::PathBuf;

use crate::Message;
use crate::PorterMain;
use crate::PorterUI;
use crate::PorterViewport;

impl PorterMain {
    pub fn request_preview_asset(&mut self) {
        if self.previewer.is_none() {
            return;
        }

        if let Some(index) = self.item_selection.first().cloned() {
            if !self.asset_manager.is_empty() {
                let manager = self.asset_manager.clone();
                let channel = self.channel.clone();
                let settings = self.settings.clone();
                let request_id = self.preview_request_id.wrapping_add(1);

                self.preview_request_id += 1;

                porter_threads::spawn(move || {
                    manager.on_preview(settings, index, request_id, PorterUI::new(channel));
                });
            }
        }
    }

    pub fn export_asset(&mut self, index: usize) {
        if self.exporting {
            return;
        }

        let manager = self.asset_manager.clone();
        let channel = self.channel.clone();
        let settings = self.settings.clone();

        self.exporting = true;
        self.export_progress = 0;

        porter_threads::spawn(move || {
            manager.on_export(settings, vec![index], PorterUI::new(channel));
        });
    }

    pub fn export_selected(&mut self) {
        if self.exporting {
            return;
        }

        if self.item_selection.is_empty() {
            return;
        }

        let manager = self.asset_manager.clone();
        let channel = self.channel.clone();
        let settings = self.settings.clone();
        let assets: Vec<usize> = self.item_selection.iter().copied().collect();

        self.exporting = true;
        self.export_progress = 0;

        porter_threads::spawn(move || {
            manager.on_export(settings, assets, PorterUI::new(channel));
        });
    }

    pub fn export_all(&mut self) {
        if self.exporting {
            return;
        }

        let manager = self.asset_manager.clone();
        let channel = self.channel.clone();
        let settings = self.settings.clone();
        let assets: Vec<usize> = (0..self.asset_manager.len()).collect();

        self.exporting = true;
        self.export_progress = 0;

        porter_threads::spawn(move || {
            manager.on_export(settings, assets, PorterUI::new(channel));
        });
    }

    pub fn load_game(&mut self) {
        let manager = self.asset_manager.clone();
        let channel = self.channel.clone();
        let settings = self.settings.clone();

        self.loading = true;

        self.item_range = 0..0;
        self.item_selection.clear();
        self.scroll_viewport_state = PorterViewport::zero();

        self.last_load = Some(Vec::new());

        porter_threads::spawn(move || {
            let result = manager.on_load_game(settings);

            if let Some(channel) = channel {
                let result = channel.unbounded_send(Message::LoadResult(result));

                debug_assert!(result.is_ok());
            }
        });
    }

    pub fn load_files(&mut self, files: Vec<PathBuf>) {
        let manager = self.asset_manager.clone();
        let channel = self.channel.clone();
        let settings = self.settings.clone();

        self.loading = true;

        self.item_range = 0..0;
        self.item_selection.clear();
        self.scroll_viewport_state = PorterViewport::zero();

        self.last_load = Some(files.clone());

        porter_threads::spawn(move || {
            let result = manager.on_load_files(settings, files);

            if let Some(channel) = channel {
                let result = channel.unbounded_send(Message::LoadResult(result));

                debug_assert!(result.is_ok());
            }
        });
    }

    pub fn check_reload_required(&mut self) {
        if self.reload_required && !self.exporting && !self.loading && !self.show_settings {
            self.reload_required = false;

            if let Some(last_load) = self.last_load.take() {
                if last_load.is_empty() {
                    self.load_game();
                } else {
                    self.load_files(last_load);
                }
            }
        }
    }
}
