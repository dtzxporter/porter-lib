use std::fs::File;
use std::path::Path;

use porter_cast::CastFile;
use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;

use porter_math::Axis;

use porter_utils::BufferWriteExt;

use crate::Animation;
use crate::AnimationError;
use crate::CurveAttribute;
use crate::CurveDataType;
use crate::Keyframes;

/// Writes an animation in cast format to the given path.
pub fn to_cast<P: AsRef<Path>>(path: P, animation: &Animation) -> Result<(), AnimationError> {
    let mut root = CastNode::root();

    let meta_node = root.create(CastId::Metadata);

    if !cfg!(feature = "debrand") {
        meta_node
            .create_property(CastPropertyId::String, "a")
            .push("DTZxPorter");
    }

    meta_node
        .create_property(CastPropertyId::String, "s")
        .push("Exported by PorterLib");

    let up_axis = match animation.up_axis {
        Axis::X => "x",
        Axis::Y => "y",
        Axis::Z => "z",
    };

    meta_node
        .create_property(CastPropertyId::String, "up")
        .push(up_axis);

    let animation_node = root.create(CastId::Animation);

    animation_node
        .create_property(CastPropertyId::Float, "fr")
        .push(animation.frame_rate);
    animation_node
        .create_property(CastPropertyId::Byte, "lo")
        .push(animation.looping);

    for curve in &animation.curves {
        let largest_frame_time = curve.largest_frame_time();

        let (num_curves, curve_properties, property_type) = match curve.attribute() {
            CurveAttribute::Translate => (3, ["tx", "ty", "tz"], CastPropertyId::Float),
            CurveAttribute::Rotate => (1, ["rq", "", ""], CastPropertyId::Vector4),
            CurveAttribute::Scale => (3, ["sx", "sy", "sz"], CastPropertyId::Float),
            CurveAttribute::Visibility => (1, ["vb", "", ""], CastPropertyId::Byte),
            CurveAttribute::Notetrack => {
                // Handled separately via notification tracks.
                continue;
            }
            CurveAttribute::BlendShape => (1, ["bs", "", ""], CastPropertyId::Float),
        };

        for (i, curve_property) in curve_properties
            .into_iter()
            .enumerate()
            .take(num_curves)
        {
            let curve_node = animation_node.create(CastId::Curve);

            curve_node
                .create_property(CastPropertyId::String, "nn")
                .push(curve.name());

            match curve.data_type() {
                CurveDataType::Absolute => {
                    curve_node
                        .create_property(CastPropertyId::String, "m")
                        .push("absolute");
                }
                CurveDataType::Additive => {
                    curve_node
                        .create_property(CastPropertyId::String, "m")
                        .push("additive");
                }
                CurveDataType::Relative => {
                    curve_node
                        .create_property(CastPropertyId::String, "m")
                        .push("relative");
                }
            }

            curve_node
                .create_property(CastPropertyId::String, "kp")
                .push(curve_property);

            let [kb, kv] = if largest_frame_time <= 0xFF {
                curve_node.create_properties([(CastPropertyId::Byte, "kb"), (property_type, "kv")])
            } else if largest_frame_time <= 0xFFFF {
                curve_node.create_properties([(CastPropertyId::Short, "kb"), (property_type, "kv")])
            } else {
                curve_node
                    .create_properties([(CastPropertyId::Integer32, "kb"), (property_type, "kv")])
            };

            kb.try_reserve_exact(curve.len())?;
            kv.try_reserve_exact(curve.len())?;

            let mut push_frame_time = |time: u32| {
                if largest_frame_time <= 0xFF {
                    kb.push(time as u8);
                } else if largest_frame_time <= 0xFFFF {
                    kb.push(time as u16);
                } else {
                    kb.push(time);
                }
            };

            match curve.keyframes() {
                Keyframes::Translate(keyframes) | Keyframes::Scale(keyframes) => {
                    for keyframe in keyframes {
                        push_frame_time(keyframe.time);
                        kv.push(keyframe.value[i]);
                    }
                }
                Keyframes::Rotate(keyframes) => {
                    for keyframe in keyframes {
                        push_frame_time(keyframe.time);
                        kv.push(keyframe.value);
                    }
                }
                Keyframes::Visibility(keyframes) => {
                    for keyframe in keyframes {
                        push_frame_time(keyframe.time);
                        kv.push(keyframe.value);
                    }
                }
                Keyframes::Notetrack(_) => {
                    // Handled separately via notification tracks.
                    continue;
                }
                Keyframes::BlendShape(keyframes) => {
                    for keyframe in keyframes {
                        push_frame_time(keyframe.time);
                        kv.push(keyframe.value);
                    }
                }
            }
        }
    }

    for curve in &animation.curves {
        let Keyframes::Notetrack(keyframes) = curve.keyframes() else {
            continue;
        };

        let track_node = animation_node.create(CastId::NotificationTrack);

        track_node
            .create_property(CastPropertyId::String, "n")
            .push(curve.name());

        let key_buffer = track_node
            .create_property(CastPropertyId::Integer32, "kb")
            .try_reserve_exact(keyframes.len())?;

        for key in keyframes {
            key_buffer.push(key.time);
        }
    }

    for curve_override in &animation.curve_mode_overrides {
        let override_node = animation_node.create(CastId::CurveModeOverride);

        override_node
            .create_property(CastPropertyId::String, "nn")
            .push(curve_override.name.as_str());

        match curve_override.data_type {
            CurveDataType::Absolute => {
                override_node
                    .create_property(CastPropertyId::String, "m")
                    .push("absolute");
            }
            CurveDataType::Additive => {
                override_node
                    .create_property(CastPropertyId::String, "m")
                    .push("additive");
            }
            CurveDataType::Relative => {
                override_node
                    .create_property(CastPropertyId::String, "m")
                    .push("relative");
            }
        }

        override_node
            .create_property(CastPropertyId::Byte, "ot")
            .push(curve_override.override_translate);

        override_node
            .create_property(CastPropertyId::Byte, "or")
            .push(curve_override.override_rotation);

        override_node
            .create_property(CastPropertyId::Byte, "os")
            .push(curve_override.override_scale);
    }

    let writer = File::create(path.as_ref().with_extension("cast"))?.buffer_write();

    let mut file = CastFile::new();

    file.push(root);
    file.write(writer)?;

    Ok(())
}
