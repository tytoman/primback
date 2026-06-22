struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> light: LightUniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.world_normal = model.normal;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal_len = length(in.world_normal);
    if (normal_len < 0.01) {
        return in.color;
    }
    let N = normalize(in.world_normal);
    let L = normalize(-light.direction.xyz);

    // Calculate diffuse and ambient lighting
    let diffuse_factor = max(dot(N, L), 0.0);
    let diffuse = light.color.rgb * diffuse_factor;
    let ambient = light.ambient.rgb;

    let lighting = ambient + diffuse;

    // Clamp the color to prevent oversaturation and apply
    let final_rgb = clamp(in.color.rgb * lighting, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(final_rgb, in.color.a);
}
