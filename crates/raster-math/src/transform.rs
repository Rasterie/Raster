use crate::{Mat3, Vec2};
use std::fmt;

/// Position, rotation and scale, kept separate.
///
/// This is what gameplay code and the inspector work with: setting a position
/// should not require decomposing a matrix, and reading a rotation should give
/// back the angle that was set. [`Mat3`] is the composed form this converts to
/// when the renderer needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub position: Vec2,
    /// Clockwise, in radians, matching the screen-space convention.
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    #[inline]
    #[must_use]
    pub const fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_rotation(rotation: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            rotation,
            scale: Vec2::ONE,
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_scale(scale: Vec2) -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_mat3(self) -> Mat3 {
        Mat3::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    #[inline]
    #[must_use]
    pub fn transform_point(self, point: Vec2) -> Vec2 {
        self.to_mat3().transform_point(point)
    }

    /// Skips translation, for directions and velocities.
    #[inline]
    #[must_use]
    pub fn transform_vector(self, vector: Vec2) -> Vec2 {
        self.to_mat3().transform_vector(vector)
    }

    /// This transform expressed inside `parent`'s space — the operation that
    /// resolves an attachment chain.
    ///
    /// Scale composes component-wise, which is only correct while the parent's
    /// rotation and the child's non-uniform scale do not interact. Attachments
    /// in a 2D game rarely combine both; a case that needs it should compose
    /// [`Mat3`]s directly.
    #[inline]
    #[must_use]
    pub fn combined_with(self, parent: Self) -> Self {
        Self {
            position: parent.transform_point(self.position),
            rotation: parent.rotation + self.rotation,
            scale: parent.scale * self.scale,
        }
    }

    /// The transform that undoes this one, or `None` when it cannot be
    /// expressed as a `Transform2D`.
    ///
    /// The inverse applies rotation before scale, while a `Transform2D` always
    /// applies scale before rotation. The two orders only agree when the scale
    /// is uniform, so a non-uniform scale has no `Transform2D` inverse — take
    /// [`Transform2D::to_mat3`] and invert that instead, which always works.
    ///
    /// Also `None` when a scale component is zero, since that collapses the
    /// plane and cannot be undone at all.
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        if self.scale.x.abs() < f32::EPSILON || self.scale.y.abs() < f32::EPSILON {
            return None;
        }
        if (self.scale.x - self.scale.y).abs() > f32::EPSILON {
            return None;
        }

        let inv_scale = Vec2::new(1.0 / self.scale.x, 1.0 / self.scale.y);
        let inv_rotation = -self.rotation;

        /*
          La transformation directe applique l'echelle, puis la rotation, puis
          la translation ; l'inverse les defait dans l'ordre oppose. L'echelle
          etant uniforme ici, la remise a l'echelle et la rotation commutent,
          et le resultat reste exprimable en Transform2D.
        */
        let inv_position = (-self.position).rotated(inv_rotation) * inv_scale;

        Some(Self {
            position: inv_position,
            rotation: inv_rotation,
            scale: inv_scale,
        })
    }

    /// Interpolated component-wise. Rotation takes the shortest way round, so
    /// interpolating from 350° to 10° passes through 0° rather than winding
    /// backwards through 180°.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation + crate::angle_delta(self.rotation, other.rotation) * t,
            scale: self.scale.lerp(other.scale, t),
        }
    }

    #[inline]
    #[must_use]
    pub fn approx_eq(self, other: Self, epsilon: f32) -> bool {
        self.position.approx_eq(other.position, epsilon)
            && (self.rotation - other.rotation).abs() <= epsilon
            && self.scale.approx_eq(other.scale, epsilon)
    }
}

impl Default for Transform2D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl From<Transform2D> for Mat3 {
    #[inline]
    fn from(t: Transform2D) -> Self {
        t.to_mat3()
    }
}

impl fmt::Display for Transform2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(pos {} rot {} scale {})",
            self.position, self.rotation, self.scale
        )
    }
}
