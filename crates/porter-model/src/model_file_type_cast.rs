use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use porter_cast::CastFile;
use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;
use porter_cast::CastPropertyValue;

use crate::ConstraintType;
use crate::MaterialTextureRefUsage;
use crate::Model;
use crate::ModelError;

/// Writes a model in cast format to the given path.
pub fn to_cast<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut root = CastNode::root();

    let model_node = root.create(CastId::Model);

    if !model.skeleton.bones.is_empty() {
        let skeleton_node = model_node.create(CastId::Skeleton);

        let mut bone_map: HashMap<usize, CastPropertyValue> =
            HashMap::with_capacity(model.skeleton.bones.len());

        for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
            let bone_node = skeleton_node.create(CastId::Bone);

            bone_node.create_property(CastPropertyId::String, "n").push(
                bone.name
                    .as_deref()
                    .unwrap_or(&format!("porter_bone_{}", bone_index)),
            );

            bone_node
                .create_property(CastPropertyId::Integer32, "p")
                .push(bone.parent as u32);

            if let (Some(local_position), Some(local_rotation)) =
                (bone.local_position, bone.local_rotation)
            {
                bone_node
                    .create_property(CastPropertyId::Vector3, "lp")
                    .push(local_position);
                bone_node
                    .create_property(CastPropertyId::Vector4, "lr")
                    .push(local_rotation);
            }

            if let (Some(world_position), Some(world_rotation)) =
                (bone.world_position, bone.world_rotation)
            {
                bone_node
                    .create_property(CastPropertyId::Vector3, "wp")
                    .push(world_position);
                bone_node
                    .create_property(CastPropertyId::Vector4, "wr")
                    .push(world_rotation);
            }

            if let Some(local_scale) = bone.local_scale {
                bone_node
                    .create_property(CastPropertyId::Vector3, "s")
                    .push(local_scale);
            }

            bone_map.insert(bone_index, CastPropertyValue::from(bone_node));
        }

        for ik_handle in &*model.skeleton.ik_handles {
            let handle_node = skeleton_node.create(CastId::IKHandle);

            if let Some(name) = &ik_handle.name {
                handle_node
                    .create_property(CastPropertyId::String, "n")
                    .push(name.as_str());
            }

            handle_node
                .create_property(CastPropertyId::Integer64, "sb")
                .push(bone_map[&ik_handle.start_bone].clone());

            handle_node
                .create_property(CastPropertyId::Integer64, "eb")
                .push(bone_map[&ik_handle.end_bone].clone());

            if let Some(target_bone) = &ik_handle.target_bone {
                handle_node
                    .create_property(CastPropertyId::Integer64, "tb")
                    .push(bone_map[target_bone].clone());
            }

            if let Some(pole_vector_bone) = &ik_handle.pole_vector_bone {
                handle_node
                    .create_property(CastPropertyId::Integer64, "pv")
                    .push(bone_map[pole_vector_bone].clone());
            }

            if let Some(pole_bone) = &ik_handle.pole_bone {
                handle_node
                    .create_property(CastPropertyId::Integer64, "pb")
                    .push(bone_map[pole_bone].clone());
            }

            handle_node
                .create_property(CastPropertyId::Byte, "tr")
                .push(ik_handle.use_target_rotation as u8);
        }

        for constraint in &*model.skeleton.constraints {
            let constraint_node = skeleton_node.create(CastId::Constraint);

            if let Some(name) = &constraint.name {
                constraint_node
                    .create_property(CastPropertyId::String, "n")
                    .push(name.as_str());
            }

            let ct = match constraint.constraint_type {
                ConstraintType::Point => "pt",
                ConstraintType::Orient => "or",
                ConstraintType::Scale => "sc",
            };

            constraint_node
                .create_property(CastPropertyId::String, "ct")
                .push(ct);

            constraint_node
                .create_property(CastPropertyId::Integer64, "cb")
                .push(bone_map[&constraint.constraint_bone].clone());

            constraint_node
                .create_property(CastPropertyId::Integer64, "tb")
                .push(bone_map[&constraint.target_bone].clone());

            constraint_node
                .create_property(CastPropertyId::Byte, "mo")
                .push(constraint.maintain_offset as u8);

            constraint_node
                .create_property(CastPropertyId::Byte, "sx")
                .push(constraint.skip_x as u8);

            constraint_node
                .create_property(CastPropertyId::Byte, "sy")
                .push(constraint.skip_y as u8);

            constraint_node
                .create_property(CastPropertyId::Byte, "sz")
                .push(constraint.skip_z as u8);
        }
    }

    let mut material_map: HashMap<usize, CastPropertyValue> =
        HashMap::with_capacity(model.materials.len());

    for (material_index, material) in model.materials.iter().enumerate() {
        let material_node = model_node.create(CastId::Material);

        material_node
            .create_property(CastPropertyId::String, "n")
            .push(material.name.as_str());

        material_node
            .create_property(CastPropertyId::String, "t")
            .push("pbr");

        for i in 0..material.len() {
            let texture = &material.textures[i];

            let file = material_node.create(CastId::File);

            file.create_property(CastPropertyId::String, "p")
                .push(texture.file_name.as_str());

            let slot = match texture.texture_usage {
                MaterialTextureRefUsage::Albedo => String::from("albedo"),
                MaterialTextureRefUsage::Diffuse => String::from("diffuse"),
                MaterialTextureRefUsage::Specular => String::from("specular"),
                MaterialTextureRefUsage::Normal => String::from("normal"),
                MaterialTextureRefUsage::Emissive => String::from("emissive"),
                MaterialTextureRefUsage::Gloss => String::from("gloss"),
                MaterialTextureRefUsage::Roughness => String::from("roughness"),
                MaterialTextureRefUsage::AmbientOcclusion => String::from("ao"),
                MaterialTextureRefUsage::Cavity => String::from("cavity"),
                MaterialTextureRefUsage::Unknown | MaterialTextureRefUsage::Anisotropy => {
                    format!("extra{}", i)
                }
            };

            let hash = CastPropertyValue::from(file);

            material_node
                .create_property(CastPropertyId::Integer64, slot)
                .push(hash);
        }

        material_map.insert(material_index, CastPropertyValue::from(material_node));
    }

    let mut mesh_map: HashMap<usize, CastPropertyValue> =
        HashMap::with_capacity(model.meshes.len());

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        let mesh_node = model_node.create(CastId::Mesh);

        if let Some(name) = &mesh.name {
            mesh_node
                .create_property(CastPropertyId::String, "n")
                .push(name.as_str());
        }

        mesh_node
            .create_property(CastPropertyId::Byte, "ul")
            .push(mesh.vertices.uv_layers() as u8);
        mesh_node
            .create_property(CastPropertyId::Byte, "mi")
            .push(mesh.vertices.maximum_influence() as u8);

        let vertex_positions = mesh_node.create_property(CastPropertyId::Vector3, "vp");

        for i in 0..mesh.vertices.len() {
            vertex_positions.push(mesh.vertices.vertex(i).position());
        }

        let vertex_normals = mesh_node.create_property(CastPropertyId::Vector3, "vn");

        for i in 0..mesh.vertices.len() {
            vertex_normals.push(mesh.vertices.vertex(i).normal());
        }

        if mesh.vertices.colors() {
            let vertex_colors = mesh_node.create_property(CastPropertyId::Integer32, "vc");

            for i in 0..mesh.vertices.len() {
                vertex_colors.push(u32::from(mesh.vertices.vertex(i).color()));
            }
        }

        for uv in 0..mesh.vertices.uv_layers() {
            let uv_layer = mesh_node.create_property(CastPropertyId::Vector2, format!("u{}", uv));

            for i in 0..mesh.vertices.len() {
                uv_layer.push(mesh.vertices.vertex(i).uv(uv));
            }
        }

        if !model.skeleton.bones.is_empty() {
            let bone_count = model.skeleton.bones.len();

            let vertex_weight_bones = if bone_count <= 0xFF {
                mesh_node.create_property(CastPropertyId::Byte, "wb")
            } else if bone_count <= 0xFFFF {
                mesh_node.create_property(CastPropertyId::Short, "wb")
            } else {
                mesh_node.create_property(CastPropertyId::Integer32, "wb")
            };

            for i in 0..mesh.vertices.len() {
                let vertex = mesh.vertices.vertex(i);

                for w in 0..mesh.vertices.maximum_influence() {
                    let weight = vertex.weight(w);

                    if bone_count <= 0xFF {
                        vertex_weight_bones.push(weight.bone as u8);
                    } else if bone_count <= 0xFFFF {
                        vertex_weight_bones.push(weight.bone);
                    } else {
                        vertex_weight_bones.push(weight.bone as u32);
                    }
                }
            }

            let vertex_weight_values = mesh_node.create_property(CastPropertyId::Float, "wv");

            for i in 0..mesh.vertices.len() {
                let vertex = mesh.vertices.vertex(i);

                for w in 0..mesh.vertices.maximum_influence() {
                    vertex_weight_values.push(vertex.weight(w).value);
                }
            }
        }

        let vertex_count = mesh.vertices.len();

        let faces = if vertex_count <= 0xFF {
            mesh_node.create_property(CastPropertyId::Byte, "f")
        } else if vertex_count <= 0xFFFF {
            mesh_node.create_property(CastPropertyId::Short, "f")
        } else {
            mesh_node.create_property(CastPropertyId::Integer32, "f")
        };

        for face in &*mesh.faces {
            if vertex_count <= 0xFF {
                faces.push(face.i3 as u8);
                faces.push(face.i2 as u8);
                faces.push(face.i1 as u8);
            } else if vertex_count <= 0xFFFF {
                faces.push(face.i3 as u16);
                faces.push(face.i2 as u16);
                faces.push(face.i1 as u16);
            } else {
                faces.push(face.i3);
                faces.push(face.i2);
                faces.push(face.i1);
            }
        }

        if !mesh.materials.is_empty() && mesh.materials[0] > -1 {
            if let Some(material) = material_map.get(&(mesh.materials[0] as usize)) {
                mesh_node
                    .create_property(CastPropertyId::Integer64, "m")
                    .push(material.clone());
            }
        }

        mesh_map.insert(mesh_index, CastPropertyValue::from(mesh_node));
    }

    for blend_shape in &*model.blend_shapes {
        let blend_shape_node = model_node.create(CastId::BlendShape);

        if let Some(name) = &blend_shape.name {
            blend_shape_node
                .create_property(CastPropertyId::String, "n")
                .push(name.as_str());
        }

        blend_shape_node
            .create_property(CastPropertyId::Integer64, "b")
            .push(mesh_map[&blend_shape.base_mesh].clone());

        let targets = blend_shape_node.create_property(CastPropertyId::Integer64, "t");

        for target in &blend_shape.target_meshes {
            targets.push(mesh_map[target].clone());
        }

        if !blend_shape.target_scales.is_empty() {
            let scales = blend_shape_node.create_property(CastPropertyId::Float, "ts");

            for scale in &blend_shape.target_scales {
                scales.push(*scale);
            }
        }
    }

    let writer = BufWriter::new(File::create(path.as_ref().with_extension("cast"))?);

    let mut file = CastFile::new();

    file.push(root);
    file.write(writer)?;

    Ok(())
}
