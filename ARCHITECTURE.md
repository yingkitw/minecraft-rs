# Architecture

## Domain layout

```
minecraft-rs
├── block          Block id enum + per-face tile/shading rules
├── noise          Deterministic seeded value noise + fbm (pure)
├── world          Voxel data and generation — GPU-free
│   ├── chunk      16×64×16 block array
│   ├── terrain    Heightfield + strata + water + trees (pure fn of seed+cx+cz)
│   ├── mesh       Face-culled mesher -> (vertices, indices)
│   └── (mod)      World: HashMap<(cx,cz), Chunk>, dirty tracking, streaming
├── game           Gameplay rules — GPU-free
│   ├── player     AABB collision, gravity, walk/fly movement
│   ├── raycast    Amanatides–Woo voxel DDA for picking
│   └── camera     wgpu-convention perspective + look-at matrices
├── render         GPU layer (wgpu)
│   ├── atlas      Procedural 4×4-tile texture atlas (RGBA8)
│   └── (mod)      Renderer: surface, pipelines, chunk buffers, crosshair
└── main           winit ApplicationHandler: input, game loop, wiring
```

Design rule: everything under `world` and `game` is CPU-only and unit-testable;
`render` is the only module that touches the GPU.

## Data flow (per frame)

```
winit events ──> App
                 ├─ mouse/keys ──> Player::step(world, input, dt)   [physics]
                 ├─ world.ensure_chunks(player pos, radius)         [gen new]
                 ├─ world.take_dirty()                              [gen+edits]
                 │      └─> mesh::build_mesh(world, cx, cz)          [remesh]
                 │      └─> renderer.upload_chunk(...)               [GPU upload]
                 └─ renderer.render(view_proj, eye)                  [draw]
```

Chunk lifecycle:

- `ensure_chunks` fills missing chunks in a 6-chunk radius; new chunks and
  their existing neighbors are marked *dirty* (borders must remesh).
- `set_block` marks its chunk dirty, plus the up-to-4 neighbor chunks if the
  block sits on a border.
- `take_dirty` drains the set each frame; each dirty chunk is remeshed and
  re-uploaded (empty meshes are dropped).

## Rendering

- One indexed triangle mesh per chunk; a face is emitted only when the
  neighbor block is non-opaque (water additionally skips same-block faces).
- Single `wgpu` pipeline: atlas texture + uniform (mvp, camera pos, fog).
  Per-vertex directional shade gives the classic look without real lighting.
- Crosshair is a second pipeline drawn in the same pass with `Always` depth.
- Depth32Float buffer recreated on resize; `CurrentSurfaceTexture` statuses
  (Outdated/Lost) trigger reconfigure; Timeout/Occluded skip the frame.

## Conventions & invariants

- Coordinates: +x east, +y up, -z forward (right-handed). Chunks are
  `div_euclid`-addressed, so negative coordinates work.
- Chunk Y range is fixed `[0, 64)`; bedrock at y=0, sea level at y=24.
- `World::block_at` returns `Air` for unloaded/out-of-range voxels — meshers
  and physics treat the world edge as empty space.
- Trees only spawn in chunk interiors (leaf radius 2) so generation stays a
  pure per-chunk function — no cross-chunk writes.
- Collision moves one axis at a time, then clamps; `on_ground` is set only by
  a downward y-resolve.
- Everything is deterministic given `SEED` (hash-based noise, hash-based
  atlas pixels) — no `rand`.
