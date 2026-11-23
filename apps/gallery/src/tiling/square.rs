use super::Tiling;

/// A grid of `cols` × `rows` square cells, centred on the origin and
/// numbered row by row from the top left. A cell's neighbours are the
/// four cells it shares an edge with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Square {
    cols: usize,
    rows: usize,
    /// Length of a cell's side, in scene units.
    side: f64,
}

impl Square {
    pub fn new(cols: usize, rows: usize, side: f64) -> Self {
        Self { cols, rows, side }
    }

    /// Returns the number of columns and rows.
    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }
}

impl Tiling for Square {
    fn cell_count(&self) -> usize {
        self.cols * self.rows
    }

    fn neighbours(&self, cell: usize, out: &mut Vec<(usize, f64)>) {
        let (x, y) = (cell % self.cols, cell / self.cols);
        let mut push = |x: usize, y: usize| out.push((y * self.cols + x, self.side));
        // Right, down, left, up: a fixed order keeps searches reproducible.
        if x + 1 < self.cols {
            push(x + 1, y);
        }
        if y + 1 < self.rows {
            push(x, y + 1);
        }
        if x > 0 {
            push(x - 1, y);
        }
        if y > 0 {
            push(x, y - 1);
        }
    }

    fn cell_at(&self, (px, py): (f64, f64)) -> Option<usize> {
        // Position in cells, measured from the top-left corner.
        let fx = px / self.side + self.cols as f64 / 2.0;
        let fy = py / self.side + self.rows as f64 / 2.0;
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (x, y) = (fx as usize, fy as usize);
        (x < self.cols && y < self.rows).then(|| y * self.cols + x)
    }
}
