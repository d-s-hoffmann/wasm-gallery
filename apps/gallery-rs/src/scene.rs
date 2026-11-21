//! Animated scenes that render into a [`Frame`].
//!
//! Nothing in this module may use a browser API (`web-sys`,
//! `wasm-bindgen`), so scenes can be rendered and tested natively.

use std::time::Duration;

mod palette;
mod plasma;

pub use plasma::Plasma;

/// An RGBA image, four bytes per pixel, rows top to bottom.
pub struct Frame {
    width: usize,
    height: usize,
    pix: Vec<u8>,
}

impl Frame {
    /// Returns a transparent black frame of the given size.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pix: vec![0; width * height * 4],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// All pixel bytes, in RGBA order.
    pub fn pix(&self) -> &[u8] {
        &self.pix
    }

    /// Yields each row's index and its pixel bytes, for writing.
    pub fn rows_mut(&mut self) -> impl Iterator<Item = (usize, &mut [u8])> {
        // max(1): chunks_exact_mut panics on 0, and a 0-wide frame has no
        // bytes anyway, so it yields no rows.
        self.pix
            .chunks_exact_mut((self.width * 4).max(1))
            .enumerate()
    }
}

/// A Scene draws one animation frame at a time.
///
/// `render` must write every pixel of `dst` and must produce the same
/// pixels for the same size and time. The size of `dst` may change
/// between calls.
pub trait Scene {
    /// A short human-readable label, e.g. for a menu entry.
    #[expect(dead_code, reason = "read by the scene menu, a later step")]
    fn name(&self) -> &'static str;

    /// Draws the frame at time `t`, measured from the animation start.
    fn render(&self, dst: &mut Frame, t: Duration);
}
