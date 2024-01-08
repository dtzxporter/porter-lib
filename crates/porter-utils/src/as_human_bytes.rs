/// Utility trait to convert numbers to bytes.
pub trait AsHumanBytes {
    /// Converts a number to a human readable byte string.
    fn as_human_bytes(&self) -> String;
}

/// Helper to implement the routine for a generic number type.
macro_rules! impl_human_bytes {
    ($typ:ty) => {
        impl AsHumanBytes for $typ {
            fn as_human_bytes(&self) -> String {
                let size = *self as u64;

                const KB: u64 = 1024;
                const MB: u64 = KB * 1024;
                const GB: u64 = MB * 1024;
                const TB: u64 = GB * 1024;
                const PB: u64 = TB * 1024;

                let (size, format): (f32, &str) = if size < KB {
                    (size as f32, "B")
                } else if size < MB {
                    (size as f32 / KB as f32, "KB")
                } else if size < GB {
                    (size as f32 / MB as f32, "MB")
                } else if size < TB {
                    (size as f32 / GB as f32, "GB")
                } else if size < PB {
                    (size as f32 / TB as f32, "TB")
                } else {
                    (size as f32 / PB as f32, "PB")
                };

                format!("{:.2} {}", size, format)
            }
        }
    };
}

impl_human_bytes!(u8);
impl_human_bytes!(u16);
impl_human_bytes!(u32);
impl_human_bytes!(u64);
impl_human_bytes!(usize);
