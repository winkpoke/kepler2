// Vertex shader

struct Uniforms {
    rotation_angle_y: f32,
    rotation_angle_z: f32,
    // window: f32,
    // level: f32,
    // slice: f32,
    _padding0: f32,
    _padding1: f32,
};

@group(1) @binding(0)
var<uniform> u_uniform: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Apply rotation (you may want to adjust this)
    let u_rotation_z = u_uniform.rotation_angle_z;
    let rotation_matrix_z = mat4x4<f32>(
        cos(u_rotation_z), -sin(u_rotation_z), 0.0, 0.0,
        sin(u_rotation_z),  cos(u_rotation_z), 0.0, 0.0,
        0.0,               0.0,                1.0, 0.0,
        0.0,               0.0,                0.0, 1.0
    );
    let u_rotation_y = u_uniform.rotation_angle_y;
    let rotation_matrix_y = mat4x4<f32>(
        cos(u_rotation_y), 0.0, sin(u_rotation_y), 0.0,
        0.0,               1.0, 0.0,               0.0,
       -sin(u_rotation_y), 0.0, cos(u_rotation_y), 0.0,
        0.0,               0.0,                0.0, 1.0
    );

    // Set the output
    out.tex_coords = model.tex_coords;
    // out.clip_position = vec4<f32>(model.position, 1.0);
    out.clip_position = rotation_matrix_z * rotation_matrix_y * vec4<f32>(model.position, 1.0);
    out.clip_position.z += 0.5;
    return out;
}
// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct UniformsFrag {
    window: f32,
    level: f32,
    slice: f32,
    _padding: f32,
}

@group(2) @binding(0)
var<uniform> u_uniform_frag: UniformsFrag;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
