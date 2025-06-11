/// Trait that allows casting any `Copy` type to a slice of bytes.
pub trait AsByteSlice: Copy + 'static {
    /// Creates a slice of bytes that represents this data.
    fn as_byte_slice(&self) -> &[u8];
}

impl<T> AsByteSlice for T
where
    T: Copy + 'static,
{
    fn as_byte_slice(&self) -> &[u8] {
        let data_ptr = self as *const T as *const u8;

        // SAFETY: We only allow structs that implement Copy and validate the size is no more than the `size_of` T.
        unsafe { std::slice::from_raw_parts(data_ptr, size_of::<T>()) }
    }
}
