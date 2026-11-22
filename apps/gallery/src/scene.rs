//! Animated scenes that render into a [`Frame`].
//!
//! Nothing in this module may use a browser API (`web-sys`,
//! `wasm-bindgen`), so scenes can be rendered and tested natively.

use std::time::Duration;

mod julia;
mod lissajous;
mod noise;
mod palette;
mod path;
mod plasma;
mod view;

pub use julia::Julia;
pub use lissajous::Lissajous;
pub use path::{Ground, PathSearch};
pub use plasma::Plasma;
pub use view::View;

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

    /// Changes the size of the frame. It keeps its allocation and only
    /// grows it when needed, so switching between sizes stops allocating.
    /// Pixel contents are left over from before; scenes redraw them all.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.pix.resize(width * height * 4, 0);
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

    /// All pixels, row by row, for writing.
    pub fn pixels_mut(&mut self) -> &mut [[u8; 4]] {
        self.pix.as_chunks_mut::<4>().0
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
/// the same frame size and view, so frames are reproducible.
pub trait Scene {
    /// A short human-readable label, e.g. for a menu entry.
    fn name(&self) -> &'static str;

    /// Tells the scene the shape of the view, width divided by height,
    /// before each frame's updates. It depends on the window's shape, not
    /// on the resolution. A scene may use it to lay itself out, e.g. to
    /// fill the screen; most ignore it.
    fn fit(&mut self, _aspect: f64) {}

    /// Advances the animation by `step`, which the caller keeps constant.
    fn update(&mut self, step: Duration);

    /// Draws the current state into `dst` as seen through `view`, writing
    /// every pixel. Positions come from [`View::mapping`], never from pixel
    /// indices directly. The size of `dst` may change between calls. It
    /// takes `&mut self` so scenes can keep scratch buffers between frames.
    fn render(&mut self, dst: &mut Frame, view: &View);
}
