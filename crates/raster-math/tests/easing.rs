use raster_math::{Easing, inverse_lerp, lerp, move_towards, remap, smoothstep};

const EPS: f32 = 1e-4;

const ALL: [Easing; 16] = [
    Easing::Linear,
    Easing::QuadIn,
    Easing::QuadOut,
    Easing::QuadInOut,
    Easing::CubicIn,
    Easing::CubicOut,
    Easing::CubicInOut,
    Easing::SineIn,
    Easing::SineOut,
    Easing::SineInOut,
    Easing::ExpoIn,
    Easing::ExpoOut,
    Easing::BackIn,
    Easing::BackOut,
    Easing::ElasticOut,
    Easing::BounceOut,
];

/// The contract every curve must honour: an animation must start exactly at
/// its origin and land exactly on its target.
#[test]
fn every_curve_maps_zero_to_zero_and_one_to_one() {
    for e in ALL {
        assert!(e.apply(0.0).abs() < EPS, "{e:?} at 0 gave {}", e.apply(0.0));
        assert!(
            (e.apply(1.0) - 1.0).abs() < EPS,
            "{e:?} at 1 gave {}",
            e.apply(1.0)
        );
    }
}

/// An accumulator that overshoots its duration must not push the value past
/// the target.
#[test]
fn input_outside_the_unit_range_is_clamped() {
    for e in ALL {
        assert!(e.apply(-5.0).abs() < EPS, "{e:?} below range");
        assert!((e.apply(5.0) - 1.0).abs() < EPS, "{e:?} above range");
    }
}

#[test]
fn linear_is_the_identity() {
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        assert!((Easing::Linear.apply(t) - t).abs() < EPS);
    }
}

#[test]
fn in_curves_start_slow_and_out_curves_start_fast() {
    // A quarter of the way in, an ease-in has covered less ground than linear,
    // and an ease-out more.
    assert!(Easing::QuadIn.apply(0.25) < 0.25);
    assert!(Easing::QuadOut.apply(0.25) > 0.25);
    assert!(Easing::CubicIn.apply(0.25) < 0.25);
    assert!(Easing::CubicOut.apply(0.25) > 0.25);
}

#[test]
fn in_out_curves_are_symmetric_about_the_middle() {
    for e in [Easing::QuadInOut, Easing::CubicInOut, Easing::SineInOut] {
        assert!((e.apply(0.5) - 0.5).abs() < EPS, "{e:?} midpoint");
        for i in 1..10 {
            let t = i as f32 / 10.0;
            let sum = e.apply(t) + e.apply(1.0 - t);
            assert!((sum - 1.0).abs() < EPS, "{e:?} asymmetric at {t}");
        }
    }
}

#[test]
fn monotonic_curves_never_go_backwards() {
    for e in [
        Easing::Linear,
        Easing::QuadIn,
        Easing::QuadOut,
        Easing::CubicIn,
        Easing::CubicOut,
        Easing::SineIn,
        Easing::SineOut,
        Easing::ExpoIn,
        Easing::ExpoOut,
    ] {
        let mut previous = e.apply(0.0);
        for i in 1..=100 {
            let current = e.apply(i as f32 / 100.0);
            assert!(current >= previous - EPS, "{e:?} decreased at {i}");
            previous = current;
        }
    }
}

/// Back and Elastic are meant to overshoot — that is their whole purpose.
#[test]
fn back_and_elastic_overshoot() {
    assert!(
        Easing::BackIn.apply(0.3) < 0.0,
        "BackIn should dip below zero"
    );
    assert!(
        Easing::BackOut.apply(0.7) > 1.0,
        "BackOut should exceed one"
    );

    let peak = (1..100)
        .map(|i| Easing::ElasticOut.apply(i as f32 / 100.0))
        .fold(0.0, f32::max);
    assert!(peak > 1.0, "ElasticOut should exceed one, peaked at {peak}");
}

#[test]
fn lerp_and_its_inverse_round_trip() {
    let (a, b) = (10.0, 20.0);
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        assert!((inverse_lerp(a, b, lerp(a, b, t)) - t).abs() < EPS);
    }
}

/// A zero-length range has no meaningful position, so it reports zero rather
/// than dividing by zero.
#[test]
fn inverse_lerp_of_an_empty_range_is_zero() {
    assert_eq!(inverse_lerp(5.0, 5.0, 5.0), 0.0);
    assert!(inverse_lerp(5.0, 5.0, 100.0).is_finite());
}

#[test]
fn remap_moves_between_ranges() {
    assert!((remap(5.0, (0.0, 10.0), (0.0, 100.0)) - 50.0).abs() < EPS);
    assert!((remap(0.0, (-1.0, 1.0), (0.0, 1.0)) - 0.5).abs() < EPS);
}

#[test]
fn smoothstep_saturates_outside_its_edges() {
    assert!(smoothstep(0.0, 1.0, -1.0).abs() < EPS);
    assert!((smoothstep(0.0, 1.0, 2.0) - 1.0).abs() < EPS);
    assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < EPS);
}

#[test]
fn move_towards_steps_then_snaps() {
    assert!((move_towards(0.0, 10.0, 3.0) - 3.0).abs() < EPS);
    assert!((move_towards(0.0, 1.0, 5.0) - 1.0).abs() < EPS);
    assert!((move_towards(10.0, 0.0, 3.0) - 7.0).abs() < EPS);
}
