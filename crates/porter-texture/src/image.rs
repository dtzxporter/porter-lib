use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

use porter_utils::AsAligned;

use crate::format_to_bpp;
use crate::format_to_wgpu;
use crate::image_file_type_dds;
use crate::image_file_type_png;
use crate::image_file_type_tiff;
use crate::is_format_compressed;
use crate::Frame;
use crate::GPUConverter;
use crate::ImageConvertOptions;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::TextureError;

use std::io::BufWriter;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use std::slice::Iter;
use std::slice::IterMut;

/// Represents an image or texture with 1-many frames.
#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
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
            format,
            frames: Vec::new(),
        })
    }

    /// Converts all frames of the image to the specified format.
    pub fn convert(
        &mut self,
        format: ImageFormat,
        options: Option<ImageConvertOptions>,
    ) -> Result<(), TextureError> {
        if self.format == format {
            return Ok(());
        }

        if is_format_compressed(format) {
            return Err(TextureError::UnsupportedImageFormat(format));
        }

        let source_format = format_to_wgpu(self.format)?;
        let target_format = format_to_wgpu(format)?;

        for frame in self.frames_mut() {
            let block_size = target_format.block_size(None).unwrap_or_default();
            let block_dims = target_format.block_dimensions();

            let bytes_per_row =
                block_size as usize * (frame.width() as usize / block_dims.0 as usize);
            let size = bytes_per_row.as_aligned(COPY_BYTES_PER_ROW_ALIGNMENT as usize)
                * (frame.height() as usize / block_dims.1 as usize);

            let mut buffer = Vec::new();

            buffer
                .try_reserve(size)
                .map_err(|_| TextureError::FrameAllocationFailed)?;

            buffer.resize(size, 0);

            let mut converter =
                GPUConverter::new(frame.width(), frame.height(), source_format, target_format);

            converter.set_options(options);
            converter.convert(frame.buffer(), &mut buffer)?;

            let truncated_size = bytes_per_row * (frame.height() as usize / block_dims.1 as usize);

            if truncated_size != size {
                for y in 0..frame.height() / block_dims.1 {
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

    /// Calculates the optimal image format required to save this image to the given file type.
    pub fn format_for_file_type(&self, file_type: ImageFileType) -> ImageFormat {
        match file_type {
            ImageFileType::Dds => image_file_type_dds::pick_format(self.format),
            ImageFileType::Png => image_file_type_png::pick_format(self.format),
            ImageFileType::Tiff => image_file_type_tiff::pick_format(self.format),
        }
    }

    /// Saves the image to the given file path in the given image file type.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: ImageFileType,
    ) -> Result<(), TextureError> {
        let output = std::fs::File::create(path)?;
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
        }
    }

    /// Allocates and creates a new frame, using the current image format.
    pub fn create_frame(&mut self, width: u32, height: u32) -> Result<&mut Frame, TextureError> {
        if width > self.width || height > self.height || width == 0 || height == 0 {
            return Err(TextureError::InvalidFrameSize(width, height));
        }

        let bits_per_pixel = format_to_bpp(self.format);
        let size = (width * height * bits_per_pixel) / 8;

        let frame = Frame::new(width, height, size)?;

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

    /// Returns the image format used by all frames in this image.
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    /// The size in bytes of all the frames in this image.
    pub fn size(&self) -> usize {
        self.frames.iter().map(|x| x.buffer().len()).sum()
    }

    /// Returns an iterator over the frames of this image.
    pub fn frames(&self) -> Iter<'_, Frame> {
        self.frames.iter()
    }

    /// Returns an iterator that allows modifying the frames of this image.
    pub fn frames_mut(&mut self) -> IterMut<'_, Frame> {
        self.frames.iter_mut()
    }

    /// Image is considered a cubemap if it has exactly 6 frames, all of which are the same size.
    pub fn is_cubemap(&self) -> bool {
        self.frames().len() == 6
            && self
                .frames()
                .all(|x| x.width() == self.width && x.height() == self.height)
    }
}
