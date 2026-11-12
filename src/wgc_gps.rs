//! Explicit WebGeocalc GPS week/TOW compatibility paths.
//!
//! These paths intentionally reproduce the observed NAIF binary64 round-trip
//! and six-decimal display. They are not exact integer epoch representations.

use crate::{naif, LeapSecondTable};
use alloc::format;
use alloc::string::String;

/// An invalid value for a WebGeocalc GPS formatting operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgcGpsError {
    /// Finite input produced non-finite kernel arithmetic.
    NonFinite,
}

impl core::fmt::Display for WgcGpsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFinite => f.write_str("non-finite WebGeocalc GPS value"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WgcGpsError {}

/// Format GPS seconds past 1980-01-06 00:00:00 UTC as WGC week/TOW.
///
/// Finite input passes through the observed N0067 TAI-to-TDB-to-TAI
/// binary64 path using the caller's LSK constants. The displayed remainder uses six
/// fractional digits; the week is not changed if display rounding reaches
/// 604800. The observed week count saturates converted seconds to i32 before
/// integer division. The displayed remainder rounds the shortest decimal
/// representation half away from zero. Tested WGC2/kernel-set coverage is
/// documented in the reference corpus and is not exhaustive.
///
/// # Errors
/// Non-finite GPS input returns the observed WGC string `0, NaN`.
/// Returns [`WgcGpsError::NonFinite`] if finite input produces non-finite
/// kernel arithmetic.
pub fn gps_seconds_to_week_tow(seconds: f64, lsk: &LeapSecondTable) -> Result<String, WgcGpsError> {
    if !seconds.is_finite() {
        return Ok(String::from("0, NaN"));
    }
    const SECONDS_PER_WEEK: f64 = 604_800.0;
    let (_, converted) = native_gps_roundtrip(seconds, lsk);
    if !converted.is_finite() {
        return Err(WgcGpsError::NonFinite);
    }
    // WGC's observed week field saturates the whole GPS-second count to
    // i32 before integer division, while its remainder uses the full f64.
    let week = (converted as i32) / 604_800;
    let tow = converted % SECONDS_PER_WEEK;
    let negative = tow.is_sign_negative();
    let magnitude = f64::from_bits(tow.to_bits() & !(1_u64 << 63));
    let decimal = format!("{magnitude}");
    let (whole, fraction) = decimal.split_once('.').unwrap_or((&decimal, ""));
    let whole = whole.parse::<u64>().expect("GPS remainder is bounded");
    let digits = fraction.as_bytes();
    let mut micros = 0_u64;
    for index in 0..6 {
        micros = micros * 10
            + digits
                .get(index)
                .map_or(0, |digit| u64::from(*digit - b'0'));
    }
    if digits.get(6).is_some_and(|digit| *digit >= b'5') {
        micros += 1;
    }
    let total = whole * 1_000_000 + micros;
    let sign = if negative { "-" } else { "" };
    Ok(format!(
        "{week}, {sign}{}.{:06}",
        total / 1_000_000,
        total % 1_000_000
    ))
}

fn native_gps_roundtrip(seconds: f64, lsk: &LeapSecondTable) -> (f64, f64) {
    const GPS_EPOCH_TAI_J2000: f64 = -630_763_181.0;
    let tai = seconds + GPS_EPOCH_TAI_J2000;
    let tt = tai + lsk.delta_t_a;
    let et = naif::tt_to_et(tt, lsk);
    let converted = (naif::et_to_tt(et, lsk) - lsk.delta_t_a) - GPS_EPOCH_TAI_J2000;
    (et, converted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_unitim_intermediate_bits_match() {
        let lsk = LeapSecondTable::from_lsk(include_str!("naif0012.tls")).unwrap();
        let rows = include_str!("../tests/reference/wgc-gps/unitim-native.tsv");
        assert_eq!(rows.lines().count(), 355);
        for row in rows.lines().skip(1) {
            let bits: Vec<u64> = row
                .split('\t')
                .map(|v| u64::from_str_radix(v, 16).unwrap())
                .collect();
            assert_eq!(bits.len(), 3);
            let seconds = f64::from_bits(bits[0]);
            let (et, converted) = native_gps_roundtrip(seconds, &lsk);
            assert_eq!(et.to_bits(), bits[1], "ET for {seconds}");
            assert_eq!(converted.to_bits(), bits[2], "GPS seconds for {seconds}");
        }
    }
    #[test]
    fn native_unitim_with_altered_constants_matches() {
        let mut lsk = LeapSecondTable::from_lsk(include_str!("naif0012.tls")).unwrap();
        lsk.k = 1.0;
        lsk.eb = 0.2;
        lsk.m0 = 0.1;
        lsk.m1 = 0.5;
        let rows = include_str!("../tests/reference/wgc-gps/unitim-altered.tsv");
        assert_eq!(rows.lines().count(), 5);
        for row in rows.lines().skip(1) {
            let bits: Vec<u64> = row
                .split('\t')
                .map(|v| u64::from_str_radix(v, 16).unwrap())
                .collect();
            assert_eq!(bits.len(), 3);
            let (et, converted) = native_gps_roundtrip(f64::from_bits(bits[0]), &lsk);
            assert_eq!(et.to_bits(), bits[1], "ET for {row}");
            assert_eq!(converted.to_bits(), bits[2], "GPS seconds for {row}");
        }
    }
}
