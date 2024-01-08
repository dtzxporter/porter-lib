use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use porter_math::Quaternion;
use porter_math::Vector3;

use porter_utils::StringWriteExt;
use porter_utils::StructWriteExt;

use crate::Animation;
use crate::AnimationError;
use crate::CurveAttribute;
use crate::CurveDataType;
use crate::KeyframeValue;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SEAnimHeader {
    magic: [u8; 6],
    version: u16,
    header_size: u16,
    animation_type: u8,
    looping: bool,
    data_presence_flags: u8,
    data_property_flags: u8,
    reserved: u16,
    framerate: f32,
    frame_count: u32,
    bone_count: u32,
    bone_modifiers: u8,
    reserved2: [u8; 3],
    notification_count: u32,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum SEAnimAnimationType {
    Absolute = 0,
    Additive = 1,
    Relative = 2,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy)]
enum SEAnimDataPresenceFlags {
    BoneLoc = 1 << 0,
    BoneRot = 1 << 1,
    BoneScale = 1 << 2,
    PresenceNote = 1 << 6,
    PresenceCustom = 1 << 7,
}

/// Writes an animation in seanim format to the given path.
pub fn to_seanim<P: AsRef<Path>>(path: P, animation: &Animation) -> Result<(), AnimationError> {
    let mut seanim = BufWriter::new(File::create(path.as_ref().with_extension("seanim"))?);

    let mut header = SEAnimHeader {
        magic: [b'S', b'E', b'A', b'n', b'i', b'm'],
        version: 0x1,
        header_size: 0x1C,
        animation_type: 0,
        looping: animation.looping,
        data_presence_flags: 0,
        data_property_flags: 0,
        reserved: 0,
        framerate: animation.framerate,
        frame_count: 0,
        bone_count: 0,
        bone_modifiers: 0,
        reserved2: [0, 0, 0],
        notification_count: 0,
    };

    let animation_type = animation.average_data_type();

    header.animation_type = match animation_type {
        CurveDataType::Absolute => SEAnimAnimationType::Absolute as u8,
        CurveDataType::Additive => SEAnimAnimationType::Additive as u8,
        CurveDataType::Relative => SEAnimAnimationType::Relative as u8,
    };

    let frame_count = animation.frame_count();
    let notification_count = animation.notification_count();

    let mut bone_names = HashSet::new();
    let mut has_locations = false;
    let mut has_rotations = false;
    let mut has_scale = false;
    let mut has_notifications = false;

    for curve in &animation.curves {
        if matches!(
            curve.attribute(),
            CurveAttribute::Translate | CurveAttribute::Rotation | CurveAttribute::Scale
        ) {
            bone_names.insert(curve.name().to_string());
        }

        match curve.attribute() {
            CurveAttribute::Translate => has_locations = true,
            CurveAttribute::Rotation => has_rotations = true,
            CurveAttribute::Scale => has_scale = true,
            CurveAttribute::Notetrack => has_notifications = true,
            _ => {
                // Unsupported.
            }
        }
    }

    if has_locations {
        header.data_presence_flags |= SEAnimDataPresenceFlags::BoneLoc as u8;
    }

    if has_rotations {
        header.data_presence_flags |= SEAnimDataPresenceFlags::BoneRot as u8;
    }

    if has_scale {
        header.data_presence_flags |= SEAnimDataPresenceFlags::BoneScale as u8;
    }

    if has_notifications {
        header.data_presence_flags |= SEAnimDataPresenceFlags::PresenceNote as u8;
    }

    let bone_names: Vec<_> = bone_names.into_iter().collect();
    let bone_count = bone_names.len();

    let mut bone_modifiers: HashMap<usize, CurveDataType> = HashMap::new();

    for curve in &animation.curves {
        let index = match bone_names.iter().position(|x| x == curve.name()) {
            Some(index) => index,
            None => continue,
        };

        if bone_modifiers.contains_key(&index) {
            continue;
        }

        if curve.data_type() == animation_type {
            continue;
        }

        bone_modifiers.insert(index, curve.data_type());
    }

    header.frame_count = frame_count;
    header.bone_count = bone_count as u32;
    header.bone_modifiers = bone_modifiers.len() as u8;
    header.notification_count = notification_count as u32;

    seanim.write_struct(header)?;

    for bone in &bone_names {
        seanim.write_null_terminated_string(bone)?;
    }

    for bone_modifier in bone_modifiers {
        if bone_count <= 0xFF {
            seanim.write_all(&(bone_modifier.0 as u8).to_le_bytes())?;
        } else if bone_count <= 0xFFFF {
            seanim.write_all(&(bone_modifier.0 as u16).to_le_bytes())?;
        } else {
            seanim.write_all(&(bone_modifier.0 as u32).to_le_bytes())?;
        }

        match bone_modifier.1 {
            CurveDataType::Absolute => {
                seanim.write_all(&(SEAnimAnimationType::Absolute as u8).to_le_bytes())?;
            }
            CurveDataType::Additive => {
                seanim.write_all(&(SEAnimAnimationType::Additive as u8).to_le_bytes())?;
            }
            CurveDataType::Relative => {
                seanim.write_all(&(SEAnimAnimationType::Relative as u8).to_le_bytes())?;
            }
        }
    }

    for bone in bone_names {
        seanim.write_all(&[0])?;

        let curves: Vec<&_> = animation
            .curves
            .iter()
            .filter(|x| x.name() == bone && !matches!(x.attribute(), CurveAttribute::Notetrack))
            .collect();

        let mut positions = 0;
        let mut rotations = 0;
        let mut scales = 0;

        for curve in &curves {
            match curve.attribute() {
                CurveAttribute::Translate => positions += curve.keyframes().len(),
                CurveAttribute::Rotation => rotations += curve.keyframes().len(),
                CurveAttribute::Scale => scales += curve.keyframes().len(),
                _ => {
                    // Unsupported.
                }
            }
        }

        if has_locations {
            if frame_count <= 0xFF {
                seanim.write_all(&(positions as u8).to_le_bytes())?;
            } else if frame_count <= 0xFFFF {
                seanim.write_all(&(positions as u16).to_le_bytes())?;
            } else {
                seanim.write_all(&(positions as u32).to_le_bytes())?;
            }

            if let Some(curve) = curves
                .iter()
                .find(|x| matches!(x.attribute(), CurveAttribute::Translate))
            {
                for keyframe in curve.keyframes() {
                    if frame_count <= 0xFF {
                        seanim.write_all(&(keyframe.time as u8).to_le_bytes())?;
                    } else if frame_count <= 0xFFFF {
                        seanim.write_all(&(keyframe.time as u16).to_le_bytes())?;
                    } else {
                        seanim.write_all(&keyframe.time.to_le_bytes())?;
                    }

                    if let KeyframeValue::Vector3(position) = keyframe.value {
                        seanim.write_struct(position)?;
                    } else {
                        seanim.write_struct(Vector3::zero())?;
                    }
                }
            }
        }

        if has_rotations {
            if frame_count <= 0xFF {
                seanim.write_all(&(rotations as u8).to_le_bytes())?;
            } else if frame_count <= 0xFFFF {
                seanim.write_all(&(rotations as u16).to_le_bytes())?;
            } else {
                seanim.write_all(&(rotations as u32).to_le_bytes())?;
            }

            if let Some(curve) = curves
                .iter()
                .find(|x| matches!(x.attribute(), CurveAttribute::Rotation))
            {
                for keyframe in curve.keyframes() {
                    if frame_count <= 0xFF {
                        seanim.write_all(&(keyframe.time as u8).to_le_bytes())?;
                    } else if frame_count <= 0xFFFF {
                        seanim.write_all(&(keyframe.time as u16).to_le_bytes())?;
                    } else {
                        seanim.write_all(&keyframe.time.to_le_bytes())?;
                    }

                    if let KeyframeValue::Quaternion(rotation) = keyframe.value {
                        seanim.write_struct(rotation)?;
                    } else {
                        seanim.write_struct(Quaternion::identity())?;
                    }
                }
            }
        }

        if has_scale {
            if frame_count <= 0xFF {
                seanim.write_all(&(scales as u8).to_le_bytes())?;
            } else if frame_count <= 0xFFFF {
                seanim.write_all(&(scales as u16).to_le_bytes())?;
            } else {
                seanim.write_all(&(scales as u32).to_le_bytes())?;
            }

            if let Some(curve) = curves
                .iter()
                .find(|x| matches!(x.attribute(), CurveAttribute::Scale))
            {
                for keyframe in curve.keyframes() {
                    if frame_count <= 0xFF {
                        seanim.write_all(&(keyframe.time as u8).to_le_bytes())?;
                    } else if frame_count <= 0xFFFF {
                        seanim.write_all(&(keyframe.time as u16).to_le_bytes())?;
                    } else {
                        seanim.write_all(&keyframe.time.to_le_bytes())?;
                    }

                    if let KeyframeValue::Vector3(scale) = keyframe.value {
                        seanim.write_struct(scale)?;
                    } else {
                        seanim.write_struct(Vector3::one())?;
                    }
                }
            }
        }
    }

    for curve in &animation.curves {
        if !matches!(curve.attribute(), CurveAttribute::Notetrack) {
            continue;
        }

        for keyframe in curve.keyframes() {
            if frame_count <= 0xFF {
                seanim.write_all(&(keyframe.time as u8).to_le_bytes())?;
            } else if frame_count <= 0xFFFF {
                seanim.write_all(&(keyframe.time as u16).to_le_bytes())?;
            } else {
                seanim.write_all(&keyframe.time.to_le_bytes())?;
            }

            seanim.write_null_terminated_string(curve.name())?;
        }
    }

    Ok(())
}
