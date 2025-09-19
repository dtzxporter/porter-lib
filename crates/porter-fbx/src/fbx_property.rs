use std::io::Error;
use std::io::Write;

use crate::FbxNode;

/// The type id of an fbx property.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FbxPropertyType {
    Byte = b'B',
    Bool = b'C',
    Integer16 = b'Y',
    Integer32 = b'I',
    Integer64 = b'L',
    Float32 = b'F',
    Float64 = b'D',
    Raw = b'R',
    String = b'S',
    ByteArray = b'b',
    BoolArray = b'c',
    Integer16Array = b'y',
    Integer32Array = b'i',
    Integer64Array = b'l',
    Float32Array = b'f',
    Float64Array = b'd',
}

/// Container that holds the value for a fbx property value.
#[derive(Debug, Clone, Copy)]
pub enum FbxPropertyValue {
    Byte(u8),
    Boolean(bool),
    Integer16(u16),
    Integer32(u32),
    Integer64(u64),
    Float32(f32),
    Float64(f64),
}

/// Container that holds a fbx property string-like value.
#[derive(Debug, Clone)]
pub enum FbxPropertyString {
    None,
    String(String),
    Buffer(Vec<u8>),
}

impl FbxPropertyString {
    /// Returns the length in bytes of the property string.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::String(string) => string.len(),
            Self::Buffer(buffer) => buffer.len(),
        }
    }

    /// Returns true if the property string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A fbx property of a node.
#[derive(Debug)]
pub struct FbxProperty {
    property_type: FbxPropertyType,
    property_values: Vec<FbxPropertyValue>,
    property_string: FbxPropertyString,
}

impl FbxProperty {
    /// Constructs a new instance of fbx property.
    pub(crate) fn new(property_type: FbxPropertyType) -> Self {
        Self {
            property_type,
            property_values: Vec::new(),
            property_string: FbxPropertyString::None,
        }
    }

    /// Gets the values of this property.
    pub(crate) fn values(&self) -> &[FbxPropertyValue] {
        &self.property_values
    }

    /// Appends an element to the property values collection.
    pub fn push<T: Into<FbxPropertyValue>>(&mut self, value: T) -> &mut Self {
        let value = value.into();

        debug_assert!(self.property_type == value);

        self.property_values.push(value);
        self
    }

    /// Appends a string to the property.
    pub fn push_string<S: Into<String>>(&mut self, value: S) -> &mut Self {
        self.property_string = FbxPropertyString::String(value.into());
        self
    }

    /// Appends a binary array to the property.
    pub fn push_raw<B: AsRef<[u8]>>(&mut self, value: B) -> &mut Self {
        self.property_string = FbxPropertyString::Buffer(value.as_ref().to_vec());
        self
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[self.property_type as u8])?;

        let array_size: Option<u32> = match self.property_type {
            FbxPropertyType::BoolArray => Some(size_of::<bool>() as u32),
            FbxPropertyType::ByteArray => Some(size_of::<u8>() as u32),
            FbxPropertyType::Float32Array => Some(size_of::<f32>() as u32),
            FbxPropertyType::Float64Array => Some(size_of::<f64>() as u32),
            FbxPropertyType::Integer16Array => Some(size_of::<u16>() as u32),
            FbxPropertyType::Integer32Array => Some(size_of::<u32>() as u32),
            FbxPropertyType::Integer64Array => Some(size_of::<u64>() as u32),
            _ => None,
        };

        if let Some(array_size) = array_size {
            let array_length = self.property_values.len() as u32;
            let uncompressed_length = array_length * array_size;

            writer.write_all(&array_length.to_le_bytes())?;
            writer.write_all(&0u32.to_le_bytes())?;
            writer.write_all(&uncompressed_length.to_le_bytes())?;
        }

        for property_value in &self.property_values {
            match property_value {
                FbxPropertyValue::Boolean(bool) => {
                    writer.write_all(&(*bool as u8).to_le_bytes())?;
                }
                FbxPropertyValue::Byte(byte) => {
                    writer.write_all(&byte.to_le_bytes())?;
                }
                FbxPropertyValue::Integer16(integer16) => {
                    writer.write_all(&integer16.to_le_bytes())?;
                }
                FbxPropertyValue::Integer32(integer32) => {
                    writer.write_all(&integer32.to_le_bytes())?;
                }
                FbxPropertyValue::Integer64(integer64) => {
                    writer.write_all(&integer64.to_le_bytes())?;
                }
                FbxPropertyValue::Float32(float32) => {
                    writer.write_all(&float32.to_le_bytes())?;
                }
                FbxPropertyValue::Float64(float64) => {
                    writer.write_all(&float64.to_le_bytes())?;
                }
            }
        }

        match &self.property_string {
            FbxPropertyString::None => {
                // No string.
            }
            FbxPropertyString::String(string) => {
                writer.write_all(&(string.len() as u32).to_le_bytes())?;
                writer.write_all(string.as_bytes())?;
            }
            FbxPropertyString::Buffer(buffer) => {
                writer.write_all(&(buffer.len() as u32).to_le_bytes())?;
                writer.write_all(buffer.as_slice())?;
            }
        }

        Ok(())
    }

    /// Gets the length of this property in bytes.
    pub(crate) fn length(&self) -> u32 {
        let mut result = size_of::<u8>() as u32;

        const SIZE_OF_ARRAY: u32 =
            size_of::<u32>() as u32 + size_of::<u32>() as u32 + size_of::<u32>() as u32;

        match self.property_type {
            FbxPropertyType::Byte => result += size_of::<u8>() as u32,
            FbxPropertyType::Bool => result += size_of::<bool>() as u32,
            FbxPropertyType::Integer16 => result += size_of::<u16>() as u32,
            FbxPropertyType::Integer32 => result += size_of::<u32>() as u32,
            FbxPropertyType::Integer64 => result += size_of::<u64>() as u32,
            FbxPropertyType::Float32 => result += size_of::<f32>() as u32,
            FbxPropertyType::Float64 => result += size_of::<f64>() as u32,
            FbxPropertyType::String | FbxPropertyType::Raw => {
                result += self.property_string.len() as u32 + size_of::<u32>() as u32
            }
            FbxPropertyType::ByteArray => {
                result += self.property_values.len() as u32 * size_of::<u8>() as u32;
                result += SIZE_OF_ARRAY;
            }
            FbxPropertyType::BoolArray => {
                result += self.property_values.len() as u32 * size_of::<bool>() as u32;
                result += SIZE_OF_ARRAY;
            }
            FbxPropertyType::Integer16Array => {
                result += self.property_values.len() as u32 * size_of::<u16>() as u32;
                result += SIZE_OF_ARRAY;
            }
            FbxPropertyType::Integer32Array => {
                result += self.property_values.len() as u32 * size_of::<u32>() as u32;
                result += SIZE_OF_ARRAY;
            }
            FbxPropertyType::Integer64Array => {
                result += self.property_values.len() as u32 * size_of::<u64>() as u32;
                result += SIZE_OF_ARRAY;
            }
            FbxPropertyType::Float32Array => {
                result += self.property_values.len() as u32 * size_of::<f32>() as u32;
                result += SIZE_OF_ARRAY;
            }
            FbxPropertyType::Float64Array => {
                result += self.property_values.len() as u32 * size_of::<f64>() as u32;
                result += SIZE_OF_ARRAY;
            }
        }

        result
    }
}

impl PartialEq<FbxPropertyValue> for FbxPropertyType {
    fn eq(&self, other: &FbxPropertyValue) -> bool {
        match other {
            FbxPropertyValue::Byte(_) => {
                matches!(self, FbxPropertyType::Byte | FbxPropertyType::ByteArray)
            }
            FbxPropertyValue::Boolean(_) => {
                matches!(self, FbxPropertyType::Bool | FbxPropertyType::BoolArray)
            }
            FbxPropertyValue::Float32(_) => {
                matches!(
                    self,
                    FbxPropertyType::Float32 | FbxPropertyType::Float32Array
                )
            }
            FbxPropertyValue::Float64(_) => matches!(
                self,
                FbxPropertyType::Float64 | FbxPropertyType::Float64Array
            ),
            FbxPropertyValue::Integer16(_) => matches!(
                self,
                FbxPropertyType::Integer16 | FbxPropertyType::Integer16Array
            ),
            FbxPropertyValue::Integer32(_) => matches!(
                self,
                FbxPropertyType::Integer32 | FbxPropertyType::Integer32Array
            ),
            FbxPropertyValue::Integer64(_) => matches!(
                self,
                FbxPropertyType::Integer64 | FbxPropertyType::Integer64Array
            ),
        }
    }
}

impl From<bool> for FbxPropertyValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<u8> for FbxPropertyValue {
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl From<u16> for FbxPropertyValue {
    fn from(value: u16) -> Self {
        Self::Integer16(value)
    }
}

impl From<u32> for FbxPropertyValue {
    fn from(value: u32) -> Self {
        Self::Integer32(value)
    }
}

impl From<u64> for FbxPropertyValue {
    fn from(value: u64) -> Self {
        Self::Integer64(value)
    }
}

impl From<f32> for FbxPropertyValue {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for FbxPropertyValue {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<&mut FbxNode> for FbxPropertyValue {
    fn from(value: &mut FbxNode) -> Self {
        Self::Integer64(value.hash())
    }
}

impl From<&FbxNode> for FbxPropertyValue {
    fn from(value: &FbxNode) -> Self {
        Self::Integer64(value.hash())
    }
}
