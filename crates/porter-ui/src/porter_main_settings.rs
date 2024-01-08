use iced::widget::*;

use iced::Alignment;
use iced::Element;
use iced::Length;

use porter_animation::AnimationFileType;
use porter_audio::AudioFileType;
use porter_model::ModelFileType;
use porter_texture::ImageFileType;

use crate::ImageNormalMapProcessing;
use crate::Message;
use crate::PorterButtonStyle;
use crate::PorterCheckboxStyle;
use crate::PorterLabelStyle;
use crate::PorterMain;
use crate::PorterPickListStyle;
use crate::PorterScrollStyle;
use crate::PorterSettings;
use crate::PorterTextInputStyle;
use crate::PreviewControlScheme;

impl PorterMain {
    /// Constructs the settings view.
    pub fn settings(&self) -> Element<Message> {
        let model_formats = self.settings.model_file_types();
        let model_format_enabled =
            |format: ModelFileType| model_formats.iter().any(|f| *f == format);

        let anim_formats = self.settings.anim_file_types();
        let anim_format_enabled =
            |format: AnimationFileType| anim_formats.iter().any(|f| *f == format);

        let audio_formats = self.settings.audio_file_types();
        let audio_format_enabled =
            |format: AudioFileType| audio_formats.iter().any(|f| *f == format);

        let mut settings = vec![
            text("Settings - General")
                .size(20.0)
                .style(PorterLabelStyle)
                .into(),
            vertical_space(2.0).into(),
            text("Choose what asset types to load and display:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            checkbox("Load Models", self.settings.load_models(), |value| {
                Message::SaveSettings(
                    self.settings
                        .update(|settings| settings.set_load_models(value)),
                )
            })
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "Load Animations",
                self.settings.load_animations(),
                |value| {
                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_load_animations(value)),
                    )
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox("Load Images", self.settings.load_images(), |value| {
                Message::SaveSettings(
                    self.settings
                        .update(|settings| settings.set_load_images(value)),
                )
            })
            .style(PorterCheckboxStyle)
            .into(),
        ];

        if self.materials_enabled {
            settings.push(
                checkbox("Load Materials", self.settings.load_materials(), |value| {
                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_load_materials(value)),
                    )
                })
                .style(PorterCheckboxStyle)
                .into(),
            );
        }

        if self.sounds_enabled {
            settings.push(
                checkbox("Load Sounds", self.settings.load_sounds(), |value| {
                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_load_sounds(value)),
                    )
                })
                .style(PorterCheckboxStyle)
                .into(),
            );
        }

        if self.raw_files_enabled {
            settings.push(
                checkbox("Load Raw Files", self.settings.load_raw_files(), |value| {
                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_load_raw_files(value)),
                    )
                })
                .style(PorterCheckboxStyle)
                .into(),
            );
        }

        settings.extend([
            vertical_space(2.0).into(),
            text("Customize the exported files directory:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            row(vec![
                text_input(
                    "Exported files directory",
                    self.settings.output_directory().to_string_lossy().as_ref(),
                )
                .on_input(|_| Message::Noop)
                .width(500.0)
                .style(PorterTextInputStyle)
                .into(),
                button("Browse")
                    .on_press(Message::PickExportFolder)
                    .style(PorterButtonStyle)
                    .into(),
            ])
            .spacing(4.0)
            .into(),
            vertical_space(2.0).into(),
            text("Choose whether or not to automatically scale assets (Recommended):")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            checkbox(
                "Automatically scale assets",
                self.settings.auto_scale(),
                |value| {
                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_auto_scale(value)),
                    )
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            vertical_space(4.0).into(),
            text("Settings - Models")
                .size(20.0)
                .style(PorterLabelStyle)
                .into(),
            vertical_space(2.0).into(),
            text("Choose what model file types to export to:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            checkbox("Cast", model_format_enabled(ModelFileType::Cast), |value| {
                Message::SaveSettings(
                    self.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::Cast, value)
                    }),
                )
            })
            .style(PorterCheckboxStyle)
            .into(),
            checkbox("OBJ", model_format_enabled(ModelFileType::Obj), |value| {
                Message::SaveSettings(
                    self.settings
                        .update(|settings| settings.set_model_file_type(ModelFileType::Obj, value)),
                )
            })
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "Valve SMD",
                model_format_enabled(ModelFileType::Smd),
                |value| {
                    Message::SaveSettings(
                        self.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Smd, value)
                        }),
                    )
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "XNALara",
                model_format_enabled(ModelFileType::XnaLara),
                |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::XnaLara, value)
                    }))
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "CoD XModel",
                model_format_enabled(ModelFileType::XModelExport),
                |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::XModelExport, value)
                    }))
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "SEModel",
                model_format_enabled(ModelFileType::SEModel),
                |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::SEModel, value)
                    }))
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "Autodesk Maya",
                model_format_enabled(ModelFileType::Maya),
                |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::Maya, value)
                    }))
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox("FBX", model_format_enabled(ModelFileType::Fbx), |value| {
                Message::SaveSettings(
                    self.settings
                        .update(|settings| settings.set_model_file_type(ModelFileType::Fbx, value)),
                )
            })
            .style(PorterCheckboxStyle)
            .into(),
            vertical_space(4.0).into(),
            text("Settings - Images")
                .size(20.0)
                .style(PorterLabelStyle)
                .into(),
            vertical_space(2.0).into(),
            text("Choose what image file type to export to:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            pick_list(
                vec!["DDS", "PNG", "TIFF"],
                match self.settings.image_file_type() {
                    ImageFileType::Dds => Some("DDS"),
                    ImageFileType::Png => Some("PNG"),
                    ImageFileType::Tiff => Some("TIFF"),
                },
                |selected| {
                    let format = match selected {
                        "DDS" => ImageFileType::Dds,
                        "PNG" => ImageFileType::Png,
                        "TIFF" => ImageFileType::Tiff,
                        _ => ImageFileType::Dds,
                    };

                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_image_file_type(format)),
                    )
                },
            )
            .style(PorterPickListStyle)
            .width(Length::Fixed(150.0))
            .into(),
            vertical_space(2.0).into(),
            text("Choose a normal map conversion method:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            pick_list(
                vec!["None", "OpenGL", "DirectX"],
                match self.settings.image_normal_map_processing() {
                    ImageNormalMapProcessing::None => Some("None"),
                    ImageNormalMapProcessing::OpenGl => Some("OpenGL"),
                    ImageNormalMapProcessing::DirectX => Some("DirectX"),
                },
                |selected| {
                    let format = match selected {
                        "None" => ImageNormalMapProcessing::None,
                        "OpenGL" => ImageNormalMapProcessing::OpenGl,
                        "DirectX" => ImageNormalMapProcessing::DirectX,
                        _ => ImageNormalMapProcessing::None,
                    };

                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_image_normal_map_processing(format)),
                    )
                },
            )
            .width(Length::Fixed(150.0))
            .style(PorterPickListStyle)
            .into(),
            vertical_space(4.0).into(),
            text("Settings - Animations")
                .size(20.0)
                .style(PorterLabelStyle)
                .into(),
            vertical_space(2.0).into(),
            text("Choose what animation file types to export to:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            checkbox(
                "Cast",
                anim_format_enabled(AnimationFileType::Cast),
                |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_anim_file_type(AnimationFileType::Cast, value)
                    }))
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            checkbox(
                "SEAnim",
                anim_format_enabled(AnimationFileType::SEAnim),
                |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_anim_file_type(AnimationFileType::SEAnim, value)
                    }))
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            vertical_space(4.0).into(),
        ]);

        if self.sounds_enabled {
            settings.extend([
                text("Settings - Audio")
                    .size(20.0)
                    .style(PorterLabelStyle)
                    .into(),
                vertical_space(2.0).into(),
                text("Choose what audio file types to export to:")
                    .style(PorterLabelStyle)
                    .into(),
                vertical_space(0.0).into(),
                checkbox(
                    "Wav 16bit pcm",
                    audio_format_enabled(AudioFileType::Wav16Pcm),
                    |value| {
                        Message::SaveSettings(self.settings.update(|settings| {
                            settings.set_audio_file_type(AudioFileType::Wav16Pcm, value)
                        }))
                    },
                )
                .style(PorterCheckboxStyle)
                .into(),
                checkbox(
                    "Wav 32bit float",
                    audio_format_enabled(AudioFileType::WavFloat),
                    |value| {
                        Message::SaveSettings(self.settings.update(|settings| {
                            settings.set_audio_file_type(AudioFileType::WavFloat, value)
                        }))
                    },
                )
                .style(PorterCheckboxStyle)
                .into(),
                checkbox("Flac", audio_format_enabled(AudioFileType::Flac), |value| {
                    Message::SaveSettings(self.settings.update(|settings| {
                        settings.set_audio_file_type(AudioFileType::Flac, value)
                    }))
                })
                .style(PorterCheckboxStyle)
                .into(),
                vertical_space(4.0).into(),
            ]);
        }

        settings.extend([
            text("Settings - Preview")
                .size(20.0)
                .style(PorterLabelStyle)
                .into(),
            vertical_space(2.0).into(),
            text("Change the preview control scheme:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            pick_list(
                vec!["Autodesk Maya", "Blender"],
                match self.settings.preview_controls() {
                    PreviewControlScheme::Maya => Some("Autodesk Maya"),
                    PreviewControlScheme::Blender => Some("Blender"),
                },
                |selected| {
                    let controls = match selected {
                        "Autodesk Maya" => PreviewControlScheme::Maya,
                        "Blender" => PreviewControlScheme::Blender,
                        _ => PreviewControlScheme::Maya,
                    };

                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_preview_controls(controls)),
                    )
                },
            )
            .style(PorterPickListStyle)
            .width(Length::Fixed(150.0))
            .into(),
            vertical_space(2.0).into(),
            text("Choose whether or not to show the preview controls overlay:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            checkbox(
                "Show controls overlay",
                self.settings.preview_overlay(),
                |value| {
                    Message::SaveSettings(
                        self.settings
                            .update(|settings| settings.set_preview_overlay(value)),
                    )
                },
            )
            .style(PorterCheckboxStyle)
            .into(),
            vertical_space(4.0).into(),
            text("Settings - Advanced")
                .size(20.0)
                .style(PorterLabelStyle)
                .into(),
        ]);

        if self.raw_files_forcable {
            settings.extend([
                vertical_space(2.0).into(),
                text("Choose whether or not treat all assets as raw files (Not recommended):")
                    .style(PorterLabelStyle)
                    .into(),
                vertical_space(0.0).into(),
                checkbox(
                    "Treat all assets as raw files",
                    self.settings.force_raw_files(),
                    |value| {
                        Message::SaveSettings(
                            self.settings
                                .update(|settings| settings.set_force_raw_files(value)),
                        )
                    },
                )
                .style(PorterCheckboxStyle)
                .into(),
            ]);
        }

        settings.extend([
            vertical_space(2.0).into(),
            text("Troubleshooting options:")
                .style(PorterLabelStyle)
                .into(),
            vertical_space(0.0).into(),
            row(vec![
                button("Reset Settings")
                    .on_press(Message::SaveSettings(PorterSettings::default()))
                    .style(PorterButtonStyle)
                    .into(),
                button("Open Config Folder")
                    .on_press(Message::OpenConfigFolder)
                    .style(PorterButtonStyle)
                    .into(),
            ])
            .align_items(Alignment::Center)
            .spacing(8.0)
            .into(),
        ]);

        scrollable(
            column(settings)
                .spacing(8.0)
                .padding(16.0)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(PorterScrollStyle)
        .into()
    }
}
