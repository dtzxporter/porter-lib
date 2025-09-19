use std::io::Error;
use std::io::Seek;
use std::io::Write;
use std::ops;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::FbxProperty;
use crate::FbxPropertyType;
use crate::FbxPropertyValue;

/// A node of a fbx document.
#[derive(Debug)]
pub struct FbxNode {
    name: String,
    properties: Vec<FbxProperty>,
    children: Vec<Self>,
    hash_next: Arc<AtomicU64>,
}

impl FbxNode {
    /// Constructs a new instance of fbx node.
    pub(crate) fn new<N: Into<String>>(name: N, hash_next: Arc<AtomicU64>) -> Self {
        Self {
            name: name.into(),
            properties: Vec::new(),
            children: Vec::new(),
            hash_next,
        }
    }

    /// Gets the hash of this node, or 0 when no hash value was found.
    pub(crate) fn hash(&self) -> u64 {
        if let Some(Some(FbxPropertyValue::Integer64(value))) =
            self.properties.first().map(|x| x.values().first())
        {
            *value
        } else {
            0
        }
    }

    /// Creates a new child node with the given name.
    pub fn create<N: Into<String>>(&mut self, name: N) -> &mut Self {
        self.children.push(Self::new(name, self.hash_next.clone()));

        let index = self.children.len() - 1;

        self.children.get_mut(index).unwrap()
    }

    /// Creates a new property with the given type.
    pub fn create_property(&mut self, property_type: FbxPropertyType) -> &mut FbxProperty {
        self.properties.push(FbxProperty::new(property_type));

        let index = self.properties.len() - 1;

        self.properties.get_mut(index).unwrap()
    }

    /// Creates a new hash property with the next available hash.
    pub fn create_hash(&mut self) {
        let hash = self.hash_next.fetch_add(1, Ordering::Relaxed);

        self.create_property(FbxPropertyType::Integer64).push(hash);
    }

    /// Serializes the node to the writer.
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), Error> {
        const HEADER_SIZE: usize =
            size_of::<u32>() + size_of::<u32>() + size_of::<u32>() + size_of::<u8>();

        if self.name.is_empty() && self.children.is_empty() && self.properties.is_empty() {
            writer.write_all(&[0; HEADER_SIZE])?;
            return Ok(());
        }

        let mut property_list_length = 0;
        let mut node_length = HEADER_SIZE as u32 + self.name.len() as u32;

        for property in &self.properties {
            property_list_length += property.length();
        }

        node_length += property_list_length;

        for child in &self.children {
            node_length += child.length();
        }

        let next_node = writer.stream_position()? as u32 + node_length;

        writer.write_all(&next_node.to_le_bytes())?;
        writer.write_all(&(self.properties.len() as u32).to_le_bytes())?;
        writer.write_all(&property_list_length.to_le_bytes())?;
        writer.write_all(&(self.name.len() as u8).to_le_bytes())?;
        writer.write_all(self.name.as_bytes())?;

        for property in &self.properties {
            property.write(writer)?;
        }

        for child in &self.children {
            child.write(writer)?;
        }

        Ok(())
    }

    /// Gets the length of this node in bytes.
    pub(crate) fn length(&self) -> u32 {
        let mut result = size_of::<u32>() as u32
            + size_of::<u32>() as u32
            + size_of::<u32>() as u32
            + size_of::<u8>() as u32
            + self.name.len() as u32;

        for child in &self.children {
            result += child.length();
        }

        for property in &self.properties {
            result += property.length();
        }

        result
    }

    /// Prepares the node for serialization, which adds an empty node after larger ones.
    pub(crate) fn prepare(&mut self) {
        for child in &mut self.children {
            child.prepare();
        }

        if !self.children.is_empty() || (self.properties.is_empty() && !self.name.is_empty()) {
            self.children
                .push(FbxNode::new(String::new(), self.hash_next.clone()));
        }
    }
}

impl ops::Index<usize> for FbxNode {
    type Output = FbxNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

impl ops::IndexMut<usize> for FbxNode {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.children[index]
    }
}
