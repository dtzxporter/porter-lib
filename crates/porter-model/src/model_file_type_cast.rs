use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use porter_cast::CastFile;
use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;
use porter_cast::CastPropertyValue;

use porter_math::Axis;
use porter_math::Vector4;

use porter_utils::BufferWriteExt;

use crate::ConstraintOffset;
use crate::ConstraintType;
use crate::MaterialParameterType;
use crate::MaterialParameterValue;
use crate::MaterialTextureRefUsage;
use crate::Model;
use crate::ModelError;
use crate::SkinningMethod;

/// Writes a model in cast format to the given path.
pub fn to_cast<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut root = CastNode::root();

    let meta_node = root.create(CastId::Metadata);

    meta_node
        .create_property(CastPropertyId::String, "a")
        .push("DTZxPorter");

    meta_node
        .create_property(CastPropertyId::String, "s")
        .push("Exported by PorterLib");

    let up_axis = match model.up_axis {
        Axis::X => "x",
        Axis::Y => "y",
        Axis::Z => "z",
    };

    meta_node
        .create_property(CastPropertyId::String, "up")
        .push(up_axis);

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

            bone_node
                .create_property(CastPropertyId::Byte, "ssc")
                .push(bone.segment_scale_compensate);

            bone_node
                .create_property(CastPropertyId::Vector3, "lp")
                .push(bone.local_position);
            bone_node
                .create_property(CastPropertyId::Vector4, "lr")
                .push(bone.local_rotation);

            bone_node
                .create_property(CastPropertyId::Vector3, "wp")
                .push(bone.world_position);
            bone_node
                .create_property(CastPropertyId::Vector4, "wr")
                .push(bone.world_rotation);

            bone_node
                .create_property(CastPropertyId::Vector3, "s")
                .push(bone.local_scale);

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

            match constraint.offset {
                ConstraintOffset::None => {
                    constraint_node
                        .create_property(CastPropertyId::Byte, "mo")
                        .push(false);
                }
                ConstraintOffset::Maintain => {
                    constraint_node
                        .create_property(CastPropertyId::Byte, "mo")
                        .push(true);
                }
                ConstraintOffset::Vector3(vector3) => {
                    constraint_node
                        .create_property(CastPropertyId::Vector3, "co")
                        .push(vector3);
                }
                ConstraintOffset::Quaternion(quaternion) => {
                    constraint_node
                        .create_property(CastPropertyId::Vector4, "co")
                        .push(quaternion);
                }
            }

            constraint_node
                .create_property(CastPropertyId::Float, "wt")
                .push(constraint.weight);

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

        let mut used_slots: HashSet<String> = HashSet::new();
        let mut extras = 0;

        let mut extra_index = || {
            let index = extras;
            extras += 1;
            index
        };

        let mut usage_to_slot = |usage: MaterialTextureRefUsage| {
            let slot = match usage {
                MaterialTextureRefUsage::Albedo => String::from("albedo"),
                MaterialTextureRefUsage::Diffuse => String::from("diffuse"),
                MaterialTextureRefUsage::Specular => String::from("specular"),
                MaterialTextureRefUsage::Normal => String::from("normal"),
                MaterialTextureRefUsage::Emissive => String::from("emissive"),
                MaterialTextureRefUsage::EmissiveMask => String::from("emask"),
                MaterialTextureRefUsage::EmissiveStrength => String::from("estrength"),
                MaterialTextureRefUsage::Gloss => String::from("gloss"),
                MaterialTextureRefUsage::Roughness => String::from("roughness"),
                MaterialTextureRefUsage::AmbientOcclusion => String::from("ao"),
                MaterialTextureRefUsage::Cavity => String::from("cavity"),
                MaterialTextureRefUsage::Metalness => String::from("metal"),
                MaterialTextureRefUsage::Anisotropy => String::from("aniso"),
                MaterialTextureRefUsage::Unknown | MaterialTextureRefUsage::Count => {
                    format!("extra{}", extra_index())
                }
            };

            if used_slots.contains(&slot) {
                format!("extra{}", extra_index())
            } else {
                used_slots.insert(slot.clone());
                slot
            }
        };

        for texture in &material.textures {
            let file = material_node.create(CastId::File);

            file.create_property(CastPropertyId::String, "p")
                .push(texture.file_name.as_str());

            let slot = usage_to_slot(texture.texture_usage);

            let hash = CastPropertyValue::from(file);

            material_node
                .create_property(CastPropertyId::Integer64, slot)
                .push(hash);
        }

        for parameter in &material.parameters {
            #[allow(clippy::needless_late_init)]
            let hash: CastPropertyValue;
            #[allow(clippy::needless_late_init)]
            let slot: String;

            match parameter.param {
                MaterialParameterType::Usage(usage) => {
                    slot = usage_to_slot(usage);
                }
            }

            match parameter.value {
                MaterialParameterValue::ColorLinear { r, g, b, a } => {
                    let color = material_node.create(CastId::Color);

                    color
                        .create_property(CastPropertyId::String, "cs")
                        .push("linear");

                    color
                        .create_property(CastPropertyId::Vector4, "rgba")
                        .push(Vector4::new(r, g, b, a));

                    hash = CastPropertyValue::from(color);
                }
                MaterialParameterValue::ColorSRGB { r, g, b, a } => {
                    let color = material_node.create(CastId::Color);

                    color
                        .create_property(CastPropertyId::String, "cs")
                        .push("srgb");

                    color
                        .create_property(CastPropertyId::Vector4, "rgba")
                        .push(Vector4::new(r, g, b, a));

                    hash = CastPropertyValue::from(color);
                }
                _ => continue,
            }

            material_node
                .create_property(CastPropertyId::Integer64, slot)
                .push(hash);
        }

        material_map.insert(material_index, CastPropertyValue::from(material_node));
    }

    for mesh in &model.meshes {
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
        mesh_node
            .create_property(CastPropertyId::Byte, "cl")
            .push(mesh.vertices.colors() as u8);

        let sm = match mesh.skinning_method {
            SkinningMethod::Linear => "linear",
            SkinningMethod::DualQuaternion => "quaternion",
        };

        mesh_node
            .create_property(CastPropertyId::String, "sm")
            .push(sm);

        let vertex_count = mesh.vertices.len();
        let vertex_positions = mesh_node.create_property(CastPropertyId::Vector3, "vp");

        vertex_positions.try_reserve_exact(vertex_count)?;

        for i in 0..vertex_count {
            vertex_positions.push(mesh.vertices.vertex(i).position());
        }

        let vertex_normals = mesh_node.create_property(CastPropertyId::Vector3, "vn");

        vertex_normals.try_reserve_exact(vertex_count)?;

        for i in 0..vertex_count {
            vertex_normals.push(mesh.vertices.vertex(i).normal());
        }

        for cl in 0..mesh.vertices.colors() {
            let color_layer =
                mesh_node.create_property(CastPropertyId::Integer32, format!("c{}", cl));

            color_layer.try_reserve_exact(vertex_count)?;

            for i in 0..vertex_count {
                color_layer.push(u32::from(mesh.vertices.vertex(i).color(cl)));
            }
        }

        for uv in 0..mesh.vertices.uv_layers() {
            let uv_layer = mesh_node.create_property(CastPropertyId::Vector2, format!("u{}", uv));

            uv_layer.try_reserve_exact(vertex_count)?;

            for i in 0..vertex_count {
                uv_layer.push(mesh.vertices.vertex(i).uv(uv));
            }
        }

        if !model.skeleton.bones.is_empty() {
            let bone_count = model.skeleton.bones.len();
            let maximum_influence = mesh.vertices.maximum_influence();

            let vertex_weight_bones = if bone_count <= 0xFF {
                mesh_node.create_property(CastPropertyId::Byte, "wb")
            } else if bone_count <= 0xFFFF {
                mesh_node.create_property(CastPropertyId::Short, "wb")
            } else {
                mesh_node.create_property(CastPropertyId::Integer32, "wb")
            };

            vertex_weight_bones.try_reserve_exact(vertex_count * maximum_influence)?;

            for i in 0..vertex_count {
                let vertex = mesh.vertices.vertex(i);

                for w in 0..maximum_influence {
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

            vertex_weight_values.try_reserve_exact(vertex_count * maximum_influence)?;

            for i in 0..mesh.vertices.len() {
                let vertex = mesh.vertices.vertex(i);

                for w in 0..mesh.vertices.maximum_influence() {
                    vertex_weight_values.push(vertex.weight(w).value);
                }
            }
        }

        let faces = if vertex_count <= 0xFF {
            mesh_node.create_property(CastPropertyId::Byte, "f")
        } else if vertex_count <= 0xFFFF {
            mesh_node.create_property(CastPropertyId::Short, "f")
        } else {
            mesh_node.create_property(CastPropertyId::Integer32, "f")
        };

        faces.try_reserve_exact(mesh.faces.len() * 3)?;

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

        if let Some(material) = mesh
            .material
            .and_then(|material_index| material_map.get(&material_index))
        {
            mesh_node
                .create_property(CastPropertyId::Integer64, "m")
                .push(material.clone());
        }

        let mesh_hash = CastPropertyValue::from(mesh_node);

        for blend_shape in &*mesh.blend_shapes {
            let blend_shape_node = model_node.create(CastId::BlendShape);
            let blend_shape_mesh = &mesh;

            blend_shape_node
                .create_property(CastPropertyId::String, "n")
                .push(blend_shape.name.as_str());

            blend_shape_node
                .create_property(CastPropertyId::Integer64, "b")
                .push(mesh_hash.clone());

            blend_shape_node
                .create_property(CastPropertyId::Float, "ts")
                .push(blend_shape.target_scale);

            let indices_size = blend_shape
                .vertex_deltas
                .keys()
                .copied()
                .max()
                .unwrap_or_default();

            let indices = if indices_size <= 0xFF {
                blend_shape_node.create_property(CastPropertyId::Byte, "vi")
            } else if indices_size <= 0xFFFF {
                blend_shape_node.create_property(CastPropertyId::Short, "vi")
            } else {
                blend_shape_node.create_property(CastPropertyId::Integer32, "vi")
            };

            indices.try_reserve_exact(blend_shape.vertex_deltas.len())?;

            for index in blend_shape.vertex_deltas.keys() {
                if indices_size <= 0xFF {
                    indices.push(*index as u8);
                } else if indices_size <= 0xFFFF {
                    indices.push(*index as u16);
                } else {
                    indices.push(*index);
                }
            }

            let positions = blend_shape_node.create_property(CastPropertyId::Vector3, "vp");

            positions.try_reserve_exact(blend_shape.vertex_deltas.len())?;

            for (vertex_index, vertex_position_delta) in &blend_shape.vertex_deltas {
                let vertex_position = blend_shape_mesh
                    .vertices
                    .vertex(*vertex_index as usize)
                    .position();

                positions.push(vertex_position + *vertex_position_delta);
            }
        }
    }

    for hair in &model.hairs {
        let hair_node = model_node.create(CastId::Hair);

        if let Some(name) = &hair.name {
            hair_node
                .create_property(CastPropertyId::String, "n")
                .push(name.as_str());
        }

        // Output each segment for every strand.
        let largest_segment: u32 = hair
            .segments
            // Get each strand, find the maximum.
            .iter()
            .max()
            .copied()
            .unwrap_or_default();

        let segments = if largest_segment <= 0xFF {
            hair_node.create_property(CastPropertyId::Byte, "se")
        } else if largest_segment <= 0xFFFF {
            hair_node.create_property(CastPropertyId::Short, "se")
        } else {
            hair_node.create_property(CastPropertyId::Integer32, "se")
        };

        segments.try_reserve_exact(hair.segments.len())?;

        for segment in &hair.segments {
            if largest_segment <= 0xFF {
                segments.push(*segment as u8);
            } else if largest_segment <= 0xFFFF {
                segments.push(*segment as u16);
            } else {
                segments.push(*segment);
            }
        }

        let particles = hair_node.create_property(CastPropertyId::Vector3, "pt");

        particles.try_reserve_exact(hair.particles.len())?;

        for particle in &hair.particles {
            particles.push(*particle);
        }

        if let Some(material) = hair
            .material
            .and_then(|material_index| material_map.get(&material_index))
        {
            hair_node
                .create_property(CastPropertyId::Integer64, "m")
                .push(material.clone());
        }
    }

    let writer = File::create(path.as_ref().with_extension("cast"))?.buffer_write();

    let mut file = CastFile::new();

    file.push(root);
    file.write(writer)?;

    Ok(())
}
