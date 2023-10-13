use std::io::Error;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use porter_utils::AsByteSlice;

use crate::CastId;
use crate::CastProperty;
use crate::CastPropertyId;

/// Base hash constant used to generate hashes.
const HASH_BASE: u64 = 0x534E495752545250;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct CastNodeHeader {
    identifier: CastId,
    node_size: u32,
    node_hash: u64,
    property_count: u32,
    child_count: u32,
}

/// A cast node.
#[derive(Debug)]
pub struct CastNode {
    identifier: CastId,
    hash: u64,
    hash_next: Arc<AtomicU64>,
    properties: Vec<CastProperty>,
    children: Vec<Self>,
}

impl CastNode {
    /// Creates a new root node.
    pub fn root() -> Self {
        Self {
            identifier: CastId::Root,
            hash: HASH_BASE,
            hash_next: Arc::new(AtomicU64::new(HASH_BASE + 1)),
            properties: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Creates a new child node with the given identifier.
    pub fn create(&mut self, identifier: CastId) -> &mut Self {
        debug_assert!(!matches!(identifier, CastId::Root));

        let child = CastNode {
            identifier,
            hash: self.hash_next.fetch_add(1, Ordering::Relaxed),
            hash_next: self.hash_next.clone(),
            properties: Vec::new(),
            children: Vec::new(),
        };

        self.children.push(child);

        let index = self.children.len() - 1;

        self.children.get_mut(index).unwrap()
    }

    /// Creates a new property with the given type and name.
    pub fn create_property<N: Into<String>>(
        &mut self,
        property_type: CastPropertyId,
        name: N,
    ) -> &mut CastProperty {
        self.properties.push(CastProperty::new(property_type, name));

        let index = self.properties.len() - 1;

        self.properties.get_mut(index).unwrap()
    }

    /// Serializes the node to the writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let header = CastNodeHeader {
            identifier: self.identifier,
            node_size: self.length(),
            node_hash: self.hash,
            property_count: self.properties.len() as u32,
            child_count: self.children.len() as u32,
        };

        writer.write_all(header.as_byte_slice())?;

        for property in &self.properties {
            property.write(writer)?;
        }

        for child in &self.children {
            child.write(writer)?;
        }

        Ok(())
    }

    /// Gets the hash of this cast node.
    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    /// Gets the length in bytes of this cast node.
    fn length(&self) -> u32 {
        let mut result = std::mem::size_of::<CastNodeHeader>() as u32;

        for child in &self.children {
            result += child.length();
        }

        for property in &self.properties {
            result += property.length();
        }

        result
    }
}
