/// A trait that adds a few methods to `Option<T>`.
pub trait OptionExt<T> {
    /// Returns `true` if the option contains the given value.
    #[must_use]
    fn contains<U>(&self, rhs: &U) -> bool
    where
        U: PartialEq<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn contains<U>(&self, lhs: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match *self {
            Some(ref rhs) => lhs == rhs,
            None => false,
        }
    }
}
