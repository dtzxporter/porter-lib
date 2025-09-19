use porter_math::Quaternion;
use porter_math::Vector3;

/// The type of constraint to apply.
#[derive(Debug, Clone, Copy)]
pub enum ConstraintType {
    Point,
    Orient,
    Scale,
}

/// Type of offset the constraint has.
#[derive(Debug, Clone, Copy)]
pub enum ConstraintOffset {
    None,
    Maintain,
    Vector3(Vector3),
    Quaternion(Quaternion),
}

/// A 3d bone constraint.
#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: Option<String>,
    pub constraint_type: ConstraintType,
    pub constraint_bone: usize,
    pub target_bone: usize,
    pub offset: ConstraintOffset,
    pub weight: f32,
    pub skip_x: bool,
    pub skip_y: bool,
    pub skip_z: bool,
}

impl Constraint {
    /// Constructs a new instance of a constraint.
    pub fn new<T: Into<ConstraintOffset>>(
        name: Option<String>,
        constraint_type: ConstraintType,
        constraint_bone: usize,
        target_bone: usize,
        offset: T,
        weight: f32,
    ) -> Self {
        Self {
            name,
            constraint_type,
            constraint_bone,
            target_bone,
            offset: offset.into(),
            weight,
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

impl From<bool> for ConstraintOffset {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Maintain,
            false => Self::None,
        }
    }
}

impl From<Vector3> for ConstraintOffset {
    fn from(value: Vector3) -> Self {
        Self::Vector3(value)
    }
}

impl From<Quaternion> for ConstraintOffset {
    fn from(value: Quaternion) -> Self {
        Self::Quaternion(value)
    }
}
