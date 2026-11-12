//! Nominal BeiDou Time (BDT), independent of the NAIF time-scale model.
//!
//! BDT counts continuous SI seconds from `2006-01-01T00:00:00 UTC`.
//! The nominal coordinate relation is `TAI - BDT = 33 seconds`. This is
//! not a model of measured BDT clock realization errors, broadcast UTC
//! corrections, satellite clock corrections or relativistic corrections.
//! Negative values extend the coordinate mathematically before its origin;
//! they do not assert that the operational system existed then.
//!
//! Integer parts retain nanoseconds exactly. Floating-point methods may
//! lose precision and do not imply nanosecond physical accuracy. Civil UTC
//! conversions use the supplied LSK and preserve its coverage and policies.
//! ET conversions delegate to the existing checked NAIF [`TimeContext`].
//!
//! References: BDS Signal In Space Interface Control Document, B1I (time
//! system); and the Beijing Satellite Navigation Center authors'
//! [ITU article on BDT and UTC](https://www.itu.int/hub/2023/06/the-beidou-navigation-satellite-system-and-the-utc-leap-second/).

use crate::contract::{ContractError, NaifEt, TimeContext};
use crate::{convert, Calendar, Epoch, LeapSecondTable, TimeError, UtcDateTime};

/// Nominal TAI minus BDT clock-coordinate offset, in seconds.
pub const TAI_MINUS_BDT_SECONDS: i64 = 33;
/// Nominal GPS minus BDT clock-coordinate offset, in seconds.
///
/// GPS elapsed seconds and BDT elapsed seconds have different origins:
/// their difference is `1356 * 604800 + 14`, not merely 14 seconds.
pub const GPS_MINUS_BDT_SECONDS: i64 = 14;
/// BDT origin as seconds since `2000-01-01T00:00:00 TAI`.
///
/// `2006-01-01T00:00:00 UTC` is `2006-01-01T00:00:33 TAI`.
pub const ORIGIN_TAI_SECONDS: i64 = 189_388_833;
/// Number of nanoseconds in one continuous seven-day BDT week.
pub const NANOSECONDS_PER_WEEK: u64 = 604_800_000_000_000;
const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// Exact nominal BDT elapsed time since `2006-01-01T00:00:00 UTC`.
///
/// Normalized parts use floor seconds and a remainder in `0..1_000_000_000`.
/// Conversion to [`Epoch`] can fail at the integer representation limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bdt {
    sec: i64,
    nanos: u32,
}

impl Bdt {
    /// Builds an exact value, normalizing signed nanoseconds; rejects overflow.
    pub fn checked_from_parts(sec: i64, nanos: i64) -> Option<Self> {
        Self::checked_from_nanoseconds(sec as i128 * NANOS_PER_SECOND + nanos as i128)
    }

    /// Builds a value from total elapsed nanoseconds; rejects overflow.
    pub fn checked_from_nanoseconds(total: i128) -> Option<Self> {
        let sec = total.div_euclid(NANOS_PER_SECOND);
        if sec < i64::MIN as i128 || sec > i64::MAX as i128 {
            return None;
        }
        Some(Self {
            sec: sec as i64,
            nanos: total.rem_euclid(NANOS_PER_SECOND) as u32,
        })
    }

    /// Builds a value from elapsed BDT seconds, rounding to the nearest nanosecond.
    ///
    /// Rejects non-finite and out-of-range input. Binary floating point may
    /// already have lost fractional information; use integer parts for exactness.
    pub fn checked_from_seconds(seconds: f64) -> Option<Self> {
        let epoch = Epoch::checked_from_tai_seconds(seconds)?;
        Some(Self {
            sec: epoch.sec(),
            nanos: epoch.nanos(),
        })
    }

    /// Returns floor elapsed seconds since the BDT origin.
    pub fn sec(self) -> i64 {
        self.sec
    }

    /// Returns the normalized nonnegative nanosecond remainder.
    pub fn nanos(self) -> u32 {
        self.nanos
    }

    /// Returns elapsed seconds as a potentially lossy floating-point value.
    pub fn seconds(self) -> f64 {
        self.sec as f64 + self.nanos as f64 * 1e-9
    }

    /// Returns exact total elapsed nanoseconds since the BDT origin.
    pub fn total_nanoseconds(self) -> i128 {
        self.sec as i128 * NANOS_PER_SECOND + self.nanos as i128
    }

    /// Converts an exact TAI epoch to nominal BDT, rejecting representation overflow.
    pub fn from_epoch(epoch: Epoch) -> Result<Self, TimeError> {
        let total = (epoch.sec() as i128 - ORIGIN_TAI_SECONDS as i128) * NANOS_PER_SECOND
            + epoch.nanos() as i128;
        Self::checked_from_nanoseconds(total).ok_or(TimeError::Overflow)
    }

    /// Converts nominal BDT to an exact TAI epoch, rejecting representation overflow.
    pub fn to_epoch(self) -> Result<Epoch, TimeError> {
        let sec = (self.sec as i128 + ORIGIN_TAI_SECONDS as i128)
            .try_into()
            .map_err(|_| TimeError::Overflow)?;
        Epoch::checked_from_parts(sec, self.nanos as i64).ok_or(TimeError::Overflow)
    }

    /// Builds a value from a full signed week and nanoseconds within that week.
    ///
    /// Weeks are not truncated broadcast week numbers and have no rollover.
    /// The remainder must be less than [`NANOSECONDS_PER_WEEK`]. Negative
    /// weeks follow Euclidean division: one nanosecond before origin is
    /// week -1, nanosecond `NANOSECONDS_PER_WEEK - 1`.
    pub fn from_week_nanoseconds(week: i64, nanos: u64) -> Result<Self, TimeError> {
        if nanos >= NANOSECONDS_PER_WEEK {
            return Err(TimeError::Overflow);
        }
        Self::checked_from_nanoseconds(week as i128 * NANOSECONDS_PER_WEEK as i128 + nanos as i128)
            .ok_or(TimeError::Overflow)
    }

    /// Returns a full signed week and its nonnegative nanosecond remainder.
    ///
    /// This uses Euclidean division and does not apply broadcast rollover.
    pub fn week_nanoseconds(self) -> (i64, u64) {
        let total = self.total_nanoseconds();
        (
            total.div_euclid(NANOSECONDS_PER_WEEK as i128) as i64,
            total.rem_euclid(NANOSECONDS_PER_WEEK as i128) as u64,
        )
    }

    /// Converts civil UTC through the supplied LSK and its coverage/policy.
    ///
    /// Integer-table conversions reject active adjacent kernel offsets whose
    /// difference cannot fit in `i32`. This is an arithmetic guard, not
    /// validation of physical clock behavior or leap-second plausibility.
    pub fn from_utc(
        utc: &UtcDateTime,
        calendar: Calendar,
        lsk: &LeapSecondTable,
    ) -> Result<Self, TimeError> {
        let day = crate::calendar::calendar_to_jdn(utc.year, utc.month, utc.day, calendar)?;
        if day >= crate::leap::JDN_1972 {
            check_active_kernel_jumps(lsk)?;
        }
        Self::from_epoch(convert::utc_to_tai(utc, calendar, lsk)?)
    }

    /// Converts to civil UTC through the supplied LSK and its coverage/policy.
    ///
    /// Returns [`TimeError::Overflow`] if the UTC year cannot fit in `i32`,
    /// or an active adjacent integer-table offset difference exceeds `i32`.
    /// The latter is an arithmetic guard, not a physical clock validation.
    pub fn to_utc(
        self,
        calendar: Calendar,
        lsk: &LeapSecondTable,
    ) -> Result<UtcDateTime, TimeError> {
        let epoch = self.to_epoch()?;
        if epoch.sec() >= lsk.first_tai_start() {
            check_active_kernel_jumps(lsk)?;
            // Determine the actual UTC day before the legacy calendar inverse
            // narrows its year to i32. This also keeps leap seconds on the day
            // that owns them. Failure here may be checked offset overflow;
            // it must not fall through into the historical conversion route.
            let day = convert::utc_jdn(epoch, lsk).ok_or(TimeError::Overflow)?;
            let minimum = crate::calendar::calendar_to_jdn(i32::MIN, 1, 1, calendar)?;
            let maximum = crate::calendar::calendar_to_jdn(i32::MAX, 12, 31, calendar)?;
            if day < minimum || day > maximum {
                return Err(TimeError::Overflow);
            }
        }
        // Pre-first-entry instants retain the existing SOFA historical path
        // and its coverage errors rather than requiring an integer UTC day.
        convert::tai_to_utc(epoch, calendar, lsk)
    }

    /// Converts checked NAIF ET through the existing context's TAI conversion.
    pub fn from_et(et: NaifEt, context: TimeContext<'_>) -> Result<Self, ContractError> {
        Ok(Self::from_epoch(context.to_epoch(et)?)?)
    }

    /// Converts to checked NAIF ET through the existing context.
    pub fn to_et(self, context: TimeContext<'_>) -> Result<NaifEt, ContractError> {
        context.to_et(self.to_epoch()?)
    }
}

// Legacy UTC inversion subtracts adjacent offsets as i32. Validate the
// active integer table before delegation; frozen entries and the separate
// historical SOFA route do not participate in that arithmetic.
fn check_active_kernel_jumps(lsk: &LeapSecondTable) -> Result<(), TimeError> {
    for (index, pair) in lsk.entries().windows(2).enumerate() {
        if lsk.is_active_index(index + 1) {
            pair[1]
                .delta_at
                .checked_sub(pair[0].delta_at)
                .ok_or(TimeError::Overflow)?;
        }
    }
    Ok(())
}
