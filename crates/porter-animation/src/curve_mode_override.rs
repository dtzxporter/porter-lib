use crate::CurveDataType;

/// An override for a curve's mode and any children.
#[derive(Debug, Clone)]
pub struct CurveModeOverride {
    pub name: String,
    pub data_type: CurveDataType,
    pub override_translate: bool,
    pub override_rotation: bool,
    pub override_scale: bool,
}

impl CurveModeOverride {
    /// Creates a new curve mode override with the given name and data_type.
    pub fn new<N: Into<String>>(name: N, data_type: CurveDataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            override_translate: false,
            override_rotation: false,
            override_scale: false,
        }
    }

    /// Sets whether or not this override applies to translate curves.
    pub const fn override_translate(mut self, value: bool) -> Self {
        self.override_translate = value;
        self
    }

    /// Sets whether or not this override applies to rotation curves.
    pub const fn override_rotation(mut self, value: bool) -> Self {
        self.override_rotation = value;
        self
    }

    /// Sets whether or not this override applies to scale curves.
    pub const fn override_scale(mut self, value: bool) -> Self {
        self.override_scale = value;
        self
    }
}
