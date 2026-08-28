use porter_math::Angles;
use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector3;

#[test]
fn matrix4x4_inverse() {
    let position = Vector3::new(4.0, 55.0, 344.0);
    let rotation = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0), Angles::Degrees);
    let scale = Vector3::new(2.0, 2.0, 2.0);

    let matrix = Matrix4x4::create_trs(position, rotation, scale);
    let inverse = matrix.inverse();

    let mul = matrix * inverse;

    assert_eq!(mul, Matrix4x4::new());
}

#[test]
fn matrix4x4_trs() {
    let position = Vector3::new(4.0, 55.0, 344.0);
    let rotation = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0), Angles::Degrees);
    let scale = Vector3::new(2.0, 2.0, 2.0);

    let mul = Matrix4x4::create_position(position)
        * Matrix4x4::create_rotation(rotation)
        * Matrix4x4::create_scale(scale);

    let create = Matrix4x4::create_trs(position, rotation, scale);

    assert_eq!(mul, create);
}

#[test]
fn matrix4x4_decompose() {
    let position = Vector3::new(4.0, 55.0, 344.0);
    let rotation = Quaternion::from_euler(Vector3::new(0.0, 90.0, 0.0), Angles::Degrees);
    let scale = Vector3::new(2.0, 2.0, 2.0);

    let matrix = Matrix4x4::create_trs(position, rotation, scale);

    let (dpos, drot, dsca) = matrix.decompose();

    assert_eq!(position, dpos);
    assert_eq!(rotation, drot);
    assert_eq!(scale, dsca);
}

#[test]
fn matrix4x4_to_euler() {
    let euler = Vector3::new(0.0, 90.0, 0.0);
    let rotation = Quaternion::from_euler(euler, Angles::Degrees);

    let matrix = Matrix4x4::create_rotation(rotation);

    let to_euler = matrix.to_euler(Angles::Degrees);

    assert_eq!(euler, to_euler);
}
