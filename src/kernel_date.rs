//! N0067 date literals from SPICE text kernels.
//!
//! A text kernel may give an epoch as a date literal that begins with `@`,
//! for example `EPOCH = @2000-JAN-01-12:00:00`. NAIF resolves such a
//! literal with `TPARSE` into **seconds past J2000**. That value is a bare
//! kernel constant: no UTC, TT or TDB conversion is implied, and no
//! leap-second kernel is consulted.
//!
//! [`parse_kernel_date`] implements the documented kernel-literal grammar.
//! It never falls back to a missing value or to a default epoch:
//! a literal it cannot resolve returns [`TimeError::KernelDateUnsupported`].
//! This distinction matters because a downstream reader must be able to
//! tell an unsupported literal apart from a variable the kernel omitted.
//!
//! # Supported forms
//!
//! * Named/numeric calendar dates, mixed dash/slash delimiters and native
//!   month-name prefixes. Year selection follows TPARTV token categories.
//! * Short years use N0067's default 1969--2068 expansion interval.
//! * Time is `HH[:MM[:SS]]`, with a decimal permitted only in its final
//!   component. Integer tokens at least 1000 are classified as years.
//! * Month, day and clock overflow follow native normalization and the
//!   original Muller-Wimberly integer expression.
//! * Julian dates marked with `JD`, numeric ISO calendar/ordinal forms,
//!   and year/day forms with a slash or explicit day-of-year marker.
//!
//! Standalone `@2000-060`, unrecognized text and unsupported grammar return
//! an explicit error. Era/zone modifiers and a configurable TSETYR context
//! are outside this resolver. Full TPARSE grammar conformance is not claimed.

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
    if body.is_empty() || !body.is_ascii() || body.bytes().any(|x| x.is_ascii_whitespace()) {
        return Err(TimeError::KernelDateUnsupported);
    }
    let upper = body.to_ascii_uppercase();
    if let Some(number) = upper
        .strip_prefix("JD")
        .or_else(|| upper.strip_suffix("JD"))
    {
        let unsigned = number.strip_prefix('-').unwrap_or(number);
        decimal(unsigned)?;
        let jd = naif::parse_f64(number).ok_or(TimeError::KernelDateUnsupported)?;
        let seconds = (jd - 2451545.0) * 86400.0;
        return calendar_result(seconds, 0, [0.0, 0.0, 0.0], true);
    }

    if let Some((date, time)) = upper
        .split_once('T')
        .filter(|(date, _)| date.bytes().all(|c| c.is_ascii_digit() || c == b'-'))
    {
        let fields: Vec<_> = date.split('-').collect();
        if fields
            .iter()
            .skip(1)
            .any(|field| integer(field).is_ok_and(|value| value >= 1000))
        {
            return Err(TimeError::KernelDateUnsupported);
        }
        let clock = clock(time)?;
        let (year, month, day) = match fields.as_slice() {
            [year, day] => (integer(year)?, 1, integer(day)?),
            [year, month, day] => (integer(year)?, integer(month)?, integer(day)?),
            _ => return Err(TimeError::KernelDateUnsupported),
        };
        return from_calendar(expand_year(year), month, day, clock);
    }

    for marker in ["//", "::"] {
        if let Some((date, time)) = body.split_once(marker) {
            let fields: Vec<_> = date.split(['-', '/']).collect();
            if let [year, day] = fields.as_slice() {
                return from_calendar(expand_year(integer(year)?), 1, integer(day)?, clock(time)?);
            }
            return Err(TimeError::KernelDateUnsupported);
        }
    }
    let fields: Vec<_> = body.split(['-', '/']).collect();
    if fields.iter().any(|field| field.is_empty()) {
        return Err(TimeError::KernelDateUnsupported);
    }
    if fields.len() == 3 && fields[2].contains(':') {
        // TPARTV's year/day/time slash production is distinct from a
        // three-component calendar date and from a standalone YYYY-DDD.
        let delimiter = body.rfind('/').ok_or(TimeError::KernelDateUnsupported)?;
        if &body[delimiter + 1..] != fields[2] {
            return Err(TimeError::KernelDateUnsupported);
        }
        return from_calendar(
            expand_year(integer(fields[0])?),
            1,
            integer(fields[1])?,
            clock(fields[2])?,
        );
    }
    let (date, clock) = match fields.as_slice() {
        [a, b, c] => ([*a, *b, *c], [0.0, 0.0, 0.0]),
        [a, b, c, time] => ([*a, *b, *c], clock(time)?),
        _ => return Err(TimeError::KernelDateUnsupported),
    };
    // TPARTV permits a decimal only in the final numeric component.
    if date[..2].iter().any(|field| field.contains('.'))
        || (fields.len() == 4 && date[2].contains('.'))
    {
        return Err(TimeError::KernelDateUnsupported);
    }
    let named = date
        .iter()
        .enumerate()
        .find_map(|(index, field)| month(field).map(|month| (index, month)));
    let years: Vec<_> = date
        .iter()
        .enumerate()
        .filter_map(|(index, field)| {
            integer(field)
                .ok()
                .filter(|value| *value >= 1000)
                .map(|value| (index, value))
        })
        .collect();
    let year_index = match years.as_slice() {
        [(index, _)] => *index,
        [] => match named {
            Some((0, _)) => 2,
            Some(_) => 0,
            None if fields.len() == 3 && body.contains('/') => 2,
            None => return Err(TimeError::KernelDateUnsupported),
        },
        _ => return Err(TimeError::KernelDateUnsupported),
    };
    let month_index = match named {
        Some((index, _)) => index,
        None if year_index == 0 => 1,
        None if year_index == 2 => 0,
        None => return Err(TimeError::KernelDateUnsupported),
    };
    if year_index == month_index {
        return Err(TimeError::KernelDateUnsupported);
    }
    let day_index = 3 - year_index - month_index;
    let year = expand_year(integer(date[year_index])?);
    let month = match named {
        Some((_, month)) => month,
        None => integer(date[month_index])?,
    };
    let day = libm::round(decimal(date[day_index])?);
    if day > i32::MAX as f64 {
        return Err(TimeError::KernelDateUnsupported);
    }
    from_calendar(year, month, day as i64, clock)
}

fn decimal(token: &str) -> Result<f64, TimeError> {
    if !token.as_bytes().first().is_some_and(u8::is_ascii_digit)
        || token.bytes().any(|c| !c.is_ascii_digit() && c != b'.')
        || token.bytes().filter(|c| *c == b'.').count() > 1
    {
        return Err(TimeError::KernelDateUnsupported);
    }
    let value = naif::parse_f64(token).ok_or(TimeError::KernelDateUnsupported)?;
    if !value.is_finite() {
        return Err(TimeError::KernelDateUnsupported);
    }
    Ok(value)
}

fn integer(token: &str) -> Result<i64, TimeError> {
    if token.is_empty() || !token.bytes().all(|c| c.is_ascii_digit()) {
        return Err(TimeError::KernelDateUnsupported);
    }
    let value = token
        .parse::<i64>()
        .map_err(|_| TimeError::KernelDateUnsupported)?;
    if value > i32::MAX as i64 {
        return Err(TimeError::KernelDateUnsupported);
    }
    Ok(value)
}

fn expand_year(year: i64) -> i64 {
    if year < 69 {
        2000 + year
    } else if year < 100 {
        1900 + year
    } else {
        year
    }
}

fn month(token: &str) -> Option<i64> {
    if token.len() < 3 {
        return None;
    }
    let token = token.to_ascii_uppercase();
    [
        "JANUARY",
        "FEBRUARY",
        "MARCH",
        "APRIL",
        "MAY",
        "JUNE",
        "JULY",
        "AUGUST",
        "SEPTEMBER",
        "OCTOBER",
        "NOVEMBER",
        "DECEMBER",
    ]
    .iter()
    .position(|name| name.starts_with(&token))
    .map(|index| index as i64 + 1)
}

fn clock(token: &str) -> Result<[f64; 3], TimeError> {
    let fields: Vec<_> = token.split(':').collect();
    if fields.is_empty() || fields.len() > 3 {
        return Err(TimeError::KernelDateUnsupported);
    }
    let mut result = [0.0; 3];
    for (index, field) in fields.iter().enumerate() {
        if index + 1 != fields.len() && field.contains('.') {
            return Err(TimeError::KernelDateUnsupported);
        }
        // TPARTV promotes integer tokens >= 1000 to year tokens before
        // resolving clocks. A decimal token remains a separate category.
        if integer(field).is_ok_and(|value| value >= 1000) {
            return Err(TimeError::KernelDateUnsupported);
        }
        result[index] = decimal(field)?;
    }
    Ok(result)
}

fn from_calendar(
    year: i64,
    month: i64,
    day: i64,
    clock: [f64; 3],
) -> Result<KernelDate, TimeError> {
    // N0067 TPARSE's Muller-Wimberly integer expression uses truncating
    // division. Preserve it even for out-of-range month/day fields.
    let days = year * 367
        - (year + (month + 9) / 12) * 7 / 4
        - ((year + (month - 9) / 7) / 100 + 1) * 3 / 4
        + month * 275 / 9
        + day
        - 730516;
    if days < i32::MIN as i64 || days > i32::MAX as i64 {
        return Err(TimeError::KernelDateUnsupported);
    }
    let seconds = (days as f64 - 0.5) * 86400.0 + clock[0] * 3600.0 + clock[1] * 60.0 + clock[2];
    calendar_result(seconds, days, clock, false)
}

fn calendar_result(
    seconds: f64,
    days: i64,
    clock: [f64; 3],
    julian: bool,
) -> Result<KernelDate, TimeError> {
    if !seconds.is_finite() {
        return Err(TimeError::KernelDateUnsupported);
    }
    let (days, hour, minute, second) = if julian {
        let midnight = seconds + 43200.0;
        let days = libm::floor(midnight / 86400.0);
        if days < i32::MIN as f64 || days > i32::MAX as f64 {
            return Err(TimeError::KernelDateUnsupported);
        }
        let remainder = midnight - days * 86400.0;
        let hour = libm::floor(remainder / 3600.0);
        let minute = libm::floor((remainder - hour * 3600.0) / 60.0);
        (
            days as i64,
            hour as u8,
            minute as u8,
            remainder - hour * 3600.0 - minute * 60.0,
        )
    } else {
        let hours = libm::floor(clock[0]);
        let minutes = clock[1] + (clock[0] - hours) * 60.0;
        let whole_minutes = libm::floor(minutes);
        let raw_seconds = clock[2] + (minutes - whole_minutes) * 60.0;
        let second_carry = libm::floor(raw_seconds / 60.0);
        let second = raw_seconds - second_carry * 60.0;
        let minutes = whole_minutes + second_carry;
        let hour_carry = libm::floor(minutes / 60.0);
        let minute = minutes - hour_carry * 60.0;
        let hours = hours + hour_carry;
        let day_carry = libm::floor(hours / 24.0);
        if day_carry > i32::MAX as f64 {
            return Err(TimeError::KernelDateUnsupported);
        }
        (
            days + day_carry as i64,
            (hours - day_carry * 24.0) as u8,
            minute as u8,
            second,
        )
    };
    let (year, month, day) = calendar::jdn_to_gregorian(J2000_JDN + days);
    Ok(KernelDate {
        seconds_past_j2000: seconds,
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
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
    fn free_text_and_duplicate_date_markers_are_rejected() {
        for text in ["@NOT_A_DATE", "@@2000-JAN-01"] {
            assert!(parse_kernel_date(text).is_err(), "{text}");
        }
    }
}
