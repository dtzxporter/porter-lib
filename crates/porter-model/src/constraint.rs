/// The type of constraint to apply.
#[derive(Debug, Clone, Copy)]
pub enum ConstraintType {
    Point,
    Orient,
    Scale,
}

/// A 3d bone constraint.
#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: Option<String>,
    pub constraint_type: ConstraintType,
    pub constraint_bone: usize,
    pub target_bone: usize,
    pub maintain_offset: bool,
    pub skip_x: bool,
    pub skip_y: bool,
    pub skip_z: bool,
}

impl Constraint {
    /// Constructs a new instance of a constraint.
    pub fn new(
        name: Option<String>,
        constraint_type: ConstraintType,
        constraint_bone: usize,
        target_bone: usize,
        maintain_offset: bool,
    ) -> Self {
        Self {
            name,
            constraint_type,
            constraint_bone,
            target_bone,
            maintain_offset,
            skip_x: false,
            skip_y: false,
            skip_z: false,
        }
    }

    /// Sets whether or not to skip the x axis.
    #[inline]
    pub fn skip_x(mut self, skip: bool) -> Self {
        self.skip_x = skip;
        self
    }

    /// Sets whether or not to skip the z axis.
    #[inline]
    pub fn skip_y(mut self, skip: bool) -> Self {
        self.skip_y = skip;
        self
    }

    /// Sets whether or not to skip the z axis.
    #[inline]
    pub fn skip_z(mut self, skip: bool) -> Self {
        self.skip_z = skip;
        self
    }
}
