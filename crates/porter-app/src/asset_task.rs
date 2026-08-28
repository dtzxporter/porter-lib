use std::path::Path;
use std::path::PathBuf;

use porter_animation::Animation;

use porter_audio::Audio;
use porter_audio::AudioFormat;

use porter_model::Material;
use porter_model::Model;
use porter_model::ModelFileType;

use porter_texture::Image;
use porter_texture::ImageConvertOptions;
use porter_texture::Transform;

use porter_threads::IntoParallelIterator;
use porter_threads::ParallelIterator;

use porter_utils::PathExt;
use porter_utils::SanitizeExt;

use porter_world::World;
use porter_world::WorldType;

use crate::Asset;
use crate::AssetContext;
use crate::AssetState;
use crate::AssetTaskAnimationOptions;
use crate::AssetTaskError;
use crate::AssetTaskImageNormalMapMode;
use crate::AssetTaskImageOptions;
use crate::AssetTaskMaterialOptions;
use crate::AssetTaskModelOptions;
use crate::AssetTaskRawFileOptions;
use crate::AssetTaskWorldOptions;
use crate::ImageNormalMapProcessing;
use crate::ModelMaterialProcessing;

/// Asset tasks that are shared across all tools.
pub trait AssetTask<A: Asset> {
    /// Asset export task for exporting an image.
    fn export_image<O>(
        &self,
        asset: &O,
        image: Image,
        options: AssetTaskImageOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset;
    /// Asset export task for exporting a model.
    fn export_model<O>(
        &self,
        asset: &O,
        model: Model,
        options: AssetTaskModelOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset;
    /// Asset export task for exporting model materials.
    fn export_model_material<O, C, E>(
        &self,
        asset: &O,
        options: AssetTaskMaterialOptions,
        callback: C,
    ) -> Material
    where
        O: Asset,
        C: FnOnce(&O, Option<PathBuf>, bool) -> Result<Material, E>;
    /// Asset export task for exporting an animation.
    fn export_animation<O>(
        &self,
        asset: &O,
        animation: Animation,
        options: AssetTaskAnimationOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset;
    /// Asset export task for exporting a sound.
    fn export_sound<O>(&self, asset: &O, audio: Audio) -> Result<(), AssetTaskError>
    where
        O: Asset;
    /// Asset export task for exporting a world.
    fn export_world<O>(
        &self,
        asset: &O,
        world: World,
        options: AssetTaskWorldOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset;
    /// Asset output path for exporting a model.
    fn output_model_path<O>(&self, asset: &O) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset;
    /// Asset output path for exporting a material.
    fn output_material_path<O>(
        &self,
        asset: &O,
        options: AssetTaskMaterialOptions,
    ) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset;
    /// Asset output path for exporting an animation set.
    fn output_animation_set_path<O>(&self, asset: &O) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset;
    /// Asset output path for exporting a raw file, includes the file name component as well.
    fn output_raw_file_path<O>(
        &self,
        asset: &O,
        options: AssetTaskRawFileOptions,
    ) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset;
    /// Asset relative instance path for exporting world assets.
    fn relative_instance_path<O>(&self, asset: &O) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset;
}

impl<A, S> AssetTask<A> for AssetContext<A, S>
where
    A: Asset,
    S: AssetState<A>,
{
    #[inline(never)]
    fn export_image<O>(
        &self,
        asset: &O,
        mut image: Image,
        options: AssetTaskImageOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let file_name = if let Some(name_override) = options.name_override {
            name_override.sanitized()
        } else {
            path.file_name()
                // Must have a file name in order to export anything.
                .ok_or(AssetTaskError::FileNameMissing)?
                .to_string_lossy()
                .into_owned()
        };

        let output_directory = options
            .override_path
            .unwrap_or_else(|| {
                self.settings
                    .output_directory()
                    .join("images")
                    .join(path.parent().unwrap_or(Path::new("")))
            });

        std::fs::create_dir_all(&output_directory)?;

        let image_options = match (
            options.normal_map_mode,
            self.settings
                .image_normal_map_processing(),
        ) {
            (AssetTaskImageNormalMapMode::Ignored, _) | (_, ImageNormalMapProcessing::None) => {
                ImageConvertOptions::None
            }
            (AssetTaskImageNormalMapMode::OpenGlAuto, ImageNormalMapProcessing::OpenGl)
            | (AssetTaskImageNormalMapMode::DirectXAuto, ImageNormalMapProcessing::DirectX) => {
                ImageConvertOptions::AutoReconstructZ
            }
            (AssetTaskImageNormalMapMode::OpenGlAuto, ImageNormalMapProcessing::DirectX)
            | (AssetTaskImageNormalMapMode::DirectXAuto, ImageNormalMapProcessing::OpenGl) => {
                ImageConvertOptions::AutoReconstructZInvertY
            }
            (AssetTaskImageNormalMapMode::OpenGlForced, ImageNormalMapProcessing::OpenGl)
            | (AssetTaskImageNormalMapMode::DirectXForced, ImageNormalMapProcessing::DirectX) => {
                ImageConvertOptions::ReconstructZ
            }
            (AssetTaskImageNormalMapMode::OpenGlForced, ImageNormalMapProcessing::DirectX)
            | (AssetTaskImageNormalMapMode::DirectXForced, ImageNormalMapProcessing::OpenGl) => {
                ImageConvertOptions::ReconstructZInvertY
            }
        };

        if options.normal_map_transform {
            match image_options {
                ImageConvertOptions::ReconstructZ => image.transform(Transform::ReconstructZ)?,
                ImageConvertOptions::ReconstructZInvertY => {
                    image.transform(Transform::ReconstructZInvertY)?
                }
                ImageConvertOptions::None => {
                    // No transform is necessary.
                }
                ImageConvertOptions::AutoReconstructZ
                | ImageConvertOptions::AutoReconstructZInvertY
                | ImageConvertOptions::UniformScaleBias(_, _) => {
                    #[cfg(debug_assertions)]
                    println!("Invalid conversion option for transform: {image_options:?}");
                    return Err(AssetTaskError::InvalidOption);
                }
            }
        }

        image.convert(
            image.format_for_file_type(self.settings.image_file_type()),
            image_options,
        )?;

        image.save(
            if options.append_extension {
                output_directory
                    .join(file_name)
                    .with_added_extension(self.settings.image_file_type())
            } else {
                output_directory
                    .join(file_name)
                    .with_extension(self.settings.image_file_type())
            },
            self.settings.image_file_type(),
        )?;

        Ok(())
    }

    #[inline(never)]
    fn export_model<O>(
        &self,
        asset: &O,
        mut model: Model,
        options: AssetTaskModelOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset,
    {
        let file_name = if let Some(name_override) = options.name_override {
            name_override.sanitized()
        } else {
            Path::new(&asset.name())
                .sanitized()
                .file_name()
                // Must have a file name in order to export anything.
                .ok_or(AssetTaskError::FileNameMissing)?
                .to_string_lossy()
                .into_owned()
        };

        let output_path = options.output_path;

        if let Some(factor) = self
            .settings
            .auto_scale_factor(options.scale_default)
        {
            model.scale(factor);
        }

        let is_hair = !model.hairs.is_empty() && model.meshes.is_empty();

        self.settings
            .model_file_types()
            .into_par_iter()
            .try_for_each(|model_file_type| {
                // Only export hair only models with cast, because it's the only format that supports it.
                if is_hair && !matches!(model_file_type, ModelFileType::Cast) {
                    return Ok(());
                }

                model.save(output_path.join(&file_name), model_file_type)
            })?;

        Ok(())
    }

    #[inline(never)]
    fn export_model_material<O, C, E>(
        &self,
        asset: &O,
        options: AssetTaskMaterialOptions,
        callback: C,
    ) -> Material
    where
        O: Asset,
        C: FnOnce(&O, Option<PathBuf>, bool) -> Result<Material, E>,
    {
        let (output_path, extract) = match self
            .settings
            .model_material_processing()
        {
            ModelMaterialProcessing::Skip => (None, false),
            ModelMaterialProcessing::InModelFolder => {
                let material_path = options
                    .override_path
                    .as_ref()
                    // Set the images target path for model material files, if we want images at all.
                    .map(|x| x.join("_images"));

                (material_path, options.extract)
            }
            ModelMaterialProcessing::InMaterialFolder => {
                let material_options = AssetTaskMaterialOptions::new()
                    .with_override_path(None)
                    .with_extract(false);
                let material_path = self
                    .output_material_path(asset, material_options)
                    .ok();

                (material_path, options.extract)
            }
        };

        let mut material = callback(asset, output_path.clone(), extract)
            .unwrap_or_else(|_| Material::with_source_name("failed_to_export", asset.name()));

        if let Some(base_path) = options.override_path
            && let Some(output_path) = output_path
        {
            // We need to make this relative to this path, otherwise, we can't find them.
            for texture in &mut material.textures {
                texture.file_path = output_path
                    .join(texture.file_path.sanitized())
                    .with_added_extension(self.settings.image_file_type())
                    .relative_from(&base_path)
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
            }
        }

        material
    }

    #[inline(never)]
    fn export_animation<O>(
        &self,
        asset: &O,
        mut animation: Animation,
        options: AssetTaskAnimationOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let file_name = if let Some(name_override) = options.name_override {
            name_override.sanitized()
        } else {
            path.file_name()
                // Must have a file name in order to export anything.
                .ok_or(AssetTaskError::FileNameMissing)?
                .to_string_lossy()
                .into_owned()
        };

        let output_directory = options
            .override_path
            .unwrap_or_else(|| {
                self.settings
                    .output_directory()
                    .join("animations")
                    .join(path.parent().unwrap_or(Path::new("")))
            });

        std::fs::create_dir_all(&output_directory)?;

        if let Some(factor) = self
            .settings
            .auto_scale_factor(options.scale_default)
        {
            animation.scale(factor);
        }

        self.settings
            .anim_file_types()
            .into_par_iter()
            .try_for_each(|anim_file_type| {
                animation.save(output_directory.join(&file_name), anim_file_type)
            })?;

        Ok(())
    }

    #[inline(never)]
    fn export_sound<O>(&self, asset: &O, mut audio: Audio) -> Result<(), AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let file_name = path
            .file_name()
            // Must have a file name in order to export anything.
            .ok_or(AssetTaskError::FileNameMissing)?;

        let output_directory = self
            .settings
            .output_directory()
            .join("sounds")
            .join(path.parent().unwrap_or(Path::new("")));

        std::fs::create_dir_all(&output_directory)?;

        audio.convert(AudioFormat::IntegerPcm)?;

        self.settings
            .audio_file_types()
            .into_par_iter()
            .try_for_each(|audio_file_type| {
                audio.save(
                    output_directory
                        .join(file_name)
                        .with_extension(audio_file_type),
                    audio_file_type,
                )
            })?;

        Ok(())
    }

    #[inline(never)]
    fn export_world<O>(
        &self,
        asset: &O,
        mut world: World,
        options: AssetTaskWorldOptions,
    ) -> Result<(), AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let file_name = path
            .file_stem()
            // Must have a file name in order to export anything.
            .ok_or(AssetTaskError::FileNameMissing)?
            .to_string_lossy();

        let file_name = if let Some(suffix) = options.name_suffix {
            format!("{}_{}", file_name, suffix)
        } else {
            file_name.into_owned()
        };

        let base = match world.world_type {
            WorldType::World => "worlds",
            WorldType::WorldEntities => "world_entities",
            WorldType::WorldPrefab => "world_prefabs",
        };

        let output_directory = self
            .settings
            .output_directory()
            .join(base)
            .join(path.parent().unwrap_or(Path::new("")));

        std::fs::create_dir_all(&output_directory)?;

        let scene_root = self
            .settings
            .output_directory()
            .join("models")
            .relative_from(&output_directory)
            .map(|path| path.to_string_lossy().into_owned());

        if let Some(factor) = self
            .settings
            .auto_scale_factor(options.scale_default)
        {
            world.scale(factor);
        }

        world.scene_root = scene_root;
        world.save(output_directory.join(file_name))?;

        Ok(())
    }

    #[inline(never)]
    fn output_model_path<O>(&self, asset: &O) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let parent = path
            .parent()
            .filter(|x| !x.as_os_str().is_empty())
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(Path::new)
                    .unwrap_or(Path::new(""))
            });

        let output_directory = self
            .settings
            .output_directory()
            .join("models")
            // Ensure that we always have at least one directory in the models folder.
            .join(parent);

        std::fs::create_dir_all(&output_directory)?;

        Ok(output_directory)
    }

    #[inline(never)]
    fn output_material_path<O>(
        &self,
        asset: &O,
        options: AssetTaskMaterialOptions,
    ) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset,
    {
        let output_directory = options
            .override_path
            .unwrap_or_else(|| {
                self.settings
                    .output_directory()
                    .join("materials")
                    .join(asset.name())
                    .sanitized()
            });

        if options.extract {
            std::fs::create_dir_all(&output_directory)?;
        }

        Ok(output_directory)
    }

    #[inline(never)]
    fn output_animation_set_path<O>(&self, asset: &O) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let output_directory = self
            .settings
            .output_directory()
            .join("animations")
            .join(path);

        std::fs::create_dir_all(&output_directory)?;

        Ok(output_directory)
    }

    #[inline(never)]
    fn output_raw_file_path<O>(
        &self,
        asset: &O,
        options: AssetTaskRawFileOptions,
    ) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let file_path = self
            .settings
            .output_directory()
            .join(options.custom_name)
            .join(path);

        if options.create_directory
            && let Some(parent) = file_path.parent()
        {
            std::fs::create_dir_all(parent)?;
        }

        Ok(file_path)
    }

    #[inline(never)]
    fn relative_instance_path<O>(&self, asset: &O) -> Result<PathBuf, AssetTaskError>
    where
        O: Asset,
    {
        let path = Path::new(&asset.name()).sanitized();

        let file_name = path
            .file_name()
            // Must have a file name in order to export anything.
            .ok_or(AssetTaskError::FileNameMissing)?;

        let relative_path = path
            .parent()
            .filter(|x| !x.as_os_str().is_empty())
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(Path::new)
                    .unwrap_or(Path::new(""))
            })
            .join(file_name);

        Ok(relative_path)
    }
}
