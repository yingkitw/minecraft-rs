//! Block types and their face/rendering properties.

/// One of the six faces of a cube.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Top,
    Bottom,
    North, // -z
    South, // +z
    East,  // +x
    West,  // -x
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Top,
        Face::Bottom,
        Face::North,
        Face::South,
        Face::East,
        Face::West,
    ];

    /// Unit normal as an integer offset.
    pub fn offset(self) -> [i32; 3] {
        match self {
            Face::Top => [0, 1, 0],
            Face::Bottom => [0, -1, 0],
            Face::North => [0, 0, -1],
            Face::South => [0, 0, 1],
            Face::East => [1, 0, 0],
            Face::West => [-1, 0, 0],
        }
    }

    /// Baked directional shading, Minecraft-style (tops brightest).
    pub fn shade(self) -> f32 {
        match self {
            Face::Top => 1.0,
            Face::Bottom => 0.5,
            Face::North | Face::South => 0.8,
            Face::East | Face::West => 0.65,
        }
    }
}

/// A voxel. Stored in chunks as `u8`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Wood = 5,
    Leaves = 6,
    Water = 7,
    Bedrock = 8,
    Planks = 9,
}

impl Block {
    pub fn from_id(id: u8) -> Block {
        match id {
            1 => Block::Grass,
            2 => Block::Dirt,
            3 => Block::Stone,
            4 => Block::Sand,
            5 => Block::Wood,
            6 => Block::Leaves,
            7 => Block::Water,
            8 => Block::Bedrock,
            9 => Block::Planks,
            _ => Block::Air,
        }
    }

    /// Blocks the player collides with (water is pass-through).
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air | Block::Water)
    }

    /// Blocks that hide neighboring faces ("fast graphics": leaves are opaque).
    pub fn is_opaque(self) -> bool {
        self.is_solid()
    }

    /// Index into the procedural texture atlas (4x4 grid of 16px tiles).
    pub fn tile(self, face: Face) -> u32 {
        use Block::*;
        match (self, face) {
            (Grass, Face::Top) => 0,
            (Grass, Face::Bottom) => 2,
            (Grass, _) => 1,
            (Dirt, _) => 2,
            (Stone, _) => 3,
            (Sand, _) => 4,
            (Wood, Face::Top | Face::Bottom) => 6,
            (Wood, _) => 5,
            (Leaves, _) => 7,
            (Water, _) => 8,
            (Bedrock, _) => 9,
            (Planks, _) => 10,
            (Air, _) => 0,
        }
    }
}
