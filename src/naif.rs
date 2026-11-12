//! Arithmetic compatible with NAIF N0067 NPARSD and UNITIM.
//!
//! Operation order matters: an ordinary correctly rounded decimal parser
//! can place an SCLK coefficient on the other side of a queried tick.
//! See ATTRIBUTION.md for the reference implementation's provenance.

use crate::{math, LeapSecondTable};

pub(crate) mod utc;

/// N0067 UNITIM TDT to TDB: evaluate the periodic term at the input TT.
pub(crate) fn tt_to_et(tt: f64, lsk: &LeapSecondTable) -> f64 {
    let anomaly = lsk.m0 + lsk.m1 * tt;
    let eccentric = anomaly + lsk.eb * math::sin(anomaly);
    tt + lsk.k * math::sin(eccentric)
}

/// N0067 UNITIM TDB to TDT, preserving its three inverse iterations.
pub(crate) fn et_to_tt(et: f64, lsk: &LeapSecondTable) -> f64 {
    et_to_tt_parts(et, [lsk.k, lsk.eb, lsk.m0, lsk.m1])
}

/// The same UNITIM inverse with explicitly supplied K, EB, M0 and M1.
pub(crate) fn et_to_tt_parts(et: f64, [k, eb, m0, m1]: [f64; 4]) -> f64 {
    let mut tt = et;
    for _ in 0..3 {
        let anomaly = m0 + m1 * tt;
        let eccentric = anomaly + eb * math::sin(anomaly);
        tt = et - k * math::sin(eccentric);
    }
    tt
}

const POWERS: [f64; 11] = [
    1.0,
    10.0,
    100.0,
    1_000.0,
    10_000.0,
    100_000.0,
    1_000_000.0,
    10_000_000.0,
    100_000_000.0,
    1_000_000_000.0,
    10_000_000_000.0,
];

/// Retains Rust's finite numeric grammar while reproducing NAIF arithmetic.
pub(crate) fn parse_f64(token: &str) -> Option<f64> {
    let normalized = token.replace('D', "E").replace('d', "e");
    if !normalized.parse::<f64>().ok()?.is_finite() {
        return None;
    }
    let (negative, unsigned) = match normalized.as_bytes().first() {
        Some(b'-') => (true, &normalized[1..]),
        Some(b'+') => (false, &normalized[1..]),
        _ => (false, normalized.as_str()),
    };
    let mut pieces = unsigned.split(['e', 'E']);
    let mantissa = pieces.next()?;
    let exponent = pieces.next();
    let mut halves = mantissa.split('.');
    let integer = halves.next()?;
    let fraction = halves.next().unwrap_or("");

    let mut integral = 0.0;
    let mut decimal = 0.0;
    let mut divisor = 1.0;
    let mut factor = 1.0;
    let mut shift = 0_i64;
    let mut round_integer = true;
    for digit in integer.bytes().map(|byte| f64::from(byte - b'0')) {
        if integral < 1e15 {
            integral = integral * 10.0 + digit;
        } else {
            shift += 1;
            factor /= 10.0;
            if round_integer {
                round_integer = false;
                if digit > 5.0 {
                    integral += 1.0;
                }
            }
        }
    }
    let zero_integer = integral == 0.0;
    let mut round_decimal = true;
    for digit in fraction.bytes().map(|byte| f64::from(byte - b'0')) {
        if zero_integer {
            if decimal < 1e15 {
                decimal = decimal * 10.0 + digit;
                shift -= 1;
            } else if round_decimal {
                round_decimal = false;
                if digit >= 5.0 {
                    decimal += 1.0;
                }
            }
        } else if divisor < 1e15 {
            decimal = decimal * 10.0 + digit;
            divisor *= 10.0;
        }
    }

    let mut entered_exponent = 0_i64;
    if let Some(exponent) = exponent {
        let (sign, digits) = match exponent.as_bytes().first() {
            Some(b'-') => (-1, &exponent[1..]),
            Some(b'+') => (1, &exponent[1..]),
            _ => (1, exponent),
        };
        for digit in digits.bytes() {
            let combined = entered_exponent + shift;
            if combined > 308 {
                return None;
            }
            if combined >= -309 {
                entered_exponent = entered_exponent * 10 + sign * i64::from(digit - b'0');
            }
        }
    }
    let mut value = integral + (decimal / divisor) * factor;
    let mut exponent = entered_exponent + shift;
    if exponent < 0 {
        while exponent < -10 && value != 0.0 {
            value /= POWERS[10];
            exponent += 10;
        }
        if value != 0.0 {
            value /= POWERS[(-exponent) as usize];
        }
    } else if exponent > 0 && value != 0.0 {
        while exponent > 10 {
            if value >= f64::MAX / POWERS[10] {
                return None;
            }
            value *= POWERS[10];
            exponent -= 10;
        }
        let power = POWERS[exponent as usize];
        if value >= f64::MAX / power {
            return None;
        }
        value *= power;
    }
    Some(if negative { -value } else { value })
}

#[cfg(test)]
mod tests {
    use super::parse_f64;

    #[test]
    fn oracle_numeric_tokens_match_bit_for_bit() {
        for line in include_str!("../tests/golden/naif_numbers.csv").lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let (token, expected) = line.split_once(',').unwrap();
            let bits = u64::from_str_radix(expected, 16).unwrap();
            assert_eq!(parse_f64(token).unwrap().to_bits(), bits, "{token}");
        }
    }

    #[test]
    fn rejects_invalid_and_non_finite_numeric_tokens() {
        for token in [
            "NaN", "inf", "-inf", "1E309", "PI", "E10", "1 2", "", "0E999999",
        ] {
            assert!(parse_f64(token).is_none(), "{token}");
        }
    }

    #[test]
    fn enormous_exponents_and_zero_do_not_require_unbounded_scaling() {
        let negative = alloc::format!("-0E-{}", "9".repeat(10_000));
        assert_eq!(
            parse_f64(&negative).unwrap().to_bits(),
            (-0.0_f64).to_bits()
        );
        let underflow = alloc::format!("1E-{}", "9".repeat(10_000));
        assert_eq!(parse_f64(&underflow), Some(0.0));
        let overflow = alloc::format!("0E{}", "9".repeat(10_000));
        assert!(parse_f64(&overflow).is_none());
    }
}
