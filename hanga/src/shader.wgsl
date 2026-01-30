// --- Vertex Shader ---
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv; 
    return out;
}

// --- Fragment Shader Driver ---

struct Uniforms {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    resolution: vec2<f32>,
    time: f32,
    padding: f32,
    mouse: vec4<f32>, // New!
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

// DRIVER: Maps S2L 'mainImage' to WGSL entry point
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // We transform UV to Fragment Coordinates for Shadertoy compat
    let fragCoord = in.uv * u.resolution;
    
    // We assume the S2L function signature:
    // fn mainImage(fragCoord: vec2, iResolution: vec3, iTime: f32, iMouse: vec4) -> vec4
    // Note: iResolution is vec3 in shadertoy (z is pixel aspect, usually 1.0)
    
    let iRes = vec3<f32>(u.resolution, 1.0);
    
    return mainImage(fragCoord, iRes, u.time, u.mouse);
}
