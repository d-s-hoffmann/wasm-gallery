//! Connects the gallery to the browser: it presents frames on a canvas,
//! drives the animation loop and builds controls. It knows nothing about
//! what is drawn.

mod animate;
mod buttons;
mod canvas;
mod slider;
mod text;

pub use animate::animate;
pub use buttons::buttons;
pub use canvas::Canvas;
pub use slider::{Slider, slider};
pub use text::Text;
