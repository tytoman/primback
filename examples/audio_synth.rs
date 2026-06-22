use primback::prelude::*;

struct AudioApp {
    sound_angle: f32,
    pulse_timer: f32,
}

impl PrimbackApp for AudioApp {
    fn init(&mut self, update_ctx: &mut UpdateContext) {
        // Set fixed camera at center looking at the sound source
        let camera = Camera::look_at(vec3(0.0, 3.0, 6.0), vec3(0.0, 1.0, 0.0), Vec3::Y);
        update_ctx.set_camera(camera);

        let light = Light {
            direction: vec3(0.0, -1.0, -1.0).normalize(),
            color: Color::new(1.0, 1.0, 1.0),
            ambient: Color::new(0.3, 0.3, 0.35),
        };
        update_ctx.set_light(light);
    }

    fn update(&mut self, update_ctx: &mut UpdateContext) {
        if update_ctx.is_key_pressed(KeyCode::Escape) {
            update_ctx.exit();
        }

        let dt = update_ctx.delta_time();

        // 1. Rotate the 3D sound source position around the center
        self.sound_angle += dt * 1.5;
        let sound_pos = vec3(
            self.sound_angle.cos() * 3.0,
            1.0,
            self.sound_angle.sin() * 3.0,
        );

        // 2. Play periodic 3D spatial sound pulse
        self.pulse_timer += dt;
        if self.pulse_timer >= 0.4 {
            self.pulse_timer = 0.0;
            // Play sound with 3D position
            let sound = Synth::sine(Pitch::A(4))
                .duration(0.08)
                .volume(0.6)
                .position(sound_pos);
            update_ctx.play_synth(sound);
        }

        // 3. Play Preset Effects (Keys 1 - 5)
        if update_ctx.is_key_pressed(KeyCode::Digit1) {
            update_ctx.play_synth(Synth::coin());
        }
        if update_ctx.is_key_pressed(KeyCode::Digit2) {
            update_ctx.play_synth(Synth::jump());
        }
        if update_ctx.is_key_pressed(KeyCode::Digit3) {
            update_ctx.play_synth(Synth::damage());
        }
        if update_ctx.is_key_pressed(KeyCode::Digit4) {
            update_ctx.play_synth(Synth::explosion());
        }
        if update_ctx.is_key_pressed(KeyCode::Digit5) {
            update_ctx.play_synth(Synth::click());
        }

        // 4. Keyboard Piano (Keys Q, W, E, R, T, Y, U, I)
        let piano_keys = [
            (KeyCode::KeyQ, Pitch::C(4)),
            (KeyCode::KeyW, Pitch::D(4)),
            (KeyCode::KeyE, Pitch::E(4)),
            (KeyCode::KeyR, Pitch::F(4)),
            (KeyCode::KeyT, Pitch::G(4)),
            (KeyCode::KeyY, Pitch::A(4)),
            (KeyCode::KeyU, Pitch::B(4)),
            (KeyCode::KeyI, Pitch::C(5)),
        ];

        for (key, pitch) in piano_keys {
            if update_ctx.is_key_pressed(key) {
                // Play triangle wave for a retro 8-bit piano sound
                update_ctx.play_synth(Synth::triangle(pitch).duration(0.2).volume(0.5));
            }
        }
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) {
        draw_ctx.clear(Color::new(0.08, 0.08, 0.1));

        // Draw flat floor
        draw_ctx.draw_shape(
            Shape::plane(),
            Transform::new_position(vec3(0.0, 0.0, 0.0)).scale(vec3(8.0, 1.0, 8.0)),
            Color::new(0.15, 0.15, 0.18),
        );

        // Draw camera position indicator (listener) at (0, 1, 0)
        draw_ctx.draw_shape(
            Shape::cube().size(vec3(0.6, 0.6, 0.6)),
            Transform::new_position(vec3(0.0, 1.0, 0.0)),
            Color::new(0.2, 0.6, 0.9),
        );

        // Calculate and draw rotating 3D sound source
        let sound_pos = vec3(
            self.sound_angle.cos() * 3.0,
            1.0,
            self.sound_angle.sin() * 3.0,
        );
        draw_ctx.draw_shape(
            Shape::sphere().radius(0.4),
            Transform::new_position(sound_pos),
            Color::new(0.9, 0.3, 0.3),
        );

        // Draw line connecting listener and emitter
        draw_ctx.draw_line_3d(
            vec3(0.0, 1.0, 0.0),
            sound_pos,
            Color::new(0.5, 0.5, 0.5).alpha(0.5),
        );

        // 3D labels for visual guidance
        if let Some(screen_pos) = draw_ctx
            .camera()
            .world_to_screen(vec3(0.0, 1.5, 0.0), draw_ctx.render_size())
        {
            let label = Text::new("Listener (Camera)").scale(1.0);
            let size = draw_ctx.measure_text(&label);
            draw_ctx.draw_text(
                label,
                RectTransform::new_position(screen_pos - vec2(size.x * 0.5, 0.0)),
                Color::new(0.2, 0.6, 0.9),
            );
        }

        if let Some(screen_pos) = draw_ctx
            .camera()
            .world_to_screen(sound_pos + vec3(0.0, 0.6, 0.0), draw_ctx.render_size())
        {
            let label = Text::new("3D Sound Source").scale(1.0);
            let size = draw_ctx.measure_text(&label);
            draw_ctx.draw_text(
                label,
                RectTransform::new_position(screen_pos - vec2(size.x * 0.5, 0.0)),
                Color::new(0.9, 0.3, 0.3),
            );
        }

        // 2D UI instructions
        draw_ctx.draw_text(
            Text::new("Audio Synthesizer & Spatial 3D Audio").scale(1.0),
            RectTransform::new_position(vec2(15.0, 15.0)),
            Color::WHITE,
        );

        // Sound effects instruction card
        let sfx_bg = Rect::new(vec2(280.0, 100.0))
            .color(Color::new(0.12, 0.12, 0.15))
            .radius(6.0);
        draw_ctx.draw_rect(sfx_bg, RectTransform::new_position(vec2(15.0, 50.0)));

        draw_ctx.draw_text(
            Text::new("SFX Presets (Press Keys 1-5):").scale(1.0),
            RectTransform::new_position(vec2(25.0, 60.0)),
            Color::new(1.0, 0.8, 0.2),
        );
        let sfx_lines = [
            "1: Coin       2: Jump",
            "3: Damage     4: Explosion",
            "5: Click",
        ];
        for (idx, line) in sfx_lines.iter().enumerate() {
            draw_ctx.draw_text(
                Text::new(*line).scale(1.0),
                RectTransform::new_position(vec2(25.0, 85.0 + (idx as f32) * 16.0)),
                Color::WHITE,
            );
        }

        // Virtual piano instruction card
        let piano_bg = Rect::new(vec2(320.0, 100.0))
            .color(Color::new(0.12, 0.12, 0.15))
            .radius(6.0);
        draw_ctx.draw_rect(piano_bg, RectTransform::new_position(vec2(305.0, 50.0)));

        draw_ctx.draw_text(
            Text::new("Keyboard Piano (Play Notes):").scale(1.0),
            RectTransform::new_position(vec2(315.0, 60.0)),
            Color::new(0.2, 0.8, 0.5),
        );
        draw_ctx.draw_text(
            Text::new("Keys:  Q   W   E   R   T   Y   U   I").scale(1.0),
            RectTransform::new_position(vec2(315.0, 85.0)),
            Color::WHITE,
        );
        draw_ctx.draw_text(
            Text::new("Notes: C4  D4  E4  F4  G4  A4  B4  C5").scale(1.0),
            RectTransform::new_position(vec2(315.0, 105.0)),
            Color::new(0.7, 0.7, 0.7),
        );
    }
}

fn main() {
    let config = PrimbackConfig {
        window_title: "Primback - Audio Synthesizer Demo".to_string(),
        render_width: 640,
        render_height: 480,
        ..Default::default()
    };

    Primback::run(
        config,
        AudioApp {
            sound_angle: 0.0,
            pulse_timer: 0.0,
        },
    );
}
