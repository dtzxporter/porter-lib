use std::fs::File;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;

use porter_utils::BufferReadExt;
use porter_utils::BufferWriteExt;
use porter_utils::VecExt;
use porter_utils::VecReadExt;

use porter_math::Rect;

use crate::Address;
use crate::Filter;
use crate::Frame;
use crate::GPUConverter;
use crate::ImageConvertOptions;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::Pixel;
use crate::Resize;
use crate::TextureError;
use crate::Transform;
use crate::image_file_type_dds;
use crate::image_file_type_png;
use crate::image_file_type_pvr;
use crate::image_file_type_tga;
use crate::image_file_type_tiff;
use crate::pack_unorm8;
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

        image.create_frame()?.fill([r, g, b, a]);

        Ok(image)
    }

    /// Creates a new 4x4 image from the given floating point color.
    pub fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32, srgb: bool) -> Result<Self, TextureError> {
        let r = pack_unorm8(r);
        let g = pack_unorm8(g);
        let b = pack_unorm8(b);
        let a = pack_unorm8(a);

        Self::from_rgba(r, g, b, a, srgb)
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
            let mut converter = GPUConverter::new(width, height, source_format, target_format);

            converter.set_options(options);

            let buffer = converter.convert(frame.buffer())?;

            frame.replace_buffer(buffer);
        }

        self.format = format;

        Ok(())
    }

    /// Transforms the image using the given algorithm.
    pub fn transform(&mut self, algorithm: Transform) -> Result<(), TextureError> {
        algorithm.apply(self)?;
        Ok(())
    }

    /// Resizes the image to the new width/height. This will drop any mipmaps if they exist.
    /// The format must be 32bits per pixel with 4 components in any order.
    pub fn resize(&mut self, width: u32, height: u32, filter: Filter) -> Result<(), TextureError> {
        if self.width == width && self.height == height {
            return Ok(());
        }

        if !self.format.is_resizable() {
            return Err(TextureError::UnsupportedImageFormat(self.format));
        }

        if width == 0 || height == 0 {
            return Err(TextureError::InvalidOperation);
        }

        match filter {
            Filter::Nearest => Resize::NearestNeighbor.apply(self, width, height)?,
            Filter::Linear => Resize::Bicubic.apply(self, width, height)?,
        }

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

    /// Flips the image and it's frames vertically. This will drop any mipmaps if they exist.
    pub fn flip_vertical(&mut self) -> Result<(), TextureError> {
        if self.format.is_compressed() {
            return Err(TextureError::UnsupportedImageFormat(self.format));
        }

        let bytes_per_row = self.format.bytes_per_row(self.width);
        let buffer_size = self
            .format
            .buffer_size(self.width, self.height);

        let height = self.height;

        for frame in self.frames_mut() {
            let mut rows = frame
                .buffer_mut()
                .chunks_exact_mut(bytes_per_row as _)
                .take(height as _);

            while let (Some(top), Some(bottom)) = (rows.next(), rows.next_back()) {
                top.swap_with_slice(bottom);
            }

            frame.truncate_buffer(buffer_size as _);
        }

        self.mipmaps = 1;

        Ok(())
    }

    /// Flips the image and it's frames horizontally. This will drop any mipmaps if they exist.
    pub fn flip_horizontal(&mut self) -> Result<(), TextureError> {
        if self.format.is_compressed() {
            return Err(TextureError::UnsupportedImageFormat(self.format));
        }

        let bytes_per_row = self.format.bytes_per_row(self.width);
        let buffer_size = self
            .format
            .buffer_size(self.width, self.height);
        let pixel_size = self.format.bits_per_pixel().div_ceil(8);

        let height = self.height;

        for frame in self.frames_mut() {
            for row in frame
                .buffer_mut()
                .chunks_exact_mut(bytes_per_row as _)
                .take(height as _)
            {
                let mut pixels = row.chunks_exact_mut(pixel_size as _);

                while let (Some(left), Some(right)) = (pixels.next(), pixels.next_back()) {
                    left.swap_with_slice(right);
                }
            }

            frame.truncate_buffer(buffer_size as _);
        }

        self.mipmaps = 1;

        Ok(())
    }

    /// Calculates the optimal image format required to save this image to the given file type.
    pub fn format_for_file_type(&self, file_type: ImageFileType) -> ImageFormat {
        match file_type {
            ImageFileType::Dds => image_file_type_dds::pick_format(self.format),
            ImageFileType::Png => image_file_type_png::pick_format(self.format),
            ImageFileType::Tiff => image_file_type_tiff::pick_format(self.format),
            ImageFileType::Tga => image_file_type_tga::pick_format(self.format),
            ImageFileType::Pvr => image_file_type_pvr::pick_format(self.format),
        }
    }

    /// Loads the image from the given path.
    pub fn load<P: AsRef<Path>>(path: P, file_type: ImageFileType) -> Result<Self, TextureError> {
        Self::load_from(&mut File::open(path)?.buffer_read(), file_type)
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
            ImageFileType::Pvr => image_file_type_pvr::from_pvr(input),
        }
    }

    /// Saves the image to the given file path in the given image file type.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: ImageFileType,
    ) -> Result<(), TextureError> {
        let mut output = File::create(path)?.buffer_write();

        self.save_to(&mut output, file_type)?;

        output.flush()?;

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
            ImageFileType::Pvr => image_file_type_pvr::to_pvr(self, output),
        }
    }

    /// Returns the size of a new frame using the current image format, dimensions, and mipmaps.
    pub fn frame_size(&self) -> u32 {
        self.frame_size_with_mipmaps(self.width, self.height, self.mipmaps)
    }

    /// Returns the size of a new frame using the current image format and given `width`, `height`, and `mipmaps`.
    pub fn frame_size_with_mipmaps(&self, width: u32, height: u32, mipmaps: u32) -> u32 {
        let mut size: u32 = 0;
        let mut mip_width = width;
        let mut mip_height = height;

        for _ in 0..mipmaps {
            size += self
                .format
                .buffer_size(mip_width, mip_height);

            mip_width = if mip_width > 1 { mip_width / 2 } else { 1 };
            mip_height = if mip_height > 1 { mip_height / 2 } else { 1 };
        }

        size
    }

    /// Allocates and creates a new frame, using the current image format.
    pub fn create_frame(&mut self) -> Result<&mut Frame, TextureError> {
        let size = self.frame_size();

        Ok(self
            .frames
            .try_push_mut(Frame::new(size)?)?)
    }

    /// Reads a new frame from the given reader, using the current image format.
    pub fn read_frame<R: Read>(&mut self, read: &mut R) -> Result<&mut Frame, TextureError> {
        let size = self.frame_size();

        Ok(self
            .frames
            .try_push_mut(Frame::with_buffer(read.read_vec(size as _)?))?)
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
        self.frames
            .iter()
            .map(|x| x.buffer().len())
            .sum()
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

    /// Samples the image at the given coordinates with the provided filter.
    pub fn sample(&self, x: f32, y: f32, z: f32, address: Address, filter: Filter) -> Pixel {
        let (x, y) = match address {
            Address::Clamp => (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)),
            Address::Wrap => (x - x.floor(), y - y.floor()),
        };

        let w = self.width as f32;
        let h = self.height as f32;

        let x = x * (w - 1.0);
        let y = y * (h - 1.0);

        let bytes_per_pixel = self.format.bits_per_pixel().div_ceil(8) as usize;

        let Some(frame) = self
            .frames
            .get(z.round() as usize)
            .or_else(|| self.frames.first())
            .map(|x| x.buffer())
        else {
            return Pixel::TRANSPARENT;
        };

        match filter {
            Filter::Nearest => {
                let x = x.round() as usize;
                let y = y.round() as usize;

                let index = (y * self.width as usize + x) * bytes_per_pixel;

                Pixel::load(self.format, &frame[index..index + bytes_per_pixel])
            }
            Filter::Linear => {
                let x0 = x.floor() as usize;
                let y0 = y.floor() as usize;
                let x1 = (x0 + 1).min(self.width as usize - 1);
                let y1 = (y0 + 1).min(self.height as usize - 1);

                let tx = x - x0 as f32;
                let ty = y - y0 as f32;

                let index00 = (y0 * self.width as usize + x0) * bytes_per_pixel;
                let index10 = (y0 * self.width as usize + x1) * bytes_per_pixel;
                let index01 = (y1 * self.width as usize + x0) * bytes_per_pixel;
                let index11 = (y1 * self.width as usize + x1) * bytes_per_pixel;

                let c00 = Pixel::load(self.format, &frame[index00..index00 + bytes_per_pixel]);
                let c10 = Pixel::load(self.format, &frame[index10..index10 + bytes_per_pixel]);
                let c01 = Pixel::load(self.format, &frame[index01..index01 + bytes_per_pixel]);
                let c11 = Pixel::load(self.format, &frame[index11..index11 + bytes_per_pixel]);

                let c0 = c00.lerp(c10, tx);
                let c1 = c01.lerp(c11, tx);

                c0.lerp(c1, ty)
            }
        }
    }

    /// Sets the format used by this image if the block sizes match.
    pub(crate) fn set_format(&mut self, format: ImageFormat) -> Result<(), TextureError> {
        let old_size = self.frame_size();
        let old_format = self.format;

        self.format = format;

        let new_size = self.frame_size();

        if new_size == old_size {
            return Ok(());
        }

        self.format = old_format;

        Err(TextureError::InvalidImageFormat(format))
    }
}
