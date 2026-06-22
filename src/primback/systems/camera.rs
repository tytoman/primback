use crate::primback::types::transform::Transform;
use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

/// Perspective camera system managing position, orientation, and projections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub transform: Transform,
    /// Vertical field of view in degrees.
    pub fovy_deg: f32,
    /// Near clipping plane distance.
    pub z_near: f32,
    /// Far clipping plane distance.
    pub z_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            fovy_deg: 45.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
}

impl Camera {
    /// Creates a new default Camera.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a camera positioned at `eye`, looking at `target`, with the specified `up` direction.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let view = Mat4::look_at_rh(eye, target, up);
        // The inverse of the view matrix is the camera's world transform matrix
        let world = view.inverse();
        let (_scale, rotation, position) = world.to_scale_rotation_translation();
        // Since look_at view matrix represents pure rotation and translation,
        // scale should always be 1.0. Explicitly set to Vec3::ONE to avoid floating point error.

        Self {
            transform: Transform::new(position, rotation, Vec3::ONE),
            ..Self::default()
        }
    }

    /// Sets the vertical field of view in degrees.
    pub fn fovy(mut self, fovy_deg: f32) -> Self {
        self.fovy_deg = fovy_deg;
        self
    }

    /// Sets the near clipping plane distance.
    ///
    /// # Note
    /// The value must be greater than 0.
    pub fn near(mut self, z_near: f32) -> Self {
        let mut clamped_near = z_near;
        if z_near <= 0.0 {
            log::warn!("z_near must be greater than 0.0. Clamping to 1e-4. Got: {}", z_near);
            clamped_near = 1e-4;
        }
        if clamped_near >= self.z_far {
            log::warn!("z_near must be less than z_far. Clamping to z_far - 1e-4. Got near: {}, far: {}", clamped_near, self.z_far);
            clamped_near = self.z_far - 1e-4;
            if clamped_near <= 0.0 {
                clamped_near = 1e-4;
                self.z_far = 2e-4;
            }
        }
        self.z_near = clamped_near;
        self
    }

    /// Sets the far clipping plane distance.
    pub fn far(mut self, z_far: f32) -> Self {
        let mut clamped_far = z_far;
        if clamped_far <= self.z_near {
            log::warn!("z_far must be greater than z_near. Clamping to z_near + 1e-4. Got far: {}, near: {}", clamped_far, self.z_near);
            clamped_far = self.z_near + 1e-4;
        }
        self.z_far = clamped_far;
        self
    }

    /// Sets the camera transform.
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Gets the view matrix (world space -> view space).
    /// Equivalent to the inverse of the camera's transform matrix.
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.to_matrix().inverse()
    }

    /// Gets the projection matrix with the specified aspect ratio.
    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        let mut z_near = self.z_near;
        let mut z_far = self.z_far;
        
        if z_near <= 0.0 {
            log::warn!("z_near in proj_matrix is <= 0.0 ({}). Correcting to 1e-4.", z_near);
            z_near = 1e-4;
        }
        if z_near >= z_far {
            log::warn!("z_near >= z_far in proj_matrix (near: {}, far: {}). Correcting z_far to near + 1e-4.", z_near, z_far);
            z_far = z_near + 1e-4;
        }

        Mat4::perspective_rh(self.fovy_deg.to_radians(), aspect, z_near, z_far)
    }

    /// Converts a 3D world space coordinate to 2D virtual screen coordinates (pixels, top-left is 0,0).
    /// Returns `None` if the point is behind the camera.
    pub fn world_to_screen(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        let view = self.view_matrix();
        let aspect = screen_size.x / screen_size.y;
        let proj = self.proj_matrix(aspect);
        let view_proj = proj * view;

        let clip_pos = view_proj * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);

        if clip_pos.w <= 0.0 {
            return None;
        }

        let ndc = clip_pos.xyz() / clip_pos.w;

        let x = (ndc.x + 1.0) * 0.5 * screen_size.x;
        let y = (1.0 - ndc.y) * 0.5 * screen_size.y;

        Some(Vec2::new(x, y))
    }

    /// Converts 2D virtual screen coordinates and NDC depth to 3D world space coordinates (unproject).
    /// `ndc_z` ranges from 0.0 (Near plane) to 1.0 (Far plane).
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2, ndc_z: f32) -> Vec3 {
        let view = self.view_matrix();
        let aspect = screen_size.x / screen_size.y;
        let proj = self.proj_matrix(aspect);
        let view_proj = proj * view;
        let inv_view_proj = view_proj.inverse();

        let ndc_x = (screen_pos.x / screen_size.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y / screen_size.y) * 2.0;

        let ndc_pos = Vec4::new(ndc_x, ndc_y, ndc_z, 1.0);
        let world_pos_w = inv_view_proj * ndc_pos;

        world_pos_w.xyz() / world_pos_w.w
    }

    /// Generates a 3D ray (origin, direction) starting from 2D virtual screen coordinates.
    /// Returns `(ray_origin, ray_direction)`.
    pub fn screen_to_ray(&self, screen_pos: Vec2, screen_size: Vec2) -> (Vec3, Vec3) {
        let near_pt = self.screen_to_world(screen_pos, screen_size, 0.0);
        let far_pt = self.screen_to_world(screen_pos, screen_size, 1.0);
        let dir = (far_pt - near_pt).normalize();
        (near_pt, dir)
    }

    /// Converts 2D virtual screen coordinates to a 3D point intersecting with the specified plane.
    /// Returns `None` if there is no intersection (e.g., ray is parallel to the plane or pointing away).
    pub fn screen_to_world_plane(
        &self,
        screen_pos: Vec2,
        screen_size: Vec2,
        plane_normal: Vec3,
        plane_origin: Vec3,
    ) -> Option<Vec3> {
        let (ray_origin, ray_dir) = self.screen_to_ray(screen_pos, screen_size);
        let denom = ray_dir.dot(plane_normal);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (plane_origin - ray_origin).dot(plane_normal) / denom;
        if t < 0.0 {
            return None;
        }
        Some(ray_origin + ray_dir * t)
    }
}
