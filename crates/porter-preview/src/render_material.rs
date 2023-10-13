use wgpu::*;

use porter_gpu::GPUInstance;
use porter_model::MaterialTextureRefUsage;
use porter_texture::Image;

use crate::RenderImage;

/// A 3d render material.
pub struct RenderMaterial {
    images: Vec<(RenderImage, MaterialTextureRefUsage)>,
    index: usize,
}

impl RenderMaterial {
    /// Constructs a new render material from the images.
    pub fn from_images(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        images: &[(MaterialTextureRefUsage, Image)],
    ) -> Self {
        Self {
            images: images
                .iter()
                .map(|image| {
                    (
                        RenderImage::from_image(instance, bind_group_layouts, &image.1),
                        image.0,
                    )
                })
                .collect(),
            index: 0,
        }
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        if self.images.is_empty() {
            return 0;
        }

        self.images[self.index].0.width()
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        if self.images.is_empty() {
            return 0;
        }

        self.images[self.index].0.height()
    }

    /// Returns the usage of the image.
    pub fn usage(&self) -> String {
        if self.images.is_empty() {
            return MaterialTextureRefUsage::Unknown.to_string();
        }

        self.images[self.index].1.to_string()
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

        self.images[self.index].0.draw(render_pass);
    }
}
