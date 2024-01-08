use xxhash_rust::xxh3::xxh3_64;

/// Utility to hash data with xxh64 algo.
pub trait HashXXH64 {
    /// Creates a xxhash64 checksum for this data.
    fn hash_xxh64(&self) -> u64;
}

impl HashXXH64 for &str {
    fn hash_xxh64(&self) -> u64 {
        xxh3_64(self.as_bytes())
    }
}

impl HashXXH64 for String {
    fn hash_xxh64(&self) -> u64 {
        xxh3_64(self.as_bytes())
    }
}

impl HashXXH64 for &[u8] {
    fn hash_xxh64(&self) -> u64 {
        xxh3_64(self)
    }
}
