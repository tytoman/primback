struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) z_index: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
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
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
