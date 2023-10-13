use iced::futures::channel::mpsc::UnboundedSender;

use std::sync::Arc;

use crate::Message;
use crate::PorterPreviewAsset;

/// Used to syncronize with the ui.
#[derive(Clone)]
pub struct PorterUI {
    channel: Arc<Option<UnboundedSender<Message>>>,
}

impl PorterUI {
    /// Constructs a new instance of the ui.
    pub fn new(channel: Option<UnboundedSender<Message>>) -> Self {
        Self {
            channel: Arc::new(channel),
        }
    }

    /// Syncs the ui with the current export progress.
    pub fn sync(&self, exporting: bool, progress: u32) {
        if let Some(channel) = self.channel.as_ref() {
            let result = channel.unbounded_send(Message::Sync(exporting, progress));

            debug_assert!(result.is_ok());
        }
    }

    /// Reports a preview asset is ready.
    pub fn preview(&self, asset: Option<PorterPreviewAsset>, request_id: u64) {
        if let Some(channel) = self.channel.as_ref() {
            let result = channel.unbounded_send(Message::Preview(asset, request_id));

            debug_assert!(result.is_ok());
        }
    }
}
