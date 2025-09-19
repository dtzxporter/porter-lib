use porter_math::Angles;
use porter_math::Quaternion;
use porter_math::Vector3;
use porter_math::degrees_to_radians;

/// A 2 bone ik solver that supports pole vectors and pole twist.
#[derive(Debug)]
pub struct IkSolver {
    start_joint: Vector3,
    mid_joint: Vector3,
    end_joint: Vector3,
    handle: Vector3,
    pole_vector: Vector3,
    twist: f32,
}

impl IkSolver {
    /// Constructs a new ik sovler.
    pub fn new() -> Self {
        Self {
            start_joint: Vector3::zero(),
            mid_joint: Vector3::zero(),
            end_joint: Vector3::zero(),
            handle: Vector3::zero(),
            pole_vector: Vector3::zero(),
            twist: 0.0,
        }
    }

    /// Sets the start joint position in world space.
    pub fn set_start_joint(&mut self, position: Vector3) {
        self.start_joint = position;
    }

    /// Sets the mid joint position in world space.
    pub fn set_mid_joint(&mut self, position: Vector3) {
        self.mid_joint = position;
    }

    /// Sets the end joint, or end effector position in world space.
    pub fn set_end_joint(&mut self, position: Vector3) {
        self.end_joint = position;
    }

    /// Sets the handle position in world space.
    pub fn set_handle(&mut self, position: Vector3) {
        self.handle = position;
    }

    /// Sets the pole vector position in world space.
    pub fn set_pole_vector(&mut self, position: Vector3) {
        self.pole_vector = position;
    }

    /// Sets the twist angle used to rotate the joint chain.
    pub fn set_twist(&mut self, twist: f32, measurment: Angles) {
        match measurment {
            Angles::Degrees => self.twist = degrees_to_radians(twist),
            Angles::Radians => self.twist = twist,
        }
    }

    /// Solves for the current ik state, and returns the world space rotation for `start_joint` and `mid_joint`.
    pub fn solve(&self) -> (Quaternion, Quaternion) {
        let vector1 = self.mid_joint - self.start_joint;
        let vector2 = self.end_joint - self.mid_joint;
        let vectorh = self.handle - self.start_joint;
        let vectore = self.end_joint - self.start_joint;

        let length1 = vector1.length();
        let length2 = vector2.length();
        let lengthh = vectorh.length();

        let vector0 = vector1 - vectore * (vector1.dot(vectore) / vectore.length_squared());

        let vector_angle12 = vector1.normalized().angle_between(vector2.normalized());
        let vector_cross12 = vector1.cross(vector2);

        let lengthh_squared = lengthh * lengthh;

        let theta = ((lengthh_squared - length1 * length1 - length2 * length2)
            / (2.0 * length1 * length2))
            .clamp(-1.0, 1.0)
            .acos();

        let q12 = if vector_cross12.length_squared() > f32::EPSILON {
            let vector_cross12_norm = vector_cross12.normalized();
            let vector_cross12_angle = theta - vector_angle12;

            Quaternion::from_axis_rotation(
                vector_cross12_norm,
                vector_cross12_angle,
                Angles::Radians,
            )
        } else {
            Quaternion::identity()
        };

        let vector2 = vector2.rotate(q12);
        let vectore = vector1 + vector2;

        let vectore_norm = vectore.normalized();
        let vectorh_norm = vectorh.normalized();

        let qeh = Quaternion::from_rotation_arc(vectore_norm, vectorh_norm);

        let vector1 = vector1.rotate(qeh);
        let vector1_norm = vector1.normalized();

        let vector1 = if vector1_norm.is_parallel(vectorh_norm) {
            vector0.rotate(qeh)
        } else {
            vector1
        };

        let pole_vector = self.pole_vector;
        let pole_vector_norm = pole_vector.normalized();

        let qnp = if !pole_vector_norm.is_parallel(vectorh_norm) && lengthh_squared != 0.0 {
            let vectorn = vector1 - vectorh * ((vector1 * vectorh) / lengthh_squared);
            let vectorp = pole_vector - vectorh * ((pole_vector * vectorh) / lengthh_squared);

            let dotnp = vectorn.dot(vectorp) / (vectorn.length() * vectorp.length());

            if (dotnp + 1.0).abs() < f32::EPSILON {
                Quaternion::from_axis_rotation(vectorh_norm, std::f32::consts::PI, Angles::Radians)
            } else {
                let vectorn_norm = vectorn.normalized();
                let vectorp_norm = vectorp.normalized();

                Quaternion::from_rotation_arc(vectorn_norm, vectorp_norm)
            }
        } else {
            Quaternion::identity()
        };

        let twist = Quaternion::from_axis_rotation(vectorh_norm, self.twist, Angles::Radians);

        let start_quat = twist * qnp * qeh;
        let mid_quat = q12;

        (start_quat, mid_quat)
    }
}

impl Default for IkSolver {
    fn default() -> Self {
        Self::new()
    }
}
