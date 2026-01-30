use glam;

pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type Mat4 = glam::Mat4;
pub type Quat = glam::Quat;

// GYOSHO EXTENSIONS

pub trait SumiMathExt {
    /// A "Sumi-style" linear interpolation that feels more organic
    fn lerp_organic(self, end: Self, t: f32) -> Self;
}

impl SumiMathExt for Vec3 {
    fn lerp_organic(self, end: Self, t: f32) -> Self {
        self.lerp(end, t)
    }
}
