use iced::widget::Column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;

use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::Task;

use porter_model::ModelFileType;

use porter_texture::ImageFileType;

use crate::AppState;
use crate::MainMessage;
use crate::Message;
use crate::PreviewControlScheme;
use crate::palette;
use crate::strings;
use crate::system;
use crate::widgets;

use super::PreviewMessage;

/// Settings component handler.
pub struct Settings {
    custom_scale: Option<String>,
    page: SettingsPage,
}

/// Messages produced by the settings component.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Save(crate::Settings),
    PickExportFolder,
    OpenExportFolder,
    OpenConfigFolder,
    ApplyCustomScale,
    ScaleInput(String),
    SetPage(SettingsPage),
}

/// Pages for the settings component.
#[derive(Debug, Clone, Copy)]
pub enum SettingsPage {
    General,
    Models,
    Images,
    #[cfg(feature = "animations")]
    Animations,
    #[cfg(all(feature = "sounds", feature = "sounds-convertible"))]
    Sounds,
    Preview,
    Advanced,
}

impl Settings {
    /// Creates a new settings component.
    pub fn new() -> Self {
        Self {
            custom_scale: None,
            page: SettingsPage::General,
        }
    }

    /// Handles updates for the settings component.
    pub fn update(&mut self, state: &mut AppState, message: SettingsMessage) -> Task<Message> {
        use SettingsMessage::*;

        match message {
            Save(settings) => self.on_save(state, settings),
            PickExportFolder => self.on_pick_export_folder(state),
            OpenExportFolder => self.on_open_export_folder(state),
            OpenConfigFolder => self.on_open_config_folder(state),
            ApplyCustomScale => self.on_apply_custom_scale(state),
            ScaleInput(input) => self.on_scale_input(state, input),
            SetPage(page) => self.on_set_page(state, page),
        }
    }

    /// Handles rendering the settings component.
    pub fn view<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        #[allow(unused_mut)]
        let mut buttons: Vec<_> = Vec::with_capacity(8);

        buttons.extend([
            widgets::settings_button("General", matches!(self.page, SettingsPage::General))
                .on_press(Message::from(SettingsMessage::SetPage(
                    SettingsPage::General,
                )))
                .into(),
            widgets::settings_button("Models", matches!(self.page, SettingsPage::Models))
                .on_press(Message::from(SettingsMessage::SetPage(
                    SettingsPage::Models,
                )))
                .into(),
            widgets::settings_button("Images", matches!(self.page, SettingsPage::Images))
                .on_press(Message::from(SettingsMessage::SetPage(
                    SettingsPage::Images,
                )))
                .into(),
        ]);

        #[cfg(feature = "animations")]
        {
            buttons.push(
                widgets::settings_button(
                    "Animations",
                    matches!(self.page, SettingsPage::Animations),
                )
                .on_press(Message::from(SettingsMessage::SetPage(
                    SettingsPage::Animations,
                )))
                .into(),
            );
        }

        #[cfg(all(feature = "sounds", feature = "sounds-convertible"))]
        {
            buttons.push(
                widgets::settings_button("Sounds", matches!(self.page, SettingsPage::Sounds))
                    .on_press(Message::from(SettingsMessage::SetPage(
                        SettingsPage::Sounds,
                    )))
                    .into(),
            );
        }

        buttons.extend([
            widgets::settings_button("Preview", matches!(self.page, SettingsPage::Preview))
                .on_press(Message::from(SettingsMessage::SetPage(
                    SettingsPage::Preview,
                )))
                .into(),
            widgets::settings_button("Advanced", matches!(self.page, SettingsPage::Advanced))
                .on_press(Message::from(SettingsMessage::SetPage(
                    SettingsPage::Advanced,
                )))
                .into(),
        ]);

        let categories = container(
            Column::from_vec(buttons)
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(4.0),
        )
        .style(settings_container_style)
        .padding(16.0)
        .width(Length::Fixed(250.0))
        .height(Length::Fill);

        let settings = match self.page {
            SettingsPage::General => self.view_general(state),
            SettingsPage::Models => self.view_models(state),
            SettingsPage::Images => self.view_images(state),
            #[cfg(feature = "animations")]
            SettingsPage::Animations => self.view_animations(state),
            #[cfg(all(feature = "sounds", feature = "sounds-convertible"))]
            SettingsPage::Sounds => self.view_sounds(state),
            SettingsPage::Preview => self.view_preview(state),
            SettingsPage::Advanced => self.view_advanced(state),
        };

        let settings = container(
            widgets::scrollable(settings)
                .spacing(4.0)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(settings_container_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16.0);

        row([categories.into(), settings.into()])
            .spacing(8.0)
            .padding(8.0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Handles rendering the general settings view.
    pub fn view_general<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));

        let load_disabled =
            cfg!(feature = "raw-files-forcible") && state.settings.force_raw_files();
        let load_setting = |setting: bool| setting && !load_disabled;

        let mut settings: Vec<Element<'a, Message>> = Vec::with_capacity(32);

        settings.extend([
            text("General Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
            text("Choose what asset types to load and display:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Load Models", load_setting(state.settings.load_models()))
                .on_toggle_maybe(if load_disabled {
                    None
                } else {
                    Some(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_load_models(value)),
                        )
                    })
                })
                .into(),
        ]);

        #[cfg(feature = "animations")]
        {
            settings.push(
                widgets::checkbox(
                    "Load Animations",
                    load_setting(state.settings.load_animations()),
                )
                .on_toggle_maybe(if load_disabled {
                    None
                } else {
                    Some(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_load_animations(value)),
                        )
                    })
                })
                .into(),
            );
        }

        settings.push(
            widgets::checkbox("Load Images", load_setting(state.settings.load_images()))
                .on_toggle_maybe(if load_disabled {
                    None
                } else {
                    Some(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_load_images(value)),
                        )
                    })
                })
                .into(),
        );

        #[cfg(feature = "materials")]
        {
            settings.push(
                widgets::checkbox(
                    "Load Materials",
                    load_setting(state.settings.load_materials()),
                )
                .on_toggle_maybe(if load_disabled {
                    None
                } else {
                    Some(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_load_materials(value)),
                        )
                    })
                })
                .into(),
            );
        }

        #[cfg(feature = "sounds")]
        {
            settings.push(
                widgets::checkbox("Load Sounds", load_setting(state.settings.load_sounds()))
                    .on_toggle_maybe(if load_disabled {
                        None
                    } else {
                        Some(move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_load_sounds(value)),
                            )
                        })
                    })
                    .into(),
            );
        }

        #[cfg(feature = "raw-files")]
        {
            use iced::widget::tooltip::Position;

            settings.push(
                widgets::tooltip(
                    widgets::checkbox("Load Raw Files", state.settings.load_raw_files())
                        .on_toggle_maybe(if load_disabled {
                            None
                        } else {
                            Some(move |value| {
                                save_message(
                                    state
                                        .settings
                                        .update(|settings| settings.set_load_raw_files(value)),
                                )
                            })
                        }),
                    "Loads all files that are exportable as-is",
                    Position::Right,
                )
                .into(),
            );
        }

        #[cfg(feature = "raw-files-forcible")]
        {
            if state.settings.force_raw_files() {
                settings.push(
                    text("(Treat all assets as raw files is enabled)")
                        .color(palette::TEXT_COLOR_WARN)
                        .into(),
                );
            }
        }

        settings.extend([
            widgets::horizontal_rule().into(),
            text("Customize the exported files directory:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            row(vec![
                widgets::text_input(
                    "Exported files directory",
                    state
                        .settings
                        .output_directory()
                        .to_string_lossy()
                        .as_ref(),
                )
                .on_input(|_| Message::Noop)
                .width(475.0)
                .into(),
                widgets::button("Browse")
                    .on_press(Message::from(SettingsMessage::PickExportFolder))
                    .into(),
                widgets::button("Open")
                    .on_press(Message::from(SettingsMessage::OpenExportFolder))
                    .into(),
            ])
            .spacing(4.0)
            .into(),
            widgets::horizontal_rule().into(),
            text("Choose whether or not to automatically scale assets (Recommended):")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Automatically scale assets", state.settings.auto_scale())
                .on_toggle(move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_auto_scale(value)),
                    )
                })
                .into(),
            space().height(2.0).into(),
            text("Set a custom asset scale factor:")
                .color(if state.settings.auto_scale() {
                    palette::TEXT_COLOR_SECONDARY
                } else {
                    palette::TEXT_COLOR_DISABLED
                })
                .size(18.0)
                .into(),
            space().height(0.0).into(),
            row([
                widgets::checkbox("Custom scale:", state.settings.custom_scale().is_some())
                    .on_toggle_maybe(if state.settings.auto_scale() {
                        Some(move |value: bool| {
                            save_message(
                                state.settings.update(|settings| {
                                    settings.set_custom_scale(value.then_some(1.0))
                                }),
                            )
                        })
                    } else {
                        None
                    })
                    .into(),
                row([
                    widgets::text_input(
                        "",
                        &self
                            .custom_scale
                            .clone()
                            .unwrap_or_else(|| {
                                state
                                    .settings
                                    .custom_scale()
                                    .map(format_custom_scale)
                                    .unwrap_or_else(|| String::from("1.0"))
                            }),
                    )
                    .on_input_maybe(
                        if state.settings.auto_scale() && state.settings.custom_scale().is_some() {
                            Some(|input| Message::from(SettingsMessage::ScaleInput(input)))
                        } else {
                            None
                        },
                    )
                    .width(Length::Fixed(120.0))
                    .into(),
                    widgets::button("Apply")
                        .on_press_maybe(
                            if state.settings.auto_scale()
                                && state.settings.custom_scale().is_some()
                            {
                                Some(Message::from(SettingsMessage::ApplyCustomScale))
                            } else {
                                None
                            },
                        )
                        .into(),
                ])
                .spacing(4.0)
                .align_y(Alignment::Center)
                .into(),
            ])
            .spacing(8.0)
            .align_y(Alignment::Center)
            .into(),
        ]);

        Column::from_vec(settings)
            .spacing(8.0)
            .padding(0.0)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    /// Handles rendering the models settings view.
    pub fn view_models<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));
        let model_formats = state.settings.model_file_types();
        let model_format_enabled = |format: ModelFileType| model_formats.contains(&format);

        let mut settings: Vec<Element<'a, Message>> = Vec::with_capacity(16);

        settings.extend([
            text("Model Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
            text("Choose what model file types to export to:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Cast", model_format_enabled(ModelFileType::Cast))
                .on_toggle(move |value| {
                    save_message(state.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::Cast, value)
                    }))
                })
                .into(),
            widgets::checkbox("OBJ", model_format_enabled(ModelFileType::Obj))
                .on_toggle(move |value| {
                    save_message(
                        state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Obj, value)
                        }),
                    )
                })
                .into(),
            widgets::checkbox("Valve SMD", model_format_enabled(ModelFileType::Smd))
                .on_toggle(move |value| {
                    save_message(
                        state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Smd, value)
                        }),
                    )
                })
                .into(),
            widgets::checkbox("XNALara", model_format_enabled(ModelFileType::XnaLara))
                .on_toggle(move |value| {
                    save_message(state.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::XnaLara, value)
                    }))
                })
                .into(),
            widgets::checkbox(
                "CoD XModel",
                model_format_enabled(ModelFileType::XModelExport),
            )
            .on_toggle(move |value| {
                save_message(state.settings.update(|settings| {
                    settings.set_model_file_type(ModelFileType::XModelExport, value)
                }))
            })
            .into(),
            widgets::checkbox("Autodesk Maya", model_format_enabled(ModelFileType::Maya))
                .on_toggle(move |value| {
                    save_message(state.settings.update(|settings| {
                        settings.set_model_file_type(ModelFileType::Maya, value)
                    }))
                })
                .into(),
            widgets::checkbox("FBX", model_format_enabled(ModelFileType::Fbx))
                .on_toggle(move |value| {
                    save_message(
                        state.settings.update(|settings| {
                            settings.set_model_file_type(ModelFileType::Fbx, value)
                        }),
                    )
                })
                .into(),
        ]);

        #[cfg(feature = "materials")]
        {
            use crate::ModelMaterialProcessing;

            settings.extend([
                widgets::horizontal_rule().into(),
                text("Choose where to export materials with models:")
                    .size(18.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                space().height(0.0).into(),
                widgets::pick_list(
                    vec!["Skip", "In Models Folder", "In Materials Folder"],
                    match state
                        .settings
                        .model_material_processing()
                    {
                        ModelMaterialProcessing::Skip => Some("Skip"),
                        ModelMaterialProcessing::InModelFolder => Some("In Models Folder"),
                        ModelMaterialProcessing::InMaterialFolder => Some("In Materials Folder"),
                    },
                    move |selected| {
                        let value = match selected {
                            "Skip" => ModelMaterialProcessing::Skip,
                            "In Models Folder" => ModelMaterialProcessing::InModelFolder,
                            "In Materials Folder" => ModelMaterialProcessing::InMaterialFolder,
                            _ => ModelMaterialProcessing::Skip,
                        };

                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_model_material_processing(value)),
                        )
                    },
                )
                .width(Length::Fixed(250.0))
                .into(),
            ]);
        }

        Column::from_vec(settings)
            .spacing(8.0)
            .padding(0.0)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    /// Handles rendering the images settings view.
    pub fn view_images<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));

        let (notice, notice_color) = match state.settings.image_file_type() {
            ImageFileType::Tga => (
                "(The selected image format may be lossy or take up more space than necessary)",
                palette::TEXT_COLOR_WARN,
            ),
            ImageFileType::Dds => (
                "(The selected image format is lossless, some software may have trouble opening it)",
                palette::TEXT_COLOR_SUCCESS,
            ),
            _ => (
                "(The selected image format is lossless and recommended for export)",
                palette::TEXT_COLOR_SUCCESS,
            ),
        };

        let mut settings: Vec<Element<'a, Message>> = Vec::with_capacity(16);

        settings.extend([
            text("Image Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
            text("Choose what image file type to export to:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::pick_list(
                vec!["DDS", "PNG", "TIFF", "TGA"],
                match state.settings.image_file_type() {
                    ImageFileType::Dds => Some("DDS"),
                    ImageFileType::Png => Some("PNG"),
                    ImageFileType::Tiff => Some("TIFF"),
                    ImageFileType::Tga => Some("TGA"),
                    _ => Some("DDS"),
                },
                move |selected| {
                    let value = match selected {
                        "DDS" => ImageFileType::Dds,
                        "PNG" => ImageFileType::Png,
                        "TIFF" => ImageFileType::Tiff,
                        "TGA" => ImageFileType::Tga,
                        _ => ImageFileType::Dds,
                    };

                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_image_file_type(value)),
                    )
                },
            )
            .width(Length::Fixed(250.0))
            .into(),
            space().height(2.0).into(),
            text(notice).color(notice_color).into(),
        ]);

        #[cfg(feature = "normal-maps-convertible")]
        {
            use crate::ImageNormalMapProcessing;

            settings.extend([
                widgets::horizontal_rule().into(),
                text("Choose a normal map conversion method:")
                    .size(18.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                space().height(0.0).into(),
                widgets::pick_list(
                    vec!["None", "OpenGL", "DirectX"],
                    match state
                        .settings
                        .image_normal_map_processing()
                    {
                        ImageNormalMapProcessing::None => Some("None"),
                        ImageNormalMapProcessing::OpenGl => Some("OpenGL"),
                        ImageNormalMapProcessing::DirectX => Some("DirectX"),
                    },
                    move |selected| {
                        let value = match selected {
                            "None" => ImageNormalMapProcessing::None,
                            "OpenGL" => ImageNormalMapProcessing::OpenGl,
                            "DirectX" => ImageNormalMapProcessing::DirectX,
                            _ => ImageNormalMapProcessing::None,
                        };

                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_image_normal_map_processing(value)),
                        )
                    },
                )
                .width(Length::Fixed(250.0))
                .into(),
            ]);
        }

        Column::from_vec(settings)
            .spacing(8.0)
            .padding(0.0)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    /// Handles rendering the animations settings view.
    #[cfg(feature = "animations")]
    pub fn view_animations<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        use porter_animation::AnimationFileType;

        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));
        let anim_formats = state.settings.anim_file_types();
        let anim_format_enabled = |format: AnimationFileType| anim_formats.contains(&format);

        let mut settings: Vec<Element<'a, Message>> = Vec::with_capacity(16);

        settings.extend([
            text("Animation Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
            text("Choose what animation file types to export to:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Cast", anim_format_enabled(AnimationFileType::Cast))
                .on_toggle(move |value| {
                    save_message(state.settings.update(|settings| {
                        settings.set_anim_file_type(AnimationFileType::Cast, value)
                    }))
                })
                .into(),
        ]);

        #[cfg(feature = "animations-bake")]
        {
            use iced::widget::tooltip::Position;

            settings.extend([
                widgets::horizontal_rule().into(),
                text("Choose whether or not to bake animations:")
                    .size(18.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                space().height(0.0).into(),
                widgets::tooltip(
                    widgets::checkbox("Bake animations", state.settings.anim_bake()).on_toggle(
                        move |value| {
                            save_message(
                                state
                                    .settings
                                    .update(|settings| settings.set_anim_bake(value)),
                            )
                        },
                    ),
                    "When enabled this will disable model constraint export",
                    Position::Right,
                )
                .into(),
            ]);
        }

        Column::from_vec(settings)
            .spacing(8.0)
            .padding(0.0)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    /// Handles rendering the sounds settings view.
    #[cfg(all(feature = "sounds", feature = "sounds-convertible"))]
    pub fn view_sounds<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        use iced::widget::column;

        use porter_audio::AudioFileType;

        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));
        let audio_formats = state.settings.audio_file_types();
        let audio_format_enabled = |format: AudioFileType| audio_formats.contains(&format);

        column([
            text("Sound Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
            text("Choose what audio file types to export to:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Wav", audio_format_enabled(AudioFileType::Wav))
                .on_toggle(move |value| {
                    save_message(
                        state.settings.update(|settings| {
                            settings.set_audio_file_type(AudioFileType::Wav, value)
                        }),
                    )
                })
                .into(),
            widgets::checkbox("Flac", audio_format_enabled(AudioFileType::Flac))
                .on_toggle(move |value| {
                    save_message(state.settings.update(|settings| {
                        settings.set_audio_file_type(AudioFileType::Flac, value)
                    }))
                })
                .into(),
        ])
        .spacing(8.0)
        .padding(0.0)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
    }

    /// Handles rendering the preview settings view.
    pub fn view_preview<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));

        let mut settings: Vec<Element<'a, Message>> = Vec::with_capacity(32);

        settings.extend([
            text("Preview Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
            text("Change the preview control scheme:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::pick_list(
                vec!["Autodesk Maya", "Blender"],
                match state.settings.preview_controls() {
                    PreviewControlScheme::Maya => Some("Autodesk Maya"),
                    PreviewControlScheme::Blender => Some("Blender"),
                },
                move |selected| {
                    let value = match selected {
                        "Autodesk Maya" => PreviewControlScheme::Maya,
                        "Blender" => PreviewControlScheme::Blender,
                        _ => PreviewControlScheme::Maya,
                    };

                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_preview_controls(value)),
                    )
                },
            )
            .width(Length::Fixed(250.0))
            .into(),
            widgets::horizontal_rule().into(),
            text("Choose whether or not to open preview in a separate window:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Separate window", state.settings.preview_window())
                .on_toggle(move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_preview_window(value)),
                    )
                })
                .into(),
            widgets::horizontal_rule().into(),
            text("Choose whether or not to show the preview controls overlay:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            widgets::checkbox("Show controls overlay", state.settings.preview_overlay())
                .on_toggle(move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_preview_overlay(value)),
                    )
                })
                .into(),
            widgets::horizontal_rule().into(),
            text("Set the preview far clip distance (May impact performance):")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            row([
                widgets::slider(10000..=1000000, state.settings.far_clip(), move |value| {
                    save_message(
                        state
                            .settings
                            .update(|settings| settings.set_far_clip(value)),
                    )
                })
                .width(400.0)
                .step(10000u32)
                .into(),
                text(state.settings.far_clip().to_string())
                    .width(100.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
            ])
            .width(Length::Shrink)
            .spacing(8.0)
            .into(),
        ]);

        #[cfg(all(feature = "sounds", feature = "sounds-convertible"))]
        {
            settings.extend([
                widgets::horizontal_rule().into(),
                text("Set the preview audio volume (Limited for safety):")
                    .size(18.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                space().height(0.0).into(),
                row([
                    widgets::slider(0..=50, state.settings.volume(), move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_volume(value)),
                        )
                    })
                    .width(400.0)
                    .step(1u32)
                    .into(),
                    text(format!("{}%", state.settings.volume()))
                        .width(100.0)
                        .color(palette::TEXT_COLOR_SECONDARY)
                        .into(),
                ])
                .width(Length::Shrink)
                .spacing(8.0)
                .into(),
            ]);
        }

        Column::from_vec(settings)
            .spacing(8.0)
            .padding(0.0)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    /// Handles rendering the advanced settings view.
    pub fn view_advanced<'a>(&'a self, state: &'a AppState) -> Element<'a, Message> {
        let save_message =
            |settings: crate::Settings| Message::from(SettingsMessage::Save(settings));

        let mut settings: Vec<_> = Vec::with_capacity(16);

        settings.extend([
            text("Advanced Settings")
                .size(24.0)
                .color(palette::TEXT_COLOR_DEFAULT)
                .into(),
            space().height(2.0).into(),
        ]);

        #[cfg(feature = "raw-files-forcible")]
        {
            use iced::widget::tooltip::Position;

            settings.extend([
                text("Choose whether or not to treat all assets as raw files:")
                    .size(18.0)
                    .color(palette::TEXT_COLOR_SECONDARY)
                    .into(),
                space().height(0.0).into(),
                widgets::tooltip(
                    widgets::checkbox(
                        "Treat all assets as raw files",
                        state.settings.force_raw_files(),
                    )
                    .on_toggle(move |value| {
                        save_message(
                            state
                                .settings
                                .update(|settings| settings.set_force_raw_files(value)),
                        )
                    }),
                    "Disables conversion of all supported asset types",
                    Position::Right,
                )
                .into(),
                widgets::horizontal_rule().into(),
            ]);
        }

        #[cfg(not(feature = "raw-files-forcible"))]
        {
            let _ = state;
        }

        settings.extend([
            text("Troubleshooting options:")
                .size(18.0)
                .color(palette::TEXT_COLOR_SECONDARY)
                .into(),
            space().height(0.0).into(),
            row([
                widgets::button("Reset Settings")
                    .on_press(save_message(crate::Settings::default()))
                    .into(),
                widgets::button("Open Config Folder")
                    .on_press(Message::from(SettingsMessage::OpenConfigFolder))
                    .into(),
            ])
            .align_y(Alignment::Center)
            .spacing(8.0)
            .into(),
        ]);

        Column::from_vec(settings)
            .spacing(8.0)
            .padding(0.0)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    /// Saves settings to state and disk.
    fn on_save(&mut self, state: &mut AppState, settings: crate::Settings) -> Task<Message> {
        if !state.reload_required {
            state.reload_required = state
                .settings
                .reload_required(&settings);
        }

        state.settings = settings;
        state.settings.save(state.name);

        self.custom_scale = state
            .settings
            .custom_scale()
            .map(format_custom_scale);

        Task::done(Message::from(PreviewMessage::SyncSettings))
    }

    /// Allows the user to pick a new export folder.
    fn on_pick_export_folder(&mut self, _: &mut AppState) -> Task<Message> {
        Task::done(Message::from(MainMessage::PickExportFolder))
    }

    /// Opens the export folder.
    fn on_open_export_folder(&mut self, state: &mut AppState) -> Task<Message> {
        system::open_folder(state.settings.output_directory());

        Task::none()
    }

    /// Opens the config folder.
    fn on_open_config_folder(&mut self, _: &mut AppState) -> Task<Message> {
        system::open_folder(system::config_dir());

        Task::none()
    }

    /// Applies the custom user provided scale value.
    fn on_apply_custom_scale(&mut self, state: &mut AppState) -> Task<Message> {
        let Some(custom_scale) = self.custom_scale.take() else {
            return Task::none();
        };

        let Ok(value) = custom_scale.parse::<f32>() else {
            let warning = MainMessage::Warning(String::from(strings::CUSTOM_SCALE_FACTOR_ERROR));
            let warning_task = Task::done(Message::from(warning));

            let reset_task = self.on_save(
                state,
                state
                    .settings
                    .update(|settings| settings.set_custom_scale(Some(1.0))),
            );

            return Task::batch([warning_task, reset_task]);
        };

        self.on_save(
            state,
            state
                .settings
                .update(|settings| settings.set_custom_scale(Some(value))),
        )
    }

    /// Occurs when the user enters a custom scale value.
    fn on_scale_input(&mut self, _: &mut AppState, input: String) -> Task<Message> {
        let input = if input.len() > 8 {
            input.chars().take(8).collect()
        } else {
            input
        };

        self.custom_scale = Some(input);

        Task::none()
    }

    /// Occurs when the user changes the settings page.
    fn on_set_page(&mut self, _: &mut AppState, page: SettingsPage) -> Task<Message> {
        self.page = page;

        Task::none()
    }
}

/// Formats a custom scale factor.
fn format_custom_scale(scale: f32) -> String {
    format!("{:?}", scale)
}

/// Style for the settings containers.
fn settings_container_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(palette::BACKGROUND_COLOR_LIGHT_050)),
        border: iced::Border {
            width: 1.0,
            color: palette::BACKGROUND_COLOR_LIGHT_100,
            radius: iced::border::Radius::new(4.0),
        },
        ..Default::default()
    }
}
