//! Procedural 4x4-tile texture atlas (64x64 RGBA) — no binary assets.
//! Deterministic per-pixel hashing gives the classic speckled look.

use crate::noise::hash01;

pub const TILE: usize = 16;
pub const COLS: usize = 4;
pub const SIZE: usize = TILE * COLS; // 64

fn jitter(seed: u32, x: usize, y: usize, base: [u8; 3], amt: i32) -> [u8; 4] {
    let n = (hash01(seed, x as i32, y as i32) - 0.5) * 2.0 * amt as f32;
    [
        (base[0] as f32 + n).clamp(0.0, 255.0) as u8,
        (base[1] as f32 + n).clamp(0.0, 255.0) as u8,
        (base[2] as f32 + n).clamp(0.0, 255.0) as u8,
        255,
    ]
}

fn px(seed: u32, tile: u32, x: usize, y: usize) -> [u8; 4] {
    let s = seed.wrapping_add(tile * 7919);
    match tile {
        0 => jitter(s, x, y, [88, 148, 62], 18),                       // grass top
        1 => {
            // grass side: ragged green lip over dirt
            let lip = 3 + (hash01(s, x as i32, 0) * 3.0) as usize;
            if y < lip { jitter(s, x, y, [88, 148, 62], 16) } else { jitter(s, x, y, [134, 96, 67], 14) }
        }
        2 => jitter(s, x, y, [134, 96, 67], 14),                      // dirt
        3 => jitter(s, x, y, [125, 125, 125], 16),                    // stone
        4 => jitter(s, x, y, [219, 207, 163], 12),                    // sand
        5 => {
            // wood bark: vertical streaks
            let streak = hash01(s, x as i32, 77) * 30.0;
            jitter(s, x, y, [(104.0 - streak) as u8, 82, 50], 10)
        }
        6 => {
            // wood rings
            let d = ((x as i32 - 8).abs() + (y as i32 - 8).abs()) % 4;
            let v = 175 - d * 18;
            jitter(s, x, y, [v as u8, (v as f32 * 0.8) as u8, 90], 8)
        }
        7 => {
            // leaves: heavy dark speckle
            let dark = hash01(s, x as i32, y as i32) < 0.35;
            if dark { jitter(s, x, y, [34, 92, 30], 10) } else { jitter(s, x, y, [58, 128, 46], 12) }
        }
        8 => {
            // water: horizontal bands
            let band = ((y / 3) % 2) as i32 * 12;
            jitter(s, x, y, [52, (95 + band) as u8, (205 + band) as u8], 8)
        }
        9 => {
            // bedrock: blotchy high-contrast gray
            let v = 55.0 + hash01(s, (x / 3) as i32, (y / 3) as i32) * 80.0;
            jitter(s, x, y, [v as u8, v as u8, v as u8], 10)
        }
        10 => {
            // planks: seams every 4 rows + staggered edge seams
            let seam = y.is_multiple_of(4)
                || (x == 15 && (y / 4).is_multiple_of(2))
                || (x == 7 && !(y / 4).is_multiple_of(2));
            if seam {
                jitter(s, x, y, [110, 85, 50], 6)
            } else {
                jitter(s, x, y, [172, 136, 82], 10)
            }
        }
        _ => jitter(s, x, y, [255, 0, 255], 0),
    }
}

/// Build the atlas pixel buffer (RGBA8, SIZE x SIZE).
pub fn build_atlas(seed: u32) -> Vec<u8> {
    let mut out = vec![0u8; SIZE * SIZE * 4];
    for tile in 0..(COLS * COLS) as u32 {
        let ox = (tile as usize % COLS) * TILE;
        let oy = (tile as usize / COLS) * TILE;
        for y in 0..TILE {
            for x in 0..TILE {
                let p = px(seed, tile, x, y);
                let i = ((oy + y) * SIZE + ox + x) * 4;
                out[i..i + 4].copy_from_slice(&p);
            }
        }
    }
    out
}
