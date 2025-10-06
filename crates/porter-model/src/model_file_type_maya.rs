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
use crate::VertexColor;

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
            "requires maya \"8.5\";\ncurrentUnit -l centimeter -a degree -t film;\nfileInfo \"application\" \"maya\";\nfileInfo \"product\" \"Maya Unlimited 8.5\";\nfileInfo \"version\" \"8.5\";\nfileInfo \"cutIdentifier\" \"200612162224-692032\";",
            "createNode transform -s -n \"persp\";\n\tsetAttr \".v\" no;\n\tsetAttr \".t\" -type \"double3\" 48.186233840145825 37.816674066853686 41.0540421364379 ;\n\tsetAttr \".r\" -type \"double3\" -29.738352729603015 49.400000000000432 0 ;\ncreateNode camera -s -n \"perspShape\" -p \"persp\";\n\tsetAttr -k off \".v\" no;\n\tsetAttr \".fl\" 34.999999999999993;\n\tsetAttr \".fcp\" 10000;\n\tsetAttr \".coi\" 73.724849603665149;\n\tsetAttr \".imn\" -type \"string\" \"persp\";\n\tsetAttr \".den\" -type \"string\" \"persp_depth\";\n\tsetAttr \".man\" -type \"string\" \"persp_mask\";\n\tsetAttr \".hc\" -type \"string\" \"viewSet -p %camera\";\ncreateNode transform -s -n \"top\";\n\tsetAttr \".v\" no;\n\tsetAttr \".t\" -type \"double3\" 0 100.1 0 ;\n\tsetAttr \".r\" -type \"double3\" -89.999999999999986 0 0 ;\ncreateNode camera -s -n \"topShape\" -p \"top\";\n\tsetAttr -k off \".v\" no;\n\tsetAttr \".rnd\" no;\n\tsetAttr \".coi\" 100.1;\n\tsetAttr \".ow\" 30;\n\tsetAttr \".imn\" -type \"string\" \"top\";\n\tsetAttr \".den\" -type \"string\" \"top_depth\";\n\tsetAttr \".man\" -type \"string\" \"top_mask\";\n\tsetAttr \".hc\" -type \"string\" \"viewSet -t %camera\";\n\tsetAttr \".o\" yes;\ncreateNode transform -s -n \"front\";\n\tsetAttr \".v\" no;\n\tsetAttr \".t\" -type \"double3\" 0 0 100.1 ;\ncreateNode camera -s -n \"frontShape\" -p \"front\";\n\tsetAttr -k off \".v\" no;\n\tsetAttr \".rnd\" no;\n\tsetAttr \".coi\" 100.1;\n\tsetAttr \".ow\" 30;\n\tsetAttr \".imn\" -type \"string\" \"front\";\n\tsetAttr \".den\" -type \"string\" \"front_depth\";\n\tsetAttr \".man\" -type \"string\" \"front_mask\";\n\tsetAttr \".hc\" -type \"string\" \"viewSet -f %camera\";\n\tsetAttr \".o\" yes;\ncreateNode transform -s -n \"side\";\n\tsetAttr \".v\" no;\n\tsetAttr \".t\" -type \"double3\" 100.1 0 0 ;\n\tsetAttr \".r\" -type \"double3\" 0 89.999999999999986 0 ;\ncreateNode camera -s -n \"sideShape\" -p \"side\";\n\tsetAttr -k off \".v\" no;\n\tsetAttr \".rnd\" no;\n\tsetAttr \".coi\" 100.1;\n\tsetAttr \".ow\" 30;\n\tsetAttr \".imn\" -type \"string\" \"side\";\n\tsetAttr \".den\" -type \"string\" \"side_depth\";\n\tsetAttr \".man\" -type \"string\" \"side_mask\";\n\tsetAttr \".hc\" -type \"string\" \"viewSet -s %camera\";\n\tsetAttr \".o\" yes;\ncreateNode lightLinker -n \"lightLinker1\";\n\tsetAttr -s 9 \".lnk\";\n\tsetAttr -s 9 \".slnk\";\ncreateNode displayLayerManager -n \"layerManager\";\ncreateNode displayLayer -n \"defaultLayer\";\ncreateNode renderLayerManager -n \"renderLayerManager\";\ncreateNode renderLayer -n \"defaultRenderLayer\";\n\tsetAttr \".g\" yes;\ncreateNode script -n \"sceneConfigurationScriptNode\";\n\tsetAttr \".b\" -type \"string\" \"playbackOptions -min 1 -max 24 -ast 1 -aet 48 \";\n\tsetAttr \".st\" 6;\nselect -ne :time1;\n\tsetAttr \".o\" 1;\nselect -ne :renderPartition;\n\tsetAttr -s 2 \".st\";\nselect -ne :renderGlobalsList1;\nselect -ne :defaultShaderList1;\n\tsetAttr -s 2 \".s\";\nselect -ne :postProcessList1;\n\tsetAttr -s 2 \".p\";\nselect -ne :lightList1;\nselect -ne :initialShadingGroup;\n\tsetAttr \".ro\" yes;\nselect -ne :initialParticleSE;\n\tsetAttr \".ro\" yes;\nselect -ne :hardwareRenderGlobals;\n\tsetAttr \".ctrs\" 256;\n\tsetAttr \".btrs\" 512;\nselect -ne :defaultHardwareRenderGlobals;\n\tsetAttr \".fn\" -type \"string\" \"im\";\n\tsetAttr \".res\" -type \"string\" \"ntsc_4d 646 485 1.333\";\nselect -ne :ikSystem;\n\tsetAttr -s 4 \".sol\";\nconnectAttr \":defaultLightSet.msg\" \"lightLinker1.lnk[0].llnk\";\nconnectAttr \":initialShadingGroup.msg\" \"lightLinker1.lnk[0].olnk\";\nconnectAttr \":defaultLightSet.msg\" \"lightLinker1.lnk[1].llnk\";\nconnectAttr \":initialParticleSE.msg\" \"lightLinker1.lnk[1].olnk\";\nconnectAttr \":defaultLightSet.msg\" \"lightLinker1.slnk[0].sllk\";\nconnectAttr \":initialShadingGroup.msg\" \"lightLinker1.slnk[0].solk\";\nconnectAttr \":defaultLightSet.msg\" \"lightLinker1.slnk[1].sllk\";\nconnectAttr \":initialParticleSE.msg\" \"lightLinker1.slnk[1].solk\";\nconnectAttr \"layerManager.dli[0]\" \"defaultLayer.id\";\nconnectAttr \"renderLayerManager.rlmi[0]\" \"defaultRenderLayer.rlid\";\nconnectAttr \"lightLinker1.msg\" \":lightList1.ln\" -na;"
        )
    )?;

    writeln!(
        maya,
        concat!("createNode transform -n \"{}\";\n", "setAttr \".ove\" yes;"),
        file_name
    )?;

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        writeln!(
            maya,
            concat!(
                "createNode transform -n \"PorterMesh_{:02x}_{}\" -p \"{}\";\n",
                "setAttr \".rp\" -type \"double3\" 0.000000 0.000000 0.000000 ;\nsetAttr \".sp\" -type \"double3\" 0.000000 0.000000 0.000000 ;\n",
                "createNode mesh -n \"MeshShape_{}\" -p \"PorterMesh_{:02x}_{}\";\n",
                "setAttr -k off \".v\";\nsetAttr \".vir\" yes;\nsetAttr \".vif\" yes;\n",
                "setAttr -s {} \".uvst\";",
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

        for i in 1..mesh.vertices.uv_layers() + 1 {
            if mesh.vertices.len() == 1 {
                writeln!(
                    maya,
                    concat!(
                        "setAttr \".uvst[{}].uvsn\" -type \"string\" \"map{}\";\n",
                        "setAttr -s 1 \".uvst[0].uvsp[0]\" -type \"float2\"",
                    ),
                    i - 1,
                    i
                )?;
            } else {
                writeln!(
                    maya,
                    concat!(
                        "setAttr \".uvst[{}].uvsn\" -type \"string\" \"map{}\";\n",
                        "setAttr -s {} \".uvst[0].uvsp[0:{}]\" -type \"float2\"",
                    ),
                    i - 1,
                    i,
                    mesh.vertices.len(),
                    mesh.vertices.len() - 1
                )?;
            }

            for v in 0..mesh.vertices.len() {
                let uv = mesh.vertices.vertex(v).uv(i - 1);

                write!(maya, " {} {}", uv.x, 1.0 - uv.y)?;
            }

            writeln!(maya, ";")?;
        }

        write!(
            maya,
            concat!(
                "setAttr \".cuvs\" -type \"string\" \"map1\";\nsetAttr \".dcol\" yes;\nsetAttr \".dcc\" -type \"string\" \"Ambient+Diffuse\";\nsetAttr \".ccls\" -type \"string\" \"colorSet1\";\nsetAttr \".clst[0].clsn\" -type \"string\" \"colorSet1\";\n",
                "setAttr -s {} \".clst[0].clsp\";\n",
                "setAttr \".clst[0].clsp[0:{}]\"",
            ),
            (mesh.faces.len() * 3),
            (mesh.faces.len() * 3) - 1
        )?;

        for face in &mesh.faces {
            let vertex1 = if mesh.vertices.colors() > 0 {
                mesh.vertices.vertex(face.i3 as usize).color(0)
            } else {
                VertexColor::new(255, 255, 255, 255)
            };

            let vertex2 = if mesh.vertices.colors() > 0 {
                mesh.vertices.vertex(face.i2 as usize).color(0)
            } else {
                VertexColor::new(255, 255, 255, 255)
            };

            let vertex3 = if mesh.vertices.colors() > 0 {
                mesh.vertices.vertex(face.i1 as usize).color(0)
            } else {
                VertexColor::new(255, 255, 255, 255)
            };

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

        writeln!(
            maya,
            concat!(
                ";\n",
                "setAttr \".covm[0]\"  0 1 1;\nsetAttr \".cdvm[0]\"  0 1 1;\nsetAttr -s {} \".vt\";"
            ),
            mesh.vertices.len()
        )?;

        if mesh.vertices.len() == 1 {
            write!(maya, "setAttr \".vt[0]\"")?;
        } else {
            write!(maya, "setAttr \".vt[0:{}]\"", mesh.vertices.len() - 1)?;
        }

        for v in 0..mesh.vertices.len() {
            let position = mesh.vertices.vertex(v).position();

            write!(maya, " {} {} {}", position.x, position.y, position.z)?;
        }

        write!(
            maya,
            concat!(";\n", "setAttr -s {} \".ed\";\n", "setAttr \".ed[0:{}]\""),
            mesh.faces.len() * 3,
            (mesh.faces.len() * 3) - 1
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

        write!(
            maya,
            concat!(
                ";\n",
                "setAttr -s {} \".n\";\n",
                "setAttr \".n[0:{}]\" -type \"float3\""
            ),
            mesh.faces.len() * 3,
            (mesh.faces.len() * 3) - 1
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

        if mesh.faces.len() == 1 {
            write!(
                maya,
                "setAttr -s {} \".fc[0]\" -type \"polyFaces\"",
                mesh.faces.len()
            )?;
        } else {
            write!(
                maya,
                "setAttr -s {} \".fc[0:{}]\" -type \"polyFaces\"",
                mesh.faces.len(),
                mesh.faces.len() - 1
            )?;
        }

        for (face_index, face) in mesh.faces.iter().enumerate() {
            let face_indices = face_index * 3;

            write!(
                maya,
                " f 3 {} {} {}",
                face_indices,
                face_indices + 1,
                face_indices + 2
            )?;

            for u in 0..mesh.vertices.uv_layers() {
                write!(maya, " mu {} 3 {} {} {}", u, { face.i3 }, { face.i2 }, {
                    face.i1
                })?;
            }

            write!(
                maya,
                " mc 0 3 {} {} {}",
                face_indices,
                face_indices + 1,
                face_indices + 2
            )?;
        }

        writeln!(
            maya,
            concat!(
                ";\n",
                "setAttr \".cd\" -type \"dataPolyComponent\" Index_Data Edge 0 ;\nsetAttr \".cvd\" -type \"dataPolyComponent\" Index_Data Vertex 0 ;\nsetAttr \".hfd\" -type \"dataPolyComponent\" Index_Data Face 0 ;"
            )
        )?;
    }

    writeln!(maya)?;

    for material in &model.materials {
        writeln!(
            maya,
            concat!(
                "createNode shadingEngine -n \"{}SG\";\n",
                "setAttr \".ihi\" 0;\n",
                "setAttr \".ro\" yes;\n",
                "createNode materialInfo -n \"{}MI\";\r\ncreateNode lambert -n \"{}\";\n",
                "createNode place2dTexture -n \"{}P2DT\";"
            ),
            material.name, material.name, material.name, material.name
        )?;

        if let Some(diffuse) = material.base_color_texture() {
            writeln!(
                maya,
                concat!(
                    "createNode file -n \"{}FILE\";\n",
                    "setAttr \".ftn\" -type \"string\" \"{}\";",
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
                    "connectAttr \"{}P2DT.ofs\" \"{}FILE.fs\";\n",
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
                "connectAttr \"{}FILE.msg\" \":defaultTextureList1.tx\" -na;\n"
            ),
            material.name, material.name, material.name, material.name
        )?;

        light_connection_index += 1;
    }

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        writeln!(
            maya,
            "setAttr -s {} \"MeshShape_{}.iog\";",
            mesh.vertices.uv_layers(),
            mesh_index
        )?;

        if mesh.vertices.uv_layers() > 0
            && let Some(material_index) = mesh.material
        {
            writeln!(
                maya,
                "connectAttr \"MeshShape_{}.iog[{}]\" \"{}SG.dsm\" -na;",
                mesh_index, 0, model.materials[material_index].name
            )?;
        }
    }

    if model.skeleton.bones.is_empty() {
        return Ok(());
    }

    writeln!(
        maya,
        "createNode transform -n \"Joints\";\nsetAttr \".ove\" yes;\n"
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
                "addAttr -ci true -sn \"liw\" -ln \"lockInfluenceWeights\" -bt \"lock\" -min 0 -max 1 -at \"bool\";\n",
                "setAttr \".uoc\" yes;\n",
                "setAttr \".ove\" yes;\n",
                "setAttr \".t\" -type \"double3\" {} {} {} ;\n",
                "setAttr \".mnrl\" -type \"double3\" -360 -360 -360 ;\n",
                "setAttr \".mxrl\" -type \"double3\" 360 360 360 ;\n",
                "setAttr \".radi\"   1.0;\n",
                "setAttr \".r\" -type \"double3\" {} {} {};\n",
                "setAttr \".scale\" -type \"double3\" {} {} {};\n"
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
        "/*\n* Autodesk Maya Bind Script\n* Exported by PorterLib\n*/\n"
    )?;

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        writeln!(
            bind,
            concat!(
                "global proc PorterMesh_{:02x}_{}_BindFunc()\n{{\n",
                "   select -r PorterMesh_{:02x}_{};",
            ),
            hash, mesh_index, hash, mesh_index
        )?;

        let maximum_influence = mesh.vertices.maximum_influence();

        let mut bone_map_index = 0;
        let mut bone_set: HashSet<u16> = HashSet::new();
        let mut reverse_bone_map: HashMap<u32, u32> = HashMap::new();
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

                    reverse_bone_map.insert(bone_map_index, weight.bone as u32);

                    bone_map_index += 1;
                }
            }
        }

        for bone in &bone_names {
            writeln!(bind, "   select -add {};", bone)?;
        }

        writeln!(
            bind,
            concat!(
                "   newSkinCluster \"-toSelectedBones -mi {} -omi true -dr 5.0 -rui false\";\n",
                "   string $clu = findRelatedSkinCluster(\"PorterMesh_{:02x}_{}\");",
            ),
            maximum_influence, hash, mesh_index
        )?;

        if bone_names.len() > 1 {
            write!(
                bind,
                "   int $NV = {};\n   matrix $WM[{}][{}] = <<",
                mesh.vertices.len(),
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

                        if weight.bone == reverse_bone_map[&(b as u32)] as u16 {
                            weight_value += weight.value;
                        }
                    }

                    write!(bind, "{}", weight_value)?;
                }
            }

            writeln!(bind, ">>;")?;
            write!(bind, "   for ($i = 0; $i < $NV; $i++) {{")?;

            write!(
                bind,
                " setAttr($clu + \".weightList[\" + $i + \"].weights[0:{}]\")",
                bone_names.len() - 1
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
            "   catch(PorterMesh_{:02x}_{}_BindFunc());",
            hash, mesh_index
        )?;
    }

    writeln!(
        bind,
        "}}\n\nglobal proc NamespacePurge()\n{{\n   string $allNodes[] = `ls`;\n   for($node in $allNodes) {{\n      string $buffer[];\n      tokenize $node \":\" $buffer;\n      string $newName = $buffer[size($buffer)-1];\n       catchQuiet(`rename $node $newName`);\n   }}\n}}\n\nprint(\"Currently binding the current model, please wait...\\n\");\nNamespacePurge();\nRunAdvancedScript();\nprint(\"The model has been binded.\\n\");\n"
    )?;

    Ok(())
}
