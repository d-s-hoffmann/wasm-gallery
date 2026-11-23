use std::time::Duration;

use super::noise::{self, Rng};
use super::palette::Rgba;
use super::{Frame, Scene, View};
use crate::pathfinding::{Bfs, Dijkstra, Mark, Search, Status};
use crate::tiling::{Square, Tiling};

/// Cells across the shorter side of the view; the longer side gets as
/// many as fit. Even, like every side; see [`CellMap::fit`].
const CELLS: usize = 48;
/// Side of a cell. At zoom 1, the view's shorter side spans scene
/// coordinates -1 to 1, so the board fills the view.
const SIDE: f64 = 2.0 / CELLS as f64;
/// Share of cells that are walls.
const WALLS: f64 = 0.3;
/// Rough width of a wall blob, and of the ragged detail on its edges,
/// in cells.
const BLOB: f64 = 9.0;
const DETAIL: f64 = 3.0;
/// Number of terrain bands, the cost of entering a floor cell of each
/// band, as a factor on the step's length, and the rough width of a
/// patch of one band, in cells.
const BANDS: usize = 4;
const COSTS: [f64; BANDS] = [1.0, 2.0, 4.0, 8.0];
const PATCH: f64 = 6.0;
/// Cells expanded per update: 240 a second at 120 updates a second.
const SPEED: usize = 2;
/// How long a finished round stays on screen.
const HOLD: Duration = Duration::from_secs(2);

const OUTSIDE: Rgba = [0x30, 0x30, 0x30, 0xff];
// Walls and both ends are black on either ground. The ends sit clear of
// walls, so they are the only lone black cells.
const WALL: Rgba = [0x00, 0x00, 0x00, 0xff];
const ENDS: Rgba = [0x00, 0x00, 0x00, 0xff];

/// The colours of one ground.
struct Palette {
    /// Open floor of each terrain band, cheapest first, before and after
    /// the search expanded it.
    floor: [Rgba; BANDS],
    visited: [Rgba; BANDS],
    /// Cells the search is about to expand.
    frontier: Rgba,
    path: Rgba,
}

/// Black and white: the path black, open floor white, and the search in
/// greys, with the cells it is about to expand darker.
const MAZE: Palette = Palette {
    floor: [[0xff, 0xff, 0xff, 0xff]; BANDS],
    visited: [[0xd4, 0xd4, 0xd4, 0xff]; BANDS],
    frontier: [0x90, 0x90, 0x90, 0xff],
    path: [0x00, 0x00, 0x00, 0xff],
};

/// From sand to forest, darker the more a cell costs. Expanded cells are
/// mixed half and half with blue, the frontier is white, the path red.
const TERRAIN: Palette = Palette {
    floor: [
        [0xf2, 0xe8, 0xc9, 0xff],
        [0xb7, 0xd3, 0x8f, 0xff],
        [0x6f, 0x9e, 0x5a, 0xff],
        [0x3e, 0x5f, 0x3a, 0xff],
    ],
    visited: [
        [0x99, 0xac, 0xcc, 0xff],
        [0x7b, 0xa1, 0xaf, 0xff],
        [0x57, 0x87, 0x95, 0xff],
        [0x3f, 0x67, 0x85, 0xff],
    ],
    frontier: [0xff, 0xff, 0xff, 0xff],
    path: [0xd0, 0x20, 0x20, 0xff],
};

/// What the search runs on, and so which algorithm and colours it uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// Even floor, searched breadth first, in black and white. The grey
    /// visited area shows how the grid measures distance: in open floor,
    /// with four neighbours, it grows as a diamond.
    Maze,
    /// Floor in bands that cost more or less to cross, searched with
    /// Dijkstra's algorithm. The visited area bulges into cheap ground.
    Terrain,
}

impl Ground {
    fn palette(self) -> &'static Palette {
        match self {
            Ground::Maze => &MAZE,
            Ground::Terrain => &TERRAIN,
        }
    }
}

/// A search across a square grid with blobs of wall, from a random start
/// to a random goal, on the given [`Ground`].
pub struct PathSearch {
    round: Round,
    /// Which cell each pixel shows. It only changes with the frame size or
    /// the view, so it lives here and outlasts rounds.
    cells: CellMap,
    /// Colour of each cell this frame, then [`OUTSIDE`] for pixels that
    /// show no cell. Reused between frames so they don't allocate.
    colors: Vec<Rgba>,
}

impl PathSearch {
    pub fn new(ground: Ground) -> Self {
        Self {
            round: Round::new(ground, 0, board(1.0)),
            cells: CellMap::default(),
            colors: Vec::new(),
        }
    }
}

impl Scene for PathSearch {
    fn name(&self) -> &'static str {
        match self.round.ground {
            Ground::Maze => "Maze (BFS)",
            Ground::Terrain => "Terrain (Dijkstra)",
        }
    }

    fn fit(&mut self, aspect: f64) {
        // A new shape needs a new board; the round restarts on it.
        let tiling = board(aspect);
        if tiling != self.round.tiling {
            self.round = Round::new(self.round.ground, self.round.number, tiling);
        }
    }

    fn update(&mut self, step: Duration) {
        if self.round.update(step) {
            let r = &self.round;
            self.round = Round::new(r.ground, r.number + 1, r.tiling);
        }
    }

    fn render(&mut self, dst: &mut Frame, view: &View) {
        let round = &self.round;
        self.cells
            .fit(round.tiling, dst.width(), dst.height(), view);

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

/// Returns the board that fills a view of the given aspect ratio: CELLS
/// across the shorter side, and as many as fit, rounded down to an even
/// number, across the longer one.
fn board(aspect: f64) -> Square {
    let long = |ratio: f64| ((CELLS as f64 * ratio / 2.0) as usize * 2).max(CELLS);
    if aspect >= 1.0 {
        Square::new(long(aspect), CELLS, SIDE)
    } else {
        Square::new(CELLS, long(1.0 / aspect), SIDE)
    }
}

/// One search on one board, from its first step to the end of its hold.
struct Round {
    ground: Ground,
    number: u64,
    tiling: Square,
    start: usize,
    goal: usize,
    walls: Vec<bool>,
    /// Terrain band of each cell, an index into COSTS; all 0 on a maze.
    bands: Vec<usize>,
    search: Box<dyn Search>,
    /// Cells on the path found, once there is one.
    path: Vec<bool>,
    /// Time left to show the finished round; `None` while searching.
    hold: Option<Duration>,
}

impl Round {
    /// Returns round number `number` on `tiling`, before its first step.
    /// Its board comes from a generator seeded with `number`, so every
    /// round is reproducible.
    fn new(ground: Ground, number: u64, tiling: Square) -> Self {
        let mut rng = Rng(number);
        // Walls and ends come first, so both grounds share them for the
        // same round number and the two searches can be compared.
        let walls = grow_walls(tiling, &mut rng);
        let (start, goal) = ends(tiling, &walls, &mut rng);
        let cells = tiling.cell_count();
        let (bands, search): (Vec<usize>, Box<dyn Search>) = match ground {
            Ground::Maze => (vec![0; cells], Box::new(Bfs::new(cells, start, Some(goal)))),
            Ground::Terrain => (
                terrain(tiling, &mut rng),
                Box::new(Dijkstra::new(cells, start, Some(goal))),
            ),
        };
        Self {
            search,
            path: vec![false; cells],
            ground,
            number,
            tiling,
            start,
            goal,
            walls,
            bands,
            hold: None,
        }
    }

    /// Advances the round by `step`; returns true once it is over.
    fn update(&mut self, step: Duration) -> bool {
        if let Some(remaining) = self.hold {
            match remaining.checked_sub(step) {
                Some(rest) if !rest.is_zero() => self.hold = Some(rest),
                _ => return true,
            }
            return false;
        }
        for _ in 0..SPEED {
            // The closure borrows only self.walls and self.bands, so
            // self.search can be borrowed mutably at the same time.
            let cost = |c: usize| (!self.walls[c]).then(|| COSTS[self.bands[c]]);
            match self.search.step(&self.tiling, &cost) {
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
        let palette = self.ground.palette();
        if cell == self.start || cell == self.goal {
            return ENDS;
        }
        if self.path[cell] {
            return palette.path;
        }
        if self.walls[cell] {
            return WALL;
        }
        let band = self.bands[cell];
        match self.search.mark(cell) {
            Mark::Unseen => palette.floor[band],
            Mark::Frontier => palette.frontier,
            Mark::Done => palette.visited[band],
        }
    }
}

/// Returns which cells of `tiling` are walls: the highest WALLS share of
/// smooth noise, so walls come in blobs.
fn grow_walls(tiling: Square, rng: &mut Rng) -> Vec<bool> {
    let (cols, rows) = tiling.size();
    let blobs = noise::smooth(cols, rows, BLOB, rng);
    let detail = noise::smooth(cols, rows, DETAIL, rng);
    let height: Vec<f64> = blobs
        .iter()
        .zip(&detail)
        .map(|(b, d)| 0.75 * b + 0.25 * d)
        .collect();

    // The cut-off is a quantile, so the share of walls is exact
    // whatever the noise looks like.
    let cut = quantile(&height, 1.0 - WALLS);
    let walls: Vec<bool> = height.iter().map(|&h| h >= cut).collect();

    // Clear lone specks, walls with at most one wall next to them, so a
    // lone black cell is always one of the ends.
    let mut next = Vec::new();
    (0..walls.len())
        .map(|c| walls[c] && wall_neighbours(tiling, &walls, c, &mut next) >= 2)
        .collect()
}

/// Returns the terrain band of each cell of `tiling`: smooth noise cut at
/// its quantiles, so each band covers the same share of the board.
fn terrain(tiling: Square, rng: &mut Rng) -> Vec<usize> {
    let (cols, rows) = tiling.size();
    let height = noise::smooth(cols, rows, PATCH, rng);
    let cuts: Vec<f64> = (1..BANDS)
        .map(|i| quantile(&height, i as f64 / BANDS as f64))
        .collect();
    height
        .iter()
        .map(|&h| cuts.iter().filter(|&&cut| h >= cut).count())
        .collect()
}

/// Returns the value that a `share` (0 to 1) of `values` lie below.
fn quantile(values: &[f64], share: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let i = (share * sorted.len() as f64) as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// Returns how many neighbours of `cell` are walls. `scratch` is reused
/// for the neighbour list.
fn wall_neighbours(
    tiling: Square,
    walls: &[bool],
    cell: usize,
    scratch: &mut Vec<(usize, f64)>,
) -> usize {
    scratch.clear();
    tiling.neighbours(cell, scratch);
    scratch.iter().filter(|&&(c, _)| walls[c]).count()
}

/// Picks a random start in the open, and a random goal in the open in the
/// farther half of what the start can reach.
fn ends(tiling: Square, walls: &[bool], rng: &mut Rng) -> (usize, usize) {
    let cells = tiling.cell_count();
    // Ends sit in the open, with no wall next to them, so they stand out
    // from the walls although both are black.
    let mut next = Vec::new();
    let mut clear: Vec<bool> = (0..cells)
        .map(|c| !walls[c] && wall_neighbours(tiling, walls, c, &mut next) == 0)
        .collect();
    if !clear.contains(&true) {
        // No such cell: settle for any floor cell.
        clear = walls.iter().map(|&w| !w).collect();
    }
    let floor = walls.iter().filter(|&&w| !w).count();
    let open: Vec<usize> = (0..cells).filter(|&c| clear[c]).collect();
    let mut best = (open[0], open[0], 0);
    // A start walled into a small pocket has no interesting goal, so try
    // a few starts and keep the one that reaches the most.
    for _ in 0..8 {
        let start = open[rng.below(open.len())];
        let mut flood = Bfs::new(cells, start, None);
        while flood.step(&tiling, &|c| (!walls[c]).then_some(1.0)) == Status::Searching {}

        let reached: Vec<(usize, u32)> = (0..cells)
            .filter_map(|c| flood.distance(c).map(|d| (c, d)))
            .collect();
        let far = reached.iter().map(|&(_, d)| d).max().unwrap_or(0);
        let goals: Vec<usize> = reached
            .iter()
            .filter(|&&(c, d)| clear[c] && d >= far.div_ceil(2))
            .map(|&(c, _)| c)
            .collect();
        let goal = goals.get(rng.below(goals.len().max(1))).copied();
        let goal = goal.unwrap_or(start);
        if reached.len() > best.2 {
            best = (start, goal, reached.len());
        }
        if reached.len() * 2 >= floor {
            break;
        }
    }
    (best.0, best.1)
}

/// For each pixel of a frame, the cell it shows, or `cell_count()` if it
/// shows none.
#[derive(Default)]
struct CellMap {
    /// What the map was built for: board, frame width, height and view.
    built_for: Option<(Square, usize, usize, View)>,
    cells: Vec<u32>,
}

impl CellMap {
    /// Rebuilds the map if the board, the frame size or the view changed
    /// since the last call.
    fn fit(&mut self, tiling: Square, width: usize, height: usize, view: &View) {
        let key = Some((tiling, width, height, *view));
        if self.built_for == key {
            return;
        }
        self.built_for = key;

        // Snapping gives every cell the same whole number of pixels, so
        // no wall is drawn thinner, thicker or not at all because of
        // where pixel edges happen to fall. Changing the resolution then
        // changes only sharpness. It needs cell edges at multiples of
        // SIDE from the origin, which holds because both sides are even.
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
