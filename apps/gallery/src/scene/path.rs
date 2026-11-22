use std::time::Duration;

use super::palette::{RAINBOW, Rgba};
use super::{Frame, Scene, View};
use crate::pathfinding::{Bfs, Mark, Status};
use crate::tiling::{Square, Tiling};

/// Cells per side of the grid. It must be even; see [`CellMap::fit`].
const CELLS: usize = 48;
/// Side of a cell. The grid spans scene coordinates -1 to 1 on both axes.
const SIDE: f64 = 2.0 / CELLS as f64;
/// Share of cells that are walls.
const WALLS: f64 = 0.3;
/// Cells expanded per update: 240 a second at 120 updates a second.
const SPEED: usize = 2;
/// How long a finished round stays on screen.
const HOLD: Duration = Duration::from_secs(2);

const OUTSIDE: Rgba = [0x0b, 0x09, 0x20, 0xff];
const FLOOR: Rgba = [0x1c, 0x14, 0x40, 0xff];
const WALL: Rgba = [0x4b, 0x3a, 0x9e, 0xff];
const FRONTIER: Rgba = [0xcd, 0xc6, 0xff, 0xff];
const PATH: Rgba = [0xff, 0xff, 0xff, 0xff];
const ENDS: Rgba = [0xff, 0x40, 0x80, 0xff];

/// Breadth-first search across a square grid with random walls.
///
/// Visited cells are coloured by their distance from the start, so the
/// shape of the frontier shows how the grid measures distance: on a
/// square grid with four neighbours, it grows as a diamond.
pub struct PathSearch {
    round: Round,
    /// Which cell each pixel shows. It only changes with the frame size or
    /// the view, so it lives here and outlasts rounds.
    cells: CellMap,
    /// Colour of each cell this frame, then [`OUTSIDE`] for pixels that
    /// show no cell. Reused between frames so they don't allocate.
    colors: Vec<Rgba>,
}

impl Default for PathSearch {
    fn default() -> Self {
        Self {
            round: Round::new(0),
            cells: CellMap::default(),
            colors: Vec::new(),
        }
    }
}

impl Scene for PathSearch {
    fn name(&self) -> &'static str {
        "Path finding"
    }

    fn update(&mut self, step: Duration) {
        if self.round.update(step) {
            self.round = Round::new(self.round.number + 1);
        }
    }

    fn render(&mut self, dst: &mut Frame, view: &View) {
        let round = &self.round;
        self.cells
            .fit(&round.tiling, dst.width(), dst.height(), view);

        // Colour each cell once, then each pixel is a lookup: a few
        // thousand cells cost far less than a million pixels.
        let cells = round.tiling.cell_count();
        self.colors.clear();
        self.colors.extend((0..cells).map(|c| round.color(c)));
        self.colors.push(OUTSIDE);

        for (px, &cell) in dst.pixels_mut().iter_mut().zip(&self.cells.cells) {
            *px = self.colors[cell as usize];
        }
    }
}

/// One search on one board, from its first step to the end of its hold.
struct Round {
    number: u64,
    tiling: Square,
    start: usize,
    goal: usize,
    walls: Vec<bool>,
    search: Bfs,
    /// Cells on the path found, once there is one.
    path: Vec<bool>,
    /// Time left to show the finished round; `None` while searching.
    hold: Option<Duration>,
}

impl Round {
    /// Returns round number `number`, before its first step. Its walls
    /// come from a generator seeded with `number`, so every round is
    /// reproducible.
    fn new(number: u64) -> Self {
        let tiling = Square::new(CELLS, CELLS, SIDE);
        let start = tiling.cell_at((-0.9, 0.0)).expect("start is on the grid");
        let goal = tiling.cell_at((0.9, 0.0)).expect("goal is on the grid");
        let cells = tiling.cell_count();

        let mut rng = Rng(number);
        let mut walls: Vec<bool> = (0..cells).map(|_| rng.chance(WALLS)).collect();
        walls[start] = false;
        walls[goal] = false;

        Self {
            search: Bfs::new(cells, start, goal),
            path: vec![false; cells],
            number,
            tiling,
            start,
            goal,
            walls,
            hold: None,
        }
    }

    /// Advances the round by `step`; returns true once it is over.
    fn update(&mut self, step: Duration) -> bool {
        if let Some(left) = self.hold {
            match left.checked_sub(step) {
                Some(left) if !left.is_zero() => self.hold = Some(left),
                _ => return true,
            }
            return false;
        }
        for _ in 0..SPEED {
            // The closure borrows only self.walls, so self.search can be
            // borrowed mutably at the same time.
            match self.search.step(&self.tiling, |c| !self.walls[c]) {
                Status::Searching => {}
                Status::Found => {
                    for cell in self.search.path() {
                        self.path[cell] = true;
                    }
                    self.hold = Some(HOLD);
                    break;
                }
                Status::Unreachable => {
                    self.hold = Some(HOLD);
                    break;
                }
            }
        }
        false
    }

    fn color(&self, cell: usize) -> Rgba {
        if cell == self.start || cell == self.goal {
            return ENDS;
        }
        if self.path[cell] {
            return PATH;
        }
        if self.walls[cell] {
            return WALL;
        }
        match self.search.mark(cell) {
            Mark::Unseen => FLOOR,
            Mark::Frontier => FRONTIER,
            Mark::Done => {
                let d = self.search.distance(cell).unwrap_or(0) as usize;
                RAINBOW[d * 4 % RAINBOW.len()]
            }
        }
    }
}

/// For each pixel of a frame, the cell it shows, or `cell_count()` if it
/// shows none.
#[derive(Default)]
struct CellMap {
    /// What the map was built for: frame width, height and view.
    built_for: Option<(usize, usize, View)>,
    cells: Vec<u32>,
}

impl CellMap {
    /// Rebuilds the map if the frame size or the view changed since the
    /// last call.
    fn fit(&mut self, tiling: &Square, width: usize, height: usize, view: &View) {
        let key = Some((width, height, *view));
        if self.built_for == key {
            return;
        }
        self.built_for = key;

        // Snapping gives every cell the same whole number of pixels, so
        // no wall is drawn thinner, thicker or not at all because of
        // where pixel edges happen to fall. Changing the resolution then
        // changes only sharpness. It needs cell edges at multiples of
        // SIDE from the origin, which holds because CELLS is even.
        let map = view.mapping(width, height).snapped(SIDE);
        let outside = tiling.cell_count() as u32;
        self.cells.clear();
        for y in 0..height {
            for x in 0..width {
                let cell = tiling.cell_at(map.to_scene(x, y));
                self.cells.push(cell.map_or(outside, |c| c as u32));
            }
        }
    }
}

/// SplitMix64, a small pseudo-random number generator. The same seed
/// always gives the same numbers.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Returns true with probability `p`.
    fn chance(&mut self, p: f64) -> bool {
        // The top 53 bits as a number in [0, 1): all an f64 can hold exactly.
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64) < p
    }
}
