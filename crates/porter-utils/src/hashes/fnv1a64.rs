/// Default seed constant.
const SEED: u64 = 0xcbf29ce484222325;
/// Prime constant.
const PRIME: u64 = 0x100000001b3;

/// Computes the fnv1a64 hash for the given buffer and given seed.
pub fn hash(buffer: &[u8], seed: Option<u64>) -> u64 {
    let mut result = seed.unwrap_or(SEED);

    for byte in buffer {
        result ^= *byte as u64;
        result = result.wrapping_mul(PRIME);
    }

    result
}
