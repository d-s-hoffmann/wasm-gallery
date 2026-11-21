use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, Window};

use crate::scene::Frame;

/// Presents [`Frame`]s on an HTML canvas element, at a resolution that
/// follows the element's displayed size.
pub struct Canvas {
    window: Window,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    frame: Frame,
    /// Most pixels a frame may have.
    max_pixels: usize,
}

impl Canvas {
    /// Returns a Canvas for the `<canvas>` element with the given id. Its
    /// frames have at most `max_pixels` pixels.
    pub fn new(id: &str, max_pixels: usize) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas: HtmlCanvasElement = document
            .get_element_by_id(id)
            .ok_or_else(|| format!("no canvas with id {id:?}"))?
            .dyn_into()?;
        let ctx = canvas
            .get_context("2d")?
            .ok_or("canvas has no 2d context")?
            .dyn_into()?;
        let mut c = Self {
            window,
            canvas,
            ctx,
            frame: Frame::new(0, 0),
            max_pixels,
        };
        c.fit();
        Ok(c)
    }

    /// Resizes the frame and the canvas's pixel buffer if the canvas's
    /// displayed size changed, e.g. after a window resize or a phone
    /// rotation. Call it before drawing each frame.
    pub fn fit(&mut self) {
        let css = (
            f64::from(self.canvas.client_width()),
            f64::from(self.canvas.client_height()),
        );
        let (w, h) = render_size(css, self.window.device_pixel_ratio(), self.max_pixels);
        if (w, h) != (self.frame.width(), self.frame.height()) {
            // Setting the buffer size clears the canvas, so only do it on
            // a change. CSS stretches the buffer to the displayed size.
            self.canvas.set_width(w as u32);
            self.canvas.set_height(h as u32);
            self.frame.resize(w, h);
        }
    }

    /// Returns the frame to draw the next picture into.
    pub fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }

    /// Copies the frame onto the canvas.
    pub fn present(&self) -> Result<(), JsValue> {
        let (w, h) = (self.frame.width() as u32, self.frame.height() as u32);
        // ImageData copies the bytes out of WASM memory into a new JS array.
        let img = ImageData::new_with_u8_clamped_array_and_sh(Clamped(self.frame.pix()), w, h)?;
        self.ctx.put_image_data(&img, 0.0, 0.0)
    }
}

/// Returns the render size for a view of `css` (width, height) CSS pixels
/// on a screen with `dpr` device pixels per CSS pixel.
///
/// That is one render pixel per device pixel if it fits in `max_pixels`,
/// else the largest size with the same aspect ratio that does. Never
/// smaller than 1×1, because a canvas can't present an empty image.
fn render_size(css: (f64, f64), dpr: f64, max_pixels: usize) -> (usize, usize) {
    let (w, h) = (css.0 * dpr, css.1 * dpr);
    // min(1.0) never scales up, and also turns a division by an empty
    // area (infinity, or NaN for 0/0) into 1.0.
    let scale = (max_pixels as f64 / (w * h)).sqrt().min(1.0);
    let side = |x: f64| (x * scale).floor().max(1.0) as usize;
    (side(w), side(h))
}
