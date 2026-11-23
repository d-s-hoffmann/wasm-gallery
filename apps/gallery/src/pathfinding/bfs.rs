use std::collections::VecDeque;

use super::{Mark, Search, Status, trace};
use crate::tiling::Tiling;

/// Breadth-first search. It expands cells in order of how many steps they
/// are from the start, so the path it finds has the fewest steps. It
/// ignores step costs.
pub struct Bfs {
    /// The cell to stop at; `None` explores everything reachable.
    goal: Option<usize>,
    /// Reached cells waiting to be expanded, oldest first.
    queue: VecDeque<usize>,
    marks: Vec<Mark>,
    /// For each reached cell: its distance in steps and the cell it was
    /// reached from. The start is reached from itself.
    reached: Vec<Option<(u32, usize)>>,
    /// Neighbour buffer, reused between steps so they don't allocate.
    scratch: Vec<(usize, f64)>,
}

impl Bfs {
    /// Starts a search from `start` to `goal` on a tiling of `cells` cells.
    /// Without a goal, it runs until it has reached every cell it can.
    pub fn new(cells: usize, start: usize, goal: Option<usize>) -> Self {
        let mut marks = vec![Mark::Unseen; cells];
        let mut reached = vec![None; cells];
        marks[start] = Mark::Frontier;
        reached[start] = Some((0, start));
        Self {
            goal,
            queue: VecDeque::from([start]),
            marks,
            reached,
            scratch: Vec::new(),
        }
    }

    /// Returns the number of steps from the start to `cell`, if reached.
    pub fn distance(&self, cell: usize) -> Option<u32> {
        self.reached[cell].map(|(d, _)| d)
    }
}

impl Search for Bfs {
    /// Expands the oldest frontier cell. It ignores what entering a cell
    /// costs, only whether it can be entered.
    fn step(&mut self, tiling: &dyn Tiling, cost: &dyn Fn(usize) -> Option<f64>) -> Status {
        let Some(cell) = self.queue.pop_front() else {
            return Status::Unreachable;
        };
        self.marks[cell] = Mark::Done;
        if Some(cell) == self.goal {
            return Status::Found;
        }
        let dist = self.reached[cell].map_or(0, |(d, _)| d);
        self.scratch.clear();
        tiling.neighbours(cell, &mut self.scratch);
        for &(next, _length) in &self.scratch {
            if self.marks[next] == Mark::Unseen && cost(next).is_some() {
                self.marks[next] = Mark::Frontier;
                self.reached[next] = Some((dist + 1, cell));
                self.queue.push_back(next);
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
