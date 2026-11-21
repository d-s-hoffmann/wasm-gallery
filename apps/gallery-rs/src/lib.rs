//! Browser client of the Rust gallery.
//!
//! For now it draws a single still frame of the Plasma scene onto the
//! `#stage` canvas. Scenes live in [`scene`] and know nothing about the
//! browser; only this file talks to it.

mod scene;

use std::time::Duration;

use scene::{Frame, Plasma, Scene};
use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

/// Runs once, as soon as the module is instantiated.
#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("stage")
        .ok_or("no canvas with id \"stage\"")?
        .dyn_into()?;
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("canvas has no 2d context")?
        .dyn_into()?;

    let (w, h) = (canvas.width(), canvas.height());
    let mut frame = Frame::new(w as usize, h as usize);
    let scene = Plasma;
    scene.render(&mut frame, Duration::ZERO);

    // ImageData copies the bytes out of WASM memory into a new JS array.
    let img = ImageData::new_with_u8_clamped_array_and_sh(Clamped(frame.pix()), w, h)?;
    ctx.put_image_data(&img, 0.0, 0.0)
}
