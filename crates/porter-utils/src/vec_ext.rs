use std::collections::TryReserveError;

/// A trait with common `Vec` extensions.
pub trait VecExt<T> {
    /// Constructs a new, empty `Vec<T>` with at least the specified capacity.
    ///
    /// The vector will be able to hold at least `capacity` elements without reallocating.
    /// This method is allowed to allocate for more elements than `capacity`. If `capacity` is zero, the vector will not allocate.
    ///
    /// ### Errors
    /// Returns an error if the capacity exceeds `isize::MAX` bytes, or if the allocator reports allocation failure.
    fn try_with_capacity(capacity: usize) -> Result<Vec<T>, TryReserveError>;
    /// Constructs a new, empty `Vec<T>` with at exactly the specified `capacity`.
    ///
    /// The vector will be able to hold exactly `capacity` elements without reallocating.
    /// If `capacity` is zero, the vector will not allocate.
    ///
    /// ### Errors
    /// Returns an error if the capacity exceeds `isize::MAX` bytes, or if the allocator reports allocation failure.
    fn try_with_exact_capacity(capacity: usize) -> Result<Vec<T>, TryReserveError>;

    /// Constructs a new `Vec<T>` with the given value and length.
    ///
    /// ### Errors
    /// Returns an error if the length exceeds `isize::MAX` bytes, or if the allocator reports allocation failure.
    fn try_new_with_value(value: T, length: usize) -> Result<Vec<T>, TryReserveError>
    where
        T: Clone;
}

impl<T> VecExt<T> for Vec<T> {
    fn try_with_capacity(capacity: usize) -> Result<Vec<T>, TryReserveError> {
        let mut vector = Vec::new();

        vector.try_reserve(capacity)?;

        Ok(vector)
    }

    fn try_with_exact_capacity(capacity: usize) -> Result<Vec<T>, TryReserveError> {
        let mut vector = Vec::new();

        vector.try_reserve_exact(capacity)?;

        Ok(vector)
    }

    fn try_new_with_value(value: T, length: usize) -> Result<Vec<T>, TryReserveError>
    where
        T: Clone,
    {
        let mut vector = Self::try_with_exact_capacity(length)?;

        vector.resize(length, value);

        Ok(vector)
    }
}
