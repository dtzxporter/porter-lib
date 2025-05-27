/// A simple knot vector implementation.
#[derive(Debug, Clone)]
pub struct KnotVector {
    knots: Vec<f32>,
}

impl KnotVector {
    /// Constructs a new knot vector with the given knots.
    pub fn new(knots: Vec<f32>) -> Self {
        debug_assert!(knots.len() >= 2, "Requires at least two knots!");

        Self { knots }
    }

    /// Gets the knot at the given index.
    pub fn get(&self, index: usize) -> Option<&f32> {
        self.knots.get(index)
    }

    /// Gets the number of knots in the vector.
    pub fn len(&self) -> usize {
        self.knots.len()
    }

    /// Whether or not this knot vector is empty.
    pub fn is_empty(&self) -> bool {
        self.knots.is_empty()
    }

    /// Finds the correct knot interval index for the given time.
    pub fn interval(&self, time: f32) -> Option<usize> {
        if self.knots.len() < 2 {
            return None;
        }

        let ext = self.knots.len() - 1;
        let mut low = 1;
        let mut high = ext - 1;

        if time < self.knots[low] {
            if time < self.knots[0] { None } else { Some(0) }
        } else if time >= self.knots[high] {
            if time > self.knots[ext] {
                None
            } else {
                Some(high)
            }
        } else {
            let mut index = (high + low) >> 1;

            loop {
                if time < self.knots[index] {
                    high = index;
                } else if time >= self.knots[index + 1] {
                    low = index
                } else {
                    return Some(index);
                }

                index = (high + low) >> 1;
            }
        }
    }
}
