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
        let block_size = self.block_size(None).unwrap_or_default();
        let block_dims = self.block_dimensions();

        let nbw = (width + (block_dims.0 - 1)) / block_dims.0;

        block_size * nbw
    }
}
