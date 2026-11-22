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

use clock::{Clock, Governor};
use scene::{Julia, Lissajous, PathSearch, Plasma, Scene, View};
use wasm_bindgen::prelude::*;

/// Length of one update step: 120 updates per second.
const STEP: Duration = Duration::from_nanos(1_000_000_000 / 120);

/// Starting pixel budget: the most pixels a frame may have. A bigger
/// screen gets more pixels up to the budget, then bigger pixels. The
/// budget then follows how long frames take, up to the menu's limit.
const START_PIXELS: usize = 160_000;

/// Fewest pixels the budget goes down to.
const MIN_PIXELS: usize = 4_096;

/// Starting limit for the budget, set in the menu.
const PIXEL_LIMIT: usize = 1 << 20;

/// How long computing a frame should take. At 60 Hz a frame lasts about
/// 16.7 ms; the rest is left for the browser to show it.
const TARGET_WORK: Duration = Duration::from_millis(8);

/// How often the menu's frame statistics are refreshed.
const STATS_EVERY: Duration = Duration::from_millis(250);

/// The most time the loop catches up on after a slow frame or a hidden tab.
const MAX_LAG: Duration = Duration::from_millis(250);

/// Everything the menu controls. It is small and `Copy`, so a [`Cell`]
/// can share it between the menu's callbacks and the loop.
#[derive(Clone, Copy)]
struct Settings {
    /// Index of the shown scene.
    scene: usize,
    /// Most pixels the budget may grow to.
    pixel_limit: usize,
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
        pixel_limit: PIXEL_LIMIT,
        view: View::default(),
    }));

    let names: Vec<_> = scenes.iter().map(|s| s.name()).collect();
    let s = Rc::clone(&settings);
    ui::buttons("controls", &names, move |i| change(&s, |x| x.scene = i))?;

    // Both sliders move in powers of two, so each notch is the same factor.
    let s = Rc::clone(&settings);
    let pixels = ui::Slider {
        label: "Max pixels",
        min: 12.0, // 4k
        max: 22.0, // 4M, about a 4K screen
        step: 0.01,
        value: (PIXEL_LIMIT as f64).log2(),
        show: |v| count(v.exp2()),
    };
    ui::slider("controls", pixels, move |v| {
        change(&s, |x| x.pixel_limit = v.exp2() as usize);
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

    let stats = ui::Text::new("stats")?;
    let mut next_stats = Duration::ZERO;
    let mut clock = Clock::new(STEP, MAX_LAG);
    let mut governor = Governor::new(TARGET_WORK, MIN_PIXELS, START_PIXELS);
    ui::animate(move |tick| {
        let s = settings.get();
        let scene = &mut scenes[s.scene];
        for _ in 0..clock.advance(tick.now) {
            scene.update(STEP);
        }

        // The canvas still has the previous frame's size, which is what
        // tick.work was measured with.
        let budget = governor.adjust(tick.work, canvas.pixels(), s.pixel_limit);
        if tick.now >= next_stats {
            let ms = tick.work.as_secs_f64() * 1e3;
            let px = count(canvas.pixels() as f64);
            stats.set(&format!("{ms:.1} ms per frame · {px} pixels"));
            next_stats = tick.now + STATS_EVERY;
        }

        canvas.fit(budget);
        scene.render(canvas.frame_mut(), &s.view);
        canvas.present()
    })
}

/// Formats a pixel count briefly, e.g. "160k" or "1.0M".
fn count(pixels: f64) -> String {
    match pixels {
        p if p < 1e6 => format!("{:.0}k", p / 1e3),
        p => format!("{:.1}M", p / 1e6),
    }
}
