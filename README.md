# minecraft-rs

A Minecraft-style voxel sandbox written in Rust — chunked procedural terrain,
first-person controls, block breaking and placing — rendered with `wgpu`.

No engine, no binary assets: terrain comes from seeded value-noise fbm, and the
block texture atlas is generated per-pixel at startup.

## Quick start

```sh
cargo run --release
```

Requires Rust 1.85+ (edition 2024) and a GPU supported by wgpu
(Metal on macOS, Vulkan/DX12 elsewhere).

## Controls

| Input | Action |
|---|---|
| WASD | Move |
| Mouse | Look (click once to capture the cursor) |
| Space | Jump / fly up |
| Left Shift | Fly down |
| F | Toggle fly mode |
| Left click | Break block |
| Right click | Place selected block |
| 1–9 | Select block (grass, dirt, stone, sand, wood, leaves, water, planks, bedrock) |
| Esc | Release cursor |

## Features

- Infinite-ish terrain: chunks (16×64×16) stream in around the player
- Heightfield generation with strata (bedrock/stone/dirt/grass/sand), sea level, trees
- Face-culled chunk meshes with baked directional shading and distance fog
- AABB player collision, gravity, jumping, and a fly mode
- Voxel DDA raycast for block picking
- Procedurally generated texture atlas — no asset files

## Tests

```sh
cargo test
```

15 integration tests cover noise determinism, terrain structure, world block
access, dirty-chunk tracking, meshing (including face culling), raycasting,
and player collision — all CPU-side, no GPU needed.

## Layout

| Path | Domain |
|---|---|
| `src/block.rs` | Block types, face properties |
| `src/noise.rs` | Seeded value noise / fbm |
| `src/world/` | Chunk storage, terrain generation, meshing, world map |
| `src/game/` | Player physics, raycast, camera |
| `src/render/` | wgpu renderer + procedural atlas |
| `src/main.rs` | winit event loop and game wiring |

See `ARCHITECTURE.md` for data flow and `SPEC.md` for behavior details.
