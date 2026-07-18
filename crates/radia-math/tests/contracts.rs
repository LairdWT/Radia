use radia_math::{ErrorScale, MathError, ReverseZPerspective, UnitDualQuat, UnitQuat, Vec3};

fn scale() -> ErrorScale {
    ErrorScale::new(100.0, 64).expect("test scale is valid")
}

fn near(left: f32, right: f32, absolute: f32, relative: f32) -> bool {
    (left - right).abs() <= absolute + relative * left.abs().max(right.abs())
}

fn near_vec(left: Vec3, right: Vec3, tolerance: f32) -> bool {
    near(left.x, right.x, tolerance, tolerance)
        && near(left.y, right.y, tolerance, tolerance)
        && near(left.z, right.z, tolerance, tolerance)
}

fn near_slice(left: &[f32], right: &[f32], tolerance: f32) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left_value, right_value)| near(*left_value, *right_value, tolerance, tolerance))
}

#[test]
fn vector_normalization_preserves_direction_and_rejects_bad_domains() {
    let source = Vec3::new(3.0, 4.0, 0.0);
    let normalized = source.try_normalize(scale()).expect("vector is valid");
    assert!(near(normalized.length(), 1.0, 2.0e-6, 2.0e-6));
    assert!(near_vec(normalized * 5.0, source, 2.0e-5));
    assert_eq!(
        Vec3::ZERO.try_normalize(scale()),
        Err(MathError::Degenerate("vector"))
    );
    assert_eq!(
        Vec3::new(f32::NAN, 0.0, 0.0).try_normalize(scale()),
        Err(MathError::NonFinite("vector"))
    );
}

#[test]
fn quaternion_rotation_composition_and_inverse_match_declared_order() {
    let rotate_y = UnitQuat::try_from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2, scale())
        .expect("axis-angle is valid");
    let rotate_x = UnitQuat::try_from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2, scale())
        .expect("axis-angle is valid");
    let source = Vec3::new(1.0, 2.0, 3.0);
    let sequential = rotate_y.rotate_vector(rotate_x.rotate_vector(source));
    let composed = rotate_y
        .compose(rotate_x, scale())
        .expect("composition remains valid")
        .rotate_vector(source);
    assert!(near_vec(sequential, composed, 4.0e-5));
    assert!(near_vec(
        rotate_y
            .conjugate()
            .rotate_vector(rotate_y.rotate_vector(source)),
        source,
        4.0e-5
    ));
    assert!(near(
        rotate_y.rotate_vector(source).length(),
        source.length(),
        4.0e-5,
        4.0e-5
    ));
}

#[test]
fn dual_quaternion_pose_obeys_study_inverse_and_composition_contracts() {
    let rotate_y = UnitQuat::try_from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2, scale())
        .expect("axis-angle is valid");
    let pose =
        UnitDualQuat::from_rotation_translation(rotate_y, Vec3::new(4.0, -2.0, 7.0), scale())
            .expect("pose is valid");
    assert!(pose.study_dot().abs() <= 8.0 * f32::EPSILON);

    let source = Vec3::new(2.0, 1.0, -3.0);
    let restored = pose.inverse().transform_point(pose.transform_point(source));
    assert!(near_vec(restored, source, 8.0e-5));

    let translation = UnitDualQuat::from_rotation_translation(
        UnitQuat::identity(),
        Vec3::new(1.0, 2.0, 3.0),
        scale(),
    )
    .expect("translation pose is valid");
    let composed = pose
        .compose(translation, scale())
        .expect("pose composition remains valid");
    assert!(near_vec(
        composed.transform_point(source),
        pose.transform_point(translation.transform_point(source)),
        8.0e-5
    ));
}

#[test]
fn quaternion_antipodes_and_gpu_component_adapter_are_explicit() {
    let positive =
        UnitQuat::try_from_wxyz(0.5, 0.5, 0.5, 0.5, scale()).expect("quaternion is valid");
    let negative =
        UnitQuat::try_from_wxyz(-0.5, -0.5, -0.5, -0.5, scale()).expect("quaternion is valid");
    assert!(near(
        positive.orientation_dot(negative).abs(),
        1.0,
        2.0e-6,
        2.0e-6
    ));
    assert!(near_vec(
        positive.rotate_vector(Vec3::X),
        negative.rotate_vector(Vec3::X),
        2.0e-6
    ));
    assert!(near_slice(
        &positive.to_wxyz(),
        &[0.5, 0.5, 0.5, 0.5],
        2.0e-6
    ));
    assert!(near_slice(
        &positive.to_gpu_xyzw(),
        &[0.5, 0.5, 0.5, 0.5],
        2.0e-6
    ));

    let pose = UnitDualQuat::from_rotation_translation(
        UnitQuat::identity(),
        Vec3::new(2.0, 4.0, 6.0),
        scale(),
    )
    .expect("pose is valid");
    assert!(near_slice(
        &pose.to_gpu_xyzw(),
        &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 0.0],
        2.0e-6
    ));
}

#[test]
fn reverse_z_projection_hits_boundaries_without_matrix_state() {
    let projection = ReverseZPerspective::new(std::f32::consts::FRAC_PI_2, 16.0 / 9.0, 0.25)
        .expect("projection is valid");
    let near_clip = projection
        .project_camera(Vec3::new(0.0, 0.0, -0.25))
        .expect("near point is visible")
        .ndc();
    let far_clip = projection
        .project_camera(Vec3::new(0.0, 0.0, -100_000.0))
        .expect("far point is visible")
        .ndc();
    assert!(near(near_clip.z, 1.0, 2.0e-6, 2.0e-6));
    assert!(far_clip.z > 0.0 && far_clip.z < 0.000_01);
    assert_eq!(
        projection.project_camera(Vec3::new(0.0, 0.0, 0.0)),
        Err(MathError::OutsideProjectionDomain)
    );
}

#[test]
fn screen_ray_and_projection_round_trip_at_pixel_center() {
    let projection = ReverseZPerspective::new(std::f32::consts::FRAC_PI_2, 1.0, 0.1)
        .expect("projection is valid");
    let camera = UnitDualQuat::from_rotation_translation(
        UnitQuat::identity(),
        Vec3::new(5.0, 2.0, 3.0),
        scale(),
    )
    .expect("camera pose is valid");
    let ray = projection
        .screen_ray_for_pixel(camera, 31, 47, 128, 128, scale())
        .expect("pixel is valid");
    let world_point = ray.origin + ray.direction * 12.0;
    let screen = projection
        .project_world(camera, world_point, 128, 128)
        .expect("ray point is visible");
    assert!(near(screen.position.x, 31.5, 3.0e-4, 3.0e-4));
    assert!(near(screen.position.y, 47.5, 3.0e-4, 3.0e-4));
    assert!(screen.depth > 0.0 && screen.depth <= 1.0);
}

#[test]
fn constructors_refuse_non_finite_and_degenerate_inputs() {
    assert_eq!(
        UnitQuat::try_from_wxyz(0.0, 0.0, 0.0, 0.0, scale()),
        Err(MathError::Degenerate("quaternion"))
    );
    assert_eq!(
        UnitQuat::try_from_axis_angle(Vec3::X, f32::INFINITY, scale()),
        Err(MathError::NonFinite("angle"))
    );
    assert!(ReverseZPerspective::new(0.0, 1.0, 0.1).is_err());
    assert!(ReverseZPerspective::new(1.0, 0.0, 0.1).is_err());
    assert!(ReverseZPerspective::new(1.0, 1.0, f32::NAN).is_err());
}
