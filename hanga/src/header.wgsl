struct Uniforms {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    resolution: vec2<f32>,
    time: f32,
    padding: f32,
    mouse: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;
