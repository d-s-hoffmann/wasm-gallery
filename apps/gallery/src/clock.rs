//! Timing for the animation loop: fixed update steps.
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
