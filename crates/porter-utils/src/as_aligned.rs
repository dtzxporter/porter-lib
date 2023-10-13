/// Utility to implement the trait.
macro_rules! impl_aligned {
    ($type:ty, $zero:expr) => {
        impl AsAligned for $type {
            fn as_aligned(&self, alignment: Self) -> Self {
                if *self % alignment > $zero {
                    *self + alignment - *self % alignment
                } else {
                    *self
                }
            }
        }
    };
    ($type:ty) => {
        impl_aligned!($type, 0);
    };
}

/// A trait that aligns numbers.
pub trait AsAligned: Copy
where
    Self: std::ops::Rem<Self>,
{
    /// Returns the number aligned to the given alignment.
    fn as_aligned(&self, alignment: Self) -> Self;
}

impl_aligned!(u8);
impl_aligned!(u16);
impl_aligned!(u32);
impl_aligned!(u64);
impl_aligned!(usize);
impl_aligned!(i8);
impl_aligned!(i16);
impl_aligned!(i32);
impl_aligned!(i64);
impl_aligned!(isize);
impl_aligned!(f32, 0.0);
impl_aligned!(f64, 0.0);
