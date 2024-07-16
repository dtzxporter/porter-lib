use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use porter_math::Vector3;

use porter_utils::StringWriteExt;
use porter_utils::StructWriteExt;

use crate::MaterialTextureRefUsage;
use crate::Model;
use crate::ModelError;
use crate::VertexColor;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SEModelHeader {
    magic: [u8; 0x7],
    version: u16,
    header_size: u16,
    file_data_presence_flags: u8,
    bone_data_presence_flags: u8,
    mesh_data_presence_flags: u8,
    bone_count: u32,
    mesh_count: u32,
    material_count: u32,
    reserved: [u8; 0x3],
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum SEModelDataPresenceFlags {
    Bone = 1 << 0,
    Mesh = 1 << 1,
    Materials = 1 << 2,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum SEModelBonePresenceFlags {
    GlobalMatrix = 1 << 0,
    LocalMatrix = 1 << 1,
    Scales = 1 << 2,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum SEModelMeshPresenceFlags {
    UVSet = 1 << 0,
    Normals = 1 << 1,
    Color = 1 << 2,
    Weights = 1 << 3,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SEModelMeshHeader {
    pub flags: u8,
    pub uv_layer_count: u8,
    pub maximum_influence: u8,
    pub vertex_count: u32,
    pub face_count: u32,
}

/// Writes a model in semodel format to the given path.
pub fn to_semodel<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut semodel = BufWriter::new(File::create(path.as_ref().with_extension("semodel"))?);

    let mut header = SEModelHeader {
        magic: [b'S', b'E', b'M', b'o', b'd', b'e', b'l'],
        version: 0x1,
        header_size: 0x14,
        file_data_presence_flags: 0,
        bone_data_presence_flags: 0,
        mesh_data_presence_flags: 0,
        bone_count: model.skeleton.bones.len() as u32,
        mesh_count: model.meshes.len() as u32,
        material_count: model.materials.len() as u32,
        reserved: [0; 3],
    };

    if !model.skeleton.bones.is_empty() {
        header.file_data_presence_flags |= SEModelDataPresenceFlags::Bone as u8;
    }

    if !model.meshes.is_empty() {
        header.file_data_presence_flags |= SEModelDataPresenceFlags::Mesh as u8;
    }

    if !model.materials.is_empty() {
        header.file_data_presence_flags |= SEModelDataPresenceFlags::Materials as u8;
    }

    let mut has_world_matrix = false;
    let mut has_local_matrix = false;
    let mut has_scale = false;

    for bone in &*model.skeleton.bones {
        if bone.local_position.is_some() || bone.local_rotation.is_some() {
            has_local_matrix = true;
        }

        if bone.world_position.is_some() || bone.world_rotation.is_some() {
            has_world_matrix = true;
        }

        if bone.local_scale.is_some() {
            has_scale = true;
        }

        if has_scale && has_local_matrix && has_world_matrix {
            break;
        }
    }

    if has_world_matrix {
        header.bone_data_presence_flags |= SEModelBonePresenceFlags::GlobalMatrix as u8;
    }

    if has_local_matrix {
        header.bone_data_presence_flags |= SEModelBonePresenceFlags::LocalMatrix as u8;
    }

    if has_scale {
        header.bone_data_presence_flags |= SEModelBonePresenceFlags::Scales as u8;
    }

    header.mesh_data_presence_flags |= SEModelMeshPresenceFlags::Normals as u8;
    header.mesh_data_presence_flags |= SEModelMeshPresenceFlags::UVSet as u8;

    let mut has_colors = false;

    for mesh in &model.meshes {
        if mesh.vertices.colors() {
            has_colors = true;
            break;
        }
    }

    if has_colors {
        header.mesh_data_presence_flags |= SEModelMeshPresenceFlags::Color as u8;
    }

    if !model.skeleton.bones.is_empty() {
        header.mesh_data_presence_flags |= SEModelMeshPresenceFlags::Weights as u8;
    }

    semodel.write_struct(header)?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        semodel.write_null_terminated_string(
            bone.name
                .as_ref()
                .unwrap_or(&format!("porter_bone_{}", bone_index)),
        )?;
    }

    for bone in &*model.skeleton.bones {
        semodel.write_all(&[0])?;
        semodel.write_all(&bone.parent.to_le_bytes())?;

        if has_world_matrix {
            semodel.write_struct(bone.world_position.unwrap_or_default())?;
            semodel.write_struct(bone.world_rotation.unwrap_or_default())?;
        }

        if has_local_matrix {
            semodel.write_struct(bone.local_position.unwrap_or_default())?;
            semodel.write_struct(bone.local_rotation.unwrap_or_default())?;
        }

        if has_scale {
            semodel.write_struct(bone.local_scale.unwrap_or_else(Vector3::one))?;
        }
    }

    for mesh in &model.meshes {
        let mesh_header = SEModelMeshHeader {
            flags: 0,
            uv_layer_count: mesh.vertices.uv_layers() as u8,
            maximum_influence: mesh.vertices.maximum_influence() as u8,
            vertex_count: mesh.vertices.len() as u32,
            face_count: mesh.faces.len() as u32,
        };

        semodel.write_struct(mesh_header)?;

        for i in 0..mesh.vertices.len() {
            semodel.write_struct(mesh.vertices.vertex(i).position())?;
        }

        for i in 0..mesh.vertices.len() {
            for uv in 0..mesh.vertices.uv_layers() {
                semodel.write_struct(mesh.vertices.vertex(i).uv(uv))?;
            }
        }

        for i in 0..mesh.vertices.len() {
            semodel.write_struct(mesh.vertices.vertex(i).normal())?;
        }

        if has_colors {
            for i in 0..mesh.vertices.len() {
                if mesh.vertices.colors() {
                    semodel.write_struct(mesh.vertices.vertex(i).color())?;
                } else {
                    semodel.write_struct(VertexColor::new(255, 255, 255, 255))?;
                }
            }
        }

        if !model.skeleton.bones.is_empty() && mesh.vertices.maximum_influence() > 0 {
            for i in 0..mesh.vertices.len() {
                let vertex = mesh.vertices.vertex(i);

                for w in 0..mesh.vertices.maximum_influence() {
                    let weight = vertex.weight(w);

                    if model.skeleton.bones.len() <= u8::MAX as usize {
                        semodel.write_all(&(weight.bone as u8).to_le_bytes())?;
                    } else if model.skeleton.bones.len() <= u16::MAX as usize {
                        semodel.write_all(&(weight.bone).to_le_bytes())?;
                    } else {
                        semodel.write_all(&(weight.bone as u32).to_le_bytes())?;
                    }

                    semodel.write_all(&weight.value.to_le_bytes())?;
                }
            }
        }

        for face in &mesh.faces {
            if mesh.vertices.len() <= u8::MAX as usize {
                semodel.write_all(&(face.i1 as u8).to_le_bytes())?;
                semodel.write_all(&(face.i2 as u8).to_le_bytes())?;
                semodel.write_all(&(face.i3 as u8).to_le_bytes())?;
            } else if mesh.vertices.len() <= u16::MAX as usize {
                semodel.write_all(&(face.i1 as u16).to_le_bytes())?;
                semodel.write_all(&(face.i2 as u16).to_le_bytes())?;
                semodel.write_all(&(face.i3 as u16).to_le_bytes())?;
            } else {
                semodel.write_all(&face.i1.to_le_bytes())?;
                semodel.write_all(&face.i2.to_le_bytes())?;
                semodel.write_all(&face.i3.to_le_bytes())?;
            }
        }

        for i in 0..mesh.vertices.uv_layers() {
            if i < mesh.materials.len() {
                semodel.write_all(&(mesh.materials[i] as i32).to_le_bytes())?;
            } else {
                semodel.write_all(&(-1i32).to_le_bytes())?;
            }
        }
    }

    for material in &model.materials {
        semodel.write_null_terminated_string(&material.name)?;
        semodel.write_all(&[1])?;

        let diffuse = if let Some(diffuse) = material.base_color_texture() {
            diffuse.file_name.as_str()
        } else {
            ""
        };

        semodel.write_null_terminated_string(diffuse)?;

        let normal = if let Some(normal) = material
            .textures
            .iter()
            .find(|&x| x.texture_usage == MaterialTextureRefUsage::Normal)
        {
            normal.file_name.as_str()
        } else {
            ""
        };

        semodel.write_null_terminated_string(normal)?;

        let specular = if let Some(specular) = material
            .textures
            .iter()
            .find(|&x| x.texture_usage == MaterialTextureRefUsage::Specular)
        {
            specular.file_name.as_str()
        } else {
            ""
        };

        semodel.write_null_terminated_string(specular)?;
    }

    Ok(())
}
