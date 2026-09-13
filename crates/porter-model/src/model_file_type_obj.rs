use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use porter_macros::assert_const;

use porter_utils::BufferWriteExt;

use crate::MaterialUsage;
use crate::Model;
use crate::ModelError;

/// Writes a model in obj format to the given path.
pub fn to_obj<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let path = path.as_ref();

    let mut obj = File::create(path.with_extension("obj"))?.buffer_write();
    let mut mtl = File::create(path.with_extension("mtl"))?.buffer_write();

    writeln!(obj, "# Exported by PorterLib")?;

    if !cfg!(feature = "debrand") {
        writeln!(obj, "# Please credit DTZxPorter for use of this asset!\n")?;
    }

    writeln!(
        obj,
        "\nmtllib {}.mtl\n",
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
    )?;

    for mesh in &model.meshes {
        for face in &mesh.faces {
            let vt1 = mesh
                .vertices
                .vertex(face.i1 as usize)
                .position();
            let vt2 = mesh
                .vertices
                .vertex(face.i2 as usize)
                .position();
            let vt3 = mesh
                .vertices
                .vertex(face.i3 as usize)
                .position();

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
            let vt1 = mesh
                .vertices
                .vertex(face.i1 as usize)
                .uv(0);
            let vt2 = mesh
                .vertices
                .vertex(face.i2 as usize)
                .uv(0);
            let vt3 = mesh
                .vertices
                .vertex(face.i3 as usize)
                .uv(0);

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
            let vt1 = mesh
                .vertices
                .vertex(face.i1 as usize)
                .normal();
            let vt2 = mesh
                .vertices
                .vertex(face.i2 as usize)
                .normal();
            let vt3 = mesh
                .vertices
                .vertex(face.i3 as usize)
                .normal();

            writeln!(
                obj,
                "vn {:.6} {:.6} {:.6}\nvn {:.6} {:.6} {:.6}\nvn {:.6} {:.6} {:.6}",
                vt1.x, vt1.y, vt1.z, vt2.x, vt2.y, vt2.z, vt3.x, vt3.y, vt3.z
            )?;
        }
    }

    let mut global_face_index = 1;

    for (i, mesh) in model.meshes.iter().enumerate() {
        let name = mesh
            .name
            .as_deref()
            .map(sanitize_obj_str)
            .unwrap_or_else(|| format!("PorterMesh{}", i));

        if let Some(material_index) = mesh.material
            && let Some(material) = model.materials.get(material_index)
        {
            writeln!(
                obj,
                "g {}\nusemtl {}",
                name,
                sanitize_obj_str(&material.name)
            )?;
        } else {
            writeln!(obj, "g {}\nusemtl default_material", name)?;
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

    assert_const!(MATERIAL_MAPPINGS.len() == MaterialUsage::Count as usize);

    for material in &model.materials {
        writeln!(
            mtl,
            "newmtl {}\nillium 4\nKd 0.00 0.00 0.00\nKa 0.00 0.00 0.00\nKs 0.50 0.50 0.50",
            sanitize_obj_str(&material.name)
        )?;

        let mut used_slots: HashSet<&str> = HashSet::new();

        for texture in &material.textures {
            if texture.is_empty() {
                continue;
            }

            let slot = MATERIAL_MAPPINGS[texture.usage as usize];

            if !used_slots.insert(slot) {
                continue;
            }

            writeln!(mtl, "{} {}", slot, texture.file_path)?;
        }
    }

    Ok(())
}

/// Sanitizes a obj str.
fn sanitize_obj_str(str: &str) -> String {
    // Obj is such an old format, it's impossible to verify the logic of all importers
    // So to be safe, only allow Ascii and separator characters.
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
