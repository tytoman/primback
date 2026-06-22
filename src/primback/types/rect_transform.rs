use glam::{IVec2, Vec2};

/// Horizontal or vertical alignment anchor for UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    #[default]
    Start,
    Center,
    End,
}

/// Transform data structure for 2D UI elements (position, rotation, scale, z_index).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectTransform {
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    pub scale: Vec2,
    pub z_index: f32,
}

impl Default for RectTransform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            z_index: 0.0,
        }
    }
}

impl RectTransform {
    /// Creates a new `RectTransform` with the specified position, rotation, scale, and z_index.
    pub fn new(position: Vec2, rotation: f32, scale: Vec2, z_index: f32) -> Self {
        Self {
            position,
            rotation,
            scale,
            z_index,
        }
    }

    /// Creates a `RectTransform` from the given position.
    pub fn new_position(position: impl Into<Vec2>) -> Self {
        Self {
            position: position.into(),
            ..Default::default()
        }
    }

    /// Creates a `RectTransform` from the given rotation (in radians).
    pub fn new_rotation(rotation: f32) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Creates a `RectTransform` from the given scale.
    pub fn new_scale(scale: impl Into<Vec2>) -> Self {
        Self {
            scale: scale.into(),
            ..Default::default()
        }
    }

    /// Creates a `RectTransform` from the given z_index.
    pub fn new_z_index(z_index: f32) -> Self {
        Self {
            z_index,
            ..Default::default()
        }
    }

    /// Sets the position (builder pattern).
    pub fn position(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }

    /// Sets the rotation (in radians, builder pattern).
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale (builder pattern).
    pub fn scale(mut self, scale: impl Into<Vec2>) -> Self {
        self.scale = scale.into();
        self
    }

    /// Sets the z-index (builder pattern).
    pub fn z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Gets the local right direction (+X) vector in world space.
    pub fn right(&self) -> Vec2 {
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        Vec2::new(cos, sin)
    }

    /// Gets the local up direction (+Y) vector in world space.
    pub fn up(&self) -> Vec2 {
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        Vec2::new(-sin, cos)
    }

    /// Transforms a direction vector from local space to world space (ignores position and scale).
    pub fn to_world_direction(&self, direction: Vec2) -> Vec2 {
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        Vec2::new(
            direction.x * cos - direction.y * sin,
            direction.x * sin + direction.y * cos,
        )
    }

    /// Transforms a direction vector from world space to local space (ignores position and scale).
    pub fn to_local_direction(&self, direction: Vec2) -> Vec2 {
        let cos = (-self.rotation).cos();
        let sin = (-self.rotation).sin();
        Vec2::new(
            direction.x * cos - direction.y * sin,
            direction.x * sin + direction.y * cos,
        )
    }

    /// Transforms a point from local space to world space (applies scale, rotation, and position).
    pub fn to_world_point(&self, point: Vec2) -> Vec2 {
        let scaled = point * self.scale;
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rotated = Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        );
        rotated + self.position
    }

    /// Transforms a point from world space to local space (applies inverse position, rotation, and scale).
    pub fn to_local_point(&self, point: Vec2) -> Vec2 {
        let translated = point - self.position;
        let cos = (-self.rotation).cos();
        let sin = (-self.rotation).sin();
        let rotated = Vec2::new(
            translated.x * cos - translated.y * sin,
            translated.x * sin + translated.y * cos,
        );
        let inv_scale = if self.scale.x != 0.0 && self.scale.y != 0.0 {
            Vec2::ONE / self.scale
        } else {
            Vec2::ZERO
        };
        rotated * inv_scale
    }

    /// Transforms a local child `RectTransform` into world space relative to this parent `RectTransform`.
    pub fn to_world_transform(&self, child: Self) -> Self {
        let position = self.to_world_point(child.position);
        let rotation = self.rotation + child.rotation;
        let scale = self.scale * child.scale;
        Self {
            position,
            rotation,
            scale,
            z_index: self.z_index + child.z_index,
        }
    }

    /// Transforms a world space `RectTransform` into local space relative to this parent `RectTransform`.
    pub fn to_local_transform(&self, world: Self) -> Self {
        self.inverse() * world
    }

    /// Returns the inverse of this `RectTransform`.
    pub fn inverse(&self) -> Self {
        let inv_scale = if self.scale.x != 0.0 && self.scale.y != 0.0 {
            Vec2::ONE / self.scale
        } else {
            Vec2::ZERO
        };
        let inv_rot = -self.rotation;
        let position = self.to_local_point(Vec2::ZERO);
        Self {
            position,
            rotation: inv_rot,
            scale: inv_scale,
            z_index: -self.z_index,
        }
    }
}

impl std::ops::Mul<RectTransform> for RectTransform {
    type Output = RectTransform;

    fn mul(self, child: RectTransform) -> Self::Output {
        self.to_world_transform(child)
    }
}

impl std::ops::MulAssign<RectTransform> for RectTransform {
    fn mul_assign(&mut self, child: RectTransform) {
        *self = *self * child;
    }
}

impl From<Vec2> for RectTransform {
    fn from(v: Vec2) -> Self {
        RectTransform::new_position(v)
    }
}

impl From<IVec2> for RectTransform {
    fn from(v: IVec2) -> Self {
        RectTransform::new_position(v.as_vec2())
    }
}

impl From<(f32, f32)> for RectTransform {
    fn from(v: (f32, f32)) -> Self {
        RectTransform::new_position(Vec2::new(v.0, v.1))
    }
}

impl From<[f32; 2]> for RectTransform {
    fn from(v: [f32; 2]) -> Self {
        RectTransform::new_position(Vec2::new(v[0], v[1]))
    }
}

impl From<(i32, i32)> for RectTransform {
    fn from(v: (i32, i32)) -> Self {
        RectTransform::new_position(Vec2::new(v.0 as f32, v.1 as f32))
    }
}

impl From<[i32; 2]> for RectTransform {
    fn from(v: [i32; 2]) -> Self {
        RectTransform::new_position(Vec2::new(v[0] as f32, v[1] as f32))
    }
}
