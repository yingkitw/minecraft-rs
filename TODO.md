# TODO

## Done

- [x] Chunked voxel world (16×64×16) with streaming around the player
- [x] Seeded deterministic terrain: heightmap strata, sea level, trees
- [x] Face-culled chunk meshing with baked directional shading
- [x] Procedural texture atlas (no asset files)
- [x] FPS controls: WASD, mouse-look, jump, gravity, fly toggle
- [x] AABB collision + voxel DDA raycast; block break/place with palette
- [x] Distance fog + crosshair; resize and surface-loss recovery
- [x] Integration tests (15) — gen, world, mesh, raycast, physics

## Next

- [ ] Greedy meshing (merge coplanar faces) — big vertex-count win on flat terrain
- [ ] Chunk unload: drop meshes/loads beyond render radius to bound memory
- [ ] World save/load (per-chunk region files or sqlite)
- [ ] Day/night cycle: animated sky color + directional light in shader
- [ ] Transparent water pass (separate alpha-blended pipeline, back-to-front)
- [ ] Swimming (water buoyancy in `Player::step`)
- [ ] Selected-block HUD instead of window-title readout (needs text/glyph pass)
- [ ] Hotbar/UI overlay drawn with the ui pipeline
- [ ] Block highlight wireframe on the raycast target
- [ ] Caves: carve with 3D noise or worm tunnels
- [ ] Ores and varied stone strata
- [ ] Simple AO (per-vertex ambient occlusion in the mesher)

## Brainstorming (competitive with other voxel engines)

- Multiplayer: server-authoritative world, chunk diffs over WebSocket/quinn
- Infinite height chunks (column segments) like modern MC's 384-block world
- Block entities / tickable blocks (furnaces, redstone-lite signals)
- Entities & mobs with the same AABB physics
- RLE-compressed chunk storage to cut memory ~4×
- Frustum culling per chunk (trivial win: skip `draw_indexed` off-screen)
- CLI flags: `--seed`, `--render-dist`, `--window-size`

## Known limitations

- Trees never cross chunk borders (interior-only placement)
- Water is opaque and doesn't flow
- No collision between player and placed blocks mid-place is checked only at
  place time — a piston-like feature would need continuous checks
- Initial mesh build of the 13×13 spawn area blocks the first frame briefly
