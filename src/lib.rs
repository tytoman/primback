//! # Primback
//!
//! Primback is a lightweight, easy-to-use 3D/2D primitive rendering engine
//! built on top of `wgpu` and `winit`. It provides quick utilities for shape generation,
//! basic physics simulation, and animations.

pub mod primback;

pub use primback::context::{
    CapsuleBuilder, Color, ColorWithAlpha, ConeBuilder, CubeBuilder, CylinderBuilder, DiskBuilder,
    DrawContext, IcoSphereBuilder, PlaneBuilder, PyramidBuilder, Rect, Shape, SphereBuilder, Text,
    TorusBuilder, UpdateContext,
};

pub use primback::systems::{
    animation::{Animatable, Animated, AnimationMode, ExpBuilder, LinearBuilder, SpringBuilder},
    audio::{AudioListener, IntoFreq, Pitch, Synth, Waveform},
    camera::Camera,
    input::{KeyCode, MouseButton},
    light::Light,
    random::Random,
};

pub use primback::types::{
    rect_transform::{Anchor, RectTransform},
    transform::{Transform, WorldTransform},
};
pub use primback::graphics::FontConfig;
pub use primback::{Primback, PrimbackApp, PrimbackConfig};

// Re-export types, macros, and rotation orders from the linear algebra library `glam`
pub use glam::{
    EulerRot, IVec2, Mat4, Quat, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles, ivec2,
    vec2, vec3, vec4,
};

pub mod prelude {
    pub use crate::{
        Anchor, Animatable, Animated, AnimationMode, AudioListener, Camera, Color, ColorWithAlpha,
        DrawContext, EulerRot, ExpBuilder, FontConfig, IVec2, IntoFreq, KeyCode, Light, LinearBuilder, Mat4,
        MouseButton, Pitch, Primback, PrimbackApp, PrimbackConfig, Quat, Random, Rect,
        RectTransform, Shape, SpringBuilder, Synth, Text, Transform, UpdateContext, Vec2,
        Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles, Waveform, WorldTransform, ivec2,
        vec2, vec3, vec4,
    };
}

