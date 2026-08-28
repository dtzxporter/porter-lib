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
    /// Constructs a new `Vec<T>` with values zeroed from the given length.
    ///
    ///  ### Errors
    /// Returns an error if the length exceeds `isize::MAX` bytes, or if the allocator reports allocation failure.
    fn try_new_zeroed(length: usize) -> Result<Vec<T>, TryReserveError>
    where
        T: Copy + 'static;
    /// Constructs a new `Vec<T>` with the given value and length.
    ///
    /// ### Errors
    /// Returns an error if the length exceeds `isize::MAX` bytes, or if the allocator reports allocation failure.
    fn try_new_with_value(value: T, length: usize) -> Result<Vec<T>, TryReserveError>
    where
        T: Clone;
    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    fn try_resize(&mut self, new_len: usize, value: T) -> Result<(), TryReserveError>
    where
        T: Clone;
    /// Appends an element to the back of a collection.
    fn try_push(&mut self, value: T) -> Result<(), TryReserveError>;
    /// Appends an element to the back of a collection, returning a reference to it.
    fn try_push_mut(&mut self, value: T) -> Result<&mut T, TryReserveError>;
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

    fn try_new_zeroed(length: usize) -> Result<Vec<T>, TryReserveError>
    where
        T: Copy + 'static,
    {
        let mut vector = Self::try_with_exact_capacity(length)?;

        // SAFETY: All elements are initialized as long as write_bytes upholds its contract.
        unsafe { vector.set_len(length) };
        // SAFETY: The vector has at least length capacity, and is aligned automatically.
        unsafe { std::ptr::write_bytes(vector.as_mut_ptr(), 0, length) };

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

    fn try_resize(&mut self, new_len: usize, value: T) -> Result<(), TryReserveError>
    where
        T: Clone,
    {
        if new_len > self.len() {
            self.try_reserve(new_len - self.len())?;
        }

        self.resize(new_len, value);

        Ok(())
    }

    fn try_push(&mut self, value: T) -> Result<(), TryReserveError> {
        self.try_reserve(1)?;
        self.push(value);

        Ok(())
    }

    fn try_push_mut(&mut self, value: T) -> Result<&mut T, TryReserveError> {
        self.try_reserve(1)?;

        Ok(self.push_mut(value))
    }
}
