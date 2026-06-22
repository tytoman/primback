use crate::primback::{
    graphics::{
        InstancedBatch, InstancedWireBatch, FontAtlas,
        vertices::{InstanceData, Line2DVertex, RectVertex, TextVertex, Vertex3D},
    },
    systems::{camera::Camera, light::Light},
    types::{
        color::{Color, ColorWithAlpha},
        rect::Rect,
        rect_transform::{Anchor, RectTransform},
        shape::{
            Shape,
            geometry::{generate_solid_geometry, generate_wire_geometry},
        },
        text::{Text},
        transform::WorldTransform,
    },
};
use glam::{Vec2, Vec3, vec2};


/// A single 2D UI draw command queued for z_index-sorted rendering.
enum DrawCommand {
    /// A UI rectangle (6 vertices forming a quad).
    Rect {
        vertices: [RectVertex; 6],
        z_index: f32,
    },
    /// A 2D line segment (2 vertices).
    Line2D {
        p1: Line2DVertex,
        p2: Line2DVertex,
        z_index: f32,
    },
    /// A text glyph batch (6 vertices per glyph, all with the same z_index).
    Text {
        vertices: Vec<TextVertex>,
        z_index: f32,
    },
}

/// Internal buffers that store all geometry vertices and batches to be rendered.
#[derive(Default)]
pub struct DrawBuffers {
    pub(crate) triangles: Vec<Vertex3D>,
    pub(crate) transparent_triangles: Vec<Vertex3D>,
    pub(crate) lines: Vec<Vertex3D>,
    pub(crate) transparent_lines: Vec<Vertex3D>,
    pub(crate) text_vertices: Vec<TextVertex>,
    pub(crate) rect_vertices: Vec<RectVertex>,
    pub(crate) line2d_vertices: Vec<Line2DVertex>,
    pub(crate) instanced_batches: Vec<InstancedBatch>,
    pub(crate) transparent_instanced_batches: Vec<InstancedBatch>,
    pub(crate) instanced_wire_batches: Vec<InstancedWireBatch>,
    pub(crate) transparent_instanced_wire_batches: Vec<InstancedWireBatch>,
}

impl DrawBuffers {
    /// Creates a new empty set of draw buffers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all vertices and instances from the buffers.
    pub fn clear(&mut self) {
        self.triangles.clear();
        self.transparent_triangles.clear();
        self.lines.clear();
        self.transparent_lines.clear();
        self.text_vertices.clear();
        self.rect_vertices.clear();
        self.line2d_vertices.clear();
        self.instanced_batches.clear();
        self.transparent_instanced_batches.clear();
        self.instanced_wire_batches.clear();
        self.transparent_instanced_wire_batches.clear();
    }
}

/// A context for issuing 2D and 3D drawing commands.
pub struct DrawContext<'a> {
    pub(crate) buffers: DrawBuffers,
    pub(crate) clear_color: Option<[f32; 4]>,

    // Queued 2D UI draw commands pending z_index sort.
    draw_commands: Vec<DrawCommand>,

    // Camera parameters
    pub(crate) camera: Camera,

    // Light parameters
    pub(crate) light: Light,

    render_size: (u32, u32),
    time_since_start: f32,

    pub(crate) font: &'a FontAtlas,
}


impl<'a> DrawContext<'a> {
    /// Creates a new `DrawContext` using the provided buffers.
    pub(crate) fn new_with_buffers(
        render_size: (u32, u32),
        time_since_start: f32,
        camera: Camera,
        light: Light,
        mut buffers: DrawBuffers,
        font: &'a FontAtlas,
    ) -> Self {
        buffers.clear();
        Self {
            buffers,
            clear_color: None,

            draw_commands: Vec::new(),

            camera,
            light,

            render_size,
            time_since_start,
            font,
        }
    }


    /// Consumes the context and returns the internal draw buffers for reuse.
    pub(crate) fn take_buffers(self) -> DrawBuffers {
        self.buffers
    }

    /// This must be called before passing vertex slices to the GPU.
    pub(crate) fn flush_sorted(&mut self) {
        // --- Sort 2D UI draw commands ---
        self.draw_commands
            .sort_by(|a, b| Self::z_index_of(a).total_cmp(&Self::z_index_of(b)));

        for cmd in self.draw_commands.drain(..) {
            match cmd {
                DrawCommand::Rect { vertices, .. } => {
                    self.buffers.rect_vertices.extend_from_slice(&vertices);
                }
                DrawCommand::Line2D { p1, p2, .. } => {
                    self.buffers.line2d_vertices.push(p1);
                    self.buffers.line2d_vertices.push(p2);
                }
                DrawCommand::Text { vertices, .. } => {
                    self.buffers.text_vertices.extend(vertices);
                }
            }
        }

        // --- Sort 3D transparent primitives (back-to-front) ---
        let camera_pos = self.camera.transform.position;

        // 1. Transparent triangles
        if !self.buffers.transparent_triangles.is_empty() {
            let mut triangles_with_depth = Vec::with_capacity(self.buffers.transparent_triangles.len() / 3);
            let num_triangles = self.buffers.transparent_triangles.len() / 3;
            for i in 0..num_triangles {
                let v0 = self.buffers.transparent_triangles[i * 3];
                let v1 = self.buffers.transparent_triangles[i * 3 + 1];
                let v2 = self.buffers.transparent_triangles[i * 3 + 2];

                let p0 = glam::Vec3::from(v0.position);
                let p1 = glam::Vec3::from(v1.position);
                let p2 = glam::Vec3::from(v2.position);
                let centroid = (p0 + p1 + p2) / 3.0;

                let dist_sq = (centroid - camera_pos).length_squared();
                triangles_with_depth.push((dist_sq, [v0, v1, v2]));
            }

            // Sort from back to front (descending order of distance)
            triangles_with_depth.sort_by(|a, b| b.0.total_cmp(&a.0));

            self.buffers.transparent_triangles.clear();
            for (_, tri) in triangles_with_depth {
                self.buffers.transparent_triangles.extend_from_slice(&tri);
            }
        }

        // 2. Transparent lines
        if !self.buffers.transparent_lines.is_empty() {
            let mut lines_with_depth = Vec::with_capacity(self.buffers.transparent_lines.len() / 2);
            let num_lines = self.buffers.transparent_lines.len() / 2;
            for i in 0..num_lines {
                let v0 = self.buffers.transparent_lines[i * 2];
                let v1 = self.buffers.transparent_lines[i * 2 + 1];

                let p0 = glam::Vec3::from(v0.position);
                let p1 = glam::Vec3::from(v1.position);
                let centroid = (p0 + p1) / 2.0;

                let dist_sq = (centroid - camera_pos).length_squared();
                lines_with_depth.push((dist_sq, [v0, v1]));
            }

            // Sort from back to front
            lines_with_depth.sort_by(|a, b| b.0.total_cmp(&a.0));

            self.buffers.transparent_lines.clear();
            for (_, line) in lines_with_depth {
                self.buffers.transparent_lines.extend_from_slice(&line);
            }
        }

        // 3. Transparent instanced batches
        for batch in &mut self.buffers.transparent_instanced_batches {
            batch.instances.sort_by(|a, b| {
                let pos_a = glam::Vec3::new(
                    a.model_matrix[3][0],
                    a.model_matrix[3][1],
                    a.model_matrix[3][2],
                );
                let pos_b = glam::Vec3::new(
                    b.model_matrix[3][0],
                    b.model_matrix[3][1],
                    b.model_matrix[3][2],
                );

                let dist_a = (pos_a - camera_pos).length_squared();
                let dist_b = (pos_b - camera_pos).length_squared();

                // Sort from back to front
                dist_b.total_cmp(&dist_a)
            });
        }

        // 4. Transparent instanced wire batches
        for batch in &mut self.buffers.transparent_instanced_wire_batches {
            batch.instances.sort_by(|a, b| {
                let pos_a = glam::Vec3::new(
                    a.model_matrix[3][0],
                    a.model_matrix[3][1],
                    a.model_matrix[3][2],
                );
                let pos_b = glam::Vec3::new(
                    b.model_matrix[3][0],
                    b.model_matrix[3][1],
                    b.model_matrix[3][2],
                );

                let dist_a = (pos_a - camera_pos).length_squared();
                let dist_b = (pos_b - camera_pos).length_squared();

                // Sort from back to front
                dist_b.total_cmp(&dist_a)
            });
        }
    }

    fn get_rgba(&self, color: impl Into<ColorWithAlpha>) -> [f32; 4] {
        let c_alpha = color.into();
        [
            c_alpha.color.r,
            c_alpha.color.g,
            c_alpha.color.b,
            c_alpha.alpha,
        ]
    }

    fn z_index_of(cmd: &DrawCommand) -> f32 {
        match cmd {
            DrawCommand::Rect { z_index, .. } => *z_index,
            DrawCommand::Line2D { z_index, .. } => *z_index,
            DrawCommand::Text { z_index, .. } => *z_index,
        }
    }

    /// Gets the elapsed time in seconds since the application started.
    pub fn time_since_start(&self) -> f32 {
        self.time_since_start
    }

    /// Gets the render target resolution size.
    pub fn render_size(&self) -> Vec2 {
        vec2(self.render_size.0 as f32, self.render_size.1 as f32)
    }

    /// Clears the screen with the specified color. The alpha component is opaque (1.0).
    pub fn clear(&mut self, color: Color) {
        self.clear_color = Some([color.r, color.g, color.b, 1.0]);
    }

    /// Gets a reference to the current Camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Gets a reference to the current Light.
    pub fn light(&self) -> &Light {
        &self.light
    }

    /// Queues a 3D primitive shape for instanced rendering.
    ///
    /// # Note
    /// Transparent instances are only sorted within the same shape batch.
    /// Mixing different transparent shapes may cause rendering/depth order artifacts.
    pub fn draw_shape_instanced(
        &mut self,
        shape: impl Into<Shape>,
        transform: impl Into<WorldTransform>,
        color: impl Into<ColorWithAlpha>,
    ) {
        let shape = shape.into();
        let transform = transform.into();
        let color = color.into();
        let rgba = self.get_rgba(color);

        let mat = transform.matrix;
        let normal_matrix = glam::Mat3::from_mat4(mat.inverse().transpose());

        let instance = InstanceData {
            model_matrix: mat.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
            color: rgba,
        };

        let is_transparent = rgba[3] < 1.0;
        let batches = if is_transparent {
            &mut self.buffers.transparent_instanced_batches
        } else {
            &mut self.buffers.instanced_batches
        };

        if let Some(batch) = batches.iter_mut().find(|b| b.shape == shape) {
            batch.instances.push(instance);
        } else {
            batches.push(InstancedBatch {
                shape,
                instances: vec![instance],
            });
        }
    }

    /// Queues a 3D wireframe shape for instanced rendering.
    ///
    /// # Note
    /// Transparent instances are only sorted within the same shape batch.
    /// Mixing different transparent shapes may cause rendering/depth order artifacts.
    pub fn draw_shape_wire_instanced(
        &mut self,
        shape: impl Into<Shape>,
        transform: impl Into<WorldTransform>,
        color: impl Into<ColorWithAlpha>,
    ) {
        let shape = shape.into();
        let transform = transform.into();
        let color = color.into();
        let rgba = self.get_rgba(color);

        let mat = transform.matrix;
        let normal_matrix = glam::Mat3::from_mat4(mat.inverse().transpose());

        let instance = InstanceData {
            model_matrix: mat.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
            color: rgba,
        };

        let is_transparent = rgba[3] < 1.0;
        let batches = if is_transparent {
            &mut self.buffers.transparent_instanced_wire_batches
        } else {
            &mut self.buffers.instanced_wire_batches
        };

        if let Some(batch) = batches.iter_mut().find(|b| b.shape == shape) {
            batch.instances.push(instance);
        } else {
            batches.push(InstancedWireBatch {
                shape,
                instances: vec![instance],
            });
        }
    }

    /// Draws a filled 3D shape.
    ///
    /// # Note
    /// This function generates all vertices on the CPU every frame.
    /// For drawing many identical shapes, use [`DrawContext::draw_shape_instanced`] instead.
    pub fn draw_shape(
        &mut self,
        shape: impl Into<Shape>,
        transform: impl Into<WorldTransform>,
        color: impl Into<ColorWithAlpha>,
    ) {
        let transform = transform.into();
        let shape = shape.into();
        let color = self.get_rgba(color);

        let (vertices, indices) = generate_solid_geometry(&shape);

        let mat = transform.matrix;
        let normal_matrix = mat.inverse().transpose();

        let target_triangles = if color[3] < 1.0 {
            &mut self.buffers.transparent_triangles
        } else {
            &mut self.buffers.triangles
        };

        for &idx in &indices {
            let v = &vertices[idx as usize];
            let world_pos = mat.transform_point3(Vec3::from(v.position));
            let world_normal = normal_matrix.transform_vector3(Vec3::from(v.normal)).normalize_or_zero();
            target_triangles.push(Vertex3D {
                position: world_pos.into(),
                normal: world_normal.into(),
                color,
            });
        }
    }

    /// Draws a 3D wireframe shape.
    ///
    /// # Note
    /// This function generates all vertices on the CPU every frame.
    /// For drawing many identical shapes, use [`DrawContext::draw_shape_wire_instanced`] instead.
    pub fn draw_shape_wire(
        &mut self,
        shape: impl Into<Shape>,
        transform: impl Into<WorldTransform>,
        color: impl Into<ColorWithAlpha>,
    ) {
        let transform = transform.into();
        let shape = shape.into();
        let color = self.get_rgba(color);

        let (vertices, indices) = generate_wire_geometry(&shape);

        let mat = transform.matrix;
        let normal_matrix = mat.inverse().transpose();

        let target_lines = if color[3] < 1.0 {
            &mut self.buffers.transparent_lines
        } else {
            &mut self.buffers.lines
        };

        for &idx in &indices {
            let v = &vertices[idx as usize];
            let world_pos = mat.transform_point3(Vec3::from(v.position));
            let world_normal = normal_matrix.transform_vector3(Vec3::from(v.normal)).normalize_or_zero();
            target_lines.push(Vertex3D {
                position: world_pos.into(),
                normal: world_normal.into(),
                color,
            });
        }
    }

    /// Draws a filled 3D triangle. Normal is calculated automatically.
    pub fn draw_triangle(
        &mut self,
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        color: impl Into<ColorWithAlpha>,
    ) {
        let color = self.get_rgba(color);
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2).normalize_or_zero().into();

        let target_triangles = if color[3] < 1.0 {
            &mut self.buffers.transparent_triangles
        } else {
            &mut self.buffers.triangles
        };

        target_triangles.push(Vertex3D {
            position: p1.into(),
            normal,
            color,
        });
        target_triangles.push(Vertex3D {
            position: p2.into(),
            normal,
            color,
        });
        target_triangles.push(Vertex3D {
            position: p3.into(),
            normal,
            color,
        });
    }

    /// Draws a 3D wireframe triangle.
    pub fn draw_triangle_wire(
        &mut self,
        p1: Vec3,
        p2: Vec3,
        p3: Vec3,
        color: impl Into<ColorWithAlpha>,
    ) {
        let color = color.into();
        self.draw_line_3d(p1, p2, color);
        self.draw_line_3d(p2, p3, color);
        self.draw_line_3d(p3, p1, color);
    }

    /// Draws a line segment in 3D space.
    pub fn draw_line_3d(&mut self, p1: Vec3, p2: Vec3, color: impl Into<ColorWithAlpha>) {
        let color = self.get_rgba(color);
        let normal = [0.0, 0.0, 0.0];
        let target_lines = if color[3] < 1.0 {
            &mut self.buffers.transparent_lines
        } else {
            &mut self.buffers.lines
        };
        target_lines.push(Vertex3D {
            position: p1.into(),
            normal,
            color,
        });
        target_lines.push(Vertex3D {
            position: p2.into(),
            normal,
            color,
        });
    }

    /// Draws a UI rectangle in 2D space (screen coordinates, top-left is 0, 0).
    pub fn draw_rect(&mut self, rect: impl Into<Rect>, transform: impl Into<RectTransform>) {
        let rect = rect.into();
        let transform = transform.into();
        let size = rect.size;

        let outline_width = rect.outline_width.unwrap_or(0.0);
        let outline_color = self.get_rgba(rect.outline_color.unwrap_or(Color::BLACK.into()));

        let shadow_offset = rect.shadow_offset.unwrap_or(Vec2::ZERO);
        let mut shadow_color = self.get_rgba(rect.shadow_color.unwrap_or(Color::BLACK.into()));
        // If shadow is not enabled (no offset specified), set color to fully transparent (alpha 0.0)
        if rect.shadow_offset.is_none() {
            shadow_color[3] = 0.0;
        }

        // 1. Calculate various sizes after applying scale
        let scale_x = transform.scale.x;
        let scale_y = transform.scale.y;
        let scale_factor = (scale_x + scale_y) * 0.5;

        let scaled_size = size * transform.scale;
        let scaled_radius = [
            rect.radius[0] * scale_factor,
            rect.radius[1] * scale_factor,
            rect.radius[2] * scale_factor,
            rect.radius[3] * scale_factor,
        ];
        let scaled_outline_width = outline_width * scale_factor;
        let scaled_shadow_offset = shadow_offset * transform.scale;

        // 2. Calculate margin based on scaled shadow offset
        let margin_left = ((-scaled_shadow_offset.x).max(0.0) + 2.0).max(0.0);
        let margin_right = ((scaled_shadow_offset.x).max(0.0) + 2.0).max(0.0);
        let margin_top = ((-scaled_shadow_offset.y).max(0.0) + 2.0).max(0.0);
        let margin_bottom = ((scaled_shadow_offset.y).max(0.0) + 2.0).max(0.0);

        // 3. Local coordinates relative to center for SDF calculation (lx, ly)
        let lx0 = -scaled_size.x * 0.5 - margin_left;
        let lx1 = scaled_size.x * 0.5 + margin_right;
        let ly0 = -scaled_size.y * 0.5 - margin_top;
        let ly1 = scaled_size.y * 0.5 + margin_bottom;

        // 4. Translation offset by anchor (displacement from center (0, 0))
        let anchor_shift_x = match rect.anchor_x {
            Anchor::Start => scaled_size.x * 0.5,
            Anchor::Center => 0.0,
            Anchor::End => -scaled_size.x * 0.5,
        };
        let anchor_shift_y = match rect.anchor_y {
            Anchor::Start => scaled_size.y * 0.5,
            Anchor::Center => 0.0,
            Anchor::End => -scaled_size.y * 0.5,
        };

        // 5. Local vertex position for affine transformation (scale is already applied, so apply only rotation and translation)
        let lp0 = vec2(lx0 + anchor_shift_x, ly0 + anchor_shift_y);
        let lp1 = vec2(lx0 + anchor_shift_x, ly1 + anchor_shift_y);
        let lp2 = vec2(lx1 + anchor_shift_x, ly0 + anchor_shift_y);
        let lp3 = vec2(lx1 + anchor_shift_x, ly1 + anchor_shift_y);

        let transform_point = |lp: Vec2| -> Vec2 {
            transform.to_world_point(lp)
        };

        let p0 = transform_point(lp0);
        let p1 = transform_point(lp1);
        let p2 = transform_point(lp2);
        let p3 = transform_point(lp3);

        // 6. Round coordinates to nearest integer to avoid rendering blur
        let p0_r = p0.round();
        let p1_r = p1.round();
        let p2_r = p2.round();
        let p3_r = p3.round();

        let color = self.get_rgba(rect.color);
        let z_index = transform.z_index;

        let vertices = [
            RectVertex {
                position: [p0_r.x, p0_r.y],
                local_pos: [lx0, ly0],
                size: [scaled_size.x, scaled_size.y],
                color,
                radius: scaled_radius,
                outline_width: scaled_outline_width,
                outline_color,
                shadow_offset: [scaled_shadow_offset.x, scaled_shadow_offset.y],
                shadow_color,
                z_index,
            },
            RectVertex {
                position: [p1_r.x, p1_r.y],
                local_pos: [lx0, ly1],
                size: [scaled_size.x, scaled_size.y],
                color,
                radius: scaled_radius,
                outline_width: scaled_outline_width,
                outline_color,
                shadow_offset: [scaled_shadow_offset.x, scaled_shadow_offset.y],
                shadow_color,
                z_index,
            },
            RectVertex {
                position: [p2_r.x, p2_r.y],
                local_pos: [lx1, ly0],
                size: [scaled_size.x, scaled_size.y],
                color,
                radius: scaled_radius,
                outline_width: scaled_outline_width,
                outline_color,
                shadow_offset: [scaled_shadow_offset.x, scaled_shadow_offset.y],
                shadow_color,
                z_index,
            },
            RectVertex {
                position: [p2_r.x, p2_r.y],
                local_pos: [lx1, ly0],
                size: [scaled_size.x, scaled_size.y],
                color,
                radius: scaled_radius,
                outline_width: scaled_outline_width,
                outline_color,
                shadow_offset: [scaled_shadow_offset.x, scaled_shadow_offset.y],
                shadow_color,
                z_index,
            },
            RectVertex {
                position: [p1_r.x, p1_r.y],
                local_pos: [lx0, ly1],
                size: [scaled_size.x, scaled_size.y],
                color,
                radius: scaled_radius,
                outline_width: scaled_outline_width,
                outline_color,
                shadow_offset: [scaled_shadow_offset.x, scaled_shadow_offset.y],
                shadow_color,
                z_index,
            },
            RectVertex {
                position: [p3_r.x, p3_r.y],
                local_pos: [lx1, ly1],
                size: [scaled_size.x, scaled_size.y],
                color,
                radius: scaled_radius,
                outline_width: scaled_outline_width,
                outline_color,
                shadow_offset: [scaled_shadow_offset.x, scaled_shadow_offset.y],
                shadow_color,
                z_index,
            },
        ];

        self.draw_commands
            .push(DrawCommand::Rect { vertices, z_index });
    }

    /// Draws a line segment in 2D space (screen coordinates, top-left is 0, 0).
    pub fn draw_line_2d(
        &mut self,
        p1: Vec2,
        p2: Vec2,
        color: impl Into<ColorWithAlpha>,
        z_index: f32,
    ) {
        let color = self.get_rgba(color);
        let v1 = Line2DVertex {
            position: p1.into(),
            color,
            z_index,
        };
        let v2 = Line2DVertex {
            position: p2.into(),
            color,
            z_index,
        };
        self.draw_commands.push(DrawCommand::Line2D {
            p1: v1,
            p2: v2,
            z_index,
        });
    }

    /// Draws UI text in 2D space (screen coordinates, top-left is 0, 0).
    /// Measures the size of a given text block based on the current font atlas.
    pub fn measure_text(&self, text: &Text) -> Vec2 {
        self.font.measure_text(text)
    }


    /// Draws UI text in 2D space (screen coordinates, top-left is 0, 0).
    pub fn draw_text(
        &mut self,
        text: impl Into<Text>,
        transform: impl Into<RectTransform>,
        color: impl Into<ColorWithAlpha>,
    ) {
        let text_obj = text.into();
        let trans_obj = transform.into();

        if text_obj.shadow_offset.is_some() || text_obj.shadow_color.is_some() {
            let offset = text_obj.shadow_offset.unwrap_or(vec2(1.0, 1.0));
            let shadow_color = text_obj
                .shadow_color
                .unwrap_or(ColorWithAlpha::from(Color::BLACK));

            let mut shadow_text = text_obj.clone();
            shadow_text.shadow_offset = None;
            shadow_text.shadow_color = None;

            let mut shadow_trans = trans_obj;
            // Shift shadow position considering parent scale and rotation.
            shadow_trans.position += offset;
            self.draw_text(shadow_text, shadow_trans, shadow_color);
        }

        let color = self.get_rgba(color);
        let char_h = self.font.native_height * text_obj.scale;

        // Apply word wrap / character wrap considering max_width
        let mut lines = Vec::new();
        for original_line in text_obj.content.split('\n') {
            if let Some(max_w) = text_obj.max_width {
                if max_w <= 0.0 {
                    log::warn!("text_obj.max_width in draw_text is <= 0.0 ({}). Skipping wrapping.", max_w);
                    lines.push(original_line.to_string());
                } else {
                    let mut current_sub_line = String::new();
                    let mut current_sub_w = 0.0;
                    for c in original_line.chars() {
                        let char_w = if let Some(glyph) = self.font.glyph(c) {
                            glyph.advance * text_obj.scale
                        } else {
                            self.font.native_height * text_obj.scale
                        };
                        if current_sub_w + char_w > max_w && !current_sub_line.is_empty() {
                            lines.push(current_sub_line);
                            current_sub_line = String::new();
                            current_sub_w = 0.0;
                        }
                        current_sub_w += char_w;
                        current_sub_line.push(c);
                    }
                    lines.push(current_sub_line);
                }
            } else {
                lines.push(original_line.to_string());
            }
        }

        let line_count = lines.len();
        let total_h = line_count as f32 * char_h;
        let offset_y = match text_obj.anchor_y {
            Anchor::Start => 0.0,
            Anchor::Center => -total_h / 2.0,
            Anchor::End => -total_h,
        };

        let z_index = trans_obj.z_index;
        let mut glyph_vertices: Vec<TextVertex> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let mut line_w = 0.0;
            for c in line.chars() {
                if let Some(glyph) = self.font.glyph(c) {
                    line_w += glyph.advance * text_obj.scale;
                } else {
                    line_w += self.font.native_height * text_obj.scale;
                }
            }

            let line_start_x = match text_obj.anchor_x {
                Anchor::Start => 0.0,
                Anchor::Center => -line_w / 2.0,
                Anchor::End => -line_w,
            };

            let mut curr_x = line_start_x;
            let curr_row_top = offset_y + (i as f32) * char_h;
            let base_y = curr_row_top + self.font.ascent * text_obj.scale;

            for c in line.chars() {
                if let Some(glyph) = self.font.glyph(c) {
                    if glyph.width > 0.0 && glyph.height > 0.0 {
                        let lx0 = curr_x + glyph.xmin * text_obj.scale;
                        let ly0 = base_y - (glyph.ymin + glyph.height) * text_obj.scale;
                        let lx1 = lx0 + glyph.width * text_obj.scale;
                        let ly1 = ly0 + glyph.height * text_obj.scale;

                        let u_min = glyph.uv_min.x;
                        let v_min = glyph.uv_min.y;
                        let u_max = glyph.uv_max.x;
                        let v_max = glyph.uv_max.y;

                        // Affine transform vertex coordinates.
                        let p0 = trans_obj.to_world_point(vec2(lx0, ly0));
                        let p1 = trans_obj.to_world_point(vec2(lx0, ly1));
                        let p2 = trans_obj.to_world_point(vec2(lx1, ly0));
                        let p3 = trans_obj.to_world_point(vec2(lx1, ly1));

                        // Round coordinates to nearest integer to avoid text rendering blur.
                        let p0_r = p0.round();
                        let p1_r = p1.round();
                        let p2_r = p2.round();
                        let p3_r = p3.round();

                        // Top-Left
                        glyph_vertices.push(TextVertex {
                            position: [p0_r.x, p0_r.y],
                            tex_coords: [u_min, v_min],
                            color,
                            z_index,
                        });
                        // Bottom-Left
                        glyph_vertices.push(TextVertex {
                            position: [p1_r.x, p1_r.y],
                            tex_coords: [u_min, v_max],
                            color,
                            z_index,
                        });
                        // Top-Right
                        glyph_vertices.push(TextVertex {
                            position: [p2_r.x, p2_r.y],
                            tex_coords: [u_max, v_min],
                            color,
                            z_index,
                        });

                        // Top-Right
                        glyph_vertices.push(TextVertex {
                            position: [p2_r.x, p2_r.y],
                            tex_coords: [u_max, v_min],
                            color,
                            z_index,
                        });
                        // Bottom-Left
                        glyph_vertices.push(TextVertex {
                            position: [p1_r.x, p1_r.y],
                            tex_coords: [u_min, v_max],
                            color,
                            z_index,
                        });
                        // Bottom-Right
                        glyph_vertices.push(TextVertex {
                            position: [p3_r.x, p3_r.y],
                            tex_coords: [u_max, v_max],
                            color,
                            z_index,
                        });
                    }

                    curr_x += glyph.advance * text_obj.scale;
                } else {
                    // Fallback for character not found in atlas: treat as space
                    curr_x += self.font.native_height * text_obj.scale;
                }
            }
        }

        if !glyph_vertices.is_empty() {
            self.draw_commands.push(DrawCommand::Text {
                vertices: glyph_vertices,
                z_index,
            });
        }
    }

}
