// --- Vertex Shader ---
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generates a Full-Screen Triangle (CCW Winding)
    // Vertices: (-1, -1), (3, -1), (-1, 3)
    // This ensures the triangle covers the entire clip space and passes culling.
    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    
    // Align UVs so (0,0) is Bottom-Left (Shadertoy Standard)
    out.uv = uv; 
    
    return out;
}

// --- Fragment Shader Driver ---

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    padding: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

// We assume 'fn mainImage(uv: vec2<f32>, time: f32) -> vec4<f32>' exists 
// because main.rs injects 'generated.wgsl' before this file.

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // MUSUBI: Calling the SumiC generated function
    return mainImage(in.uv, u.time);
}
