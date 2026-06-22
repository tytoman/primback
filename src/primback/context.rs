pub mod draw;
pub mod update;

pub use draw::{DrawBuffers, DrawContext};
pub use update::UpdateContext;

// Re-export split definitions to maintain compatibility
pub use crate::primback::graphics::vertices::{
    InstanceData, InstanceVertex, Line2DVertex, RectVertex, TextVertex, Vertex3D,
};
pub use crate::primback::types::color::{Color, ColorWithAlpha};
pub use crate::primback::types::rect::Rect;
pub use crate::primback::types::rect_transform::{Anchor, RectTransform};
pub use crate::primback::types::shape::{
    CapsuleBuilder, ConeBuilder, CubeBuilder, CylinderBuilder, DiskBuilder, IcoSphereBuilder,
    PlaneBuilder, PyramidBuilder, Shape, SphereBuilder, TorusBuilder,
};
pub use crate::primback::types::text::Text;
