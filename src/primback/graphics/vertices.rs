use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// Vertex structure for instanced meshes.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl InstanceVertex {
    /// Returns the vertex buffer layout descriptor.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // location 0: position
                VertexAttribute {
                    offset: std::mem::offset_of!(InstanceVertex, position) as BufferAddress,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // location 1: normal
                VertexAttribute {
                    offset: std::mem::offset_of!(InstanceVertex, normal) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Instance data containing the model transformation matrix, normal transformation matrix, and color.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 3]; 3],
    pub color: [f32; 4],
}

impl InstanceData {
    /// Returns the vertex buffer layout descriptor for instance data.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // location 2-5: model_matrix
                VertexAttribute {
                    offset: std::mem::offset_of!(InstanceData, model_matrix) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::offset_of!(InstanceData, model_matrix)
                        + std::mem::size_of::<[f32; 4]>()) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::offset_of!(InstanceData, model_matrix)
                        + std::mem::size_of::<[f32; 4]>() * 2) as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: (std::mem::offset_of!(InstanceData, model_matrix)
                        + std::mem::size_of::<[f32; 4]>() * 3) as BufferAddress,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                // location 6: color
                VertexAttribute {
                    offset: std::mem::offset_of!(InstanceData, color) as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                // location 7-9: normal_matrix
                VertexAttribute {
                    offset: std::mem::offset_of!(InstanceData, normal_matrix) as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (std::mem::offset_of!(InstanceData, normal_matrix)
                        + std::mem::size_of::<[f32; 3]>()) as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: (std::mem::offset_of!(InstanceData, normal_matrix)
                        + std::mem::size_of::<[f32; 3]>() * 2) as BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// A mesh cached on the GPU containing vertex and index buffers.
pub struct CachedMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

/// Vertex structure for standard 3D rendering (position, normal, color).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex3D {
    /// Returns the vertex buffer layout descriptor.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3D>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // location 0: position
                VertexAttribute {
                    offset: std::mem::offset_of!(Vertex3D, position) as BufferAddress,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // location 1: normal
                VertexAttribute {
                    offset: std::mem::offset_of!(Vertex3D, normal) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                // location 2: color
                VertexAttribute {
                    offset: std::mem::offset_of!(Vertex3D, color) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Vertex structure used for the fullscreen upscale render pass.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScaleVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl ScaleVertex {
    /// Returns the vertex buffer layout descriptor.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<ScaleVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: std::mem::offset_of!(ScaleVertex, position) as BufferAddress,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(ScaleVertex, tex_coords) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Vertex structure used for rendering UI rectangles with SDF features.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectVertex {
    pub position: [f32; 2],
    pub local_pos: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub radius: [f32; 4],
    pub outline_width: f32,
    pub outline_color: [f32; 4],
    pub shadow_offset: [f32; 2],
    pub shadow_color: [f32; 4],
    pub z_index: f32,
}

impl RectVertex {
    /// Returns the vertex buffer layout descriptor.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<RectVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, position) as BufferAddress,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, local_pos) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, size) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, color) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, radius) as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, outline_width) as BufferAddress,
                    shader_location: 5,
                    format: VertexFormat::Float32,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, outline_color) as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, shadow_offset) as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, shadow_color) as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(RectVertex, z_index) as BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Vertex structure used for rendering 2D lines.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Line2DVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub z_index: f32,
}

impl Line2DVertex {
    /// Returns the vertex buffer layout descriptor.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Line2DVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: std::mem::offset_of!(Line2DVertex, position) as BufferAddress,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(Line2DVertex, color) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(Line2DVertex, z_index) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Vertex structure used for rendering UI text.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub z_index: f32,
}

impl TextVertex {
    /// Returns the vertex buffer layout descriptor.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: std::mem::offset_of!(TextVertex, position) as BufferAddress,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(TextVertex, tex_coords) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(TextVertex, color) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::offset_of!(TextVertex, z_index) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}

