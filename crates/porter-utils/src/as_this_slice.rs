/// A trait which converts a slice from one T: Copy to another O: Copy.
pub trait AsThisSlice<O> {
    type Output: Copy;

    /// Converts the given slice to another slice type.
    fn as_this_slice(&self) -> &[Self::Output];
}

impl<T, O> AsThisSlice<O> for &[T]
where
    T: Copy,
    O: Copy,
{
    type Output = O;

    fn as_this_slice(&self) -> &[Self::Output] {
        let pointer = self.as_ptr();
        let size_in_bytes = std::mem::size_of_val(*self);
        let output_size = std::mem::size_of::<Self::Output>();

        let size_in_elements = if output_size > size_in_bytes {
            0
        } else {
            size_in_bytes / output_size
        };

        unsafe { std::slice::from_raw_parts(pointer as *const Self::Output, size_in_elements) }
    }
}
