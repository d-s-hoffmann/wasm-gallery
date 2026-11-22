//! Path-finding algorithms as step machines. Each call to `step` does a
//! small, fixed amount of work, so a scene can show the search as it goes.
//!
//! Plain Rust with no browser API.

mod bfs;

pub use bfs::Bfs;

use crate::tiling::Tiling;

/// A search from a start cell, run one step at a time.
pub trait Search {
    /// Expands one cell. `cost` gives the cost of entering a cell, as a
    /// factor on the length of the step there, or `None` for a cell that
    /// can't be entered, such as a wall.
    fn step(&mut self, tiling: &dyn Tiling, cost: &dyn Fn(usize) -> Option<f64>) -> Status;

    fn mark(&self, cell: usize) -> Mark;

    /// Returns the cells of the path found, from the goal back to the
    /// start, or nothing if the goal hasn't been reached.
    fn path(&self) -> Vec<usize>;
}

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

/// Follows `from`, the cell each cell was reached from, back from `goal`
/// to the start, which is reached from itself.
fn trace(goal: Option<usize>, from: impl Fn(usize) -> Option<usize>) -> Vec<usize> {
    let mut path = Vec::new();
    let Some(mut cell) = goal else {
        return path;
    };
    while let Some(prev) = from(cell) {
        path.push(cell);
        if prev == cell {
            break; // the start
        }
        cell = prev;
    }
    path
}
