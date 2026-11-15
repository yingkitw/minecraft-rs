//! minecraft-rs: a Minecraft-style voxel sandbox in Rust.
//!
//! The library half holds all CPU-side game logic (world, physics, raycast,
//! camera) so it can be unit-tested without a GPU. `src/main.rs` wires it to
//! winit/wgpu for the playable binary.

pub mod block;
pub mod game;
pub mod noise;
pub mod render;
pub mod world;
