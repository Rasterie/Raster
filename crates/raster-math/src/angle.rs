use std::f32::consts::{PI, TAU};

/// Ramene un angle dans `(-π, π]` : une rotation qui s'accumule devient sinon
/// incomparable des qu'elle depasse un tour.
#[inline]
#[must_use]
pub fn wrap_angle(radians: f32) -> f32 {
    let wrapped = radians.rem_euclid(TAU);
    if wrapped > PI { wrapped - TAU } else { wrapped }
}

/// La rotation signee la plus courte de `from` a `to` : de 350° a 10° on passe
/// par 0°, pas par 180°.
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

/// Avance de `max_delta` au plus vers `to` : une vitesse de rotation
/// constante, la ou une interpolation ralentirait en approchant.
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
