use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use porter_cast::CastFile;
use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;

use crate::Animation;
use crate::AnimationError;
use crate::CurveAttribute;
use crate::CurveDataType;
use crate::KeyframeValue;

/// Writes an animation in cast format to the given path.
pub fn to_cast<P: AsRef<Path>>(path: P, animation: &Animation) -> Result<(), AnimationError> {
    let mut root = CastNode::root();

    let animation_node = root.create(CastId::Animation);

    animation_node
        .create_property(CastPropertyId::Float, "fr")
        .push(animation.framerate);
    animation_node
        .create_property(CastPropertyId::Byte, "lo")
        .push(animation.looping as u8);

    for curve in &animation.curves {
        if matches!(curve.attribute(), CurveAttribute::Notetrack) {
            continue;
        }

        let (num_curves, curve_props) = match curve.attribute() {
            CurveAttribute::Rotation => (1, ["rq", "", ""]),
            CurveAttribute::Scale => (3, ["sx", "sy", "sz"]),
            CurveAttribute::Translate => (3, ["tx", "ty", "tz"]),
            CurveAttribute::Visibility => (1, ["vb", "", ""]),
            _ => (0, ["", "", ""]),
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

            for keyframe in curve.keyframes() {
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
                _ => CastPropertyId::Byte,
            };

            let keyvalue_buffer = curve_node.create_property(property_type, "kv");

            for keyframe in curve.keyframes() {
                match keyframe.value {
                    KeyframeValue::Bool(bool) => {
                        keyvalue_buffer.push(bool as u8);
                    }
                    KeyframeValue::Quaternion(rotation) => {
                        keyvalue_buffer.push(rotation);
                    }
                    KeyframeValue::Vector3(vector) => {
                        keyvalue_buffer.push(vector[i]);
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

        let key_buffer = track_node.create_property(CastPropertyId::Integer32, "kb");

        for key in curve.keyframes() {
            key_buffer.push(key.time);
        }
    }

    let writer = BufWriter::new(File::create(path.as_ref().with_extension("cast"))?);

    let mut file = CastFile::new();

    file.push(root);
    file.write(writer)?;

    Ok(())
}
