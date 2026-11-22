use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use wasm_bindgen::prelude::*;

/// The JS function that requestAnimationFrame calls, with a timestamp in
/// milliseconds. Returning `Err` throws it as a JS exception.
type Callback = Closure<dyn FnMut(f64) -> Result<(), JsValue>>;

/// What the loop knows at the start of a frame.
#[derive(Clone, Copy, Debug)]
pub struct Tick {
    /// The browser's timestamp for this frame.
    pub now: Duration,
    /// How long the previous frame's callback took to run.
    pub work: Duration,
}

/// Calls `frame` once per display refresh, via requestAnimationFrame. It
/// returns immediately; the browser drives the loop from then on. If
/// `frame` fails, the loop stops and the error appears in the browser
/// console.
pub fn animate(
    mut frame: impl FnMut(Tick) -> Result<(), JsValue> + 'static,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let performance = window.performance().ok_or("no performance timer")?;

    // The callback schedules itself again, so it needs a handle to itself.
    // `slot` holds it; the callback keeps a second Rc to the slot. This
    // cycle is never freed, which keeps the loop alive for the page's life.
    let slot: Rc<RefCell<Option<Callback>>> = Rc::default();
    let tick = {
        let slot = Rc::clone(&slot);
        let window = window.clone();
        let mut work = Duration::ZERO;
        Closure::new(move |ms: f64| {
            let started = performance.now();
            frame(Tick {
                now: millis(ms),
                work,
            })?;
            work = millis(performance.now() - started);

            let next = slot.borrow();
            let next = next.as_ref().ok_or("animation callback is gone")?;
            window.request_animation_frame(next.as_ref().unchecked_ref())?;
            Ok(())
        })
    };
    window.request_animation_frame(tick.as_ref().unchecked_ref())?;
    *slot.borrow_mut() = Some(tick);
    Ok(())
}

/// Converts a browser time in milliseconds to a Duration.
fn millis(ms: f64) -> Duration {
    // f64::max ignores NaN, so from_secs_f64 can't panic here.
    Duration::from_secs_f64(ms.max(0.0) / 1000.0)
}
