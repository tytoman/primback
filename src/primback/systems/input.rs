use std::collections::HashSet;
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
use glam::Vec2;

/// Manages keyboard and mouse input states, including click states and movements.
pub struct InputState {
    keys_held: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,

    // Mouse position in virtual screen coordinates
    mouse_position: Vec2,
    // Mouse movement delta since last frame in virtual screen coordinates
    mouse_delta: Vec2,
    // Mouse wheel delta since last frame
    mouse_wheel: Vec2,

    // Mouse button states
    mouse_buttons_held: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
}

impl InputState {
    /// Creates a new `InputState` instance.
    pub fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            mouse_wheel: Vec2::ZERO,
            mouse_buttons_held: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
        }
    }

    /// Clears temporary single-frame input states at the start of a new frame.
    pub fn clear_frame_states(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();

        // Clear mouse delta and single-press mouse states at the start of every frame
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel = Vec2::ZERO;
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
    }

    /// Clears all input states. Useful when the window loses focus.
    pub fn clear_all_states(&mut self) {
        self.keys_held.clear();
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel = Vec2::ZERO;
        self.mouse_buttons_held.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
    }

    /// Processes key events received from winit.
    pub fn handle_key_event(&mut self, event: &winit::event::KeyEvent) {
        let key = event.physical_key;
        if let winit::keyboard::PhysicalKey::Code(key_code) = key {
            if event.state.is_pressed() {
                if !self.keys_held.contains(&key_code) {
                    self.keys_held.insert(key_code);
                    self.keys_pressed.insert(key_code);
                }
            } else {
                if self.keys_held.contains(&key_code) {
                    self.keys_held.remove(&key_code);
                    self.keys_released.insert(key_code);
                }
            }
        }
    }

    /// Processes mouse button events received from winit.
    pub fn handle_mouse_input(&mut self, button: MouseButton, state: winit::event::ElementState) {
        if state.is_pressed() {
            if !self.mouse_buttons_held.contains(&button) {
                self.mouse_buttons_held.insert(button);
                self.mouse_buttons_pressed.insert(button);
            }
        } else {
            if self.mouse_buttons_held.contains(&button) {
                self.mouse_buttons_held.remove(&button);
                self.mouse_buttons_released.insert(button);
            }
        }
    }

    /// Sets the current mouse position and accumulates the delta distance.
    pub fn set_mouse_position_and_delta(&mut self, new_pos: Vec2) {
        let delta = new_pos - self.mouse_position;
        self.mouse_position = new_pos;
        self.mouse_delta += delta;
    }

    /// Directly adds to the mouse delta movement. Used when the cursor is locked.
    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_delta += delta;
    }

    /// Returns true if the specified key is currently held down.
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// Returns true if the specified key was pressed this frame.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Returns true if the specified key was released this frame.
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    /// Gets the current mouse position in virtual screen coordinates.
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Gets the mouse delta movement since the last frame in virtual screen coordinates.
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Returns true if the specified mouse button is currently held down.
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_held.contains(&button)
    }

    /// Returns true if the specified mouse button was pressed this frame.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Returns true if the specified mouse button was released this frame.
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }

    /// Processes mouse wheel scroll events received from winit.
    pub fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                self.mouse_wheel += glam::vec2(x, y);
            }
            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                // Approximate line delta from pixel delta
                let scale = 0.1;
                self.mouse_wheel += glam::vec2(pos.x as f32 * scale, pos.y as f32 * scale);
            }
        }
    }

    /// Gets the mouse wheel delta movement since the last frame.
    pub fn mouse_wheel(&self) -> Vec2 {
        self.mouse_wheel
    }
}

