use glam::{Vec2, Vec3, vec2};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::primback::{
    graphics::FontAtlas,
    systems::{
        audio::{AudioListener, AudioSystem, Synth},
        camera::Camera,
        input::InputState,
        light::Light,
    },
    types::text::Text,
};

/// Context provided to the update lifecycle hook. Contains input state, time tracking, camera control, and audio systems.
pub struct UpdateContext<'a> {
    input: &'a InputState,
    render_size: (u32, u32),
    delta_time: f32,
    time_since_start: f32,
    pub(crate) camera: &'a mut Camera,
    pub(crate) light: &'a mut Light,
    pub(crate) cursor_grab_request: Option<bool>,
    pub(crate) cursor_visible_request: Option<bool>,
    pub(crate) exit_requested: bool,
    pub(crate) audio: &'a mut AudioSystem,
    pub(crate) font: &'a FontAtlas,
}

impl<'a> UpdateContext<'a> {
    /// Creates a new `UpdateContext`.
    pub(crate) fn new(
        input: &'a InputState,
        render_size: (u32, u32),
        delta_time: f32,
        time_since_start: f32,
        camera: &'a mut Camera,
        light: &'a mut Light,
        audio: &'a mut AudioSystem,
        font: &'a FontAtlas,
    ) -> Self {
        Self {
            input,
            render_size,
            delta_time,
            time_since_start,
            camera,
            light,
            cursor_grab_request: None,
            cursor_visible_request: None,
            exit_requested: false,
            audio,
            font,
        }
    }


    /// Gets the delta time in seconds since the last frame.
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Gets the elapsed time in seconds since the start of the application.
    pub fn time_since_start(&self) -> f32 {
        self.time_since_start
    }

    /// Gets the render target resolution size.
    pub fn render_size(&self) -> Vec2 {
        vec2(self.render_size.0 as f32, self.render_size.1 as f32)
    }

    /// Requests the application event loop to terminate.
    pub fn exit(&mut self) {
        self.exit_requested = true;
    }

    /// Returns true if the specified key is currently held down.
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.input.is_key_down(key)
    }

    /// Returns true if the specified key was pressed this frame.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.input.is_key_pressed(key)
    }

    /// Returns true if the specified key was released this frame.
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.input.is_key_released(key)
    }

    /// Gets the current mouse position in render coordinates.
    pub fn mouse_position(&self) -> Vec2 {
        self.input.mouse_position()
    }

    /// Gets the mouse movement delta in render coordinates since the last frame.
    pub fn mouse_delta(&self) -> Vec2 {
        self.input.mouse_delta()
    }

    /// Gets the mouse wheel delta movement since the last frame.
    pub fn mouse_wheel(&self) -> Vec2 {
        self.input.mouse_wheel()
    }

    /// Returns true if the specified mouse button is currently held down.
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.input.is_mouse_button_down(button)
    }

    /// Returns true if the specified mouse button was pressed this frame.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.input.is_mouse_button_pressed(button)
    }

    /// Returns true if the specified mouse button was released this frame.
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.input.is_mouse_button_released(button)
    }

    /// Requests to lock or release the mouse cursor inside the window.
    pub fn set_cursor_grab(&mut self, grab: bool) {
        self.cursor_grab_request = Some(grab);
    }

    /// Sets whether the mouse cursor should be visible.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible_request = Some(visible);
    }

    /// Get a reference to the current Camera.
    pub fn camera(&self) -> &Camera {
        self.camera
    }

    /// Sets the active Camera configuration.
    pub fn set_camera(&mut self, camera: Camera) {
        *self.camera = camera;
    }

    /// Converts a 3D world space coordinate to 2D render coordinates.
    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<Vec2> {
        self.camera.world_to_screen(world_pos, self.render_size())
    }

    /// Converts 2D render coordinates to 3D world space at the specified depth.
    pub fn screen_to_world(&self, screen_pos: Vec2, z_depth: f32) -> Vec3 {
        self.camera
            .screen_to_world(screen_pos, self.render_size(), z_depth)
    }

    /// Converts 2D render coordinates to a 3D point on a plane.
    pub fn screen_to_world_plane(
        &self,
        screen_pos: Vec2,
        plane_normal: Vec3,
        plane_origin: Vec3,
    ) -> Option<Vec3> {
        self.camera.screen_to_world_plane(
            screen_pos,
            self.render_size(),
            plane_normal,
            plane_origin,
        )
    }

    /// Gets a reference to the current Light.
    pub fn light(&self) -> &Light {
        self.light
    }

    /// Sets the active Light configuration.
    pub fn set_light(&mut self, light: Light) {
        *self.light = light;
    }

    /// Plays a synthesizer sound.
    pub fn play_synth(&mut self, synth: impl Into<Synth>) {
        let synth = synth.into();
        let listener = if synth.position.is_some() {
            Some(AudioListener {
                position: self.camera.transform.position,
                right: self.camera.transform.right(),
            })
        } else {
            None
        };
        self.audio.play_synth(synth, listener);
    }

    /// Gets the current master volume scale.
    pub fn master_volume(&self) -> f32 {
        self.audio.master_volume()
    }

    /// Sets the master volume scale.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.audio.set_master_volume(volume);
    }

    /// Measures the size of a given text block based on the current font atlas.
    pub fn measure_text(&self, text: &Text) -> Vec2 {
        self.font.measure_text(text)
    }
}


