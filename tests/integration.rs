//! Integration tests for the CPU-side game logic (no GPU needed).

use minecraft_rs::block::Block;
use minecraft_rs::game::player::{MoveInput, Player};
use minecraft_rs::game::raycast::raycast;
use minecraft_rs::noise::{fbm, hash01};
use minecraft_rs::world::chunk::{CHUNK_X, CHUNK_Y, CHUNK_Z, Chunk};
use minecraft_rs::world::mesh::build_mesh;
use minecraft_rs::world::{World, terrain};

const SEED: u32 = 42;

fn world_with(radius: i32) -> World {
    let mut w = World::new(SEED);
    w.ensure_chunks(0.0, 0.0, radius);
    w
}

// --- noise ---

#[test]
fn noise_is_deterministic_and_bounded() {
    for i in -20..20 {
        let a = fbm(SEED, i as f32 * 3.7, i as f32 * -1.3, 4);
        let b = fbm(SEED, i as f32 * 3.7, i as f32 * -1.3, 4);
        assert_eq!(a, b);
        assert!((0.0..1.0).contains(&a), "fbm out of range: {a}");
        assert!((0.0..1.0).contains(&hash01(SEED, i, i * 7)));
    }
}

#[test]
fn different_seeds_give_different_terrain() {
    let a: Vec<i32> = (0..10).map(|i| terrain::height_at(1, i * 17, i * 29)).collect();
    let b: Vec<i32> = (0..10).map(|i| terrain::height_at(2, i * 17, i * 29)).collect();
    assert_ne!(a, b);
}

// --- terrain generation ---

#[test]
fn generated_terrain_has_expected_structure() {
    let mut w = World::new(SEED);
    w.ensure_chunks(0.0, 0.0, 0); // just chunk (0,0)

    // Bedrock floor.
    for x in 0..CHUNK_X as i32 {
        assert_eq!(w.block_at(x, 0, 0), Block::Bedrock);
    }
    // Every column: solid ground exists, and air above the surface.
    for x in 0..CHUNK_X as i32 {
        for z in 0..CHUNK_Z as i32 {
            let h = w.surface_height(x, z).expect("column must have ground");
            assert!(h > 0 && h < CHUNK_Y as i32 - 1);
            assert!(w.block_at(x, h, z).is_solid());
            let above = w.block_at(x, h + 1, z);
            assert!(matches!(above, Block::Air | Block::Water | Block::Leaves | Block::Wood));
        }
    }
    // Same seed regenerates identical chunks.
    let mut c1 = Chunk::empty();
    let mut c2 = Chunk::empty();
    terrain::generate(SEED, 0, 0, &mut c1);
    terrain::generate(SEED, 0, 0, &mut c2);
    assert_eq!(c1.blocks, c2.blocks);
}

// --- world block access ---

#[test]
fn block_at_works_across_chunk_borders() {
    let w = world_with(1);
    // Coordinates either side of the x=0 border are in different chunks.
    let west = w.block_at(-1, 0, 0);
    let east = w.block_at(0, 0, 0);
    assert_eq!(west, Block::Bedrock);
    assert_eq!(east, Block::Bedrock);
    // Out of range / unloaded -> Air.
    assert_eq!(w.block_at(0, -1, 0), Block::Air);
    assert_eq!(w.block_at(0, CHUNK_Y as i32, 0), Block::Air);
    assert_eq!(w.block_at(9999, 10, 0), Block::Air);
}

#[test]
fn set_block_marks_dirty_and_survives() {
    let mut w = world_with(0);
    let h = w.surface_height(5, 5).unwrap();
    assert!(w.set_block(5, h + 3, 5, Block::Planks));
    assert_eq!(w.block_at(5, h + 3, 5), Block::Planks);
    assert!(!w.take_dirty().is_empty());
    // Setting into an unloaded chunk fails.
    assert!(!w.set_block(9999, 10, 0, Block::Stone));
}

// --- meshing ---

#[test]
fn empty_chunk_meshes_to_nothing() {
    let mut w = World::new(SEED);
    w.chunks.insert((0, 0), Chunk::empty());
    let mesh = build_mesh(&w, 0, 0);
    assert!(mesh.vertices.is_empty() && mesh.indices.is_empty());
}

#[test]
fn single_block_has_six_faces() {
    let mut w = World::new(SEED);
    let mut c = Chunk::empty();
    c.set(8, 30, 8, Block::Stone);
    w.chunks.insert((0, 0), c);
    let mesh = build_mesh(&w, 0, 0);
    assert_eq!(mesh.vertices.len(), 24);
    assert_eq!(mesh.indices.len(), 36);
}

#[test]
fn touching_neighbors_cull_shared_faces() {
    let mut w = World::new(SEED);
    let mut c = Chunk::empty();
    c.set(8, 30, 8, Block::Stone);
    c.set(9, 30, 8, Block::Stone);
    w.chunks.insert((0, 0), c);
    let mesh = build_mesh(&w, 0, 0);
    // Two cubes share one face pair: 12 - 2 = 10 faces = 40 verts / 60 idx.
    assert_eq!(mesh.vertices.len(), 40);
    assert_eq!(mesh.indices.len(), 60);
}

#[test]
fn generated_chunk_produces_nonempty_mesh() {
    let w = world_with(0);
    let mesh = build_mesh(&w, 0, 0);
    assert!(!mesh.vertices.is_empty());
    assert_eq!(mesh.vertices.len() % 4, 0);
    assert_eq!(mesh.indices.len() % 6, 0);
}

// --- raycast ---

#[test]
fn raycast_hits_ground_below() {
    let w = world_with(1);
    let h = w.surface_height(3, 3).unwrap();
    let origin = glam::Vec3::new(3.5, h as f32 + 10.0, 3.5);
    let hit = raycast(&w, origin, glam::Vec3::NEG_Y, 20.0).expect("must hit ground");
    assert_eq!(hit.block.y, h);
    assert_eq!(hit.normal, glam::IVec3::Y);
}

#[test]
fn raycast_misses_through_air() {
    let w = world_with(1);
    let origin = glam::Vec3::new(0.5, 60.0, 0.5);
    assert!(raycast(&w, origin, glam::Vec3::X, 5.0).is_none());
}

#[test]
fn raycast_reports_adjacent_place_cell() {
    let w = world_with(1);
    let h = w.surface_height(6, 6).unwrap();
    let origin = glam::Vec3::new(6.5, h as f32 + 8.0, 6.5);
    let hit = raycast(&w, origin, glam::Vec3::NEG_Y, 20.0).unwrap();
    let place = hit.block + hit.normal;
    assert_eq!(place.y, h + 1);
    assert!(matches!(w.block_at(place.x, place.y, place.z), Block::Air | Block::Water));
}

// --- player physics ---

#[test]
fn player_falls_and_lands_on_ground() {
    let w = world_with(1);
    let h = w.surface_height(0, 0).unwrap();
    let mut p = Player::new(glam::Vec3::new(0.5, h as f32 + 5.0, 0.5));
    for _ in 0..600 {
        p.step(&w, MoveInput::default(), 1.0 / 60.0);
        if p.on_ground {
            break;
        }
    }
    assert!(p.on_ground);
    assert!((p.pos.y - (h as f32 + 1.0)).abs() < 0.01);
}

#[test]
fn player_is_blocked_by_walls() {
    let mut w = World::new(SEED);
    // Flat floor at y=0..1, wall at x=5.
    let mut c = Chunk::empty();
    for x in 0..CHUNK_X {
        for z in 0..CHUNK_Z {
            c.set(x, 0, z, Block::Stone);
        }
    }
    for y in 1..5 {
        for z in 0..CHUNK_Z {
            c.set(5, y, z, Block::Stone);
        }
    }
    w.chunks.insert((0, 0), c);
    let mut p = Player::new(glam::Vec3::new(3.5, 1.01, 8.5));
    let input = MoveInput { forward: 0.0, strafe: 0.0, jump: false, down: false };
    for _ in 0..30 {
        p.step(&w, input, 1.0 / 60.0);
    }
    assert!(p.on_ground);
    // Walk straight +x into the wall via yaw facing +x.
    p.yaw = -std::f32::consts::FRAC_PI_2; // fwd = (-sin(-pi/2),0,-cos(-pi/2)) = (1,0,0)
    let walk = MoveInput { forward: 1.0, strafe: 0.0, jump: false, down: false };
    for _ in 0..120 {
        p.step(&w, walk, 1.0 / 60.0);
    }
    assert!(p.pos.x < 5.0, "player tunneled through wall: {}", p.pos.x);
    assert!(p.pos.x > 4.0, "player stopped too early: {}", p.pos.x);
}

#[test]
fn player_cannot_overlap_placed_block() {
    let w = world_with(1);
    let h = w.surface_height(0, 0).unwrap();
    let p = Player::new(glam::Vec3::new(0.5, h as f32 + 1.0, 0.5));
    assert!(p.overlaps_voxel(0, h + 1, 0));
    assert!(!p.overlaps_voxel(0, h + 5, 0));
}
