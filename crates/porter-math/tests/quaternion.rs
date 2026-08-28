use porter_math::Angles;
use porter_math::Quaternion;
use porter_math::Vector3;

#[test]
fn quaternion_inverse() {
    let euler = Vector3::new(0.0, 90.0, 0.0);
    let quaternion = Quaternion::from_euler(euler, Angles::Degrees);

    let inverse = quaternion.inverse();

    let mul = quaternion * inverse;

    assert_eq!(mul, Quaternion::identity())
}

#[test]
fn quaternion_slerp() {
    let euler = Vector3::zero();
    let euler2 = Vector3::new(0.0, 180.0, 0.0);

    let quat = Quaternion::from_euler(euler, Angles::Degrees);
    let quat2 = Quaternion::from_euler(euler2, Angles::Degrees);

    let slerp = quat.slerp(quat2, 0.5);

    assert_eq!(slerp, Quaternion::new(0.0, -0.70710677, 0.0, 0.7071068))
}

#[test]
fn quaternion_to_euler() {
    let euler = Vector3::new(0.0, 90.0, 0.0);
    let quaternion = Quaternion::from_euler(euler, Angles::Degrees);

    let to_euler = quaternion.to_euler(Angles::Degrees);

    assert_eq!(euler, to_euler);
}
