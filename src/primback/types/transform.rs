use glam::{EulerRot, Mat4, Quat, Vec3};

/// Represents a 3D affine transformation containing translation, rotation, and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The position vector (position in 3D space).
    pub position: Vec3,
    /// The rotation quaternion.
    pub rotation: Quat,
    /// The scale vector.
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Creates a new `Transform` with the specified position, rotation, and scale.
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates a new `Transform` with the specified position, using default scale and identity rotation.
    pub fn new_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Creates a new `Transform` with the specified rotation, using default position and scale.
    pub fn new_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Creates a new `Transform` with the specified scale, using default position and identity rotation.
    pub fn new_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Sets the position vector (builder pattern).
    pub fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// Sets the rotation quaternion (builder pattern).
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale vector (builder pattern).
    pub fn scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Computes the TRS (Position-Rotation-Scale) model matrix.
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Gets the local right direction (+X) vector in world space.
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Gets the local up direction (+Y) vector in world space.
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Gets the local forward direction (-Z) vector in world space (matching right-handed convention).
    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    /// Transforms a direction vector from local space to world space (ignores translation and scale).
    pub fn to_world_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * direction
    }

    /// Transforms a direction vector from world space to local space (ignores translation and scale).
    pub fn to_local_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation.inverse() * direction
    }

    /// Transforms a point from local space to world space (applies scale, rotation, and position).
    pub fn to_world_point(&self, point: Vec3) -> Vec3 {
        (self.rotation * (point * self.scale)) + self.position
    }

    /// Transforms a point from world space to local space (applies inverse position, rotation, and scale).
    pub fn to_local_point(&self, point: Vec3) -> Vec3 {
        let inv_rot = self.rotation.inverse();
        let translated = point - self.position;
        let rotated = inv_rot * translated;
        rotated / self.scale
    }

    /// Transforms a local child `Transform` into world space relative to this parent `Transform`.
    ///
    /// `parent.to_world_transform(child)` applies the child's local transformation first,
    /// and then applies the parent's world transformation.
    pub fn to_world_transform(&self, child: Self) -> WorldTransform {
        WorldTransform::from(*self) * child
    }

    /// Transforms a world space `WorldTransform` into local space relative to this parent `Transform`.
    pub fn to_local_transform(&self, world: WorldTransform) -> Self {
        let inv_parent = self.to_matrix().inverse();
        let local_matrix = inv_parent * world.matrix;
        let (scale, rotation, position) = local_matrix.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Returns the inverse of this `Transform`.
    ///
    /// `t.inverse().to_world_transform(t)` is approximately the identity `Transform`.
    pub fn inverse(&self) -> Self {
        let (scale, rotation, position) =
            self.to_matrix().inverse().to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates a new `Transform` from Euler angles (yaw, pitch, roll) in YXZ order.
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self {
            rotation: Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll),
            ..Default::default()
        }
    }

    /// Sets the rotation using Euler angles (yaw, pitch, roll) in YXZ order (builder pattern).
    pub fn euler(mut self, yaw: f32, pitch: f32, roll: f32) -> Self {
        self.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        self
    }

    /// Multiplies the current rotation by a rotation defined by Euler angles (yaw, pitch, roll) in YXZ order.
    pub fn rotate_euler(&mut self, yaw: f32, pitch: f32, roll: f32) {
        self.rotation = self.rotation * Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }

    /// Multiplies the current rotation by a rotation around an arbitrary axis.
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotation = self.rotation * Quat::from_axis_angle(axis, angle);
    }
}

/// Represents a global 3D transformation containing translation, rotation, scale, and shear.
/// Internally uses a 4x4 matrix to preserve precise transformations under nested non-uniform scaling and rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldTransform {
    pub matrix: Mat4,
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }
}

impl WorldTransform {
    /// Creates a new `WorldTransform` from a `Mat4`.
    pub fn new(matrix: Mat4) -> Self {
        Self { matrix }
    }

    /// Computes the PRS (Position-Rotation-Scale) values from the matrix.
    /// Note that this is a lossy decomposition if the matrix contains shear/skew.
    pub fn to_transform(&self) -> Transform {
        let (scale, rotation, position) = self.matrix.to_scale_rotation_translation();
        Transform {
            position,
            rotation,
            scale,
        }
    }

    /// Gets the position vector.
    pub fn position(&self) -> Vec3 {
        self.matrix.w_axis.truncate()
    }

    /// Gets the rotation quaternion (approximate polar decomposition if matrix has shear).
    pub fn rotation(&self) -> Quat {
        let (_, rotation, _) = self.matrix.to_scale_rotation_translation();
        rotation
    }

    /// Gets the scale vector (lengths of the column vectors).
    pub fn lossy_scale(&self) -> Vec3 {
        let x = self.matrix.col(0).truncate().length();
        let y = self.matrix.col(1).truncate().length();
        let z = self.matrix.col(2).truncate().length();
        Vec3::new(x, y, z)
    }

    /// Gets the local right direction (+X) vector in world space.
    pub fn right(&self) -> Vec3 {
        self.matrix.transform_vector3(Vec3::X).normalize_or_zero()
    }

    /// Gets the local up direction (+Y) vector in world space.
    pub fn up(&self) -> Vec3 {
        self.matrix.transform_vector3(Vec3::Y).normalize_or_zero()
    }

    /// Gets the local forward direction (-Z) vector in world space (matching right-handed convention).
    pub fn forward(&self) -> Vec3 {
        self.matrix.transform_vector3(-Vec3::Z).normalize_or_zero()
    }

    /// Transforms a direction vector (ignores translation).
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.matrix.transform_vector3(direction)
    }

    /// Transforms a point (applies scale, rotation, translation, and shear).
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix.transform_point3(point)
    }

    /// Transforms a direction vector from world space to local space.
    pub fn transform_direction_local(&self, direction: Vec3) -> Vec3 {
        self.matrix.inverse().transform_vector3(direction)
    }

    /// Transforms a point from world space to local space.
    pub fn transform_point_local(&self, point: Vec3) -> Vec3 {
        self.matrix.inverse().transform_point3(point)
    }

    /// Returns the inverse of this `WorldTransform`.
    pub fn inverse(&self) -> Self {
        Self {
            matrix: self.matrix.inverse(),
        }
    }
}

impl From<Mat4> for WorldTransform {
    fn from(matrix: Mat4) -> Self {
        Self { matrix }
    }
}

impl From<Transform> for WorldTransform {
    fn from(t: Transform) -> Self {
        Self {
            matrix: t.to_matrix(),
        }
    }
}

impl From<WorldTransform> for Mat4 {
    fn from(w: WorldTransform) -> Self {
        w.matrix
    }
}

/// Allows multiplying a `WorldTransform` with a local `Transform` to get a combined `WorldTransform`.
impl std::ops::Mul<Transform> for WorldTransform {
    type Output = WorldTransform;

    fn mul(self, child: Transform) -> Self::Output {
        Self {
            matrix: self.matrix * child.to_matrix(),
        }
    }
}

/// Allows multiplying two `WorldTransform`s.
impl std::ops::Mul<WorldTransform> for WorldTransform {
    type Output = WorldTransform;

    fn mul(self, rhs: WorldTransform) -> Self::Output {
        Self {
            matrix: self.matrix * rhs.matrix,
        }
    }
}

/// Allows multiplying two `Transform`s (`parent * child`) to get a combined `WorldTransform`.
impl std::ops::Mul<Transform> for Transform {
    type Output = WorldTransform;

    fn mul(self, child: Transform) -> Self::Output {
        WorldTransform::from(self) * child
    }
}

/// Allows multiplying and assigning two `Transform`s (`parent *= child`) to combine them.
/// Note: This is lossy as it decomposes the resulting world transform back into a local TRS `Transform`.
impl std::ops::MulAssign<Transform> for Transform {
    fn mul_assign(&mut self, child: Transform) {
        let world = WorldTransform::from(*self) * child;
        *self = world.to_transform();
    }
}
