# Agent Notes

## Build & verify

- `cargo build` — must succeed (edition 2024, Rust ≥ 1.85)
- `cargo test` — must stay green; all tests are CPU-only (`tests/integration.rs`)
- `cargo clippy` — keep warning-free
- Smoke run: `cargo run --release` opens a window; a 10 s run without panic
  validates GPU init + shaders (nothing is logged on success)

## Conventions

- `world` and `game` modules must never import `wgpu`/`winit` — GPU code lives
  only in `render` so tests need no device.
- `gen` is a reserved keyword in edition 2024: the generation module is named
  `terrain`.
- Terrain/atlas randomness comes from `noise::hash01` — do not add `rand`.
- wgpu is pinned to the 30.x API: `Option<...>` fields in pipeline/buffer
  descriptors, `CurrentSurfaceTexture` enum (not `SurfaceError`),
  `queue.present(frame)`, `immediate_size` (not `push_constant_ranges`).
- glam 0.34 has no `look_to_rh` — use `game::camera::look_to`.
- After world/block/mesh changes: update `SPEC.md`; after structure changes:
  `ARCHITECTURE.md`; keep `TODO.md` current.

## Adding a block

1. Variant in `block.rs` (`is_solid`/`is_opaque`/`tile`).
2. Tile painter arm in `render/atlas.rs` (`px`) at the same tile index.
3. Palette entry in `main.rs` if placeable.
