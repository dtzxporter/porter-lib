use crate::KnotVector;
use crate::Vector3;

/// A cubic bezier position spline curve.
#[derive(Debug, Clone)]
pub struct Vector3Spline {
    knots: KnotVector,
    keys: Vec<Vector3>,
    tangents_a: Vec<Vector3>,
    tangents_b: Vec<Vector3>,
}

impl Vector3Spline {
    /// Construct a new vector3 spline curve with the given data.
    pub fn new(
        knots: KnotVector,
        keys: Vec<Vector3>,
        tangents_a: Vec<Vector3>,
        tangents_b: Vec<Vector3>,
    ) -> Self {
        debug_assert!(
            tangents_a.len() == tangents_b.len(),
            "Must have equal tangents!"
        );

        debug_assert!(
            keys.len() == knots.len(),
            "Must have the same number of keys as knots!"
        );

        debug_assert!(
            keys.len() > tangents_a.len() && keys.len() > tangents_b.len(),
            "Must have one more key than tangents!"
        );

        Self {
            knots,
            keys,
            tangents_a,
            tangents_b,
        }
    }

    /// Evaluates the curve for the given time so long as the time is within the curve bounds.
    pub fn evaluate(&self, time: f32) -> Option<Vector3> {
        let interval = self.knots.interval(time)?;

        let u = (time - *self.knots.get(interval)?)
            / (*self.knots.get(interval + 1)? - *self.knots.get(interval)?);

        let omu = 1.0 - u;
        let omu2 = omu * omu;
        let u2 = u * u;
        let b0 = omu2 * omu;
        let b1 = 3.0 * u * omu2;
        let b2 = 3.0 * u2 * omu;
        let b3 = u2 * u;

        let xt = (self.keys[interval] * b0)
            + (self.tangents_a[interval] * b1)
            + (self.tangents_b[interval] * b2)
            + (self.keys[interval + 1] * b3);

        Some(xt)
    }
}
