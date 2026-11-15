//! Chunk mesher: emits one quad per visible (non-occluded) block face.
//! CPU-only so it can be unit-tested without a GPU.

use crate::block::{Block, Face};
use crate::world::World;
use crate::world::chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub shade: f32,
}

#[derive(Default)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Corner offsets for each face, CCW as seen from outside the cube.
/// Verified: `cross(v1 - v0, v2 - v0)` equals the outward normal.
const FACE_CORNERS: [([[f32; 3]; 4], Face); 6] = [
    ([[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]], Face::Top),
    ([[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]], Face::Bottom),
    ([[1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]], Face::North),
    ([[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]], Face::South),
    ([[1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]], Face::East),
    ([[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]], Face::West),
];

const UV_CORNERS: [[f32; 2]; 4] = [[0., 0.], [0., 1.], [1., 1.], [1., 0.]];

/// 4x4 tile atlas: tile index -> uv rectangle.
fn tile_uv(tile: u32) -> (f32, f32, f32) {
    let col = (tile % 4) as f32;
    let row = (tile / 4) as f32;
    (col * 0.25, row * 0.25, 0.25)
}

/// Build vertex/index buffers for chunk (cx, cz). Neighbor lookups go through
/// `world` so faces on chunk borders cull correctly against adjacent chunks.
pub fn build_mesh(world: &World, cx: i32, cz: i32) -> MeshData {
    let mut mesh = MeshData::default();
    let base_x = cx * CHUNK_X as i32;
    let base_z = cz * CHUNK_Z as i32;

    for y in 0..CHUNK_Y as i32 {
        for z in 0..CHUNK_Z as i32 {
            for x in 0..CHUNK_X as i32 {
                let wx = base_x + x;
                let wz = base_z + z;
                let block = world.block_at(wx, y, wz);
                if block == Block::Air {
                    continue;
                }
                for (corners, face) in FACE_CORNERS {
                    let [ox, oy, oz] = face.offset();
                    let neighbor = world.block_at(wx + ox, y + oy, wz + oz);
                    let visible = if block == Block::Water {
                        !neighbor.is_opaque() && neighbor != Block::Water
                    } else {
                        !neighbor.is_opaque()
                    };
                    if !visible {
                        continue;
                    }
                    let (u0, v0, cell) = tile_uv(block.tile(face));
                    let base = mesh.vertices.len() as u32;
                    for (i, c) in corners.iter().enumerate() {
                        mesh.vertices.push(Vertex {
                            pos: [wx as f32 + c[0], y as f32 + c[1], wz as f32 + c[2]],
                            uv: [u0 + UV_CORNERS[i][0] * cell, v0 + UV_CORNERS[i][1] * cell],
                            shade: face.shade(),
                        });
                    }
                    mesh.indices
                        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
    }
    mesh
}
