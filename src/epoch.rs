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
    ///
    /// Panics when the result cannot be represented. Use
    /// [`Epoch::checked_from_parts`] for untrusted input.
    pub fn from_parts(sec: i64, nanos: i64) -> Self {
        Self::checked_from_parts(sec, nanos).expect("epoch overflow")
    }

    /// Checked form of [`Epoch::from_parts`].
    pub fn checked_from_parts(sec: i64, nanos: i64) -> Option<Self> {
        let total = (sec as i128)
            .checked_mul(1_000_000_000)?
            .checked_add(nanos as i128)?;
        Self::from_nanoseconds(total)
    }

    /// Builds an epoch from a floating-point TAI second count.
    ///
    /// Panics on a non-finite or out-of-range value. Use
    /// [`Epoch::checked_from_tai_seconds`] for untrusted input.
    pub fn from_tai_seconds(seconds: f64) -> Self {
        Self::checked_from_tai_seconds(seconds).expect("invalid or out-of-range epoch")
    }

    /// Checked form of [`Epoch::from_tai_seconds`].
    pub fn checked_from_tai_seconds(seconds: f64) -> Option<Self> {
        if !seconds.is_finite() {
            return None;
        }
        let whole = math::floor(seconds);
        if whole < i64::MIN as f64 || whole >= i64::MAX as f64 {
            return None;
        }
        let frac = seconds - whole;
        let nanos = math::round(frac * 1_000_000_000.0) as i64;
        Self::checked_from_parts(whole as i64, nanos)
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

    /// Checked addition of a duration.
    pub fn checked_add(self, rhs: Duration) -> Option<Self> {
        Self::from_nanoseconds(
            self.total_nanoseconds()
                .checked_add(rhs.total_nanoseconds())?,
        )
    }

    /// Checked subtraction of a duration.
    pub fn checked_sub(self, rhs: Duration) -> Option<Self> {
        Self::from_nanoseconds(
            self.total_nanoseconds()
                .checked_sub(rhs.total_nanoseconds())?,
        )
    }

    fn total_nanoseconds(&self) -> i128 {
        self.sec as i128 * 1_000_000_000 + self.nanos as i128
    }

    fn from_nanoseconds(total: i128) -> Option<Self> {
        let sec = total.div_euclid(1_000_000_000);
        let nanos = total.rem_euclid(1_000_000_000);
        if sec < i64::MIN as i128 || sec > i64::MAX as i128 {
            return None;
        }
        Some(Self {
            sec: sec as i64,
            nanos: nanos as u32,
        })
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
        self.checked_add(rhs).expect("epoch overflow")
    }
}

impl Sub<Duration> for Epoch {
    type Output = Epoch;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs).expect("epoch overflow")
    }
}

impl Sub<Epoch> for Epoch {
    type Output = Duration;

    fn sub(self, rhs: Epoch) -> Self::Output {
        let total = self
            .total_nanoseconds()
            .checked_sub(rhs.total_nanoseconds())
            .expect("duration overflow");
        Duration::from_nanoseconds(total).expect("duration overflow")
    }
}
