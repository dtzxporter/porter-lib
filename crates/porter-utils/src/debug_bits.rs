/// Utility trait used to print bits to the console.
pub trait DebugBits {
    /// Prints the bits to the console.
    fn debug_bits(&self);
}

/// Helper to implement debug bits.
macro_rules! impl_debug_bits {
    ($typ:ty) => {
        impl DebugBits for $typ {
            fn debug_bits(&self) {
                let size = <$typ>::BITS as usize * 8;

                print!("0b");

                for i in (0..size).rev() {
                    print!("{}", (*self >> i as $typ) & 0x1);
                }

                println!();
            }
        }
    };
}

impl_debug_bits!(i8);
impl_debug_bits!(i16);
impl_debug_bits!(i32);
impl_debug_bits!(i64);
impl_debug_bits!(u8);
impl_debug_bits!(u16);
impl_debug_bits!(u32);
impl_debug_bits!(u64);
impl_debug_bits!(usize);
