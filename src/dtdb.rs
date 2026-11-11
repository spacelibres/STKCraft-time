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
//! The unchecked legacy functions evaluate the geocentric series. The
//! checked [`dtdb_minus_tt_at_observer`] also includes the observer terms
//! from ERFA, derived from SOFA, using caller-supplied UT1 and coordinates.
//! The geocentric DE405 comparison above is not a universal accuracy bound
//! for observer corrections or for arbitrary ephemerides.
//!
//! # Relationship to the other TDB functions
//!
//! The crate exposes three distinct TDB-related quantities on purpose:
//!
//! * [`crate::convert::tai_to_et`] — NAIF `ET`, for SPICE interoperability
//!   (~30 us).
//! * [`crate::tdb::tai_to_tdb`] — a compact HEASARC approximation.
//! * [`tai_to_tdb_iau`] here — the IAU reference series (~3 ns), used by
//!   [`crate::coord`] for TCB.
//!
//! They are not interchangeable; choose by the reference you must match.
//!
//! # Cost
//!
//! One conversion evaluates 787 sines. That is intentional and exact; it
//! is not on any inner loop of this crate.
//!
//! ERFA-derived observer expressions: Copyright (C) 2013-2023, NumFOCUS
//! Foundation. See `ATTRIBUTION.md` for the full ERFA notice and license.

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
    dtdb_minus_tt_split(jd_tt, 0.0)
}

/// Geocentric `TDB - TT` in seconds at a split TT Julian Date.
///
/// The date is `jd1 + jd2`, evaluated without first collapsing the absolute
/// Julian Date. J2000 plus an offset in `jd2` preserves the most precision.
/// Like [`dtdb_minus_tt`], this is unchecked and may extrapolate; use
/// [`dtdb_minus_tt_at_observer`] with `u_km = v_km = 0` for a checked result.
pub fn dtdb_minus_tt_split(jd1: f64, jd2: f64) -> f64 {
    // Time since J2000.0 in Julian millennia.
    let t = ((jd1 - DJ00) + jd2) / DJM;

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

/// Invalid arguments to the checked SOFA/ERFA observer model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DtdbError {
    /// A date, UT1, longitude, or observer coordinate is NaN or infinite.
    NonFiniteInput,
    /// UT1 must be a fraction of a day in `[0, 1)`.
    InvalidUt1Fraction,
    /// The cylindrical distance from the spin axis is negative.
    InvalidObserverDistance,
    /// The TT date is outside the library's 1950-2050 model guard.
    OutsideModelRange,
}

impl core::fmt::Display for DtdbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NonFiniteInput => "TDB model arguments must be finite",
            Self::InvalidUt1Fraction => "UT1 day fraction must be in [0, 1)",
            Self::InvalidObserverDistance => {
                "distance from the Earth spin axis must be nonnegative"
            }
            Self::OutsideModelRange => "TT date is outside JD 2433282.5 through 2469807.5",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DtdbError {}

/// Observer-dependent `TDB - TT` in seconds from the SOFA/ERFA model.
///
/// `jd1 + jd2` is a split TT Julian Date (TT can replace the formal TDB
/// argument with no practical effect on this model). `ut1_fraction` is
/// UT1 time of day in `[0, 1)`, not UTC or the fractional part of a JD;
/// the caller must obtain UT1 independently, for example from EOP data.
/// `east_longitude_radians` is east-positive longitude. `u_km` is the
/// nonnegative distance from Earth's spin axis; `v_km` is the signed
/// distance north of the equatorial plane. These are cylindrical coordinates,
/// not geodetic latitude and height. Zero `u_km` and `v_km` selects the
/// geocentre and makes UT1 and longitude irrelevant to the numerical result.
///
/// The library rejects dates outside the inclusive TT interval
/// 1950-01-01 through 2050-01-01 (JD 2433282.5 through 2469807.5).
/// This is an explicit library guard around the published comparison
/// interval, not a discontinuity in the scientific model. No physical
/// bound is imposed on observer coordinates beyond `u_km >= 0`.
/// The Moyer/Murray observer correction approximates Earth's barycentric
/// velocity dotted with observer position divided by c squared; the
/// geocentric DE405 accuracy statement does not bound observer/EOP errors.
///
/// Adapted from ERFA `dtdb.c`, commit
/// `1a8044cde5b7763295d472a6443387239127c6c8` (SOFA-derived).
pub fn dtdb_minus_tt_at_observer(
    jd1: f64,
    jd2: f64,
    ut1_fraction: f64,
    east_longitude_radians: f64,
    u_km: f64,
    v_km: f64,
) -> Result<f64, DtdbError> {
    if ![jd1, jd2, ut1_fraction, east_longitude_radians, u_km, v_km]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(DtdbError::NonFiniteInput);
    }
    if !(0.0..1.0).contains(&ut1_fraction) {
        return Err(DtdbError::InvalidUt1Fraction);
    }
    if u_km < 0.0 {
        return Err(DtdbError::InvalidObserverDistance);
    }
    // Error-free two-sum retains a fractional part smaller than one absolute
    // JD ulp when checking the inclusive domain. This sum is only used for
    // validation; evaluation below continues to use the split date.
    let date_sum = jd1 + jd2;
    let jd2_virtual = date_sum - jd1;
    let date_error = (jd1 - (date_sum - jd2_virtual)) + (jd2 - jd2_virtual);
    if !date_sum.is_finite()
        || date_sum < 2_433_282.5
        || (date_sum == 2_433_282.5 && date_error < 0.0)
        || date_sum > 2_469_807.5
        || (date_sum == 2_469_807.5 && date_error > 0.0)
    {
        return Err(DtdbError::OutsideModelRange);
    }
    let days_since_j2000 = (jd1 - DJ00) + jd2;
    let t = days_since_j2000 / DJM;
    let tsol = ut1_fraction * core::f64::consts::TAU + east_longitude_radians;
    let w = t / 3600.0;
    let d2r = core::f64::consts::PI / 180.0;
    let elsun = ((280.46645683 + 1296027711.03429 * w) % 360.0) * d2r;
    let emsun = ((357.52910918 + 1295965810.481 * w) % 360.0) * d2r;
    let d = ((297.85019547 + 16029616012.090 * w) % 360.0) * d2r;
    let elj = ((34.35151874 + 109306899.89453 * w) % 360.0) * d2r;
    let els = ((50.07744430 + 44046398.47038 * w) % 360.0) * d2r;
    let wt = 0.00029e-10 * u_km * math::sin(tsol + elsun - els)
        + 0.00100e-10 * u_km * math::sin(tsol - 2.0 * emsun)
        + 0.00133e-10 * u_km * math::sin(tsol - d)
        + 0.00133e-10 * u_km * math::sin(tsol + elsun - elj)
        - 0.00229e-10 * u_km * math::sin(tsol + 2.0 * elsun + emsun)
        - 0.02200e-10 * v_km * math::cos(elsun + emsun)
        + 0.05312e-10 * u_km * math::sin(tsol - emsun)
        - 0.13677e-10 * u_km * math::sin(tsol + 2.0 * elsun)
        - 1.31840e-10 * v_km * math::cos(elsun)
        + 3.17679e-10 * u_km * math::sin(tsol);
    let result = wt + dtdb_minus_tt_split(jd1, jd2);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(DtdbError::NonFiniteInput)
    }
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
