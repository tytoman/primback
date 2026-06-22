use glam::{Mat4, Quat, Vec2, Vec3, vec2, vec3};
use primback::prelude::*;

// ============================================================================
// Color
// ============================================================================

#[test]
fn color_new() {
    let c = Color::new(0.5, 0.6, 0.7);
    assert_eq!(c.r, 0.5);
    assert_eq!(c.g, 0.6);
    assert_eq!(c.b, 0.7);
}

#[test]
fn color_constants() {
    assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0));
    assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0));
    assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0));
    assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0));
    assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0));
}

#[test]
fn color_lerp() {
    let a = Color::BLACK;
    let b = Color::WHITE;
    let mid = a.lerp(b, 0.5);
    assert!((mid.r - 0.5).abs() < 1e-6);
    assert!((mid.g - 0.5).abs() < 1e-6);
    assert!((mid.b - 0.5).abs() < 1e-6);
}

#[test]
fn color_lerp_boundaries() {
    let a = Color::RED;
    let b = Color::BLUE;
    let start = a.lerp(b, 0.0);
    let end = a.lerp(b, 1.0);
    assert_eq!(start, a);
    assert_eq!(end, b);
}

#[test]
fn color_with_alpha() {
    let c = Color::RED.alpha(0.5);
    assert_eq!(c.color, Color::RED);
    assert_eq!(c.alpha, 0.5);
}

#[test]
fn color_from_into_color_with_alpha() {
    let c: ColorWithAlpha = Color::GREEN.into();
    assert_eq!(c.color, Color::GREEN);
    assert_eq!(c.alpha, 1.0);
}

#[test]
fn color_with_alpha_lerp() {
    let a = Color::RED.alpha(0.0);
    let b = Color::BLUE.alpha(1.0);
    let mid = a.lerp(b, 0.5);
    assert!((mid.alpha - 0.5).abs() < 1e-6);
}

#[test]
fn color_from_hsv_red() {
    let c = Color::from_hsv(0.0, 1.0, 1.0);
    assert!((c.r - 1.0).abs() < 1e-6);
    assert!(c.g.abs() < 1e-6);
    assert!(c.b.abs() < 1e-6);
}

#[test]
fn color_from_hsv_green() {
    let c = Color::from_hsv(120.0, 1.0, 1.0);
    assert!(c.r.abs() < 1e-6);
    assert!((c.g - 1.0).abs() < 1e-6);
    assert!(c.b.abs() < 1e-6);
}

#[test]
fn color_from_hsv_blue() {
    let c = Color::from_hsv(240.0, 1.0, 1.0);
    assert!(c.r.abs() < 1e-6);
    assert!(c.g.abs() < 1e-6);
    assert!((c.b - 1.0).abs() < 1e-6);
}

// ============================================================================
// Transform
// ============================================================================

#[test]
fn transform_default_is_identity() {
    let t = Transform::default();
    assert_eq!(t.position, Vec3::ZERO);
    assert_eq!(t.rotation, Quat::IDENTITY);
    assert_eq!(t.scale, Vec3::ONE);
}

#[test]
fn transform_new() {
    let pos = vec3(1.0, 2.0, 3.0);
    let rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let scl = vec3(2.0, 2.0, 2.0);
    let t = Transform::new(pos, rot, scl);
    assert_eq!(t.position, pos);
    assert_eq!(t.scale, scl);
}

#[test]
fn transform_to_matrix_identity() {
    let t = Transform::default();
    let mat = t.to_matrix();
    let diff = mat - Mat4::IDENTITY;
    for i in 0..4 {
        let col = diff.col(i);
        assert!(
            col.length() < 1e-6,
            "Identity transform should produce identity matrix"
        );
    }
}

#[test]
fn transform_to_matrix_position() {
    let t = Transform::new_position(vec3(3.0, 4.0, 5.0));
    let mat = t.to_matrix();
    let pos = mat.transform_point3(Vec3::ZERO);
    assert!((pos - vec3(3.0, 4.0, 5.0)).length() < 1e-6);
}

#[test]
fn transform_direction_vectors() {
    let t = Transform::default();
    assert!((t.right() - Vec3::X).length() < 1e-6);
    assert!((t.up() - Vec3::Y).length() < 1e-6);
    assert!((t.forward() - (-Vec3::Z)).length() < 1e-6);
}

#[test]
fn transform_to_world_point_and_back() {
    let t = Transform::new(vec3(10.0, 0.0, 0.0), Quat::IDENTITY, vec3(2.0, 2.0, 2.0));
    let local_pt = vec3(1.0, 1.0, 1.0);
    let world_pt = t.to_world_point(local_pt);
    let back = t.to_local_point(world_pt);
    assert!((back - local_pt).length() < 1e-5);
}

#[test]
fn transform_inverse_round_trip() {
    let t = Transform::new(
        vec3(1.0, 2.0, 3.0),
        Quat::from_rotation_y(0.5),
        vec3(1.5, 1.5, 1.5),
    );
    let inv = t.inverse();
    let combined = t.to_world_transform(inv.into());
    let result = combined.to_transform();
    assert!(
        (result.position).length() < 1e-4,
        "Inverse should produce near-identity position"
    );
    assert!(
        (result.scale - Vec3::ONE).length() < 1e-4,
        "Inverse should produce near-identity scale"
    );
}

#[test]
fn transform_builder_pattern() {
    let t = Transform::default()
        .position(vec3(1.0, 2.0, 3.0))
        .scale(vec3(2.0, 2.0, 2.0));
    assert_eq!(t.position, vec3(1.0, 2.0, 3.0));
    assert_eq!(t.scale, vec3(2.0, 2.0, 2.0));
}

#[test]
fn transform_from_euler() {
    let t = Transform::from_euler(0.0, 0.0, 0.0);
    assert!((t.rotation - Quat::IDENTITY).length() < 1e-6);
}

// ============================================================================
// WorldTransform
// ============================================================================

#[test]
fn world_transform_from_transform() {
    let t = Transform::new_position(vec3(5.0, 0.0, 0.0));
    let wt: WorldTransform = t.into();
    assert!((wt.position() - vec3(5.0, 0.0, 0.0)).length() < 1e-6);
}

#[test]
fn world_transform_mul() {
    let parent = Transform::new_position(vec3(10.0, 0.0, 0.0));
    let child = Transform::new_position(vec3(0.0, 5.0, 0.0));
    let combined = parent * child;
    assert!((combined.position() - vec3(10.0, 5.0, 0.0)).length() < 1e-5);
}

#[test]
fn world_transform_inverse() {
    let t = Transform::new(vec3(1.0, 2.0, 3.0), Quat::from_rotation_z(1.0), Vec3::ONE);
    let wt = WorldTransform::from(t);
    let inv = wt.inverse();
    let identity = wt * inv;
    let diff = identity.matrix - Mat4::IDENTITY;
    for i in 0..4 {
        assert!(
            diff.col(i).length() < 1e-4,
            "Inverse should produce identity"
        );
    }
}

// ============================================================================
// Rect (UI Rectangle)
// ============================================================================

#[test]
fn rect_new() {
    let r = Rect::new(vec2(100.0, 50.0));
    assert_eq!(r.size, vec2(100.0, 50.0));
    assert_eq!(r.color, ColorWithAlpha::from(Color::WHITE));
    assert_eq!(r.radius, [0.0; 4]);
}

#[test]
fn rect_builder_pattern() {
    let r = Rect::new(vec2(200.0, 100.0))
        .color(Color::RED)
        .radius(10.0)
        .anchor(Anchor::Center, Anchor::Center);
    assert_eq!(r.anchor_x, Anchor::Center);
    assert_eq!(r.anchor_y, Anchor::Center);
    assert_eq!(r.radius, [10.0; 4]);
}

#[test]
fn rect_individual_radius() {
    let r = Rect::new(vec2(100.0, 100.0))
        .radius_tl(1.0)
        .radius_tr(2.0)
        .radius_br(3.0)
        .radius_bl(4.0);
    assert_eq!(r.radius, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn rect_outline_toggle() {
    let r = Rect::new(vec2(100.0, 100.0)).outline(true);
    assert!(r.outline_width.is_some());
    assert!(r.outline_color.is_some());

    let r2 = r.outline(false);
    assert!(r2.outline_width.is_none());
    assert!(r2.outline_color.is_none());
}

#[test]
fn rect_shadow_toggle() {
    let r = Rect::new(vec2(100.0, 100.0)).shadow(true);
    assert!(r.shadow_offset.is_some());
    assert!(r.shadow_color.is_some());

    let r2 = r.shadow(false);
    assert!(r2.shadow_offset.is_none());
    assert!(r2.shadow_color.is_none());
}

#[test]
fn rect_contains_point_basic() {
    let r = Rect::new(vec2(100.0, 100.0));
    let t = RectTransform::new_position(vec2(50.0, 50.0));
    // Center of the rect
    assert!(r.contains_point(&t, vec2(100.0, 100.0)));
    // Well outside
    assert!(!r.contains_point(&t, vec2(500.0, 500.0)));
}

#[test]
fn rect_from_vec2() {
    let r: Rect = vec2(64.0, 32.0).into();
    assert_eq!(r.size, vec2(64.0, 32.0));
}

// ============================================================================
// RectTransform
// ============================================================================

#[test]
fn rect_transform_default() {
    let t = RectTransform::default();
    assert_eq!(t.position, Vec2::ZERO);
    assert_eq!(t.rotation, 0.0);
    assert_eq!(t.scale, Vec2::ONE);
    assert_eq!(t.z_index, 0.0);
}

#[test]
fn rect_transform_new() {
    let t = RectTransform::new(vec2(1.0, 2.0), 0.5, vec2(3.0, 4.0), 5.0);
    assert_eq!(t.position, vec2(1.0, 2.0));
    assert_eq!(t.rotation, 0.5);
    assert_eq!(t.scale, vec2(3.0, 4.0));
    assert_eq!(t.z_index, 5.0);
}

#[test]
fn rect_transform_from_position() {
    let t = RectTransform::new_position(vec2(10.0, 20.0));
    assert_eq!(t.position, vec2(10.0, 20.0));
}

#[test]
fn rect_transform_to_world_point_no_rotation() {
    let t = RectTransform::default().position(vec2(100.0, 200.0));
    let result = t.to_world_point(vec2(10.0, 20.0));
    assert!((result - vec2(110.0, 220.0)).length() < 1e-6);
}

#[test]
fn rect_transform_to_world_point_with_scale() {
    let t = RectTransform::default()
        .position(vec2(0.0, 0.0))
        .scale(vec2(2.0, 3.0));
    let result = t.to_world_point(vec2(5.0, 10.0));
    assert!((result - vec2(10.0, 30.0)).length() < 1e-6);
}

#[test]
fn rect_transform_to_world_transform() {
    let parent = RectTransform::default().position(vec2(100.0, 100.0));
    let child = RectTransform::default().position(vec2(10.0, 20.0));
    let combined = parent.to_world_transform(child);
    assert!((combined.position - vec2(110.0, 120.0)).length() < 1e-6);
}

#[test]
fn rect_transform_to_local_point() {
    let t = RectTransform::new(
        vec2(10.0, 20.0),
        std::f32::consts::FRAC_PI_2,
        vec2(2.0, 2.0),
        0.0,
    );
    let world_pt = vec2(10.0, 40.0);
    let local_pt = t.to_local_point(world_pt);
    assert!((local_pt - vec2(10.0, 0.0)).length() < 1e-5);
}

#[test]
fn rect_transform_inverse_round_trip() {
    let t = RectTransform::new(vec2(10.0, 20.0), 0.5, vec2(2.0, 3.0), 1.5);
    let inv = t.inverse();
    let combined = t * inv;
    assert!((combined.position - Vec2::ZERO).length() < 1e-5);
    assert!((combined.rotation - 0.0).abs() < 1e-5);
    assert!((combined.scale - Vec2::ONE).length() < 1e-5);
    assert!((combined.z_index - 0.0).abs() < 1e-5);
}

#[test]
fn rect_transform_from_conversions() {
    let t1: RectTransform = vec2(5.0, 10.0).into();
    assert_eq!(t1.position, vec2(5.0, 10.0));

    let t2: RectTransform = (3.0_f32, 4.0_f32).into();
    assert_eq!(t2.position, vec2(3.0, 4.0));

    let t3: RectTransform = [1.0_f32, 2.0_f32].into();
    assert_eq!(t3.position, vec2(1.0, 2.0));

    let t4: RectTransform = (7_i32, 8_i32).into();
    assert_eq!(t4.position, vec2(7.0, 8.0));
}

// ============================================================================
// Text
// ============================================================================

#[test]
fn text_new() {
    let t = Text::new("Hello");
    assert_eq!(t.content, "Hello");
    assert_eq!(t.scale, 1.0);
    assert_eq!(t.anchor_x, Anchor::Start);
    assert_eq!(t.anchor_y, Anchor::Start);
    assert!(t.shadow_offset.is_none());
    assert!(t.max_width.is_none());
}

#[test]
fn text_builder_pattern() {
    let t = Text::new("World")
        .scale(2.0)
        .anchor(Anchor::Center, Anchor::End)
        .max_width(200.0);
    assert_eq!(t.scale, 2.0);
    assert_eq!(t.anchor_x, Anchor::Center);
    assert_eq!(t.anchor_y, Anchor::End);
    assert_eq!(t.max_width, Some(200.0));
}

#[test]
fn text_shadow_toggle() {
    let t = Text::new("Shadow").shadow(true);
    assert!(t.shadow_offset.is_some());
    assert!(t.shadow_color.is_some());

    let t2 = t.shadow(false);
    assert!(t2.shadow_offset.is_none());
    assert!(t2.shadow_color.is_none());
}

#[test]
fn text_from_string() {
    let t: Text = "auto".into();
    assert_eq!(t.content, "auto");
}

#[test]
fn text_max_width_zero_treated_as_none() {
    let t = Text::new("test").max_width(0.0);
    assert_eq!(t.max_width, None);
}

// ============================================================================
// Camera
// ============================================================================

#[test]
fn camera_default() {
    let cam = Camera::new();
    assert_eq!(cam.fovy_deg, 45.0);
    assert_eq!(cam.z_near, 0.1);
    assert_eq!(cam.z_far, 100.0);
}

#[test]
fn camera_look_at() {
    let cam = Camera::look_at(vec3(0.0, 5.0, 10.0), vec3(0.0, 0.0, 0.0), Vec3::Y);
    // The camera should be positioned approximately at the eye position
    let eye = cam.transform.position;
    assert!((eye - vec3(0.0, 5.0, 10.0)).length() < 1e-4);
}

#[test]
fn camera_builder_pattern() {
    let cam = Camera::new().fovy(60.0).near(0.5).far(500.0);
    assert_eq!(cam.fovy_deg, 60.0);
    assert_eq!(cam.z_near, 0.5);
    assert_eq!(cam.z_far, 500.0);
}

#[test]
fn camera_view_matrix_identity_camera() {
    let cam = Camera::new();
    let view = cam.view_matrix();
    // Default camera at origin should produce identity-ish view matrix
    let diff = view - cam.transform.to_matrix().inverse();
    for i in 0..4 {
        assert!(diff.col(i).length() < 1e-5);
    }
}

#[test]
fn camera_proj_matrix_valid() {
    let cam = Camera::new();
    let proj = cam.proj_matrix(16.0 / 9.0);
    // Projection matrix should not be identity
    assert!((proj - Mat4::IDENTITY).col(0).length() > 0.01);
}

#[test]
fn camera_world_to_screen_origin() {
    let cam = Camera::look_at(vec3(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
    let screen_size = vec2(320.0, 240.0);
    let result = cam.world_to_screen(Vec3::ZERO, screen_size);
    assert!(result.is_some());
    let pos = result.unwrap();
    // Origin should project near the center of the screen
    assert!((pos.x - 160.0).abs() < 1.0);
    assert!((pos.y - 120.0).abs() < 1.0);
}

#[test]
fn camera_world_to_screen_behind_returns_none() {
    let cam = Camera::look_at(vec3(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
    let screen_size = vec2(320.0, 240.0);
    // Point behind the camera
    let result = cam.world_to_screen(vec3(0.0, 0.0, 20.0), screen_size);
    assert!(result.is_none());
}

// ============================================================================
// Light
// ============================================================================

#[test]
fn light_default() {
    let l = Light::new();
    assert_eq!(l.color, Color::new(1.0, 1.0, 1.0));
    assert_eq!(l.ambient, Color::new(0.2, 0.2, 0.2));
    // Direction should be normalized
    assert!((l.direction.length() - 1.0).abs() < 1e-6);
}

#[test]
fn light_builder_pattern() {
    let l = Light::new()
        .direction(vec3(0.0, -1.0, 0.0))
        .color(Color::YELLOW)
        .ambient(Color::new(0.1, 0.1, 0.1));
    assert!((l.direction - vec3(0.0, -1.0, 0.0)).length() < 1e-6);
    assert_eq!(l.color, Color::YELLOW);
    assert_eq!(l.ambient, Color::new(0.1, 0.1, 0.1));
}

#[test]
fn light_direction_normalized() {
    let l = Light::new().direction(vec3(3.0, 4.0, 0.0));
    assert!((l.direction.length() - 1.0).abs() < 1e-6);
}

// ============================================================================
// Random
// ============================================================================

#[test]
fn random_deterministic_with_seed() {
    let mut rng1 = Random::new(42);
    let mut rng2 = Random::new(42);
    for _ in 0..10 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn random_different_seeds_differ() {
    let mut rng1 = Random::new(1);
    let mut rng2 = Random::new(2);
    // Very unlikely to be equal
    assert_ne!(rng1.next_u64(), rng2.next_u64());
}

#[test]
fn random_f32_in_range() {
    let mut rng = Random::new(123);
    for _ in 0..100 {
        let v = rng.next_f32();
        assert!(
            v >= 0.0 && v < 1.0,
            "next_f32 should be in [0.0, 1.0), got {v}"
        );
    }
}

#[test]
fn random_range() {
    let mut rng = Random::new(456);
    for _ in 0..100 {
        let v = rng.range(10.0, 20.0);
        assert!(
            v >= 10.0 && v < 20.0,
            "range should be in [10.0, 20.0), got {v}"
        );
    }
}

#[test]
fn random_int_range() {
    let mut rng = Random::new(789);
    for _ in 0..100 {
        let v = rng.int_range(0, 5);
        assert!(v >= 0 && v <= 5, "int_range should be in [0, 5], got {v}");
    }
}

#[test]
fn random_int_range_same_value() {
    let mut rng = Random::new(42);
    let v = rng.int_range(7, 7);
    assert_eq!(v, 7);
}

#[test]
fn random_chance_never() {
    let mut rng = Random::new(42);
    for _ in 0..100 {
        assert!(!rng.chance(0.0));
    }
}

#[test]
fn random_chance_always() {
    let mut rng = Random::new(42);
    for _ in 0..100 {
        assert!(rng.chance(1.0));
    }
}

#[test]
fn random_vec3_in_sphere() {
    let mut rng = Random::new(42);
    for _ in 0..50 {
        let v = rng.vec3_in_sphere();
        assert!(
            v.length() <= 1.0 + 1e-6,
            "Point should be inside unit sphere"
        );
    }
}

#[test]
fn random_vec3_on_sphere() {
    let mut rng = Random::new(42);
    for _ in 0..50 {
        let v = rng.vec3_on_sphere();
        assert!(
            (v.length() - 1.0).abs() < 1e-5,
            "Point should be on unit sphere surface"
        );
    }
}

#[test]
fn random_color() {
    let mut rng = Random::new(42);
    let c = rng.color();
    assert!(c.r >= 0.0 && c.r < 1.0);
    assert!(c.g >= 0.0 && c.g < 1.0);
    assert!(c.b >= 0.0 && c.b < 1.0);
}

#[test]
fn random_zero_seed() {
    // seed 0 should not cause issues
    let mut rng = Random::new(0);
    let v = rng.next_u64();
    assert_ne!(v, 0, "seed=0 should still produce non-zero output");
}

// ============================================================================
// Animation
// ============================================================================

#[test]
fn animated_new_initial_value() {
    let anim = Animated::new(5.0_f32, AnimationMode::linear());
    assert_eq!(anim.current(), 5.0);
    assert_eq!(anim.target(), 5.0);
}

#[test]
fn animated_set_target() {
    let mut anim = Animated::new(0.0_f32, AnimationMode::linear());
    anim.set_target(10.0);
    assert_eq!(anim.target(), 10.0);
    assert_eq!(anim.current(), 0.0);
}

#[test]
fn animated_linear_approaches_target() {
    let mut anim = Animated::new(0.0_f32, AnimationMode::linear().speed(10.0));
    anim.set_target(10.0);
    // After enough time, should reach the target
    for _ in 0..100 {
        anim.update(0.016);
    }
    assert!((anim.current() - 10.0).abs() < 0.01);
}

#[test]
fn animated_exp_approaches_target() {
    let mut anim = Animated::new(0.0_f32, AnimationMode::exp().speed(10.0));
    anim.set_target(100.0);
    for _ in 0..200 {
        anim.update(0.016);
    }
    assert!((anim.current() - 100.0).abs() < 0.1);
}

#[test]
fn animated_spring_approaches_target() {
    let mut anim = Animated::new(
        0.0_f32,
        AnimationMode::spring().stiffness(100.0).damping(20.0),
    );
    anim.set_target(50.0);
    for _ in 0..500 {
        anim.update(0.016);
    }
    assert!((anim.current() - 50.0).abs() < 0.5);
}

#[test]
fn animated_set_current_resets() {
    let mut anim = Animated::new(0.0_f32, AnimationMode::spring());
    anim.set_target(100.0);
    anim.update(0.1);
    anim.set_current(50.0);
    assert_eq!(anim.current(), 50.0);
    assert_eq!(anim.velocity(), 0.0);
}

#[test]
fn animated_zero_dt_no_change() {
    let mut anim = Animated::new(5.0_f32, AnimationMode::linear());
    anim.set_target(10.0);
    anim.update(0.0);
    assert_eq!(anim.current(), 5.0);
}

#[test]
fn animated_vec3() {
    let mut anim = Animated::new(Vec3::ZERO, AnimationMode::linear().speed(10.0));
    anim.set_target(vec3(10.0, 0.0, 0.0));
    for _ in 0..100 {
        anim.update(0.016);
    }
    assert!((anim.current() - vec3(10.0, 0.0, 0.0)).length() < 0.1);
}

#[test]
fn animation_mode_builder_from_conversion() {
    let mode: AnimationMode = AnimationMode::linear().speed(3.0).into();
    assert_eq!(mode, AnimationMode::Linear { speed: 3.0 });

    let mode2: AnimationMode = AnimationMode::exp().speed(7.0).into();
    assert_eq!(mode2, AnimationMode::Exp { speed: 7.0 });

    let mode3: AnimationMode = AnimationMode::spring().stiffness(100.0).damping(5.0).into();
    assert_eq!(
        mode3,
        AnimationMode::Spring {
            stiffness: 100.0,
            damping: 5.0
        }
    );
}

// ============================================================================
// Shape Builders
// ============================================================================

#[test]
fn shape_cube_builds() {
    let _shape: Shape = Shape::cube().build();
}

#[test]
fn shape_sphere_builds() {
    let _shape: Shape = Shape::sphere().build();
}

#[test]
fn shape_cylinder_builds() {
    let _shape: Shape = Shape::cylinder().build();
}

#[test]
fn shape_cone_builds() {
    let _shape: Shape = Shape::cone().build();
}

#[test]
fn shape_torus_builds() {
    let _shape: Shape = Shape::torus().build();
}

#[test]
fn shape_capsule_builds() {
    let _shape: Shape = Shape::capsule().build();
}

#[test]
fn shape_plane_builds() {
    let _shape: Shape = Shape::plane().build();
}

#[test]
fn shape_disk_builds() {
    let _shape: Shape = Shape::disk().build();
}

#[test]
fn shape_pyramid_builds() {
    let _shape: Shape = Shape::pyramid().build();
}

#[test]
fn shape_ico_sphere_builds() {
    let _shape: Shape = Shape::ico_sphere().build();
}

#[test]
fn shape_into_conversion() {
    let _shape: Shape = Shape::cube().into();
}

#[test]
fn shape_equality() {
    let a = Shape::cube().build();
    let b = Shape::cube().build();
    assert_eq!(a, b);
}

#[test]
fn shape_inequality() {
    let cube = Shape::cube().build();
    let sphere = Shape::sphere().build();
    assert_ne!(cube, sphere);
}

#[test]
fn shape_lit_unlit() {
    let s = Shape::cube().unlit().build();
    assert!(!s.is_lit());

    let s2 = Shape::cube().lit(true).build();
    assert!(s2.is_lit());
}

// ============================================================================
// Audio (Pitch)
// ============================================================================

#[test]
fn pitch_a4_is_440hz() {
    let freq = Pitch::A(4).freq();
    assert!((freq - 440.0).abs() < 0.01);
}

#[test]
fn pitch_c4_is_middle_c() {
    let freq = Pitch::C(4).freq();
    assert!((freq - 261.63).abs() < 0.1);
}

#[test]
fn pitch_octave_transpose() {
    let a4 = Pitch::A(4);
    let a5 = a4.octave(1);
    let freq_ratio = a5.freq() / a4.freq();
    assert!(
        (freq_ratio - 2.0).abs() < 0.01,
        "Octave up should double the frequency"
    );
}

#[test]
fn pitch_into_freq_trait() {
    let freq: f32 = Pitch::A(4).into_freq();
    assert!((freq - 440.0).abs() < 0.01);

    let direct: f32 = 440.0_f32.into_freq();
    assert_eq!(direct, 440.0);
}

#[test]
fn pitch_enharmonic_equivalence() {
    // Cs and Db should have the same frequency
    let cs4 = Pitch::Cs(4).freq();
    let db4 = Pitch::Db(4).freq();
    assert!((cs4 - db4).abs() < 0.01);
}

// ============================================================================
// Animatable trait
// ============================================================================

#[test]
fn animatable_f32_lerp() {
    let result = Animatable::lerp(0.0_f32, 10.0, 0.5);
    assert!((result - 5.0).abs() < 1e-6);
}

#[test]
fn animatable_vec3_lerp() {
    let a = Vec3::ZERO;
    let b = vec3(10.0, 20.0, 30.0);
    let mid = a.lerp(b, 0.5);
    assert!((mid - vec3(5.0, 10.0, 15.0)).length() < 1e-5);
}
