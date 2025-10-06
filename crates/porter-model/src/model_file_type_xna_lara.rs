use std::fs::File;
use std::io::Write;
use std::path::Path;

use porter_math::normalize_array_f32;

use porter_utils::BufferWriteExt;

use crate::Model;
use crate::ModelError;
use crate::VertexColor;
use crate::WeightBoneId;

/// Writes a model in xna lara format to the given path.
pub fn to_xna_lara<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut xna = File::create(path.as_ref().with_extension("mesh.ascii"))?.buffer_write();

    writeln!(xna, "{}", model.skeleton.bones.len())?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        let world_position = bone.world_position;

        writeln!(
            xna,
            "{}\n{}\n{:.6} {:.6} {:.6}",
            bone.name
                .as_ref()
                .unwrap_or(&format!("porter_bone_{}", bone_index)),
            bone.parent,
            world_position.x,
            world_position.y,
            world_position.z
        )?;
    }

    writeln!(xna, "{}", model.meshes.len())?;

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        writeln!(
            xna,
            "PorterMesh{}\n{}\n{}",
            mesh_index,
            mesh.vertices.uv_layers(),
            mesh.vertices.uv_layers()
        )?;

        for i in 0..mesh.vertices.uv_layers() {
            if let Some(material_index) = mesh.material
                && let Some(diffuse) = model.materials[material_index].base_color_texture()
            {
                writeln!(xna, "{}\n{}", diffuse.file_name, i)?;
            } else {
                writeln!(xna, "default_material\n{}", i)?;
            }
        }

        writeln!(xna, "{}", mesh.vertices.len())?;

        for i in 0..mesh.vertices.len() {
            let vertex = mesh.vertices.vertex(i);

            let position = vertex.position();
            let normal = vertex.normal();
            let color = if mesh.vertices.colors() > 0 {
                vertex.color(0)
            } else {
                VertexColor::new(255, 255, 255, 255)
            };

            writeln!(
                xna,
                "{:.6} {:.6} {:.6}\n{:.6} {:.6} {:.6}\n{} {} {} {}",
                position.x,
                position.y,
                position.z,
                normal.x,
                normal.y,
                normal.z,
                color.r,
                color.g,
                color.b,
                color.a
            )?;

            for uv in 0..mesh.vertices.uv_layers() {
                let uv = vertex.uv(uv);

                writeln!(xna, "{:.6} {:.6}", uv.x, uv.y)?;
            }

            let mut bones: [WeightBoneId; 4] = [0; 4];
            let mut values: [f32; 4] = [0.0; 4];

            for w in 0..mesh.vertices.maximum_influence() {
                let mut weight = vertex.weight(w);

                for i in 0..4 {
                    if weight.value > values[i] {
                        let temp_bone = bones[i];
                        let temp_value = values[i];

                        bones[i] = weight.bone;
                        values[i] = weight.value;

                        weight.bone = temp_bone;
                        weight.value = temp_value;
                    }
                }
            }

            values = normalize_array_f32(values);

            writeln!(
                xna,
                "{} {} {} {}\n{:.6} {:.6} {:.6} {:.6}",
                bones[0], bones[1], bones[2], bones[3], values[0], values[1], values[2], values[3]
            )?;
        }

        writeln!(xna, "{}", mesh.faces.len())?;

        for face in &mesh.faces {
            writeln!(xna, "{} {} {}", { face.i1 }, { face.i2 }, { face.i3 })?;
        }
    }

    Ok(())
}
