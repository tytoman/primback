/// A representation of an RGB color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    /// Creates a new RGB color.
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Creates an RGB color from 8-bit values.
    pub const fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    /// Creates a ColorWithAlpha from 8-bit RGBA values.
    pub const fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> ColorWithAlpha {
        ColorWithAlpha {
            color: Color::from_rgb_u8(r, g, b),
            alpha: a as f32 / 255.0,
        }
    }

    // Basic standard color constants
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0);
    pub const CYAN: Color = Color::new(0.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::new(1.0, 0.0, 1.0);
    pub const GRAY: Color = Color::new(0.5, 0.5, 0.5);
    pub const ORANGE: Color = Color::new(1.0, 0.5, 0.0);
    pub const PURPLE: Color = Color::new(0.5, 0.0, 0.5);
    pub const PINK: Color = Color::new(1.0, 0.75, 0.8);

    /// Creates an RGB color from HSV values.
    ///
    /// * `h`: Hue [0.0, 360.0]
    /// * `s`: Saturation [0.0, 1.0]
    /// * `v`: Value [0.0, 1.0]
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self::new(r + m, g + m, b + m)
    }
}

/// An RGB color with an associated alpha component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorWithAlpha {
    pub color: Color,
    pub alpha: f32,
}

impl Color {
    /// Combines the RGB color with an alpha component.
    pub fn alpha(self, alpha: f32) -> ColorWithAlpha {
        ColorWithAlpha { color: self, alpha }
    }

    /// Linearly interpolates between two colors.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }
}

impl ColorWithAlpha {
    pub const CLEAR: ColorWithAlpha = ColorWithAlpha {
        color: Color::BLACK,
        alpha: 0.0,
    };

    /// Linearly interpolates between two colors with alpha components.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            color: self.color.lerp(other.color, t),
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }
}

impl From<Color> for ColorWithAlpha {
    fn from(color: Color) -> Self {
        Self { color, alpha: 1.0 }
    }
}
