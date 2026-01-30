use crate::math::{Vec2, Vec3, floor, fract, dot, mix};

/// A stateless hash function: Input (x,y) -> Output (Random Float 0..1)
/// "Gold" for shaders because it requires no texture lookups.
pub fn hash(p: Vec2) -> f32 {
    // Porting the classic "hash12" logic commonly used in shaders.
    // Ensure we work in Vec3 space for the intermediate calculation as per standard GLSL implementations
    let p3_in = Vec3::new(p.x, p.y, p.x);
    let mut p3 = fract(p3_in * 0.1031);
    
    // dot(Vec3, Vec3) works now thanks to DotProduct trait
    p3 += dot(p3, Vec3::new(p3.y, p3.z, p3.x) + 33.33); 
    
    // Return fractional part of scalar result
    ((p3.x + p3.y) * p3.z).fract()
}

/// 2D Noise (Value Noise)
pub fn noise(p: Vec2) -> f32 {
    let i = floor(p); // Works for Vec2 via GenType
    let f = fract(p); // Works for Vec2 via GenType

    // Four corners in 2D of a tile
    let a = hash(i);
    let b = hash(i + Vec2::new(1.0, 0.0));
    let c = hash(i + Vec2::new(0.0, 1.0));
    let d = hash(i + Vec2::new(1.0, 1.0));

    // Cubic Hermite Interpolation (same as smoothstep)
    let u = f * f * (3.0 - 2.0 * f);

    // Mix 4 corners percentages
    mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y
}

/// Fractal Brownian Motion (FBM) - The "Cloudy" look
pub fn fbm(mut p: Vec2, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    
    for _ in 0..octaves {
        value += amplitude * noise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    value
}
