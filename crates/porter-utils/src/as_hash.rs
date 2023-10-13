use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// Used to hash certain values.
pub trait AsHash: Hash {
    /// Computes a 64bit hash for the value.
    fn as_hash(&self) -> u64;
}

impl<T> AsHash for T
where
    T: Hash,
{
    fn as_hash(&self) -> u64 {
        let hash = DefaultHasher::new();

        self.hash(&mut DefaultHasher::new());

        hash.finish()
    }
}
