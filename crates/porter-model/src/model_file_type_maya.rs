use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use porter_math::Angles;

use porter_utils::BufferWriteExt;
use porter_utils::HashExt;

use crate::Model;
use crate::ModelError;

/// Writes a model in maya format to the given path.
pub fn to_maya<P: AsRef<Path>>(path: P, model: &Model) -> Result<(), ModelError> {
    let path = path.as_ref();
    let file_name = path
        .file_stem()
        .map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("porter_model"));
    let hash = file_name.hash_xxh364() as u32;

    let mut maya = File::create(path.with_extension("ma"))?.buffer_write();

    writeln!(
        maya,
        concat!(
            "//Maya ASCII 8.5 scene\n\n",
            "// Exported by PorterLib\n",
            "// Please credit DTZxPorter for use of this asset!\n\n",
            "requires maya \"8.5\";\n",
            "currentUnit -l centimeter -a degree -t film;\n",
            "fileInfo \"application\" \"maya\";\n",
            "fileInfo \"product\" \"Maya Unlimited 8.5\";\n",
            "fileInfo \"version\" \"8.5\";\n",
            "fileInfo \"cutIdentifier\" \"200612162224-692032\";\n",
            "createNode transform -s -n \"persp\";\n",
            "\tsetAttr \".v\" no;\n",
            "\tsetAttr \".t\" -type \"double3\" 48.186233840145825 37.816674066853686 41.0540421364379;\n",
            "\tsetAttr \".r\" -type \"double3\" -29.738352729603015 49.400000000000432 0;\n",
            "createNode camera -s -n \"perspShape\" -p \"persp\";\n",
            "\tsetAttr -k off \".v\" no;\n\tsetAttr \".fl\" 34.999999999999993;\n",
            "\tsetAttr \".coi\" 73.724849603665149;\n",
            "\tsetAttr \".imn\" -type \"string\" \"persp\";\n",
            "\tsetAttr \".fcp\" 10000;\n",
            "\tsetAttr \".den\" -type \"string\" \"persp_depth\";\n",
            "\tsetAttr \".man\" -type \"string\" \"persp_mask\";\n",
            "\tsetAttr \".hc\" -type \"string\" \"viewSet -p %camera\";\n",
            "createNode transform -s -n \"top\";\n",
            "\tsetAttr \".v\" no;\n",
            "\tsetAttr \".t\" -type \"double3\" 0 100.1 0;\n",
            "\tsetAttr \".r\" -type \"double3\" -89.999999999999986 0 0;\n",
            "createNode camera -s -n \"topShape\" -p \"top\";\n",
            "\tsetAttr -k off \".v\" no;\n",
            "\tsetAttr \".rnd\" no;\n",
            "\tsetAttr \".coi\" 100.1;\n",
            "\tsetAttr \".ow\" 30;\n",
            "\tsetAttr \".imn\" -type \"string\" \"top\";\n",
            "\tsetAttr \".den\" -type \"string\" \"top_depth\";\n",
            "\tsetAttr \".man\" -type \"string\" \"top_mask\";\n",
            "\tsetAttr \".hc\" -type \"string\" \"viewSet -t %camera\";\n",
            "\tsetAttr \".o\" yes;\n",
            "createNode transform -s -n \"front\";\n",
            "\tsetAttr \".v\" no;\n",
            "\tsetAttr \".t\" -type \"double3\" 0 0 100.1;\n",
            "createNode camera -s -n \"frontShape\" -p \"front\";\n",
            "\tsetAttr -k off \".v\" no;\n",
            "\tsetAttr \".rnd\" no;\n",
            "\tsetAttr \".coi\" 100.1;\n",
            "\tsetAttr \".ow\" 30;\n",
            "\tsetAttr \".imn\" -type \"string\" \"front\";\n",
            "\tsetAttr \".den\" -type \"string\" \"front_depth\";\n",
            "\tsetAttr \".man\" -type \"string\" \"front_mask\";\n",
            "\tsetAttr \".hc\" -type \"string\" \"viewSet -f %camera\";\n",
            "\tsetAttr \".o\" yes;\n",
            "createNode transform -s -n \"side\";\n",
            "\tsetAttr \".v\" no;\n",
            "\tsetAttr \".t\" -type \"double3\" 100.1 0 0;\n",
            "\tsetAttr \".r\" -type \"double3\" 0 89.999999999999986 0;\n",
            "createNode camera -s -n \"sideShape\" -p \"side\";\n",
            "\tsetAttr -k off \".v\" no;\n",
            "\tsetAttr \".rnd\" no;\n",
            "\tsetAttr \".coi\" 100.1;\n",
            "\tsetAttr \".ow\" 30;\n",
            "\tsetAttr \".imn\" -type \"string\" \"side\";\n",
            "\tsetAttr \".den\" -type \"string\" \"side_depth\";\n",
            "\tsetAttr \".man\" -type \"string\" \"side_mask\";\n",
            "\tsetAttr \".hc\" -type \"string\" \"viewSet -s %camera\";\n",
            "\tsetAttr \".o\" yes;\n",
            "createNode lightLinker -n \"lightLinker1\";\n",
            "\tsetAttr -s 9 \".lnk\";\n",
            "\tsetAttr -s 9 \".slnk\";\n",
            "createNode displayLayerManager -n \"layerManager\";\n",
            "createNode displayLayer -n \"defaultLayer\";\n",
            "createNode renderLayerManager -n \"renderLayerManager\";\n",
            "createNode renderLayer -n \"defaultRenderLayer\";\n",
            "\tsetAttr \".g\" yes;\n",
            "createNode script -n \"sceneConfigurationScriptNode\";\n",
            "\tsetAttr \".b\" -type \"string\" \"playbackOptions -min 1 -max 24 -ast 1 -aet 48 \";\n",
            "\tsetAttr \".st\" 6;\n",
            "select -ne :time1;\n",
            "\tsetAttr \".o\" 1;\n",
            "select -ne :renderPartition;\n",
            "\tsetAttr -s 2 \".st\";\n",
            "select -ne :renderGlobalsList1;\n",
            "select -ne :defaultShaderList1;\n",
            "\tsetAttr -s 2 \".s\";\n",
            "select -ne :postProcessList1;\n",
            "\tsetAttr -s 2 \".p\";\n",
            "select -ne :lightList1;\n",
            "select -ne :initialShadingGroup;\n",
            "\tsetAttr \".ro\" yes;\n",
            "select -ne :initialParticleSE;\n",
            "\tsetAttr \".ro\" yes;\n",
            "select -ne :hardwareRenderGlobals;\n",
            "\tsetAttr \".ctrs\" 256;\n",
            "\tsetAttr \".btrs\" 512;\n",
            "select -ne :defaultHardwareRenderGlobals;\n",
            "\tsetAttr \".fn\" -type \"string\" \"im\";\n",
            "\tsetAttr \".res\" -type \"string\" \"ntsc_4d 646 485 1.333\";\n",
            "select -ne :ikSystem;\n",
            "\tsetAttr -s 4 \".sol\";\n",
            "connectAttr \":defaultLightSet.msg\" \"lightLinker1.lnk[0].llnk\";\n",
            "connectAttr \":initialShadingGroup.msg\" \"lightLinker1.lnk[0].olnk\";\n",
            "connectAttr \":defaultLightSet.msg\" \"lightLinker1.lnk[1].llnk\";\n",
            "connectAttr \":initialParticleSE.msg\" \"lightLinker1.lnk[1].olnk\";\n",
            "connectAttr \":defaultLightSet.msg\" \"lightLinker1.slnk[0].sllk\";\n",
            "connectAttr \":initialShadingGroup.msg\" \"lightLinker1.slnk[0].solk\";\n",
            "connectAttr \":defaultLightSet.msg\" \"lightLinker1.slnk[1].sllk\";\n",
            "connectAttr \":initialParticleSE.msg\" \"lightLinker1.slnk[1].solk\";\n",
            "connectAttr \"layerManager.dli[0]\" \"defaultLayer.id\";\n",
            "connectAttr \"renderLayerManager.rlmi[0]\" \"defaultRenderLayer.rlid\";\n",
            "connectAttr \"lightLinker1.msg\" \":lightList1.ln\" -na;"
        )
    )?;

    writeln!(
        maya,
        concat!(
            "createNode transform -n \"{}\";\n",
            "\tsetAttr \".ove\" yes;"
        ),
        file_name
    )?;

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        writeln!(
            maya,
            concat!(
                "createNode transform -n \"PorterMesh_{:02x}_{}\" -p \"{}\";\n",
                "\tsetAttr \".rp\" -type \"double3\" 0.000000 0.000000 0.000000;\n",
                "\tsetAttr \".sp\" -type \"double3\" 0.000000 0.000000 0.000000;\n",
                "createNode mesh -n \"MeshShape_{}\" -p \"PorterMesh_{:02x}_{}\";\n",
                "\tsetAttr -k off \".v\";\n",
                "\tsetAttr \".vir\" yes;\n",
                "\tsetAttr \".vif\" yes;\n",
                "\tsetAttr -s {} \".uvst\";",
            ),
            hash,
            mesh_index,
            file_name,
            mesh_index,
            hash,
            mesh_index,
            mesh.vertices.uv_layers()
        )?;

        if mesh.vertices.is_empty() || mesh.faces.is_empty() {
            continue;
        }

        for uv_layer in 0..mesh.vertices.uv_layers() {
            write!(
                maya,
                concat!(
                    "\tsetAttr \".uvst[{}].uvsn\" -type \"string\" \"map{}\";\n",
                    "\tsetAttr -s {} \".uvst[{}].uvsp[{}]\" -type \"float2\"",
                ),
                uv_layer,
                uv_layer + 1,
                mesh.vertices.len(),
                uv_layer,
                maya_range(mesh.vertices.len())
            )?;

            for v in 0..mesh.vertices.len() {
                let uv = mesh.vertices.vertex(v).uv(uv_layer);

                write!(maya, " {} {}", uv.x, 1.0 - uv.y)?;
            }

            writeln!(maya, ";")?;
        }

        writeln!(
            maya,
            concat!(
                "\tsetAttr \".cuvs\" -type \"string\" \"map1\";\n",
                "\tsetAttr \".dcc\" -type \"string\" \"Ambient+Diffuse\";",
            )
        )?;

        for color_layer in 0..mesh.vertices.colors() {
            write!(
                maya,
                concat!(
                    "\tsetAttr \".ccls\" -type \"string\" \"colorSet{}\";\n",
                    "\tsetAttr \".clst[{}].clsn\" -type \"string\" \"colorSet{}\";\n",
                    "\tsetAttr -s {} \".clst[{}].clsp\";\n",
                    "\tsetAttr \".clst[{}].clsp[{}]\"",
                ),
                color_layer + 1,
                color_layer,
                color_layer + 1,
                mesh.faces.len() * 3,
                color_layer,
                color_layer,
                maya_range(mesh.faces.len() * 3)
            )?;

            for face in &mesh.faces {
                let vertex1 = mesh.vertices.vertex(face.i3 as usize).color(color_layer);
                let vertex2 = mesh.vertices.vertex(face.i2 as usize).color(color_layer);
                let vertex3 = mesh.vertices.vertex(face.i1 as usize).color(color_layer);

                write!(
                    maya,
                    " {} {} {} {} {} {} {} {} {} {} {} {}",
                    vertex1.r as f32 / 255.0,
                    vertex1.g as f32 / 255.0,
                    vertex1.b as f32 / 255.0,
                    vertex1.a as f32 / 255.0,
                    vertex2.r as f32 / 255.0,
                    vertex2.g as f32 / 255.0,
                    vertex2.b as f32 / 255.0,
                    vertex2.a as f32 / 255.0,
                    vertex3.r as f32 / 255.0,
                    vertex3.g as f32 / 255.0,
                    vertex3.b as f32 / 255.0,
                    vertex3.a as f32 / 255.0
                )?;
            }

            writeln!(maya, ";")?;
        }

        writeln!(
            maya,
            concat!(
                "\tsetAttr \".covm[0]\"  0 1 1;\n",
                "\tsetAttr \".cdvm[0]\"  0 1 1;\n",
                "\tsetAttr -s {} \".vt\";",
            ),
            mesh.vertices.len()
        )?;

        write!(
            maya,
            "\tsetAttr \".vt[{}]\"",
            maya_range(mesh.vertices.len())
        )?;

        for v in 0..mesh.vertices.len() {
            let position = mesh.vertices.vertex(v).position();

            write!(maya, " {} {} {}", position.x, position.y, position.z)?;
        }

        writeln!(maya, ";")?;

        write!(
            maya,
            concat!("\tsetAttr -s {} \".ed\";\n", "\tsetAttr \".ed[{}]\""),
            mesh.faces.len() * 3,
            maya_range(mesh.faces.len() * 3)
        )?;

        for face in &mesh.faces {
            write!(
                maya,
                " {} {} 0 {} {} 0 {} {} 0",
                { face.i3 },
                { face.i2 },
                { face.i2 },
                { face.i1 },
                { face.i1 },
                { face.i3 }
            )?;
        }

        writeln!(maya, ";")?;

        write!(
            maya,
            concat!(
                "\tsetAttr -s {} \".n\";\n",
                "\tsetAttr \".n[{}]\" -type \"float3\""
            ),
            mesh.faces.len() * 3,
            maya_range(mesh.faces.len() * 3)
        )?;

        for face in &mesh.faces {
            let vertex1 = mesh.vertices.vertex(face.i3 as usize).normal();
            let vertex2 = mesh.vertices.vertex(face.i2 as usize).normal();
            let vertex3 = mesh.vertices.vertex(face.i1 as usize).normal();

            write!(
                maya,
                " {} {} {} {} {} {} {} {} {}",
                vertex1.x,
                vertex1.y,
                vertex1.z,
                vertex2.x,
                vertex2.y,
                vertex2.z,
                vertex3.x,
                vertex3.y,
                vertex3.z
            )?;
        }

        writeln!(maya, ";")?;

        write!(
            maya,
            "\tsetAttr -s {} \".fc[{}]\" -type \"polyFaces\"",
            mesh.faces.len(),
            maya_range(mesh.faces.len())
        )?;

        for (face_index, face) in mesh.faces.iter().enumerate() {
            let face_indices = face_index * 3;

            write!(
                maya,
                " f 3 {} {} {}",
                face_indices,
                face_indices + 1,
                face_indices + 2
            )?;

            for uv_layer in 0..mesh.vertices.uv_layers() {
                write!(
                    maya,
                    " mu {} 3 {} {} {}",
                    uv_layer,
                    { face.i3 },
                    { face.i2 },
                    { face.i1 }
                )?;
            }

            for color_layer in 0..mesh.vertices.colors() {
                write!(
                    maya,
                    " mc {} 3 {} {} {}",
                    color_layer,
                    face_indices,
                    face_indices + 1,
                    face_indices + 2
                )?;
            }
        }

        writeln!(maya, ";")?;

        writeln!(
            maya,
            concat!(
                "\tsetAttr \".cd\" -type \"dataPolyComponent\" Index_Data Edge 0;\n",
                "\tsetAttr \".cvd\" -type \"dataPolyComponent\" Index_Data Vertex 0;\n",
                "\tsetAttr \".hfd\" -type \"dataPolyComponent\" Index_Data Face 0;"
            )
        )?;
    }

    for material in &model.materials {
        writeln!(
            maya,
            concat!(
                "createNode shadingEngine -n \"{}SG\";\n",
                "\tsetAttr \".ihi\" 0;\n",
                "\tsetAttr \".ro\" yes;\n",
                "createNode materialInfo -n \"{}MI\";\n",
                "createNode lambert -n \"{}\";\n",
                "createNode place2dTexture -n \"{}P2DT\";"
            ),
            material.name, material.name, material.name, material.name
        )?;

        if let Some(diffuse) = material.base_color_texture() {
            writeln!(
                maya,
                concat!(
                    "createNode file -n \"{}FILE\";\n",
                    "\tsetAttr \".ftn\" -type \"string\" \"{}\";",
                ),
                material.name,
                diffuse.file_name.replace('\\', "\\\\")
            )?;
        }
    }

    let mut light_connection_index = 2;

    for material in &model.materials {
        writeln!(
            maya,
            concat!(
                "connectAttr \":defaultLightSet.msg\" \"lightLinker1.lnk[{}].llnk\";\n",
                "connectAttr \"{}SG.msg\" \"lightLinker1.lnk[{}].olnk\";\n",
                "connectAttr \":defaultLightSet.msg\" \"lightLinker1.slnk[{}].sllk\";\n",
                "connectAttr \"{}SG.msg\" \"lightLinker1.slnk[{}].solk\";\n",
                "connectAttr \"{}.oc\" \"{}SG.ss\";\n",
                "connectAttr \"{}SG.msg\" \"{}MI.sg\";\n",
                "connectAttr \"{}.msg\" \"{}MI.m\";",
            ),
            light_connection_index,
            material.name,
            light_connection_index,
            light_connection_index,
            material.name,
            light_connection_index,
            material.name,
            material.name,
            material.name,
            material.name,
            material.name,
            material.name,
        )?;

        let has_diffuse = material.base_color_texture().is_some();

        if has_diffuse {
            writeln!(
                maya,
                concat!(
                    "connectAttr \"{}FILE.msg\" \"{}MI.t\" -na;\n",
                    "connectAttr \"{}FILE.oc\" \"{}.c\";\n",
                    "connectAttr \"{}P2DT.c\" \"{}FILE.c\";\n",
                    "connectAttr \"{}P2DT.tf\" \"{}FILE.tf\";\n",
                    "connectAttr \"{}P2DT.rf\" \"{}FILE.rf\";\n",
                    "connectAttr \"{}P2DT.mu\" \"{}FILE.mu\";\n",
                    "connectAttr \"{}P2DT.mv\" \"{}FILE.mv\";\n",
                    "connectAttr \"{}P2DT.s\" \"{}FILE.s\";\n",
                    "connectAttr \"{}P2DT.wu\" \"{}FILE.wu\";\n",
                    "connectAttr \"{}P2DT.wv\" \"{}FILE.wv\";\n",
                    "connectAttr \"{}P2DT.re\" \"{}FILE.re\";\n",
                    "connectAttr \"{}P2DT.of\" \"{}FILE.of\";\n",
                    "connectAttr \"{}P2DT.r\" \"{}FILE.ro\";\n",
                    "connectAttr \"{}P2DT.n\" \"{}FILE.n\";\n",
                    "connectAttr \"{}P2DT.vt1\" \"{}FILE.vt1\";\n",
                    "connectAttr \"{}P2DT.vt2\" \"{}FILE.vt2\";\n",
                    "connectAttr \"{}P2DT.vt3\" \"{}FILE.vt3\";\n",
                    "connectAttr \"{}P2DT.vc1\" \"{}FILE.vc1\";\n",
                    "connectAttr \"{}P2DT.o\" \"{}FILE.uv\";\n",
                    "connectAttr \"{}P2DT.ofs\" \"{}FILE.fs\";",
                ),
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
                material.name,
            )?;
        }

        writeln!(
            maya,
            concat!(
                "connectAttr \"{}SG.pa\" \":renderPartition.st\" -na;\n",
                "connectAttr \"{}.msg\" \":defaultShaderList1.s\" -na;\n",
                "connectAttr \"{}P2DT.msg\" \":defaultRenderUtilityList1.u\" -na;\n",
                "connectAttr \"{}FILE.msg\" \":defaultTextureList1.tx\" -na;"
            ),
            material.name, material.name, material.name, material.name
        )?;

        light_connection_index += 1;
    }

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        if mesh.vertices.uv_layers() > 0
            && let Some(material_index) = mesh.material
        {
            writeln!(
                maya,
                "connectAttr \"MeshShape_{}.iog\" \"{}SG.dsm\" -na;",
                mesh_index, model.materials[material_index].name
            )?;
        }
    }

    if model.skeleton.bones.is_empty() {
        return Ok(());
    }

    writeln!(
        maya,
        concat!(
            "createNode transform -n \"Joints\";\n",
            "\tsetAttr \".ove\" yes;"
        )
    )?;

    for (bone_index, bone) in model.skeleton.bones.iter().enumerate() {
        if bone.parent == -1 {
            writeln!(
                maya,
                "createNode joint -n \"{}\" -p \"Joints\";",
                bone.name
                    .as_deref()
                    .unwrap_or(&format!("porter_bone_{}", bone_index))
            )?;
        } else {
            writeln!(
                maya,
                "createNode joint -n \"{}\" -p \"{}\";",
                bone.name
                    .as_deref()
                    .unwrap_or(&format!("porter_bone_{}", bone_index)),
                model.skeleton.bones[bone.parent as usize]
                    .name
                    .as_deref()
                    .unwrap_or(&format!("porter_bone_{}", bone.parent))
            )?;
        }

        let rotation = bone.local_rotation.to_euler(Angles::Degrees);
        let position = bone.local_position;
        let scale = bone.local_scale;

        writeln!(
            maya,
            concat!(
                "\taddAttr -ci true -sn \"liw\" -ln \"lockInfluenceWeights\" -bt \"lock\" -min 0 -max 1 -at \"bool\";\n",
                "\tsetAttr \".uoc\" yes;\n",
                "\tsetAttr \".ove\" yes;\n",
                "\tsetAttr \".t\" -type \"double3\" {} {} {};\n",
                "\tsetAttr \".mnrl\" -type \"double3\" -360 -360 -360;\n",
                "\tsetAttr \".mxrl\" -type \"double3\" 360 360 360;\n",
                "\tsetAttr \".radi\"   1.0;\n",
                "\tsetAttr \".r\" -type \"double3\" {} {} {};\n",
                "\tsetAttr \".scale\" -type \"double3\" {} {} {};"
            ),
            position.x,
            position.y,
            position.z,
            rotation.x,
            rotation.y,
            rotation.z,
            scale.x,
            scale.y,
            scale.z
        )?;
    }

    let mut bind = File::create(
        path.with_file_name(format!("{}_BIND", file_name))
            .with_extension("mel"),
    )?
    .buffer_write();

    writeln!(
        bind,
        concat!(
            "/*\n",
            "* Autodesk Maya Bind Script\n",
            "* Exported by PorterLib\n",
            "*/\n"
        )
    )?;

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        writeln!(
            bind,
            concat!(
                "global proc PorterMesh_{:02x}_{}_BindFunc()\n",
                "{{\n",
                "\tselect -r PorterMesh_{:02x}_{};",
            ),
            hash, mesh_index, hash, mesh_index
        )?;

        let maximum_influence = mesh.vertices.maximum_influence();

        let mut bone_map_index = 0;
        let mut bone_set: HashSet<u16> = HashSet::new();
        let mut reverse_bone_map: HashMap<u32, u16> = HashMap::new();
        let mut bone_names = Vec::new();

        for v in 0..mesh.vertices.len() {
            let vertex = mesh.vertices.vertex(v);

            for w in 0..mesh.vertices.maximum_influence() {
                let weight = vertex.weight(w);

                if bone_set.insert(weight.bone) {
                    bone_names.push(
                        model.skeleton.bones[weight.bone as usize]
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("porter_bone_{}", { weight.bone })),
                    );

                    reverse_bone_map.insert(bone_map_index, weight.bone);

                    bone_map_index += 1;
                }
            }
        }

        for bone in &bone_names {
            writeln!(bind, "\tselect -add {};", bone)?;
        }

        writeln!(
            bind,
            concat!(
                "\tnewSkinCluster \"-toSelectedBones -mi {} -omi true -dr 5.0 -rui false\";\n",
                "\tstring $clu = findRelatedSkinCluster(\"PorterMesh_{:02x}_{}\");",
            ),
            maximum_influence, hash, mesh_index
        )?;

        if bone_names.len() > 1 {
            write!(
                bind,
                "\tmatrix $WM[{}][{}] = <<",
                mesh.vertices.len(),
                bone_names.len()
            )?;

            for v in 0..mesh.vertices.len() {
                let vertex = mesh.vertices.vertex(v);

                if v != 0 {
                    write!(bind, ";")?;
                }

                for b in 0..bone_names.len() {
                    if b != 0 {
                        write!(bind, ",")?;
                    }

                    let mut weight_value = 0.0;

                    for w in 0..mesh.vertices.maximum_influence() {
                        let weight = vertex.weight(w);

                        if weight.bone == reverse_bone_map[&(b as u32)] {
                            weight_value += weight.value;
                        }
                    }

                    write!(bind, "{}", weight_value)?;
                }
            }

            writeln!(bind, ">>;")?;

            write!(
                bind,
                "\tfor ($i = 0; $i < {}; $i++) {{",
                mesh.vertices.len()
            )?;

            write!(
                bind,
                " setAttr($clu + \".weightList[\" + $i + \"].weights[{}]\")",
                maya_range(bone_names.len())
            )?;

            for b in 0..bone_names.len() {
                write!(bind, " $WM[$i][{}]", b)?;
            }

            writeln!(bind, "; }}")?;
        }

        writeln!(bind, "}}\n")?;
    }

    writeln!(bind, "global proc RunAdvancedScript()\n{{")?;

    for mesh_index in 0..model.meshes.len() {
        writeln!(
            bind,
            "\tcatch(PorterMesh_{:02x}_{}_BindFunc());",
            hash, mesh_index
        )?;
    }

    writeln!(
        bind,
        concat!(
            "}}\n",
            "\n",
            "global proc NamespacePurge()\n",
            "{{\n",
            "\tstring $allNodes[] = `ls`;\n",
            "\tfor($node in $allNodes) {{\n",
            "\t\tstring $buffer[];\n",
            "\t\ttokenize $node \":\" $buffer;\n",
            "\t\tstring $newName = $buffer[size($buffer)-1];\n",
            "\t\tcatchQuiet(`rename $node $newName`);\n",
            "\t}}\n",
            "}}\n",
            "\n",
            "print(\"Currently binding the current model, please wait...\\n\");\n",
            "NamespacePurge();\n",
            "RunAdvancedScript();\n",
            "print(\"The model has been binded.\\n\");"
        )
    )?;

    Ok(())
}

/// Utility method to format a maya range.
fn maya_range(len: usize) -> String {
    if len <= 1 {
        String::from("0")
    } else {
        format!("0:{}", len - 1)
    }
}
