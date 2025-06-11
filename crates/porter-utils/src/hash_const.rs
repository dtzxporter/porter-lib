use xxhash_rust::const_xxh3::xxh3_64;

/// Utility to hash data at compile time with different algos.
pub struct HashConst;

impl HashConst {
    /// Creates a compile time xxhash64 checksum for this str.
    pub const fn hash_xxh64_str(str: &str) -> u64 {
        xxh3_64(str.as_bytes())
    }

    /// Creates a compile time xxhash64 checksum for this data.
    pub const fn hash_xxh64_slice(data: &[u8]) -> u64 {
        xxh3_64(data)
    }
}
