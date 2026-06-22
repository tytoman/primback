use crate::primback::graphics::vertices::{
    InstanceData, InstanceVertex, Line2DVertex, RectVertex, ScaleVertex, TextVertex, Vertex3D,
};

struct PipelineDesc<'a> {
    label: Option<&'a str>,
    layout: &'a wgpu::PipelineLayout,
    shader: &'a wgpu::ShaderModule,
    vertex_buffers: &'a [wgpu::VertexBufferLayout<'static>],
    topology: wgpu::PrimitiveTopology,
    cull_mode: Option<wgpu::Face>,
    blend: Option<wgpu::BlendState>,
    depth_write: Option<bool>,
    color_format: wgpu::TextureFormat,
}

fn build_pipeline(device: &wgpu::Device, desc: PipelineDesc) -> wgpu::RenderPipeline {
    let depth_stencil = desc.depth_write.map(|write_enabled| wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: Some(write_enabled),
        depth_compare: Some(wgpu::CompareFunction::LessEqual),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: desc.label,
        layout: Some(desc.layout),
        vertex: wgpu::VertexState {
            module: desc.shader,
            entry_point: Some("vs_main"),
            buffers: desc.vertex_buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: desc.shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: desc.color_format,
                blend: desc.blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: desc.topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: desc.cull_mode,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Creates render pipelines for rendering basic 3D triangles and lines.
pub fn create_primitive_pipelines(
    device: &wgpu::Device,
    primitive_bind_group_layout: &wgpu::BindGroupLayout,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
) {
    let primitive_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/primitive.wgsl"));
    let primitive_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Primitive Pipeline Layout"),
            bind_group_layouts: &[Some(primitive_bind_group_layout)],
            immediate_size: 0,
        });

    let triangle_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Triangle Render Pipeline"),
            layout: &primitive_pipeline_layout,
            shader: &primitive_shader,
            vertex_buffers: &[Vertex3D::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            blend: Some(wgpu::BlendState::REPLACE),
            depth_write: Some(true),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    let transparent_triangle_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Transparent Triangle Render Pipeline"),
            layout: &primitive_pipeline_layout,
            shader: &primitive_shader,
            vertex_buffers: &[Vertex3D::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: Some(false),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    let line_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Line Render Pipeline"),
            layout: &primitive_pipeline_layout,
            shader: &primitive_shader,
            vertex_buffers: &[Vertex3D::desc()],
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::REPLACE),
            depth_write: Some(true),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    let transparent_line_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Transparent Line Render Pipeline"),
            layout: &primitive_pipeline_layout,
            shader: &primitive_shader,
            vertex_buffers: &[Vertex3D::desc()],
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: Some(false),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    (
        triangle_pipeline,
        line_pipeline,
        transparent_triangle_pipeline,
        transparent_line_pipeline,
    )
}

/// Creates a render pipeline for UI text rendering.
pub fn create_text_pipeline(
    device: &wgpu::Device,
    screen_size_bind_group_layout: &wgpu::BindGroupLayout,
    font_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let text_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/text.wgsl"));
    let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Text Pipeline Layout"),
        bind_group_layouts: &[
            Some(screen_size_bind_group_layout),
            Some(font_bind_group_layout),
        ],
        immediate_size: 0,
    });

    build_pipeline(
        device,
        PipelineDesc {
            label: Some("Text Render Pipeline"),
            layout: &text_pipeline_layout,
            shader: &text_shader,
            vertex_buffers: &[TextVertex::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: None,
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    )
}

/// Creates a render pipeline for UI rect rendering.
pub fn create_rect_pipeline(
    device: &wgpu::Device,
    screen_size_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let rect_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/rect.wgsl"));
    let rect_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Rect Pipeline Layout"),
        bind_group_layouts: &[Some(screen_size_bind_group_layout)],
        immediate_size: 0,
    });

    build_pipeline(
        device,
        PipelineDesc {
            label: Some("Rect Render Pipeline"),
            layout: &rect_pipeline_layout,
            shader: &rect_shader,
            vertex_buffers: &[RectVertex::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: None,
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    )
}

/// Creates a render pipeline for UI 2D line rendering.
pub fn create_line2d_pipeline(
    device: &wgpu::Device,
    screen_size_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let line2d_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/line2d.wgsl"));
    let line2d_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Line2D Pipeline Layout"),
        bind_group_layouts: &[Some(screen_size_bind_group_layout)],
        immediate_size: 0,
    });

    build_pipeline(
        device,
        PipelineDesc {
            label: Some("Line2D Render Pipeline"),
            layout: &line2d_pipeline_layout,
            shader: &line2d_shader,
            vertex_buffers: &[Line2DVertex::desc()],
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: None,
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    )
}

/// Creates a render pipeline for upscaling the low-res render target to the physical window.
pub fn create_scaler_pipeline(
    device: &wgpu::Device,
    scaler_bind_group_layout: &wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let scaler_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/scaler.wgsl"));
    let scaler_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Scaler Pipeline Layout"),
        bind_group_layouts: &[Some(scaler_bind_group_layout)],
        immediate_size: 0,
    });

    build_pipeline(
        device,
        PipelineDesc {
            label: Some("Scaler Render Pipeline"),
            layout: &scaler_pipeline_layout,
            shader: &scaler_shader,
            vertex_buffers: &[ScaleVertex::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: None,
            color_format: surface_format,
        },
    )
}

/// Creates render pipelines for instanced 3D solid and wireframe shapes.
pub fn create_instance_pipelines(
    device: &wgpu::Device,
    primitive_pipeline_layout: &wgpu::PipelineLayout,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
) {
    let instance_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/instance.wgsl"));

    let instance_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Instance Render Pipeline"),
            layout: primitive_pipeline_layout,
            shader: &instance_shader,
            vertex_buffers: &[InstanceVertex::desc(), InstanceData::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            blend: Some(wgpu::BlendState::REPLACE),
            depth_write: Some(true),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    let transparent_instance_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Transparent Instance Render Pipeline"),
            layout: primitive_pipeline_layout,
            shader: &instance_shader,
            vertex_buffers: &[InstanceVertex::desc(), InstanceData::desc()],
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: Some(false),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    let instance_wire_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Instance Wire Render Pipeline"),
            layout: primitive_pipeline_layout,
            shader: &instance_shader,
            vertex_buffers: &[InstanceVertex::desc(), InstanceData::desc()],
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::REPLACE),
            depth_write: Some(true),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    let transparent_instance_wire_pipeline = build_pipeline(
        device,
        PipelineDesc {
            label: Some("Transparent Instance Wire Render Pipeline"),
            layout: primitive_pipeline_layout,
            shader: &instance_shader,
            vertex_buffers: &[InstanceVertex::desc(), InstanceData::desc()],
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write: Some(false),
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
        },
    );

    (
        instance_pipeline,
        instance_wire_pipeline,
        transparent_instance_pipeline,
        transparent_instance_wire_pipeline,
    )
}
