use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use static_assertions::const_assert;

use crate::MaterialTextureRefUsage;
use crate::Model;
use crate::ModelError;

/// Writes a model in obj format to the given path.
pub fn to_obj<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let path = path.as_ref();

    let mut obj = BufWriter::new(File::create(path.with_extension("obj"))?);
    let mut mtl = BufWriter::new(File::create(path.with_extension("mtl"))?);

    writeln!(
        obj,
        "# Exported by PorterLib\n# Please credit DTZxPorter for use of this asset!\n"
    )?;

    writeln!(
        obj,
        "\nmtllib {}\n",
        path.file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    )?;

    for mesh in &model.meshes {
        for face in &mesh.faces {
            let vt1 = mesh.vertices.vertex(face.i1 as usize).position();
            let vt2 = mesh.vertices.vertex(face.i2 as usize).position();
            let vt3 = mesh.vertices.vertex(face.i3 as usize).position();

            writeln!(
                obj,
                "v {:.6} {:.6} {:.6}\nv {:.6} {:.6} {:.6}\nv {:.6} {:.6} {:.6}",
                vt1.x, vt1.y, vt1.z, vt2.x, vt2.y, vt2.z, vt3.x, vt3.y, vt3.z
            )?;
        }
    }

    for mesh in &model.meshes {
        if mesh.vertices.uv_layers() == 0 {
            continue;
        }

        for face in &mesh.faces {
            let vt1 = mesh.vertices.vertex(face.i1 as usize).uv(0);
            let vt2 = mesh.vertices.vertex(face.i2 as usize).uv(0);
            let vt3 = mesh.vertices.vertex(face.i3 as usize).uv(0);

            writeln!(
                obj,
                "vt {:.6} {:.6}\nvt {:.6} {:.6}\nvt {:.6} {:.6}",
                vt1.x,
                1.0 - vt1.y,
                vt2.x,
                1.0 - vt2.y,
                vt3.x,
                1.0 - vt3.y,
            )?;
        }
    }

    for mesh in &model.meshes {
        for face in &mesh.faces {
            let vt1 = mesh.vertices.vertex(face.i1 as usize).normal();
            let vt2 = mesh.vertices.vertex(face.i2 as usize).normal();
            let vt3 = mesh.vertices.vertex(face.i3 as usize).normal();

            writeln!(
                obj,
                "vn {:.6} {:.6} {:.6}\nvn {:.6} {:.6} {:.6}\nvn {:.6} {:.6} {:.6}",
                vt1.x, vt1.y, vt1.z, vt2.x, vt2.y, vt2.z, vt3.x, vt3.y, vt3.z
            )?;
        }
    }

    let mut global_face_index = 1;

    for mesh in &model.meshes {
        if let Some(material_index) = mesh.material {
            writeln!(
                obj,
                "g {}\nusemtl {}",
                model.materials[material_index].name, model.materials[material_index].name
            )?;
        } else {
            writeln!(obj, "g default_material\nusemtl default_material")?;
        }

        let use_tex_coords = mesh.vertices.uv_layers() > 0;

        for _ in &mesh.faces {
            if use_tex_coords {
                writeln!(
                    obj,
                    "f {}/{}/{} {}/{}/{} {}/{}/{}",
                    global_face_index + 2,
                    global_face_index + 2,
                    global_face_index + 2,
                    global_face_index + 1,
                    global_face_index + 1,
                    global_face_index + 1,
                    global_face_index,
                    global_face_index,
                    global_face_index
                )?;
            } else {
                writeln!(
                    obj,
                    "f {}//{} {}//{} {}//{}",
                    global_face_index + 2,
                    global_face_index + 2,
                    global_face_index + 1,
                    global_face_index + 1,
                    global_face_index,
                    global_face_index
                )?;
            }

            global_face_index += 3;
        }
    }

    // These must match the enumeration for texture usage.
    const MATERIAL_MAPPINGS: [&str; 14] = [
        "map_Kd",    // Albedo (Diffuse)
        "map_Kd",    // Diffuse (Diffuse)
        "map_Ks",    // Specular (Specular)
        "norm",      // Normal (Normal extension)
        "map_Ke",    // Emissive (Emissive extension)
        "emask",     // Emissive Mask (Custom extension)
        "estrength", // Emissive Strength (Custom extension)
        "map_Pr",    // Gloss (Roughness extension)
        "map_Pr",    // Roughness (Roughness extension)
        "map_RMA",   // AmbientOcclusion (RMA Extension)
        "aniso",     // Anisotropy (Anisotropy extension)
        "detail",    // Cavity (Custom extension)
        "map_Pm",    // Metallic (Metallic extension)
        "map_Unk",   // Unknown (Custom extension)
    ];

    const_assert!(MATERIAL_MAPPINGS.len() == MaterialTextureRefUsage::Count as usize);

    for material in &model.materials {
        writeln!(
            mtl,
            "newmtl {}\nillium 4\nKd 0.00 0.00 0.00\nKa 0.00 0.00 0.00\nKs 0.50 0.50 0.50",
            material.name
        )?;

        for texture in &material.textures {
            if texture.is_empty() {
                continue;
            }

            writeln!(
                mtl,
                "{} {}",
                MATERIAL_MAPPINGS[texture.texture_usage as usize], texture.file_name
            )?;
        }
    }

    Ok(())
}
