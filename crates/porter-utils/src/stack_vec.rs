use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::IoSlice;
use std::io::Result;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;
use std::slice::SliceIndex;

/// A simple stack allocated vector for `Copy` types.
#[repr(transparent)]
pub struct StackVec<T, const SIZE: usize> {
    buffer: Cursor<[T; SIZE]>,
}

impl<T, const SIZE: usize> StackVec<T, SIZE>
where
    T: Copy,
{
    /// Constructs a new stack allocated vector from the given buffer.
    #[inline]
    pub const fn new(buffer: [T; SIZE]) -> Self {
        Self {
            buffer: Cursor::new(buffer),
        }
    }

    /// Returns the number of elements that can fit in this stack vector.
    #[inline]
    pub fn capacity(&self) -> usize {
        SIZE
    }

    /// Returns the number of elements in the stack vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.position() as usize
    }

    /// Returns whether or not the stack vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the stack vector.
    ///
    /// # Panics
    /// Will panic if out of capacity.
    #[inline]
    pub fn push(&mut self, value: T) {
        let position = self.reserve(1);

        self.buffer.get_mut()[position] = value;
    }

    /// Inserts an element at position `index` within the stack vector, shifting all,
    /// elements after it to the right.
    ///
    /// # Panics
    /// Panics if `index > len` or if out of capacity.
    pub fn insert(&mut self, index: usize, element: T) {
        let position = self.len();

        if index > position {
            panic!("insertion index (is {index}) should be <= len (is {position})");
        }

        let position = self.reserve(1);

        self.buffer
            .get_mut()
            .copy_within(index..position, index + 1);

        self.buffer.get_mut()[index] = element;
    }

    /// Removes and returns the element at position `index` within the stack vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    #[track_caller]
    pub fn remove(&mut self, index: usize) -> T {
        let position = self.len();

        if index >= position {
            panic!("removal index (is {index}) should be < len (is {position})");
        }

        let result = self.buffer.get_ref()[position];

        self.buffer
            .get_mut()
            .copy_within(index + 1..position, index);

        self.buffer.set_position(position as u64 - 1);

        result
    }

    /// Extends this stack vector with the given slice.
    ///
    /// # Panics
    /// Will panic if out of capacity.
    #[inline]
    pub fn extend_from_slice<S: AsRef<[T]>>(&mut self, slice: S) {
        let slice = slice.as_ref();
        let position = self.reserve(slice.len());

        self.buffer.get_mut()[position..position + slice.len()].copy_from_slice(slice);
    }

    /// Resizes the stack vector in-place so that `len` is equal to `new_len`.
    ///
    /// # Panics
    /// Will panic if `new_len > capacity`.
    pub fn resize(&mut self, new_len: usize, value: T) {
        let position = self.len();

        if new_len > position {
            let additional = new_len - position;
            let position = self.reserve(additional);

            for i in position..position + additional {
                self.buffer.get_mut()[i] = value;
            }
        } else {
            self.buffer.set_position(new_len as u64);
        }
    }

    /// Removes all elements from the stack vector.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.set_position(0);
    }

    /// Try to reserve capacity for at least additioanl more elements to be inserted in the given stack vec.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<()> {
        if self.len() + additional > self.capacity() {
            Err(Error::from(ErrorKind::OutOfMemory))
        } else {
            Ok(())
        }
    }

    /// Try to reserve capacity for at least additioanl more elements to be inserted in the given stack vec.
    ///
    /// This is just an alias to `try_reserve(additional)` because we are just checking if we can fit more elements.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<()> {
        self.try_reserve(additional)
    }

    /// Reserves space for additional elements.
    #[inline]
    fn reserve(&mut self, additional: usize) -> usize {
        let position = self.len();

        if position + additional > self.capacity() {
            panic!("capacity overflow");
        }

        self.buffer
            .set_position(position as u64 + additional as u64);

        position
    }
}

impl<T, const SIZE: usize> From<[T; SIZE]> for StackVec<T, SIZE>
where
    T: Copy,
{
    #[inline]
    fn from(value: [T; SIZE]) -> Self {
        Self::new(value)
    }
}

impl<const SIZE: usize> Write for StackVec<u8, SIZE> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        self.buffer.write_vectored(bufs)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.buffer.write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        self.buffer.flush()
    }
}

impl<T, const SIZE: usize> Deref for StackVec<T, SIZE> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: Data is always initialized, and position is managed by Cursor<[T; SIZE]>.
        unsafe {
            from_raw_parts(
                self.buffer.get_ref().as_ptr(),
                self.buffer.position() as usize,
            )
        }
    }
}

impl<T, const SIZE: usize> DerefMut for StackVec<T, SIZE> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Data is always initialized, and position is managed by Cursor<[T; SIZE]>.
        unsafe {
            from_raw_parts_mut(
                self.buffer.get_mut().as_mut_ptr(),
                self.buffer.position() as usize,
            )
        }
    }
}

impl<T, const SIZE: usize> AsRef<[T]> for StackVec<T, SIZE> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const SIZE: usize> AsMut<[T]> for StackVec<T, SIZE> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, I: SliceIndex<[T]>, const SIZE: usize> Index<I> for StackVec<T, SIZE> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, const SIZE: usize> IndexMut<I> for StackVec<T, SIZE> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}
