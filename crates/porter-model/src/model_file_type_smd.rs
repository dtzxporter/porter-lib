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

    writeln!(
        smd,
        "version 1\n// Exported by PorterLib\n// Please credit DTZxPorter for use of this asset!\nnodes"
    )?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        writeln!(
            smd,
            "{} \"{}\" {}",
            bone_index,
            bone.name
                .as_ref()
                .unwrap_or(&format!("porter_bone_{}", bone_index)),
            bone.parent
        )?;
    }

    writeln!(smd, "end\nskeleton\ntime 0")?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        let local_rotation = bone.local_rotation.to_euler(Angles::Radians);
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

        let material = match mesh.material {
            Some(index) => model.materials[index].name.as_str(),
            None => "default_material",
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
