use std::collections::TryReserveError;

use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

use porter_math::Quaternion;
use porter_math::Vector2;
use porter_math::Vector3;
use porter_math::Vector4;

use porter_utils::StringReadExt;
use porter_utils::StringWriteExt;
use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;
use porter_utils::VecExt;

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
    Vector4(Vector4),
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

    /// The name of this property.
    pub fn name(&self) -> &str {
        &self.property_name
    }

    /// Returns the values of this property as the given type.
    pub fn values<T>(&self) -> impl Iterator<Item = T> + '_
    where
        T: TryFrom<CastPropertyValue>,
    {
        self.property_values
            .iter()
            .cloned()
            .filter_map(|x| x.try_into().ok())
    }

    /// Clears the values in this property.
    pub fn clear(&mut self) {
        self.property_values.clear();
    }

    /// Serializes the property to the writer.
    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
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

    /// Deserializes a property from the given reader.
    pub(crate) fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let header: CastPropertyHeader = reader.read_struct()?;

        let name = reader.read_sized_string(header.name_size as usize, false)?;

        let mut values = Vec::try_with_exact_capacity(header.array_length as _)?;

        for _ in 0..header.array_length {
            match header.identifier {
                CastPropertyId::Byte => {
                    values.push(CastPropertyValue::Byte(reader.read_struct()?));
                }
                CastPropertyId::Short => {
                    values.push(CastPropertyValue::Short(reader.read_struct()?));
                }
                CastPropertyId::Integer32 => {
                    values.push(CastPropertyValue::Integer32(reader.read_struct()?));
                }
                CastPropertyId::Integer64 => {
                    values.push(CastPropertyValue::Integer64(reader.read_struct()?));
                }
                CastPropertyId::Float => {
                    values.push(CastPropertyValue::Float(reader.read_struct()?));
                }
                CastPropertyId::Double => {
                    values.push(CastPropertyValue::Double(reader.read_struct()?));
                }
                CastPropertyId::String => {
                    values.push(CastPropertyValue::String(
                        reader.read_null_terminated_string()?,
                    ));
                }
                CastPropertyId::Vector2 => {
                    values.push(CastPropertyValue::Vector2(reader.read_struct()?));
                }
                CastPropertyId::Vector3 => {
                    values.push(CastPropertyValue::Vector3(reader.read_struct()?));
                }
                CastPropertyId::Vector4 => {
                    values.push(CastPropertyValue::Vector4(reader.read_struct()?));
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Unknown cast property identifier!",
                    ));
                }
            }
        }

        Ok(Self {
            property_type: header.identifier,
            property_values: values,
            property_name: name,
        })
    }

    /// Gets the length of the cast property in bytes.
    pub(crate) fn length(&self) -> u32 {
        let mut result = size_of::<CastPropertyHeader>() as u32;

        result += self.property_name.len() as u32;

        match self.property_type {
            CastPropertyId::Byte => result += self.property_values.len() as u32,
            CastPropertyId::Short => {
                result += (size_of::<u16>() * self.property_values.len()) as u32
            }
            CastPropertyId::Integer32 => {
                result += (size_of::<u32>() * self.property_values.len()) as u32
            }
            CastPropertyId::Integer64 => {
                result += (size_of::<u64>() * self.property_values.len()) as u32
            }
            CastPropertyId::Float => {
                result += (size_of::<f32>() * self.property_values.len()) as u32
            }
            CastPropertyId::Double => {
                result += (size_of::<f64>() * self.property_values.len()) as u32
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
                result += (size_of::<Vector2>() * self.property_values.len()) as u32
            }
            CastPropertyId::Vector3 => {
                result += (size_of::<Vector3>() * self.property_values.len()) as u32
            }
            CastPropertyId::Vector4 => {
                result += (size_of::<Quaternion>() * self.property_values.len()) as u32
            }
            CastPropertyId::Unknown => {
                // Not in use.
            }
        }

        result
    }

    /// Tries to reserve capacity for at least `additional` more values to be inserted into the given `Property`.
    /// The property may reserve more space to speculatively avoid frequent reallocations.
    /// After calling `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if capacity is already sufficient. This method preserves the contents even if an error occurs.
    ///
    /// # Errors
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.property_values.try_reserve(additional)
    }

    /// Tries to reserve the minimum capacity for at least `additional` values to be inserted in the given `Property`.
    /// Unlike `try_reserve`, this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `try_reserve_exact`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests.
    /// Therefore, capacity can not be relied upon to be precisely minimal. Prefer `try_reserve` if future insertions are expected.
    ///
    /// # Errors
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.property_values.try_reserve_exact(additional)
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

impl From<bool> for CastPropertyValue {
    fn from(value: bool) -> Self {
        if value { Self::Byte(1) } else { Self::Byte(0) }
    }
}

impl From<u8> for CastPropertyValue {
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl TryFrom<CastPropertyValue> for u8 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Byte(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for u8!",
                ));
            }
        })
    }
}

impl From<u16> for CastPropertyValue {
    fn from(value: u16) -> Self {
        Self::Short(value)
    }
}

impl TryFrom<CastPropertyValue> for u16 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Byte(value) => value as u16,
            CastPropertyValue::Short(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for u16!",
                ));
            }
        })
    }
}

impl From<u32> for CastPropertyValue {
    fn from(value: u32) -> Self {
        Self::Integer32(value)
    }
}

impl TryFrom<CastPropertyValue> for u32 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Byte(value) => value as u32,
            CastPropertyValue::Short(value) => value as u32,
            CastPropertyValue::Integer32(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for u32!",
                ));
            }
        })
    }
}

impl From<u64> for CastPropertyValue {
    fn from(value: u64) -> Self {
        Self::Integer64(value)
    }
}

impl TryFrom<CastPropertyValue> for u64 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Byte(value) => value as u64,
            CastPropertyValue::Short(value) => value as u64,
            CastPropertyValue::Integer32(value) => value as u64,
            CastPropertyValue::Integer64(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for u64!",
                ));
            }
        })
    }
}

impl From<f32> for CastPropertyValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl TryFrom<CastPropertyValue> for f32 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Float(value) => value,
            CastPropertyValue::Double(value) => value as f32,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for f32!",
                ));
            }
        })
    }
}

impl From<f64> for CastPropertyValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl TryFrom<CastPropertyValue> for f64 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Float(value) => value as f64,
            CastPropertyValue::Double(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for f64!",
                ));
            }
        })
    }
}

impl From<String> for CastPropertyValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl TryFrom<CastPropertyValue> for String {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::String(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for String!",
                ));
            }
        })
    }
}

impl From<&str> for CastPropertyValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vector2> for CastPropertyValue {
    fn from(value: Vector2) -> Self {
        Self::Vector2(value)
    }
}

impl TryFrom<CastPropertyValue> for Vector2 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Vector2(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for Vector2!",
                ));
            }
        })
    }
}

impl From<Vector3> for CastPropertyValue {
    fn from(value: Vector3) -> Self {
        Self::Vector3(value)
    }
}

impl TryFrom<CastPropertyValue> for Vector3 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Vector3(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for Vector3!",
                ));
            }
        })
    }
}

impl From<Quaternion> for CastPropertyValue {
    fn from(value: Quaternion) -> Self {
        Self::Vector4(Vector4::from(value))
    }
}

impl TryFrom<CastPropertyValue> for Quaternion {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Vector4(value) => Quaternion::from(value),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for Quaternion!",
                ));
            }
        })
    }
}

impl From<Vector4> for CastPropertyValue {
    fn from(value: Vector4) -> Self {
        Self::Vector4(value)
    }
}

impl TryFrom<CastPropertyValue> for Vector4 {
    type Error = Error;

    #[inline]
    fn try_from(value: CastPropertyValue) -> Result<Self, Self::Error> {
        Ok(match value {
            CastPropertyValue::Vector4(value) => value,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid cast property value for Vector4!",
                ));
            }
        })
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
