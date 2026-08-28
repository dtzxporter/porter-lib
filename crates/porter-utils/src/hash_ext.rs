use crate::hashes;

/// Utility trait that adds custom hash algorithms to byte data.
pub trait HashExt {
    /// Creates a xxhash3 64bit hash for this data.
    fn hash_xxh364(&self) -> u64;
    /// Creates a xxhash3 64bit hash for this data with a custom seed.
    fn hash_xxh364_with_seed(&self, seed: u64) -> u64;
    /// Creates a murmur(a) 64bit hash for this data.
    fn hash_murmura64(&self) -> u64;
    /// Creates a murmur(a) 64bit hash for this data with a custom seed.
    fn hash_murmura64_with_seed(&self, seed: u64) -> u64;
    /// Creates a fnv1(a) 64bit hash for this data.
    fn hash_fnv1a64(&self) -> u64;
    /// Creates a fnv1(a) 64bit hash for this data with a custom seed.
    fn hash_fnv1a64_with_seed(&self, seed: u64) -> u64;
    /// Creates a fnv1 64bit hash for this data.
    fn hash_fnv164(&self) -> u64;
    /// Creates a fnv1 64bit hash for this data with a custom seed.
    fn hash_fnv164_with_seed(&self, seed: u64) -> u64;
    /// Creates a fnv1(a) 32bit hash for this data.
    fn hash_fnv1a32(&self) -> u32;
    /// Creates a fnv1(a) 32bit hash for this data with a custom seed.
    fn hash_fnv1a32_with_seed(&self, seed: u32) -> u32;
    /// Creates a fnv1 32bit hash for this data.
    fn hash_fnv132(&self) -> u32;
    /// Creates a fnv1 32bit hash for this data with a custom seed.
    fn hash_fnv132_with_seed(&self, seed: u32) -> u32;
}

impl<T> HashExt for T
where
    T: AsRef<[u8]>,
{
    fn hash_xxh364(&self) -> u64 {
        xxhash_rust::xxh3::xxh3_64_with_seed(self.as_ref(), 0)
    }

    fn hash_xxh364_with_seed(&self, seed: u64) -> u64 {
        xxhash_rust::xxh3::xxh3_64_with_seed(self.as_ref(), seed)
    }

    fn hash_murmura64(&self) -> u64 {
        hashes::murmura64::hash(self.as_ref(), None)
    }

    fn hash_murmura64_with_seed(&self, seed: u64) -> u64 {
        hashes::murmura64::hash(self.as_ref(), Some(seed))
    }

    fn hash_fnv1a64(&self) -> u64 {
        hashes::fnv1a64::hash(self.as_ref(), None)
    }

    fn hash_fnv1a64_with_seed(&self, seed: u64) -> u64 {
        hashes::fnv1a64::hash(self.as_ref(), Some(seed))
    }

    fn hash_fnv164(&self) -> u64 {
        hashes::fnv164::hash(self.as_ref(), None)
    }

    fn hash_fnv164_with_seed(&self, seed: u64) -> u64 {
        hashes::fnv164::hash(self.as_ref(), Some(seed))
    }

    fn hash_fnv1a32(&self) -> u32 {
        hashes::fnv1a32::hash(self.as_ref(), None)
    }

    fn hash_fnv1a32_with_seed(&self, seed: u32) -> u32 {
        hashes::fnv1a32::hash(self.as_ref(), Some(seed))
    }

    fn hash_fnv132(&self) -> u32 {
        hashes::fnv132::hash(self.as_ref(), None)
    }

    fn hash_fnv132_with_seed(&self, seed: u32) -> u32 {
        hashes::fnv132::hash(self.as_ref(), Some(seed))
    }
}
