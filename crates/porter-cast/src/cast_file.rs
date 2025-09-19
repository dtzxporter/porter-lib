use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use porter_utils::StructReadExt;
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

    /// Returns a slice of roots in the file.
    pub fn roots(&self) -> &[CastNode] {
        &self.root_nodes
    }

    /// Returns a mutable slice of roots in the file.
    pub fn roots_mut(&mut self) -> &mut [CastNode] {
        &mut self.root_nodes
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

    /// Deserializes a cast file from the reader.
    pub fn read<R: Read>(mut reader: R) -> Result<Self, Error> {
        let header: CastHeader = reader.read_struct()?;

        if header.magic != 0x74736163 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid cast file magic!",
            ));
        }

        let mut root_nodes = Vec::new();

        root_nodes
            .try_reserve_exact(header.root_nodes as usize)
            .map_err(|x| Error::new(ErrorKind::OutOfMemory, x))?;

        for _ in 0..header.root_nodes {
            root_nodes.push(CastNode::read(&mut reader)?);
        }

        let mut largest_hash_next: u64 = 0;

        for root in &root_nodes {
            largest_hash_next = largest_hash_next.max(root.largest_hash());
        }

        let hash_next = Arc::new(AtomicU64::new(largest_hash_next.wrapping_add(1)));

        for root in &mut root_nodes {
            root.set_hash_next(hash_next.clone());
        }

        Ok(Self { root_nodes })
    }
}
