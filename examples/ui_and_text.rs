use primback::prelude::*;

struct UIApp {
    click_count: u32,
    button_rect: Rect,
    button_transform: RectTransform,
    is_hovered: bool,
}

impl PrimbackApp for UIApp {
    fn init(&mut self, update_ctx: &mut UpdateContext) {
        // Set mouse visible and disable grab for standard UI usage
        update_ctx.set_cursor_grab(false);
        update_ctx.set_cursor_visible(true);
    }

    fn update(&mut self, update_ctx: &mut UpdateContext) {
        // Close on ESC
        if update_ctx.is_key_pressed(KeyCode::Escape) {
            update_ctx.exit();
        }

        // Get mouse position and check if it's over the button
        let mouse_pos = update_ctx.mouse_position();
        self.is_hovered = self
            .button_rect
            .contains_point(&self.button_transform, mouse_pos);

        // Click detection
        if self.is_hovered && update_ctx.is_mouse_button_pressed(MouseButton::Left) {
            self.click_count += 1;
            // Play a click sound effect
            update_ctx.play_synth(Synth::click());
        }
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) {
        // Clear screen with a nice slate color
        draw_ctx.clear(Color::new(0.08, 0.09, 0.12));

        let screen_size = draw_ctx.render_size();

        // 1. Title Bar (Anchored at Top Center)
        let title_bg = Rect::new(vec2(screen_size.x, 40.0))
            .color(Color::new(0.12, 0.14, 0.18))
            .anchor_x(Anchor::Start)
            .anchor_y(Anchor::Start);
        draw_ctx.draw_rect(title_bg, RectTransform::new_position(vec2(0.0, 0.0)));

        let title_text = Text::new("Primback UI & Text Demo").scale(1.0);
        let title_size = draw_ctx.measure_text(&title_text);
        draw_ctx.draw_text(
            title_text,
            RectTransform::new_position(vec2((screen_size.x - title_size.x) * 0.5, 12.0)),
            Color::new(1.0, 0.8, 0.2),
        );

        // 2. Info Card (Left Area)
        let card_rect = Rect::new(vec2(250.0, 160.0))
            .color(Color::new(0.15, 0.17, 0.22))
            .radius(12.0)
            .outline_width(1.5)
            .outline_color(Color::new(0.25, 0.27, 0.35))
            .shadow_offset(vec2(4.0, 4.0))
            .shadow_color(Color::new(0.02, 0.02, 0.04).alpha(0.6));
        draw_ctx.draw_rect(card_rect, RectTransform::new_position(vec2(20.0, 60.0)));

        draw_ctx.draw_text(
            Text::new("Card Component").scale(1.0),
            RectTransform::new_position(vec2(35.0, 75.0)),
            Color::new(0.4, 0.7, 1.0),
        );
        draw_ctx.draw_text(
            Text::new("SDF-based features:").scale(1.0),
            RectTransform::new_position(vec2(35.0, 100.0)),
            Color::new(0.7, 0.7, 0.7),
        );
        draw_ctx.draw_text(
            Text::new("- Round Corners (radius)").scale(1.0),
            RectTransform::new_position(vec2(35.0, 120.0)),
            Color::WHITE,
        );
        draw_ctx.draw_text(
            Text::new("- Custom Borders (outline)").scale(1.0),
            RectTransform::new_position(vec2(35.0, 138.0)),
            Color::WHITE,
        );
        draw_ctx.draw_text(
            Text::new("- Drop Shadows (shadow)").scale(1.0),
            RectTransform::new_position(vec2(35.0, 156.0)),
            Color::WHITE,
        );

        // 3. Interactive Button (Right Area)
        let btn_color = if self.is_hovered {
            Color::new(0.25, 0.65, 0.45) // Lighter green on hover
        } else {
            Color::new(0.18, 0.50, 0.35) // Normal green
        };

        let btn_outline = if self.is_hovered {
            Color::new(0.4, 0.9, 0.6)
        } else {
            Color::new(0.2, 0.6, 0.4)
        };

        self.button_rect = Rect::new(vec2(200.0, 50.0))
            .color(btn_color)
            .radius(8.0)
            .outline_width(2.0)
            .outline_color(btn_outline)
            .shadow_offset(vec2(0.0, 3.0))
            .shadow_color(Color::new(0.0, 0.0, 0.0).alpha(0.5));

        draw_ctx.draw_rect(self.button_rect.clone(), self.button_transform);

        // Draw text inside the button
        let btn_label = Text::new("CLICK ME").scale(1.0);
        let btn_label_size = draw_ctx.measure_text(&btn_label);
        // Center text on the button
        let text_x = self.button_transform.position.x + (200.0 - btn_label_size.x) * 0.5;
        let text_y = self.button_transform.position.y + (50.0 - btn_label_size.y) * 0.5;
        draw_ctx.draw_text(
            btn_label,
            RectTransform::new_position(vec2(text_x, text_y)),
            Color::WHITE,
        );

        // Draw click count
        let count_str = format!("Clicks: {}", self.click_count);
        let count_text = Text::new(&count_str).scale(1.0);
        let count_size = draw_ctx.measure_text(&count_text);
        draw_ctx.draw_text(
            count_text,
            RectTransform::new_position(vec2(
                self.button_transform.position.x + (200.0 - count_size.x) * 0.5,
                self.button_transform.position.y + 70.0,
            )),
            Color::WHITE,
        );

        // 4. Anchoring Demonstration (Bottom Status Bar)
        let bottom_text = Text::new("Press ESC to Exit").scale(1.0);
        let bottom_size = draw_ctx.measure_text(&bottom_text);
        // Bottom-Right anchor simulation
        let bottom_trans = RectTransform::new_position(vec2(
            screen_size.x - bottom_size.x - 20.0,
            screen_size.y - bottom_size.y - 15.0,
        ));
        draw_ctx.draw_text(bottom_text, bottom_trans, Color::new(0.5, 0.5, 0.6));
    }
}

fn main() {
    let config = PrimbackConfig {
        window_title: "Primback - UI and Text Demo".to_string(),
        render_width: 640,
        render_height: 480,
        ..Default::default()
    };

    // Place the button at (350, 100)
    let button_transform = RectTransform::new_position(vec2(350.0, 100.0));

    Primback::run(
        config,
        UIApp {
            click_count: 0,
            button_rect: Rect::new(vec2(200.0, 50.0)),
            button_transform,
            is_hovered: false,
        },
    );
}
