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

/// A Scene is an animation with state, driven in two phases.
///
/// [`update`](Scene::update) advances the state by one fixed time step;
/// conditional logic belongs there. [`render`](Scene::render) draws the
/// current state and must not change what later frames look like. After
/// the same sequence of updates, a scene must produce the same pixels for
/// the same frame size, so frames are reproducible.
pub trait Scene {
    /// A short human-readable label, e.g. for a menu entry.
    #[expect(dead_code, reason = "read by the scene menu, a later step")]
    fn name(&self) -> &'static str;

    /// Advances the animation by `step`, which the caller keeps constant.
    #[expect(dead_code, reason = "called by the animation loop, next commit")]
    fn update(&mut self, step: Duration);

    /// Draws the current state into `dst`, writing every pixel. The size
    /// of `dst` may change between calls. It takes `&mut self` so scenes
    /// can keep scratch buffers between frames.
    fn render(&mut self, dst: &mut Frame);
}
