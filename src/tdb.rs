//! Compact two-term approximation to geocentric TDB.
//!
//! NAIF's `ET` uses the one-term approximation documented in the LSK:
//!
//! ```text
//! TDB - TT = K * sin(E)
//! ```
//!
//! which NAIF states is accurate to about 30 microseconds. NASA HEASARC
//! publishes this approximate relation in The ABC of XTE: A Time Tutorial:
//! <https://heasarc.gsfc.nasa.gov/docs/xte/abc/time_tutorial.html>.
//!
//! ```text
//! TDB - TT = 0.001658 * sin(g) + 0.000014 * sin(2g)
//! g = 357.53 deg + 0.9856003 deg * (JD_TT - 2451545.0)
//! ```
//!
//! The two models are **not interchangeable**. Use [`crate::convert::tai_to_et`]
//! for NAIF/SPICE interoperability. This compact approximation is not a
//! guaranteed accuracy improvement: a daily 1950-2050 comparison with
//! geocentric ERFA `dtdb` has a maximum residual of about 36.5 us. For the
//! 787-term geocentric model use [`crate::dtdb`]. Observer-dependent
//! topocentric and spacecraft proper-time terms are not implemented here.

use crate::convert::{tai_to_tt, tt_to_tai};
use crate::epoch::Epoch;
use crate::leap::LeapSecondTable;
use crate::math;

/// `TDB - TT` in seconds using the HEASARC two-term approximation.
pub fn tdb_minus_tt_seconds(jd_tt: f64) -> f64 {
    let g_deg = 357.53 + 0.985_600_3 * (jd_tt - 2_451_545.0);
    let g = math::to_radians(g_deg);
    0.001_658 * math::sin(g) + 0.000_014 * math::sin(2.0 * g)
}

/// Converts a TAI epoch to TDB seconds past J2000 TDB using the compact model.
pub fn tai_to_tdb(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    let tt = tai_to_tt(epoch, lsk);
    let jd_tt = 2_451_545.0 + tt / 86_400.0;
    tt + tdb_minus_tt_seconds(jd_tt)
}

/// Converts TDB seconds past J2000 TDB to a TAI epoch using the compact model.
pub fn tdb_to_tai(tdb: f64, lsk: &LeapSecondTable) -> Epoch {
    let mut tt = tdb;

    for _ in 0..4 {
        let jd_tt = 2_451_545.0 + tt / 86_400.0;
        let next = tdb - tdb_minus_tt_seconds(jd_tt);
        if math::abs(next - tt) < 1e-12 {
            tt = next;
            break;
        }
        tt = next;
    }

    tt_to_tai(tt, lsk)
}
