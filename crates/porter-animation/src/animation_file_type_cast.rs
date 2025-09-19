use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use porter_cast::CastFile;
use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;

use porter_math::Axis;

use crate::Animation;
use crate::AnimationError;
use crate::CurveAttribute;
use crate::CurveDataType;
use crate::KeyframeValue;

/// Writes an animation in cast format to the given path.
pub fn to_cast<P: AsRef<Path>>(path: P, animation: &Animation) -> Result<(), AnimationError> {
    let mut root = CastNode::root();

    let meta_node = root.create(CastId::Metadata);

    meta_node
        .create_property(CastPropertyId::String, "a")
        .push("DTZxPorter");

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
        .push(animation.framerate);
    animation_node
        .create_property(CastPropertyId::Byte, "lo")
        .push(animation.looping);

    for curve in &animation.curves {
        let (num_curves, curve_props) = match curve.attribute() {
            CurveAttribute::Rotation => (1, ["rq", "", ""]),
            CurveAttribute::Scale => (3, ["sx", "sy", "sz"]),
            CurveAttribute::Translate => (3, ["tx", "ty", "tz"]),
            CurveAttribute::Visibility => (1, ["vb", "", ""]),
            CurveAttribute::Notetrack => {
                // Handled separately via notification tracks.
                continue;
            }
            CurveAttribute::BlendShape => (1, ["bs", "", ""]),
        };

        for i in 0..num_curves {
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
                .push(curve_props[i]);

            let largest_frame_time = curve.largest_frame_time();

            let keyframe_buffer = if largest_frame_time <= 0xFF {
                curve_node.create_property(CastPropertyId::Byte, "kb")
            } else if largest_frame_time <= 0xFFFF {
                curve_node.create_property(CastPropertyId::Short, "kb")
            } else {
                curve_node.create_property(CastPropertyId::Integer32, "kb")
            };

            let keyframes = curve.keyframes();

            keyframe_buffer.try_reserve_exact(keyframes.len())?;

            for keyframe in keyframes {
                if largest_frame_time <= 0xFF {
                    keyframe_buffer.push(keyframe.time as u8);
                } else if largest_frame_time <= 0xFFFF {
                    keyframe_buffer.push(keyframe.time as u16);
                } else {
                    keyframe_buffer.push(keyframe.time);
                }
            }

            let property_type = match curve.attribute() {
                CurveAttribute::Rotation => CastPropertyId::Vector4,
                CurveAttribute::Translate => CastPropertyId::Float,
                CurveAttribute::Scale => CastPropertyId::Float,
                CurveAttribute::Visibility => CastPropertyId::Byte,
                CurveAttribute::Notetrack => unreachable!(),
                CurveAttribute::BlendShape => CastPropertyId::Float,
            };

            let keyvalue_buffer = curve_node.create_property(property_type, "kv");

            keyvalue_buffer.try_reserve_exact(keyframes.len())?;

            for keyframe in keyframes {
                match keyframe.value {
                    KeyframeValue::Bool(bool) => {
                        keyvalue_buffer.push(bool);
                    }
                    KeyframeValue::Quaternion(rotation) => {
                        keyvalue_buffer.push(rotation);
                    }
                    KeyframeValue::Vector3(vector) => {
                        keyvalue_buffer.push(vector[i]);
                    }
                    KeyframeValue::Float(float) => {
                        keyvalue_buffer.push(float);
                    }
                    KeyframeValue::None => {
                        // No value.
                    }
                }
            }
        }
    }

    for curve in &animation.curves {
        if !matches!(curve.attribute(), CurveAttribute::Notetrack) {
            continue;
        }

        let track_node = animation_node.create(CastId::NotificationTrack);

        track_node
            .create_property(CastPropertyId::String, "n")
            .push(curve.name());

        let keyframes = curve.keyframes();

        let key_buffer = track_node.create_property(CastPropertyId::Integer32, "kb");

        key_buffer.try_reserve_exact(keyframes.len())?;

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

    let writer = BufWriter::new(File::create(path.as_ref().with_extension("cast"))?);

    let mut file = CastFile::new();

    file.push(root);
    file.write(writer)?;

    Ok(())
}
