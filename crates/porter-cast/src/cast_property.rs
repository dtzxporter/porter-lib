use std::io::Error;
use std::io::Write;

use porter_math::Quaternion;
use porter_math::Vector2;
use porter_math::Vector3;

use porter_utils::StringWriteExt;
use porter_utils::StructWriteExt;

use crate::CastNode;
use crate::CastPropertyId;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct CastPropertyHeader {
    identifier: CastPropertyId,
    name_size: u16,
    array_length: u32,
}

/// Container that holds the value for a cast property value.
#[derive(Debug, Clone)]
pub enum CastPropertyValue {
    Byte(u8),
    Short(u16),
    Integer32(u32),
    Integer64(u64),
    Float(f32),
    Double(f64),
    String(String),
    Vector2(Vector2),
    Vector3(Vector3),
    Vector4(Quaternion),
}

/// A cast property of a node.
#[derive(Debug)]
pub struct CastProperty {
    property_type: CastPropertyId,
    property_values: Vec<CastPropertyValue>,
    property_name: String,
}

impl CastProperty {
    /// Constructs a new instance of cast property.
    pub fn new<N: Into<String>>(property_type: CastPropertyId, name: N) -> Self {
        Self {
            property_type,
            property_values: Vec::new(),
            property_name: name.into().to_lowercase(),
        }
    }

    /// Appends an element to the property values collection.
    pub fn push<T: Into<CastPropertyValue>>(&mut self, value: T) -> &mut Self {
        let value = value.into();

        debug_assert!(self.property_type == value);

        self.property_values.push(value);
        self
    }

    /// Serializes the property to the writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let header = CastPropertyHeader {
            identifier: self.property_type,
            name_size: self.property_name.len() as u16,
            array_length: self.property_values.len() as u32,
        };

        writer.write_struct(header)?;
        writer.write_all(self.property_name.as_bytes())?;

        for property_value in &self.property_values {
            match property_value {
                CastPropertyValue::Byte(byte) => {
                    writer.write_all(&byte.to_le_bytes())?;
                }
                CastPropertyValue::Short(short) => {
                    writer.write_all(&short.to_le_bytes())?;
                }
                CastPropertyValue::Integer32(integer32) => {
                    writer.write_all(&integer32.to_le_bytes())?;
                }
                CastPropertyValue::Integer64(integer64) => {
                    writer.write_all(&integer64.to_le_bytes())?;
                }
                CastPropertyValue::Float(float) => {
                    writer.write_all(&float.to_le_bytes())?;
                }
                CastPropertyValue::Double(double) => {
                    writer.write_all(&double.to_le_bytes())?;
                }
                CastPropertyValue::String(string) => {
                    writer.write_null_terminated_string(string)?;
                }
                CastPropertyValue::Vector2(vector2) => {
                    writer.write_struct(*vector2)?;
                }
                CastPropertyValue::Vector3(vector3) => {
                    writer.write_struct(*vector3)?;
                }
                CastPropertyValue::Vector4(vector4) => {
                    writer.write_struct(*vector4)?;
                }
            }
        }

        Ok(())
    }

    /// Gets the length of the cast property in bytes.
    pub(crate) fn length(&self) -> u32 {
        let mut result = std::mem::size_of::<CastPropertyHeader>() as u32;

        result += self.property_name.len() as u32;

        match self.property_type {
            CastPropertyId::Byte => result += self.property_values.len() as u32,
            CastPropertyId::Short => {
                result += (std::mem::size_of::<u16>() * self.property_values.len()) as u32
            }
            CastPropertyId::Integer32 => {
                result += (std::mem::size_of::<u32>() * self.property_values.len()) as u32
            }
            CastPropertyId::Integer64 => {
                result += (std::mem::size_of::<u64>() * self.property_values.len()) as u32
            }
            CastPropertyId::Float => {
                result += (std::mem::size_of::<f32>() * self.property_values.len()) as u32
            }
            CastPropertyId::Double => {
                result += (std::mem::size_of::<f64>() * self.property_values.len()) as u32
            }
            CastPropertyId::String => {
                result += self
                    .property_values
                    .iter()
                    .filter_map(|x| match x {
                        CastPropertyValue::String(v) => Some(v),
                        _ => None,
                    })
                    .map(|x| x.len() + 1)
                    .sum::<usize>() as u32;
            }
            CastPropertyId::Vector2 => {
                result += (std::mem::size_of::<Vector2>() * self.property_values.len()) as u32
            }
            CastPropertyId::Vector3 => {
                result += (std::mem::size_of::<Vector3>() * self.property_values.len()) as u32
            }
            CastPropertyId::Vector4 => {
                result += (std::mem::size_of::<Quaternion>() * self.property_values.len()) as u32
            }
        }

        result
    }
}

impl PartialEq<CastPropertyValue> for CastPropertyId {
    fn eq(&self, other: &CastPropertyValue) -> bool {
        match other {
            CastPropertyValue::Byte(_) => matches!(self, CastPropertyId::Byte),
            CastPropertyValue::Short(_) => matches!(self, CastPropertyId::Short),
            CastPropertyValue::Integer32(_) => matches!(self, CastPropertyId::Integer32),
            CastPropertyValue::Integer64(_) => matches!(self, CastPropertyId::Integer64),
            CastPropertyValue::Float(_) => matches!(self, CastPropertyId::Float),
            CastPropertyValue::Double(_) => matches!(self, CastPropertyId::Double),
            CastPropertyValue::String(_) => matches!(self, CastPropertyId::String),
            CastPropertyValue::Vector2(_) => matches!(self, CastPropertyId::Vector2),
            CastPropertyValue::Vector3(_) => matches!(self, CastPropertyId::Vector3),
            CastPropertyValue::Vector4(_) => matches!(self, CastPropertyId::Vector4),
        }
    }
}

impl From<u8> for CastPropertyValue {
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl From<u16> for CastPropertyValue {
    fn from(value: u16) -> Self {
        Self::Short(value)
    }
}

impl From<u32> for CastPropertyValue {
    fn from(value: u32) -> Self {
        Self::Integer32(value)
    }
}

impl From<u64> for CastPropertyValue {
    fn from(value: u64) -> Self {
        Self::Integer64(value)
    }
}

impl From<f32> for CastPropertyValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for CastPropertyValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<String> for CastPropertyValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for CastPropertyValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Vector2> for CastPropertyValue {
    fn from(value: Vector2) -> Self {
        Self::Vector2(value)
    }
}

impl From<Vector3> for CastPropertyValue {
    fn from(value: Vector3) -> Self {
        Self::Vector3(value)
    }
}

impl From<Quaternion> for CastPropertyValue {
    fn from(value: Quaternion) -> Self {
        Self::Vector4(value)
    }
}

impl From<&mut CastNode> for CastPropertyValue {
    fn from(value: &mut CastNode) -> Self {
        Self::Integer64(value.hash())
    }
}

impl From<&CastNode> for CastPropertyValue {
    fn from(value: &CastNode) -> Self {
        Self::Integer64(value.hash())
    }
}
