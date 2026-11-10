//! NAIF/SPICE-aligned astronomical time scales.
//!
//! `stkcraft-time` is a clean-room implementation of the time-scale model
//! described in the NAIF *SPICE Time Subsystem Required Reading*. The
//! leap-second data is read from a NAIF Leapseconds Kernel; no leap
//! second is hard-coded in the conversion logic.
//!
//! # Scales
//!
//! * [`Calendar`] selects Gregorian, Julian or Mixed calendar arithmetic.
//! * [`Epoch`] is the canonical internal value: continuous TAI seconds
//!   plus nanoseconds since `2000-01-01T00:00:00 TAI`.
//! * [`UtcDateTime`] is the civil UTC representation, including
//!   `23:59:60.x` on positive-leap-second days.
//! * [`Duration`] is an exact signed duration with nanosecond
//!   resolution.
//! * [`TimeSeries`] iterates uniformly spaced epochs.
//! * [`convert`] provides UTC, TAI, TT, TDB/ET, GPS and UT1 conversions.
//! * [`coord`] provides the IAU coordinate scales TCG and TCB.
//! * [`dtdb`] provides the IAU Fairhead-Bretagnon TDB model.
//! * [`eop`] holds the IERS Earth Orientation Parameters that UT1 requires.
//!
//! # Kernel selection
//!
//! Every UTC conversion is a pure function of the leap-second kernel
//! that is loaded. There is no globally implied kernel and no leap
//! second is hard-coded in the conversion logic. Load the kernel you
//! intend to use with [`LeapSecondTable::from_lsk`]; the bundled
//! [`LeapSecondTable::builtin`] is only a snapshot of `naif0012.tls`.
//! Two kernels with different `DELTET/DELTA_AT` lists produce different
//! `TAI - UTC` after the first differing date. [`LeapSecondTable::coverage`]
//! reports which dates a kernel actually covers.
//!
//! # Accuracy
//!
//! Internal epoch arithmetic is integer-based, and the nanosecond field
//! is exactly that: a step of the internal representation, not a claim
//! of nanosecond accuracy. Real accuracy is bounded by the model and the
//! data:
//!
//! * The leap-second kernel supplies whole-second steps; there is no
//!   sub-second leap data to be had.
//! * The TT/TDB model (`TDB - TT = K * sin(E)`) is the NAIF model, whose
//!   stated accuracy is about 30 microseconds.
//! * `ET` is an `f64` by definition, exactly as in SPICE.
//!
//! Agreement with CSPICE is a match against a reference engine, not a
//! physical truth claim.
//!
//! # Leap-second policy
//!
//! Leap seconds are data, not code. [`LeapSecondTable`] follows the LSK
//! it was built from. To model an explicit policy assumption, construct a
//! table whose entries reflect that assumption; the crate never infers
//! one from a date.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/spacelibres/STKCraft_web/main/public/brand/albatross.png"
)]

extern crate alloc;

pub mod calendar;
pub mod convert;
pub mod coord;
pub mod dtdb;
pub mod duration;
pub mod eop;
pub mod epoch;
pub mod error;
pub mod format;
pub mod leap;
pub mod lsk;
pub mod sclk;
pub mod series;
pub mod tdb;
pub mod timescale;

mod dtdb_table;
mod math;

#[cfg(kani)]
mod verification;

pub use calendar::Calendar;
pub use duration::Duration;
pub use eop::{EopSample, EopSeries, EopTable};
pub use epoch::{Epoch, UtcDateTime};
pub use error::TimeError;
pub use leap::{LeapEntry, LeapSecondPolicy, LeapSecondSource, LeapSecondTable};
pub use sclk::{SclkKernel, SclkTimeSystem};
pub use series::TimeSeries;
pub use timescale::TimeScale;
