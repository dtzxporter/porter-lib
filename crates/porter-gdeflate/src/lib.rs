mod error;

pub use error::*;

use gdeflate_sys::*;

/// Decompresses one or more gdeflate compressed pages into the decompressed buffer. Returns the decompressed size written when successful.
pub fn gdeflate_decompress<C: AsRef<[Vec<u8>]>, D: AsMut<[u8]>>(
    compressed_pages: C,
    mut decompressed_buffer: D,
) -> Result<usize, GDeflateError> {
    let compressed_pages = compressed_pages.as_ref();
    let decompressed_buffer = decompressed_buffer.as_mut();

    // SAFETY: We check if the returned pointer is null below, and exit early.
    let decompressor = unsafe { libdeflate_alloc_gdeflate_decompressor() };

    if decompressor.is_null() {
        return Err(GDeflateError::DecompressFailed);
    }

    let mut pages: Vec<_> = compressed_pages
        .iter()
        .map(|x| libdeflate_gdeflate_in_page {
            data: x.as_ptr() as _,
            nbytes: x.len(),
        })
        .collect();

    let mut size: usize = 0;

    // SAFETY:
    // Both pointers must be valid because they came from slices and their sizes are fixed.
    // The decompressor was checked above to make sure it allocated properly.
    let result = unsafe {
        libdeflate_gdeflate_decompress(
            decompressor,
            pages.as_mut_ptr(),
            pages.len(),
            decompressed_buffer.as_mut_ptr() as _,
            decompressed_buffer.len(),
            &mut size,
        )
    };

    // SAFETY: We can only get here if the decompressor was not null.
    unsafe { libdeflate_free_gdeflate_decompressor(decompressor) };

    if result != libdeflate_result_LIBDEFLATE_SUCCESS {
        return Err(GDeflateError::DecompressFailed);
    }

    Ok(size)
}
