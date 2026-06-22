pub mod context;
pub mod graphics;
pub mod systems;
pub mod types;

use std::{sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, WindowId},
};

use crate::{
    FontConfig,
    primback::{
        context::{
            draw::{DrawBuffers, DrawContext},
            update::UpdateContext,
        },
        graphics::{Graphics, RenderParams},
        systems::{audio::AudioSystem, camera::Camera, input::InputState, light::Light},
    },
};
use glam::{Vec3, vec2};

/// A trait that defines the lifecycle callbacks for a Primback application.
pub trait PrimbackApp {
    /// Called once when the application starts, before the first frame update.
    fn init(&mut self, _update_ctx: &mut UpdateContext) {}

    /// Called every frame to update the application state.
    fn update(&mut self, update_ctx: &mut UpdateContext);

    /// Called every frame to draw 3D/2D primitives.
    fn draw(&mut self, draw_ctx: &mut DrawContext);
}

/// Configuration settings for the Primback application window and renderer.
#[derive(Clone, Debug)]
pub struct PrimbackConfig {
    /// The title of the window.
    pub window_title: String,
    /// The physical window width on startup.
    pub window_width: u32,
    /// The physical window height on startup.
    pub window_height: u32,
    /// The render resolution width for independent 2D rendering.
    pub render_width: u32,
    /// The render resolution height for independent 2D rendering.
    pub render_height: u32,
    /// Optional target frame rate limit.
    pub target_fps: Option<u32>,
    /// Whether to enable vertical sync (VSync).
    pub vsync: bool,
    /// The default font configuration used for text rendering.
    pub font: FontConfig,
}

impl Default for PrimbackConfig {
    fn default() -> Self {
        Self {
            window_title: "Primback App".to_string(),
            window_width: 960,
            window_height: 720,
            render_width: 320,
            render_height: 240,
            target_fps: None,
            vsync: true,
            font: FontConfig {
                data: include_bytes!("assets/unscii-8.ttf"),
                native_height: 8.0,
                chars: None,
            },
        }
    }
}

/// The main engine entry point for running a Primback application.
pub struct Primback;

impl Primback {
    /// Launches the Primback application event loop.
    pub fn run<A: PrimbackApp + 'static>(config: PrimbackConfig, app: A) {
        let event_loop = EventLoop::new().expect("failed to create event loop");

        // Poll control flow: run the event loop continuously even if there are no OS events.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut state = PrimbackState::new(config, app);
        event_loop
            .run_app(&mut state)
            .expect("error running event loop")
    }
}

struct PrimbackState<A: PrimbackApp> {
    config: PrimbackConfig,
    app: A,
    window: Option<Arc<winit::window::Window>>,
    graphics: Option<Graphics>,
    input: InputState,
    cursor_locked: bool,
    focused: bool,
    camera: Camera,
    light: Light,
    start_time: Instant,
    last_frame_time: Instant,
    initialized: bool,
    draw_buffers: DrawBuffers,
    audio: AudioSystem,
    target_fps: Option<u32>,
}

impl<A: PrimbackApp> PrimbackState<A> {
    fn new(config: PrimbackConfig, app: A) -> Self {
        let now = Instant::now();
        let camera = Camera::look_at(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y).fovy(45.0);
        let mut target_fps = config.target_fps;
        if let Some(0) = target_fps {
            log::warn!("target_fps is set to Some(0). Treating as None (unlimited FPS).");
            target_fps = None;
        }

        Self {
            config,
            app,
            window: None,
            graphics: None,
            input: InputState::new(),
            cursor_locked: false,
            focused: true,
            camera,
            light: Light::default(),
            start_time: now,
            last_frame_time: now,
            initialized: false,
            draw_buffers: DrawBuffers::new(),
            audio: AudioSystem::new(),
            target_fps,
        }
    }
}

impl<A: PrimbackApp> ApplicationHandler for PrimbackState<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // If already initialized, do nothing.
        if self.window.is_some() {
            return;
        }

        // Window initialization.
        let window_attributes = winit::window::Window::default_attributes()
            .with_title(&self.config.window_title)
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.config.window_width,
                self.config.window_height,
            ));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("failed to create window"),
        );
        self.window = Some(window.clone());

        // TODO: Web support branching.
        let graphics = pollster::block_on(Graphics::new(
            window,
            self.config.render_width,
            self.config.render_height,
            self.config.vsync,
            &self.config.font,
        ));
        self.graphics = Some(graphics);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let window = match &self.window {
            Some(w) => w,
            None => return,
        };

        if window.id() != id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                let graphics = match &mut self.graphics {
                    Some(g) => g,
                    None => return,
                };

                // Sleep to match target FPS if configured.
                if self.initialized
                    && let Some(target_fps) = self.target_fps
                {
                    use std::time::Duration;

                    let target_duration = Duration::from_secs_f64(1.0 / target_fps as f64);
                    let elapsed = Instant::now().duration_since(self.last_frame_time);
                    if elapsed < target_duration {
                        let sleep_duration = target_duration - elapsed;
                        let sleep_start = Instant::now();

                        if Duration::from_millis(2) < sleep_duration {
                            std::thread::sleep(sleep_duration - Duration::from_millis(1));
                        }

                        while Instant::now().duration_since(sleep_start) < sleep_duration {
                            std::thread::yield_now();
                        }
                    }
                }

                // Update times.
                let now = Instant::now();
                if !self.initialized {
                    self.start_time = now;
                    self.last_frame_time = now;
                }
                let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;
                let time_since_start = now.duration_since(self.start_time).as_secs_f32();

                // Update phase.
                {
                    let mut update_ctx = UpdateContext::new(
                        &self.input,
                        (self.config.render_width, self.config.render_height),
                        delta_time,
                        time_since_start,
                        &mut self.camera,
                        &mut self.light,
                        &mut self.audio,
                        graphics.font_atlas(),
                    );

                    // Initialization on first frame.
                    if !self.initialized {
                        self.app.init(&mut update_ctx);
                        self.initialized = true;
                    }

                    self.app.update(&mut update_ctx);

                    // Check exit request.
                    if update_ctx.exit_requested {
                        event_loop.exit();
                        return;
                    }

                    // Handle cursor lock/visibility.
                    if let Some(grab) = update_ctx.cursor_grab_request {
                        let mode = if grab {
                            CursorGrabMode::Locked
                        } else {
                            CursorGrabMode::None
                        };

                        if let Err(_) = window.set_cursor_grab(mode) {
                            if grab {
                                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                            }
                        }

                        self.cursor_locked = grab;
                    }
                    if let Some(visible) = update_ctx.cursor_visible_request {
                        window.set_cursor_visible(visible);
                    }
                }

                self.audio.update(delta_time);

                // Draw phase.
                {
                    let buffers = std::mem::take(&mut self.draw_buffers);

                    let mut draw_ctx = DrawContext::new_with_buffers(
                        (self.config.render_width, self.config.render_height),
                        time_since_start,
                        self.camera,
                        self.light,
                        buffers,
                        graphics.font_atlas(),
                    );

                    self.app.draw(&mut draw_ctx);

                    // Sort all queued 2D UI commands by z_index (back to front)
                    // before building the vertex buffers sent to the GPU.
                    draw_ctx.flush_sorted();

                    let clear_color = draw_ctx.clear_color;
                    let camera = draw_ctx.camera;
                    let light = draw_ctx.light;

                    // Consume draw_ctx to release graphics borrow
                    let buffers = draw_ctx.take_buffers();

                    let render_result = graphics.render(RenderParams {
                        triangles: &buffers.triangles,
                        transparent_triangles: &buffers.transparent_triangles,
                        lines: &buffers.lines,
                        transparent_lines: &buffers.transparent_lines,
                        rect_vertices: &buffers.rect_vertices,
                        text_vertices: &buffers.text_vertices,
                        line2d_vertices: &buffers.line2d_vertices,
                        instanced_batches: &buffers.instanced_batches,
                        transparent_instanced_batches: &buffers.transparent_instanced_batches,
                        instanced_wire_batches: &buffers.instanced_wire_batches,
                        transparent_instanced_wire_batches: &buffers.transparent_instanced_wire_batches,
                        clear_color,
                        camera: &camera,
                        light_dir: light.direction,
                        light_color: [light.color.r, light.color.g, light.color.b],
                        light_ambient: [light.ambient.r, light.ambient.g, light.ambient.b],
                    });

                    match render_result {
                        Ok(crate::primback::graphics::RenderResult::Success) => {}
                        Ok(crate::primback::graphics::RenderResult::Suboptimal) => {
                            graphics.resize(window.inner_size());
                        }
                        Err(wgpu::CurrentSurfaceTexture::Lost)
                        | Err(wgpu::CurrentSurfaceTexture::Outdated) => {
                            graphics.resize(window.inner_size());
                        }
                        Err(e) => {
                            log::error!("Render error: {:?}", e);
                        }
                    }

                    self.draw_buffers = buffers;
                };

                self.input.clear_frame_states();
            }
            WindowEvent::Focused(focused) => {
                self.focused = focused;
                if !focused {
                    self.input.clear_all_states();
                    if self.cursor_locked {
                        if let Some(window) = &self.window {
                            let _ = window.set_cursor_grab(CursorGrabMode::None);
                        }
                    }
                } else {
                    if self.cursor_locked {
                        if let Some(window) = &self.window {
                            let mode = CursorGrabMode::Locked;
                            if let Err(_) = window.set_cursor_grab(mode) {
                                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                            }
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if self.focused {
                    self.input.handle_key_event(&event);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.focused && !self.cursor_locked {
                    if let Some(graphics) = &self.graphics {
                        let (ww, wh) = graphics.window_size();
                        let ww = ww as f32;
                        let wh = wh as f32;
                        let vw = self.config.render_width as f32;
                        let vh = self.config.render_height as f32;

                        let target_aspect = vw / vh;
                        let window_aspect = ww / wh;

                        let (w_ratio, h_ratio) = if window_aspect > target_aspect {
                            let draw_w = wh * target_aspect;
                            (draw_w / ww, 1.0)
                        } else {
                            let draw_h = ww / target_aspect;
                            (1.0, draw_h / wh)
                        };

                        let ndc_x = (position.x as f32 / ww) * 2.0 - 1.0;
                        let ndc_y = 1.0 - (position.y as f32 / wh) * 2.0;

                        // Calculate normalized coordinates [0, 1] within the letterbox.
                        let norm_x = ((ndc_x + w_ratio) / (2.0 * w_ratio)).clamp(0.0, 1.0);
                        let norm_y = ((h_ratio - ndc_y) / (2.0 * h_ratio)).clamp(0.0, 1.0);

                        let vx = norm_x * vw;
                        let vy = norm_y * vh;

                        self.input.set_mouse_position_and_delta(vec2(vx, vy));
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if self.focused {
                    self.input.handle_mouse_input(button, state);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if self.focused {
                    self.input.handle_mouse_wheel(delta);
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if self.focused && self.cursor_locked {
            if let winit::event::DeviceEvent::MouseMotion { delta } = event {
                // Scale mouse motion delta relative to render screen aspect ratio.
                let scale_x = if let Some(graphics) = &self.graphics {
                    let (ww, _) = graphics.window_size();
                    self.config.render_width as f32 / ww as f32
                } else {
                    1.0
                };
                let scale_y = if let Some(graphics) = &self.graphics {
                    let (_, wh) = graphics.window_size();
                    self.config.render_height as f32 / wh as f32
                } else {
                    1.0
                };

                self.input
                    .add_mouse_delta(vec2(delta.0 as f32 * scale_x, delta.1 as f32 * scale_y));
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Send next frame redraw request in Poll mode.
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
