use std::time::Duration;

use super::palette::RAINBOW;
use super::{Frame, Scene, View};

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

    fn render(&mut self, dst: &mut Frame, view: &View) {
        let s = self.t.as_secs_f64();
        let map = view.mapping(dst.width(), dst.height());
        for (y, row) in dst.rows_mut() {
            // View the row as whole pixels: &mut [[u8; 4]]. The second
            // half of the split is the leftover bytes, always empty here.
            let (pixels, _) = row.as_chunks_mut::<4>();
            for (x, px) in pixels.iter_mut().enumerate() {
                let (u, v) = map.to_scene(x, y);
                let v = (u * 5.5 + s).sin()
                    + (v * 11.0 - s * 1.4).sin()
                    + ((u + v) * 3.8 + s * 0.8).sin()
                    + (u.hypot(v) * 7.0 - s).sin();
                // v is in [-4, 4]. `as usize` saturates: negatives and NaN
                // become 0, so only the upper end needs clamping.
                let idx = (((v + 4.0) * 32.0) as usize).min(RAINBOW.len() - 1);
                *px = RAINBOW[idx];
            }
        }
    }
}
