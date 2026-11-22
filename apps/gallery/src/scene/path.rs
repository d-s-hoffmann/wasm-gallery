use std::time::Duration;

use super::palette::{RAINBOW, Rgba};
use super::{Frame, Scene, View};
use crate::pathfinding::{Bfs, Mark, Status};
use crate::tiling::{Square, Tiling};

/// Cells per side of the grid.
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
    tiling: Square,
    round: u64,
    start: usize,
    goal: usize,
    walls: Vec<bool>,
    search: Bfs,
    /// Cells on the path found, once there is one.
    path: Vec<bool>,
    /// Time left to show a finished round; `None` while searching.
    hold: Option<Duration>,
}

impl Default for PathSearch {
    fn default() -> Self {
        Self::round(0)
    }
}

impl PathSearch {
    /// Returns the start of round number `round`. Its walls come from a
    /// generator seeded with `round`, so every round is reproducible.
    fn round(round: u64) -> Self {
        let tiling = Square::new(CELLS, CELLS, SIDE);
        let start = tiling.cell_at((-0.9, 0.0)).expect("start is on the grid");
        let goal = tiling.cell_at((0.9, 0.0)).expect("goal is on the grid");
        let cells = tiling.cell_count();

        let mut rng = Rng(round);
        let mut walls: Vec<bool> = (0..cells).map(|_| rng.chance(WALLS)).collect();
        walls[start] = false;
        walls[goal] = false;

        Self {
            search: Bfs::new(cells, start, goal),
            path: vec![false; cells],
            tiling,
            round,
            start,
            goal,
            walls,
            hold: None,
        }
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

impl Scene for PathSearch {
    fn name(&self) -> &'static str {
        "Path finding"
    }

    fn update(&mut self, step: Duration) {
        if let Some(left) = self.hold {
            match left.checked_sub(step) {
                Some(left) if !left.is_zero() => self.hold = Some(left),
                _ => *self = Self::round(self.round + 1),
            }
            return;
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
                    return;
                }
                Status::Unreachable => {
                    self.hold = Some(HOLD);
                    return;
                }
            }
        }
    }

    fn render(&mut self, dst: &mut Frame, view: &View) {
        let map = view.mapping(dst.width(), dst.height());
        for (y, row) in dst.rows_mut() {
            let (pixels, _) = row.as_chunks_mut::<4>();
            for (x, px) in pixels.iter_mut().enumerate() {
                *px = match self.tiling.cell_at(map.to_scene(x, y)) {
                    Some(cell) => self.color(cell),
                    None => OUTSIDE,
                };
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
