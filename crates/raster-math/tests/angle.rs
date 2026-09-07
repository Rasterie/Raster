use raster_math::{
    angle_delta, degrees_to_radians, lerp_angle, radians_to_degrees, rotate_towards, wrap_angle,
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const EPS: f32 = 1e-5;

#[test]
fn wrapping_leaves_angles_already_in_range() {
    assert!((wrap_angle(0.0)).abs() < EPS);
    assert!((wrap_angle(1.0) - 1.0).abs() < EPS);
    assert!((wrap_angle(-1.0) + 1.0).abs() < EPS);
}

#[test]
fn wrapping_folds_full_turns_away() {
    assert!(wrap_angle(TAU).abs() < EPS);
    assert!((wrap_angle(TAU + 1.0) - 1.0).abs() < EPS);
    assert!((wrap_angle(-TAU - 1.0) + 1.0).abs() < EPS);
}

#[test]
fn wrapping_always_lands_in_range() {
    for i in -20..20 {
        let a = i as f32 * 1.7;
        let w = wrap_angle(a);
        assert!(w > -PI - EPS && w <= PI + EPS, "{a} wrapped to {w}");
    }
}

/// The point of the whole module: a turret at 350° aiming at 10° must turn
/// 20° forwards, not 340° backwards.
#[test]
fn delta_takes_the_short_way_round() {
    let from = degrees_to_radians(350.0);
    let to = degrees_to_radians(10.0);
    let delta = radians_to_degrees(angle_delta(from, to));

    assert!(
        (delta - 20.0).abs() < 1e-3,
        "expected 20 degrees, got {delta}"
    );
}

#[test]
fn delta_is_signed() {
    assert!(angle_delta(0.0, 1.0) > 0.0);
    assert!(angle_delta(1.0, 0.0) < 0.0);
}

#[test]
fn lerp_angle_hits_both_ends() {
    assert!((lerp_angle(0.5, 1.5, 0.0) - 0.5).abs() < EPS);
    assert!((wrap_angle(lerp_angle(0.5, 1.5, 1.0) - 1.5)).abs() < EPS);
}

#[test]
fn lerp_angle_crosses_zero_rather_than_unwinding() {
    let mid = wrap_angle(lerp_angle(
        degrees_to_radians(350.0),
        degrees_to_radians(10.0),
        0.5,
    ));
    // Halfway between 350 and 10 is 0, not 180.
    assert!(mid.abs() < 1e-3, "got {} degrees", radians_to_degrees(mid));
}

/// A turn-rate limit advances by a fixed amount each call, unlike an
/// interpolation which eases in.
#[test]
fn rotate_towards_advances_by_a_fixed_step() {
    let result = rotate_towards(0.0, 1.0, 0.25);
    assert!((result - 0.25).abs() < EPS);
}

#[test]
fn rotate_towards_snaps_when_within_reach() {
    assert!((rotate_towards(0.0, 0.1, 0.25) - 0.1).abs() < EPS);
}

#[test]
fn rotate_towards_turns_the_short_way() {
    let from = degrees_to_radians(350.0);
    let to = degrees_to_radians(10.0);
    let stepped = rotate_towards(from, to, degrees_to_radians(5.0));

    // Moving forwards from 350 lands on 355, not 345.
    assert!((radians_to_degrees(wrap_angle(stepped)) + 5.0).abs() < 1e-2);
}

#[test]
fn degree_conversion_round_trips() {
    for d in [0.0, 45.0, 90.0, 180.0, -90.0, 359.0] {
        assert!((radians_to_degrees(degrees_to_radians(d)) - d).abs() < 1e-3);
    }
    assert!((degrees_to_radians(90.0) - FRAC_PI_2).abs() < EPS);
}
