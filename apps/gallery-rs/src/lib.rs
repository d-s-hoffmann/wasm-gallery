//! Browser client of the Rust gallery.
//!
//! This file only wires modules together: scenes come from [`scene`], the
//! fixed-step timing from [`clock`], and everything that talks to the
//! browser lives in [`ui`].

mod clock;
mod scene;
mod ui;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use clock::Clock;
use scene::{Julia, Lissajous, Plasma, Scene};
use wasm_bindgen::prelude::*;

/// Length of one update step: 120 updates per second.
const STEP: Duration = Duration::from_nanos(1_000_000_000 / 120);

/// The most time the loop catches up on after a slow frame or a hidden tab.
const MAX_LAG: Duration = Duration::from_millis(250);

/// Runs once, as soon as the module is instantiated.
#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let mut canvas = ui::Canvas::new("stage")?;

    let mut scenes: Vec<Box<dyn Scene>> = vec![
        Box::new(Plasma::default()),
        Box::new(Julia::default()),
        Box::new(Lissajous::default()),
    ];
    // Index of the shown scene, shared by the buttons and the loop.
    let current = Rc::new(Cell::new(0));

    let names: Vec<_> = scenes.iter().map(|s| s.name()).collect();
    let select = Rc::clone(&current);
    ui::buttons("controls", &names, move |i| select.set(i))?;

    let mut clock = Clock::new(STEP, MAX_LAG);
    ui::animate(move |now| {
        let scene = &mut scenes[current.get()];
        for _ in 0..clock.advance(now) {
            scene.update(STEP);
        }
        scene.render(canvas.frame_mut());
        canvas.present()
    })
}
