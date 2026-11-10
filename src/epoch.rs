//! Canonical epoch type and the civil UTC representation.

use core::ops::{Add, Sub};

use crate::duration::Duration;
use crate::math;

/// A continuous TAI epoch.
///
/// The value is stored as whole TAI seconds plus nanoseconds since
/// `2000-01-01T00:00:00 TAI`. TAI has no leap seconds, so this
/// representation is monotonic and suitable for arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Epoch {
    sec: i64,
    nanos: u32,
}

impl Epoch {
    /// Builds an epoch from TAI seconds and nanoseconds since
    /// `2000-01-01T00:00:00 TAI`, normalising the nanosecond field.
    pub fn from_parts(sec: i64, nanos: i64) -> Self {
        let sec = sec + nanos.div_euclid(1_000_000_000);
        let nanos = nanos.rem_euclid(1_000_000_000);
        Self {
            sec,
            nanos: nanos as u32,
        }
    }

    /// Builds an epoch from a floating-point TAI second count.
    pub fn from_tai_seconds(seconds: f64) -> Self {
        let whole = math::floor(seconds);
        let frac = seconds - whole;
        Self::from_parts(whole as i64, math::round(frac * 1_000_000_000.0) as i64)
    }

    /// Returns the TAI second count as a floating-point value.
    pub fn tai_seconds(&self) -> f64 {
        self.sec as f64 + self.nanos as f64 * 1e-9
    }

    /// Returns the whole TAI seconds since the internal origin.
    pub fn sec(&self) -> i64 {
        self.sec
    }

    /// Returns the nanosecond remainder.
    pub fn nanos(&self) -> u32 {
        self.nanos
    }
}

/// A civil UTC calendar representation.
///
/// `second` is normally in `0.0 .. 60.0`. On a positive-leap-second day
/// it may be in `60.0 .. 61.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UtcDateTime {
    /// Astronomical year numbering (1 BC is year 0).
    pub year: i32,
    /// Month in `1..=12`.
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Hour in `0..=23`.
    pub hour: u8,
    /// Minute in `0..=59`.
    pub minute: u8,
    /// Seconds, including the fractional part.
    pub second: f64,
}

impl UtcDateTime {
    /// Creates a UTC representation. Range validation is performed by the
    /// conversion routines, because the legal day of month depends on the
    /// selected calendar and the leap-second table.
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

impl Add<Duration> for Epoch {
    type Output = Epoch;

    fn add(self, rhs: Duration) -> Self::Output {
        let sec = self.sec.checked_add(rhs.sec()).expect("epoch overflow");
        Epoch::from_parts(sec, self.nanos as i64 + rhs.nanos() as i64)
    }
}

impl Sub<Duration> for Epoch {
    type Output = Epoch;

    fn sub(self, rhs: Duration) -> Self::Output {
        let sec = self.sec.checked_sub(rhs.sec()).expect("epoch overflow");
        Epoch::from_parts(sec, self.nanos as i64 - rhs.nanos() as i64)
    }
}

impl Sub<Epoch> for Epoch {
    type Output = Duration;

    fn sub(self, rhs: Epoch) -> Self::Output {
        let sec = self.sec.checked_sub(rhs.sec).expect("duration overflow");
        Duration::from_parts(sec, self.nanos as i64 - rhs.nanos as i64)
    }
}
