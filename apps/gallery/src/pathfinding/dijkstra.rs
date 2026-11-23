use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{Mark, Search, Status, trace};
use crate::tiling::Tiling;

/// Dijkstra's algorithm. It expands cells in order of the cost of the
/// cheapest way to them from the start, so the path it finds is the
/// cheapest. Where every cell costs the same, it spreads like
/// breadth-first search.
pub struct Dijkstra {
    /// The cell to stop at; `None` explores everything reachable.
    goal: Option<usize>,
    /// Reached cells by cost so far, cheapest on top. A cell is pushed
    /// again each time a cheaper way to it turns up; the stale entries
    /// are skipped when they come up.
    heap: BinaryHeap<Entry>,
    marks: Vec<Mark>,
    /// For each reached cell: the cost of the cheapest way to it found so
    /// far and the cell it was reached from. The start is reached from
    /// itself.
    reached: Vec<Option<(f64, usize)>>,
    /// Neighbour buffer, reused between steps so they don't allocate.
    scratch: Vec<(usize, f64)>,
}

impl Dijkstra {
    /// Starts a search from `start` to `goal` on a tiling of `cells` cells.
    /// Without a goal, it runs until it has reached every cell it can.
    pub fn new(cells: usize, start: usize, goal: Option<usize>) -> Self {
        let mut marks = vec![Mark::Unseen; cells];
        let mut reached = vec![None; cells];
        marks[start] = Mark::Frontier;
        reached[start] = Some((0.0, start));
        Self {
            goal,
            heap: BinaryHeap::from([Entry {
                cost: 0.0,
                cell: start,
            }]),
            marks,
            reached,
            scratch: Vec::new(),
        }
    }
}

impl Search for Dijkstra {
    /// Expands the cheapest frontier cell. A step costs its length times
    /// the cost of the cell it enters.
    fn step(&mut self, tiling: &dyn Tiling, cost: &dyn Fn(usize) -> Option<f64>) -> Status {
        let cell = loop {
            let Some(Entry { cell, .. }) = self.heap.pop() else {
                return Status::Unreachable;
            };
            if self.marks[cell] != Mark::Done {
                break cell;
            }
        };
        self.marks[cell] = Mark::Done;
        if Some(cell) == self.goal {
            return Status::Found;
        }
        let here = self.reached[cell].map_or(0.0, |(c, _)| c);
        self.scratch.clear();
        tiling.neighbours(cell, &mut self.scratch);
        for &(next, length) in &self.scratch {
            let Some(enter) = cost(next) else {
                continue; // a wall
            };
            let total = here + length * enter;
            let cheaper = self.reached[next].is_none_or(|(c, _)| total < c);
            if self.marks[next] != Mark::Done && cheaper {
                self.marks[next] = Mark::Frontier;
                self.reached[next] = Some((total, cell));
                self.heap.push(Entry {
                    cost: total,
                    cell: next,
                });
            }
        }
        Status::Searching
    }

    fn mark(&self, cell: usize) -> Mark {
        self.marks[cell]
    }

    fn path(&self) -> Vec<usize> {
        trace(self.goal, |c| self.reached[c].map(|(_, from)| from))
    }
}

/// A cell waiting in the heap, with the cost of the way to it.
///
/// `BinaryHeap` keeps the greatest entry on top, so entries compare in
/// reverse: the cheapest is the greatest. Ties go to the lower cell
/// number, so the search order is reproducible.
struct Entry {
    cost: f64,
    cell: usize,
}

// An f64 can't be ordered on its own, because NaN is neither less nor
// greater than anything. total_cmp gives every f64 a place, so Entry can
// promise a total order, which BinaryHeap needs.
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then(other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Entry {}
