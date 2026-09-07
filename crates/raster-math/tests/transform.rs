use raster_math::{Mat3, Transform2D, Vec2};
use std::f32::consts::{FRAC_PI_2, PI};

const EPS: f32 = 1e-4;

#[test]
fn identity_leaves_points_alone() {
    let p = Vec2::new(3.0, 4.0);
    assert!(Transform2D::IDENTITY.transform_point(p).approx_eq(p, EPS));
    assert!(Mat3::IDENTITY.transform_point(p).approx_eq(p, EPS));
}

#[test]
fn translation_moves_points() {
    let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
    assert!(
        t.transform_point(Vec2::ZERO)
            .approx_eq(Vec2::new(10.0, 20.0), EPS)
    );
}

/// A direction has no position, so translating it must be a no-op. Getting
/// this wrong makes velocities drift by the object's position.
#[test]
fn translation_does_not_affect_vectors() {
    let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
    let v = Vec2::new(1.0, 0.0);
    assert!(t.transform_vector(v).approx_eq(v, EPS));
}

#[test]
fn rotation_turns_x_towards_y() {
    let t = Transform2D::from_rotation(FRAC_PI_2);
    assert!(t.transform_point(Vec2::X).approx_eq(Vec2::Y, EPS));
}

#[test]
fn scale_multiplies_component_wise() {
    let t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
    assert!(
        t.transform_point(Vec2::ONE)
            .approx_eq(Vec2::new(2.0, 3.0), EPS)
    );
}

/// Scale must apply before rotation, or a rotated object's scale would be
/// measured along the wrong axes.
#[test]
fn scale_applies_before_rotation() {
    let t = Transform2D::new(Vec2::ZERO, FRAC_PI_2, Vec2::new(2.0, 1.0));
    // X is scaled to (2, 0), then rotated a quarter turn onto (0, 2).
    assert!(
        t.transform_point(Vec2::X)
            .approx_eq(Vec2::new(0.0, 2.0), EPS)
    );
}

#[test]
fn inverse_undoes_a_uniformly_scaled_transform() {
    let t = Transform2D::new(Vec2::new(10.0, -5.0), 0.7, Vec2::splat(2.0));
    let inv = t.inverse().expect("invertible");
    let p = Vec2::new(3.0, 4.0);

    let round_trip = inv.transform_point(t.transform_point(p));
    assert!(round_trip.approx_eq(p, EPS), "got {round_trip}");
}

/// The inverse rotates before scaling, while a Transform2D always scales
/// before rotating. Those orders only agree for a uniform scale, so the
/// non-uniform case has no Transform2D inverse and must say so rather than
/// returning a wrong one.
#[test]
fn a_non_uniform_scale_has_no_transform_inverse() {
    let t = Transform2D::new(Vec2::new(10.0, -5.0), 0.7, Vec2::new(2.0, 3.0));
    assert!(t.inverse().is_none());
}

/// The matrix form has no such restriction: it can represent any affine
/// transform, so inverting through Mat3 always works.
#[test]
fn matrix_inverse_handles_a_non_uniform_scale() {
    let m = Transform2D::new(Vec2::new(10.0, -5.0), 0.7, Vec2::new(2.0, 3.0)).to_mat3();
    let inv = m.inverse().expect("invertible");
    let p = Vec2::new(3.0, 4.0);

    let round_trip = inv.transform_point(m.transform_point(p));
    assert!(round_trip.approx_eq(p, EPS), "got {round_trip}");
}

#[test]
fn a_zero_scale_has_no_inverse() {
    assert!(
        Transform2D::from_scale(Vec2::new(0.0, 1.0))
            .inverse()
            .is_none()
    );
    assert!(Mat3::from_scale(Vec2::new(1.0, 0.0)).inverse().is_none());
}

#[test]
fn combining_with_identity_changes_nothing() {
    let t = Transform2D::new(Vec2::new(3.0, 4.0), 0.5, Vec2::new(2.0, 2.0));
    assert!(t.combined_with(Transform2D::IDENTITY).approx_eq(t, EPS));
}

/// Attachment resolution: a child at (1, 0) on a parent rotated a quarter turn
/// and moved to (10, 10) ends up at (10, 11).
#[test]
fn combining_places_a_child_in_its_parent_space() {
    let parent = Transform2D::new(Vec2::new(10.0, 10.0), FRAC_PI_2, Vec2::ONE);
    let child = Transform2D::from_position(Vec2::X);

    let world = child.combined_with(parent);
    assert!(
        world.position.approx_eq(Vec2::new(10.0, 11.0), EPS),
        "got {}",
        world.position
    );
    assert!((world.rotation - FRAC_PI_2).abs() < EPS);
}

#[test]
fn matrix_composition_applies_right_hand_side_first() {
    let translate = Mat3::from_translation(Vec2::new(10.0, 0.0));
    let rotate = Mat3::from_rotation(FRAC_PI_2);

    // Rotate first, then translate.
    let composed = translate * rotate;
    assert!(
        composed
            .transform_point(Vec2::X)
            .approx_eq(Vec2::new(10.0, 1.0), EPS)
    );
}

#[test]
fn determinant_reports_the_area_scale() {
    assert!((Mat3::IDENTITY.determinant() - 1.0).abs() < EPS);
    assert!((Mat3::from_scale(Vec2::splat(2.0)).determinant() - 4.0).abs() < EPS);
    // Rotation preserves area.
    assert!((Mat3::from_rotation(0.7).determinant() - 1.0).abs() < EPS);
}

#[test]
fn transform_converts_to_the_same_matrix() {
    let t = Transform2D::new(Vec2::new(3.0, 4.0), 0.6, Vec2::new(2.0, 1.5));
    let p = Vec2::new(1.0, 2.0);
    assert!(
        t.transform_point(p)
            .approx_eq(t.to_mat3().transform_point(p), EPS)
    );
}

/// Interpolating a rotation must take the short way round: from just under a
/// full turn to just over zero should pass through zero, not unwind backwards.
#[test]
fn lerp_takes_the_shortest_rotation() {
    let a = Transform2D::from_rotation(-0.1);
    let b = Transform2D::from_rotation(0.1);
    let mid = a.lerp(b, 0.5);
    assert!(mid.rotation.abs() < EPS, "got {}", mid.rotation);

    // Across the wrap point, the delta stays small.
    let a = Transform2D::from_rotation(PI - 0.1);
    let b = Transform2D::from_rotation(-PI + 0.1);
    let mid = a.lerp(b, 0.5);
    assert!(
        mid.rotation.abs() > 3.0,
        "should pass through π, got {}",
        mid.rotation
    );
}

#[test]
fn padded_array_is_column_major_with_the_implied_row() {
    let m = Mat3::from_translation(Vec2::new(5.0, 6.0));
    let a = m.to_cols_array_padded();

    assert_eq!(a[0], 1.0);
    assert_eq!(a[5], 1.0);
    assert_eq!(a[8], 5.0);
    assert_eq!(a[9], 6.0);
    assert_eq!(a[10], 1.0);
}
