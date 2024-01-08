/// Utility trait to extract digits from a number.
pub trait ExtractDigits<T> {
    /// Extracts the upper left hand side digits.
    fn extract_upper_digits(&self, len: usize) -> T;
    /// Extracts the lower right hand side digits.
    fn extract_lower_digits(&self, len: usize) -> T;
}

/// Helper to implement the routine for a generic number type.
macro_rules! impl_extract_digits {
    ($typ:ty) => {
        impl ExtractDigits<$typ> for $typ {
            fn extract_upper_digits(&self, len: usize) -> $typ {
                let divisor = 10u64.pow(len as u32);
                let result = *self as u64 / divisor;

                result as $typ
            }

            fn extract_lower_digits(&self, len: usize) -> $typ {
                let divisor = 10u64.pow(len as u32);
                let result = *self as u64 % divisor;

                result as $typ
            }
        }
    };
}

impl_extract_digits!(i16);
impl_extract_digits!(u16);
impl_extract_digits!(i32);
impl_extract_digits!(u32);
impl_extract_digits!(i64);
impl_extract_digits!(u64);
impl_extract_digits!(usize);
