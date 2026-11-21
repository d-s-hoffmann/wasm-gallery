use std::time::Duration;

use super::palette::{RAINBOW, Rgba};
use super::{Frame, Scene};

/// Escape-time iteration limit.
const MAX_ITER: u32 = 60;

/// Colour of points that never escape.
const INSIDE: Rgba = [12, 9, 36, 255];

/// A Julia set whose constant slowly orbits the origin.
#[derive(Default)]
pub struct Julia {
    /// Time since the animation started: the sum of all update steps.
    t: Duration,
}

impl Scene for Julia {
    fn name(&self) -> &'static str {
        "Julia set"
    }

    fn update(&mut self, step: Duration) {
        self.t += step;
    }

    fn render(&mut self, dst: &mut Frame) {
        let (sin, cos) = (self.t.as_secs_f64() * 0.15).sin_cos();
        let (cr, ci) = (cos * 0.7885, sin * 0.7885);

        let (w, h) = (dst.width() as f64, dst.height() as f64);
        // Integer halves first, as in the Go version.
        let half_w = (dst.width() / 2) as f64;
        let half_h = (dst.height() / 2) as f64;
        for (y, row) in dst.rows_mut() {
            let zy = (y as f64 - half_h) / (h / 2.6);
            let (pixels, _) = row.as_chunks_mut::<4>();
            for (x, px) in pixels.iter_mut().enumerate() {
                let zx = (x as f64 - half_w) / (w / 3.2);
                *px = color(escape(zx, zy, cr, ci));
            }
        }
    }
}

/// Iterates z → z² + c from z = (zx, zy). If |z| exceeds 2, returns the
/// step count and the final |z|²; returns `None` if it never does.
fn escape(mut zx: f64, mut zy: f64, cr: f64, ci: f64) -> Option<(u32, f64)> {
    for n in 0..MAX_ITER {
        let (zx2, zy2) = (zx * zx, zy * zy);
        if zx2 + zy2 > 4.0 {
            return Some((n, zx2 + zy2));
        }
        zy = 2.0 * zx * zy + ci;
        zx = zx2 - zy2 + cr;
    }
    None
}

/// Maps an escape result to a colour, using the smooth (normalised)
/// iteration count to avoid banding.
fn color(escaped: Option<(u32, f64)>) -> Rgba {
    let Some((n, abs2)) = escaped else {
        return INSIDE;
    };
    let mu = (f64::from(n) + 1.0 - abs2.sqrt().ln().log2()).max(0.0);
    RAINBOW[(mu * 16.0) as usize % RAINBOW.len()]
}
