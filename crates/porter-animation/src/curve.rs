use crate::AnimationError;
use crate::Keyframe;
use crate::KeyframeValue;

/// The attribute of the node a curve is animating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurveAttribute {
    /// Animates the translation of this node as a vector3.
    Translate,
    /// Animates the rotation of this node as a quaternion.
    Rotation,
    /// Animates the scale of this node.
    Scale,
    /// Animates the visibility of this node.
    Visibility,
    /// Animates the node as if it were a notification track.
    Notetrack,
    /// Animates the weight of this node as blend shape key.
    BlendShape,
}

/// Curve data type represents how the data is stored relative to the node's attribute value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurveDataType {
    /// Curve data is the absolute value of the node attribute.
    Absolute,
    /// Curve data is added and blended to the existing value of the node attribute.
    Additive,
    /// Curve data is added to the resting position of the node attribute.
    Relative,
}

/// A curve of an animation animates a node and specific attribute with keyframes.
#[derive(Debug, Clone)]
pub struct Curve {
    name: String,
    attribute: CurveAttribute,
    data_type: CurveDataType,
    keyframes: Vec<Keyframe>,
}

impl Curve {
    /// Creates a new curve with the given name, attribute, and data type.
    pub fn new<N: Into<String>>(
        name: N,
        attribute: CurveAttribute,
        data_type: CurveDataType,
    ) -> Self {
        Self {
            name: name.into(),
            attribute,
            data_type,
            keyframes: Vec::new(),
        }
    }

    /// Returns the name of the node this curve targets.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the attribute of the node this curve targets.
    pub fn attribute(&self) -> CurveAttribute {
        self.attribute
    }

    /// Returns the data type of the keyframes.
    pub fn data_type(&self) -> CurveDataType {
        self.data_type
    }

    /// Sets the data type of the keyframes.
    pub fn set_data_type(&mut self, data_type: CurveDataType) {
        self.data_type = data_type;
    }

    /// Returns the keyframes of this curve.
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Returns the keyframes mutable of this curve.
    pub fn keyframes_mut(&mut self) -> &mut [Keyframe] {
        &mut self.keyframes
    }

    /// Returns the largest frame time in this curve.
    pub fn largest_frame_time(&self) -> u32 {
        let mut result = 0;

        for keyframe in self.keyframes() {
            result = result.max(keyframe.time);
        }

        result
    }

    /// Insert a keyframe at the given time with the given value.
    pub fn insert<T: Into<KeyframeValue>>(&mut self, time: u32, value: T) {
        let value = value.into();

        debug_assert!(match self.attribute {
            CurveAttribute::Translate => matches!(value, KeyframeValue::Vector3(_)),
            CurveAttribute::Rotation => matches!(value, KeyframeValue::Quaternion(_)),
            CurveAttribute::Scale => matches!(value, KeyframeValue::Vector3(_)),
            CurveAttribute::Visibility => matches!(value, KeyframeValue::Bool(_)),
            CurveAttribute::Notetrack => matches!(value, KeyframeValue::None),
            CurveAttribute::BlendShape => matches!(value, KeyframeValue::Float(_)),
        });

        self.keyframes.push(Keyframe { time, value });
    }

    /// Returns the number of keyframes in this curve.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Returns whether or not this curve has any keyframes.
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Tries to reserve capacity for at least `additional` more keyframes to be inserted into the given `Curve`.
    /// The property may reserve more space to speculatively avoid frequent reallocations.
    /// After calling `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if capacity is already sufficient. This method preserves the contents even if an error occurs.
    ///
    /// # Errors
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), AnimationError> {
        self.keyframes
            .try_reserve(additional)
            .map_err(|_| AnimationError::CurveAllocationFailed)
    }

    /// Tries to reserve the minimum capacity for at least `additional` keyframes to be inserted in the given `Curve`.
    /// Unlike `try_reserve`, this will not deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `try_reserve_exact`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests.
    /// Therefore, capacity can not be relied upon to be precisely minimal. Prefer `try_reserve` if future insertions are expected.
    ///
    /// # Errors
    /// If the capacity overflows, or the allocator reports a failure, then an error is returned.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), AnimationError> {
        self.keyframes
            .try_reserve_exact(additional)
            .map_err(|_| AnimationError::CurveAllocationFailed)
    }
}
