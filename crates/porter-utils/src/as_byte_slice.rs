/// Trait that allows casting any `Copy` type to a slice of bytes.
pub trait AsByteSlice: Copy {
    /// Creates a slice of bytes that represents this data.
    fn as_byte_slice(&self) -> &[u8];
}

impl<T> AsByteSlice for T
where
    T: Copy,
{
    fn as_byte_slice(&self) -> &[u8] {
        let data_ptr = self as *const T as *const u8;

        unsafe { std::slice::from_raw_parts(data_ptr, std::mem::size_of::<T>()) }
    }
}
