use raster_math::Vec2;
use std::f32::consts::{FRAC_PI_2, PI};

const EPS: f32 = 1e-5;

#[test]
fn arithmetic_is_component_wise() {
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::new(1.0, 2.0);

    assert_eq!(a + b, Vec2::new(4.0, 6.0));
    assert_eq!(a - b, Vec2::new(2.0, 2.0));
    assert_eq!(a * b, Vec2::new(3.0, 8.0));
    assert_eq!(a / b, Vec2::new(3.0, 2.0));
    assert_eq!(-a, Vec2::new(-3.0, -4.0));
}

#[test]
fn scalar_multiplication_commutes() {
    let v = Vec2::new(3.0, 4.0);
    assert_eq!(v * 2.0, 2.0 * v);
}

#[test]
fn assignment_operators_match_their_binary_forms() {
    let mut v = Vec2::new(1.0, 2.0);
    v += Vec2::new(3.0, 4.0);
    assert_eq!(v, Vec2::new(4.0, 6.0));

    v -= Vec2::new(1.0, 1.0);
    assert_eq!(v, Vec2::new(3.0, 5.0));

    v *= 2.0;
    assert_eq!(v, Vec2::new(6.0, 10.0));

    v /= 2.0;
    assert_eq!(v, Vec2::new(3.0, 5.0));
}

#[test]
fn length_of_a_three_four_triangle() {
    let v = Vec2::new(3.0, 4.0);
    assert!((v.length() - 5.0).abs() < EPS);
    assert!((v.length_squared() - 25.0).abs() < EPS);
}

#[test]
fn distance_is_symmetric() {
    let a = Vec2::new(1.0, 1.0);
    let b = Vec2::new(4.0, 5.0);
    assert!((a.distance(b) - 5.0).abs() < EPS);
    assert!((a.distance(b) - b.distance(a)).abs() < EPS);
}

#[test]
fn normalizing_yields_unit_length() {
    let v = Vec2::new(3.0, 4.0).normalized();
    assert!((v.length() - 1.0).abs() < EPS);
    assert!(v.is_normalized());
}

/// A zero-length vector has no direction. Returning zero rather than NaN keeps
/// a degenerate case from poisoning every later computation.
#[test]
fn normalizing_a_zero_vector_yields_zero_not_nan() {
    assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    assert!(!Vec2::ZERO.normalized().is_nan());
}

#[test]
fn clamp_length_shortens_only_when_longer() {
    let long = Vec2::new(3.0, 4.0);
    assert!((long.clamp_length(2.5).length() - 2.5).abs() < EPS);

    let short = Vec2::new(0.3, 0.4);
    assert_eq!(short.clamp_length(10.0), short);
}

#[test]
fn dot_of_perpendicular_vectors_is_zero() {
    assert!(Vec2::X.dot(Vec2::Y).abs() < EPS);
    assert!((Vec2::X.dot(Vec2::X) - 1.0).abs() < EPS);
}

#[test]
fn cross_is_antisymmetric() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    assert!((a.cross(b) + b.cross(a)).abs() < EPS);
}

#[test]
fn rotating_by_a_quarter_turn_maps_x_to_y() {
    let r = Vec2::X.rotated(FRAC_PI_2);
    assert!(r.approx_eq(Vec2::Y, EPS), "got {r}");
}

#[test]
fn rotating_full_circle_returns_to_start() {
    let v = Vec2::new(3.0, 4.0);
    assert!(v.rotated(2.0 * PI).approx_eq(v, 1e-4));
}

#[test]
fn rotation_preserves_length() {
    let v = Vec2::new(3.0, 4.0);
    for i in 0..16 {
        let rotated = v.rotated(i as f32 * PI / 8.0);
        assert!((rotated.length() - v.length()).abs() < EPS);
    }
}

/// `perpendicular` exists to be exact where `rotated(FRAC_PI_2)` accumulates
/// floating-point error.
#[test]
fn perpendicular_is_exact() {
    assert_eq!(Vec2::X.perpendicular(), Vec2::Y);
    assert_eq!(Vec2::new(3.0, 4.0).perpendicular(), Vec2::new(-4.0, 3.0));
}

#[test]
fn perpendicular_is_orthogonal() {
    let v = Vec2::new(3.0, 4.0);
    assert!(v.dot(v.perpendicular()).abs() < EPS);
}

#[test]
fn angle_round_trips_through_from_angle() {
    for i in 0..8 {
        let a = i as f32 * PI / 4.0 - PI + 0.1;
        assert!((Vec2::from_angle(a).angle() - a).abs() < EPS, "angle {a}");
    }
}

#[test]
fn angle_to_is_signed() {
    assert!((Vec2::X.angle_to(Vec2::Y) - FRAC_PI_2).abs() < EPS);
    assert!((Vec2::Y.angle_to(Vec2::X) + FRAC_PI_2).abs() < EPS);
}

#[test]
fn lerp_hits_both_ends_and_the_middle() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 20.0);

    assert_eq!(a.lerp(b, 0.0), a);
    assert_eq!(a.lerp(b, 1.0), b);
    assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 10.0));
}

#[test]
fn projecting_onto_an_axis_keeps_only_that_component() {
    let v = Vec2::new(3.0, 4.0);
    assert!(v.project_onto(Vec2::X).approx_eq(Vec2::new(3.0, 0.0), EPS));
    assert_eq!(v.project_onto(Vec2::ZERO), Vec2::ZERO);
}

#[test]
fn reflecting_off_a_floor_inverts_vertical_motion() {
    let velocity = Vec2::new(1.0, 1.0);
    let reflected = velocity.reflect(Vec2::UP);
    assert!(
        reflected.approx_eq(Vec2::new(1.0, -1.0), EPS),
        "got {reflected}"
    );
}

#[test]
fn min_max_and_clamp_are_component_wise() {
    let a = Vec2::new(1.0, 5.0);
    let b = Vec2::new(3.0, 2.0);

    assert_eq!(a.min(b), Vec2::new(1.0, 2.0));
    assert_eq!(a.max(b), Vec2::new(3.0, 5.0));

    let v = Vec2::new(-1.0, 10.0);
    assert_eq!(v.clamp(Vec2::ZERO, Vec2::splat(5.0)), Vec2::new(0.0, 5.0));
}

#[test]
fn rounding_helpers_agree_with_f32() {
    let v = Vec2::new(1.7, -2.3);
    assert_eq!(v.floor(), Vec2::new(1.0, -3.0));
    assert_eq!(v.ceil(), Vec2::new(2.0, -2.0));
    assert_eq!(v.round(), Vec2::new(2.0, -2.0));
}

/// Snapping is what puts pixel art on the grid, so half-way values must round
/// away from zero rather than to even.
#[test]
fn snap_rounds_halves_away_from_zero() {
    assert_eq!(Vec2::new(0.5, 1.5).snap(), Vec2::new(1.0, 2.0));
    assert_eq!(Vec2::new(-0.5, -1.5).snap(), Vec2::new(-1.0, -2.0));
}

#[test]
fn up_is_negative_y_in_screen_space() {
    assert_eq!(Vec2::UP, Vec2::new(0.0, -1.0));
    assert_eq!(Vec2::DOWN, Vec2::new(0.0, 1.0));
}

#[test]
fn nan_and_infinity_are_detected() {
    assert!(Vec2::new(f32::NAN, 0.0).is_nan());
    assert!(!Vec2::new(f32::INFINITY, 0.0).is_nan());
    assert!(!Vec2::new(f32::INFINITY, 0.0).is_finite());
    assert!(Vec2::new(1.0, 2.0).is_finite());
}

#[test]
fn conversions_round_trip() {
    let v = Vec2::new(3.0, 4.0);
    assert_eq!(Vec2::from((3.0, 4.0)), v);
    assert_eq!(Vec2::from([3.0, 4.0]), v);
    assert_eq!(v.to_array(), [3.0, 4.0]);
}

#[test]
fn display_is_readable() {
    assert_eq!(Vec2::new(1.5, -2.0).to_string(), "(1.5, -2)");
}
