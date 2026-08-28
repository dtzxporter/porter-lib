/// An error that can occur during gdeflate compression / decompression.
#[derive(Debug, Clone)]
pub enum GDeflateError {
    DecompressFailed,
    CompressFailed,
}
