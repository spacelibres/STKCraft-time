//! IAU TDB model: the Fairhead & Bretagnon (1990) series.
//!
//! `TDB - TT` is the quasi-periodic term of the general-relativistic
//! transformation between the terrestrial and barycentric coordinate
//! scales. NAIF's `ET` approximates it with `K * sin(E)`, which NAIF
//! quotes as accurate to about 30 microseconds. The IAU/SOFA reference
//! implementation (`iauDtdb` / ERFA `dtdb`) uses the full
//! Fairhead & Bretagnon series of 787 terms, whose stated accuracy is
//! **better than ±3 nanoseconds over 1950-2050** against numerical
//! integrations based on JPL DE405.
//!
//! This module implements that series. It is a geocentric model: the
//! observer-dependent topocentric terms of `iauDtdb` (which need a
//! longitude and the observer's distance from the Earth's axis and
//! equator) are omitted, equivalent to zero `u` and `v`.
//!
//! # Relationship to the other TDB functions
//!
//! The crate exposes three distinct TDB-related quantities on purpose:
//!
//! * [`crate::convert::tai_to_et`] — NAIF `ET`, for SPICE interoperability
//!   (~30 us).
//! * [`crate::tdb::tai_to_tdb`] — a compact two-term ESA approximation.
//! * [`tai_to_tdb_iau`] here — the IAU reference series (~3 ns), used by
//!   [`crate::coord`] for TCB.
//!
//! They are not interchangeable; choose by the reference you must match.
//!
//! # Cost
//!
//! One conversion evaluates 787 sines. That is intentional and exact; it
//! is not on any inner loop of this crate.

use crate::convert::{tai_to_tt, tt_to_tai};
use crate::dtdb_table::FAIRHD;
use crate::epoch::Epoch;
use crate::leap::LeapSecondTable;
use crate::math;

/// Julian Date of J2000.0.
const DJ00: f64 = 2_451_545.0;

/// Days per Julian millennium.
const DJM: f64 = 365_250.0;

/// Seconds per day.
const SEC_PER_DAY: f64 = 86_400.0;

/// `TDB - TT` in seconds for a TT Julian Date.
///
/// This is the geocentric Fairhead & Bretagnon series, i.e. ERFA `dtdb`
/// called with `u = v = 0`.
pub fn dtdb_minus_tt(jd_tt: f64) -> f64 {
    // Time since J2000.0 in Julian millennia.
    let t = (jd_tt - DJ00) / DJM;

    let mut w0 = 0.0;
    for row in &FAIRHD[0..474] {
        w0 += row[0] * math::sin(row[1] * t + row[2]);
    }

    let mut w1 = 0.0;
    for row in &FAIRHD[474..679] {
        w1 += row[0] * math::sin(row[1] * t + row[2]);
    }

    let mut w2 = 0.0;
    for row in &FAIRHD[679..764] {
        w2 += row[0] * math::sin(row[1] * t + row[2]);
    }

    let mut w3 = 0.0;
    for row in &FAIRHD[764..784] {
        w3 += row[0] * math::sin(row[1] * t + row[2]);
    }

    let mut w4 = 0.0;
    for row in &FAIRHD[784..787] {
        w4 += row[0] * math::sin(row[1] * t + row[2]);
    }

    // Multiply by powers of T and combine.
    let wf = t * (t * (t * (t * w4 + w3) + w2) + w1) + w0;

    // Adjustments to use JPL planetary masses instead of IAU values.
    let wj = 0.00065e-6 * math::sin(6069.776754 * t + 4.021194)
        + 0.00033e-6 * math::sin(213.299095 * t + 5.543132)
        + (-0.00196e-6) * math::sin(6208.294251 * t + 5.696701)
        + (-0.00173e-6) * math::sin(74.781599 * t + 2.435900)
        + 0.03638e-6 * t * t;

    wf + wj
}

/// Converts a TAI epoch to IAU TDB seconds past `JD 2451545.0` on the TDB
/// scale.
pub fn tai_to_tdb_iau(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    let tt = tai_to_tt(epoch, lsk);
    let jd_tt = DJ00 + tt / SEC_PER_DAY;
    tt + dtdb_minus_tt(jd_tt)
}

/// Converts IAU TDB seconds past `JD 2451545.0` to a TAI epoch.
///
/// The series is evaluated at TT, so the fixed point is solved by
/// iteration; it converges immediately because `TDB - TT` is below 2 ms.
pub fn tdb_iau_to_tai(tdb: f64, lsk: &LeapSecondTable) -> Epoch {
    let mut tt = tdb;

    for _ in 0..4 {
        let jd_tt = DJ00 + tt / SEC_PER_DAY;
        let next = tdb - dtdb_minus_tt(jd_tt);
        if math::abs(next - tt) < 1e-12 {
            tt = next;
            break;
        }
        tt = next;
    }

    tt_to_tai(tt, lsk)
}
