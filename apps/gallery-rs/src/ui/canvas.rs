use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use crate::scene::Frame;

/// Presents [`Frame`]s on an HTML canvas element.
pub struct Canvas {
    ctx: CanvasRenderingContext2d,
    frame: Frame,
}

impl Canvas {
    /// Returns a Canvas for the `<canvas>` element with the given id,
    /// sized to the element's width and height attributes.
    pub fn new(id: &str) -> Result<Self, JsValue> {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("no document")?;
        let canvas: HtmlCanvasElement = document
            .get_element_by_id(id)
            .ok_or_else(|| format!("no canvas with id {id:?}"))?
            .dyn_into()?;
        let ctx = canvas
            .get_context("2d")?
            .ok_or("canvas has no 2d context")?
            .dyn_into()?;
        let frame = Frame::new(canvas.width() as usize, canvas.height() as usize);
        Ok(Self { ctx, frame })
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
