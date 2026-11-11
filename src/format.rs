//! Calendar string formatting.

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use crate::calendar::{self, J2000_JDN};
use crate::epoch::UtcDateTime;
use crate::error::TimeError;
use crate::math;

/// Maximum fractional digits: the internal representation is nanosecond
/// resolution, and larger values only allocate memory without adding data.
const MAX_PRECISION: usize = 9;

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
pub fn iso8601(dt: &UtcDateTime, precision: usize) -> String {
    let dt = normalize_datetime(dt, precision);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{}Z",
        dt.year,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        pad_seconds(dt.second, precision)
    )
}

/// Formats a representation in the SPICE calendar style,
/// e.g. `2000 JAN 01 12:00:00.000`.
pub fn spice_calendar(dt: &UtcDateTime, precision: usize) -> String {
    let dt = normalize_datetime(dt, precision);
    let month = MONTHS[(dt.month.saturating_sub(1) as usize) % 12];
    format!(
        "{:04} {} {:02} {:02}:{:02}:{}",
        dt.year,
        month,
        dt.day,
        dt.hour,
        dt.minute,
        pad_seconds(dt.second, precision)
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
    let dt = normalize_datetime(dt, precision);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{}+00:00",
        dt.year,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        pad_seconds(dt.second, precision)
    )
}

/// Formats a UTC representation as RFC 2822, e.g.
/// `Tue, 29 Feb 2000 14:57:29 +0000`.
pub fn rfc2822(dt: &UtcDateTime) -> String {
    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} +0000",
        WEEKDAY_SHORT[weekday_index(dt)],
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
}

impl<'a> Formatter<'a> {
    /// Creates a formatter.
    pub fn new(dt: &'a UtcDateTime, format: &'a Format) -> Self {
        Self { dt, format }
    }
}

impl fmt::Display for Formatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let normalized = normalize_datetime(self.dt, MAX_PRECISION);
        let dt = &normalized;

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
                Token::DayOfYear => write!(f, "{:03}", day_of_year(dt))?,
                Token::WeekdayShort => f.write_str(WEEKDAY_SHORT[weekday_index(dt)])?,
                Token::WeekdayLong => f.write_str(WEEKDAY_LONG[weekday_index(dt)])?,
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
    let format = Format::parse(pattern)?;
    Ok(Formatter::new(dt, &format).to_string())
}

fn weekday_index(dt: &UtcDateTime) -> usize {
    let jdn = calendar::gregorian_to_jdn(dt.year, dt.month, dt.day);
    (jdn + 1).rem_euclid(7) as usize
}

fn day_of_year(dt: &UtcDateTime) -> u16 {
    const CUMULATIVE: [u16; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let index = (dt.month as usize).min(12);
    let mut day = CUMULATIVE[index] + dt.day as u16;
    if calendar::is_gregorian_leap(dt.year) && dt.month > 2 {
        day += 1;
    }
    day
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

fn normalize_datetime(dt: &UtcDateTime, precision: usize) -> UtcDateTime {
    let mut out = *dt;
    let mut second = round_seconds(dt.second, precision);

    if second >= 60.0 {
        if dt.second >= 60.0 && second < 61.0 {
            // A leap second that still rounds inside 60.x is kept.
        } else {
            second = 0.0;
            increment_minute(&mut out);
        }
    }

    out.second = second;
    out
}

fn increment_minute(dt: &mut UtcDateTime) {
    dt.minute += 1;
    if dt.minute >= 60 {
        dt.minute = 0;
        dt.hour += 1;
        if dt.hour >= 24 {
            dt.hour = 0;
            increment_day(dt);
        }
    }
}

fn increment_day(dt: &mut UtcDateTime) {
    let jdn = calendar::gregorian_to_jdn(dt.year, dt.month, dt.day);
    let (year, month, day) = calendar::jdn_to_gregorian(jdn + 1);
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
