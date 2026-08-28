#![deny(unsafe_code)]
#![deny(unused_macros)]

#[macro_export]
macro_rules! assert_size {
    ($ty:ty, $size:expr) => {
        const _: () = assert!(size_of::<$ty>() == $size);
    };
}

#[macro_export]
macro_rules! assert_const {
    ($expr:expr) => {
        const _: () = assert!($expr);
    };
}
