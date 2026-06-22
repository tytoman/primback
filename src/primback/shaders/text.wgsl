struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) z_index: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
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
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    return out;
}

@group(1) @binding(0)
var font_texture: texture_2d<f32>;
@group(1) @binding(1)
var font_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(font_texture, font_sampler, in.tex_coords);
    let alpha = tex_color.r;

    if (alpha < 0.05) {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
