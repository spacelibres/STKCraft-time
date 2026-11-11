//! Coordinate time scales: TCG and TCB.
//!
//! TCG (Geocentric Coordinate Time) and TCB (Barycentric Coordinate Time)
//! are the IAU 2000/2006 coordinate time scales. They are linear
//! functions of TT and TDB:
//!
//! ```text
//! TCG - TT  = L_G / (1 - L_G) * (JD_TT  - T0) * 86400
//! TCB - TDB = (L_B * (JD_TDB - T0) * 86400 - TDB0) / (1 - L_B)
//! ```
//!
//! where `T0 = 1977-01-01T00:00:00 TAI` (`JD 2443144.5003725` in TT),
//! `L_G = 6.969290134e-10`, `L_B = 1.550519768e-8` and
//! `TDB0 = -6.55e-5 s` are the IAU/SOFA constants (ERFA `tttcg`,
//! `tcgtt`, `tcbtdb`, `tdbtcb`).
//!
//! Coordinates here are seconds past the Julian Date `2451545.0`
//! (`2000-01-01T12:00:00`) on the respective scale, so they line up with
//! [`crate::convert::tai_to_tt`] and [`crate::convert::tai_to_et`].
//!
//! Because `JD - T0 = (JD_J2000 - T0) + t/86400`, the relations are
//! exactly linear in `t`, the seconds past J2000. They are evaluated that
//! way; reconstructing an absolute Julian Date from a value near 5e8
//! seconds would lose about `1e-4 s` to `f64` rounding.
//!
//! # Precision, stated honestly
//!
//! TCG depends only on TT, which is `TAI + 32.184 s`; the TCG result is
//! limited only by the constants and `f64` arithmetic.
//!
//! TCB uses the IAU Fairhead-Bretagnon TDB model in [`crate::dtdb`],
//! accurate to better than 3 nanoseconds over 1950-2050, so TCB matches
//! the IAU/SOFA definition to the `f64` representation limit.
//!
//! These values are `f64` seconds near 5e8 at modern epochs, so their
//! spacing is about 60 ns. Neither scale depends on an ephemeris kernel.

use crate::convert::{tai_to_tt, tt_to_tai};
use crate::dtdb::{tai_to_tdb_iau, tdb_iau_to_tai};
use crate::epoch::Epoch;
use crate::leap::LeapSecondTable;

/// IAU 2000 `L_G` (exact): the TCG/TT rate ratio.
pub const L_G: f64 = 6.969_290_134e-10;

/// IAU 2006 `L_B` (exact): the TCB/TDB rate ratio.
pub const L_B: f64 = 1.550_519_768e-8;

/// Defining offset of the IAU TCB/TDB affine relation, in seconds.
///
/// This is the SOFA/ERFA `TDB0` constant used by the TCB alignment.
pub const TDB0: f64 = -6.55e-5;

/// Julian Date of `2000-01-01T12:00:00`, the coordinate origin.
const JD_J2000: f64 = 2_451_545.0;

/// Julian Date of `1977-01-01T00:00:00 TAI` expressed in TT.
const JD_T0_TT: f64 = 2_443_144.500_372_5;

/// Days from T0 to the J2000 origin (`JD_J2000 - T0`).
const J2000_MINUS_T0_DAYS: f64 = JD_J2000 - JD_T0_TT;

/// Seconds per day, as a multiplier.
const SEC_PER_DAY: f64 = 86_400.0;

/// `L_G / (1 - L_G)`, the TT-to-TCG rate used by ERFA `tttcg`.
fn tcg_rate() -> f64 {
    L_G / (1.0 - L_G)
}

/// `L_B / (1 - L_B)`, the TDB-to-TCB rate used by ERFA `tdbtcb`.
fn tcb_rate() -> f64 {
    L_B / (1.0 - L_B)
}

/// Constant term of the TCB linear relation.
fn tcb_offset() -> f64 {
    let rate = tcb_rate();
    J2000_MINUS_T0_DAYS * rate * SEC_PER_DAY - TDB0 * (1.0 + rate)
}

/// `TCG - TT` in seconds at a TAI epoch.
pub fn tcg_minus_tt(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    let rate = tcg_rate();
    let tt = tai_to_tt(epoch, lsk);
    J2000_MINUS_T0_DAYS * rate * SEC_PER_DAY + tt * rate
}

/// Converts a TAI epoch to TCG seconds past `JD 2451545.0` on the TCG
/// scale.
///
/// The value is zero at the TCG instant whose Julian Date is
/// `2451545.0`; at the physical J2000 TT instant it reads about
/// `+0.5058 s` because TCG runs faster than TT.
pub fn tai_to_tcg(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    tai_to_tt(epoch, lsk) + tcg_minus_tt(epoch, lsk)
}

/// Converts TCG seconds past `JD 2451545.0` on the TCG scale to a TAI
/// epoch.
pub fn tcg_to_tai(tcg: f64, lsk: &LeapSecondTable) -> Epoch {
    let rate = tcg_rate();
    let offset = J2000_MINUS_T0_DAYS * rate * SEC_PER_DAY;
    let tt = (tcg - offset) / (1.0 + rate);
    tt_to_tai(tt, lsk)
}

/// `TCB - TDB` in seconds for a TDB value in seconds past `JD 2451545.0`.
pub fn tcb_minus_tdb(tdb: f64) -> f64 {
    // ERFA tdbtcb: TCB = TDB - TDB0*(1 + elbb) + (TDB - T0)*elbb, with
    // elbb = L_B/(1 - L_B). Written in seconds past J2000 this is
    // TCB = tdb*(1 + elbb) + tcb_offset().
    tdb * tcb_rate() + tcb_offset()
}

/// Converts a TAI epoch to TCB seconds past `JD 2451545.0` on the TCB
/// scale.
///
/// This uses the IAU Fairhead-Bretagnon TDB model from [`crate::dtdb`],
/// so it matches the IAU/SOFA definition of TCB rather than NAIF `ET`.
pub fn tai_to_tcb(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    let tdb = tai_to_tdb_iau(epoch, lsk);
    tdb + tcb_minus_tdb(tdb)
}

/// Converts TCB seconds past `JD 2451545.0` on the TCB scale to a TAI
/// epoch.
pub fn tcb_to_tai(tcb: f64, lsk: &LeapSecondTable) -> Epoch {
    // TCB = TDB * (1 + elbb) + tcb_offset().
    let tdb = (tcb - tcb_offset()) / (1.0 + tcb_rate());
    tdb_iau_to_tai(tdb, lsk)
}
