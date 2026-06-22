# Primback

`Primback` is a lightweight, easy-to-use 2D/3D primitive rendering engine for Rust.

## Example Code (`main.rs`)

```rust
use primback::prelude::*;

struct MyApp {
    rotation: f32,
}

impl PrimbackApp for MyApp {
    fn init(&mut self, update_ctx: &mut UpdateContext) {
        // Configure camera and light
        update_ctx.set_camera(Camera::look_at(
            vec3(0.0, 5.0, 10.0), // Eye position
            vec3(0.0, 0.0, 0.0),  // Target position
            Vec3::Y,              // Up vector
        ));
        update_ctx.set_light(Light {
            direction: vec3(1.0, -1.0, -1.0).normalize(),
            color: Color::new(1.0, 1.0, 1.0),
            ambient: Color::new(0.2, 0.2, 0.2),
        });
    }

    fn update(&mut self, update_ctx: &mut UpdateContext) {
        self.rotation += update_ctx.delta_time();

        if update_ctx.is_key_pressed(KeyCode::Escape) {
            update_ctx.exit();
        }
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) {
        draw_ctx.clear(Color::new(0.1, 0.1, 0.15));

        // Draw a rotating red cube
        let transform = Transform::new(
            vec3(0.0, 0.0, 0.0),
            Quat::from_rotation_y(self.rotation),
            Vec3::ONE,
        );
        draw_ctx.draw_shape(
            Shape::cube(),
            transform,
            Color::new(0.8, 0.2, 0.2),
        );
    }
}

fn main() {
    let config = PrimbackConfig {
        window_title: "My Primback App".to_string(),
        ..Default::default()
    };
    Primback::run(config, MyApp { rotation: 0.0 });
}
```

## Running Examples

The repository includes several feature-specific examples:

```bash
# Basic 3D primitives and rendering setup
cargo run --example basic_3d

# Input handling and mouse-locked camera control
cargo run --example input_and_camera

# Rounded UI cards, shadows, and text rendering
cargo run --example ui_and_text

# Easing and spring-based animations
cargo run --example animation

# Audio synthesizer and 3D spatial audio
cargo run --example audio_synth
```

## License

This project is licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
