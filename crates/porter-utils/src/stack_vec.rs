use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::IoSlice;
use std::io::Result;
use std::io::Write;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::SliceIndex;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;

/// A simple stack allocated vector for `Copy` types.
#[repr(transparent)]
pub struct StackVec<T, const SIZE: usize> {
    buffer: Cursor<[T; SIZE]>,
}

impl<T, const SIZE: usize> StackVec<T, SIZE>
where
    T: Copy,
{
    /// Constructs a new stack allocated vector.
    #[inline]
    pub const fn new() -> Self {
        let backing: MaybeUninit<[T; SIZE]> = MaybeUninit::uninit();

        // SAFETY: Access to the backing buffer is protected by the cursor length.
        // As elements are initialized, they become available through the vector.
        let buffer = unsafe { backing.assume_init() };

        Self {
            buffer: Cursor::new(buffer),
        }
    }

    /// Returns the number of elements that can fit in this stack vector.
    #[inline]
    pub const fn capacity(&self) -> usize {
        SIZE
    }

    /// Returns the number of elements in the stack vector.
    #[inline]
    pub const fn len(&self) -> usize {
        self.buffer.position() as usize
    }

    /// Forces the length of the stack vector to `new_len`.
    ///
    /// # Safety
    /// - `new_len` must be less than or equal to [`capacity()`].
    /// - The elements at `old_len..new_len` must be initialized.
    ///
    /// [`capacity()`]: StackVec::capacity
    #[inline]
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        self.buffer.set_position(new_len as _);
    }

    /// Returns whether or not the stack vector is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the stack vector.
    ///
    /// # Panics
    /// Will panic if out of capacity.
    #[inline]
    pub const fn push(&mut self, value: T) {
        let position = self.reserve(1);
        let buffer = self.buffer.get_mut();

        buffer[position] = value;
    }

    /// Inserts an element at position `index` within the stack vector, shifting all,
    /// elements after it to the right.
    ///
    /// # Panics
    /// Panics if `index > len` or if out of capacity.
    #[track_caller]
    pub fn insert(&mut self, index: usize, element: T) {
        let position = self.len();

        if index > position {
            panic!("insertion index (is {index}) should be <= len (is {position})");
        }

        let position = self.reserve(1);
        let buffer = self.buffer.get_mut();

        buffer.copy_within(index..position, index + 1);
        buffer[index] = element;
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

        let buffer = self.buffer.get_mut();
        let result = buffer[position];

        buffer.copy_within(index + 1..position, index);

        self.buffer
            .set_position(position as u64 - 1);

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
        let buffer = self.buffer.get_mut();

        buffer[position..position + slice.len()].copy_from_slice(slice);
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
            let buffer = self.buffer.get_mut();

            for item in buffer
                .iter_mut()
                .skip(position)
                .take(additional)
            {
                *item = value;
            }
        } else {
            self.buffer.set_position(new_len as u64);
        }
    }

    /// Removes all elements from the stack vector.
    #[inline]
    pub const fn clear(&mut self) {
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
    const fn reserve(&mut self, additional: usize) -> usize {
        let position = self.len();

        if position + additional > self.capacity() {
            panic!("capacity overflow");
        }

        self.buffer
            .set_position(position as u64 + additional as u64);

        position
    }
}

impl<T, const SIZE: usize> Default for StackVec<T, SIZE>
where
    T: Copy,
{
    fn default() -> Self {
        Self::new()
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
