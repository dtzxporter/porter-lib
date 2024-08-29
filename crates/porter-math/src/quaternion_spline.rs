use crate::KnotVector;
use crate::Quaternion;
use crate::Vector3;

/// A span of a quaternion spline curve.
#[derive(Debug, Clone, Copy)]
struct QuaternionSplineSpan {
    q0: Quaternion,
    w1: Vector3,
    w2: Vector3,
    w3: Vector3,
    ka: f32,
    kb: f32,
}

/// A cubic bezier quaternion spline curve.
#[derive(Debug, Clone)]
pub struct QuaternionSpline {
    knots: KnotVector,
    keys: Vec<Quaternion>,
    vels_a: Vec<Vector3>,
    vels_b: Vec<Vector3>,
}

impl QuaternionSpline {
    /// Construct a new quaternion spline curve with the given data.
    pub fn new(
        knots: KnotVector,
        keys: Vec<Quaternion>,
        vels_a: Vec<Vector3>,
        vels_b: Vec<Vector3>,
    ) -> Self {
        debug_assert!(vels_a.len() == vels_b.len(), "Must have equal velocities!");

        debug_assert!(
            keys.len() == knots.len(),
            "Must have the same number of keys as knots!"
        );

        debug_assert!(
            keys.len() > vels_a.len() && keys.len() > vels_b.len(),
            "Must have one more key than velocities!"
        );

        Self {
            knots,
            keys,
            vels_a,
            vels_b,
        }
    }

    /// Evaluates the curve for the given time so long as the time is within the curve bounds.
    pub fn evaluate(&self, time: f32) -> Option<Quaternion> {
        let span = self.get_span(time)?;

        let u = (time - span.ka) / (span.kb - span.ka);

        let omu = 1.0 - u;
        let u2 = u * u;
        let omu2 = omu * omu;
        let c3 = u2 * u;
        let c2 = 3.0 * u2 - 2.0 * c3;
        let c1 = 1.0 - omu2 * omu;

        let e1 = Quaternion::from_log_vector(span.w1 * c1);
        let e2 = Quaternion::from_log_vector(span.w2 * c2);
        let e3 = Quaternion::from_log_vector(span.w3 * c3);

        let qt = span.q0 * (e1 * (e2 * e3));

        Some(qt)
    }

    /// Gets the curve span for the given time.
    fn get_span(&self, time: f32) -> Option<QuaternionSplineSpan> {
        let interval = self.knots.interval(time)?;

        let q0 = self.keys[interval];
        let w1 = self.vels_a[interval];
        let w3 = self.vels_b[interval];

        let dq = Quaternion::from_log_vector(w1);
        let q1 = q0 * dq;

        let dq = !Quaternion::from_log_vector(w3);
        let q2 = self.keys[interval + 1] * dq;

        let q1 = !q1;
        let dq = q1 * q2;
        let w2 = dq.to_log_vector();

        let ka = *self.knots.get(interval)?;
        let kb = *self.knots.get(interval + 1)?;

        Some(QuaternionSplineSpan {
            q0,
            w1,
            w2,
            w3,
            ka,
            kb,
        })
    }
}
