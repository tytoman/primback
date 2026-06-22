use crate::primback::types::color::Color;
use glam::{Vec2, Vec3, vec2, vec3};
use web_time::SystemTime;

/// Pseudo-random number generator based on Xorshift64*.
#[derive(Clone, Copy, Debug)]
pub struct Random {
    state: u64,
}

impl Random {
    /// Creates a new random number generator from the specified seed value.
    /// Ensures the generator works correctly even if the seed is 0.
    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            state: if seed == 0 { 0xCAFEEBEEF00D5EED } else { seed },
        };
        // Warm up the initial state by advancing a few times
        for _ in 0..4 {
            let _ = rng.next_u64();
        }
        rng
    }

    /// Initializes with a random seed based on the current system time (SystemTime).
    /// Works correctly even on WASM targets.
    pub fn new_random() -> Self {
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        // Mix the seed further with a fixed hash since time changes constantly
        Self::new(seed ^ 0x5555555555555555)
    }

    /// Generates a random 64-bit integer.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 12;
        x ^= x >> 25;
        x ^= x << 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Generates a random 32-bit integer.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Generates a random f32 in the range [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        let val = self.next_u32() >> 8; // 24-bit precision
        (val as f32) / (1 << 24) as f32
    }

    /// Generates a random f64 in the range [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        let val = self.next_u64() >> 11; // 53-bit precision
        (val as f64) / (1u64 << 53) as f64
    }

    /// Generates a random f32 in the range [min, max).
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Generates a random i32 in the range [min, max] (inclusive).
    pub fn int_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min + 1) as u32;
        let val = self.next_u32() % range;
        min + val as i32
    }

    /// Generates a boolean that is true with the specified probability (0.0 to 1.0).
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    /// Generates a Vec2 where each component is in the range [min, max).
    pub fn vec2_range(&mut self, min: f32, max: f32) -> Vec2 {
        vec2(self.range(min, max), self.range(min, max))
    }

    /// Generates a Vec3 where each component is in the range [min, max).
    pub fn vec3_range(&mut self, min: f32, max: f32) -> Vec3 {
        vec3(
            self.range(min, max),
            self.range(min, max),
            self.range(min, max),
        )
    }

    /// Generates a random point (Vec3) inside a sphere of radius 1.
    pub fn vec3_in_sphere(&mut self) -> Vec3 {
        loop {
            let v = vec3(
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
                self.range(-1.0, 1.0),
            );
            if v.length_squared() <= 1.0 {
                return v;
            }
        }
    }

    /// Generates a random direction vector (Vec3) on a sphere of radius 1.
    pub fn vec3_on_sphere(&mut self) -> Vec3 {
        let u = self.next_f32();
        let v = self.next_f32();
        let theta = u * 2.0 * std::f32::consts::PI;
        let phi = (v * 2.0 - 1.0).acos();

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        vec3(sin_phi * cos_theta, sin_phi * sin_theta, cos_phi)
    }

    /// Generates a random Color (RGB, without alpha).
    pub fn color(&mut self) -> Color {
        Color::new(self.next_f32(), self.next_f32(), self.next_f32())
    }

    /// Generates a random color using the HSV color model.
    ///
    /// * `h_min`, `h_max`: Hue range [0.0, 360.0]
    /// * `s_min`, `s_max`: Saturation range [0.0, 1.0]
    /// * `v_min`, `v_max`: Value range [0.0, 1.0]
    pub fn color_hsv(
        &mut self,
        h_min: f32,
        h_max: f32,
        s_min: f32,
        s_max: f32,
        v_min: f32,
        v_max: f32,
    ) -> Color {
        let h = self.range(h_min, h_max);
        let s = self.range(s_min, s_max);
        let v = self.range(v_min, v_max);
        Color::from_hsv(h, s, v)
    }
}

