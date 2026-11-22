//! Timing for the animation loop: fixed update steps, and a pixel budget
//! that follows how long frames take.
//!
//! Plain Rust with no browser API: it only turns measured times into
//! decisions.

use std::time::Duration;

/// Counts how many fixed steps to run for each displayed frame.
///
/// Scenes advance in equal steps, whatever the display's refresh rate, so
/// the same number of steps always gives the same state.
pub struct Clock {
    step: Duration,
    max_lag: Duration,
    /// Timestamp of the previous frame; `None` before the first one.
    last: Option<Duration>,
    /// Elapsed time not yet covered by steps.
    lag: Duration,
}

impl Clock {
    /// Returns a clock that runs steps of length `step` and never catches
    /// up on more than `max_lag` at once.
    pub fn new(step: Duration, max_lag: Duration) -> Self {
        Self {
            step,
            max_lag,
            last: None,
            lag: Duration::ZERO,
        }
    }

    /// Records a frame at time `now` and returns how many steps to run.
    ///
    /// The first frame runs none. A long gap, such as a hidden tab, counts
    /// as at most `max_lag`, so the animation pauses instead of jumping.
    pub fn advance(&mut self, now: Duration) -> u32 {
        let elapsed = match self.last {
            Some(last) => now.saturating_sub(last),
            None => Duration::ZERO,
        };
        self.last = Some(now);
        self.lag = (self.lag + elapsed).min(self.max_lag);

        let mut steps = 0;
        while self.lag >= self.step {
            self.lag -= self.step;
            steps += 1;
        }
        steps
    }
}

/// Picks a pixel budget so that a frame takes about `target` to compute.
///
/// From the last frame it estimates the cost of one pixel, and moves the
/// budget halfway towards what fits. It ignores differences under 20 %,
/// so the canvas isn't resized on every small change in timing.
pub struct Governor {
    target: Duration,
    min: usize,
    budget: usize,
}

impl Governor {
    /// Returns a governor that starts at `budget` pixels and never goes
    /// below `min`.
    pub fn new(target: Duration, min: usize, budget: usize) -> Self {
        Self {
            target,
            min,
            budget,
        }
    }

    /// Records that the last frame took `work` for `pixels` pixels, and
    /// returns the budget for the next frame, at most `limit`.
    pub fn adjust(&mut self, work: Duration, pixels: usize, limit: usize) -> usize {
        if pixels > 0 {
            // Browsers round their timer to as much as 1 ms, so a fast
            // frame can measure as 0; count it as half a millisecond.
            let work = work.max(Duration::from_micros(500)).as_secs_f64();
            let fits = self.target.as_secs_f64() / work * pixels as f64;
            let now = self.budget as f64;
            if (fits - now).abs() > now * 0.2 {
                self.budget = ((now + fits) / 2.0) as usize;
            }
        }
        self.budget = self.budget.clamp(self.min, limit.max(self.min));
        self.budget
    }
}
