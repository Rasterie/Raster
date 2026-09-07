use std::f32::consts::{PI, TAU};

/// Wraps an angle into `(-π, π]`.
///
/// Angles accumulate: a spinning object's rotation grows without bound, and
/// comparing two such angles fails once they differ by a full turn. Wrapping
/// keeps them comparable.
#[inline]
#[must_use]
pub fn wrap_angle(radians: f32) -> f32 {
    let wrapped = radians.rem_euclid(TAU);
    if wrapped > PI { wrapped - TAU } else { wrapped }
}

/// The shortest signed rotation from `from` to `to`, in `(-π, π]`.
///
/// This is what makes a turret turn the short way round: interpolating from
/// 350° to 10° should cross 0°, not wind backwards through 180°.
#[inline]
#[must_use]
pub fn angle_delta(from: f32, to: f32) -> f32 {
    wrap_angle(to - from)
}

/// Interpolates along the shortest path between two angles.
#[inline]
#[must_use]
pub fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    from + angle_delta(from, to) * t
}

/// Moves `from` towards `to` by at most `max_delta`, taking the shortest path.
///
/// Unlike an interpolation, this advances by a fixed amount per call, which is
/// what a turn-rate limit needs: the object turns at a constant speed rather
/// than easing in.
#[inline]
#[must_use]
pub fn rotate_towards(from: f32, to: f32, max_delta: f32) -> f32 {
    let delta = angle_delta(from, to);
    if delta.abs() <= max_delta {
        to
    } else {
        from + delta.signum() * max_delta
    }
}

#[inline]
#[must_use]
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * (PI / 180.0)
}

#[inline]
#[must_use]
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * (180.0 / PI)
}
