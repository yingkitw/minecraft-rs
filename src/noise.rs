//! Deterministic value noise + fbm for terrain generation.
//! No external RNG: everything hashes (seed, x, z) so worlds are reproducible.

fn hash2(seed: u32, x: i32, z: i32) -> u32 {
    let mut h = seed
        .wrapping_add((x as u32).wrapping_mul(0x27d4_eb2d))
        .wrapping_add((z as u32).wrapping_mul(0x1656_67b1));
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h
}

/// Deterministic float in [0, 1) for a lattice point.
pub fn hash01(seed: u32, x: i32, z: i32) -> f32 {
    (hash2(seed, x, z) >> 8) as f32 / (1u32 << 24) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Bilinear value noise in [0, 1).
pub fn value_noise(seed: u32, x: f32, z: f32) -> f32 {
    let xi = x.floor() as i32;
    let zi = z.floor() as i32;
    let xf = smooth(x - xi as f32);
    let zf = smooth(z - zi as f32);

    let a = hash01(seed, xi, zi);
    let b = hash01(seed, xi + 1, zi);
    let c = hash01(seed, xi, zi + 1);
    let d = hash01(seed, xi + 1, zi + 1);

    let top = a + (b - a) * xf;
    let bot = c + (d - c) * xf;
    top + (bot - top) * zf
}

/// Fractal Brownian motion over `value_noise`, roughly in [0, 1).
pub fn fbm(seed: u32, x: f32, z: f32, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for i in 0..octaves {
        sum += amp * value_noise(seed.wrapping_add(i * 101), x * freq, z * freq);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}
