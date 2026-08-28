use std::path::PathBuf;

use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::UnboundedSender;

use crate::AssetPreview;
use crate::Message;

/// Control the app from anywhere.
#[derive(Debug, Clone)]
pub struct Controller {
    channel: UnboundedSender<Message>,
}

impl Controller {
    /// Constructs a new controller with an empty channel.
    pub fn new() -> Self {
        let (tx, _) = mpsc::unbounded();

        Self { channel: tx }
    }

    /// Constructs a new controller with the given channel.
    pub fn with_channel(channel: UnboundedSender<Message>) -> Self {
        Self { channel }
    }

    /// Requests the given files be loaded by the app.
    pub fn load_files(&self, files: Vec<PathBuf>) {
        let result = self
            .channel
            .unbounded_send(Message::LoadFiles(files));

        debug_assert!(result.is_ok());
    }

    /// Requests that dropped files be loaded.
    pub fn load_files_dropped(&self) {
        let result = self
            .channel
            .unbounded_send(Message::LoadFilesDropped);

        debug_assert!(result.is_ok());
    }

    /// Notifies the app of a load result.
    pub fn load_update(&self, result: Result<(), String>, files: Option<Vec<PathBuf>>) {
        let result = self
            .channel
            .unbounded_send(Message::LoadUpdate(result, files));

        debug_assert!(result.is_ok());
    }

    /// Notifies the app of progress being made during an operation.
    pub fn progress_update(&self, progress: u32) {
        let result = self
            .channel
            .unbounded_send(Message::ProgressUpdate(false, progress));

        debug_assert!(result.is_ok());
    }

    /// Notifies the app of progress being finished during an operation.
    pub fn progress_finish(&self) {
        let result = self
            .channel
            .unbounded_send(Message::ProgressUpdate(true, 100));

        debug_assert!(result.is_ok());
    }

    /// Notifies the app of a preview asset being ready.
    pub fn preview_update(&self, request_id: u64, asset: AssetPreview) {
        let result = self
            .channel
            .unbounded_send(Message::PreviewUpdate(request_id, asset));

        debug_assert!(result.is_ok());
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
