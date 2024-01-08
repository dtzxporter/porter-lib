use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use iced::Color;

/// Represents the status of an asset.
#[repr(transparent)]
#[derive(Debug)]
pub struct PorterAssetStatus {
    status: AtomicUsize,
}

impl PorterAssetStatus {
    /// Asset was loaded.
    pub fn loaded() -> Self {
        Self {
            status: AtomicUsize::new(0),
        }
    }

    /// Whether or not the status is loaded.
    pub fn is_loaded(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 0
    }

    /// Asset was exported.
    pub fn exported() -> Self {
        Self {
            status: AtomicUsize::new(1),
        }
    }

    /// Whether or not the status is exported.
    pub fn is_exported(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 1
    }

    /// Asset had an error.
    pub fn error() -> Self {
        Self {
            status: AtomicUsize::new(2),
        }
    }

    /// Whether or not the status is error.
    pub fn is_error(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 2
    }

    /// Asset is a placeholder.
    pub fn placeholder() -> Self {
        Self {
            status: AtomicUsize::new(3),
        }
    }

    /// Whether or not the status is placeholder.
    pub fn is_placeholder(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 3
    }

    /// Asset is currently exporting.
    pub fn exporting() -> Self {
        Self {
            status: AtomicUsize::new(4),
        }
    }

    /// Whether or not the status is exporting.
    pub fn is_exporting(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 4
    }

    /// Sets the status.
    pub fn set(&self, status: Self) {
        self.status
            .store(status.status.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    /// Gets the color of this status.
    pub fn color(&self) -> Color {
        match self.status.load(Ordering::Relaxed) {
            0 => Color::from_rgb8(35, 206, 107),
            1 => Color::from_rgb8(33, 184, 235),
            2 => Color::from_rgb8(212, 175, 55),
            3 => Color::from_rgb8(236, 52, 202),
            4 => Color::from_rgb8(144, 122, 214),
            _ => unreachable!(),
        }
    }
}

impl ToString for PorterAssetStatus {
    fn to_string(&self) -> String {
        match self.status.load(Ordering::Relaxed) {
            0 => String::from("Loaded"),
            1 => String::from("Exported"),
            2 => String::from("Error"),
            3 => String::from("Placeholder"),
            4 => String::from("Exporting..."),
            _ => unreachable!(),
        }
    }
}
