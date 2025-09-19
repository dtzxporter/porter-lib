use wgpu::*;

use porter_gpu::GPUInstance;

use porter_model::MaterialTextureRefUsage;

use porter_texture::Image;
use porter_texture::ImageFormat;

use crate::PreviewError;
use crate::RenderImage;

/// A 3d render material.
pub struct RenderMaterial {
    images: Vec<(Option<RenderImage>, ImageFormat, MaterialTextureRefUsage)>,
    index: usize,
}

impl RenderMaterial {
    /// Constructs a new render material from the images.
    pub fn from_images(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        images: &[(MaterialTextureRefUsage, Image)],
    ) -> Result<Self, PreviewError> {
        let mut images: Vec<(Option<RenderImage>, ImageFormat, MaterialTextureRefUsage)> = images
            .iter()
            .map(|(usage, image)| {
                let format = image.format();
                let image = RenderImage::from_image(instance, bind_group_layouts, image);
                let usage = *usage;

                (image.ok(), format, usage)
            })
            .collect();

        images.sort_by_key(|(image, _, usage)| {
            let usage = *usage;
            let bounds = (
                u32::MAX
                    - image
                        .as_ref()
                        .map(|image| image.width())
                        .unwrap_or_default(),
                u32::MAX
                    - image
                        .as_ref()
                        .map(|image| image.height())
                        .unwrap_or_default(),
            );

            (usage, bounds)
        });

        Ok(Self { images, index: 0 })
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        if self.images.is_empty() {
            return 0;
        }

        self.images
            .get(self.index)
            .and_then(|entry| entry.0.as_ref())
            .map(|image| image.width())
            .unwrap_or_default()
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        if self.images.is_empty() {
            return 0;
        }

        self.images
            .get(self.index)
            .and_then(|entry| entry.0.as_ref())
            .map(|image| image.height())
            .unwrap_or_default()
    }

    /// Returns whether or not the image is in sRGB colorspace.
    pub fn srgb(&self) -> bool {
        if self.images.is_empty() {
            return false;
        }

        self.images
            .get(self.index)
            .map(|entry| entry.1.is_srgb())
            .unwrap_or_default()
    }

    /// Returns the usage of the image.
    pub fn usage(&self) -> String {
        if self.images.is_empty() {
            return MaterialTextureRefUsage::Unknown.to_string();
        }

        self.images[self.index].2.to_string()
    }

    /// The current material image index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// The number of images in this material.
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Whether or not the material is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether or not the current material image had an error.
    pub fn is_error(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        self.images
            .get(self.index)
            .map(|entry| entry.0.is_none())
            .unwrap_or(false)
    }

    /// Advances to the next image.
    pub fn next(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.index + 1 >= self.len() {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    /// Draws the material using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        if self.images.is_empty() {
            return;
        }

        if let Some((Some(image), _, _)) = self.images.get(self.index) {
            image.draw(render_pass);
        }
    }
}
