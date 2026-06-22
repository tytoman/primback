use std::{collections::HashMap, sync::Arc};

use wgpu::{
    RenderPassColorAttachment, RenderPassDescriptor,
    util::DeviceExt,
};


use crate::primback::{
    graphics::{
        buffer::DynamicBuffer,
        pipeline::create_scaler_pipeline,
        vertices::{
            CachedMesh, InstanceData, Line2DVertex, RectVertex, ScaleVertex, TextVertex, Vertex3D,
        },
    },
    systems::camera::Camera,
    types::shape::{
        Shape,
        geometry::{generate_solid_geometry, generate_wire_geometry},
    },
};
use glam::Vec3;

pub mod buffer;
pub mod font_atlas;
pub mod pipeline;
pub mod vertices;

pub use font_atlas::{FontAtlas, FontConfig, GlyphInfo};

/// A batch of 3D primitive instances sharing the same geometry shape.
pub struct InstancedBatch {
    pub shape: Shape,
    pub instances: Vec<InstanceData>,
}

/// A batch of 3D wireframe instances sharing the same geometry shape.
pub struct InstancedWireBatch {
    pub shape: Shape,
    pub instances: Vec<InstanceData>,
}

/// Parameters passed to the render function.
pub struct RenderParams<'a> {
    pub triangles: &'a [Vertex3D],
    pub transparent_triangles: &'a [Vertex3D],
    pub lines: &'a [Vertex3D],
    pub transparent_lines: &'a [Vertex3D],
    pub rect_vertices: &'a [RectVertex],
    pub text_vertices: &'a [TextVertex],
    pub line2d_vertices: &'a [Line2DVertex],
    pub instanced_batches: &'a [InstancedBatch],
    /// Transparent instanced batches.
    ///
    /// # Note
    /// Only sorted within the same shape batch, which may cause depth artifacts when mixed.
    pub transparent_instanced_batches: &'a [InstancedBatch],
    pub instanced_wire_batches: &'a [InstancedWireBatch],
    /// Transparent instanced wire batches.
    ///
    /// # Note
    /// Only sorted within the same shape batch, which may cause depth artifacts when mixed.
    pub transparent_instanced_wire_batches: &'a [InstancedWireBatch],
    pub clear_color: Option<[f32; 4]>,
    pub camera: &'a Camera,
    pub light_dir: Vec3,
    pub light_color: [f32; 3],
    pub light_ambient: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    direction: [f32; 4],
    color: [f32; 4],
    ambient: [f32; 4],
}

/// The result of a rendering operation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RenderResult {
    Success,
    Suboptimal,
}

/// Wgpu rendering subsystem that manages device, queue, render targets, pipelines, and mesh caches.
pub struct Graphics {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    render_width: u32,
    render_height: u32,

    // Render Target Texture
    rt_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    scaler_bind_group: wgpu::BindGroup,

    camera_buffer: wgpu::Buffer,
    light_buffer: wgpu::Buffer,
    primitive_bind_group: wgpu::BindGroup,

    // Primitive Pipelines
    opaque_triangle_pipeline: wgpu::RenderPipeline,
    transparent_triangle_pipeline: wgpu::RenderPipeline,
    opaque_line_pipeline: wgpu::RenderPipeline,
    transparent_line_pipeline: wgpu::RenderPipeline,

    scaler_pipeline: wgpu::RenderPipeline,
    scaler_vertex_buffer: wgpu::Buffer,

    // Dynamic Buffers
    triangle_buffer: DynamicBuffer<Vertex3D>,
    transparent_triangle_buffer: DynamicBuffer<Vertex3D>,
    line_buffer: DynamicBuffer<Vertex3D>,
    transparent_line_buffer: DynamicBuffer<Vertex3D>,

    // Text rendering resources
    text_pipeline: wgpu::RenderPipeline,
    text_buffer: DynamicBuffer<TextVertex>,
    font_atlas: FontAtlas,
    // Held to keep the GPU buffer alive; bound via screen_size_bind_group and never written to after init.
    _screen_size_buffer: wgpu::Buffer,

    screen_size_bind_group: wgpu::BindGroup,

    // 2D rect rendering resources
    rect_pipeline: wgpu::RenderPipeline,
    rect_buffer: DynamicBuffer<RectVertex>,

    // 2D line rendering resources
    line2d_pipeline: wgpu::RenderPipeline,
    line2d_buffer: DynamicBuffer<Line2DVertex>,

    opaque_instance_pipeline: wgpu::RenderPipeline,
    transparent_instance_pipeline: wgpu::RenderPipeline,
    opaque_instance_wire_pipeline: wgpu::RenderPipeline,
    transparent_instance_wire_pipeline: wgpu::RenderPipeline,
    instance_data_buffer: DynamicBuffer<InstanceData>,
    mesh_cache: HashMap<Shape, CacheEntry>,
    wire_mesh_cache: HashMap<Shape, CacheEntry>,
    frame_count: u64,
}

const MAX_UNUSED_FRAMES: u64 = 120;

struct CacheEntry {
    mesh: CachedMesh,
    last_used_frame: u64,
}

impl Graphics {
    /// Creates a new `Graphics` renderer state associated with the given window.
    pub async fn new(
        window: Arc<winit::window::Window>,
        render_width: u32,
        render_height: u32,
        vsync: bool,
        font_config: &FontConfig,
    ) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(window.clone())),
        );

        // TODO: Is unwrap fine here?
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find appropriate GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                ..wgpu::DeviceDescriptor::default()
            })
            .await
            .expect("Failed to create wgpu device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = if vsync {
            wgpu::PresentMode::Fifo
        } else {
            if surface_caps
                .present_modes
                .contains(&wgpu::PresentMode::Mailbox)
            {
                wgpu::PresentMode::Mailbox
            } else if surface_caps
                .present_modes
                .contains(&wgpu::PresentMode::Immediate)
            {
                wgpu::PresentMode::Immediate
            } else {
                wgpu::PresentMode::Fifo
            }
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let rt_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RenderTarget Texture"),
            size: wgpu::Extent3d {
                width: render_width,
                height: render_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let rt_view = rt_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: render_width,
                height: render_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let rt_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Neighbor Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size: std::mem::size_of::<LightUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let primitive_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Primitive Bind Group Layout"),
                entries: &[
                    // binding 0: Camera Uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 1: Light Uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let primitive_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Primitive Bind Group"),
            layout: &primitive_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });

        let (
            opaque_triangle_pipeline,
            opaque_line_pipeline,
            transparent_triangle_pipeline,
            transparent_line_pipeline,
        ) = pipeline::create_primitive_pipelines(&device, &primitive_bind_group_layout);

        let scaler_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scaler Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let scaler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scaler Bind Group"),
            layout: &scaler_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&rt_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&rt_sampler),
                },
            ],
        });

        let scaler_pipeline =
            create_scaler_pipeline(&device, &scaler_bind_group_layout, surface_format);

        let scaler_vertices =
            calculate_scale_vertices(render_width, render_height, size.width, size.height);
        let scaler_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scaler Vertex Buffer"),
            contents: bytemuck::cast_slice(&scaler_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Load font texture using FontAtlas
        let font_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Font Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let font_atlas = font_atlas::FontAtlas::new(
            &device,
            &queue,
            &font_bind_group_layout,
            &rt_sampler,
            font_config,
        ).expect("Failed to create FontAtlas");


        // Screen size uniform buffer
        let screen_size_data = [render_width as f32, render_height as f32];
        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Size Uniform Buffer"),
            contents: bytemuck::cast_slice(&screen_size_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let screen_size_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Screen Size Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let screen_size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen Size Bind Group"),
            layout: &screen_size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        // Construct pipelines
        let text_pipeline = pipeline::create_text_pipeline(
            &device,
            &screen_size_bind_group_layout,
            &font_bind_group_layout,
        );
        let rect_pipeline = pipeline::create_rect_pipeline(&device, &screen_size_bind_group_layout);
        let line2d_pipeline =
            pipeline::create_line2d_pipeline(&device, &screen_size_bind_group_layout);

        // Dynamic Buffers
        let triangle_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            2048,
        );
        let transparent_triangle_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            2048,
        );
        let line_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            2048,
        );
        let transparent_line_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            2048,
        );
        let text_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            512,
        );
        let rect_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            512,
        );
        let line2d_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            512,
        );

        let primitive_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Primitive Pipeline Layout"),
                bind_group_layouts: &[Some(&primitive_bind_group_layout)],
                immediate_size: 0,
            });

        let (
            opaque_instance_pipeline,
            opaque_instance_wire_pipeline,
            transparent_instance_pipeline,
            transparent_instance_wire_pipeline,
        ) = pipeline::create_instance_pipelines(&device, &primitive_pipeline_layout);

        let instance_data_buffer = DynamicBuffer::new(
            &device,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            1024,
        );

        let mesh_cache = HashMap::new();
        let wire_mesh_cache = HashMap::new();

        Self {
            device,
            queue,
            surface,
            config,
            render_width,
            render_height,
            rt_view,
            depth_view,
            scaler_bind_group,
            camera_buffer,
            light_buffer,
            primitive_bind_group,
            scaler_pipeline,
            scaler_vertex_buffer,
            opaque_triangle_pipeline,
            transparent_triangle_pipeline,
            opaque_line_pipeline,
            transparent_line_pipeline,
            triangle_buffer,
            transparent_triangle_buffer,
            line_buffer,
            transparent_line_buffer,
            text_pipeline,
            text_buffer,
            font_atlas,

            _screen_size_buffer: screen_size_buffer,
            screen_size_bind_group,
            rect_pipeline,
            rect_buffer,
            line2d_pipeline,
            line2d_buffer,
            opaque_instance_pipeline,
            transparent_instance_pipeline,
            opaque_instance_wire_pipeline,
            transparent_instance_wire_pipeline,
            instance_data_buffer,
            mesh_cache,
            wire_mesh_cache,
            frame_count: 0,
        }
    }

    /// Gets the current physical window size.
    pub fn window_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Resizes the surface configuration and updates upscaling parameters.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let scaler_vertices = calculate_scale_vertices(
                self.render_width,
                self.render_height,
                new_size.width,
                new_size.height,
            );
            self.queue.write_buffer(
                &self.scaler_vertex_buffer,
                0,
                bytemuck::cast_slice(&scaler_vertices),
            );
        }
    }

    fn get_or_create_mesh(&mut self, shape: &Shape) -> &CachedMesh {
        let frame_count = self.frame_count;
        let entry = self.mesh_cache.entry(*shape).or_insert_with(|| {
            let (vertices, indices) = generate_solid_geometry(shape);
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instanced Mesh Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instanced Mesh Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
            let mesh = CachedMesh {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            };
            CacheEntry {
                mesh,
                last_used_frame: frame_count,
            }
        });
        entry.last_used_frame = frame_count;
        &entry.mesh
    }

    fn get_or_create_wire_mesh(&mut self, shape: &Shape) -> &CachedMesh {
        let frame_count = self.frame_count;
        let entry = self.wire_mesh_cache.entry(*shape).or_insert_with(|| {
            let (vertices, indices) = generate_wire_geometry(shape);
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instanced Wire Mesh Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instanced Wire Mesh Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
            let mesh = CachedMesh {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            };
            CacheEntry {
                mesh,
                last_used_frame: frame_count,
            }
        });
        entry.last_used_frame = frame_count;
        &entry.mesh
    }

    /// Performs rendering of 3D primitives, UI elements, and upscales the result to the window.
    pub fn render(
        &mut self,
        params: RenderParams,
    ) -> Result<RenderResult, wgpu::CurrentSurfaceTexture> {
        let RenderParams {
            triangles,
            transparent_triangles,
            lines,
            transparent_lines,
            rect_vertices,
            text_vertices,
            line2d_vertices,
            instanced_batches,
            transparent_instanced_batches,
            instanced_wire_batches,
            transparent_instanced_wire_batches,
            clear_color,
            camera,
            light_dir,
            light_color,
            light_ambient,
        } = params;

        self.frame_count += 1;

        // --- 0. Allocate mesh cache and transfer data for instancing ---
        for batch in instanced_batches {
            self.get_or_create_mesh(&batch.shape);
        }
        for batch in transparent_instanced_batches {
            self.get_or_create_mesh(&batch.shape);
        }
        for batch in instanced_wire_batches {
            self.get_or_create_wire_mesh(&batch.shape);
        }
        for batch in transparent_instanced_wire_batches {
            self.get_or_create_wire_mesh(&batch.shape);
        }

        let frame_count = self.frame_count;
        self.mesh_cache.retain(|_, entry| frame_count - entry.last_used_frame < MAX_UNUSED_FRAMES);
        self.wire_mesh_cache.retain(|_, entry| frame_count - entry.last_used_frame < MAX_UNUSED_FRAMES);

        let mut all_instances = Vec::new();
        for batch in instanced_batches {
            all_instances.extend_from_slice(&batch.instances);
        }
        let solid_instance_count = all_instances.len();

        for batch in instanced_wire_batches {
            all_instances.extend_from_slice(&batch.instances);
        }
        let _opaque_wire_instance_count = all_instances.len() - solid_instance_count;
        let total_opaque_instance_count = all_instances.len();

        for batch in transparent_instanced_batches {
            all_instances.extend_from_slice(&batch.instances);
        }
        let _transparent_solid_instance_count = all_instances.len() - total_opaque_instance_count;
        let total_transparent_solid_instance_count = all_instances.len();

        for batch in transparent_instanced_wire_batches {
            all_instances.extend_from_slice(&batch.instances);
        }

        if !all_instances.is_empty() {
            self.instance_data_buffer
                .write(&self.device, &self.queue, &all_instances);
        }

        // --- 1. Create and update Uniform buffers ---
        let aspect = self.render_width as f32 / self.render_height as f32;
        let view_proj = camera.proj_matrix(aspect) * camera.view_matrix();

        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // Normalize directional light vector
        let l_dir = light_dir.normalize_or_zero();
        let light_uniform = LightUniform {
            direction: [l_dir.x, l_dir.y, l_dir.z, 0.0],
            color: [light_color[0], light_color[1], light_color[2], 0.0],
            ambient: [light_ambient[0], light_ambient[1], light_ambient[2], 0.0],
        };
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[light_uniform]),
        );

        // --- 2. Write to GPU dynamic vertex buffers ---
        self.triangle_buffer
            .write(&self.device, &self.queue, triangles);
        self.transparent_triangle_buffer
            .write(&self.device, &self.queue, transparent_triangles);
        self.line_buffer.write(&self.device, &self.queue, lines);
        self.transparent_line_buffer
            .write(&self.device, &self.queue, transparent_lines);
        self.rect_buffer
            .write(&self.device, &self.queue, rect_vertices);
        self.text_buffer
            .write(&self.device, &self.queue, text_vertices);
        self.line2d_buffer
            .write(&self.device, &self.queue, line2d_vertices);

        let (output, is_suboptimal) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => (texture, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => (texture, true),
            other => return Err(other),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // --- Pass 1: Draw 3D primitives to low-resolution color/depth texture ---
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Primitive Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.rt_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(c) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: c[0] as f64,
                                g: c[1] as f64,
                                b: c[2] as f64,
                                a: c[3] as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            rpass.set_bind_group(0, &self.primitive_bind_group, &[]);

            // --- Phase 1: Draw opaque primitives ---

            // Draw solids (filled triangles)
            if !triangles.is_empty() {
                rpass.set_pipeline(&self.opaque_triangle_pipeline);
                rpass.set_vertex_buffer(0, self.triangle_buffer.buffer.slice(..));
                rpass.draw(0..triangles.len() as u32, 0..1);
            }

            // Draw lines (wireframe)
            if !lines.is_empty() {
                rpass.set_pipeline(&self.opaque_line_pipeline);
                rpass.set_vertex_buffer(0, self.line_buffer.buffer.slice(..));
                rpass.draw(0..lines.len() as u32, 0..1);
            }

            // Draw instances (solid)
            let mut instance_offset = 0;
            if !instanced_batches.is_empty() {
                rpass.set_pipeline(&self.opaque_instance_pipeline);
                for batch in instanced_batches {
                    let instance_count = batch.instances.len() as u32;
                    if instance_count > 0 {
                        let cached = &self.mesh_cache.get(&batch.shape).unwrap().mesh;
                        rpass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                        rpass.set_vertex_buffer(1, self.instance_data_buffer.buffer.slice(..));
                        rpass.set_index_buffer(
                            cached.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        rpass.draw_indexed(
                            0..cached.index_count,
                            0,
                            instance_offset..(instance_offset + instance_count),
                        );
                        instance_offset += instance_count;
                    }
                }
            }

            // Draw instances (wireframe)
            let mut wire_instance_offset = solid_instance_count as u32;
            if !instanced_wire_batches.is_empty() {
                rpass.set_pipeline(&self.opaque_instance_wire_pipeline);
                for batch in instanced_wire_batches {
                    let instance_count = batch.instances.len() as u32;
                    if instance_count > 0 {
                        let cached = &self.wire_mesh_cache.get(&batch.shape).unwrap().mesh;
                        rpass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                        rpass.set_vertex_buffer(1, self.instance_data_buffer.buffer.slice(..));
                        rpass.set_index_buffer(
                            cached.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        rpass.draw_indexed(
                            0..cached.index_count,
                            0,
                            wire_instance_offset..(wire_instance_offset + instance_count),
                        );
                        wire_instance_offset += instance_count;
                    }
                }
            }

            // --- Phase 2: Draw transparent primitives ---

            // Draw transparent solids (filled triangles)
            if !transparent_triangles.is_empty() {
                rpass.set_pipeline(&self.transparent_triangle_pipeline);
                rpass.set_vertex_buffer(0, self.transparent_triangle_buffer.buffer.slice(..));
                rpass.draw(0..transparent_triangles.len() as u32, 0..1);
            }

            // Draw transparent lines (wireframe)
            if !transparent_lines.is_empty() {
                rpass.set_pipeline(&self.transparent_line_pipeline);
                rpass.set_vertex_buffer(0, self.transparent_line_buffer.buffer.slice(..));
                rpass.draw(0..transparent_lines.len() as u32, 0..1);
            }

            // Draw transparent instances (solid)
            let mut trans_instance_offset = total_opaque_instance_count as u32;
            if !transparent_instanced_batches.is_empty() {
                rpass.set_pipeline(&self.transparent_instance_pipeline);
                for batch in transparent_instanced_batches {
                    let instance_count = batch.instances.len() as u32;
                    if instance_count > 0 {
                        let cached = &self.mesh_cache.get(&batch.shape).unwrap().mesh;
                        rpass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                        rpass.set_vertex_buffer(1, self.instance_data_buffer.buffer.slice(..));
                        rpass.set_index_buffer(
                            cached.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        rpass.draw_indexed(
                            0..cached.index_count,
                            0,
                            trans_instance_offset..(trans_instance_offset + instance_count),
                        );
                        trans_instance_offset += instance_count;
                    }
                }
            }

            // Draw transparent instances (wireframe)
            let mut trans_wire_instance_offset = total_transparent_solid_instance_count as u32;
            if !transparent_instanced_wire_batches.is_empty() {
                rpass.set_pipeline(&self.transparent_instance_wire_pipeline);
                for batch in transparent_instanced_wire_batches {
                    let instance_count = batch.instances.len() as u32;
                    if instance_count > 0 {
                        let cached = &self.wire_mesh_cache.get(&batch.shape).unwrap().mesh;
                        rpass.set_vertex_buffer(0, cached.vertex_buffer.slice(..));
                        rpass.set_vertex_buffer(1, self.instance_data_buffer.buffer.slice(..));
                        rpass.set_index_buffer(
                            cached.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        rpass.draw_indexed(
                            0..cached.index_count,
                            0,
                            trans_wire_instance_offset..(trans_wire_instance_offset + instance_count),
                        );
                        trans_wire_instance_offset += instance_count;
                    }
                }
            }
        }

        // --- Pass 1b: UI rendering pass ---
        // Draw order is determined by CPU-side z_index sorting (flush_sorted), so
        // the depth buffer is not used in this pass.
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.rt_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // Draw rects
            if !rect_vertices.is_empty() {
                rpass.set_pipeline(&self.rect_pipeline);
                rpass.set_bind_group(0, &self.screen_size_bind_group, &[]);
                rpass.set_vertex_buffer(0, self.rect_buffer.buffer.slice(..));
                rpass.draw(0..rect_vertices.len() as u32, 0..1);
            }

            // Draw 2D lines
            if !line2d_vertices.is_empty() {
                rpass.set_pipeline(&self.line2d_pipeline);
                rpass.set_bind_group(0, &self.screen_size_bind_group, &[]);
                rpass.set_vertex_buffer(0, self.line2d_buffer.buffer.slice(..));
                rpass.draw(0..line2d_vertices.len() as u32, 0..1);
            }

            // Draw text
            if !text_vertices.is_empty() {
                rpass.set_pipeline(&self.text_pipeline);
                rpass.set_bind_group(0, &self.screen_size_bind_group, &[]);
                rpass.set_bind_group(1, &self.font_atlas.bind_group, &[]);

                rpass.set_vertex_buffer(0, self.text_buffer.buffer.slice(..));
                rpass.draw(0..text_vertices.len() as u32, 0..1);
            }
        }

        // --- Pass 2: Nearest-neighbor upscale rendering to window surface ---
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Scaler Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear letterbox padding with black
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            rpass.set_pipeline(&self.scaler_pipeline);
            rpass.set_bind_group(0, &self.scaler_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.scaler_vertex_buffer.slice(..));
            rpass.draw(0..6, 0..1); // 6 vertices (quad composed of two triangles)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        if is_suboptimal {
            Ok(RenderResult::Suboptimal)
        } else {
            Ok(RenderResult::Success)
        }
    }

    pub fn font_atlas(&self) -> &FontAtlas {
        &self.font_atlas
    }
}



fn calculate_scale_vertices(vw: u32, vh: u32, ww: u32, wh: u32) -> [ScaleVertex; 6] {
    let vw = vw as f32;
    let vh = vh as f32;
    let ww = ww as f32;
    let wh = wh as f32;

    let target_aspect = vw / vh;
    let window_aspect = ww / wh;

    let (w_ratio, h_ratio) = if window_aspect > target_aspect {
        // Window is wider -> black bars on left and right
        let draw_w = wh * target_aspect;
        (draw_w / ww, 1.0)
    } else {
        // Window is taller -> black bars on top and bottom
        let draw_h = ww / target_aspect;
        (1.0, draw_h / wh)
    };

    // 6 vertices for TriangleList. NDC coordinates (-1..1)
    [
        // 1st triangle (top-left, bottom-left, top-right)
        ScaleVertex {
            position: [-w_ratio, h_ratio],
            tex_coords: [0.0, 0.0],
        },
        ScaleVertex {
            position: [-w_ratio, -h_ratio],
            tex_coords: [0.0, 1.0],
        },
        ScaleVertex {
            position: [w_ratio, h_ratio],
            tex_coords: [1.0, 0.0],
        },
        // 2nd triangle (top-right, bottom-left, bottom-right)
        ScaleVertex {
            position: [w_ratio, h_ratio],
            tex_coords: [1.0, 0.0],
        },
        ScaleVertex {
            position: [-w_ratio, -h_ratio],
            tex_coords: [0.0, 1.0],
        },
        ScaleVertex {
            position: [w_ratio, -h_ratio],
            tex_coords: [1.0, 1.0],
        },
    ]
}

