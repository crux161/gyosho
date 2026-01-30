/// KANTEI: The Judge / The Appraisal System
///
/// This module defines the "Grades" of hardware capability.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    /// **1st Dan (Ink)**: Pure CPU / Software Rasterizer.
    /// Safe for: ESP32, Embedded, Playdate, Server (Eshi).
    /// Restrictions: No Floating Point Textures, No Compute Shaders.
    Ink = 1,

    /// **2nd Dan (Paper)**: Basic GPU (OpenGL ES 2.0 / WebGL 1).
    /// Safe for: Raspberry Pi 4, Old Android, Standard Web.
    /// Restrictions: Limited Texture Units, No Compute.
    Paper = 2,

    /// **3rd Dan (Brush)**: Modern GPU (Metal / Vulkan / DX12).
    /// Safe for: iPhone (post-2018), PC, Mac, Modern WebGPU.
    /// Restrictions: Standard Graphics Pipeline only.
    Brush = 3,

    /// **4th Dan (Gold)**: Specialized Hardware (RT Cores, Neural Engine).
    /// Safe for: M-Series Mac, RTX Cards.
    /// Restrictions: Requires specific vendor extensions.
    Gold = 4,
}

/// A marker trait for hardware capabilities.
/// Functions can require these to enforce Kantei checks.
pub trait Capability {
    fn required_grade() -> Grade;
}


/// Requires a standard rasterization pipeline (Triangles)
pub struct Rasterizer;
impl Capability for Rasterizer { fn required_grade() -> Grade { Grade::Paper } }

/// Requires Compute Shaders (General Purpose GPU Math)
pub struct Compute;
impl Capability for Compute { fn required_grade() -> Grade { Grade::Brush } }

/// Requires Hardware Ray Tracing
pub struct RayTracing;
impl Capability for RayTracing { fn required_grade() -> Grade { Grade::Gold } }

/// Requires 64-bit Floating Point Precision (Double)
pub struct Float64;
impl Capability for Float64 { fn required_grade() -> Grade { Grade::Ink } } // CPU can always do f64!
