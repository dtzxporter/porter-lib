use std::marker::PhantomData;
use std::slice::ChunksExact;
use std::slice::Iter;

use crate::CastPropertyValue;

enum CastPropertyBuffer<'a> {
    Bytes(ChunksExact<'a, u8>),
    Strings(Iter<'a, String>),
}

/// Iterator over the values in a cast property.
pub struct CastPropertyIter<'a, T: CastPropertyValue<'a>> {
    inner: CastPropertyBuffer<'a>,
    _phantom: PhantomData<T>,
}

impl<'a, T> CastPropertyIter<'a, T>
where
    T: CastPropertyValue<'a>,
{
    /// Constructs a new cast property iter for byte values.
    pub(crate) const fn from_bytes(bytes: ChunksExact<'a, u8>) -> Self {
        Self {
            inner: CastPropertyBuffer::Bytes(bytes),
            _phantom: PhantomData,
        }
    }

    /// Constructs a new cast property iter for string values.
    pub(crate) const fn from_strings(strings: Iter<'a, String>) -> Self {
        Self {
            inner: CastPropertyBuffer::Strings(strings),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for CastPropertyIter<'a, T>
where
    T: CastPropertyValue<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            CastPropertyBuffer::Bytes(bytes) => bytes.next().map(T::from_bytes),
            CastPropertyBuffer::Strings(strings) => strings
                .next()
                .map(|string| T::from_string(string.as_str())),
        }
    }
}
