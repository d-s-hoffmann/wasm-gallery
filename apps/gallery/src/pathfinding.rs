//! Path-finding algorithms as step machines. Each call to `step` does a
//! small, fixed amount of work, so a scene can show the search as it goes.
//!
//! Plain Rust with no browser API.

mod bfs;

pub use bfs::Bfs;

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
