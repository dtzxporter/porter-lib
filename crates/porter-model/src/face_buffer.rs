use static_assertions::assert_eq_size;

/// Represents one face or triangle indices for a polygon mesh.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Face {
    /// The first index for this face.
    pub i1: u32,
    /// The second index for this face.
    pub i2: u32,
    /// The third index for this face.
    pub i3: u32,
}

assert_eq_size!([u8; 0xC], Face);

impl Face {
    /// Constructs a new face.
    pub fn new(i1: u32, i2: u32, i3: u32) -> Self {
        Self { i1, i2, i3 }
    }

    /// Swaps the winding order of the face in place.
    #[allow(clippy::manual_swap)]
    pub fn swap_order(&mut self) {
        let tmp = self.i3;

        self.i3 = self.i1;
        self.i1 = tmp;
    }

    /// Swaps the winding order of the face, returning the new face.
    #[allow(clippy::manual_swap)]
    pub fn swapped_order(mut self) -> Self {
        let tmp = self.i3;

        self.i3 = self.i1;
        self.i1 = tmp;
        self
    }
}

impl From<(u16, u16, u16)> for Face {
    #[inline]
    fn from(value: (u16, u16, u16)) -> Self {
        Self {
            i1: value.0 as u32,
            i2: value.1 as u32,
            i3: value.2 as u32,
        }
    }
}

impl From<(u32, u32, u32)> for Face {
    #[inline]
    fn from(value: (u32, u32, u32)) -> Self {
        Self {
            i1: value.0,
            i2: value.1,
            i3: value.2,
        }
    }
}

/// A buffer of triangle vertex indices for a polygon mesh.
pub type FaceBuffer = Vec<Face>;
