//! Voxel raycast: Amanatides & Woo DDA. Used for block break/place picking.

use crate::world::World;
use glam::{IVec3, Vec3};

pub struct RayHit {
    /// Voxel that was hit.
    pub block: IVec3,
    /// Face normal of the hit (the voxel adjacent where placement would go).
    pub normal: IVec3,
    pub dist: f32,
}

/// March `dir` (must be normalized) from `origin` up to `max_dist` blocks.
/// Returns the first solid voxel, or None.
pub fn raycast(world: &World, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<RayHit> {
    let mut voxel = origin.floor().as_ivec3();
    let step = dir.signum().as_ivec3();

    // Distance along the ray to the next voxel boundary, per axis.
    let mut t_max = Vec3::ZERO;
    for i in 0..3 {
        t_max[i] = if dir[i] > 0.0 {
            (voxel[i] as f32 + 1.0 - origin[i]) / dir[i]
        } else if dir[i] < 0.0 {
            (origin[i] - voxel[i] as f32) / -dir[i]
        } else {
            f32::INFINITY
        };
    }
    let t_delta = Vec3::new(
        if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f32::INFINITY },
        if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f32::INFINITY },
        if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f32::INFINITY },
    );

    let mut normal = IVec3::ZERO;
    let mut dist = 0.0;
    // Check the starting voxel too (e.g. eye inside a solid block).
    for _ in 0..512 {
        if world.block_at(voxel.x, voxel.y, voxel.z).is_solid() {
            return Some(RayHit { block: voxel, normal, dist });
        }
        let axis = if t_max.x <= t_max.y && t_max.x <= t_max.z {
            0
        } else if t_max.y <= t_max.z {
            1
        } else {
            2
        };
        voxel[axis] += step[axis];
        dist = t_max[axis];
        t_max[axis] += t_delta[axis];
        normal = IVec3::ZERO;
        normal[axis] = -step[axis];
        if dist > max_dist {
            return None;
        }
    }
    None
}
