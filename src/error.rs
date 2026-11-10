//! Error types shared by all modules.

use core::fmt;

/// Errors returned by `stkcraft-time`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeError {
    /// A calendar date is not valid for the selected calendar.
    InvalidDate { year: i32, month: u8, day: u8 },
    /// The time of day is outside its legal range.
    InvalidTime { hour: u8, minute: u8 },
    /// The seconds component is outside `0.0 .. 61.0`.
    InvalidSecond,
    /// A formatting pattern contains an unsupported token.
    InvalidFormat,
    /// An EOP table was empty.
    EopMissing,
    /// An IERS EOP file could not be parsed.
    EopParse,
    /// A spacecraft clock kernel could not be parsed.
    SclkParse,
    /// A required spacecraft clock kernel variable is missing.
    SclkMissing(&'static str),
    /// The spacecraft clock type is not supported.
    SclkUnsupported,
    /// The requested epoch lies outside the EOP table.
    Ut1OutOfRange,
    /// A UTC time that does not exist at a leap-second boundary:
    /// `23:59:60.x` on a day without a positive leap second, or
    /// `23:59:59.x` on a shortened (86399-second) day.
    InvalidLeapSecond,
    /// The requested UTC epoch lies before the first LSK entry.
    UtcOutOfRange,
    /// The LSK text could not be parsed.
    LskParse,
    /// A required LSK variable is missing.
    LskMissing(&'static str),
    /// An arithmetic operation overflowed the internal representation.
    Overflow,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::InvalidDate { year, month, day } => {
                write!(f, "invalid calendar date {year:04}-{month:02}-{day:02}")
            }
            TimeError::InvalidTime { hour, minute } => {
                write!(f, "invalid time of day {hour:02}:{minute:02}")
            }
            TimeError::InvalidSecond => write!(f, "invalid seconds component"),
            TimeError::InvalidFormat => write!(f, "unsupported formatting token"),
            TimeError::EopMissing => write!(f, "EOP table is empty"),
            TimeError::EopParse => write!(f, "could not parse the IERS EOP file"),
            TimeError::SclkParse => write!(f, "could not parse the spacecraft clock kernel"),
            TimeError::SclkMissing(name) => {
                write!(f, "spacecraft clock kernel is missing {name}")
            }
            TimeError::SclkUnsupported => {
                write!(
                    f,
                    "unsupported spacecraft clock type (only type 1 is supported)"
                )
            }
            TimeError::Ut1OutOfRange => {
                write!(f, "requested epoch lies outside the EOP table")
            }
            TimeError::InvalidLeapSecond => {
                write!(f, "UTC time does not exist at this leap-second boundary")
            }
            TimeError::UtcOutOfRange => {
                write!(f, "UTC epoch lies outside the supplied leap-second table")
            }
            TimeError::LskParse => write!(f, "could not parse the leap-second kernel"),
            TimeError::LskMissing(name) => write!(f, "leap-second kernel is missing {name}"),
            TimeError::Overflow => write!(f, "time arithmetic overflow"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TimeError {}
