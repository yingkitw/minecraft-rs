//! Camera math: wgpu-convention perspective + look-at view matrix.

use glam::{Mat4, Vec3, Vec4};

/// Right-handed perspective with wgpu clip depth z in [0, 1]
/// (glam's `perspective_rh` targets OpenGL's [-1, 1], so we build our own).
pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fovy * 0.5).tan();
    Mat4::from_cols(
        Vec4::new(f / aspect, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f, 0.0, 0.0),
        Vec4::new(0.0, 0.0, far / (near - far), -1.0),
        Vec4::new(0.0, 0.0, near * far / (near - far), 0.0),
    )
}

/// Right-handed look-at view matrix (glam no longer ships `look_to_rh`).
pub fn look_to(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
    let f = dir.normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);
    Mat4::from_cols(
        Vec4::new(s.x, u.x, -f.x, 0.0),
        Vec4::new(s.y, u.y, -f.y, 0.0),
        Vec4::new(s.z, u.z, -f.z, 0.0),
        Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
    )
}

/// Combined view-projection for a camera at `eye` looking along `dir`.
pub fn view_proj(eye: Vec3, dir: Vec3, aspect: f32) -> Mat4 {
    perspective(70f32.to_radians(), aspect, 0.1, 500.0) * look_to(eye, dir, Vec3::Y)
}
