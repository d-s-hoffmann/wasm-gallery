//! Path-finding algorithms as step machines. Each call to `step` does a
//! small, fixed amount of work, so a scene can show the search as it goes.
//!
//! Plain Rust with no browser API.

use std::collections::VecDeque;

use crate::tiling::Tiling;

/// How far a search has got with one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Not reached yet.
    Unseen,
    /// Reached, waiting to be expanded.
    Frontier,
    /// Expanded: all its neighbours have been looked at.
    Done,
}

/// What a search step found out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Searching,
    Found,
    /// The frontier ran out before the goal was reached, or, without a
    /// goal, every reachable cell has been expanded.
    Unreachable,
}

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

    /// Expands the oldest frontier cell. It only enters cells for which
    /// `open` returns true.
    pub fn step(&mut self, tiling: &impl Tiling, open: impl Fn(usize) -> bool) -> Status {
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
        for &(next, _cost) in &self.scratch {
            if self.marks[next] == Mark::Unseen && open(next) {
                self.marks[next] = Mark::Frontier;
                self.reached[next] = Some((dist + 1, cell));
                self.queue.push_back(next);
            }
        }
        Status::Searching
    }

    pub fn mark(&self, cell: usize) -> Mark {
        self.marks[cell]
    }

    /// Returns the number of steps from the start to `cell`, if reached.
    pub fn distance(&self, cell: usize) -> Option<u32> {
        self.reached[cell].map(|(d, _)| d)
    }

    /// Returns the cells of the path found, from the goal back to the
    /// start, or nothing if the goal hasn't been reached.
    pub fn path(&self) -> Vec<usize> {
        let mut path = Vec::new();
        let Some(mut cell) = self.goal else {
            return path;
        };
        while let Some((_, from)) = self.reached[cell] {
            path.push(cell);
            if from == cell {
                break; // the start
            }
            cell = from;
        }
        path
    }
}
