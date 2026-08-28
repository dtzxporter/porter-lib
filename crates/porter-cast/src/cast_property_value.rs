use porter_math::Quaternion;
use porter_math::Vector2;
use porter_math::Vector3;
use porter_math::Vector4;

use crate::CastPropertyId;

/// Prevent anyone downstream from implementing cast property value.
mod sealed {
    pub(crate) trait Private {}
}

/// Support for conversion two/from cast property values and their base types.
#[allow(private_bounds)]
pub trait CastPropertyValue<'a>: sealed::Private {
    /// The cast property that this type belongs to.
    const PROPERTY_ID: CastPropertyId;

    /// Converts a sequence of bytes to this value type, the bytes are always in little endian order.
    fn from_bytes(bytes: &[u8]) -> Self;
    /// Converts a string to this value type.
    fn from_string(string: &'a str) -> Self;
    /// Converts this type to a sequence of bytes in little endian order.
    fn to_bytes(&self, bytes: &mut [u8]);
    /// Converts this type to a string.
    fn to_string(self) -> String;
}

/// Utility to implement cast value trait on byte types.
macro_rules! impl_byte_value {
    ($ty:ty, $property:expr) => {
        impl sealed::Private for $ty {}

        impl CastPropertyValue<'_> for $ty {
            const PROPERTY_ID: CastPropertyId = $property;

            #[inline(always)]
            fn from_bytes(bytes: &[u8]) -> Self {
                <$ty>::from_ne_bytes(bytes.try_into().unwrap())
            }

            #[inline(always)]
            fn from_string(_: &str) -> Self {
                Default::default()
            }

            #[inline(always)]
            fn to_bytes(&self, bytes: &mut [u8]) {
                bytes[..{ Self::PROPERTY_ID.stride() }].copy_from_slice(&self.to_le_bytes());
            }

            #[inline(always)]
            fn to_string(self) -> String {
                String::new()
            }
        }
    };
}

impl_byte_value!(u8, CastPropertyId::Byte);
impl_byte_value!(u16, CastPropertyId::Short);
impl_byte_value!(u32, CastPropertyId::Integer32);
impl_byte_value!(u64, CastPropertyId::Integer64);
impl_byte_value!(f32, CastPropertyId::Float);
impl_byte_value!(f64, CastPropertyId::Double);
impl_byte_value!(Vector2, CastPropertyId::Vector2);
impl_byte_value!(Vector3, CastPropertyId::Vector3);
impl_byte_value!(Vector4, CastPropertyId::Vector4);
impl_byte_value!(Quaternion, CastPropertyId::Vector4);

impl sealed::Private for bool {}
impl sealed::Private for String {}
impl sealed::Private for &str {}

impl CastPropertyValue<'_> for bool {
    const PROPERTY_ID: CastPropertyId = CastPropertyId::Byte;

    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0] > 0
    }

    #[inline(always)]
    fn from_string(_: &str) -> Self {
        Default::default()
    }

    #[inline(always)]
    fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = *self as _;
    }

    #[inline(always)]
    fn to_string(self) -> String {
        String::new()
    }
}

impl CastPropertyValue<'_> for String {
    const PROPERTY_ID: CastPropertyId = CastPropertyId::String;

    #[inline(always)]
    fn from_bytes(_: &[u8]) -> Self {
        Self::new()
    }

    #[inline(always)]
    fn from_string(string: &str) -> Self {
        string.to_owned()
    }

    #[inline(always)]
    fn to_bytes(&self, _: &mut [u8]) {}

    #[inline(always)]
    fn to_string(self) -> String {
        self
    }
}

impl<'a> CastPropertyValue<'a> for &'a str {
    const PROPERTY_ID: CastPropertyId = CastPropertyId::String;

    #[inline(always)]
    fn from_bytes(_: &[u8]) -> Self {
        ""
    }

    #[inline(always)]
    fn to_bytes(&self, _: &mut [u8]) {}

    #[inline(always)]
    fn from_string(string: &'a str) -> Self {
        string
    }

    #[inline(always)]
    fn to_string(self) -> String {
        self.to_owned()
    }
}
