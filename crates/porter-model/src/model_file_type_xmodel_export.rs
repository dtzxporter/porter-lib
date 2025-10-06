use std::fs::File;
use std::io::Write;
use std::path::Path;

use porter_utils::BufferWriteExt;

use crate::Model;
use crate::ModelError;

/// Utility to write a face vertex and it's info.
macro_rules! write_face_vertex {
    ($xmodel:ident, $mesh:ident, $vertex_count:ident, $vertex_index:ident, $face:expr) => {
        if $vertex_count > u16::MAX as usize {
            writeln!($xmodel, "VERT32 {}", $face as usize + $vertex_index)?;
        } else {
            writeln!($xmodel, "VERT {}", $face as usize + $vertex_index)?;
        }

        let vertex = $mesh.vertices.vertex($face as usize);

        let normal = vertex.normal();

        let color = if $mesh.vertices.colors() > 0 {
            vertex.color(0)
        } else {
            Default::default()
        };

        let uv_layers = $mesh.vertices.uv_layers();

        write!(
            $xmodel,
            concat!(
                "NORMAL {:.6} {:.6} {:.6}\n",
                "COLOR {:.6} {:.6} {:.6} {:.6}\n",
                "UV {}"
            ),
            normal.x,
            normal.y,
            normal.z,
            (color.r as f32) / 255.0,
            (color.g as f32) / 255.0,
            (color.b as f32) / 255.0,
            (color.a as f32) / 255.0,
            uv_layers.max(1)
        )?;

        for i in 0..uv_layers.max(1) {
            let uv = if i < uv_layers {
                vertex.uv(i)
            } else {
                Default::default()
            };

            write!($xmodel, " {:.6} {:.6}", uv.x, uv.y)?;
        }

        writeln!($xmodel)?;
    };
}

/// Writes a model in xmodel export format to the given path.
pub fn to_xmodel_export<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut xmodel = File::create(path.as_ref().with_extension("xmodel_export"))?.buffer_write();

    writeln!(
        xmodel,
        "// Exported by PorterLib\n// Please credit DTZxPorter for use of this asset!"
    )?;

    writeln!(
        xmodel,
        "MODEL\nVERSION 6\nNUMBONES {}",
        model.skeleton.bones.len()
    )?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        writeln!(
            xmodel,
            "BONE {} {} \"{}\"",
            bone_index,
            bone.parent,
            bone.name
                .as_ref()
                .unwrap_or(&format!("porter_bone_{}", bone_index))
        )?;
    }

    writeln!(xmodel)?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        let rotation = bone.world_rotation.to_4x4();
        let position = bone.world_position;
        let scale = bone.local_scale;

        writeln!(
            xmodel,
            "BONE {}\nOFFSET {:.6}, {:.6}, {:.6}\nSCALE {:.6}, {:.6}, {:.6}\nX {:.6}, {:.6}, {:.6}\nY {:.6}, {:.6}, {:.6}\nZ {:.6}, {:.6}, {:.6}\n",
            bone_index,
            position.x,
            position.y,
            position.z,
            scale.x,
            scale.y,
            scale.z,
            rotation.mat::<0, 0>(),
            rotation.mat::<0, 1>(),
            rotation.mat::<0, 2>(),
            rotation.mat::<1, 0>(),
            rotation.mat::<1, 1>(),
            rotation.mat::<1, 2>(),
            rotation.mat::<2, 0>(),
            rotation.mat::<2, 1>(),
            rotation.mat::<2, 2>(),
        )?;
    }

    let vertex_count = model.vertex_count();

    if vertex_count > u16::MAX as usize {
        writeln!(xmodel, "NUMVERTS32 {}", vertex_count)?;
    } else {
        writeln!(xmodel, "NUMVERTS {}", vertex_count)?;
    }

    let mut vertex_index: usize = 0;

    for mesh in &model.meshes {
        for i in 0..mesh.vertices.len() {
            let vertex = mesh.vertices.vertex(i);
            let position = vertex.position();

            if vertex_count > u16::MAX as usize {
                writeln!(
                    xmodel,
                    "VERT32 {}\nOFFSET {:.6}, {:.6}, {:.6}",
                    vertex_index, position.x, position.y, position.z
                )?;
            } else {
                writeln!(
                    xmodel,
                    "VERT {}\nOFFSET {:.6}, {:.6}, {:.6}",
                    vertex_index, position.x, position.y, position.z
                )?;
            }

            let weights = vertex.unique_weights();

            writeln!(xmodel, "BONES {}", weights.len())?;

            for (bone, value) in weights {
                writeln!(xmodel, "BONE {} {:.6}", bone, value)?;
            }

            vertex_index += 1;
        }
    }

    let face_count = model.face_count();

    let mut vertex_index: usize = 0;
    let mut needs_default_material = false;

    writeln!(xmodel, "NUMFACES {}", face_count)?;

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        for face in &mesh.faces {
            let material_index = match mesh.material {
                Some(index) => index,
                None => {
                    needs_default_material = true;
                    model.materials.len()
                }
            };

            if mesh_index > u8::MAX as usize {
                writeln!(xmodel, "TRI16 {} {} 0 0", mesh_index, material_index)?;
            } else {
                writeln!(xmodel, "TRI {} {} 0 0", mesh_index, material_index)?;
            }

            write_face_vertex!(xmodel, mesh, vertex_count, vertex_index, face.i3);
            write_face_vertex!(xmodel, mesh, vertex_count, vertex_index, face.i1);
            write_face_vertex!(xmodel, mesh, vertex_count, vertex_index, face.i2);
        }

        vertex_index += mesh.vertices.len();
    }

    writeln!(xmodel, "NUMOBJECTS {}", model.meshes.len())?;

    for i in 0..model.meshes.len() {
        writeln!(xmodel, "OBJECT {} \"PorterMesh_{}\"", i, i)?;
    }

    if needs_default_material {
        writeln!(xmodel, "NUMMATERIALS {}", model.materials.len() + 1)?;
    } else {
        writeln!(xmodel, "NUMMATERIALS {}", model.materials.len())?;
    }

    for (material_index, material) in model.materials.iter().enumerate() {
        write!(
            xmodel,
            "MATERIAL {} \"{}\" \"Phong\" \"",
            material_index, material.name
        )?;

        if let Some(diffuse) = material.base_color_texture() {
            write!(xmodel, "color:{}", diffuse.file_name)?;
        }

        writeln!(
            xmodel,
            "\"\nCOLOR 0.000000 0.000000 0.000000 1.000000\nTRANSPARENCY 0.000000 0.000000 0.000000 1.000000\nAMBIENTCOLOR 1.000000 1.000000 1.000000 1.000000\nINCANDESCENCE 0.000000 0.000000 0.000000 1.000000\nCOEFFS 0.800000 0.000000\nGLOW 0.000000 0\nREFRACTIVE 6 1.000000\nSPECULARCOLOR 0.500000 0.500000 0.500000 1.000000\nREFLECTIVECOLOR 0.000000 0.000000 0.000000 1.000000\nREFLECTIVE 1 0.500000\nBLINN -1.000000 -1.000000\nPHONG 20.000000"
        )?;
    }

    if needs_default_material {
        writeln!(
            xmodel,
            "MATERIAL {} \"default_material\" \"Phong\" \"\"\nCOLOR 0.000000 0.000000 0.000000 1.000000\nTRANSPARENCY 0.000000 0.000000 0.000000 1.000000\nAMBIENTCOLOR 1.000000 1.000000 1.000000 1.000000\nINCANDESCENCE 0.000000 0.000000 0.000000 1.000000\nCOEFFS 0.800000 0.000000\nGLOW 0.000000 0\nREFRACTIVE 6 1.000000\nSPECULARCOLOR 0.500000 0.500000 0.500000 1.000000\nREFLECTIVECOLOR 0.000000 0.000000 0.000000 1.000000\nREFLECTIVE 1 0.500000\nBLINN -1.000000 -1.000000\nPHONG 20.000000",
            model.materials.len()
        )?;
    }

    Ok(())
}
