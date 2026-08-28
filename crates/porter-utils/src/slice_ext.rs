use std::collections::TryReserveError;
use std::slice::from_raw_parts;

use crate::VecExt;

pub trait SliceExt<T>
where
    T: Copy + 'static,
{
    /// Casts the given slice to a slice of another type.
    ///
    /// # Safety:
    /// - If the slice doesn't fit in the target slice, it's length is truncated to fit.
    fn as_this_slice<O>(&self) -> &[O]
    where
        O: Copy + 'static;
    /// Tries to copy `self` into a new `vec` and truncates the length if necessary.
    fn try_to_vec_truncated(&self, len: usize) -> Result<Vec<T>, TryReserveError>;
}

impl<T, U> SliceExt<U> for T
where
    T: AsRef<[U]>,
    U: Copy + 'static,
{
    #[inline]
    fn as_this_slice<O>(&self) -> &[O]
    where
        O: Copy + 'static,
    {
        let slice = self.as_ref();
        let ptr = slice.as_ptr();

        let size_per_element = size_of::<O>();
        let size_in_bytes = size_of_val(slice);

        let size_in_elements = if size_per_element > size_in_bytes {
            0
        } else {
            size_in_bytes / size_per_element
        };

        // SAFETY: We ensure that both T/O are Copy types and we have a correct size in elements.
        unsafe { from_raw_parts(ptr as *const O, size_in_elements) }
    }

    #[inline]
    fn try_to_vec_truncated(&self, len: usize) -> Result<Vec<U>, TryReserveError> {
        let slice = self.as_ref();
        let slice_len = slice.len().min(len);

        let mut result = Vec::try_with_exact_capacity(slice_len)?;

        // SAFETY: Buffers do not overlap and we ensure that at least slice_len capacity is available.
        unsafe {
            slice
                .as_ptr()
                .copy_to_nonoverlapping(result.as_mut_ptr(), slice_len)
        };

        // SAFETY: As long as copy_to_nonoverlapping upholds its contract, slice_len was initialized.
        unsafe { result.set_len(slice_len) };

        Ok(result)
    }
}
