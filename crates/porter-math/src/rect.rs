/// A 2d rectangular region.
#[derive(Default, Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// Constructs a new rect.
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Gets the left of the rect.
    pub fn left(&self) -> u32 {
        self.x
    }

    /// Gets the top of the rect.
    pub fn top(&self) -> u32 {
        self.y
    }

    /// Gets the right of the rect.
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    /// Gets the bottom of the rect.
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    /// Returns the intersection of two rectangles.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);

        let width = (self.right().min(other.right()) as i32 - x as i32).max(0) as u32;
        let height = (self.bottom().min(other.bottom()) as i32 - y as i32).max(0) as u32;

        if width > 0 && height > 0 {
            Some(Rect::new(x, y, width, height))
        } else {
            None
        }
    }

    /// Returns true if the other rectangle intersects this rectangle.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.intersection(other).is_some()
    }
}
