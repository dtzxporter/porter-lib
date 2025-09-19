use crate::Face;
use crate::FaceBuffer;

/// Useful methods for working with face buffers.
pub trait FaceBufferExt {
    /// Creates a new triangle list face buffer from the given strip indices and separator.
    fn from_strips_u16(strips: &[u16], separator: u16) -> FaceBuffer;
    /// Creates a new triangle list face buffer from the given strip indices and separator.
    fn from_strips_u32(strips: &[u32], separator: u32) -> FaceBuffer;
    /// Swaps the winding order of all of the faces.
    fn swap_order(&mut self);
}

impl FaceBufferExt for FaceBuffer {
    fn from_strips_u16(strips: &[u16], separator: u16) -> FaceBuffer {
        let mut buffer = FaceBuffer::new();
        let mut i = 0;

        while i < strips.len() {
            if i + 2 >= strips.len() || strips[i] == separator {
                i += 1;
                continue;
            }

            let mut triangle_count = 0;

            while i + 2 < strips.len()
                && strips[i] != separator
                && strips[i + 1] != separator
                && strips[i + 2] != separator
            {
                if triangle_count % 2 == 0 {
                    buffer.push(Face::new(
                        strips[i] as u32,
                        strips[i + 1] as u32,
                        strips[i + 2] as u32,
                    ));
                } else {
                    buffer.push(Face::new(
                        strips[i + 2] as u32,
                        strips[i + 1] as u32,
                        strips[i] as u32,
                    ));
                }

                i += 1;
                triangle_count += 1;
            }

            i += 1;
        }

        buffer
    }

    fn from_strips_u32(strips: &[u32], separator: u32) -> FaceBuffer {
        let mut buffer = FaceBuffer::new();
        let mut i = 0;

        while i < strips.len() {
            if i + 2 >= strips.len() || strips[i] == separator {
                i += 1;
                continue;
            }

            let mut triangle_count = 0;

            while i + 2 < strips.len()
                && strips[i] != separator
                && strips[i + 1] != separator
                && strips[i + 2] != separator
            {
                if triangle_count % 2 == 0 {
                    buffer.push(Face::new(strips[i], strips[i + 1], strips[i + 2]));
                } else {
                    buffer.push(Face::new(strips[i + 2], strips[i + 1], strips[i]));
                }

                i += 1;
                triangle_count += 1;
            }

            i += 1;
        }

        buffer
    }

    fn swap_order(&mut self) {
        for face in self {
            face.swap_order();
        }
    }
}
