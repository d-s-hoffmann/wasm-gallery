use std::time::Duration;

use super::palette::RAINBOW;
use super::{Frame, Scene};

/// An animated plasma made of overlapping sine waves.
#[derive(Default)]
pub struct Plasma {
    /// Time since the animation started: the sum of all update steps.
    t: Duration,
}

impl Scene for Plasma {
    fn name(&self) -> &'static str {
        "Plasma"
    }

    fn update(&mut self, step: Duration) {
        self.t += step;
    }

    fn render(&mut self, dst: &mut Frame) {
        let s = self.t.as_secs_f64();
        // Integer halves first, as in the Go version.
        let cx = (dst.width() / 2) as f64;
        let cy = (dst.height() / 2) as f64;
        for (y, row) in dst.rows_mut() {
            let fy = y as f64;
            // View the row as whole pixels: &mut [[u8; 4]]. The second
            // half of the split is the leftover bytes, always empty here.
            let (pixels, _) = row.as_chunks_mut::<4>();
            for (x, px) in pixels.iter_mut().enumerate() {
                let fx = x as f64;
                let v = (fx / 18.0 + s).sin()
                    + (fy / 9.0 - s * 1.4).sin()
                    + ((fx + fy) / 26.0 + s * 0.8).sin()
                    + ((fx - cx).hypot(fy - cy) / 14.0 - s).sin();
                // v is in [-4, 4]. `as usize` saturates: negatives and NaN
                // become 0, so only the upper end needs clamping.
                let idx = (((v + 4.0) * 32.0) as usize).min(RAINBOW.len() - 1);
                *px = RAINBOW[idx];
            }
        }
    }
}
