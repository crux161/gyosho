use crate::math::{Vec3, max, min, length3, abs, mix, clamp};

/// Signed Distance Function: Sphere
pub fn sd_sphere(p: Vec3, r: f32) -> f32 {
    length3(p) - r
}

/// Signed Distance Function: Box
pub fn sd_box(p: Vec3, b: Vec3) -> f32 {
    let q = abs(p) - b;
    
    // FIX: max(q, 0.0) -> max(q, Vec3::ZERO)
    // We mix types explicitly because Rust won't auto-promote float to vec3
    let zero = Vec3::ZERO;
    
    let outside = length3(max(q, zero));
    let inside = min(max(q.x, max(q.y, q.z)), 0.0);
    
    outside + inside
}

pub fn op_union(d1: f32, d2: f32) -> f32 {
    min(d1, d2)
}

pub fn op_subtraction(d1: f32, d2: f32) -> f32 {
    max(-d1, d2)
}

pub fn op_intersection(d1: f32, d2: f32) -> f32 {
    max(d1, d2)
}

pub fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    mix(d2, d1, h) - k * h * (1.0 - h)
}
