use std::collections::TryReserveError;

use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

use porter_macros::assert_size;

use porter_utils::StringReadExt;
use porter_utils::StringWriteExt;
use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;
use porter_utils::VecExt;
use porter_utils::VecReadExt;

use crate::CastPropertyId;
use crate::CastPropertyIter;
use crate::CastPropertyValue;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CastPropertyHeader {
    identifier: CastPropertyId,
    name_size: u16,
    array_length: u32,
}

assert_size!(CastPropertyHeader, 8);

#[derive(Debug)]
enum CastPropertyValues {
    ByteBuffer(Vec<u8>),
    StringBuffer(Vec<String>),
}

/// A cast property of a node.
#[derive(Debug)]
pub struct CastProperty {
    property_type: CastPropertyId,
    property_values: CastPropertyValues,
    property_name: String,
}

impl CastProperty {
    /// Constructs a new instance of cast property.
    pub fn new<N: AsRef<str>>(property_type: CastPropertyId, name: N) -> Self {
        Self {
            property_type,
            property_values: if matches!(property_type, CastPropertyId::String) {
                CastPropertyValues::StringBuffer(Vec::new())
            } else {
                CastPropertyValues::ByteBuffer(Vec::new())
            },
            property_name: name.as_ref().to_lowercase(),
        }
    }

    /// Appends an element to the property values collection.
    pub fn push<'a, T: CastPropertyValue<'a>>(&mut self, value: T) {
        debug_assert!(self.property_type == T::PROPERTY_ID);

        match T::PROPERTY_ID {
            CastPropertyId::Byte
            | CastPropertyId::Short
            | CastPropertyId::Integer32
            | CastPropertyId::Integer64
            | CastPropertyId::Float
            | CastPropertyId::Double
            | CastPropertyId::Vector2
            | CastPropertyId::Vector3
            | CastPropertyId::Vector4 => {
                let mut bytes: [u8; CastPropertyId::MAXIMUM_STRIDE] =
                    [0; CastPropertyId::MAXIMUM_STRIDE];

                value.to_bytes(&mut bytes);

                if let CastPropertyValues::ByteBuffer(buffer) = &mut self.property_values {
                    buffer.extend(&bytes[..{ T::PROPERTY_ID.stride() }]);
                }
            }
            CastPropertyId::String => {
                if let CastPropertyValues::StringBuffer(buffer) = &mut self.property_values {
                    buffer.push(value.to_string());
                }
            }
            CastPropertyId::Unknown => {
                // Unreachable, used for reading only.
            }
        }
    }

    /// The name of this property.
    pub fn name(&self) -> &str {
        &self.property_name
    }

    /// Returns the values of this property as the given type.
    pub fn values<'a, T>(&'a self) -> CastPropertyIter<'a, T>
    where
        T: CastPropertyValue<'a>,
    {
        debug_assert!(self.property_type == T::PROPERTY_ID);

        match &self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => {
                CastPropertyIter::from_bytes(buffer.chunks_exact(T::PROPERTY_ID.stride()))
            }
            CastPropertyValues::StringBuffer(buffer) => {
                CastPropertyIter::from_strings(buffer.iter())
            }
        }
    }

    /// Removes all values in this property.
    pub fn clear(&mut self) {
        match &mut self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => buffer.clear(),
            CastPropertyValues::StringBuffer(buffer) => buffer.clear(),
        }
    }

    /// Tries to reserve capacity for at least `additional` more values to be inserted into the given `Property`.
    /// The property may reserve more space to speculatively avoid frequent reallocations.
    /// After calling `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if capacity is already sufficient. This method preserves the contents even if an error occurs.
    ///
    /// # Errors
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned.
    pub fn try_reserve(&mut self, additional: usize) -> Result<&mut Self, TryReserveError> {
        match &mut self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => {
                // Buffer needs to reserve the size in bytes.
                buffer.try_reserve(self.property_type.stride() * additional)?;
            }
            CastPropertyValues::StringBuffer(buffer) => {
                // Buffer is already in string format.
                buffer.try_reserve(additional)?;
            }
        }

        Ok(self)
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
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<&mut Self, TryReserveError> {
        match &mut self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => {
                // Buffer needs to reserve the size in bytes.
                buffer.try_reserve_exact(self.property_type.stride() * additional)?
            }
            CastPropertyValues::StringBuffer(buffer) => {
                // Buffer is already in string format.
                buffer.try_reserve_exact(additional)?
            }
        }

        Ok(self)
    }

    /// Serializes the property to the writer.
    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let array_length = match &self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => {
                // Buffer needs conversion from bytes to elements.
                buffer.len() / self.property_type.stride()
            }
            CastPropertyValues::StringBuffer(buffer) => {
                // Buffer is already in string format.
                buffer.len()
            }
        };

        let header = CastPropertyHeader {
            identifier: self.property_type,
            name_size: self.property_name.len() as u16,
            array_length: array_length as u32,
        };

        writer.write_struct(header)?;
        writer.write_all(self.property_name.as_bytes())?;

        match &self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => {
                writer.write_all(buffer)?;
            }
            CastPropertyValues::StringBuffer(buffer) => {
                for string in buffer {
                    writer.write_null_terminated_string(string)?;
                }
            }
        }

        Ok(())
    }

    /// Deserializes a property from the given reader.
    pub(crate) fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let header: CastPropertyHeader = reader.read_struct()?;

        let name = reader.read_sized_string(header.name_size as usize, false)?;

        let values = match header.identifier {
            CastPropertyId::Byte
            | CastPropertyId::Short
            | CastPropertyId::Integer32
            | CastPropertyId::Integer64
            | CastPropertyId::Float
            | CastPropertyId::Double
            | CastPropertyId::Vector2
            | CastPropertyId::Vector3
            | CastPropertyId::Vector4 => {
                let buffer: Vec<u8> =
                    reader.read_vec(header.identifier.stride() * header.array_length as usize)?;

                CastPropertyValues::ByteBuffer(buffer)
            }
            CastPropertyId::String => {
                let mut strings: Vec<String> =
                    Vec::try_with_exact_capacity(header.array_length as _)?;

                for _ in 0..header.array_length {
                    strings.push(reader.read_null_terminated_string()?);
                }

                CastPropertyValues::StringBuffer(strings)
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unknown cast property identifier!",
                ));
            }
        };

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

        match &self.property_values {
            CastPropertyValues::ByteBuffer(buffer) => {
                result += buffer.len() as u32;
            }
            CastPropertyValues::StringBuffer(buffer) => {
                result += buffer
                    .iter()
                    .map(|string| string.len() + 1)
                    // Size of all strings with a null terminating character.
                    .sum::<usize>() as u32;
            }
        }

        result
    }
}
