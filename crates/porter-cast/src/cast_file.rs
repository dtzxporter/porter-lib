use std::io::Error;
use std::io::Write;

use porter_utils::StructWriteExt;

use crate::CastNode;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct CastHeader {
    magic: u32,
    version: u32,
    root_nodes: u32,
    flags: u32,
}

/// A cast file.
#[derive(Debug, Default)]
pub struct CastFile {
    root_nodes: Vec<CastNode>,
}

impl CastFile {
    /// Constructs a new cast file.
    pub fn new() -> Self {
        Self {
            root_nodes: Vec::new(),
        }
    }

    /// Appends a root node to the file.
    pub fn push(&mut self, node: CastNode) {
        self.root_nodes.push(node);
    }

    /// Serializes the cast file to the writer.
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let header = CastHeader {
            magic: 0x74736163,
            version: 1,
            root_nodes: self.root_nodes.len() as u32,
            flags: 0,
        };

        writer.write_struct(header)?;

        for root in &self.root_nodes {
            root.write(&mut writer)?;
        }

        Ok(())
    }
}
