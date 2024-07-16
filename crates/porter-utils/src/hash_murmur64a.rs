/// Murmur64 constant.
const MIX: u64 = 0xc6a4a7935bd1e995;

/// Used to shift the hash.
#[inline(always)]
fn slack(lhs: u64, rhs: u64) -> u64 {
    lhs ^ lhs >> rhs
}

/// Computes the murmur64a hash (murmur2) for the given buffer with the given seed.
fn murmur64a(buffer: &[u8], seed: u64) -> u64 {
    let mut result = seed ^ (buffer.len() as u64).wrapping_mul(MIX);
    let mut chunks = buffer.chunks_exact(8);

    for chunk in &mut chunks {
        result = (result
            ^ slack(
                u64::from_le_bytes(chunk.try_into().unwrap()).wrapping_mul(MIX),
                47,
            )
            .wrapping_mul(MIX))
        .wrapping_mul(MIX)
    }

    let result = {
        let remainder = chunks.remainder();

        if remainder.is_empty() {
            result
        } else {
            (result
                ^ remainder
                    .iter()
                    .rev()
                    .fold(0, |r, &i| (i as u64) | (r << 8)))
            .wrapping_mul(MIX)
        }
    };

    slack(slack(result, 47).wrapping_mul(MIX), 47)
}

/// Utility to hash data with murmur64a algo.
pub trait HashMurMur64A {
    /// Creates a murmur64a checksum for this data.
    fn hash_murmur64a(&self) -> u64;
}

impl HashMurMur64A for &[u8] {
    fn hash_murmur64a(&self) -> u64 {
        murmur64a(self, 0)
    }
}

impl HashMurMur64A for &str {
    fn hash_murmur64a(&self) -> u64 {
        murmur64a(self.as_bytes(), 0)
    }
}

impl HashMurMur64A for String {
    fn hash_murmur64a(&self) -> u64 {
        murmur64a(self.as_bytes(), 0)
    }
}
