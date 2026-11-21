use std::f64::consts::TAU;
use std::time::Duration;

use super::palette::Rgba;
use super::{Frame, Scene, View};

/// Line segments per curve.
const STEPS: u32 = 1400;
/// Amplitude of the curve, in scene units; 1 would touch the shorter edge.
const AMPLITUDE: f64 = 0.94;
/// Stroke width, in scene units.
const WIDTH: f64 = 0.015;

const BACKGROUND: Rgba = [0x0b, 0x09, 0x20, 0xff];
const STROKE: Rgba = [0x8f, 0x6b, 0xff, 0xff];

/// A 3:4 Lissajous curve with a drifting phase.
#[derive(Default)]
pub struct Lissajous {
    /// Time since the animation started: the sum of all update steps.
    t: Duration,
    /// Stroke coverage per pixel, 0 to 1. Kept between frames so render
    /// doesn't allocate.
    cov: Vec<f32>,
}

impl Scene for Lissajous {
    fn name(&self) -> &'static str {
        "Lissajous"
    }

    fn update(&mut self, step: Duration) {
        self.t += step;
    }

    fn render(&mut self, dst: &mut Frame, view: &View) {
        let (w, h) = (dst.width(), dst.height());
        if w == 0 || h == 0 {
            return;
        }
        // Keeps the allocation; only grows it when the frame gets bigger.
        self.cov.clear();
        self.cov.resize(w * h, 0.0);

        let s = self.t.as_secs_f64();
        let map = view.mapping(w, h);
        let point = |i: u32| {
            let u = TAU * f64::from(i) / f64::from(STEPS);
            map.to_pixel((
                AMPLITUDE * (3.0 * u + s * 0.9).sin(),
                AMPLITUDE * (4.0 * u).sin(),
            ))
        };
        let radius = WIDTH / 2.0 * map.unit();
        let mut p0 = point(0);
        for i in 1..=STEPS {
            let p1 = point(i);
            stroke_segment(&mut self.cov, w, h, p0, p1, radius);
            p0 = p1;
        }

        for ((_, row), cov) in dst.rows_mut().zip(self.cov.chunks_exact(w)) {
            let (pixels, _) = row.as_chunks_mut::<4>();
            for (px, &c) in pixels.iter_mut().zip(cov) {
                *px = mix(BACKGROUND, STROKE, c);
            }
        }
    }
}

/// Rasterises the segment p0–p1 with half-width r into the w×h coverage
/// buffer cov. Coverage ramps linearly over one pixel around the stroke
/// edge. Each pixel keeps its highest coverage, so joints and
/// self-intersections don't get brighter than the stroke itself.
fn stroke_segment(
    cov: &mut [f32],
    w: usize,
    h: usize,
    (x0, y0): (f64, f64),
    (x1, y1): (f64, f64),
    r: f64,
) {
    let pad = r + 0.5;
    // `as usize` saturates, so negative coordinates become 0.
    let min_x = (x0.min(x1) - pad).floor() as usize;
    let max_x = ((x0.max(x1) + pad).ceil() as usize).min(w - 1);
    let min_y = (y0.min(y1) - pad).floor() as usize;
    let max_y = ((y0.max(y1) + pad).ceil() as usize).min(h - 1);

    let (dx, dy) = (x1 - x0, y1 - y0);
    let len2 = dx * dx + dy * dy;
    for py in min_y..=max_y {
        for px in min_x..=max_x {
            // Distance from the pixel centre to the closest point on the segment.
            let (qx, qy) = (px as f64 + 0.5 - x0, py as f64 + 0.5 - y0);
            let k = if len2 > 0.0 {
                ((qx * dx + qy * dy) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let d = (qx - k * dx).hypot(qy - k * dy);
            let c = (pad - d).clamp(0.0, 1.0) as f32;
            let i = py * w + px;
            cov[i] = cov[i].max(c);
        }
    }
}

/// Blends from a to b by k in [0, 1], returning an opaque colour.
fn mix(a: Rgba, b: Rgba, k: f32) -> Rgba {
    if k <= 0.0 {
        return a;
    }
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * k + 0.5) as u8;
    [ch(a[0], b[0]), ch(a[1], b[1]), ch(a[2], b[2]), 255]
}
