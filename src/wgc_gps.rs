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
    /// The week/TOW input has an invalid format.
    InvalidFormat,
    /// Finite input produced non-finite kernel arithmetic.
    NonFinite,
}

impl core::fmt::Display for WgcGpsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFormat => f.write_str("invalid WebGeocalc GPS week/TOW format"),
            Self::NonFinite => f.write_str("non-finite WebGeocalc GPS value"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WgcGpsError {}

/// Output time system for WebGeocalc day-of-year calendar strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgcCalendarSystem {
    /// Coordinated Universal Time.
    Utc,
    /// Barycentric dynamical time.
    Tdb,
    /// Terrestrial time.
    Tt,
    /// Terrestrial dynamical time, an alias of TT with a distinct label.
    Tdt,
    /// Global Positioning System time.
    Gps,
}

/// Parse the observed WebGeocalc week/TOW input into GPS seconds.
///
/// The week multiplication follows the observed signed 32-bit wrapping
/// arithmetic. No range restriction is imposed on the time of week.
///
/// # Errors
/// Returns [`WgcGpsError::InvalidFormat`] for unsupported input syntax.
pub fn parse_week_tow(input: &str) -> Result<f64, WgcGpsError> {
    let input = trim_wgc(input);
    if input.contains(['\r', '\n']) {
        return Err(WgcGpsError::InvalidFormat);
    }
    let mut fields = input.split(", ");
    let week = fields.next().ok_or(WgcGpsError::InvalidFormat)?;
    let tow = fields.next().ok_or(WgcGpsError::InvalidFormat)?;
    let week = parse_wgc_week(week)?;
    let tow = parse_wgc_double(trim_wgc(tow))?;
    Ok(f64::from(week.wrapping_mul(604_800)) + tow)
}

// WGC accepts the tested BMP decimal digit blocks in the integer week
// field, including mixed scripts, but rejects supplementary-plane digits.
// The adjacent NASA corpus covers every digit in these 37 blocks.
fn parse_wgc_week(value: &str) -> Result<i32, WgcGpsError> {
    const ZEROES: [u32; 37] = [
        0x0030, 0x0660, 0x06f0, 0x07c0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66,
        0x0ce6, 0x0d66, 0x0de6, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946,
        0x19d0, 0x1a80, 0x1a90, 0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0,
        0xa9f0, 0xaa50, 0xabf0, 0xff10,
    ];
    let (negative, digits) = match value.as_bytes().first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    if digits.is_empty() {
        return Err(WgcGpsError::InvalidFormat);
    }
    let mut number = 0_i64;
    for ch in digits.chars() {
        let code = u32::from(ch);
        let digit = ZEROES
            .iter()
            .find_map(|zero| code.checked_sub(*zero).filter(|d| *d < 10))
            .ok_or(WgcGpsError::InvalidFormat)?;
        number = number
            .checked_mul(10)
            .and_then(|n| n.checked_add(i64::from(digit)))
            .filter(|n| {
                *n <= if negative {
                    2_147_483_648
                } else {
                    2_147_483_647
                }
            })
            .ok_or(WgcGpsError::InvalidFormat)?;
    }
    Ok(if negative {
        (-number) as i32
    } else {
        number as i32
    })
}

/// Convert WebGeocalc GPS seconds past GPS zero to checked NAIF ET.
///
/// # Errors
/// Returns [`WgcGpsError::NonFinite`] for non-finite input or arithmetic.
pub fn gps_seconds_to_et(
    seconds: f64,
    lsk: &LeapSecondTable,
) -> Result<crate::contract::NaifEt, WgcGpsError> {
    if !seconds.is_finite() {
        return Err(WgcGpsError::NonFinite);
    }
    let (et, _) = native_gps_roundtrip(seconds, lsk);
    crate::contract::NaifEt::from_seconds(et).map_err(|_| WgcGpsError::NonFinite)
}

/// Convert a WebGeocalc GPS week/TOW input string to checked NAIF ET.
///
/// # Errors
/// Reports invalid input syntax and non-finite input or arithmetic.
pub fn week_tow_to_et(
    input: &str,
    lsk: &LeapSecondTable,
) -> Result<crate::contract::NaifEt, WgcGpsError> {
    gps_seconds_to_et(parse_week_tow(input)?, lsk)
}

/// Convert checked ET to the six-decimal numeric GPS output cell observed
/// from WebGeocalc. This is a display-rounded number, not a lossless epoch.
///
/// # Errors
/// Returns [`WgcGpsError::NonFinite`] for non-finite kernel arithmetic.
pub fn et_to_gps_display_seconds(
    et: crate::contract::NaifEt,
    lsk: &LeapSecondTable,
) -> Result<f64, WgcGpsError> {
    let seconds = (crate::naif::et_to_tt(et.seconds(), lsk) - lsk.delta_t_a) + 630_763_181.0;
    if !seconds.is_finite() {
        return Err(WgcGpsError::NonFinite);
    }
    round_display_six(seconds)
}

fn round_display_six(value: f64) -> Result<f64, WgcGpsError> {
    let negative = value.is_sign_negative();
    let magnitude = f64::from_bits(value.to_bits() & !(1_u64 << 63));
    let decimal = format!("{magnitude}");
    let Some((whole, fraction)) = decimal.split_once('.') else {
        return Ok(value);
    };
    let mut whole = whole.parse::<u64>().map_err(|_| WgcGpsError::NonFinite)?;
    let digits = fraction.as_bytes();
    let mut micros = 0_u32;
    for index in 0..6 {
        micros = micros * 10
            + digits
                .get(index)
                .map_or(0, |digit| u32::from(*digit - b'0'));
    }
    if digits.get(6).is_some_and(|digit| *digit >= b'5') {
        micros += 1;
    }
    if micros == 1_000_000 {
        whole += 1;
        micros = 0;
    }
    let sign = if negative { "-" } else { "" };
    format!("{sign}{whole}.{micros:06}")
        .parse()
        .map_err(|_| WgcGpsError::NonFinite)
}

/// Format a checked ET value as a six-decimal WGC UTC day-of-year string.
///
/// This follows N0067 TIMOUT's six-digit rounding and TTRANS day/seconds
/// arithmetic. It uses the explicit native LSK model.
///
/// # Errors
/// Propagates the checked UTC conversion error when ET is out of range.
pub fn et_to_utc_calendar_doy(
    et: crate::contract::NaifEt,
    lsk: &LeapSecondTable,
) -> Result<String, crate::contract::ContractError> {
    et_to_calendar_impl(et, WgcCalendarSystem::Utc, lsk, false)
}

fn et_to_utc_calendar_impl(
    et: crate::contract::NaifEt,
    lsk: &LeapSecondTable,
    ymd: bool,
) -> Result<String, crate::contract::ContractError> {
    use crate::contract::ContractError;
    use crate::leap::{LeapSecondPolicy, LeapSecondSource};
    if lsk.source() != LeapSecondSource::IersOnly || lsk.policy() != LeapSecondPolicy::TableDriven {
        return Err(ContractError::NaifKernelModelRequired);
    }
    let mut factor = 1.0;
    for _ in 0..6 {
        factor *= 0.1;
    }
    let tdt = crate::naif::et_to_tt(et.seconds(), lsk) + 0.5 * factor;
    let (day, seconds) = crate::naif::utc::day_seconds_from_tdt(
        tdt,
        lsk.entries(),
        [lsk.delta_t_a, lsk.k, lsk.eb, lsk.m0, lsk.m1],
    )?;
    let (hour, minute, second) = calendar_time_parts(seconds);
    let reconstructed_seconds = hour * 3_600.0 + minute * 60.0 + second;
    let previous = crate::naif::utc::tdt_from_day_seconds(
        day,
        reconstructed_seconds,
        lsk.entries(),
        lsk.delta_t_a,
    )?;
    Ok(calendar_string(
        day,
        seconds,
        (tdt - previous).clamp(0.0, 1.0),
        "UTC",
        ymd,
    ))
}

/// Format ET in one of WebGeocalc's calendar day-of-year output systems.
///
/// # Errors
/// Returns an error for invalid kernel arithmetic, incompatible UTC models,
/// or values outside the native TTRANS calendar domain.
pub fn et_to_calendar_doy(
    et: crate::contract::NaifEt,
    system: WgcCalendarSystem,
    lsk: &LeapSecondTable,
) -> Result<String, crate::contract::ContractError> {
    et_to_calendar_impl(et, system, lsk, false)
}

/// Format ET as WebGeocalc's six-decimal calendar year/month/day output.
///
/// # Errors
/// Returns an error for invalid kernel arithmetic, incompatible UTC models,
/// or values outside the native TTRANS calendar domain.
pub fn et_to_calendar_ymd(
    et: crate::contract::NaifEt,
    system: WgcCalendarSystem,
    lsk: &LeapSecondTable,
) -> Result<String, crate::contract::ContractError> {
    et_to_calendar_impl(et, system, lsk, true)
}

fn et_to_calendar_impl(
    et: crate::contract::NaifEt,
    system: WgcCalendarSystem,
    lsk: &LeapSecondTable,
    ymd: bool,
) -> Result<String, crate::contract::ContractError> {
    use crate::contract::ContractError;
    if system == WgcCalendarSystem::Utc {
        return et_to_utc_calendar_impl(et, lsk, ymd);
    }
    let mut factor = 1.0;
    for _ in 0..6 {
        factor *= 0.1;
    }
    let (seconds, label) = match system {
        WgcCalendarSystem::Tdb => (et.seconds(), "TDB"),
        WgcCalendarSystem::Tt => (crate::naif::et_to_tt(et.seconds(), lsk), "TT"),
        WgcCalendarSystem::Tdt => (crate::naif::et_to_tt(et.seconds(), lsk), "TDT"),
        WgcCalendarSystem::Gps => (
            crate::naif::et_to_tt(et.seconds(), lsk) - lsk.delta_t_a - 19.0,
            "GPS",
        ),
        WgcCalendarSystem::Utc => unreachable!(),
    };
    let rounded = seconds + 0.5 * factor;
    let shifted = rounded + 43_200.0;
    if !shifted.is_finite() {
        return Err(ContractError::NonFinite);
    }
    let ratio = shifted / 86_400.0;
    let mut days = if ratio > 0.0 {
        crate::math::floor(ratio)
    } else {
        -crate::math::floor(-ratio)
    };
    let mut remainder = shifted - days * 86_400.0;
    if remainder < 0.0 {
        days -= 1.0;
        remainder += 86_400.0;
    }
    if days < (i32::MIN as f64 - 730_119.0) || days > (i32::MAX as f64 - 730_119.0) {
        return Err(ContractError::EpochOutOfRange);
    }
    let day = (days + 730_119.0) as i32;
    let (hour, minute, second) = calendar_time_parts(remainder);
    let reconstructed_seconds = hour * 3_600.0 + minute * 60.0 + second;
    let previous = days * 86_400.0 - 43_200.0 + reconstructed_seconds;
    Ok(calendar_string(
        day,
        remainder,
        (rounded - previous).clamp(0.0, 1.0),
        label,
        ymd,
    ))
}

fn calendar_time_parts(seconds: f64) -> (f64, f64, f64) {
    let extra = (seconds - 86_400.0 + 1.0).max(0.0);
    let ordinary = seconds - extra;
    let hour = crate::math::floor(ordinary / 3_600.0);
    let remainder = ordinary - hour * 3_600.0;
    let minute = crate::math::floor(remainder / 60.0);
    let second = remainder - minute * 60.0 + extra;
    (hour, minute, second)
}

fn calendar_string(day: i32, seconds: f64, correction: f64, label: &str, ymd: bool) -> String {
    let jdn = i64::from(day) - 730_119 + crate::calendar::J2000_JDN;
    let (year, month, date) = crate::calendar::jdn_to_gregorian(jdn);
    let doy = jdn - crate::calendar::gregorian_to_jdn(year, 1, 1) + 1;
    let (hour, minute, second) = calendar_time_parts(seconds);
    let second = second + correction;
    let whole = crate::math::floor(second);
    let micro = crate::math::floor((second - whole) * 1_000_000.0) as u32;
    let year_text = if year <= 0 {
        format!("{} BC", 1_i64 - i64::from(year))
    } else if year < 1000 {
        format!("{year} AD")
    } else {
        format!("{year}")
    };
    let date_text = if ymd {
        format!("{year_text}-{month:02}-{date:02} ")
    } else {
        format!("{year_text}-{doy:03}/")
    };
    format!(
        "{date_text}{:02}:{:02}:{:02}.{micro:06} {label}",
        hour as u32, minute as u32, whole as u32
    )
}

fn trim_wgc(value: &str) -> &str {
    value.trim_matches(|c: char| c <= '\u{20}')
}

fn parse_wgc_double(value: &str) -> Result<f64, WgcGpsError> {
    let (negative, unsigned) = match value.as_bytes().first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    match unsigned {
        "NaN" => return Ok(f64::NAN),
        "Infinity" => {
            return Ok(if negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            })
        }
        _ => {}
    }
    let value = value.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(value);
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return parse_hex_float(&unsigned[2..], negative);
    }
    if !unsigned.bytes().any(|b| b.is_ascii_digit())
        || unsigned
            .bytes()
            .any(|b| !b.is_ascii_digit() && !matches!(b, b'.' | b'e' | b'E' | b'+' | b'-'))
    {
        return Err(WgcGpsError::InvalidFormat);
    }
    value.parse().map_err(|_| WgcGpsError::InvalidFormat)
}

// Convert hexadecimal significands to binary64 with ties-to-even, including
// subnormal and overflow boundaries. The sticky bit retains all discarded
// input digits rather than truncating long mantissas.
fn parse_hex_float(value: &str, negative: bool) -> Result<f64, WgcGpsError> {
    let (mantissa, exponent) = value
        .split_once(['p', 'P'])
        .ok_or(WgcGpsError::InvalidFormat)?;
    let (exp_negative, exponent) = match exponent.as_bytes().first() {
        Some(b'-') => (true, &exponent[1..]),
        Some(b'+') => (false, &exponent[1..]),
        _ => (false, exponent),
    };
    if exponent.is_empty() || !exponent.bytes().all(|b| b.is_ascii_digit()) {
        return Err(WgcGpsError::InvalidFormat);
    }
    let mut exp = 0_i64;
    for digit in exponent.bytes() {
        exp = exp
            .saturating_mul(10)
            .saturating_add(i64::from(digit - b'0'));
    }
    if exp_negative {
        exp = -exp;
    }
    let mut point = false;
    let mut digits = 0_i64;
    let mut before_point = 0_i64;
    let mut first = None;
    let mut bits = 0_u64;
    let mut count = 0_u32;
    let mut sticky = false;
    for byte in mantissa.bytes() {
        if byte == b'.' && !point {
            point = true;
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return Err(WgcGpsError::InvalidFormat),
        };
        if !point {
            before_point = before_point.saturating_add(1);
        }
        if first.is_none() && digit != 0 {
            first = Some((digits, 7_i64 - i64::from(digit.leading_zeros())));
        }
        digits = digits.saturating_add(1);
        for shift in (0..4).rev() {
            let bit = (digit >> shift) & 1;
            if count == 0 && bit == 0 {
                continue;
            }
            if count < 54 {
                bits = (bits << 1) | u64::from(bit);
                count += 1;
            } else if bit != 0 {
                sticky = true;
            }
        }
    }
    if digits == 0 {
        return Err(WgcGpsError::InvalidFormat);
    }
    let sign = if negative { 1_u64 << 63 } else { 0 };
    let Some((first_index, high_bit)) = first else {
        return Ok(f64::from_bits(sign));
    };
    let mut e = exp
        .saturating_add(
            before_point
                .saturating_sub(first_index)
                .saturating_sub(1)
                .saturating_mul(4),
        )
        .saturating_add(high_bit);
    if e > 1023 {
        return Ok(f64::from_bits(sign | (0x7ff_u64 << 52)));
    }
    if e < -1075 {
        return Ok(f64::from_bits(sign));
    }
    let target = if e >= -1022 {
        53_u32
    } else {
        (e + 1075) as u32
    };
    bits <<= 54 - count;
    let mut rounded = bits >> (54 - target);
    let round = (bits >> (53 - target)) & 1 != 0;
    let lower_mask = (1_u64 << (53 - target)) - 1;
    if round && (sticky || (bits & lower_mask) != 0 || rounded & 1 != 0) {
        rounded += 1;
    }
    if e < -1022 {
        return Ok(f64::from_bits(sign | rounded));
    }
    if rounded == (1_u64 << 53) {
        rounded >>= 1;
        e += 1;
    }
    if e > 1023 {
        return Ok(f64::from_bits(sign | (0x7ff_u64 << 52)));
    }
    Ok(f64::from_bits(
        sign | (((e + 1023) as u64) << 52) | (rounded & ((1_u64 << 52) - 1)),
    ))
}

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
