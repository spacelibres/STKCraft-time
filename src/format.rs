//! Calendar string formatting.

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use crate::calendar::{self, Calendar, J2000_JDN};
use crate::epoch::UtcDateTime;
use crate::error::TimeError;
use crate::leap::LeapSecondTable;
use crate::math;

/// Maximum fractional digits: the internal representation is nanosecond
/// resolution, and larger values only allocate memory without adding data.
const MAX_PRECISION: usize = 9;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// Options for calendar formatting.
///
/// The leap-second table decides how long the day is when rounding
/// carries past its last second: a positive-leap-second day ends at
/// `23:59:60.999`, a normal day at `23:59:59.999`, and a day with a
/// negative leap second at `23:59:58.999`. The calendar decides how a
/// carry into the next day moves the date.
///
/// Without a table a day is assumed to be 86 400 s, unless the value
/// already carries `second >= 60`, in which case a 86 401-second day is
/// assumed so an actual leap second still prints.
#[derive(Clone, Copy, Debug)]
pub struct FormatOptions<'a> {
    precision: usize,
    calendar: Calendar,
    leap_seconds: Option<&'a LeapSecondTable>,
}

impl<'a> FormatOptions<'a> {
    /// Gregorian calendar, no leap-second table.
    pub fn new(precision: usize) -> Self {
        Self {
            precision,
            calendar: Calendar::Gregorian,
            leap_seconds: None,
        }
    }

    /// Selects the calendar used when rounding carries into the next day.
    pub fn with_calendar(mut self, calendar: Calendar) -> Self {
        self.calendar = calendar;
        self
    }

    /// Supplies the leap-second table used to determine the day length.
    pub fn with_leap_seconds(mut self, lsk: &'a LeapSecondTable) -> Self {
        self.leap_seconds = Some(lsk);
        self
    }

    /// Fractional digits, before clamping to nanoseconds.
    pub fn precision(&self) -> usize {
        self.precision
    }

    /// Calendar used for day carries.
    pub fn calendar(&self) -> Calendar {
        self.calendar
    }
}

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const MONTH_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAY_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

const WEEKDAY_LONG: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Formats a UTC representation as ISO 8601 with a trailing `Z`.
///
/// A day is assumed to be 86 400 s; use [`iso8601_with`] with a
/// leap-second table when a carry could land on a leap second.
pub fn iso8601(dt: &UtcDateTime, precision: usize) -> String {
    iso8601_with(dt, &FormatOptions::new(precision))
}

/// Formats a UTC representation as ISO 8601 using explicit options.
pub fn iso8601_with(dt: &UtcDateTime, options: &FormatOptions<'_>) -> String {
    let dt = normalize_datetime(dt, options);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{}Z",
        dt.year,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        pad_seconds(dt.second, options.precision)
    )
}

/// Formats a representation in the SPICE calendar style,
/// e.g. `2000 JAN 01 12:00:00.000`.
pub fn spice_calendar(dt: &UtcDateTime, precision: usize) -> String {
    spice_calendar_with(dt, &FormatOptions::new(precision))
}

/// Formats a representation in the SPICE calendar style using explicit
/// options.
pub fn spice_calendar_with(dt: &UtcDateTime, options: &FormatOptions<'_>) -> String {
    let dt = normalize_datetime(dt, options);
    let month = MONTHS[(dt.month.saturating_sub(1) as usize) % 12];
    format!(
        "{:04} {} {:02} {:02}:{:02}:{}",
        dt.year,
        month,
        dt.day,
        dt.hour,
        dt.minute,
        pad_seconds(dt.second, options.precision)
    )
}

/// Formats `ET` as a calendar TDB string. This is the `etcal` behaviour
/// and requires no leap-second kernel.
pub fn etcal(et: f64, precision: usize) -> String {
    if !et.is_finite() {
        return "INVALID ET".to_string();
    }

    // Split before adding an absolute Julian Date: adding a microsecond to
    // 2451545.0 would round it away at f64 resolution.
    let day_offset = math::floor(et / 86_400.0);
    if day_offset < i64::MIN as f64 || day_offset >= i64::MAX as f64 {
        return "INVALID ET".to_string();
    }
    let seconds = et - day_offset * 86_400.0;
    let jdn = J2000_JDN + day_offset as i64;
    let seconds_of_day = 43_200.0 + seconds;
    let (year, month, day, hour, minute, second) = calendar_from_jdn_seconds(jdn, seconds_of_day);

    spice_calendar(
        &UtcDateTime::new(year, month, day, hour, minute, second),
        precision,
    )
}

/// Decomposes a Julian Date into a Gregorian calendar representation.
pub fn jd_to_calendar(jd: f64) -> (i32, u8, u8, u8, u8, f64) {
    let shifted = jd + 0.5;
    let mut jdn = math::floor(shifted) as i64;
    let mut seconds = (shifted - jdn as f64) * 86_400.0;

    if seconds >= 86_400.0 {
        seconds -= 86_400.0;
        jdn += 1;
    } else if seconds < 0.0 {
        seconds += 86_400.0;
        jdn -= 1;
    }

    let (year, month, day) = calendar::jdn_to_gregorian(jdn);
    let hour = math::floor(seconds / 3_600.0) as u8;
    let rest = seconds - hour as f64 * 3_600.0;
    let minute = math::floor(rest / 60.0) as u8;
    let second = rest - minute as f64 * 60.0;

    (year, month, day, hour, minute, second)
}

/// Formats a UTC representation as RFC 3339 with an explicit `+00:00`
/// offset.
pub fn rfc3339(dt: &UtcDateTime, precision: usize) -> String {
    rfc3339_with(dt, &FormatOptions::new(precision))
}

/// Formats a UTC representation as RFC 3339 using explicit options.
pub fn rfc3339_with(dt: &UtcDateTime, options: &FormatOptions<'_>) -> String {
    let dt = normalize_datetime(dt, options);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{}+00:00",
        dt.year,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        pad_seconds(dt.second, options.precision)
    )
}

/// Formats a UTC representation as RFC 2822, e.g.
/// `Tue, 29 Feb 2000 14:57:29 +0000`.
pub fn rfc2822(dt: &UtcDateTime) -> String {
    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} +0000",
        WEEKDAY_SHORT[weekday_index(dt, Calendar::Gregorian)],
        dt.day,
        MONTH_SHORT[(dt.month.saturating_sub(1) as usize) % 12],
        dt.year,
        dt.hour,
        dt.minute,
        math::floor(dt.second) as u8
    )
}

/// A parsed C89 `strftime`-style format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Format {
    tokens: Vec<Token>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Literal(String),
    Year4,
    Year2,
    Month2,
    Day2,
    DaySpace,
    Hour2,
    Minute2,
    Second2,
    DayOfYear,
    WeekdayShort,
    WeekdayLong,
    MonthShort,
    MonthLong,
    Offset,
    Zone,
    Fraction,
    Percent,
}

impl Format {
    /// Parses a C89-style format string.
    ///
    /// Supported tokens: `%Y %y %m %d %e %H %M %S %j %a %A %b %B %z %Z %f %%`.
    pub fn parse(pattern: &str) -> Result<Self, TimeError> {
        let mut tokens = Vec::new();
        let mut literal = String::new();
        let mut chars = pattern.chars();

        while let Some(ch) = chars.next() {
            if ch != '%' {
                literal.push(ch);
                continue;
            }

            if !literal.is_empty() {
                tokens.push(Token::Literal(core::mem::take(&mut literal)));
            }

            let token = match chars.next() {
                Some('Y') => Token::Year4,
                Some('y') => Token::Year2,
                Some('m') => Token::Month2,
                Some('d') => Token::Day2,
                Some('e') => Token::DaySpace,
                Some('H') => Token::Hour2,
                Some('M') => Token::Minute2,
                Some('S') => Token::Second2,
                Some('j') => Token::DayOfYear,
                Some('a') => Token::WeekdayShort,
                Some('A') => Token::WeekdayLong,
                Some('b') => Token::MonthShort,
                Some('B') => Token::MonthLong,
                Some('z') => Token::Offset,
                Some('Z') => Token::Zone,
                Some('f') => Token::Fraction,
                Some('%') => Token::Percent,
                _ => return Err(TimeError::InvalidFormat),
            };
            tokens.push(token);
        }

        if !literal.is_empty() {
            tokens.push(Token::Literal(literal));
        }

        Ok(Self { tokens })
    }
}

/// A formatter borrowing a UTC representation and a parsed [`Format`].
pub struct Formatter<'a> {
    dt: &'a UtcDateTime,
    format: &'a Format,
    options: FormatOptions<'a>,
}

impl<'a> Formatter<'a> {
    /// Creates a formatter that assumes a 86 400-second Gregorian day.
    pub fn new(dt: &'a UtcDateTime, format: &'a Format) -> Self {
        Self {
            dt,
            format,
            options: FormatOptions::new(MAX_PRECISION),
        }
    }

    /// Creates a formatter with explicit calendar and leap-second options.
    pub fn with_options(
        dt: &'a UtcDateTime,
        format: &'a Format,
        options: &FormatOptions<'a>,
    ) -> Self {
        Self {
            dt,
            format,
            options: *options,
        }
    }
}

impl fmt::Display for Formatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let normalized = normalize_datetime(self.dt, &self.options);
        let dt = &normalized;
        let calendar = self.options.calendar;

        for token in &self.format.tokens {
            match token {
                Token::Literal(value) => f.write_str(value.as_str())?,
                Token::Year4 => write!(f, "{:04}", dt.year)?,
                Token::Year2 => write!(f, "{:02}", dt.year.rem_euclid(100))?,
                Token::Month2 => write!(f, "{:02}", dt.month)?,
                Token::Day2 => write!(f, "{:02}", dt.day)?,
                Token::DaySpace => write!(f, "{:2}", dt.day)?,
                Token::Hour2 => write!(f, "{:02}", dt.hour)?,
                Token::Minute2 => write!(f, "{:02}", dt.minute)?,
                Token::Second2 => write!(f, "{:02}", math::floor(dt.second) as u8)?,
                Token::DayOfYear => write!(f, "{:03}", day_of_year(dt, calendar))?,
                Token::WeekdayShort => f.write_str(WEEKDAY_SHORT[weekday_index(dt, calendar)])?,
                Token::WeekdayLong => f.write_str(WEEKDAY_LONG[weekday_index(dt, calendar)])?,
                Token::MonthShort => {
                    f.write_str(MONTH_SHORT[(dt.month.saturating_sub(1) as usize) % 12])?
                }
                Token::MonthLong => {
                    f.write_str(MONTH_LONG[(dt.month.saturating_sub(1) as usize) % 12])?
                }
                Token::Offset => f.write_str("+0000")?,
                Token::Zone => f.write_str("UTC")?,
                Token::Fraction => write!(f, "{:09}", fractional_nanos(dt))?,
                Token::Percent => f.write_str("%")?,
            }
        }

        Ok(())
    }
}

/// Formats a UTC representation with a C89-style pattern.
pub fn strftime(dt: &UtcDateTime, pattern: &str) -> Result<String, TimeError> {
    strftime_with(dt, pattern, &FormatOptions::new(MAX_PRECISION))
}

/// Formats a UTC representation with a C89-style pattern and explicit
/// options.
pub fn strftime_with(
    dt: &UtcDateTime,
    pattern: &str,
    options: &FormatOptions<'_>,
) -> Result<String, TimeError> {
    let format = Format::parse(pattern)?;
    Ok(Formatter::with_options(dt, &format, options).to_string())
}

fn weekday_index(dt: &UtcDateTime, calendar: Calendar) -> usize {
    let jdn = calendar::calendar_to_jdn(dt.year, dt.month, dt.day, calendar)
        .unwrap_or_else(|_| calendar::gregorian_to_jdn(dt.year, dt.month, dt.day));
    (jdn + 1).rem_euclid(7) as usize
}

fn day_of_year(dt: &UtcDateTime, calendar: Calendar) -> u16 {
    let jdn = calendar::calendar_to_jdn(dt.year, dt.month, dt.day, calendar)
        .unwrap_or_else(|_| calendar::gregorian_to_jdn(dt.year, dt.month, dt.day));
    let start = calendar::calendar_to_jdn(dt.year, 1, 1, calendar)
        .unwrap_or_else(|_| calendar::gregorian_to_jdn(dt.year, 1, 1));
    (jdn - start + 1).clamp(0, u16::MAX as i64) as u16
}

fn fractional_nanos(dt: &UtcDateTime) -> u32 {
    let fraction = dt.second - math::floor(dt.second);
    let nanos = math::round(fraction * 1_000_000_000.0) as i64;
    if nanos >= 1_000_000_000 {
        0
    } else {
        nanos.max(0) as u32
    }
}

fn clamp_precision(precision: usize) -> usize {
    precision.min(MAX_PRECISION)
}

fn round_seconds(second: f64, precision: usize) -> f64 {
    const POW10: [f64; 10] = [
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
    ];
    let factor = POW10[clamp_precision(precision)];
    math::round(second * factor) / factor
}

fn normalize_datetime(dt: &UtcDateTime, options: &FormatOptions<'_>) -> UtcDateTime {
    let second = round_seconds(dt.second, options.precision);
    let mut out = *dt;

    if second >= second_limit(dt, options) {
        out.second = 0.0;
        increment_minute(&mut out, options.calendar);
    } else {
        out.second = second;
    }

    out
}

/// Largest expressible second (exclusive) on the day of `dt`.
///
/// Rounding at or above this value belongs to the next minute, and on
/// `23:59` to the next day.
fn second_limit(dt: &UtcDateTime, options: &FormatOptions<'_>) -> f64 {
    if let Some(day_length) = day_length(dt, options) {
        if day_length >= SECONDS_PER_DAY + 1.0 {
            return 61.0;
        }
        if day_length <= SECONDS_PER_DAY - 1.0 {
            return 59.0;
        }
        return 60.0;
    }

    // Without a table, a value that already has `second >= 60` can only be
    // a leap second, so keep it rather than moving it to the next day.
    if dt.second >= 60.0 {
        61.0
    } else {
        60.0
    }
}

fn day_length(dt: &UtcDateTime, options: &FormatOptions<'_>) -> Option<f64> {
    let lsk = options.leap_seconds?;
    let jdn = calendar::calendar_to_jdn(dt.year, dt.month, dt.day, options.calendar).ok()?;
    let before = lsk.delta_at_for_jdn(jdn)? as f64;
    let after = lsk.delta_at_for_jdn(jdn + 1)? as f64;
    Some(SECONDS_PER_DAY + (after - before))
}

fn increment_minute(dt: &mut UtcDateTime, calendar: Calendar) {
    dt.minute += 1;
    if dt.minute >= 60 {
        dt.minute = 0;
        dt.hour += 1;
        if dt.hour >= 24 {
            dt.hour = 0;
            increment_day(dt, calendar);
        }
    }
}

fn increment_day(dt: &mut UtcDateTime, calendar: Calendar) {
    let Ok(jdn) = calendar::calendar_to_jdn(dt.year, dt.month, dt.day, calendar) else {
        return;
    };
    let (year, month, day) = calendar::jdn_to_calendar(jdn + 1, calendar);
    dt.year = year;
    dt.month = month;
    dt.day = day;
}

fn calendar_from_jdn_seconds(mut jdn: i64, mut seconds: f64) -> (i32, u8, u8, u8, u8, f64) {
    while seconds >= 86_400.0 {
        seconds -= 86_400.0;
        jdn += 1;
    }
    while seconds < 0.0 {
        seconds += 86_400.0;
        jdn -= 1;
    }

    let (year, month, day) = calendar::jdn_to_gregorian(jdn);
    let hour = math::floor(seconds / 3_600.0) as u8;
    let rest = seconds - hour as f64 * 3_600.0;
    let minute = math::floor(rest / 60.0) as u8;
    let second = rest - minute as f64 * 60.0;

    (year, month, day, hour, minute, second)
}

fn pad_seconds(second: f64, precision: usize) -> String {
    let precision = clamp_precision(precision);
    let formatted = format!("{:.*}", precision, second);
    if second < 10.0 {
        let mut out = String::with_capacity(formatted.len() + 1);
        out.push('0');
        out.push_str(&formatted);
        out
    } else {
        formatted
    }
}
