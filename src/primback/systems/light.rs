use crate::primback::types::color::Color;
use glam::Vec3;

/// Light settings including direction, color, and ambient light color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub direction: Vec3,
    pub color: Color,
    pub ambient: Color,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.5, -1.0, -0.3).normalize(),
            color: Color::new(1.0, 1.0, 1.0),
            ambient: Color::new(0.2, 0.2, 0.2),
        }
    }
}

impl Light {
    /// Creates a new Light configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the light direction.
    pub fn direction(mut self, direction: Vec3) -> Self {
        if direction.length_squared() <= 0.0 {
            log::warn!("Light direction must be a non-zero vector. Falling back to default direction.");
            self.direction = Vec3::new(-0.5, -1.0, -0.3).normalize();
        } else {
            self.direction = direction.normalize();
        }
        self
    }

    /// Sets the light color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the ambient light color.
    pub fn ambient(mut self, ambient: Color) -> Self {
        self.ambient = ambient;
        self
    }
}
