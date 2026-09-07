//! Easing functions, for animation and interpolation.
//!
//! Every function maps `[0, 1]` to `[0, 1]`, with `f(0) == 0` and `f(1) == 1`.
//! Input outside that range is clamped, so an accumulator that overshoots its
//! duration cannot produce a value beyond the target.

use std::f32::consts::PI;

/// The shape of an easing curve.
///
/// `In` starts slow, `Out` ends slow, `InOut` does both. `Out` is the right
/// default for anything responding to input: it moves immediately, which reads
/// as responsive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Easing {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SineIn,
    SineOut,
    SineInOut,
    ExpoIn,
    ExpoOut,
    BackIn,
    BackOut,
    ElasticOut,
    BounceOut,
}

impl Easing {
    /// Applies the curve to `t`, clamping it to `[0, 1]` first.
    #[must_use]
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::QuadIn => t * t,
            Self::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::CubicIn => t * t * t,
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::SineIn => 1.0 - ((t * PI) / 2.0).cos(),
            Self::SineOut => ((t * PI) / 2.0).sin(),
            Self::SineInOut => -((PI * t).cos() - 1.0) / 2.0,
            Self::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0f32).powf(10.0 * t - 10.0)
                }
            }
            Self::ExpoOut => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0f32).powf(-10.0 * t)
                }
            }
            // The overshoot constant is the conventional one from Penner's
            // easing equations; it gives about 10% past the target.
            Self::BackIn => {
                const C: f32 = 1.70158;
                (C + 1.0) * t * t * t - C * t * t
            }
            Self::BackOut => {
                const C: f32 = 1.70158;
                let u = t - 1.0;
                1.0 + (C + 1.0) * u * u * u + C * u * u
            }
            Self::ElasticOut => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    const P: f32 = 2.0 * PI / 3.0;
                    (2.0f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * P).sin() + 1.0
                }
            }
            Self::BounceOut => bounce_out(t),
        }
    }
}

fn bounce_out(t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;

    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        let t = t - 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        let t = t - 2.25 / D;
        N * t * t + 0.9375
    } else {
        let t = t - 2.625 / D;
        N * t * t + 0.984375
    }
}

/// Linear interpolation, unclamped so it can extrapolate deliberately.
#[inline]
#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The inverse of [`lerp`]: where `value` sits between `a` and `b`, as a
/// fraction. Returns 0 when `a` and `b` are equal, since no position is
/// meaningful on a zero-length range.
#[inline]
#[must_use]
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Maps a value from one range to another.
#[inline]
#[must_use]
pub fn remap(value: f32, from: (f32, f32), to: (f32, f32)) -> f32 {
    lerp(to.0, to.1, inverse_lerp(from.0, from.1, value))
}

/// Hermite interpolation between two edges: a smooth S-curve, zero below
/// `edge0` and one above `edge1`.
#[inline]
#[must_use]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = inverse_lerp(edge0, edge1, x).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Moves `current` towards `target` by at most `max_delta`.
#[inline]
#[must_use]
pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}
