//! Explicit structured raw NAIF tables, separate from scientific validation.
use super::{ContractError, NaifEt, NaifJulianUtc};
use crate::{naif, LeapEntry, TimeError};

/// Explicit `DELTET` constants for native time arithmetic, without defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NaifDeltet {
    /// `DELTET/DELTA_T_A`, seconds.
    pub delta_t_a: f64,
    /// `DELTET/K`, seconds.
    pub k: f64,
    /// `DELTET/EB`.
    pub eb: f64,
    /// First `DELTET/M` component, radians.
    pub m0: f64,
    /// Second `DELTET/M` component, radians per second.
    pub m1: f64,
}

impl NaifDeltet {
    pub(crate) fn values(self) -> [f64; 5] {
        [self.delta_t_a, self.k, self.eb, self.m0, self.m1]
    }
}

/// Native JDUTC context borrowing structured entries in their original order.
///
/// This is an explicitly selected NAIF computational convention, including its
/// historical extrapolation. It is not a scientific/civil leap-second table.
/// Unlike [`crate::LeapSecondTable::new`], it does not sort entries or reject
/// non-increasing TAI boundaries at construction. Native paired-table errors
/// occur at calculation time. Only the first 140 entries enter that table.
/// Arbitrary raw kernel variables and fractional dates/offsets are not modeled.
#[derive(Clone, Copy, Debug)]
pub struct NaifUtcContext<'a> {
    entries: &'a [LeapEntry],
    deltet: NaifDeltet,
}

impl<'a> NaifUtcContext<'a> {
    /// Borrows a nonempty entry slice and explicitly supplied finite constants.
    ///
    /// Entry order, duplicates and TAI boundaries are preserved for native
    /// calculation-stage validation. The scientific constructor is unchanged.
    ///
    /// ```
    /// use stkcraft_time::{LeapEntry, contract::{NaifDeltet, NaifEt, NaifUtcContext}};
    /// let entries = [
    ///     LeapEntry { jdn: 2441318, delta_at: 100000 },
    ///     LeapEntry { jdn: 2441319, delta_at: 0 },
    /// ];
    /// let constants = NaifDeltet {
    ///     delta_t_a: 32.184, k: 0.001657, eb: 0.01671,
    ///     m0: 6.239996, m1: 1.99096871e-7,
    /// };
    /// let context = NaifUtcContext::from_raw_parts(&entries, constants).unwrap();
    /// let error = context.et_to_spice_julian_utc(NaifEt::from_seconds(0.).unwrap()).unwrap_err();
    /// assert_eq!(error.to_string(), "SPICE(BADLEAPSECONDS)");
    /// ```
    pub fn from_raw_parts(
        entries: &'a [LeapEntry],
        deltet: NaifDeltet,
    ) -> Result<Self, ContractError> {
        if entries.is_empty() {
            return Err(TimeError::LskMissing("DELTET/DELTA_AT").into());
        }
        if deltet.values().iter().any(|value| !value.is_finite()) {
            return Err(ContractError::NonFinite);
        }
        Ok(Self { entries, deltet })
    }

    /// Native ET-to-JDUTC bits or the calculation-stage paired-table error.
    ///
    /// No integer Epoch, civil-calendar conversion, sorting or scientific
    /// historical model participates. Finite/domain guards remain explicit.
    pub fn et_to_spice_julian_utc(self, et: NaifEt) -> Result<NaifJulianUtc, ContractError> {
        naif::utc::julian_utc(et.seconds(), self.entries, self.deltet.values()).map(NaifJulianUtc)
    }
}
