use std::fs::File;
use std::io::Write;
use std::path::Path;

use porter_math::Angles;
use porter_math::Vector2;

use porter_utils::BufferWriteExt;

use crate::Model;
use crate::ModelError;

/// Utility to write a face vertex and it's information.
macro_rules! write_face_vertex {
    ($smd:ident, $mesh:ident, $face:expr) => {
        let vertex = $mesh.vertices.vertex($face as usize);

        let position = vertex.position();
        let normal = vertex.normal();
        let uv = if $mesh.vertices.uv_layers() > 0 {
            vertex.uv(0)
        } else {
            Vector2::zero()
        };

        let weights = vertex.unique_weights();

        write!(
            $smd,
            "0 {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {}",
            position.x,
            position.y,
            position.z,
            normal.x,
            normal.y,
            normal.z,
            uv.x,
            1.0 - uv.y,
            weights.len()
        )?;

        for (bone, value) in weights {
            write!($smd, " {} {:.6}", bone, value)?;
        }

        writeln!($smd)?;
    };
}

/// Writes a model in smd format to the given path.
pub fn to_smd<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut smd = File::create(path.as_ref().with_extension("smd"))?.buffer_write();

    writeln!(smd, "version 1")?;
    writeln!(smd, "// Exported by PorterLib")?;

    if !cfg!(feature = "debrand") {
        writeln!(smd, "// Please credit DTZxPorter for use of this asset!")?;
    }

    writeln!(smd, "nodes")?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        writeln!(
            smd,
            "{} \"{}\" {}",
            bone_index,
            bone.name
                .as_deref()
                .map(sanitize_smd_bone_str)
                .unwrap_or_else(|| format!("porter_bone_{}", bone_index)),
            bone.parent
        )?;
    }

    writeln!(smd, "end\nskeleton\ntime 0")?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        let local_rotation = bone
            .local_rotation
            .to_euler(Angles::Radians);
        let local_position = bone.local_position;

        writeln!(
            smd,
            "{} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6}",
            bone_index,
            local_position.x,
            local_position.y,
            local_position.z,
            local_rotation.x,
            local_rotation.y,
            local_rotation.z
        )?;
    }

    writeln!(smd, "end")?;

    for mesh in &model.meshes {
        writeln!(smd, "triangles")?;

        let material = if let Some(material_index) = mesh.material
            && let Some(material) = model.materials.get(material_index)
        {
            &sanitize_smd_mat_str(&material.name)
        } else {
            "default_material"
        };

        for face in &mesh.faces {
            writeln!(smd, "{}", material)?;

            write_face_vertex!(smd, mesh, face.i3);
            write_face_vertex!(smd, mesh, face.i2);
            write_face_vertex!(smd, mesh, face.i1);
        }

        writeln!(smd, "end")?;
    }

    Ok(())
}

/// Sanitizes a smd bone string.
fn sanitize_smd_bone_str(str: &str) -> String {
    // Smd bones are string quoted, so we can allow spaces and some symbols.
    // But we need to make sure we don't allow path chars.
    str.trim()
        .chars()
        .map(|ch| match ch {
            '/' | '?' | '<' | '>' | '\\' | ':' | '*' | '|' | '"' | '\r' | '\n' => '_',
            _ => ch,
        })
        .collect()
}

/// Sanitizes a smd mat string.
fn sanitize_smd_mat_str(str: &str) -> String {
    // Smd materials are regular Ascii strings to only allow those characters.
    str.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
