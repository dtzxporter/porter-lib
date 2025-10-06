/// Default seed constant.
const SEED: u64 = 0x0;
/// Mixing constant.
const MIX: u64 = 0xc6a4a7935bd1e995;

/// Used to shift the hash.
#[inline(always)]
fn mix(lhs: u64, rhs: u64) -> u64 {
    lhs ^ (lhs >> rhs)
}

/// Computes the murmura64 hash for the given buffer and given seed.
pub fn hash(buffer: &[u8], seed: Option<u64>) -> u64 {
    let mut result = seed.unwrap_or(SEED) ^ (buffer.len() as u64).wrapping_mul(MIX);
    let mut chunks = buffer.chunks_exact(8);

    for chunk in &mut chunks {
        result = (result
            ^ mix(
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

    mix(mix(result, 47).wrapping_mul(MIX), 47)
}
