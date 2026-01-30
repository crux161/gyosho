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
// (Uniforms 'u' are now available globally from header.wgsl)

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let fragCoord = in.uv * u.resolution;
    
    // Call SumiC function. We only pass fragCoord.
    // iTime, iResolution, etc. are handled as globals by SumiC translation.
    return mainImage(fragCoord);
}
