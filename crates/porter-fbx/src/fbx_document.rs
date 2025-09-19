use std::io::Error;
use std::io::Seek;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use porter_utils::StructWriteExt;

use crate::FbxNode;
use crate::FbxPropertyType;

/// Footer data for the fbx.
const FOOTER_DATA: [u8; 166] = [
    0xFA, 0xBC, 0xAB, 0x09, 0xD0, 0xC8, 0xD4, 0x66, 0xB1, 0x76, 0xFB, 0x83, 0x1C, 0xF7, 0x26, 0x7E,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE8, 0x1C, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x5A, 0x8C, 0x6A, 0xDE, 0xF5, 0xD9, 0x7E, 0xEC, 0xE9,
    0x0C, 0xE3, 0x75, 0x8F, 0x29, 0x0B,
];

/// Header extension properties.
const HEADER_EXTENSION_PROPERTIES: [(&str, u32); 3] = [
    ("FBXHeaderVersion", 1003),
    ("FBXVersion", 7400),
    ("EncryptionType", 0),
];

/// Header extension time properties.
const HEADER_EXTENSION_TIME_PROPERTIES: [(&str, u32); 8] = [
    ("Version", 1000),
    ("Year", 2019),
    ("Month", 4),
    ("Day", 26),
    ("Hour", 10),
    ("Minute", 39),
    ("Second", 36),
    ("Millisecond", 240),
];

/// Header creator info.
const HEADER_EXTENSION_CREATOR_PROPERTY: (&str, &str) = ("Creator", "Exported by DTZxPorter");

/// Header file node type.
const HEADER_FILE_ID_NODE: (&str, [u8; 16]) = (
    "FileId",
    [
        0x28, 0xb3, 0x2a, 0xeb, 0xb6, 0x24, 0xcc, 0xc2, 0xbf, 0xc8, 0xb0, 0x2a, 0xa9, 0x2b, 0xfc,
        0xf1,
    ],
);

/// Header creation time.
const HEADER_CREATION_TIME_NODE: (&str, &str) = ("CreationTime", "1970-01-01 10:00:00:000");

/// Header node definitions.
const HEADER_DEFINITIONS_NODE: [(&str, &str, &str); 8] = [
    ("ObjectType", "GlobalSettings", ""),
    ("ObjectType", "NodeAttribute", ""),
    ("ObjectType", "Geometry", "FbxMesh"),
    ("ObjectType", "Model", "FbxNode"),
    ("ObjectType", "Pose", ""),
    ("ObjectType", "Deformer", ""),
    ("ObjectType", "Material", "FbxSurfacePhong"),
    ("ObjectType", "Texture", "FbxFileTexture"),
];

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct FbxHeader {
    magic: [u8; 0x15],
    version_minor: u16,
    version_major: u32,
}

/// A fbx document.
#[derive(Debug)]
pub struct FbxDocument {
    root_nodes: Vec<FbxNode>,
    hash_next: Arc<AtomicU64>,
    objects_node: usize,
    connections_node: usize,
    takes_node: usize,
    root_node: usize,
}

impl FbxDocument {
    /// Constructs a new fbx document instance.
    pub fn new() -> Self {
        let mut result = Self {
            root_nodes: Vec::new(),
            hash_next: Arc::new(AtomicU64::new(0)),
            objects_node: 0,
            connections_node: 0,
            takes_node: 0,
            root_node: 0,
        };

        result.initialize_fbx_header_extension();
        result.initialize_generics();
        result.initialize_global_settings();
        result.initialize_documents();
        result.initialize_references();
        result.initialize_definitions();
        result.initialize_dynamics();

        result
    }

    /// Creates a new root node with the given name.
    pub fn create<N: Into<String>>(&mut self, name: N) -> &mut FbxNode {
        self.root_nodes
            .push(FbxNode::new(name, self.hash_next.clone()));

        let index = self.root_nodes.len() - 1;

        self.root_nodes.get_mut(index).unwrap()
    }

    /// Serializes the document to the writer.
    pub fn write<W: Write + Seek>(&mut self, mut writer: W) -> Result<(), Error> {
        let header = FbxHeader {
            magic: *b"Kaydara FBX Binary  \0",
            version_minor: 26,
            version_major: 7400,
        };

        writer.write_struct(header)?;

        for child in &mut self.root_nodes {
            child.prepare();
        }

        for child in &self.root_nodes {
            child.write(&mut writer)?;
        }

        const HEADER_SIZE: usize =
            size_of::<u32>() + size_of::<u32>() + size_of::<u32>() + size_of::<u8>();

        writer.write_all(&[0; HEADER_SIZE])?;
        writer.write_all(&FOOTER_DATA)?;

        Ok(())
    }

    /// Gets the objects node of this document.
    pub fn objects_node(&mut self) -> &mut FbxNode {
        &mut self.root_nodes[self.objects_node]
    }

    /// Gets the connections node of this document.
    pub fn connections_node(&mut self) -> &mut FbxNode {
        &mut self.root_nodes[self.connections_node]
    }

    /// Gets the takes node of this document.
    pub fn takes_node(&mut self) -> &mut FbxNode {
        &mut self.root_nodes[self.takes_node]
    }

    /// Gets the root node of this document.
    pub fn root_node(&mut self) -> &mut FbxNode {
        &mut self.root_nodes[self.root_node][1][0]
    }

    /// Initializes the header extension nodes.
    fn initialize_fbx_header_extension(&mut self) {
        let header = self.create("FBXHeaderExtension");

        for property in HEADER_EXTENSION_PROPERTIES {
            header
                .create(property.0)
                .create_property(FbxPropertyType::Integer32)
                .push(property.1);
        }

        let header = self.create("CreationTimeStamp");

        for property in HEADER_EXTENSION_TIME_PROPERTIES {
            header
                .create(property.0)
                .create_property(FbxPropertyType::Integer32)
                .push(property.1);
        }
    }

    /// Initializes generics.
    fn initialize_generics(&mut self) {
        self.create(HEADER_FILE_ID_NODE.0)
            .create_property(FbxPropertyType::Raw)
            .push_raw(HEADER_FILE_ID_NODE.1);

        self.create(HEADER_CREATION_TIME_NODE.0)
            .create_property(FbxPropertyType::String)
            .push_string(HEADER_CREATION_TIME_NODE.1);

        self.create(HEADER_EXTENSION_CREATOR_PROPERTY.0)
            .create_property(FbxPropertyType::String)
            .push_string(HEADER_EXTENSION_CREATOR_PROPERTY.1);
    }

    /// Initializes global settings.
    fn initialize_global_settings(&mut self) {
        let settings = self.create("GlobalSettings");

        settings
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(1000u32);

        let properties = settings.create("Properties70");

        let prop = properties.create("P");

        prop.create_property(FbxPropertyType::String)
            .push_string("UpAxis");
        prop.create_property(FbxPropertyType::String)
            .push_string("int");
        prop.create_property(FbxPropertyType::String)
            .push_string("Integer");
        prop.create_property(FbxPropertyType::String)
            .push_string("");
        prop.create_property(FbxPropertyType::Integer32).push(1u32);

        let prop = properties.create("P");

        prop.create_property(FbxPropertyType::String)
            .push_string("FrontAxis");
        prop.create_property(FbxPropertyType::String)
            .push_string("int");
        prop.create_property(FbxPropertyType::String)
            .push_string("Integer");
        prop.create_property(FbxPropertyType::String)
            .push_string("");
        prop.create_property(FbxPropertyType::Integer32).push(2u32);
    }

    /// Initializes documents.
    fn initialize_documents(&mut self) {
        self.root_node = self.root_nodes.len();

        let documents = self.create("Documents");

        documents
            .create("Count")
            .create_property(FbxPropertyType::Integer32)
            .push(1u32);

        let document = documents.create("Document");

        document
            .create_property(FbxPropertyType::Integer64)
            .push(318825901u64);
        document
            .create_property(FbxPropertyType::String)
            .push_string("Scene");
        document
            .create_property(FbxPropertyType::String)
            .push_string("Scene");

        document.create("RootNode").create_hash();
    }

    /// Initializes references.
    fn initialize_references(&mut self) {
        self.create("References");
    }

    /// Initializes definitions.
    fn initialize_definitions(&mut self) {
        let definitions = self.create("Definitions");

        definitions
            .create("Version")
            .create_property(FbxPropertyType::Integer32)
            .push(100u32);

        definitions
            .create("Count")
            .create_property(FbxPropertyType::Integer32)
            .push(HEADER_DEFINITIONS_NODE.len() as u32);

        for definition in HEADER_DEFINITIONS_NODE {
            let def = definitions.create(definition.0);

            def.create_property(FbxPropertyType::String)
                .push_string(definition.1);

            def.create("Count")
                .create_property(FbxPropertyType::Integer32)
                .push(1u32);

            if !definition.2.is_empty() {
                def.create("PropertyTemplate")
                    .create_property(FbxPropertyType::String)
                    .push_string(definition.2);
            }
        }
    }

    /// Initializes dynamics.
    fn initialize_dynamics(&mut self) {
        self.objects_node = self.root_nodes.len();
        self.create("Objects");

        self.connections_node = self.root_nodes.len();
        self.create("Connections");

        self.takes_node = self.root_nodes.len();
        self.create("Takes")
            .create("Current")
            .create_property(FbxPropertyType::String)
            .push_string("");
    }
}

impl Default for FbxDocument {
    fn default() -> Self {
        Self::new()
    }
}
