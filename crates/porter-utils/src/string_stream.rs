use std::io::Read;

/// Default extra characters to consider.
const DEFAULT_EXTRA_CHARS: &str = "_-";

/// Extracts ascii strings from a stream of bytes.
pub struct StringStream<S: Read> {
    stream: S,
    min_length: usize,
    max_length: Option<usize>,
    extra_chars: &'static str,
}

impl<S> StringStream<S>
where
    S: Read,
{
    /// Constructs a new instance of string stream with the given underlying stream.
    pub const fn new(stream: S) -> Self {
        Self {
            stream,
            min_length: 3,
            max_length: None,
            extra_chars: DEFAULT_EXTRA_CHARS,
        }
    }

    /// Specifies the minimum string length. (Default: 3)
    pub const fn with_min_length(mut self, min_length: usize) -> Self {
        self.min_length = min_length;
        self
    }

    /// Specifies the maximum string length. (Default: infinite)
    pub const fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Specifies extra characters to consider a valid string. (Default: '_-')
    pub const fn with_extra_chars(mut self, extra_chars: &'static str) -> Self {
        self.extra_chars = extra_chars;
        self
    }
}

impl<S> Iterator for StringStream<S>
where
    S: Read,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = String::with_capacity(128);
        let mut scratch: [u8; 1] = [0; 1];
        let mut eof = false;

        while !eof {
            loop {
                if self
                    .stream
                    .read_exact(&mut scratch)
                    .is_err()
                {
                    eof = true;
                    break;
                }

                let ch = scratch[0] as char;

                if ch.is_ascii_uppercase()
                    || ch.is_ascii_lowercase()
                    || ch.is_ascii_digit()
                    || self.extra_chars.contains(ch)
                {
                    result.push(ch);
                } else {
                    break;
                }
            }

            if result.len() >= self.min_length
                && self
                    .max_length
                    .is_none_or(|x| result.len() <= x)
            {
                return Some(result);
            } else if eof {
                return None;
            } else {
                result.clear();
                continue;
            }
        }

        None
    }
}
