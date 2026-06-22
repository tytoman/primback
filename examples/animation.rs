use primback::prelude::*;

struct AnimationApp {
    linear_pos: Animated<Vec3>,
    exp_pos: Animated<Vec3>,
    spring_pos: Animated<Vec3>,
    target_pos: Vec3,
    timer: f32,
    rng: Random,
}

impl PrimbackApp for AnimationApp {
    fn init(&mut self, update_ctx: &mut UpdateContext) {
        let camera = Camera::look_at(vec3(0.0, 4.0, 8.0), vec3(0.0, 1.0, 0.0), Vec3::Y);
        update_ctx.set_camera(camera);

        let light = Light {
            direction: vec3(0.5, -1.0, -0.5).normalize(),
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
        self.timer += dt;

        // Change target position every 2.5 seconds
        if self.timer >= 2.5 {
            self.timer = 0.0;
            // Generate a random position in the XZ plane with Y=1.0
            let x = self.rng.range(-4.0, 4.0);
            let z = self.rng.range(-2.0, 2.0);
            self.target_pos = vec3(x, 1.0, z);

            // Update targets of animated positions
            self.linear_pos.set_target(self.target_pos);
            self.exp_pos.set_target(self.target_pos);
            self.spring_pos.set_target(self.target_pos);

            // Play synth click sound
            update_ctx.play_synth(Synth::click());
        }

        // Update the animated states
        self.linear_pos.update(dt);
        self.exp_pos.update(dt);
        self.spring_pos.update(dt);
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) {
        draw_ctx.clear(Color::new(0.07, 0.07, 0.09));

        // Draw flat dark ground
        draw_ctx.draw_shape(
            Shape::plane(),
            Transform::new_position(vec3(0.0, 0.0, 0.0)).scale(vec3(12.0, 1.0, 8.0)),
            Color::new(0.12, 0.12, 0.14),
        );

        // Draw Target Position indicator (small red sphere)
        draw_ctx.draw_shape(
            Shape::sphere().radius(0.15),
            Transform::new_position(self.target_pos),
            Color::new(0.9, 0.1, 0.1),
        );

        // Helper to draw connecting lines from shape's default start positions (approximate columns)
        // so users can visually see the movement paths

        // 1. Draw Linear sphere (Magenta)
        let linear_val = self.linear_pos.current();
        draw_ctx.draw_shape(
            Shape::sphere().radius(0.4),
            Transform::new_position(vec3(linear_val.x, linear_val.y + 0.5, linear_val.z)),
            Color::new(0.8, 0.2, 0.8),
        );
        // Draw 3D text label above the linear sphere (using screen projections!)
        if let Some(screen_pos) = draw_ctx.camera().world_to_screen(
            vec3(linear_val.x, linear_val.y + 1.2, linear_val.z),
            draw_ctx.render_size(),
        ) {
            let label = Text::new("Linear").scale(1.0);
            let size = draw_ctx.measure_text(&label);
            draw_ctx.draw_text(
                label,
                RectTransform::new_position(screen_pos - vec2(size.x * 0.5, 0.0)),
                Color::new(0.8, 0.2, 0.8),
            );
        }

        // 2. Draw Exp sphere (Yellow/Gold)
        let exp_val = self.exp_pos.current();
        draw_ctx.draw_shape(
            Shape::sphere().radius(0.4),
            Transform::new_position(vec3(exp_val.x, exp_val.y, exp_val.z)),
            Color::new(0.9, 0.7, 0.1),
        );
        if let Some(screen_pos) = draw_ctx.camera().world_to_screen(
            vec3(exp_val.x, exp_val.y + 0.7, exp_val.z),
            draw_ctx.render_size(),
        ) {
            let label = Text::new("Exp").scale(1.0);
            let size = draw_ctx.measure_text(&label);
            draw_ctx.draw_text(
                label,
                RectTransform::new_position(screen_pos - vec2(size.x * 0.5, 0.0)),
                Color::new(0.9, 0.7, 0.1),
            );
        }

        // 3. Draw Spring sphere (Cyan)
        let spring_val = self.spring_pos.current();
        draw_ctx.draw_shape(
            Shape::sphere().radius(0.4),
            Transform::new_position(vec3(spring_val.x, spring_val.y - 0.5, spring_val.z)),
            Color::new(0.1, 0.7, 0.8),
        );
        if let Some(screen_pos) = draw_ctx.camera().world_to_screen(
            vec3(spring_val.x, spring_val.y + 0.2, spring_val.z),
            draw_ctx.render_size(),
        ) {
            let label = Text::new("Spring").scale(1.0);
            let size = draw_ctx.measure_text(&label);
            draw_ctx.draw_text(
                label,
                RectTransform::new_position(screen_pos - vec2(size.x * 0.5, 0.0)),
                Color::new(0.1, 0.7, 0.8),
            );
        }

        // 2D Overlay
        draw_ctx.draw_text(
            Text::new("Interpolation Comparison Demo").scale(1.0),
            RectTransform::new_position(vec2(15.0, 15.0)),
            Color::WHITE,
        );
        draw_ctx.draw_text(
            Text::new("Comparing Linear vs Exponential vs Spring dynamics").scale(1.0),
            RectTransform::new_position(vec2(15.0, 35.0)),
            Color::new(0.6, 0.6, 0.7),
        );
    }
}

fn main() {
    let config = PrimbackConfig {
        window_title: "Primback - Animation Demo".to_string(),
        ..Default::default()
    };

    let start_pos = vec3(-3.0, 1.0, 0.0);

    // Initialize the Animated positions with different modes
    let linear_pos = Animated::new(start_pos, AnimationMode::linear().speed(3.0));
    let exp_pos = Animated::new(start_pos, AnimationMode::exp().speed(4.0));
    let spring_pos = Animated::new(
        start_pos,
        AnimationMode::spring().stiffness(40.0).damping(5.0),
    );

    Primback::run(
        config,
        AnimationApp {
            linear_pos,
            exp_pos,
            spring_pos,
            target_pos: start_pos,
            timer: 2.0, // Trigger immediate target switch after startup
            rng: Random::new_random(),
        },
    );
}
