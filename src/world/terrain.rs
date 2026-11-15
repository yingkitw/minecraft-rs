//! Procedural terrain generation: heightfield + strata + water + trees.

use crate::block::Block;
use crate::noise::{fbm, hash01};
use crate::world::chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z, Chunk};

pub const SEA_LEVEL: i32 = 24;
const MIN_H: f32 = 16.0;
const MAX_H: f32 = 38.0;

/// Terrain surface height at a world column (world-space x/z).
pub fn height_at(seed: u32, wx: i32, wz: i32) -> i32 {
    let n = fbm(seed, wx as f32 * 0.012, wz as f32 * 0.012, 4);
    (MIN_H + n * (MAX_H - MIN_H)) as i32
}

/// Fill `chunk` (chunk coords cx, cz) with terrain for `seed`.
pub fn generate(seed: u32, cx: i32, cz: i32, chunk: &mut Chunk) {
    for lz in 0..CHUNK_Z {
        for lx in 0..CHUNK_X {
            let wx = cx * CHUNK_X as i32 + lx as i32;
            let wz = cz * CHUNK_Z as i32 + lz as i32;
            let h = height_at(seed, wx, wz).min(CHUNK_Y as i32 - 1);
            let beach = h <= SEA_LEVEL + 1;

            for ly in 0..CHUNK_Y as i32 {
                let block = if ly == 0 {
                    Block::Bedrock
                } else if ly >= h {
                    if ly < SEA_LEVEL { Block::Water } else { Block::Air }
                } else if ly == h - 1 {
                    if beach { Block::Sand } else { Block::Grass }
                } else if ly >= h - 4 {
                    if beach { Block::Sand } else { Block::Dirt }
                } else {
                    Block::Stone
                };
                if block != Block::Air {
                    chunk.set(lx, ly as usize, lz, block);
                }
            }
        }
    }
    place_trees(seed, cx, cz, chunk);
}

/// Sparse trees. Restricted to chunk interiors (leaves radius 2) so no tree
/// ever crosses a chunk border — keeps generation a pure per-chunk function.
fn place_trees(seed: u32, cx: i32, cz: i32, chunk: &mut Chunk) {
    const TRUNK: i32 = 4;
    for lz in 2..CHUNK_Z - 2 {
        for lx in 2..CHUNK_X - 2 {
            let wx = cx * CHUNK_X as i32 + lx as i32;
            let wz = cz * CHUNK_Z as i32 + lz as i32;
            let h = height_at(seed, wx, wz);
            if h <= SEA_LEVEL + 1 || h + TRUNK + 3 >= CHUNK_Y as i32 {
                continue;
            }
            if hash01(seed ^ 0x07e5_5eed, wx, wz) > 0.006 {
                continue;
            }
            for dy in 1..=TRUNK {
                chunk.set(lx, (h + dy) as usize, lz, Block::Wood);
            }
            let top = h + TRUNK;
            for dy in [top - 1, top, top + 1] {
                let r = if dy == top + 1 { 1 } else { 2 };
                for dx in -r..=r {
                    for dz in -r..=r {
                        if dx == 0 && dz == 0 && dy <= top {
                            continue;
                        }
                        let (tx, ty, tz) = (lx as i32 + dx, dy, lz as i32 + dz);
                        if chunk.get(tx as usize, ty as usize, tz as usize) == Block::Air {
                            chunk.set(tx as usize, ty as usize, tz as usize, Block::Leaves);
                        }
                    }
                }
            }
        }
    }
}
