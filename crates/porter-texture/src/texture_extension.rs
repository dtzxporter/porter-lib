use wgpu::AstcChannel;
use wgpu::TextureFormat;
use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

use porter_utils::AsAligned;

/// Extensions for the TextureFormat.
pub trait TextureExtensions {
    /// Size of the buffer for the given width and height.
    fn buffer_size(&self, width: u32, height: u32) -> u64;
    /// Size of the buffer for the given width and height aligned.
    fn buffer_size_aligned(&self, width: u32, height: u32) -> u64;
    /// Size of one row for the given width.
    fn bytes_per_row(&self, width: u32) -> u32;
    /// Returns true if the texture format is unorm, else it's snorm.
    fn is_unorm(&self) -> bool;
}

impl TextureExtensions for TextureFormat {
    fn buffer_size(&self, width: u32, height: u32) -> u64 {
        let height: u64 = height as u64;

        let block_dims = self.block_dimensions();

        let nbh = height + (block_dims.1 as u64 - 1) / block_dims.1 as u64;

        self.bytes_per_row(width) as u64 * nbh
    }

    fn buffer_size_aligned(&self, width: u32, height: u32) -> u64 {
        let height: u64 = height as u64;

        let block_dims = self.block_dimensions();

        let nbh = height + (block_dims.1 as u64 - 1) / block_dims.1 as u64;

        let bytes_per_row = self.bytes_per_row(width) as u64;

        bytes_per_row.as_aligned(COPY_BYTES_PER_ROW_ALIGNMENT as u64) * nbh
    }

    fn bytes_per_row(&self, width: u32) -> u32 {
        let block_size = self.block_copy_size(None).unwrap_or_default();
        let block_dims = self.block_dimensions();

        let nbw = (width + (block_dims.0 - 1)) / block_dims.0;

        block_size * nbw
    }

    fn is_unorm(&self) -> bool {
        matches!(
            self,
            TextureFormat::R8Unorm
                | TextureFormat::R16Unorm
                | TextureFormat::Rg8Unorm
                | TextureFormat::Rg16Unorm
                | TextureFormat::Rgba8Unorm
                | TextureFormat::Rgba8UnormSrgb
                | TextureFormat::Bgra8Unorm
                | TextureFormat::Bgra8UnormSrgb
                | TextureFormat::Rgb10a2Unorm
                | TextureFormat::Rgba16Unorm
                | TextureFormat::Depth16Unorm
                | TextureFormat::Bc1RgbaUnorm
                | TextureFormat::Bc1RgbaUnormSrgb
                | TextureFormat::Bc2RgbaUnorm
                | TextureFormat::Bc2RgbaUnormSrgb
                | TextureFormat::Bc3RgbaUnorm
                | TextureFormat::Bc3RgbaUnormSrgb
                | TextureFormat::Bc4RUnorm
                | TextureFormat::Bc5RgUnorm
                | TextureFormat::Bc6hRgbUfloat
                | TextureFormat::Bc7RgbaUnorm
                | TextureFormat::Bc7RgbaUnormSrgb
                | TextureFormat::Etc2Rgb8Unorm
                | TextureFormat::Etc2Rgb8UnormSrgb
                | TextureFormat::Etc2Rgb8A1Unorm
                | TextureFormat::Etc2Rgb8A1UnormSrgb
                | TextureFormat::Etc2Rgba8Unorm
                | TextureFormat::Etc2Rgba8UnormSrgb
                | TextureFormat::EacR11Unorm
                | TextureFormat::EacRg11Unorm
                | TextureFormat::Astc {
                    channel: AstcChannel::Unorm,
                    ..
                }
                | TextureFormat::Astc {
                    channel: AstcChannel::UnormSrgb,
                    ..
                }
        )
    }
}
