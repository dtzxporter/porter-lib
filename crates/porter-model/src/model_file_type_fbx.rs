use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

use porter_fbx::FbxDocument;
use porter_fbx::FbxNode;
use porter_fbx::FbxPropertyType;
use porter_fbx::FbxPropertyValue;

use porter_math::Angles;
use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::MaterialTextureRef;
use crate::MaterialTextureRefUsage;
use crate::Model;
use crate::ModelError;

/// Adds an object connection from->to.
fn add_object_connection<F: Into<FbxPropertyValue>, T: Into<FbxPropertyValue>>(
    connection_node: &mut FbxNode,
    from: F,
    to: T,
) {
    let connection = connection_node.create("C");

    connection
        .create_property(FbxPropertyType::String)
        .push_string("OO");
    connection
        .create_property(FbxPropertyType::Integer64)
        .push(from);
    connection
        .create_property(FbxPropertyType::Integer64)
        .push(to);
}

/// Adds an object property connection from->to[property].
fn add_object_property_connection<
    F: Into<FbxPropertyValue>,
    T: Into<FbxPropertyValue>,
    P: Into<String>,
>(
    connection_node: &mut FbxNode,
    from: F,
    to: T,
    property: P,
) {
    let connection = connection_node.create("C");

    connection
        .create_property(FbxPropertyType::String)
        .push_string("OP");
    connection
        .create_property(FbxPropertyType::Integer64)
        .push(from);
    connection
        .create_property(FbxPropertyType::Integer64)
        .push(to);
    connection
        .create_property(FbxPropertyType::String)
        .push_string(property);
}

/// Creates and connects a texture node to a material.
fn initialize_texture_node(
    root: &mut FbxDocument,
    texture: &MaterialTextureRef,
    material_hash: FbxPropertyValue,
    connection: &str,
) {
    let texture_node = root.objects_node().create("Texture");
    let texture_name = PathBuf::from(texture.file_name.as_str())
        .file_stem()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("not_found"));

    texture_node.create_hash();
    texture_node
        .create_property(FbxPropertyType::String)
        .push_string(format!("{}\u{0000}\u{0001}Texture", texture_name));
    texture_node
        .create_property(FbxPropertyType::String)
        .push_string("");

    texture_node
        .create("Type")
        .create_property(FbxPropertyType::String)
        .push_string("TextureVideoClip");
    texture_node
        .create("Version")
        .create_property(FbxPropertyType::Integer32)
        .push(202u32);
    texture_node
        .create("TextureName")
        .create_property(FbxPropertyType::String)
        .push_string(format!("{}\u{0000}\u{0001}Texture", texture_name));

    let properties = texture_node.create("Properties70");

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("CurrentTextureBlendMode");
        props
            .create_property(FbxPropertyType::String)
            .push_string("enum");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props.create_property(FbxPropertyType::Integer32).push(0u32);
    }

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("UVSet");
        props
            .create_property(FbxPropertyType::String)
            .push_string("KString");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::String)
            .push_string("map1");
    }

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("UseMaterial");
        props
            .create_property(FbxPropertyType::String)
            .push_string("bool");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props.create_property(FbxPropertyType::Integer32).push(1u32);
    }

    texture_node
        .create("Media")
        .create_property(FbxPropertyType::String)
        .push_string(format!("{}\u{0000}\u{0001}Video", texture_name));

    texture_node
        .create("FileName")
        .create_property(FbxPropertyType::String)
        .push_string(texture.file_name.replace('\\', "/"));
    texture_node
        .create("RelativeFilename")
        .create_property(FbxPropertyType::String)
        .push_string(texture.file_name.as_str());

    let texture_hash = FbxPropertyValue::from(texture_node);

    add_object_property_connection(
        root.connections_node(),
        texture_hash,
        material_hash,
        connection,
    );
}

/// Adds basic properties to the model and skeleton root nodes.
fn initialize_root_node(root_node: &mut FbxNode) {
    root_node
        .create("Version")
        .create_property(FbxPropertyType::Integer32)
        .push(232u32);

    let properties = root_node.create("Properties70");

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("PreRotation");
        props
            .create_property(FbxPropertyType::String)
            .push_string("Vector3D");
        props
            .create_property(FbxPropertyType::String)
            .push_string("Vector");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::Float64)
            .push(-90.0f64);
        props.create_property(FbxPropertyType::Float64).push(0.0f64);
        props.create_property(FbxPropertyType::Float64).push(0.0f64);
    }

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("RotationActive");
        props
            .create_property(FbxPropertyType::String)
            .push_string("bool");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props.create_property(FbxPropertyType::Integer32).push(1u32);
    }

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("InheritType");
        props
            .create_property(FbxPropertyType::String)
            .push_string("enum");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props.create_property(FbxPropertyType::Integer32).push(1u32);
    }

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("ScalingMax");
        props
            .create_property(FbxPropertyType::String)
            .push_string("Vector3D");
        props
            .create_property(FbxPropertyType::String)
            .push_string("Vector");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props.create_property(FbxPropertyType::Float64).push(0.0);
        props.create_property(FbxPropertyType::Float64).push(0.0);
        props.create_property(FbxPropertyType::Float64).push(0.0);
    }

    {
        let props = properties.create("P");

        props
            .create_property(FbxPropertyType::String)
            .push_string("DefaultAttributeIndex");
        props
            .create_property(FbxPropertyType::String)
            .push_string("int");
        props
            .create_property(FbxPropertyType::String)
            .push_string("Integer");
        props
            .create_property(FbxPropertyType::String)
            .push_string("");
        props.create_property(FbxPropertyType::Integer32).push(0u32);
    }
}

/// Writes a model in fbx format to the given path.
pub fn to_fbx<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let mut root = FbxDocument::new();
    let root_hash = FbxPropertyValue::from(root.root_node());

    let mut joints_map: HashMap<usize, FbxPropertyValue> =
        HashMap::with_capacity(model.skeleton.bones.len());

    if !model.skeleton.bones.is_empty() {
        let joints = root.objects_node().create("Model");

        joints.create_hash();
        joints
            .create_property(FbxPropertyType::String)
            .push_string("Joints\u{0000}\u{0001}Model");
        joints
            .create_property(FbxPropertyType::String)
            .push_string("Null");

        initialize_root_node(joints);

        let joints_hash = FbxPropertyValue::from(joints);

        add_object_connection(root.connections_node(), joints_hash, root_hash);

        for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
            let skeleton = root.objects_node().create("NodeAttribute");

            skeleton.create_hash();
            skeleton
                .create_property(FbxPropertyType::String)
                .push_string("\u{0000}\u{0001}NodeAttribute");
            skeleton
                .create_property(FbxPropertyType::String)
                .push_string("LimbNode");

            let properties = skeleton.create("Properties70").create("P");

            properties
                .create_property(FbxPropertyType::String)
                .push_string("Size");
            properties
                .create_property(FbxPropertyType::String)
                .push_string("double");
            properties
                .create_property(FbxPropertyType::String)
                .push_string("Number");
            properties
                .create_property(FbxPropertyType::String)
                .push_string("");
            properties
                .create_property(FbxPropertyType::Float64)
                .push(1000.0 / 30.0);

            skeleton
                .create("TypeFlags")
                .create_property(FbxPropertyType::String)
                .push_string("Skeleton");

            let skeleton_hash = FbxPropertyValue::from(skeleton);

            let joint = root.objects_node().create("Model");

            joint.create_hash();
            joint
                .create_property(FbxPropertyType::String)
                .push_string(format!(
                    "{}\u{0000}\u{0001}Model",
                    bone.name
                        .as_deref()
                        .unwrap_or(&format!("porter_bone_{}", bone_index))
                ));
            joint
                .create_property(FbxPropertyType::String)
                .push_string("LimbNode");

            joint
                .create("Version")
                .create_property(FbxPropertyType::Integer32)
                .push(232u32);

            let properties = joint.create("Properties70");

            {
                let props = properties.create("P");

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("RotationActive");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("bool");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props.create_property(FbxPropertyType::Integer32).push(1u32);
            }

            {
                let props = properties.create("P");

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("InheritType");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("enum");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props.create_property(FbxPropertyType::Integer32).push(1u32);
            }

            {
                let props = properties.create("P");

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("ScalingMax");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Vector3D");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Vector");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props.create_property(FbxPropertyType::Float64).push(0.0f64);
                props.create_property(FbxPropertyType::Float64).push(0.0f64);
                props.create_property(FbxPropertyType::Float64).push(0.0f64);
            }

            {
                let props = properties.create("P");

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("DefaultAttributeIndex");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("int");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Integer");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props.create_property(FbxPropertyType::Integer32).push(0u32);
            }

            {
                let props = properties.create("P");
                let position = bone.local_position.unwrap_or_default();

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Lcl Translation");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Lcl Translation");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("A");
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(position.x as f64);
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(position.y as f64);
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(position.z as f64);
            }

            {
                let props = properties.create("P");
                let rotation = bone
                    .local_rotation
                    .unwrap_or_default()
                    .euler_angles(Angles::Degrees);

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Lcl Rotation");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Lcl Rotation");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("A");
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(rotation.x as f64);
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(rotation.y as f64);
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(rotation.z as f64);
            }

            {
                let props = properties.create("P");
                let scale = bone.local_scale.unwrap_or(Vector3::one());

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Lcl Scaling");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Lcl Scaling");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("A");
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(scale.x as f64);
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(scale.y as f64);
                props
                    .create_property(FbxPropertyType::Float64)
                    .push(scale.z as f64);
            }

            {
                let props = properties.create("P");

                props
                    .create_property(FbxPropertyType::String)
                    .push_string("lockInfluenceWeights");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("Bool");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("");
                props
                    .create_property(FbxPropertyType::String)
                    .push_string("A+U");
                props.create_property(FbxPropertyType::Integer32).push(0u32);
            }

            let joint_hash = FbxPropertyValue::from(joint);

            add_object_connection(root.connections_node(), skeleton_hash, joint_hash);

            if bone.parent >= 0 {
                add_object_connection(
                    root.connections_node(),
                    joint_hash,
                    joints_map[&(bone.parent as usize)],
                );
            } else {
                add_object_connection(root.connections_node(), joint_hash, joints_hash);
            }

            joints_map.insert(bone_index, joint_hash);
        }
    }

    let mut material_map: HashMap<usize, FbxPropertyValue> = HashMap::new();

    for (material_index, material) in model.materials.iter().enumerate() {
        let material_node = root.objects_node().create("Material");

        material_node.create_hash();
        material_node
            .create_property(FbxPropertyType::String)
            .push_string(format!("{}\u{0000}\u{0001}Material", material.name));
        material_node
            .create_property(FbxPropertyType::String)
            .push_string("");

        material_node
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(102u32);
        material_node
            .create("ShadingModel")
            .create_property(FbxPropertyType::String)
            .push_string("Lambert");
        material_node
            .create("MultiLayer")
            .create_property(FbxPropertyType::Integer32)
            .push(0u32);

        let material_hash = FbxPropertyValue::from(material_node);

        material_map.insert(material_index, material_hash);

        if let Some(diffuse) = material.base_color_texture() {
            initialize_texture_node(&mut root, diffuse, material_hash, "DiffuseColor");
        }

        if let Some(normal) = material
            .textures
            .iter()
            .find(|x| x.texture_usage == MaterialTextureRefUsage::Normal)
        {
            initialize_texture_node(&mut root, normal, material_hash, "NormalMap");
        }
    }

    let model_node = root.objects_node().create("Model");

    model_node.create_hash();
    model_node
        .create_property(FbxPropertyType::String)
        .push_string(format!(
            "{}\u{0000}\u{0001}Model",
            path.as_ref()
                .file_stem()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_else(|| String::from("PorterModel"))
        ));
    model_node
        .create_property(FbxPropertyType::String)
        .push_string("Null");

    initialize_root_node(model_node);

    let model_hash = FbxPropertyValue::from(model_node);

    add_object_connection(root.connections_node(), model_hash, root_hash);

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        let mesh_node = root.objects_node().create("Model");

        mesh_node.create_hash();
        mesh_node
            .create_property(FbxPropertyType::String)
            .push_string(format!("PorterMesh{}\u{0000}\u{0001}Model", mesh_index));
        mesh_node
            .create_property(FbxPropertyType::String)
            .push_string("Mesh");

        mesh_node
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(232u32);

        let properties = mesh_node.create("Properties70");

        {
            let props = properties.create("P");

            props
                .create_property(FbxPropertyType::String)
                .push_string("Lcl Rotation");
            props
                .create_property(FbxPropertyType::String)
                .push_string("Lcl Rotation");
            props
                .create_property(FbxPropertyType::String)
                .push_string("");
            props
                .create_property(FbxPropertyType::String)
                .push_string("A");
            props.create_property(FbxPropertyType::Float64).push(0.0f64);
            props.create_property(FbxPropertyType::Float64).push(0.0f64);
            props.create_property(FbxPropertyType::Float64).push(0.0f64);
        }

        {
            let props = properties.create("P");

            props
                .create_property(FbxPropertyType::String)
                .push_string("DefaultAttributeIndex");
            props
                .create_property(FbxPropertyType::String)
                .push_string("int");
            props
                .create_property(FbxPropertyType::String)
                .push_string("Integer");
            props
                .create_property(FbxPropertyType::String)
                .push_string("");
            props.create_property(FbxPropertyType::Integer32).push(0u32);
        }

        {
            let props = properties.create("P");

            props
                .create_property(FbxPropertyType::String)
                .push_string("InheritType");
            props
                .create_property(FbxPropertyType::String)
                .push_string("enum");
            props
                .create_property(FbxPropertyType::String)
                .push_string("");
            props
                .create_property(FbxPropertyType::String)
                .push_string("");
            props.create_property(FbxPropertyType::Integer32).push(1u32);
        }

        mesh_node
            .create("MultiLayer")
            .create_property(FbxPropertyType::Integer32)
            .push(0u32);
        mesh_node
            .create("MultiTake")
            .create_property(FbxPropertyType::Integer32)
            .push(0u32);
        mesh_node
            .create("Shading")
            .create_property(FbxPropertyType::Bool)
            .push(true);
        mesh_node
            .create("Culling")
            .create_property(FbxPropertyType::String)
            .push_string("CullingOff");

        let mesh_hash = FbxPropertyValue::from(mesh_node);

        let geometry = root.objects_node().create("Geometry");

        geometry.create_hash();
        geometry
            .create_property(FbxPropertyType::String)
            .push_string(format!("PorterMesh{}\u{0000}\u{0001}Geometry", mesh_index));
        geometry
            .create_property(FbxPropertyType::String)
            .push_string("Mesh");

        geometry.create("Properties70");

        geometry
            .create("GeometryVersion")
            .create_property(FbxPropertyType::Integer32)
            .push(124u32);

        let vertex_buffer = geometry
            .create("Vertices")
            .create_property(FbxPropertyType::Float64Array);

        for i in 0..mesh.vertices.len() {
            let position = mesh.vertices.vertex(i).position();

            vertex_buffer.push(position.x as f64);
            vertex_buffer.push(position.y as f64);
            vertex_buffer.push(position.z as f64);
        }

        let face_buffer = geometry
            .create("PolygonVertexIndex")
            .create_property(FbxPropertyType::Integer32Array);

        for face in &mesh.faces {
            face_buffer.push(face.i3);
            face_buffer.push(face.i2);
            face_buffer.push(0xFFFFFFFF ^ (face.i1));
        }

        let layer_normals = geometry.create("LayerElementNormal");

        layer_normals
            .create_property(FbxPropertyType::Integer32)
            .push(0u32);

        layer_normals
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(101u32);
        layer_normals
            .create("Name")
            .create_property(FbxPropertyType::String)
            .push_string("");
        layer_normals
            .create("MappingInformationType")
            .create_property(FbxPropertyType::String)
            .push_string("ByVertice");
        layer_normals
            .create("ReferenceInformationType")
            .create_property(FbxPropertyType::String)
            .push_string("Direct");

        let normals_buffer = layer_normals
            .create("Normals")
            .create_property(FbxPropertyType::Float64Array);

        for i in 0..mesh.vertices.len() {
            let normal = mesh.vertices.vertex(i).normal();

            normals_buffer.push(normal.x as f64);
            normals_buffer.push(normal.y as f64);
            normals_buffer.push(normal.z as f64);
        }

        if mesh.vertices.colors() {
            let layer_color = geometry.create("LayerElementColor");

            layer_color
                .create_property(FbxPropertyType::Integer32)
                .push(0u32);
            layer_color
                .create("Name")
                .create_property(FbxPropertyType::String)
                .push_string("colorSet1");
            layer_color
                .create("Version")
                .create_property(FbxPropertyType::Integer32)
                .push(101u32);
            layer_color
                .create("MappingInformationType")
                .create_property(FbxPropertyType::String)
                .push_string("ByVertice");
            layer_color
                .create("ReferenceInformationType")
                .create_property(FbxPropertyType::String)
                .push_string("Direct");

            let color_buffer = layer_color
                .create("Colors")
                .create_property(FbxPropertyType::Float64Array);

            for v in 0..mesh.vertices.len() {
                let color = mesh.vertices.vertex(v).color();

                color_buffer.push(color.r as f64 / 255.0);
                color_buffer.push(color.g as f64 / 255.0);
                color_buffer.push(color.b as f64 / 255.0);
                color_buffer.push(color.a as f64 / 255.0);
            }
        }

        for i in 0..mesh.vertices.uv_layers() {
            let layer_uvs = geometry.create("LayerElementUV");

            layer_uvs
                .create_property(FbxPropertyType::Integer32)
                .push(i as u32);
            layer_uvs
                .create("Name")
                .create_property(FbxPropertyType::String)
                .push_string(format!("map{}", i + 1));
            layer_uvs
                .create("Version")
                .create_property(FbxPropertyType::Integer32)
                .push(101u32);
            layer_uvs
                .create("MappingInformationType")
                .create_property(FbxPropertyType::String)
                .push_string("ByVertice");
            layer_uvs
                .create("ReferenceInformationType")
                .create_property(FbxPropertyType::String)
                .push_string("Direct");

            let uvs_buffer = layer_uvs
                .create("UV")
                .create_property(FbxPropertyType::Float64Array);

            for v in 0..mesh.vertices.len() {
                let uv = mesh.vertices.vertex(v).uv(i);

                uvs_buffer.push(uv.x as f64);
                uvs_buffer.push(1.0 - uv.y as f64);
            }
        }

        if !mesh.materials.is_empty() && mesh.materials[0] >= 0 {
            let layer_material = geometry.create("LayerElementMaterial");

            layer_material
                .create_property(FbxPropertyType::Integer32)
                .push(0u32);

            layer_material
                .create("Version")
                .create_property(FbxPropertyType::Integer32)
                .push(101u32);
            layer_material
                .create("Name")
                .create_property(FbxPropertyType::String)
                .push_string("");
            layer_material
                .create("MappingInformationType")
                .create_property(FbxPropertyType::String)
                .push_string("AllSame");
            layer_material
                .create("ReferenceInformationType")
                .create_property(FbxPropertyType::String)
                .push_string("IndexToDirect");

            layer_material
                .create("Materials")
                .create_property(FbxPropertyType::Integer32Array)
                .push(0u32);
        }

        let layer_info = geometry.create("Layer");

        layer_info
            .create_property(FbxPropertyType::Integer32)
            .push(0u32);

        layer_info
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(100u32);

        {
            let layer_element = layer_info.create("LayerElement");

            layer_element
                .create("Type")
                .create_property(FbxPropertyType::String)
                .push_string("LayerElementNormal");
            layer_element
                .create("TypeIndex")
                .create_property(FbxPropertyType::Integer32)
                .push(0u32);
        }

        if mesh.vertices.colors() {
            let layer_element = layer_info.create("LayerElement");

            layer_element
                .create("Type")
                .create_property(FbxPropertyType::String)
                .push_string("LayerElementColor");
            layer_element
                .create("TypeIndex")
                .create_property(FbxPropertyType::Integer32)
                .push(0u32);
        }

        if mesh.vertices.uv_layers() > 0 {
            let layer_element = layer_info.create("LayerElement");

            layer_element
                .create("Type")
                .create_property(FbxPropertyType::String)
                .push_string("LayerElementUV");
            layer_element
                .create("TypeIndex")
                .create_property(FbxPropertyType::Integer32)
                .push(0u32);
        }

        if !mesh.materials.is_empty() && mesh.materials[0] >= 0 {
            let layer_element = layer_info.create("LayerElement");

            layer_element
                .create("Type")
                .create_property(FbxPropertyType::String)
                .push_string("LayerElementMaterial");
            layer_element
                .create("TypeIndex")
                .create_property(FbxPropertyType::Integer32)
                .push(0u32);
        }

        for i in 1..mesh.vertices.uv_layers() {
            let layer_info = geometry.create("Layer");

            layer_info
                .create_property(FbxPropertyType::Integer32)
                .push(i as u32);

            let layer_element = layer_info.create("LayerElement");

            layer_element
                .create("Type")
                .create_property(FbxPropertyType::String)
                .push_string("LayerElementUV");
            layer_element
                .create("TypeIndex")
                .create_property(FbxPropertyType::Integer32)
                .push(i as u32);
        }

        let geometry_hash = FbxPropertyValue::from(geometry);

        add_object_connection(root.connections_node(), mesh_hash, model_hash);
        add_object_connection(root.connections_node(), geometry_hash, mesh_hash);

        if !mesh.materials.is_empty() && mesh.materials[0] >= 0 {
            if let Some(material) = material_map.get(&(mesh.materials[0] as usize)) {
                add_object_connection(root.connections_node(), *material, mesh_hash);
            }
        }

        if mesh.vertices.maximum_influence() == 0 {
            continue;
        }

        let deformer = root.objects_node().create("Deformer");

        deformer.create_hash();
        deformer
            .create_property(FbxPropertyType::String)
            .push_string(format!("PorterMesh{}\u{0000}\u{0001}Deformer", mesh_index));
        deformer
            .create_property(FbxPropertyType::String)
            .push_string("Skin");

        deformer
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(101u32);
        deformer
            .create("Link_DeformAcuracy")
            .create_property(FbxPropertyType::Float64)
            .push(50.0f64);

        let deformer_hash = FbxPropertyValue::from(deformer);

        add_object_connection(root.connections_node(), deformer_hash, geometry_hash);

        let mut sub_deformers: HashMap<u16, BTreeMap<usize, f32>> = HashMap::new();

        for i in 0..mesh.vertices.len() {
            let vertex = mesh.vertices.vertex(i);

            for w in 0..mesh.vertices.maximum_influence() {
                let weight = vertex.weight(w);

                match sub_deformers.entry(weight.bone).or_default().entry(i) {
                    Entry::Occupied(mut e) => {
                        e.insert(e.get() + weight.value);
                    }
                    Entry::Vacant(e) => {
                        e.insert(weight.value);
                    }
                }
            }
        }

        let mut bind_pose_ids: HashSet<u16> = sub_deformers.keys().copied().collect();

        for bone_id in sub_deformers.keys() {
            let mut current_parent = model.skeleton.bones[*bone_id as usize].parent;

            while current_parent >= 0 {
                bind_pose_ids.insert(current_parent as u16);

                current_parent = model.skeleton.bones[current_parent as usize].parent;
            }
        }

        let bind_pose = root.objects_node().create("Pose");

        bind_pose.create_hash();
        bind_pose
            .create_property(FbxPropertyType::String)
            .push_string(format!("Pose\u{0000}\u{0001}skinCluster{}", mesh_index + 1));
        bind_pose
            .create_property(FbxPropertyType::String)
            .push_string("BindPose");

        bind_pose
            .create("Type")
            .create_property(FbxPropertyType::String)
            .push_string("BindPose");
        bind_pose
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(100u32);
        bind_pose
            .create("NbPoseNodes")
            .create_property(FbxPropertyType::Integer32)
            .push(bind_pose_ids.len() as u32 + 1);

        for bone_id in bind_pose_ids {
            let pose_node = bind_pose.create("PoseNode");

            pose_node
                .create("Node")
                .create_property(FbxPropertyType::Integer64)
                .push(joints_map[&(bone_id as usize)]);

            let matrix = pose_node
                .create("Matrix")
                .create_property(FbxPropertyType::Float64Array);

            let global_matrix = model.skeleton.bones[bone_id as usize].world_matrix();

            for i in 0..16 {
                matrix.push(global_matrix[i] as f64);
            }
        }

        {
            let pose_node = bind_pose.create("PoseNode");

            pose_node
                .create("Node")
                .create_property(FbxPropertyType::Integer64)
                .push(mesh_hash);

            let matrix = pose_node
                .create("Matrix")
                .create_property(FbxPropertyType::Float64Array);

            let global_matrix = Matrix4x4::new();

            for i in 0..16 {
                matrix.push(global_matrix[i] as f64);
            }
        }

        for (bone_id, weights) in sub_deformers {
            let sub_deformer = root.objects_node().create("Deformer");

            sub_deformer.create_hash();
            sub_deformer
                .create_property(FbxPropertyType::String)
                .push_string(format!(
                    "PorterMesh{}_Bone{}\u{0000}\u{0001}SubDeformer",
                    mesh_index, bone_id
                ));
            sub_deformer
                .create_property(FbxPropertyType::String)
                .push_string("Cluster");

            sub_deformer
                .create("Version")
                .create_property(FbxPropertyType::Integer32)
                .push(100u32);

            let indices_buffer = sub_deformer
                .create("Indexes")
                .create_property(FbxPropertyType::Integer32Array);

            for index in weights.keys() {
                indices_buffer.push(*index as u32);
            }

            let value_buffer = sub_deformer
                .create("Weights")
                .create_property(FbxPropertyType::Float64Array);

            for weight in weights.values() {
                value_buffer.push(*weight as f64);
            }

            let transform_link_matrix = model.skeleton.bones[bone_id as usize].world_matrix();
            let transform_matrix = transform_link_matrix.inverse();

            let transform = sub_deformer
                .create("Transform")
                .create_property(FbxPropertyType::Float64Array);

            for i in 0..16 {
                transform.push(transform_matrix[i] as f64);
            }

            let transform_link = sub_deformer
                .create("TransformLink")
                .create_property(FbxPropertyType::Float64Array);

            for i in 0..16 {
                transform_link.push(transform_link_matrix[i] as f64);
            }

            let sub_deformer_hash = FbxPropertyValue::from(sub_deformer);

            add_object_connection(root.connections_node(), sub_deformer_hash, deformer_hash);
            add_object_connection(
                root.connections_node(),
                joints_map[&(bone_id as usize)],
                sub_deformer_hash,
            );
        }
    }

    let writer = BufWriter::new(File::create(path.as_ref().with_extension("fbx"))?);

    root.write(writer)?;

    Ok(())
}
