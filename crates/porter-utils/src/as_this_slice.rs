/// A trait which converts a slice from one T: Copy to another O: Copy.
pub trait AsThisSlice<O> {
    type Output: Copy + 'static;

    /// Converts the given slice to another slice type.
    fn as_this_slice(&self) -> &[Self::Output];
}

impl<T, O> AsThisSlice<O> for &[T]
where
    T: Copy + 'static,
    O: Copy + 'static,
{
    type Output = O;

    fn as_this_slice(&self) -> &[Self::Output] {
        let pointer = self.as_ptr();
        let size_in_bytes = size_of_val(*self);
        let output_size = size_of::<Self::Output>();

        let size_in_elements = if output_size > size_in_bytes {
            0
        } else {
            size_in_bytes / output_size
        };

        // SAFETY: We only support slices who's T is Copy and calculate the correct size in elements for the new slice.
        unsafe { std::slice::from_raw_parts(pointer as *const Self::Output, size_in_elements) }
    }
}
