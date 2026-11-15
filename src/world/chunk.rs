//! Fixed-size voxel storage: 16 x 64 x 16 blocks.

use crate::block::Block;

pub const CHUNK_X: usize = 16;
pub const CHUNK_Y: usize = 64;
pub const CHUNK_Z: usize = 16;
pub const BLOCK_COUNT: usize = CHUNK_X * CHUNK_Y * CHUNK_Z;

#[inline]
pub const fn index(x: usize, y: usize, z: usize) -> usize {
    (y * CHUNK_Z + z) * CHUNK_X + x
}

pub struct Chunk {
    /// Flat `[y][z][x]` block ids.
    pub blocks: Box<[u8; BLOCK_COUNT]>,
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk {
            blocks: Box::new([0; BLOCK_COUNT]),
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        Block::from_id(self.blocks[index(x, y, z)])
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[index(x, y, z)] = block as u8;
    }
}
