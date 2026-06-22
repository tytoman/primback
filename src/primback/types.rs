//! Core geometric, color, and transformation data structures.

pub mod color;
pub mod rect;
pub mod rect_transform;
pub mod shape;
pub mod text;
pub mod transform;

pub use color::{Color, ColorWithAlpha};
pub use rect::Rect;
pub use shape::{
    CapsuleBuilder, ConeBuilder, CubeBuilder, CylinderBuilder, DiskBuilder, IcoSphereBuilder,
    PlaneBuilder, PyramidBuilder, Shape, SphereBuilder, TorusBuilder,
};
pub use text::Text;

