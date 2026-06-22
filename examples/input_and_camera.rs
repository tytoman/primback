use primback::prelude::*;

struct InputCameraApp {
    yaw: f32,
    pitch: f32,
    mouse_locked: bool,
}

impl PrimbackApp for InputCameraApp {
    fn init(&mut self, update_ctx: &mut UpdateContext) {
        // Set initial camera position
        let camera = Camera::look_at(vec3(0.0, 2.0, 5.0), vec3(0.0, 1.0, 0.0), Vec3::Y);
        update_ctx.set_camera(camera);

        // Lock cursor on startup
        update_ctx.set_cursor_grab(true);
        update_ctx.set_cursor_visible(false);
    }

    fn update(&mut self, update_ctx: &mut UpdateContext) {
        // Toggle cursor lock with Escape key
        if update_ctx.is_key_pressed(KeyCode::Escape) {
            self.mouse_locked = !self.mouse_locked;
            update_ctx.set_cursor_grab(self.mouse_locked);
            update_ctx.set_cursor_visible(!self.mouse_locked);
        }

        // Camera rotation (only when mouse is locked)
        if self.mouse_locked {
            let delta = update_ctx.mouse_delta();
            let sensitivity = 0.15;
            self.yaw -= delta.x * sensitivity;
            self.pitch -= delta.y * sensitivity;
            // Clamp pitch to prevent turning upside down
            self.pitch = self.pitch.clamp(-89.0, 89.0);

            // Update camera rotation
            let mut camera = *update_ctx.camera();
            camera.transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                self.yaw.to_radians(),
                self.pitch.to_radians(),
                0.0,
            );
            update_ctx.set_camera(camera);
        }

        // Camera movement (WASD + Space/Shift for Up/Down)
        let mut camera = *update_ctx.camera();
        let move_speed = 5.0 * update_ctx.delta_time();
        let mut move_dir = Vec3::ZERO;

        let forward = camera.transform.forward();
        // Project forward vector on XZ plane to keep movement horizontal
        let forward_horizontal = vec3(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = camera.transform.right();

        if update_ctx.is_key_down(KeyCode::KeyW) {
            move_dir += forward_horizontal;
        }
        if update_ctx.is_key_down(KeyCode::KeyS) {
            move_dir -= forward_horizontal;
        }
        if update_ctx.is_key_down(KeyCode::KeyA) {
            move_dir -= right;
        }
        if update_ctx.is_key_down(KeyCode::KeyD) {
            move_dir += right;
        }
        if update_ctx.is_key_down(KeyCode::Space) {
            move_dir += Vec3::Y;
        }
        if update_ctx.is_key_down(KeyCode::ShiftLeft) {
            move_dir -= Vec3::Y;
        }

        camera.transform.position += move_dir.normalize_or_zero() * move_speed;
        update_ctx.set_camera(camera);

        // Play synth sound when left-clicking
        if update_ctx.is_mouse_button_pressed(MouseButton::Left) {
            update_ctx.play_synth(Synth::coin());
        }
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) {
        draw_ctx.clear(Color::new(0.1, 0.1, 0.12));

        // Draw a simple 3D checkerboard floor
        let grid_size = 10;
        let spacing = 2.0;
        for x in -grid_size..=grid_size {
            for z in -grid_size..=grid_size {
                let color = if (x + z) % 2 == 0 {
                    Color::new(0.2, 0.2, 0.22)
                } else {
                    Color::new(0.15, 0.15, 0.17)
                };
                let pos = vec3(x as f32 * spacing, 0.0, z as f32 * spacing);
                draw_ctx.draw_shape(
                    Shape::plane(),
                    Transform::new_position(pos).scale(vec3(spacing, 1.0, spacing)),
                    color,
                );
            }
        }

        // Draw some colorful pillars as reference points in 3D space
        for i in 0..8 {
            let angle = (i as f32) * (std::f32::consts::PI / 4.0);
            let radius = 6.0;
            let px = angle.cos() * radius;
            let pz = angle.sin() * radius;
            let color = Color::from_hsv((i as f32) * 45.0, 0.8, 0.8);
            draw_ctx.draw_shape(
                Shape::cylinder().radius(0.3).height(2.0),
                Transform::new_position(vec3(px, 1.0, pz)),
                color,
            );
        }

        // Draw 2D UI instructions on screen
        let screen_w = draw_ctx.render_size().x;

        // Draw guidance box
        let ui_rect = Rect::new(vec2(280.0, 110.0))
            .color(Color::new(0.0, 0.0, 0.0))
            .radius(8.0)
            .shadow_offset(vec2(2.0, 2.0));
        let ui_trans = RectTransform::new_position(vec2(10.0, 10.0));
        draw_ctx.draw_rect(ui_rect, ui_trans);

        // Render instructions text inside the box
        let instructions = [
            "FPS Camera Controls:",
            "- WASD: Move horizontally",
            "- Space / Shift: Fly Up / Down",
            "- Mouse: Look around",
            "- Left Click: Play coin sound",
            "- ESC: Toggle Mouse Lock",
        ];

        for (idx, line) in instructions.iter().enumerate() {
            let text_trans = RectTransform::new_position(vec2(20.0, 18.0 + (idx as f32) * 14.0));
            let color = if idx == 0 {
                Color::new(1.0, 0.8, 0.2) // Highlight title
            } else {
                Color::WHITE
            };
            draw_ctx.draw_text(Text::new(*line).scale(1.0), text_trans, color);
        }

        // Display current lock status at the bottom center
        let status_str = if self.mouse_locked {
            "Mouse Locked (ESC to Unlock)"
        } else {
            "Mouse Unlocked (ESC to Lock)"
        };
        let status_color = if self.mouse_locked {
            Color::new(0.2, 0.8, 0.2)
        } else {
            Color::new(0.8, 0.2, 0.2)
        };
        let text_size = draw_ctx.measure_text(&Text::new(status_str));
        let status_trans = RectTransform::new_position(vec2(
            (screen_w - text_size.x) * 0.5,
            draw_ctx.render_size().y - 25.0,
        ));
        draw_ctx.draw_text(
            Text::new(status_str)
                .scale(1.0)
                .shadow_offset(vec2(1.0, 1.0)),
            status_trans,
            status_color,
        );
    }
}

fn main() {
    let config = PrimbackConfig {
        window_title: "Primback - Input and Camera Demo".to_string(),
        render_width: 640,
        render_height: 480,
        ..Default::default()
    };

    Primback::run(
        config,
        InputCameraApp {
            yaw: 0.0,
            pitch: 0.0,
            mouse_locked: true,
        },
    );
}
