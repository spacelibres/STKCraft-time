//! Exact signed durations with nanosecond resolution.
//!
//! The representation is an `i64` count of seconds plus a normalised
//! `i32` count of nanoseconds in `0..1_000_000_000`. This gives 1 ns
//! resolution over roughly ±292 billion years, which is far wider than
//! the ±65,536-century range of fixed-point century representations.
//!
//! All arithmetic is integer arithmetic. No floating-point accumulation
//! is used by the core operations.

use core::fmt;
use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::math;

/// A signed duration with nanosecond resolution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    sec: i64,
    nanos: i32,
}

impl Duration {
    /// Zero duration.
    pub const ZERO: Self = Self { sec: 0, nanos: 0 };
    /// One nanosecond.
    pub const NANOSECOND: Self = Self { sec: 0, nanos: 1 };
    /// One second.
    pub const SECOND: Self = Self { sec: 1, nanos: 0 };

    /// Builds a duration from seconds and nanoseconds, normalising the
    /// nanosecond field into `0..1_000_000_000`.
    pub fn from_parts(sec: i64, nanos: i64) -> Self {
        Self::checked_from_parts(sec, nanos).expect("duration overflow")
    }

    /// Checked form of [`Duration::from_parts`].
    pub fn checked_from_parts(sec: i64, nanos: i64) -> Option<Self> {
        let carry = nanos.div_euclid(1_000_000_000);
        let nanos = nanos.rem_euclid(1_000_000_000);
        let sec = sec.checked_add(carry)?;
        Some(Self {
            sec,
            nanos: nanos as i32,
        })
    }

    /// Builds a duration from a total number of nanoseconds.
    pub fn from_nanoseconds(ns: i128) -> Option<Self> {
        let sec = ns.div_euclid(1_000_000_000);
        let nanos = ns.rem_euclid(1_000_000_000);
        if sec < i64::MIN as i128 || sec > i64::MAX as i128 {
            return None;
        }
        Some(Self {
            sec: sec as i64,
            nanos: nanos as i32,
        })
    }

    /// Builds a duration from a floating-point second count.
    pub fn from_seconds_f64(seconds: f64) -> Self {
        let whole = math::floor(seconds);
        let frac = seconds - whole;
        Self::from_parts(whole as i64, math::round(frac * 1_000_000_000.0) as i64)
    }

    /// Whole seconds component. For negative durations this is the floor,
    /// so `-1.5 s` is stored as `sec = -2, nanos = 500_000_000`.
    pub fn sec(&self) -> i64 {
        self.sec
    }

    /// Nanosecond component, always in `0..1_000_000_000`.
    pub fn nanos(&self) -> i32 {
        self.nanos
    }

    /// Total nanoseconds as an `i128`.
    pub fn total_nanoseconds(&self) -> i128 {
        self.sec as i128 * 1_000_000_000 + self.nanos as i128
    }

    /// Floating-point seconds. This is the boundary at which precision is
    /// limited by `f64`.
    pub fn seconds_f64(&self) -> f64 {
        self.sec as f64 + self.nanos as f64 * 1e-9
    }

    /// Returns `true` when the duration is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.sec == 0 && self.nanos == 0
    }

    /// Returns `-1`, `0` or `1`.
    pub fn signum(&self) -> i8 {
        if self.sec < 0 || (self.sec == 0 && self.nanos < 0) {
            -1
        } else if self.sec > 0 || self.nanos > 0 {
            1
        } else {
            0
        }
    }

    /// Absolute value.
    pub fn abs(&self) -> Self {
        if *self < Self::ZERO {
            self.checked_neg().expect("duration overflow")
        } else {
            *self
        }
    }

    /// Checked negation.
    pub fn checked_neg(&self) -> Option<Self> {
        let sec = self.sec.checked_neg()?;
        Some(Self::from_parts(sec, -(self.nanos as i64)))
    }

    /// Checked addition.
    pub fn checked_add(&self, other: Self) -> Option<Self> {
        let nanos = self.nanos as i64 + other.nanos as i64;
        let carry = nanos.div_euclid(1_000_000_000);
        let nanos = nanos.rem_euclid(1_000_000_000);
        let sec = self.sec.checked_add(other.sec)?.checked_add(carry)?;
        Some(Self {
            sec,
            nanos: nanos as i32,
        })
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: Self) -> Option<Self> {
        self.checked_add(other.checked_neg()?)
    }

    /// Checked multiplication by an integer.
    pub fn checked_mul_i64(&self, rhs: i64) -> Option<Self> {
        let total = self.total_nanoseconds().checked_mul(rhs as i128)?;
        Self::from_nanoseconds(total)
    }

    /// Rounds toward negative infinity to a multiple of `step`.
    pub fn floor(&self, step: Self) -> Self {
        let step_ns = positive_step(step);
        let total = self.total_nanoseconds();
        Self::from_nanoseconds(total.div_euclid(step_ns) * step_ns).expect("duration overflow")
    }

    /// Rounds toward positive infinity to a multiple of `step`.
    pub fn ceil(&self, step: Self) -> Self {
        let step_ns = positive_step(step);
        let total = self.total_nanoseconds();
        let quotient = total.div_euclid(step_ns);
        let remainder = total.rem_euclid(step_ns);
        let result = if remainder == 0 {
            quotient * step_ns
        } else {
            (quotient + 1) * step_ns
        };
        Self::from_nanoseconds(result).expect("duration overflow")
    }

    /// Rounds to the nearest multiple of `step`, with ties rounded up.
    pub fn round(&self, step: Self) -> Self {
        let step_ns = positive_step(step);
        let total = self.total_nanoseconds();
        let quotient = total.div_euclid(step_ns);
        let remainder = total.rem_euclid(step_ns);
        let result = if remainder * 2 >= step_ns {
            (quotient + 1) * step_ns
        } else {
            quotient * step_ns
        };
        Self::from_nanoseconds(result).expect("duration overflow")
    }
}

fn positive_step(step: Duration) -> i128 {
    let step_ns = step.total_nanoseconds();
    assert!(step_ns > 0, "step must be positive");
    step_ns
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).expect("duration overflow")
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).expect("duration overflow")
    }
}

impl Neg for Duration {
    type Output = Duration;

    fn neg(self) -> Self::Output {
        self.checked_neg().expect("duration overflow")
    }
}

impl Mul<i64> for Duration {
    type Output = Duration;

    fn mul(self, rhs: i64) -> Self::Output {
        self.checked_mul_i64(rhs).expect("duration overflow")
    }
}

impl Div<i64> for Duration {
    type Output = Duration;

    fn div(self, rhs: i64) -> Self::Output {
        assert!(rhs != 0, "division by zero");
        let total = self.total_nanoseconds() / rhs as i128;
        Self::from_nanoseconds(total).expect("duration overflow")
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.total_nanoseconds();
        let sign = if total < 0 { "-" } else { "" };
        let absolute = total.abs();
        let sec = absolute / 1_000_000_000;
        let nanos = absolute % 1_000_000_000;

        if nanos == 0 {
            write!(f, "{sign}{sec} s")
        } else {
            write!(f, "{sign}{sec}.{nanos:09} s")
        }
    }
}
