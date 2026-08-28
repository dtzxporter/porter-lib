/// Default seed constant.
const SEED: u32 = 0x811c9dc5;
/// Prime constant.
const PRIME: u32 = 0x1000193;

/// Computes the fnv1a32 hash for the given buffer and given seed.
pub fn hash(buffer: &[u8], seed: Option<u32>) -> u32 {
    let mut result = seed.unwrap_or(SEED);

    for byte in buffer {
        result ^= *byte as u32;
        result = result.wrapping_mul(PRIME);
    }

    result
}
