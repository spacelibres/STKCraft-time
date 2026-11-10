//! A uniform time series between two epochs.
//!
//! The iterator yields `start`, `start + step`, `start + 2*step`, ... up
//! to `end`. It is deliberately simple and integer-based: no floating
//! point accumulation is involved.

use crate::duration::Duration;
use crate::epoch::Epoch;

/// An iterator over uniformly spaced epochs.
#[derive(Clone, Debug)]
pub struct TimeSeries {
    end: Epoch,
    step: Duration,
    inclusive: bool,
    cursor: Option<Epoch>,
}

impl TimeSeries {
    /// Creates a series that includes `end` when it lies exactly on a step.
    pub fn inclusive(start: Epoch, end: Epoch, step: Duration) -> Self {
        Self {
            end,
            step,
            inclusive: true,
            cursor: Some(start),
        }
    }

    /// Creates a series that stops before `end`.
    pub fn exclusive(start: Epoch, end: Epoch, step: Duration) -> Self {
        Self {
            end,
            step,
            inclusive: false,
            cursor: Some(start),
        }
    }
}

impl Iterator for TimeSeries {
    type Item = Epoch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step <= Duration::ZERO {
            self.cursor = None;
            return None;
        }

        let current = self.cursor?;
        let within = if self.inclusive {
            current <= self.end
        } else {
            current < self.end
        };

        if !within {
            self.cursor = None;
            return None;
        }

        self.cursor = Some(current + self.step);
        Some(current)
    }
}
