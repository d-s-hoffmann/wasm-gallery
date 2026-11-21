//! Browser client of the Rust gallery.
//!
//! This file only wires modules together: scenes come from [`scene`], the
//! fixed-step timing from [`clock`], and everything that talks to the
//! browser lives in [`ui`]. [`tiling`] and [`pathfinding`] are the boards
//! and algorithms of the path-finding scene.

mod clock;
mod pathfinding;
mod scene;
mod tiling;
mod ui;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use clock::Clock;
use scene::{Julia, Lissajous, PathSearch, Plasma, Scene, View};
use wasm_bindgen::prelude::*;

/// Length of one update step: 120 updates per second.
const STEP: Duration = Duration::from_nanos(1_000_000_000 / 120);

/// Starting pixel budget: the most pixels a frame may have. A bigger
/// screen gets more pixels up to this budget, then bigger pixels. About
/// 2.5 times the old 320×200 view. The menu changes it.
const MAX_PIXELS: usize = 160_000;

/// The most time the loop catches up on after a slow frame or a hidden tab.
const MAX_LAG: Duration = Duration::from_millis(250);

/// Everything the menu controls. It is small and `Copy`, so a [`Cell`]
/// can share it between the menu's callbacks and the loop.
#[derive(Clone, Copy)]
struct Settings {
    /// Index of the shown scene.
    scene: usize,
    max_pixels: usize,
    view: View,
}

/// Changes the shared settings through `f`.
fn change(settings: &Cell<Settings>, f: impl FnOnce(&mut Settings)) {
    let mut s = settings.get();
    f(&mut s);
    settings.set(s);
}

/// Runs once, as soon as the module is instantiated.
#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let mut canvas = ui::Canvas::new("stage")?;

    let mut scenes: Vec<Box<dyn Scene>> = vec![
        Box::new(Plasma::default()),
        Box::new(Julia::default()),
        Box::new(Lissajous::default()),
        Box::new(PathSearch::default()),
    ];
    let settings = Rc::new(Cell::new(Settings {
        scene: 0,
        max_pixels: MAX_PIXELS,
        view: View::default(),
    }));

    let names: Vec<_> = scenes.iter().map(|s| s.name()).collect();
    let s = Rc::clone(&settings);
    ui::buttons("controls", &names, move |i| change(&s, |x| x.scene = i))?;

    // Both sliders move in powers of two, so each notch is the same factor.
    let s = Rc::clone(&settings);
    let pixels = ui::Slider {
        label: "Pixels",
        min: 12.0, // 4k
        max: 22.0, // 4M, about a 4K screen
        step: 0.01,
        value: (MAX_PIXELS as f64).log2(),
        show: |v| match v.exp2() {
            p if p < 1e6 => format!("{:.0}k", p / 1e3),
            p => format!("{:.1}M", p / 1e6),
        },
    };
    ui::slider("controls", pixels, move |v| {
        change(&s, |x| x.max_pixels = v.exp2() as usize);
    })?;

    let s = Rc::clone(&settings);
    let zoom = ui::Slider {
        label: "Zoom",
        min: -2.0, // ¼×
        max: 4.0,  // 16×
        step: 0.01,
        value: 0.0,
        show: |v| format!("{:.2}×", v.exp2()),
    };
    ui::slider("controls", zoom, move |v| {
        change(&s, |x| x.view.zoom = v.exp2());
    })?;

    let mut clock = Clock::new(STEP, MAX_LAG);
    ui::animate(move |now| {
        let s = settings.get();
        let scene = &mut scenes[s.scene];
        for _ in 0..clock.advance(now) {
            scene.update(STEP);
        }
        canvas.fit(s.max_pixels);
        scene.render(canvas.frame_mut(), &s.view);
        canvas.present()
    })
}
