use primback::prelude::*;

struct Basic3DApp {
    rotation_angle: f32,
}

impl PrimbackApp for Basic3DApp {
    fn init(&mut self, update_ctx: &mut UpdateContext) {
        // Set the camera position and direction
        let camera = Camera::look_at(
            vec3(0.0, 5.0, 10.0), // Eye position
            vec3(0.0, 0.5, 0.0),  // Look-at target
            Vec3::Y,              // Up vector
        );
        update_ctx.set_camera(camera);

        // Configure the directional light direction and colors
        let light = Light {
            direction: vec3(1.0, -1.0, -1.0).normalize(),
            color: Color::new(1.0, 0.95, 0.9),
            ambient: Color::new(0.2, 0.2, 0.25),
        };
        update_ctx.set_light(light);
    }

    fn update(&mut self, update_ctx: &mut UpdateContext) {
        // Rotate shapes over time
        self.rotation_angle += update_ctx.delta_time() * 0.5;

        // Close the application if Escape key is pressed
        if update_ctx.is_key_pressed(KeyCode::Escape) {
            update_ctx.exit();
        }
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) {
        // Clear the screen with a dark background color
        draw_ctx.clear(Color::new(0.05, 0.05, 0.08));

        // Draw a green grid at the bottom
        let plane_transform =
            Transform::new_position(vec3(0.0, -1.0, 0.0)).scale(vec3(15.0, 1.0, 15.0));
        draw_ctx.draw_shape_wire(
            Shape::plane().subdivisions(10),
            plane_transform,
            Color::new(0.2, 0.3, 0.2),
        );

        // 1. Red Cube on the left
        let cube_rot = Quat::from_rotation_y(self.rotation_angle);
        let cube_transform = Transform::new_position(vec3(-3.0, 0.5, 0.0)).rotation(cube_rot);
        draw_ctx.draw_shape(
            Shape::cube().size(vec3(1.5, 1.5, 1.5)),
            cube_transform,
            Color::new(0.8, 0.2, 0.2),
        );

        // 2. Blue Sphere in the middle
        let sphere_transform = Transform::new_position(vec3(0.0, 0.5, 0.0));
        draw_ctx.draw_shape(
            Shape::sphere().radius(0.8),
            sphere_transform,
            Color::new(0.2, 0.4, 0.8),
        );

        // 3. Yellow Cone on the right
        let cone_rot = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
            * Quat::from_rotation_z(self.rotation_angle);
        let cone_transform = Transform::new(vec3(3.0, 0.5, 0.0), cone_rot, Vec3::ONE);
        draw_ctx.draw_shape(
            Shape::cone().radius(0.6).height(1.2),
            cone_transform,
            Color::new(0.8, 0.8, 0.2),
        );

        // 4. Cyan Torus in the front (Wireframe mode to show off structure)
        let torus_rot = Quat::from_rotation_x(self.rotation_angle)
            * Quat::from_rotation_y(self.rotation_angle * 0.5);
        let torus_transform = Transform::new(vec3(0.0, 0.5, 3.0), torus_rot, Vec3::ONE);
        draw_ctx.draw_shape_wire(
            Shape::torus().major_radius(0.8).minor_radius(0.2),
            torus_transform,
            Color::new(0.2, 0.8, 0.8),
        );

        // 5. Magenta Cylinder in the back (Unlit mode to show differences in lighting)
        let cylinder_rot = Quat::from_rotation_z(self.rotation_angle);
        let cylinder_transform = Transform::new(vec3(0.0, 0.8, -3.0), cylinder_rot, Vec3::ONE);
        draw_ctx.draw_shape(
            Shape::cylinder().radius(0.5).height(1.4).unlit(),
            cylinder_transform,
            Color::new(0.8, 0.2, 0.8),
        );
    }
}

fn main() {
    let config = PrimbackConfig {
        window_title: "Primback - Basic 3D Demo".to_string(),
        ..Default::default()
    };

    Primback::run(
        config,
        Basic3DApp {
            rotation_angle: 0.0,
        },
    );
}
