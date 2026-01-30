use crate::math::{Vec3, Mat4};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_radians: f32,
    pub aspect_ratio: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_radians: 45.0f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
}

impl Camera {
    /// 1. The View Matrix (World Space -> Camera Space)
    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// 2. The Projection Matrix (Camera Space -> Clip Space)
    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_radians, self.aspect_ratio, self.near_clip, self.far_clip)
    }
}
