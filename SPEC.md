# Spec

## Blocks

| Id | Name | Solid | Opaque | Notes |
|----|------|-------|--------|-------|
| 0 | Air | no | no | |
| 1 | Grass | yes | yes | different top/side/bottom tiles |
| 2 | Dirt | yes | yes | |
| 3 | Stone | yes | yes | |
| 4 | Sand | yes | yes | |
| 5 | Wood | yes | yes | bark sides, rings on top/bottom |
| 6 | Leaves | yes | yes | opaque ("fast graphics") |
| 7 | Water | no | no | rendered opaque; no collision or flow |
| 8 | Bedrock | yes | yes | cannot be broken |
| 9 | Planks | yes | yes | |

## World

- Chunk: 16 × 64 × 16 voxels, flat `[y][z][x]` u8 array.
- World: `HashMap<(i32,i32), Chunk>`; streams in a 6-chunk radius around the player.
- `block_at` → `Air` outside loaded chunks or y ∉ [0,64).

## Generation (seeded, deterministic)

- Heightmap: `height = 16 + fbm(x·0.012, z·0.012, 4 octaves) · 22`, clamped to chunk.
- Strata: bedrock (y=0) → stone → dirt/sand (4 layers) → grass/sand surface.
- Sea level = 24: below-sea surfaces get sand; air below sea level becomes water.
- Trees: 0.6% per interior column on grass above sea level; trunk 4 high,
  leaf blob radius 2 (top layer radius 1). Interior only (local x,z ∈ [2,14]).

## Meshing

- One quad per face where the face-adjacent block is non-opaque; for water the
  neighbor must also not be water.
- Vertex: `pos: f32×3, uv: f32×2 (tile atlas), shade: f32`.
- Face shades: top 1.0, bottom 0.5, north/south 0.8, east/west 0.65.

## Player

- AABB 0.6 × 1.8 × 0.6, eye at +1.62. Walk 5.5 m/s, fly 12 m/s.
- Gravity 26 m/s² (clamp −50), jump velocity 8.5.
- Collision: axis-sequential move-then-clamp; safe-net teleport above world if y < −16.

## Interaction

- Raycast: DDA, max reach 6 blocks from the eye.
- Break: left click → block becomes Air (Bedrock is immune).
- Place: right click → selected palette block goes into `hit + normal` cell,
  only if the cell is Air/Water and does not intersect the player AABB.
- Palette (keys 1–9): grass, dirt, stone, sand, wood, leaves, water, planks, bedrock.

## Rendering

- wgpu, surface via `get_default_config`, `AutoVsync`.
- Atlas: 64×64 RGBA, 4×4 grid of 16×16 tiles, generated per-pixel from the seed.
- Fog: linear mix to sky color (0.55, 0.75, 1.0) from 48 to 110 blocks.
- Clear color = sky; Depth32Float; back-face culling, CCW front faces.
