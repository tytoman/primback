use glam::{Vec2, Vec3, Vec4};

/// Trait representing a type that can be animated (interpolated).
pub trait Animatable: Copy {
    /// Zero value (for initial velocity)
    fn zero() -> Self;
    /// Addition
    fn add(self, other: Self) -> Self;
    /// Subtraction
    fn sub(self, other: Self) -> Self;
    /// Scalar multiplication (for velocity or physical calculations)
    fn scale(self, factor: f32) -> Self;
    /// Linear interpolation (lerp)
    fn lerp(self, target: Self, t: f32) -> Self;
    /// Returns the magnitude of the value (length, or absolute value for scalars)
    fn magnitude(self) -> f32;
}

// Implementation for f32
impl Animatable for f32 {
    #[inline]
    fn zero() -> Self {
        0.0
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self - other
    }

    #[inline]
    fn scale(self, factor: f32) -> Self {
        self * factor
    }

    #[inline]
    fn lerp(self, target: Self, t: f32) -> Self {
        self + (target - self) * t
    }

    #[inline]
    fn magnitude(self) -> f32 {
        self.abs()
    }
}

// Implementation for glam::Vec2
impl Animatable for Vec2 {
    #[inline]
    fn zero() -> Self {
        Vec2::ZERO
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self - other
    }

    #[inline]
    fn scale(self, factor: f32) -> Self {
        self * factor
    }

    #[inline]
    fn lerp(self, target: Self, t: f32) -> Self {
        self.lerp(target, t)
    }

    #[inline]
    fn magnitude(self) -> f32 {
        self.length()
    }
}

// Implementation for glam::Vec3
impl Animatable for Vec3 {
    #[inline]
    fn zero() -> Self {
        Vec3::ZERO
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self - other
    }

    #[inline]
    fn scale(self, factor: f32) -> Self {
        self * factor
    }

    #[inline]
    fn lerp(self, target: Self, t: f32) -> Self {
        self.lerp(target, t)
    }

    #[inline]
    fn magnitude(self) -> f32 {
        self.length()
    }
}

// Implementation for glam::Vec4
impl Animatable for Vec4 {
    #[inline]
    fn zero() -> Self {
        Vec4::ZERO
    }

    #[inline]
    fn add(self, other: Self) -> Self {
        self + other
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        self - other
    }

    #[inline]
    fn scale(self, factor: f32) -> Self {
        self * factor
    }

    #[inline]
    fn lerp(self, target: Self, t: f32) -> Self {
        self.lerp(target, t)
    }

    #[inline]
    fn magnitude(self) -> f32 {
        self.length()
    }
}

/// Mode defining the animation behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationMode {
    /// Linear interpolation (constant speed movement that stops exactly at the target)
    Linear { speed: f32 },
    /// Exponential decay interpolation (smooth easing that decelerates as it gets closer to the target)
    Exp { speed: f32 },
    /// Spring physical model
    Spring { stiffness: f32, damping: f32 },
}

impl AnimationMode {
    /// Creates a builder for the `Linear` mode with default settings (speed: 5.0).
    pub fn linear() -> LinearBuilder {
        LinearBuilder { speed: 5.0 }
    }

    /// Creates a builder for the `Exp` mode with default settings (speed: 5.0).
    pub fn exp() -> ExpBuilder {
        ExpBuilder { speed: 5.0 }
    }

    /// Creates a builder for the `Spring` mode with default settings (stiffness: 50.0, damping: 10.0).
    pub fn spring() -> SpringBuilder {
        SpringBuilder {
            stiffness: 50.0,
            damping: 10.0,
        }
    }
}

/// Builder for the Linear animation mode.
#[derive(Clone, Copy, Debug)]
pub struct LinearBuilder {
    speed: f32,
}

impl LinearBuilder {
    /// Sets the movement speed.
    pub fn speed(mut self, speed: f32) -> Self {
        if speed <= 0.0 {
            log::warn!("Linear animation speed must be greater than 0.0. Clamping to 1e-6. Got: {}", speed);
            self.speed = 1e-6;
        } else {
            self.speed = speed;
        }
        self
    }

    /// Builds and generates the `AnimationMode`.
    pub fn build(self) -> AnimationMode {
        AnimationMode::Linear { speed: self.speed }
    }
}

impl From<LinearBuilder> for AnimationMode {
    fn from(builder: LinearBuilder) -> Self {
        builder.build()
    }
}

/// Builder for the Exp animation mode.
#[derive(Clone, Copy, Debug)]
pub struct ExpBuilder {
    speed: f32,
}

impl ExpBuilder {
    /// Sets the decay speed.
    pub fn speed(mut self, speed: f32) -> Self {
        if speed <= 0.0 {
            log::warn!("Exponential animation speed must be greater than 0.0. Clamping to 1e-6. Got: {}", speed);
            self.speed = 1e-6;
        } else {
            self.speed = speed;
        }
        self
    }

    /// Builds and generates the `AnimationMode`.
    pub fn build(self) -> AnimationMode {
        AnimationMode::Exp { speed: self.speed }
    }
}

impl From<ExpBuilder> for AnimationMode {
    fn from(builder: ExpBuilder) -> Self {
        builder.build()
    }
}

/// Builder for the Spring animation mode.
#[derive(Clone, Copy, Debug)]
pub struct SpringBuilder {
    stiffness: f32,
    damping: f32,
}

impl SpringBuilder {
    /// Sets the stiffness (strength of the spring).
    pub fn stiffness(mut self, stiffness: f32) -> Self {
        if stiffness <= 0.0 {
            log::warn!("Spring stiffness must be greater than 0.0. Clamping to 1e-6. Got: {}", stiffness);
            self.stiffness = 1e-6;
        } else {
            self.stiffness = stiffness;
        }
        self
    }

    /// Sets the damping coefficient.
    pub fn damping(mut self, damping: f32) -> Self {
        if damping < 0.0 {
            log::warn!("Spring damping must be non-negative. Clamping to 0.0. Got: {}", damping);
            self.damping = 0.0;
        } else {
            self.damping = damping;
        }
        self
    }

    /// Builds and generates the `AnimationMode`.
    pub fn build(self) -> AnimationMode {
        AnimationMode::Spring {
            stiffness: self.stiffness,
            damping: self.damping,
        }
    }
}

impl From<SpringBuilder> for AnimationMode {
    fn from(builder: SpringBuilder) -> Self {
        builder.build()
    }
}

/// A structure that manages an animatable value.
#[derive(Clone, Debug)]
pub struct Animated<T: Animatable> {
    current: T,
    target: T,
    velocity: T,
    pub mode: AnimationMode,
}

impl<T: Animatable> Animated<T> {
    /// Creates a new `Animated` instance.
    pub fn new(initial_value: T, mode: impl Into<AnimationMode>) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            velocity: T::zero(),
            mode: mode.into(),
        }
    }

    /// Gets the current value.
    pub fn current(&self) -> T {
        self.current
    }

    /// Gets the target value.
    pub fn target(&self) -> T {
        self.target
    }

    /// Sets the target value. This starts the follow animation.
    pub fn set_target(&mut self, target: T) {
        self.target = target;
    }

    /// Overwrites the current value. The velocity is reset to zero.
    pub fn set_current(&mut self, value: T) {
        self.current = value;
        self.velocity = T::zero();
    }

    /// Gets the internal velocity (for spring physics).
    pub fn velocity(&self) -> T {
        self.velocity
    }

    /// Sets the internal velocity directly.
    pub fn set_velocity(&mut self, velocity: T) {
        self.velocity = velocity;
    }

    /// Updates the state based on the elapsed time in seconds and returns the new current value.
    pub fn update(&mut self, dt: f32) -> T {
        if dt <= 0.0 {
            return self.current;
        }

        // Validate and clamp animation parameters to prevent unstable simulation or infinite loop
        match &mut self.mode {
            AnimationMode::Linear { speed } => {
                if *speed <= 0.0 {
                    log::warn!("Animated Linear speed is <= 0.0 ({}). Clamping to 1e-6.", *speed);
                    *speed = 1e-6;
                }
            }
            AnimationMode::Exp { speed } => {
                if *speed <= 0.0 {
                    log::warn!("Animated Exp speed is <= 0.0 ({}). Clamping to 1e-6.", *speed);
                    *speed = 1e-6;
                }
            }
            AnimationMode::Spring { stiffness, damping } => {
                if *stiffness <= 0.0 {
                    log::warn!("Animated Spring stiffness is <= 0.0 ({}). Clamping to 1e-6.", *stiffness);
                    *stiffness = 1e-6;
                }
                if *damping < 0.0 {
                    log::warn!("Animated Spring damping is < 0.0 ({}). Clamping to 0.0.", *damping);
                    *damping = 0.0;
                }
            }
        }

        // If dt is extremely large (due to frame drops, etc.), restrict the step size to a maximum of 0.1s
        // to keep the simulation stable, dividing it into substeps if necessary.
        let max_step = 0.1;
        let mut remaining = dt;

        while remaining > 0.0 {
            let step = remaining.min(max_step);
            remaining -= step;

            match self.mode {
                AnimationMode::Linear { speed } => {
                    let diff = self.target.sub(self.current);
                    let dist = diff.magnitude();
                    if dist > 0.0 {
                        let step_dist = speed * step;
                        if step_dist >= dist {
                            self.current = self.target;
                        } else {
                            let dir = diff.scale(1.0 / dist);
                            self.current = self.current.add(dir.scale(step_dist));
                        }
                    }
                }
                AnimationMode::Exp { speed } => {
                    // Exponential easing (decelerates as it approaches)
                    let t = 1.0 - (-speed * step).exp();
                    self.current = self.current.lerp(self.target, t.clamp(0.0, 1.0));
                }
                AnimationMode::Spring { stiffness, damping } => {
                    // F = -k * x - c * v
                    let displacement = self.current.sub(self.target);
                    let spring_force = displacement.scale(-stiffness);
                    let damping_force = self.velocity.scale(-damping);
                    let acceleration = spring_force.add(damping_force);

                    // Update using Semi-implicit Euler integration (stable and conserves energy well)
                    self.velocity = self.velocity.add(acceleration.scale(step));
                    self.current = self.current.add(self.velocity.scale(step));
                }
            }
        }

        self.current
    }
}

