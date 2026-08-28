use porter_math::Angles;
use porter_math::Matrix3x3;
use porter_math::Quaternion;
use porter_math::Vector3;

#[test]
fn matrix3x3_trs() {
    let rotation = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0), Angles::Degrees);
    let scale = Vector3::new(2.0, 2.0, 2.0);

    let mul = Matrix3x3::create_rotation(rotation) * Matrix3x3::create_scale(scale);

    let create = Matrix3x3::create_rs(rotation, scale);

    assert_eq!(mul, create);
}

#[test]
fn matrix3x3_decompose() {
    let rotation = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0), Angles::Degrees);
    let scale = Vector3::new(2.0, 2.0, 2.0);

    let matrix = Matrix3x3::create_rs(rotation, scale);

    let (drot, dsca) = matrix.decompose();

    assert_eq!(rotation, drot);
    assert_eq!(scale, dsca);
}

#[test]
fn matrix3x3_to_euler() {
    let euler = Vector3::new(0.0, 90.0, 0.0);
    let rotation = Quaternion::from_euler(euler, Angles::Degrees);

    let matrix = Matrix3x3::create_rotation(rotation);

    let to_euler = matrix.to_euler(Angles::Degrees);

    assert_eq!(euler, to_euler);
}
