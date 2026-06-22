struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) radius: vec4<f32>, // [top_left, top_right, bottom_right, bottom_left]
    @location(5) outline_width: f32,
    @location(6) outline_color: vec4<f32>,
    @location(7) shadow_offset: vec2<f32>,
    @location(8) shadow_color: vec4<f32>,
    @location(9) z_index: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radius: vec4<f32>,
    @location(4) outline_width: f32,
    @location(5) outline_color: vec4<f32>,
    @location(6) shadow_offset: vec2<f32>,
    @location(7) shadow_color: vec4<f32>,
}

struct ScreenSizeUniform {
    size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> screen_size: ScreenSizeUniform;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let ndc_x = (model.position.x / screen_size.size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (model.position.y / screen_size.size.y) * 2.0;
    let ndc_z = clamp(1.0 - (model.z_index / 10000.0), 0.0, 1.0);
    out.clip_position = vec4<f32>(ndc_x, ndc_y, ndc_z, 1.0);
    out.local_pos = model.local_pos;
    out.size = model.size;
    out.color = model.color;
    out.radius = model.radius;
    out.outline_width = model.outline_width;
    out.outline_color = model.outline_color;
    out.shadow_offset = model.shadow_offset;
    out.shadow_color = model.shadow_color;
    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    // Select the corner radius for each quadrant
    // r.x: top_left (p.x < 0, p.y < 0)
    // r.y: top_right (p.x > 0, p.y < 0)
    // r.z: bottom_right (p.x > 0, p.y > 0)
    // r.w: bottom_left (p.x < 0, p.y > 0)
    var r_corner = 0.0;
    if (p.x > 0.0) {
        if (p.y > 0.0) {
            r_corner = r.z; // bottom_right
        } else {
            r_corner = r.y; // top_right
        }
    } else {
        if (p.y > 0.0) {
            r_corner = r.w; // bottom_left
        } else {
            r_corner = r.x; // top_left
        }
    }

    let max_radius = min(b.x, b.y);
    let rad = min(r_corner, max_radius);

    let q = abs(p) - b + vec2<f32>(rad);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - rad;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = in.size * 0.5;

    // SDF of the rect body
    let d = sd_rounded_box(in.local_pos, half_size, in.radius);

    // SDF of the shadow (shifting sampling points in opposite direction of offset)
    let d_shadow = sd_rounded_box(in.local_pos - in.shadow_offset, half_size, in.radius);

    // Binary classification (no anti-aliasing)
    let is_inside_rect = d <= 0.0;
    let is_inside_outline = in.outline_width > 0.0 && d >= -in.outline_width && d <= 0.0;
    let is_inside_shadow = in.shadow_color.a > 0.0 && d_shadow <= 0.0;

    // Color determination priority:
    // 1. Rect body interior (excluding outline)
    // 2. Outline
    // 3. Shadow
    if (is_inside_rect) {
        if (is_inside_outline) {
            return in.outline_color;
        } else {
            return in.color;
        }
    } else if (is_inside_shadow) {
        return in.shadow_color;
    }

    discard;
}
