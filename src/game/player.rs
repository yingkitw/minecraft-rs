//! First-person player: WASD + jump + fly, AABB vs voxel collision.
//! Collision resolves one axis at a time (standard Minecraft approach).

use crate::world::World;
use crate::world::chunk::CHUNK_Y;
use glam::Vec3;

pub const EYE_HEIGHT: f32 = 1.62;
pub const HALF_WIDTH: f32 = 0.3;
pub const HEIGHT: f32 = 1.8;
const GRAVITY: f32 = 26.0;
const JUMP_VEL: f32 = 8.5;
const WALK_SPEED: f32 = 5.5;
const FLY_SPEED: f32 = 12.0;
const EPS: f32 = 1e-3;

#[derive(Default, Clone, Copy)]
pub struct MoveInput {
    pub forward: f32, // -1..1
    pub strafe: f32,  // -1..1
    pub jump: bool,
    pub down: bool,
}

pub struct Player {
    /// Center of the feet (bottom-center of the AABB).
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,   // radians, 0 = -z
    pub pitch: f32, // radians, clamped to +-pi/2
    pub on_ground: bool,
    pub flying: bool,
}

impl Player {
    pub fn new(pos: Vec3) -> Player {
        Player {
            pos,
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            flying: false,
        }
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::Y * EYE_HEIGHT
    }

    /// Unit look direction from yaw/pitch. yaw=0 faces -z, +yaw turns right.
    pub fn look_dir(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(-sy * cp, sp, -cy * cp)
    }

    /// AABB as (min, max) in world space.
    pub fn aabb(&self) -> (Vec3, Vec3) {
        (
            self.pos - Vec3::new(HALF_WIDTH, 0.0, HALF_WIDTH),
            self.pos + Vec3::new(HALF_WIDTH, HEIGHT, HALF_WIDTH),
        )
    }

    /// True if the AABB at `pos` overlaps any solid block.
    fn collides(world: &World, pos: Vec3) -> bool {
        let (min, max) = (
            pos - Vec3::new(HALF_WIDTH, 0.0, HALF_WIDTH),
            pos + Vec3::new(HALF_WIDTH, HEIGHT, HALF_WIDTH),
        );
        for y in min.y.floor() as i32..=max.y.floor() as i32 {
            for z in min.z.floor() as i32..=max.z.floor() as i32 {
                for x in min.x.floor() as i32..=max.x.floor() as i32 {
                    if world.block_at(x, y, z).is_solid() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Whether a block placed at voxel (x,y,z) would intersect the player.
    pub fn overlaps_voxel(&self, x: i32, y: i32, z: i32) -> bool {
        let (min, max) = self.aabb();
        x as f32 + 1.0 > min.x
            && (x as f32) < max.x
            && y as f32 + 1.0 > min.y
            && (y as f32) < max.y
            && z as f32 + 1.0 > min.z
            && (z as f32) < max.z
    }

    pub fn step(&mut self, world: &World, input: MoveInput, dt: f32) {
        let (sy, cy) = self.yaw.sin_cos();
        // Forward on the ground plane: yaw=0 -> -z.
        let fwd = Vec3::new(-sy, 0.0, -cy);
        let right = Vec3::new(cy, 0.0, -sy);
        let wish = (fwd * input.forward + right * input.strafe).normalize_or_zero();

        if self.flying {
            self.vel = wish * FLY_SPEED;
            self.vel.y = match (input.jump, input.down) {
                (true, false) => FLY_SPEED,
                (false, true) => -FLY_SPEED,
                _ => 0.0,
            };
        } else {
            self.vel.x = wish.x * WALK_SPEED;
            self.vel.z = wish.z * WALK_SPEED;
            self.vel.y = (self.vel.y - GRAVITY * dt).max(-50.0);
            if input.jump && self.on_ground {
                self.vel.y = JUMP_VEL;
                self.on_ground = false;
            }
        }

        self.on_ground = false;
        self.move_axis(world, 0, self.vel.x * dt);
        self.move_axis(world, 2, self.vel.z * dt);
        self.move_axis(world, 1, self.vel.y * dt);

        // Safety net: never fall through the world.
        if self.pos.y < -16.0 {
            self.pos.y = CHUNK_Y as f32;
            self.vel = Vec3::ZERO;
        }
    }

    /// Move along one axis, then clamp the AABB against any solid voxel hit.
    fn move_axis(&mut self, world: &World, axis: usize, delta: f32) {
        if delta == 0.0 {
            return;
        }
        self.pos[axis] += delta;
        if !Self::collides(world, self.pos) {
            return;
        }
        // Binary-search-free clamp: step the position back out of the block.
        let (min, max) = self.aabb();
        let coord = if delta > 0.0 { max[axis] } else { min[axis] };
        let v = coord.floor() as i32;
        self.pos[axis] = if delta > 0.0 {
            match axis {
                1 => v as f32 - HEIGHT - EPS,
                _ => v as f32 - HALF_WIDTH - EPS,
            }
        } else {
            match axis {
                1 => (v + 1) as f32 + EPS,
                _ => (v + 1) as f32 + HALF_WIDTH + EPS,
            }
        };
        // The clamp above uses the deepest penetrated voxel found via `coord`;
        // verify it resolved and, if not, revert the move entirely.
        if Self::collides(world, self.pos) {
            self.pos[axis] -= delta;
        }
        if axis == 1 {
            if delta < 0.0 {
                self.on_ground = true;
            }
            self.vel.y = 0.0;
        } else {
            self.vel[axis] = 0.0;
        }
    }
}
