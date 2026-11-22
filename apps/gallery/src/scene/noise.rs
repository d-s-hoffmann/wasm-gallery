//! Seeded randomness for scenes: a small generator and smooth noise. The
//! same seed always gives the same numbers, so frames stay reproducible.

/// SplitMix64, a small pseudo-random number generator.
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Returns a number in [0, 1).
    pub fn unit(&mut self) -> f64 {
        // The top 53 bits: all an f64 can hold exactly.
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    /// Returns a number in 0..n; n must not be 0.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// Returns smooth noise for a grid of `cols` × `rows` cells: one value in
/// [0, 1) per cell, row by row.
///
/// Random values sit on a coarser lattice, one every `scale` cells, and
/// each cell blends the four lattice values around it. Neighbouring cells
/// get similar values, so a threshold cuts the noise into blobs about
/// `scale` cells wide instead of speckles.
pub fn smooth(cols: usize, rows: usize, scale: f64, rng: &mut Rng) -> Vec<f64> {
    let lattice_cols = (cols as f64 / scale) as usize + 2;
    let lattice_rows = (rows as f64 / scale) as usize + 2;
    let lattice: Vec<f64> = (0..lattice_cols * lattice_rows)
        .map(|_| rng.unit())
        .collect();
    let at = |x: usize, y: usize| lattice[y * lattice_cols + x];

    let mut out = Vec::with_capacity(cols * rows);
    for y in 0..rows {
        let (iy, ty) = split(y, scale);
        for x in 0..cols {
            let (ix, tx) = split(x, scale);
            let top = lerp(at(ix, iy), at(ix + 1, iy), tx);
            let bottom = lerp(at(ix, iy + 1), at(ix + 1, iy + 1), tx);
            out.push(lerp(top, bottom, ty));
        }
    }
    out
}

/// Splits cell position `i` into the lattice point before it and how far
/// past it the cell is, eased so the blend has no visible creases.
fn split(i: usize, scale: f64) -> (usize, f64) {
    let f = i as f64 / scale;
    let t = f.fract();
    (f as usize, t * t * (3.0 - 2.0 * t))
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
