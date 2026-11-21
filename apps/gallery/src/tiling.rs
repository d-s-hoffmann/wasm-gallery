//! Tilings: ways to cut part of the plane into cells, the boards that path
//! finding runs on.
//!
//! Plain Rust with no browser API. Positions are scene coordinates, the
//! same ones a scene's `View` maps to pixels, so a tiling looks the same
//! at any resolution.

mod square;

pub use square::Square;

/// Part of the plane cut into cells numbered `0..cell_count()`.
pub trait Tiling {
    fn cell_count(&self) -> usize;

    /// Appends each neighbour of `cell` to `out`, with the cost of the
    /// step there, in scene units.
    fn neighbours(&self, cell: usize, out: &mut Vec<(usize, f64)>);

    /// Returns the cell that contains the scene point `p`, or `None` if
    /// `p` lies outside the tiling.
    fn cell_at(&self, p: (f64, f64)) -> Option<usize>;
}
