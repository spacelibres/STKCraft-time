//! N0067 date literals from SPICE text kernels.
//!
//! A text kernel may give an epoch as a date literal that begins with `@`,
//! for example `EPOCH = @2000-JAN-01-12:00:00`. NAIF resolves such a
//! literal with `TPARSE` into **seconds past J2000**. That value is a bare
//! kernel constant: no UTC, TT or TDB conversion is implied, and no
//! leap-second kernel is consulted.
//!
//! [`parse_kernel_date`] implements the literal forms a kernel actually
//! carries. It never falls back to a missing value or to a default epoch:
//! a literal it cannot resolve returns [`TimeError::KernelDateUnsupported`].
//! This distinction matters because a downstream reader must be able to
//! tell an unsupported literal apart from a variable the kernel omitted.
//!
//! # Supported forms
//!
//! * Three date fields, then an optional time field. Fields are separated
//!   by `-` or by `/`, but not by a mixture of both.
//! * Date orders `Y-M-D`, `D-MON-Y`, and `MON-D-Y`, where the month may
//!   be a three-letter abbreviation or a full English name in any case.
//!   A month may also be numeric when the position is unambiguous.
//! * Time of day is `HH:MM` or `HH:MM:SS[.fraction]`.
//! * A day past the end of its month rolls forward exactly as NAIF does:
//!   `@2001-FEB-29` is `2001-03-01`.
//!
//! Two-digit years, ordinal day-of-year forms (`@2000-060`), empty
//! literals, and free text are rejected.

use alloc::vec::Vec;

use crate::calendar::{self, J2000_JDN};
use crate::error::TimeError;
use crate::naif;

/// A resolved kernel date literal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KernelDate {
    /// Seconds past J2000 (`2000-01-01T12:00:00`) as a bare kernel
    /// constant, matching the value a SPICE text kernel assigns.
    pub seconds_past_j2000: f64,
    /// Normalized proleptic-Gregorian year.
    pub year: i32,
    /// Normalized month, `1..=12`.
    pub month: u8,
    /// Normalized day of month.
    pub day: u8,
    /// Hour of day, `0..=23`.
    pub hour: u8,
    /// Minute of hour, `0..=59`.
    pub minute: u8,
    /// Second of minute, `0.0 .. 60.0`.
    pub second: f64,
}

/// Resolves a kernel date literal such as `@2000-JAN-01-12:00:00`.
///
/// The leading `@` is required. The result is the exact N0067/TPARSE
/// value: proleptic-Gregorian calendar arithmetic expressed as seconds
/// past J2000, with no leap-second or time-scale conversion.
///
/// ```rust
/// use stkcraft_time::kernel_date::parse_kernel_date;
///
/// let epoch = parse_kernel_date("@2001-FEB-29")?;
/// assert_eq!(epoch.seconds_past_j2000, 36_676_800.0);
/// assert_eq!((epoch.year, epoch.month, epoch.day), (2001, 3, 1));
/// # Ok::<(), stkcraft_time::TimeError>(())
/// ```
///
/// # Errors
///
/// Returns [`TimeError::KernelDateUnsupported`] when the literal is not a
/// supported form. It never returns a missing-kernel error, because a date
/// literal is present data, not an absent variable.
pub fn parse_kernel_date(text: &str) -> Result<KernelDate, TimeError> {
    let body = text
        .strip_prefix('@')
        .ok_or(TimeError::KernelDateUnsupported)?;

    if body.is_empty() || !body.is_ascii() {
        return Err(TimeError::KernelDateUnsupported);
    }

    let separator = date_separator(body)?;
    let fields: Vec<&str> = body.split(separator).collect();
    if fields.iter().any(|field| field.is_empty()) {
        return Err(TimeError::KernelDateUnsupported);
    }

    let (date_fields, time_field) = split_time(&fields)?;
    let (year_index, year) = find_year(date_fields)?;
    let (month_index, month) = find_month(date_fields, year_index)?;
    let day_index = 3 - year_index - month_index;
    let day = parse_integer(date_fields[day_index])?;

    if year_index == month_index || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(TimeError::KernelDateUnsupported);
    }

    let (hour, minute, second) = match time_field {
        Some(field) => parse_time_of_day(field)?,
        None => (0, 0, 0.0),
    };

    // `gregorian_to_jdn` is affine in the day number, so a day beyond the
    // end of its month carries into the following month. That is the NAIF
    // normalization this parser must reproduce, not an accident.
    let jdn = calendar::gregorian_to_jdn(year, month as u8, day as u8);
    let (normalized_year, normalized_month, normalized_day) = calendar::jdn_to_gregorian(jdn);

    let day_seconds = hour as f64 * 3_600.0 + minute as f64 * 60.0 + second;
    let seconds_past_j2000 = (jdn - J2000_JDN) as f64 * 86_400.0 - 43_200.0 + day_seconds;

    if !seconds_past_j2000.is_finite() {
        return Err(TimeError::KernelDateUnsupported);
    }

    Ok(KernelDate {
        seconds_past_j2000,
        year: normalized_year,
        month: normalized_month,
        day: normalized_day,
        hour,
        minute,
        second,
    })
}

fn date_separator(body: &str) -> Result<char, TimeError> {
    let dash = body.contains('-');
    let slash = body.contains('/');

    match (dash, slash) {
        (true, false) => Ok('-'),
        (false, true) => Ok('/'),
        _ => Err(TimeError::KernelDateUnsupported),
    }
}

/// Splits three date fields plus an optional trailing time field.
fn split_time<'a>(fields: &'a [&'a str]) -> Result<(&'a [&'a str], Option<&'a str>), TimeError> {
    match fields.last() {
        Some(last) if last.contains(':') => {
            if fields.len() != 4 {
                return Err(TimeError::KernelDateUnsupported);
            }
            Ok((&fields[..3], Some(*last)))
        }
        _ => {
            if fields.len() != 3 {
                return Err(TimeError::KernelDateUnsupported);
            }
            Ok((fields, None))
        }
    }
}

/// Finds the single four-digit year field.
fn find_year(fields: &[&str]) -> Result<(usize, i32), TimeError> {
    let mut found: Option<(usize, i32)> = None;

    for (index, field) in fields.iter().enumerate() {
        if let Some(year) = parse_year(field) {
            if found.is_some() {
                return Err(TimeError::KernelDateUnsupported);
            }
            found = Some((index, year));
        }
    }

    found.ok_or(TimeError::KernelDateUnsupported)
}

/// The year must be four ASCII digits; a two-digit year is ambiguous.
fn parse_year(token: &str) -> Option<i32> {
    if token.len() != 4 || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let year = token.parse::<i32>().ok()?;
    if year >= 1000 {
        Some(year)
    } else {
        None
    }
}

/// Finds the month field and its numeric value.
///
/// A month name is unambiguous, so it wins wherever it appears. With only
/// numbers, the remaining fields are taken in month-then-day order, which
/// is the NAIF convention for both `Y-M-D` and `M-D-Y`.
fn find_month(fields: &[&str], year_index: usize) -> Result<(usize, u32), TimeError> {
    if let Some((index, month)) = fields
        .iter()
        .enumerate()
        .find_map(|(index, field)| month_from_name(field).map(|month| (index, month)))
    {
        return Ok((index, month));
    }

    let month_index = if year_index == 0 { 1 } else { 0 };
    let month = parse_integer(fields[month_index])?;
    Ok((month_index, month))
}

fn month_from_name(token: &str) -> Option<u32> {
    let month = match token.to_ascii_uppercase().as_str() {
        "JAN" | "JANUARY" => 1,
        "FEB" | "FEBRUARY" => 2,
        "MAR" | "MARCH" => 3,
        "APR" | "APRIL" => 4,
        "MAY" => 5,
        "JUN" | "JUNE" => 6,
        "JUL" | "JULY" => 7,
        "AUG" | "AUGUST" => 8,
        "SEP" | "SEPT" | "SEPTEMBER" => 9,
        "OCT" | "OCTOBER" => 10,
        "NOV" | "NOVEMBER" => 11,
        "DEC" | "DECEMBER" => 12,
        _ => return None,
    };

    Some(month)
}

fn parse_time_of_day(field: &str) -> Result<(u8, u8, f64), TimeError> {
    let parts: Vec<&str> = field.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(TimeError::KernelDateUnsupported);
    }

    let hour = parse_integer(parts[0])?;
    let minute = parse_integer(parts[1])?;
    if hour > 23 || minute > 59 {
        return Err(TimeError::KernelDateUnsupported);
    }

    let second = if parts.len() == 3 {
        parse_seconds(parts[2])?
    } else {
        0.0
    };

    Ok((hour as u8, minute as u8, second))
}

/// Parses an unsigned ASCII integer with no sign or exponent.
fn parse_integer(token: &str) -> Result<u32, TimeError> {
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TimeError::KernelDateUnsupported);
    }

    token
        .parse::<u32>()
        .map_err(|_| TimeError::KernelDateUnsupported)
}

/// Parses `SS` or `SS.fraction` with NAIF decimal arithmetic.
fn parse_seconds(token: &str) -> Result<f64, TimeError> {
    if token.is_empty()
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return Err(TimeError::KernelDateUnsupported);
    }

    if token.bytes().filter(|byte| *byte == b'.').count() > 1 {
        return Err(TimeError::KernelDateUnsupported);
    }

    let second = naif::parse_f64(token).ok_or(TimeError::KernelDateUnsupported)?;
    if !(0.0..60.0).contains(&second) {
        return Err(TimeError::KernelDateUnsupported);
    }

    Ok(second)
}

#[cfg(test)]
mod tests {
    use super::parse_kernel_date;

    #[test]
    fn single_digit_lsk_day_is_not_a_year() {
        let date = parse_kernel_date("@1972-JAN-1").unwrap();
        assert_eq!(date.seconds_past_j2000, -883_656_000.0);
    }

    #[test]
    fn numeric_month_positions_are_unambiguous() {
        let year_first = parse_kernel_date("@1990-01-02").unwrap();
        let year_last = parse_kernel_date("@01/02/1990").unwrap();

        assert_eq!(
            (year_first.year, year_first.month, year_first.day),
            (1990, 1, 2)
        );
        assert_eq!(
            (year_last.year, year_last.month, year_last.day),
            (1990, 1, 2)
        );
    }

    #[test]
    fn mixed_separators_and_free_text_are_rejected() {
        for text in ["@2000-01/01", "@2000/01-01", "@NOT_A_DATE", "@@2000-JAN-01"] {
            assert!(parse_kernel_date(text).is_err(), "{text}");
        }
    }
}
