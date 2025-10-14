use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

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

    /// Returns the identifier of this node.
    pub fn identifier(&self) -> CastId {
        self.identifier
    }

    /// Finds a property by the given name.
    pub fn property<N: AsRef<str>>(&self, name: N) -> Option<&CastProperty> {
        self.properties.iter().find(|x| x.name() == name.as_ref())
    }

    /// Returns a slice of children of this node.
    pub fn children(&self) -> &[CastNode] {
        &self.children
    }

    /// Returns a mutable slice of children of this node.
    pub fn children_mut(&mut self) -> &mut [CastNode] {
        &mut self.children
    }

    /// Iterates over all children of the given type.
    pub fn children_of_type(&self, identifier: CastId) -> impl Iterator<Item = &CastNode> {
        self.children
            .iter()
            .filter(move |x| x.identifier == identifier)
    }

    /// Iterates over all mutable children of the given type.
    pub fn children_of_type_mut(
        &mut self,
        identifier: CastId,
    ) -> impl Iterator<Item = &mut CastNode> {
        self.children
            .iter_mut()
            .filter(move |x| x.identifier == identifier)
    }

    /// Finds a child by the given hash.
    pub fn child_by_hash(&self, hash: u64) -> Option<&CastNode> {
        self.children.iter().find(|x| x.hash == hash)
    }

    /// Finds a mutable child by the given hash.
    pub fn child_by_hash_mut(&mut self, hash: u64) -> Option<&mut CastNode> {
        self.children.iter_mut().find(|x| x.hash == hash)
    }

    /// Serializes the node to the writer.
    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let header = CastNodeHeader {
            identifier: self.identifier,
            node_size: self.length(),
            node_hash: self.hash,
            property_count: self.properties.len() as u32,
            child_count: self.children.len() as u32,
        };

        writer.write_struct(header)?;

        for property in &self.properties {
            property.write(writer)?;
        }

        for child in &self.children {
            child.write(writer)?;
        }

        Ok(())
    }

    /// Deserializes the node from the reader.
    pub(crate) fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let header: CastNodeHeader = reader.read_struct()?;

        let mut properties = Vec::new();

        properties.try_reserve_exact(header.property_count as _)?;

        for _ in 0..header.property_count {
            properties.push(CastProperty::read(reader)?);
        }

        let mut children = Vec::new();

        children.try_reserve_exact(header.child_count as _)?;

        for _ in 0..header.child_count {
            children.push(Self::read(reader)?);
        }

        Ok(Self {
            identifier: header.identifier,
            hash: header.node_hash,
            hash_next: Arc::new(AtomicU64::new(0)),
            properties,
            children,
        })
    }

    /// Gets the hash of this cast node.
    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    /// Gets the largest hash value of this cast node and it's children.
    pub(crate) fn largest_hash(&self) -> u64 {
        self.children
            .iter()
            .map(|x| x.hash)
            .max()
            .unwrap_or(0)
            .max(self.hash)
    }

    /// Sets a new hash next value for this cast node and it's children.
    pub(crate) fn set_hash_next(&mut self, hash_next: Arc<AtomicU64>) {
        for child in &mut self.children {
            child.set_hash_next(hash_next.clone());
        }

        self.hash_next = hash_next;
    }

    /// Gets the length in bytes of this cast node.
    fn length(&self) -> u32 {
        let mut result = size_of::<CastNodeHeader>() as u32;

        for child in &self.children {
            result += child.length();
        }

        for property in &self.properties {
            result += property.length();
        }

        result
    }
}
