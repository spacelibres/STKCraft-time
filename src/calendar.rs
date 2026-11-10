//! Calendar arithmetic and Julian Date conversion.
//!
//! The default calendar is the proleptic Gregorian calendar, matching
//! NAIF/SPICE default behaviour. The Julian calendar and the "Mixed"
//! calendar (Julian before 1582-10-15, Gregorian afterwards) are also
//! supported.

use crate::error::TimeError;

/// Julian Day Number of `2000-01-01` (Gregorian). This is also the day
/// containing the J2000 epoch (`2000-01-01T12:00:00`).
pub const J2000_JDN: i64 = 2_451_545;

/// Julian Day Number of `1980-01-06`, the GPS epoch date.
pub const GPS_EPOCH_JDN: i64 = 2_444_245;

/// Julian Day Number of `1582-10-15`, the first Gregorian day in the
/// Mixed calendar.
pub const GREGORIAN_SWITCH_JDN: i64 = 2_299_161;

/// Calendar system used to interpret a date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calendar {
    /// Proleptic Gregorian calendar.
    Gregorian,
    /// Julian calendar.
    Julian,
    /// Julian before 1582-10-15, Gregorian on and after that date.
    Mixed,
}

/// Returns `true` when `year` is a leap year in the Gregorian calendar.
pub fn is_gregorian_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Returns `true` when `year` is a leap year in the Julian calendar.
pub fn is_julian_leap(year: i32) -> bool {
    year % 4 == 0
}

/// Number of days in the given month, or `None` if the month is invalid.
pub fn days_in_month(year: i32, month: u8, cal: Calendar) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => {
            let leap = match cal {
                Calendar::Gregorian => is_gregorian_leap(year),
                Calendar::Julian => is_julian_leap(year),
                Calendar::Mixed => {
                    if (year, month, 1) < (1582, 10, 15) {
                        is_julian_leap(year)
                    } else {
                        is_gregorian_leap(year)
                    }
                }
            };
            Some(if leap { 29 } else { 28 })
        }
        _ => None,
    }
}

/// Converts a Gregorian calendar date to its Julian Day Number.
pub fn gregorian_to_jdn(year: i32, month: u8, day: u8) -> i64 {
    let a = (14 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    day as i64 + (153 * m + 2) / 5 + 365 * y + y.div_euclid(4) - y.div_euclid(100)
        + y.div_euclid(400)
        - 32045
}

/// Converts a Julian calendar date to its Julian Day Number.
pub fn julian_to_jdn(year: i32, month: u8, day: u8) -> i64 {
    let a = (14 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    day as i64 + (153 * m + 2) / 5 + 365 * y + y.div_euclid(4) - 32083
}

/// Converts a Julian Day Number to a Gregorian calendar date.
pub fn jdn_to_gregorian(jdn: i64) -> (i32, u8, u8) {
    let a = jdn + 32044;
    let b = (4 * a + 3).div_euclid(146097);
    let c = a - (146097 * b).div_euclid(4);
    let d = (4 * c + 3).div_euclid(1461);
    let e = c - (1461 * d).div_euclid(4);
    let m = (5 * e + 2).div_euclid(153);
    let day = e - (153 * m + 2).div_euclid(5) + 1;
    let month = m + 3 - 12 * m.div_euclid(10);
    let year = 100 * b + d - 4800 + m.div_euclid(10);
    (year as i32, month as u8, day as u8)
}

/// Converts a Julian Day Number to a Julian calendar date.
pub fn jdn_to_julian(jdn: i64) -> (i32, u8, u8) {
    let c = jdn + 32082;
    let d = (4 * c + 3).div_euclid(1461);
    let e = c - (1461 * d).div_euclid(4);
    let m = (5 * e + 2).div_euclid(153);
    let day = e - (153 * m + 2).div_euclid(5) + 1;
    let month = m + 3 - 12 * m.div_euclid(10);
    let year = d - 4800 + m.div_euclid(10);
    (year as i32, month as u8, day as u8)
}

/// Converts a Julian Day Number to a calendar date in the selected calendar.
pub fn jdn_to_calendar(jdn: i64, cal: Calendar) -> (i32, u8, u8) {
    match cal {
        Calendar::Gregorian => jdn_to_gregorian(jdn),
        Calendar::Julian => jdn_to_julian(jdn),
        Calendar::Mixed => {
            if jdn >= GREGORIAN_SWITCH_JDN {
                jdn_to_gregorian(jdn)
            } else {
                jdn_to_julian(jdn)
            }
        }
    }
}

/// Validates a calendar date and converts it to a Julian Day Number.
pub fn calendar_to_jdn(year: i32, month: u8, day: u8, cal: Calendar) -> Result<i64, TimeError> {
    if !(1..=12).contains(&month) {
        return Err(TimeError::InvalidDate { year, month, day });
    }
    match days_in_month(year, month, cal) {
        Some(max) if day >= 1 && day <= max => {}
        _ => return Err(TimeError::InvalidDate { year, month, day }),
    }

    match cal {
        Calendar::Gregorian => Ok(gregorian_to_jdn(year, month, day)),
        Calendar::Julian => Ok(julian_to_jdn(year, month, day)),
        Calendar::Mixed => {
            // The ten days 1582-10-05 .. 1582-10-14 never existed in the
            // Mixed calendar.
            if year == 1582 && month == 10 && (5..=14).contains(&day) {
                return Err(TimeError::InvalidDate { year, month, day });
            }
            if (year, month, day) < (1582, 10, 15) {
                Ok(julian_to_jdn(year, month, day))
            } else {
                Ok(gregorian_to_jdn(year, month, day))
            }
        }
    }
}

/// Converts a calendar date and time of day to a Julian Date.
pub fn calendar_to_jd(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: f64,
    cal: Calendar,
) -> Result<f64, TimeError> {
    if hour > 23 || minute > 59 || !(0.0..60.0).contains(&second) {
        return Err(TimeError::InvalidSecond);
    }
    let jdn = calendar_to_jdn(year, month, day, cal)?;
    let day_fraction = (hour as f64 * 3_600.0 + minute as f64 * 60.0 + second) / 86_400.0;
    Ok(jdn as f64 - 0.5 + day_fraction)
}

/// Converts a Julian Date to a Modified Julian Date:
/// `MJD = JD - 2400000.5`. SPICE itself does not define MJD because the
/// abbreviation is ambiguous; this crate uses the IAU definition.
pub fn jd_to_mjd(jd: f64) -> f64 {
    jd - 2_400_000.5
}

/// Converts a Modified Julian Date to a Julian Date:
/// `JD = MJD + 2400000.5`.
pub fn mjd_to_jd(mjd: f64) -> f64 {
    mjd + 2_400_000.5
}
