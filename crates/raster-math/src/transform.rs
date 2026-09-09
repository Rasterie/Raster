use crate::{Mat3, Vec2};
use std::fmt;

/// Position, rotation et echelle, gardees separees : lire une rotation doit
/// rendre l'angle qu'on a pose, sans decomposer de matrice.
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

    /// Cette transformation dans l'espace de `parent`.
    ///
    /// L'echelle se compose composante par composante : correct tant que la
    /// rotation du parent et une echelle non uniforme ne se croisent pas.
    #[inline]
    #[must_use]
    pub fn combined_with(self, parent: Self) -> Self {
        Self {
            position: parent.transform_point(self.position),
            rotation: parent.rotation + self.rotation,
            scale: parent.scale * self.scale,
        }
    }

    /// L'inverse, ou `None` s'il n'est pas exprimable ici.
    ///
    /// L'inverse tourne avant de mettre a l'echelle, un `Transform2D` fait
    /// l'inverse : les deux ne coincident qu'a echelle uniforme. Passer par
    /// [`Transform2D::to_mat3`] pour le cas general.
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

        // L'echelle etant uniforme, rotation et mise a l'echelle commutent.
        let inv_position = (-self.position).rotated(inv_rotation) * inv_scale;

        Some(Self {
            position: inv_position,
            rotation: inv_rotation,
            scale: inv_scale,
        })
    }

    /// Interpole composante par composante ; la rotation prend le chemin court.
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
