use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Seek;
use std::io::Write;
use std::path::Path;

use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

use porter_utils::AsAligned;

use porter_math::Rect;

use crate::Frame;
use crate::GPUConverter;
use crate::ImageConvertOptions;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::ResizeAlgorithm;
use crate::TextureError;
use crate::TextureExtensions;
use crate::TransformAlgorithm;
use crate::image_file_type_dds;
use crate::image_file_type_png;
use crate::image_file_type_tga;
use crate::image_file_type_tiff;
use crate::software_swizzle_image;
use crate::software_unpack_image;

/// Represents an image or texture with 1-many frames.
#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    mipmaps: u32,
    format: ImageFormat,
    frames: Vec<Frame>,
}

impl Image {
    /// Creates a new image with the given base dimensions and image format.
    pub fn new(width: u32, height: u32, format: ImageFormat) -> Result<Self, TextureError> {
        if format == ImageFormat::Unknown {
            return Err(TextureError::InvalidImageFormat(format));
        }

        if width == 0 || height == 0 {
            return Err(TextureError::InvalidImageSize(width, height));
        }

        Ok(Self {
            width,
            height,
            mipmaps: 1,
            format,
            frames: Vec::new(),
        })
    }

    /// Creates a new image with the given base dimensions, mipmaps, and image format.
    pub fn with_mipmaps(
        width: u32,
        height: u32,
        mipmaps: u32,
        format: ImageFormat,
    ) -> Result<Self, TextureError> {
        if format == ImageFormat::Unknown {
            return Err(TextureError::InvalidImageFormat(format));
        }

        if width == 0 || height == 0 {
            return Err(TextureError::InvalidImageSize(width, height));
        }

        if mipmaps == 0 {
            return Err(TextureError::InvalidMipMaps(mipmaps));
        }

        Ok(Self {
            width,
            height,
            mipmaps,
            format,
            frames: Vec::new(),
        })
    }

    /// Creates a new 4x4 image from the given color.
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8, srgb: bool) -> Result<Self, TextureError> {
        let mut image = Image::new(
            4,
            4,
            if srgb {
                ImageFormat::R8G8B8A8UnormSrgb
            } else {
                ImageFormat::R8G8B8A8Unorm
            },
        )?;

        image
            .create_frame()?
            .buffer_mut()
            // 4x4 slice of rgba data.
            .copy_from_slice(&[
                r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a,
                r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a,
                r, g, b, a, r, g, b, a,
            ]);

        Ok(image)
    }

    /// Creates a new 4x4 image from the given floating point color.
    pub fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32, srgb: bool) -> Result<Self, TextureError> {
        let r = ((r * 255.0) as u32).clamp(0, 255) as u8;
        let g = ((g * 255.0) as u32).clamp(0, 255) as u8;
        let b = ((b * 255.0) as u32).clamp(0, 255) as u8;
        let a = ((a * 255.0) as u32).clamp(0, 255) as u8;

        Self::from_rgba(r, g, b, a, srgb)
    }

    /// Sets the format used by this image if the block sizes match.
    pub(crate) fn set_format(&mut self, format: ImageFormat) -> Result<(), TextureError> {
        let old_size = self.frame_size_with_mipmaps(self.width, self.height, self.mipmaps);
        let old_format = self.format;

        self.format = format;

        let new_size = self.frame_size_with_mipmaps(self.width, self.height, self.mipmaps);

        if new_size == old_size {
            return Ok(());
        }

        self.format = old_format;

        Err(TextureError::InvalidImageFormat(format))
    }

    /// Converts all frames of the image to the specified format.
    pub fn convert(
        &mut self,
        format: ImageFormat,
        options: ImageConvertOptions,
    ) -> Result<(), TextureError> {
        if self.format == format {
            return Ok(());
        }

        if format.is_compressed() {
            return Err(TextureError::UnsupportedImageFormat(format));
        }

        if self.format.is_unpack_required() {
            software_unpack_image(self)?;

            if self.format == format {
                return Ok(());
            }
        }

        if self.format.is_swizzled(format) {
            software_swizzle_image(self, format)?;

            self.format = format;

            return Ok(());
        }

        if self.format.is_int() {
            return Err(TextureError::UnsupportedImageFormat(format));
        }

        let source_format = self.format.to_wgpu()?;
        let target_format = format.to_wgpu()?;

        self.mipmaps = 1;

        let width = self.width;
        let height = self.height;

        for frame in self.frames_mut() {
            let block_dims = target_format.block_dimensions();

            let bytes_per_row = target_format.bytes_per_row(width) as usize;
            let size = target_format.buffer_size_aligned(width, height) as usize;

            let mut buffer = Vec::new();

            buffer
                .try_reserve_exact(size)
                .map_err(|_| TextureError::FrameAllocationFailed)?;

            buffer.resize(size, 0);

            let mut converter = GPUConverter::new(width, height, source_format, target_format);

            converter.set_options(options);
            converter.convert(frame.buffer(), &mut buffer)?;

            let truncated_size = target_format.buffer_size(width, height) as usize;

            if truncated_size != size {
                let nbh = height.div_ceil(block_dims.1);

                for y in 0..nbh {
                    let source = y as usize
                        * bytes_per_row.as_aligned(COPY_BYTES_PER_ROW_ALIGNMENT as usize);
                    let dest = y as usize * bytes_per_row;

                    buffer.copy_within(source..source + bytes_per_row, dest);
                }

                buffer.resize(truncated_size, 0);

                frame.replace_buffer(buffer);
            } else {
                frame.replace_buffer(buffer);
            }
        }

        self.format = format;

        Ok(())
    }

    /// Transforms the image using the given algorithm.
    pub fn transform(&mut self, algorithm: TransformAlgorithm) -> Result<(), TextureError> {
        algorithm.transform(self)?;
        Ok(())
    }

    /// Resizes the image to the new width/height. This will drop any mipmaps if they exist.
    /// The format must be 32bits per pixel with 4 components in any order.
    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        algorithm: ResizeAlgorithm,
    ) -> Result<(), TextureError> {
        if !self.format.is_resizable() {
            return Err(TextureError::UnsupportedImageFormat(self.format));
        }

        if width == 0 || height == 0 {
            return Err(TextureError::InvalidOperation);
        }

        algorithm.resize(self, width, height)?;

        Ok(())
    }

    /// Copies a rectangle from the given src image to the destination in this image,
    /// truncating the image as necessary on any edge. Both formats must be the same,
    /// and not a compressed image format.
    pub fn copy_rect(
        &mut self,
        src: &Self,
        src_rect: Rect,
        mut dest_x: i32,
        mut dest_y: i32,
    ) -> Result<(), TextureError> {
        if self.format != src.format {
            return Err(TextureError::UnsupportedImageFormat(self.format));
        }

        if self.format.is_compressed() || src.format.is_compressed() {
            return Err(TextureError::UnsupportedImageFormat(self.format));
        }

        if self.frames.len() != src.frames.len() {
            return Err(TextureError::InvalidOperation);
        }

        let Some(frame_src) = src.frames.first() else {
            return Err(TextureError::InvalidOperation);
        };

        let Some(frame_dest) = self.frames.first_mut() else {
            return Err(TextureError::InvalidOperation);
        };

        if src_rect.x > src.width() || src_rect.y > src.height() {
            return Err(TextureError::InvalidOperation);
        }

        let bits_per_pixel = src.format.bits_per_pixel();

        if bits_per_pixel < 8 {
            return Err(TextureError::UnsupportedImageFormat(src.format));
        }

        let bytes_per_pixel = bits_per_pixel.div_ceil(8);

        let mut src_x = src_rect.x as i32;
        let mut src_y = src_rect.y as i32;
        let mut src_width = src_rect.width as i32;
        let mut src_height = src_rect.height as i32;

        // Truncate the left region.
        if dest_x < 0 {
            src_width += dest_x;
            src_x -= dest_x;

            dest_x = 0;
        }

        // Truncate the top region.
        if dest_y < 0 {
            src_height += dest_y;
            src_y -= dest_y;

            dest_y = 0;
        }

        // Truncate the right region.
        if dest_x + src_width > self.width as i32 {
            src_width -= (dest_x + src_width) - self.width as i32;
        }

        // Truncate the bottom region.
        if dest_y + src_height > self.height as i32 {
            src_height -= (dest_y + src_height) - self.height as i32;
        }

        // Truncate the source width.
        if src_x + src_width > src.width() as i32 {
            src_width -= (src_x + src_width) - src.width() as i32;
        }

        // Truncate the source height.
        if src_y + src_height > src.height() as i32 {
            src_height -= (src_y + src_height) - src.height() as i32;
        }

        if src_width <= 0 || src_height <= 0 {
            return Ok(());
        }

        let src_bytes_per_row = src.width() * bytes_per_pixel;
        let src_copy_bytes = (src_width as u32 * bytes_per_pixel) as usize;

        let mut src_offset =
            ((src_y as u32 * src_bytes_per_row) + (src_x as u32 * bytes_per_pixel)) as usize;

        let dest_bytes_per_row = self.width * bytes_per_pixel;

        let mut dest_offset =
            ((dest_y as u32 * dest_bytes_per_row) + (dest_x as u32 * bytes_per_pixel)) as usize;

        for _ in 0..src_height {
            frame_dest.buffer_mut()[dest_offset..dest_offset + src_copy_bytes]
                .copy_from_slice(&frame_src.buffer()[src_offset..src_offset + src_copy_bytes]);

            src_offset += src_bytes_per_row as usize;
            dest_offset += dest_bytes_per_row as usize;
        }

        Ok(())
    }

    /// Calculates the optimal image format required to save this image to the given file type.
    pub fn format_for_file_type(&self, file_type: ImageFileType) -> ImageFormat {
        match file_type {
            ImageFileType::Dds => image_file_type_dds::pick_format(self.format),
            ImageFileType::Png => image_file_type_png::pick_format(self.format),
            ImageFileType::Tiff => image_file_type_tiff::pick_format(self.format),
            ImageFileType::Tga => image_file_type_tga::pick_format(self.format),
        }
    }

    /// Loads the image from the given path.
    pub fn load<P: AsRef<Path>>(path: P, file_type: ImageFileType) -> Result<Self, TextureError> {
        let input = File::open(path)?;
        let mut buffered = BufReader::new(input);

        Self::load_from(&mut buffered, file_type)
    }

    /// Loads the image from the given input buffer with the given file type.
    pub fn load_from<I: BufRead + Seek>(
        input: &mut I,
        file_type: ImageFileType,
    ) -> Result<Self, TextureError> {
        match file_type {
            ImageFileType::Dds => image_file_type_dds::from_dds(input),
            ImageFileType::Png => image_file_type_png::from_png(input),
            ImageFileType::Tiff => image_file_type_tiff::from_tiff(input),
            ImageFileType::Tga => image_file_type_tga::from_tga(input),
        }
    }

    /// Saves the image to the given file path in the given image file type.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: ImageFileType,
    ) -> Result<(), TextureError> {
        let output = File::create(path)?;
        let mut buffered = BufWriter::new(output);

        self.save_to(&mut buffered, file_type)?;

        buffered.flush()?;

        Ok(())
    }

    /// Saves the image to the given output buffer in the given image file type.
    pub fn save_to<O: Write + Seek>(
        &self,
        output: &mut O,
        file_type: ImageFileType,
    ) -> Result<(), TextureError> {
        match file_type {
            ImageFileType::Dds => image_file_type_dds::to_dds(self, output),
            ImageFileType::Png => image_file_type_png::to_png(self, output),
            ImageFileType::Tiff => image_file_type_tiff::to_tiff(self, output),
            ImageFileType::Tga => image_file_type_tga::to_tga(self, output),
        }
    }

    /// Returns the size of a new frame using the current image format and mipmaps.
    pub fn frame_size(&self, width: u32, height: u32) -> u32 {
        self.frame_size_with_mipmaps(width, height, self.mipmaps)
    }

    /// Returns the size of a new frame using the current image format and overriding the mipmaps.
    pub fn frame_size_with_mipmaps(&self, width: u32, height: u32, mipmaps: u32) -> u32 {
        let mut size: u32 = 0;
        let mut mip_width = width;
        let mut mip_height = height;

        for _ in 0..mipmaps {
            size += self.format.buffer_size(mip_width, mip_height);

            mip_width = if mip_width > 1 { mip_width / 2 } else { 1 };
            mip_height = if mip_height > 1 { mip_height / 2 } else { 1 };
        }

        size
    }

    /// Allocates and creates a new frame, using the current image format.
    pub fn create_frame(&mut self) -> Result<&mut Frame, TextureError> {
        let size = self.frame_size(self.width, self.height);

        let frame = Frame::new(size)?;

        self.frames
            .try_reserve(1)
            .map_err(|_| TextureError::FrameAllocationFailed)?;

        self.frames.push(frame);

        self.frames
            .last_mut()
            .ok_or(TextureError::FrameAllocationFailed)
    }

    /// Returns the base width of the image, all frames must be <= this width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the base height of the image, all frames must be <= this height.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the number of mipmaps in the image, all frames must have this many. (Default: 1)
    pub fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    /// Returns the image format used by all frames in this image.
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    /// The size in bytes of all the frames and mipmaps in this image.
    pub fn size(&self) -> usize {
        self.frames.iter().map(|x| x.buffer().len()).sum()
    }

    /// Returns an iterator over the frames of this image.
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    /// Returns an iterator that allows modifying the frames of this image.
    pub fn frames_mut(&mut self) -> &mut [Frame] {
        &mut self.frames
    }

    /// Image is considered a cubemap if it has exactly 6 frames.
    pub fn is_cubemap(&self) -> bool {
        self.frames.len() == 6
    }
}
