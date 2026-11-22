use std::time::Duration;

use super::palette::RAINBOW;
use super::{Frame, Scene, View};

/// An animated plasma made of overlapping sine waves.
#[derive(Default)]
pub struct Plasma {
    /// Time since the animation started: the sum of all update steps.
    t: Duration,
    /// Per column and per row: the scene coordinate and the parts of the
    /// waves that depend only on it. Reused between frames.
    cols: Vec<Wave>,
    rows: Vec<Wave>,
}

/// What one column (or row) contributes to every pixel in it.
#[derive(Clone, Copy)]
struct Wave {
    /// Scene x of a column, or scene y of a row.
    at: f64,
    /// The wave along this axis alone.
    own: f64,
    /// Sine and cosine of this axis's part of the diagonal wave.
    sin: f64,
    cos: f64,
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

        // Three of the four waves depend on x alone or y alone, once the
        // diagonal one is split with sin(a + b) = sin a cos b + cos a sin b.
        // So they are computed once per column and row, not per pixel.
        // This relies on pixels lying in straight rows and columns.
        let wave = |at: f64, own: f64, diagonal: f64| {
            let (sin, cos) = diagonal.sin_cos();
            Wave {
                at,
                own: own.sin(),
                sin,
                cos,
            }
        };
        self.cols.clear();
        self.cols.extend((0..dst.width()).map(|x| {
            let u = map.to_scene(x, 0).0;
            wave(u, u * 5.5 + s, u * 3.8 + s * 0.8)
        }));
        self.rows.clear();
        self.rows.extend((0..dst.height()).map(|y| {
            let v = map.to_scene(0, y).1;
            wave(v, v * 11.0 - s * 1.4, v * 3.8)
        }));

        for ((_, row), r) in dst.rows_mut().zip(&self.rows) {
            // View the row as whole pixels: &mut [[u8; 4]]. The second
            // half of the split is the leftover bytes, always empty here.
            let (pixels, _) = row.as_chunks_mut::<4>();
            for (px, c) in pixels.iter_mut().zip(&self.cols) {
                let v = c.own
                    + r.own
                    + (c.sin * r.cos + c.cos * r.sin)
                    + (c.at.hypot(r.at) * 7.0 - s).sin();
                // v is in [-4, 4]. `as usize` saturates: negatives and NaN
                // become 0, so only the upper end needs clamping.
                let idx = (((v + 4.0) * 32.0) as usize).min(RAINBOW.len() - 1);
                *px = RAINBOW[idx];
            }
        }
    }
}
