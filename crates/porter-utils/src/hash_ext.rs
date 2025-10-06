use xxhash_rust::xxh3::xxh3_64_with_seed;

use crate::hashes;

/// Utility trait that adds custom hash algorithms to byte data.
pub trait HashExt {
    /// Creates a xxhash3 64bit hash for this data.
    fn hash_xxh364(&self) -> u64;
    /// Creates a murmur(a) 64bit hash for this data.
    fn hash_murmura64(&self) -> u64;
    /// Creates a fnv1(a) 64bit hash for this data.
    fn hash_fnv1a64(&self) -> u64;
}

impl HashExt for &[u8] {
    fn hash_xxh364(&self) -> u64 {
        xxh3_64_with_seed(self, 0)
    }

    fn hash_murmura64(&self) -> u64 {
        hashes::murmura64::hash(self, None)
    }

    fn hash_fnv1a64(&self) -> u64 {
        hashes::fnv1a64::hash(self, None)
    }
}

impl HashExt for &str {
    fn hash_xxh364(&self) -> u64 {
        self.as_bytes().hash_xxh364()
    }

    fn hash_murmura64(&self) -> u64 {
        self.as_bytes().hash_murmura64()
    }

    fn hash_fnv1a64(&self) -> u64 {
        self.as_bytes().hash_fnv1a64()
    }
}

impl HashExt for String {
    fn hash_xxh364(&self) -> u64 {
        self.as_bytes().hash_xxh364()
    }

    fn hash_murmura64(&self) -> u64 {
        self.as_bytes().hash_murmura64()
    }

    fn hash_fnv1a64(&self) -> u64 {
        self.as_bytes().hash_fnv1a64()
    }
}
