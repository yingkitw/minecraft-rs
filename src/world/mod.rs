//! The world domain: chunk map, block access, terrain streaming.

pub mod chunk;
pub mod mesh;
pub mod terrain;

use crate::block::Block;
use crate::world::chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z, Chunk};
use std::collections::{HashMap, HashSet};

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    /// Chunks whose meshes are stale and need rebuilding.
    dirty: HashSet<(i32, i32)>,
    seed: u32,
}

impl World {
    pub fn new(seed: u32) -> World {
        World {
            chunks: HashMap::new(),
            dirty: HashSet::new(),
            seed,
        }
    }

    /// Block at world coords; `Air` for unloaded chunks / out of vertical range.
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        if !(0..CHUNK_Y as i32).contains(&y) {
            return Block::Air;
        }
        let cx = x.div_euclid(CHUNK_X as i32);
        let cz = z.div_euclid(CHUNK_Z as i32);
        match self.chunks.get(&(cx, cz)) {
            Some(c) => c.get(
                x.rem_euclid(CHUNK_X as i32) as usize,
                y as usize,
                z.rem_euclid(CHUNK_Z as i32) as usize,
            ),
            None => Block::Air,
        }
    }

    /// Write a block; returns false if the chunk isn't loaded or y is out of range.
    /// Marks the owning chunk (and border neighbors) dirty for remeshing.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) -> bool {
        if !(0..CHUNK_Y as i32).contains(&y) {
            return false;
        }
        let cx = x.div_euclid(CHUNK_X as i32);
        let cz = z.div_euclid(CHUNK_Z as i32);
        let lx = x.rem_euclid(CHUNK_X as i32) as usize;
        let lz = z.rem_euclid(CHUNK_Z as i32) as usize;
        let Some(chunk) = self.chunks.get_mut(&(cx, cz)) else {
            return false;
        };
        chunk.set(lx, y as usize, lz, block);
        self.mark_dirty(cx, cz, lx, lz);
        true
    }

    fn mark_dirty(&mut self, cx: i32, cz: i32, lx: usize, lz: usize) {
        self.dirty.insert((cx, cz));
        // Edits on a chunk border change the neighbor's visible faces too.
        if lx == 0 {
            self.dirty.insert((cx - 1, cz));
        }
        if lx == CHUNK_X - 1 {
            self.dirty.insert((cx + 1, cz));
        }
        if lz == 0 {
            self.dirty.insert((cx, cz - 1));
        }
        if lz == CHUNK_Z - 1 {
            self.dirty.insert((cx, cz + 1));
        }
    }

    /// Generate any missing chunks within `radius` of the world position.
    /// New chunks and their existing neighbors are marked dirty (need remesh
    /// so shared border faces get culled).
    pub fn ensure_chunks(&mut self, wx: f32, wz: f32, radius: i32) {
        let pcx = (wx as i32).div_euclid(CHUNK_X as i32);
        let pcz = (wz as i32).div_euclid(CHUNK_Z as i32);
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let key = (pcx + dx, pcz + dz);
                if self.chunks.contains_key(&key) {
                    continue;
                }
                let mut chunk = Chunk::empty();
                terrain::generate(self.seed, key.0, key.1, &mut chunk);
                self.chunks.insert(key, chunk);
                self.dirty.insert(key);
                for (nx, nz) in [
                    (key.0 - 1, key.1),
                    (key.0 + 1, key.1),
                    (key.0, key.1 - 1),
                    (key.0, key.1 + 1),
                ] {
                    if self.chunks.contains_key(&(nx, nz)) {
                        self.dirty.insert((nx, nz));
                    }
                }
            }
        }
    }

    /// Chunks needing a (re)mesh: dirty ones plus freshly generated coords.
    pub fn take_dirty(&mut self) -> Vec<(i32, i32)> {
        self.dirty.drain().collect()
    }

    /// Highest solid block y at a world column, if the chunk is loaded.
    pub fn surface_height(&self, wx: i32, wz: i32) -> Option<i32> {
        let cx = wx.div_euclid(CHUNK_X as i32);
        let cz = wz.div_euclid(CHUNK_Z as i32);
        let chunk = self.chunks.get(&(cx, cz))?;
        let lx = wx.rem_euclid(CHUNK_X as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_Z as i32) as usize;
        for y in (0..CHUNK_Y).rev() {
            if chunk.get(lx, y, lz).is_solid() {
                return Some(y as i32);
            }
        }
        None
    }
}
