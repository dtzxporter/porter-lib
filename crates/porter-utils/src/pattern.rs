use std::io;
use std::io::Read;

use std::fmt::Debug;

use memchr::memchr_iter;

/// Maximum pattern length in bytes.
const MAXIMUM_LENGTH: usize = 32;
/// Size in bytes to scan buffers.
const SCAN_BUFFER_SIZE: usize = 0x100000;

/// A compiled pattern used to search for bytes.
pub struct Pattern {
    data: [u8; MAXIMUM_LENGTH],
    mask: [u8; MAXIMUM_LENGTH],
    len: usize,
}

impl Pattern {
    /// Constructs and compiles a new pattern.
    pub const fn new(pattern: &str) -> Self {
        let mut data: [u8; MAXIMUM_LENGTH] = [0; MAXIMUM_LENGTH];
        let mut mask: [u8; MAXIMUM_LENGTH] = [0; MAXIMUM_LENGTH];
        let mut len: usize = 0;

        let mut temp_digit: u8 = 0;
        let mut temp_flag = false;
        let mut last_unknown = false;

        let pattern = pattern.as_bytes();
        let mut offset = 0;

        while offset < pattern.len() {
            let ch = pattern[offset] as char;

            if ch.is_ascii_whitespace() {
                last_unknown = false;
            } else if ch == '?' {
                // Ignore any initial wildcards in the pattern because they don't mean anything.
                // Forces the data to always start with a valid byte to search for.
                if len == 0 {
                    if temp_flag {
                        panic!("Pattern was malformed!");
                    }
                    continue;
                }

                if last_unknown {
                    last_unknown = false;
                } else {
                    if len == MAXIMUM_LENGTH {
                        panic!("Pattern exceeds current maximum length of bytes!");
                    }

                    data[len] = 0x0;
                    mask[len] = 0x0;

                    len += 1;

                    last_unknown = true;
                }
            } else if ch.is_ascii_hexdigit() {
                let digit = match ch.to_digit(16) {
                    Some(digit) => digit,
                    None => 0,
                };

                if !temp_flag {
                    temp_digit = (digit << 4) as u8;
                    temp_flag = true;
                } else {
                    temp_digit |= digit as u8;
                    temp_flag = false;

                    if len == MAXIMUM_LENGTH {
                        panic!("Pattern exceeds current maximum length of bytes!");
                    }

                    data[len] = temp_digit;
                    mask[len] = 0xFF;

                    len += 1;
                }

                last_unknown = false;
            }

            offset += 1;
        }

        Self { data, mask, len }
    }

    /// Scans the given buffer for this pattern and returns the byte offset if found.
    pub fn scan<B: AsRef<[u8]>>(&self, buffer: B) -> Option<usize> {
        let buffer = buffer.as_ref();

        if self.len == 0 {
            return None;
        }

        if self.len <= 4 {
            self.scan_fast_32bit(buffer)
        } else if self.len <= 8 {
            self.scan_fast_64bit(buffer)
        } else if self.len <= 16 {
            self.scan_fast_128bit(buffer)
        } else {
            self.scan_slow(buffer)
        }
    }

    /// Scans the given reader for this pattern and returns the byte offset from the current position if found.
    pub fn scan_from<R: Read>(&self, mut read: R) -> Result<Option<usize>, io::Error> {
        let mut scratch = vec![0; SCAN_BUFFER_SIZE];
        let mut offset = 0;
        let mut overlap = 0;

        loop {
            let len = read.read(&mut scratch[overlap..])?;

            if let Some(result) = self.scan(&scratch[0..len + overlap]) {
                return Ok(Some(result + offset));
            }

            // Overlap at least self.len bytes from the end of the buffer
            // So we can check for matches at chunk boundaries.
            if (overlap + len) < self.len {
                overlap += len;
            } else {
                // Copy the end self.len bytes to the start.
                scratch.copy_within(len - self.len..len, 0);

                overlap = self.len;
                offset += len - self.len;
            }

            if len == 0 {
                break;
            }
        }

        Ok(None)
    }

    /// Fast scan path for patterns <= 16 bytes.
    fn scan_fast_128bit(&self, buffer: &[u8]) -> Option<usize> {
        let mut mask: [u8; 16] = [0; 16];
        let mut compare: [u8; 16] = [0; 16];

        // For whatever reason the code generation is much better using a manual copy.
        #[allow(clippy::manual_memcpy)]
        for i in 0..self.len {
            mask[i] = self.mask[i];
            compare[i] = self.data[i];
        }

        let mask = u128::from_ne_bytes(mask);
        let compare = u128::from_ne_bytes(compare);

        let mut load: [u8; 16] = [0; 16];

        for potential in memchr_iter(self.data[0], buffer) {
            if potential + self.len >= buffer.len() {
                continue;
            }

            for i in 0..16 {
                if i >= self.len {
                    load[i] = 0;
                } else {
                    load[i] = buffer[potential + i];
                }
            }

            let load = u128::from_ne_bytes(load);

            let result = load ^ compare;
            let result = result & mask;

            if result == 0 {
                return Some(potential);
            }
        }

        None
    }

    /// Fast scan path for patterns <= 8 bytes.
    fn scan_fast_64bit(&self, buffer: &[u8]) -> Option<usize> {
        let mut mask: [u8; 8] = [0; 8];
        let mut compare: [u8; 8] = [0; 8];

        // For whatever reason the code generation is much better using a manual copy.
        #[allow(clippy::manual_memcpy)]
        for i in 0..self.len {
            mask[i] = self.mask[i];
            compare[i] = self.data[i];
        }

        let mask = u64::from_ne_bytes(mask);
        let compare = u64::from_ne_bytes(compare);

        let mut load: [u8; 8] = [0; 8];

        for potential in memchr_iter(self.data[0], buffer) {
            if potential + self.len >= buffer.len() {
                continue;
            }

            for i in 0..8 {
                if i >= self.len {
                    load[i] = 0;
                } else {
                    load[i] = buffer[potential + i];
                }
            }

            let load = u64::from_ne_bytes(load);

            let result = load ^ compare;
            let result = result & mask;

            if result == 0 {
                return Some(potential);
            }
        }

        None
    }

    /// Fast scan path for patterns <= 4 bytes.
    fn scan_fast_32bit(&self, buffer: &[u8]) -> Option<usize> {
        let mut mask: [u8; 4] = [0; 4];
        let mut compare: [u8; 4] = [0; 4];

        // For whatever reason the code generation is much better using a manual copy.
        #[allow(clippy::manual_memcpy)]
        for i in 0..self.len {
            mask[i] = self.mask[i];
            compare[i] = self.data[i];
        }

        let mask = u32::from_ne_bytes(mask);
        let compare = u32::from_ne_bytes(compare);

        let mut load: [u8; 4] = [0; 4];

        for potential in memchr_iter(self.data[0], buffer) {
            if potential + self.len >= buffer.len() {
                continue;
            }

            for i in 0..4 {
                if i >= self.len {
                    load[i] = 0;
                } else {
                    load[i] = buffer[potential + i];
                }
            }

            let load = u32::from_ne_bytes(load);

            let result = load ^ compare;
            let result = result & mask;

            if result == 0 {
                return Some(potential);
            }
        }

        None
    }

    /// Slow scan fallback for patterns > 16 bytes.
    fn scan_slow(&self, buffer: &[u8]) -> Option<usize> {
        for potential in memchr_iter(self.data[0], buffer) {
            if potential + self.len >= buffer.len() {
                continue;
            }

            let mut matched = true;

            for i in 0..self.len {
                if self.mask[i] == 0x0 {
                    continue;
                }

                if self.data[i] != buffer[potential + i] {
                    matched = false;
                    break;
                }
            }

            if matched {
                return Some(potential);
            }
        }

        None
    }
}

impl Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pattern")
            .field("data", &&self.data[0..self.len])
            .field("mask", &&self.mask[0..self.len])
            .finish()
    }
}
