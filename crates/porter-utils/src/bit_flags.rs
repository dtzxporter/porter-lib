use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

/// Wrapper that provides methods for reading and writing bit flags.
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitFlags<B> {
    inner: B,
}

/// Helper to implement bit flags.
macro_rules! impl_bit_flags {
    ($typ:ty) => {
        impl BitFlags<$typ> {
            /// Constructs a new bit flags with no bits set.
            pub const fn new() -> Self {
                Self { inner: 0 }
            }

            /// Consumes the bit flags and returns the inner value.
            pub const fn into_inner(self) -> $typ {
                self.inner
            }

            /// Whether all bits are unset.
            pub const fn is_empty(&self) -> bool {
                self.inner == 0
            }

            /// Whether all bits set in `other` are set in `self`.
            pub const fn contains(&self, other: $typ) -> bool {
                (self.inner & other) == other
            }

            /// Unsets all the bits.
            pub const fn clear(&mut self) {
                self.inner = 0
            }

            /// Sets or unsets the bits in `other` based on `value`.
            pub const fn set(&mut self, other: $typ, value: bool) {
                if value {
                    self.inner |= other;
                } else {
                    self.inner &= !other;
                }
            }

            /// Swaps the byte order in the bit flags.
            pub const fn swap_bytes(self) -> Self {
                Self {
                    inner: self.inner.swap_bytes(),
                }
            }
        }

        impl From<$typ> for BitFlags<$typ> {
            #[inline(always)]
            fn from(value: $typ) -> BitFlags<$typ> {
                BitFlags { inner: value }
            }
        }

        impl Deref for BitFlags<$typ> {
            type Target = $typ;

            #[inline(always)]
            fn deref(&self) -> &$typ {
                &self.inner
            }
        }

        impl DerefMut for BitFlags<$typ> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut $typ {
                &mut self.inner
            }
        }

        impl Debug for BitFlags<$typ> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let size = <$typ>::BITS as usize;

                write!(f, "0b")?;

                for i in (0..size).rev() {
                    write!(f, "{}", (self.inner >> i as $typ) & 0x1)?;
                }

                write!(f, " ({:#02x?})", self.inner)?;
                Ok(())
            }
        }
    };
}

impl_bit_flags!(i8);
impl_bit_flags!(i16);
impl_bit_flags!(i32);
impl_bit_flags!(i64);
impl_bit_flags!(isize);
impl_bit_flags!(u8);
impl_bit_flags!(u16);
impl_bit_flags!(u32);
impl_bit_flags!(u64);
impl_bit_flags!(usize);

/// Used to group similar bit flags into a shared structure.
#[macro_export]
macro_rules! bitflags {
    (
        impl $struct:ty : $typ:ty {
            $($body:tt)*
        }
    ) => {
        impl $struct {
            $($body)*
        }

        impl From<$typ> for $struct {
            #[inline(always)]
            fn from(value: $typ) -> Self {
                Self($crate::BitFlags::<$typ>::from(value))
            }
        }

        impl ::std::ops::Deref for $struct {
            type Target = $crate::BitFlags<$typ>;

            fn deref(&self) -> &$crate::BitFlags<$typ> {
                &self.0
            }
        }

        impl ::std::ops::DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut $crate::BitFlags<$typ> {
                &mut self.0
            }
        }
    };
}
