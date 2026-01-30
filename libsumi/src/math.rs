use glam;

// Re-export specific glam types
pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type Mat2 = glam::Mat2;
pub type Mat3 = glam::Mat3;
pub type Mat4 = glam::Mat4;
pub type Quat = glam::Quat;

// --- Traits for GLSL-like Polymorphism ---

pub trait GenType: Copy {
    fn floor(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
    fn mix(self, other: Self, t: f32) -> Self;
}

pub trait DotProduct: Copy {
    fn dot(self, other: Self) -> f32;
}

// --- Implementations ---

// f32
impl GenType for f32 {
    fn floor(self) -> Self { self.floor() }
    fn fract(self) -> Self { self.fract() }
    fn abs(self) -> Self { self.abs() }
    fn min(self, other: Self) -> Self { self.min(other) }
    fn max(self, other: Self) -> Self { self.max(other) }
    fn clamp(self, min: Self, max: Self) -> Self { self.clamp(min, max) }
    fn mix(self, other: Self, t: f32) -> Self { self * (1.0 - t) + other * t }
}
impl DotProduct for f32 {
    fn dot(self, other: Self) -> f32 { self * other }
}

// Vec2
impl GenType for Vec2 {
    fn floor(self) -> Self { self.floor() }
    fn fract(self) -> Self { self - self.floor() }
    fn abs(self) -> Self { self.abs() }
    fn min(self, other: Self) -> Self { self.min(other) }
    fn max(self, other: Self) -> Self { self.max(other) }
    fn clamp(self, min: Self, max: Self) -> Self { self.clamp(min, max) }
    fn mix(self, other: Self, t: f32) -> Self { self.lerp(other, t) }
}
impl DotProduct for Vec2 {
    fn dot(self, other: Self) -> f32 { self.dot(other) }
}

// Vec3
impl GenType for Vec3 {
    fn floor(self) -> Self { self.floor() }
    fn fract(self) -> Self { self - self.floor() }
    fn abs(self) -> Self { self.abs() }
    fn min(self, other: Self) -> Self { self.min(other) }
    fn max(self, other: Self) -> Self { self.max(other) }
    fn clamp(self, min: Self, max: Self) -> Self { self.clamp(min, max) }
    fn mix(self, other: Self, t: f32) -> Self { self.lerp(other, t) }
}
impl DotProduct for Vec3 {
    fn dot(self, other: Self) -> f32 { self.dot(other) }
}

// Vec4
impl GenType for Vec4 {
    fn floor(self) -> Self { self.floor() }
    fn fract(self) -> Self { self - self.floor() }
    fn abs(self) -> Self { self.abs() }
    fn min(self, other: Self) -> Self { self.min(other) }
    fn max(self, other: Self) -> Self { self.max(other) }
    fn clamp(self, min: Self, max: Self) -> Self { self.clamp(min, max) }
    fn mix(self, other: Self, t: f32) -> Self { self.lerp(other, t) }
}
impl DotProduct for Vec4 {
    fn dot(self, other: Self) -> f32 { self.dot(other) }
}

// --- Global GLSL Wrappers ---

pub fn floor<T: GenType>(x: T) -> T { x.floor() }
pub fn fract<T: GenType>(x: T) -> T { x.fract() }
pub fn abs<T: GenType>(x: T) -> T { x.abs() }
pub fn min<T: GenType>(a: T, b: T) -> T { a.min(b) }
pub fn max<T: GenType>(a: T, b: T) -> T { a.max(b) }
pub fn clamp<T: GenType>(v: T, min_val: T, max_val: T) -> T { v.clamp(min_val, max_val) }
pub fn mix<T: GenType>(x: T, y: T, a: f32) -> T { x.mix(y, a) }
pub fn dot<T: DotProduct>(x: T, y: T) -> f32 { x.dot(y) }

// --- Scalar Only Math ---
// These strictly return f32 and usually take f32, or specific vector ops
pub fn radians(degrees: f32) -> f32 { degrees.to_radians() }
pub fn degrees(radians: f32) -> f32 { radians.to_degrees() }
pub fn sin(n: f32) -> f32 { n.sin() }
pub fn cos(n: f32) -> f32 { n.cos() }
pub fn tan(n: f32) -> f32 { n.tan() }
pub fn asin(n: f32) -> f32 { n.asin() }
pub fn acos(n: f32) -> f32 { n.acos() }
pub fn atan(y: f32, x: f32) -> f32 { y.atan2(x) }
pub fn sqrt(n: f32) -> f32 { n.sqrt() }
pub fn exp(n: f32) -> f32 { n.exp() }
pub fn log(n: f32) -> f32 { n.ln() }
pub fn pow(n: f32, e: f32) -> f32 { n.powf(e) }
pub fn sign(n: f32) -> f32 { n.signum() }
pub fn ceil(n: f32) -> f32 { n.ceil() }
pub fn step(edge: f32, x: f32) -> f32 { if x < edge { 0.0 } else { 1.0 } }
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// --- Vector Utils ---
pub fn cross(a: Vec3, b: Vec3) -> Vec3 { a.cross(b) }
pub fn length2(v: Vec2) -> f32 { v.length() }
pub fn length3(v: Vec3) -> f32 { v.length() }
pub fn normalize2(v: Vec2) -> Vec2 { v.normalize() }
pub fn normalize3(v: Vec3) -> Vec3 { v.normalize() }
pub fn reflect(i: Vec3, n: Vec3) -> Vec3 { i - 2.0 * i.dot(n) * n }
