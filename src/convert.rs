//! Conversion routines between the supported time scales.

use crate::calendar::{self, Calendar, GPS_EPOCH_JDN, J2000_JDN};
use crate::eop::EopTable;
use crate::epoch::{Epoch, UtcDateTime};
use crate::error::TimeError;
use crate::leap::{sofa_delta_at, LeapSecondSource, LeapSecondTable, JDN_1960, JDN_1972};
use crate::math;

/// `TAI - UTC` at the GPS epoch (`1980-01-06T00:00:00 UTC`).
const GPS_EPOCH_DELTA_AT: i64 = 19;

/// TAI seconds past the internal origin at the GPS epoch.
const GPS_EPOCH_TAI: i64 = (GPS_EPOCH_JDN - J2000_JDN) * 86_400 + GPS_EPOCH_DELTA_AT;

/// Converts a civil UTC representation to a continuous TAI epoch.
pub fn utc_to_tai(
    dt: &UtcDateTime,
    calendar: Calendar,
    lsk: &LeapSecondTable,
) -> Result<Epoch, TimeError> {
    if !(1..=12).contains(&dt.month) || dt.day < 1 {
        return Err(TimeError::InvalidDate {
            year: dt.year,
            month: dt.month,
            day: dt.day,
        });
    }

    let max_day =
        calendar::days_in_month(dt.year, dt.month, calendar).ok_or(TimeError::InvalidDate {
            year: dt.year,
            month: dt.month,
            day: dt.day,
        })?;
    if dt.day > max_day {
        return Err(TimeError::InvalidDate {
            year: dt.year,
            month: dt.month,
            day: dt.day,
        });
    }

    if dt.hour > 23 || dt.minute > 59 {
        return Err(TimeError::InvalidTime {
            hour: dt.hour,
            minute: dt.minute,
        });
    }
    if !(dt.second >= 0.0 && dt.second < 61.0) {
        return Err(TimeError::InvalidSecond);
    }

    let jdn = calendar::calendar_to_jdn(dt.year, dt.month, dt.day, calendar)?;

    if dt.second >= 60.0 {
        if jdn < JDN_1972 {
            return Err(TimeError::InvalidLeapSecond);
        }
        if dt.hour != 23 || dt.minute != 59 {
            return Err(TimeError::InvalidLeapSecond);
        }
        let current = lsk.delta_at_for_jdn(jdn).ok_or(TimeError::UtcOutOfRange)?;
        let next = lsk
            .delta_at_for_jdn(jdn + 1)
            .ok_or(TimeError::InvalidLeapSecond)?;
        if next <= current {
            return Err(TimeError::InvalidLeapSecond);
        }
    }

    // A negative leap second removes `23:59:59` from the end of the
    // preceding UTC day, which is then only 86399 seconds long. Accepting
    // that label would alias it with the following day's `00:00:00`.
    if jdn >= JDN_1972 && dt.hour == 23 && dt.minute == 59 && dt.second >= 59.0 {
        let current = lsk.delta_at_for_jdn(jdn).ok_or(TimeError::UtcOutOfRange)?;
        let next = lsk
            .delta_at_for_jdn(jdn + 1)
            .ok_or(TimeError::InvalidLeapSecond)?;
        if next < current {
            return Err(TimeError::InvalidLeapSecond);
        }
    }

    let whole_seconds = math::floor(dt.second);
    let time_of_day = dt.hour as i64 * 3_600 + dt.minute as i64 * 60 + whole_seconds as i64;
    let nominal = (jdn - J2000_JDN) * 86_400 + time_of_day;

    if jdn >= JDN_1972 {
        let delta_at = lsk.delta_at_for_jdn(jdn).ok_or(TimeError::UtcOutOfRange)?;
        let tai_sec = nominal + delta_at as i64;
        let nanos = math::round((dt.second - whole_seconds) * 1_000_000_000.0) as i64;
        return Ok(Epoch::from_parts(tai_sec, nanos));
    }

    let day_fraction = (time_of_day as f64 + (dt.second - whole_seconds)) / 86_400.0;
    let delta_at = lsk
        .delta_at_f64(jdn, day_fraction)
        .ok_or(TimeError::UtcOutOfRange)?;
    let delta_whole = math::floor(delta_at);
    let delta_fraction = delta_at - delta_whole;
    let tai_sec = nominal + delta_whole as i64;
    let nanos =
        math::round(((dt.second - whole_seconds) + delta_fraction) * 1_000_000_000.0) as i64;

    Ok(Epoch::from_parts(tai_sec, nanos))
}

/// Converts a continuous TAI epoch to a civil UTC representation.
pub fn tai_to_utc(
    epoch: Epoch,
    calendar: Calendar,
    lsk: &LeapSecondTable,
) -> Result<UtcDateTime, TimeError> {
    let tai_sec = epoch.sec();
    let nanos = epoch.nanos() as f64 * 1e-9;

    if tai_sec < lsk.first_tai_start() {
        return tai_to_utc_pre_1972(epoch, calendar, lsk);
    }

    let index = lsk
        .interval_index(tai_sec)
        .ok_or(TimeError::UtcOutOfRange)?;
    let entries = lsk.entries();
    let delta_at = entries[index].delta_at as i64;
    let nominal = tai_sec - delta_at;

    if let Some(next) = entries.get(index + 1) {
        if lsk.is_active_index(index + 1) {
            let next_nominal = lsk.nominal_sec(index + 1);
            if nominal >= next_nominal {
                let leap_len = (next.delta_at - entries[index].delta_at) as f64;
                let fraction = (nominal - next_nominal) as f64 + nanos;
                if fraction < leap_len {
                    let (year, month, day) = calendar::jdn_to_calendar(next.jdn - 1, calendar);
                    return Ok(UtcDateTime {
                        year,
                        month,
                        day,
                        hour: 23,
                        minute: 59,
                        second: 60.0 + fraction,
                    });
                }
            }
        }
    }

    let days = nominal.div_euclid(86_400);
    let time_of_day = nominal.rem_euclid(86_400);
    let jdn = days + J2000_JDN;
    let (year, month, day) = calendar::jdn_to_calendar(jdn, calendar);

    Ok(UtcDateTime {
        year,
        month,
        day,
        hour: (time_of_day / 3_600) as u8,
        minute: ((time_of_day % 3_600) / 60) as u8,
        second: (time_of_day % 60) as f64 + nanos,
    })
}

fn tai_to_utc_pre_1972(
    epoch: Epoch,
    calendar: Calendar,
    lsk: &LeapSecondTable,
) -> Result<UtcDateTime, TimeError> {
    if lsk.source() != LeapSecondSource::Sofa {
        return Err(TimeError::UtcOutOfRange);
    }

    let tai_value = epoch.tai_seconds();
    let mut utc_nominal = tai_value;

    for _ in 0..4 {
        let mjd = 51_544.0 + utc_nominal / 86_400.0;
        let delta = sofa_delta_at(mjd).ok_or(TimeError::UtcOutOfRange)?;
        let next = tai_value - delta;
        if math::abs(next - utc_nominal) < 1e-9 {
            utc_nominal = next;
            break;
        }
        utc_nominal = next;
    }

    let start_1960 = (JDN_1960 - J2000_JDN) as f64 * 86_400.0;
    if utc_nominal < start_1960 {
        return Err(TimeError::UtcOutOfRange);
    }

    let days = math::div_euclid(utc_nominal, 86_400.0);
    let time_of_day = math::rem_euclid(utc_nominal, 86_400.0);
    let jdn = days as i64 + J2000_JDN;
    let (year, month, day) = calendar::jdn_to_calendar(jdn, calendar);

    let hour = math::floor(time_of_day / 3_600.0) as u8;
    let rest = time_of_day - hour as f64 * 3_600.0;
    let minute = math::floor(rest / 60.0) as u8;
    let second = rest - minute as f64 * 60.0;

    Ok(UtcDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

/// Converts a TAI epoch to TDB seconds past the J2000 epoch (`ET`).
///
/// The periodic `TDB - TT` term depends on `ET`, so the fixed point is
/// solved by iteration. NAIF states that this model is accurate to about
/// 30 microseconds.
pub fn tai_to_et(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    let tai = epoch.tai_seconds();
    let base = tai + lsk.delta_t_a - 43_200.0;
    let mut et = base;

    for _ in 0..4 {
        let mean_anomaly = lsk.m0 + lsk.m1 * et;
        let eccentric_anomaly = mean_anomaly + lsk.eb * math::sin(mean_anomaly);
        let next = base + lsk.k * math::sin(eccentric_anomaly);
        if math::abs(next - et) < 1e-12 {
            return next;
        }
        et = next;
    }

    et
}

/// Converts TDB seconds past J2000 (`ET`) to a TAI epoch.
pub fn et_to_tai(et: f64, lsk: &LeapSecondTable) -> Epoch {
    let mean_anomaly = lsk.m0 + lsk.m1 * et;
    let eccentric_anomaly = mean_anomaly + lsk.eb * math::sin(mean_anomaly);
    let tai = et - lsk.delta_t_a + 43_200.0 - lsk.k * math::sin(eccentric_anomaly);
    Epoch::from_tai_seconds(tai)
}

/// Converts a TAI epoch to TT seconds past J2000 TT.
pub fn tai_to_tt(epoch: Epoch, lsk: &LeapSecondTable) -> f64 {
    epoch.tai_seconds() + lsk.delta_t_a - 43_200.0
}

/// Converts TT seconds past J2000 TT to a TAI epoch.
pub fn tt_to_tai(tt: f64, lsk: &LeapSecondTable) -> Epoch {
    Epoch::from_tai_seconds(tt - lsk.delta_t_a + 43_200.0)
}

/// Converts a TAI epoch to GPS seconds past the GPS epoch.
pub fn tai_to_gps(epoch: Epoch) -> f64 {
    epoch.tai_seconds() - GPS_EPOCH_TAI as f64
}

/// Converts GPS seconds past the GPS epoch to a TAI epoch.
pub fn gps_to_tai(gps: f64) -> Epoch {
    Epoch::from_tai_seconds(gps + GPS_EPOCH_TAI as f64)
}

/// Converts a civil UTC representation directly to `ET`.
pub fn utc_to_et(
    dt: &UtcDateTime,
    calendar: Calendar,
    lsk: &LeapSecondTable,
) -> Result<f64, TimeError> {
    Ok(tai_to_et(utc_to_tai(dt, calendar, lsk)?, lsk))
}

/// Converts `ET` directly to a civil UTC representation.
pub fn et_to_utc(
    et: f64,
    calendar: Calendar,
    lsk: &LeapSecondTable,
) -> Result<UtcDateTime, TimeError> {
    tai_to_utc(et_to_tai(et, lsk), calendar, lsk)
}

/// Modified Julian Date of the UTC day containing `jdn`.
fn utc_mjd(jdn: i64) -> f64 {
    51_544.0 + (jdn - J2000_JDN) as f64
}

/// Returns `TAI - UTC` in whole seconds in force at `epoch`.
fn delta_at_at_epoch(epoch: Epoch, lsk: &LeapSecondTable) -> Option<i32> {
    let index = lsk.interval_index(epoch.sec())?;
    Some(lsk.entries()[index].delta_at)
}

/// Returns the Julian Day Number of the UTC day containing `epoch`.
///
/// During a positive leap second the value stays on the day that owns
/// `23:59:60`, matching [`tai_to_utc`].
fn utc_jdn(epoch: Epoch, lsk: &LeapSecondTable) -> Option<i64> {
    let tai_sec = epoch.sec();
    if tai_sec < lsk.first_tai_start() {
        return None;
    }

    let index = lsk.interval_index(tai_sec)?;
    let entries = lsk.entries();
    let nominal = tai_sec - entries[index].delta_at as i64;

    if let Some(next) = entries.get(index + 1) {
        if lsk.is_active_index(index + 1) && nominal >= lsk.nominal_sec(index + 1) {
            return Some(next.jdn - 1);
        }
    }

    Some(nominal.div_euclid(86_400) + J2000_JDN)
}

/// TAI seconds past the internal origin at `00:00:00 UTC` of `jdn`.
fn tai_at_utc_day_start(jdn: i64, lsk: &LeapSecondTable) -> Option<i64> {
    let delta_at = lsk.delta_at_for_jdn(jdn)?;
    Some((jdn - J2000_JDN) * 86_400 + delta_at as i64)
}

/// Returns `UT1 - UTC` in seconds at `epoch`, using an IERS EOP table.
///
/// The EOP samples are converted to continuous `UT1 - TAI` using the
/// leap-second table before interpolation. This keeps a leap-second step
/// from being smeared when the table has missing days around the step.
///
/// Returns [`TimeError::Ut1OutOfRange`] when the epoch or the bracketing
/// EOP samples lie outside the table.
pub fn ut1_minus_utc(
    epoch: Epoch,
    lsk: &LeapSecondTable,
    eop: &EopTable,
) -> Result<f64, TimeError> {
    let jdn = utc_jdn(epoch, lsk).ok_or(TimeError::Ut1OutOfRange)?;
    let mjd_start = utc_mjd(jdn);
    let samples = eop.samples();

    if samples.len() < 2 {
        return Err(TimeError::Ut1OutOfRange);
    }

    let index = samples.partition_point(|sample| sample.mjd <= mjd_start);
    if index == 0 || index >= samples.len() {
        return Err(TimeError::Ut1OutOfRange);
    }

    let before = samples[index - 1];
    let after = samples[index];
    let before_jdn = mjd_to_jdn(before.mjd);
    let after_jdn = mjd_to_jdn(after.mjd);

    let before_tai = tai_at_utc_day_start(before_jdn, lsk).ok_or(TimeError::Ut1OutOfRange)? as f64;
    let after_tai = tai_at_utc_day_start(after_jdn, lsk).ok_or(TimeError::Ut1OutOfRange)? as f64;
    let before_delta = lsk
        .delta_at_for_jdn(before_jdn)
        .ok_or(TimeError::Ut1OutOfRange)? as f64;
    let after_delta = lsk
        .delta_at_for_jdn(after_jdn)
        .ok_or(TimeError::Ut1OutOfRange)? as f64;

    let before_ut1_tai = before.ut1_minus_utc - before_delta;
    let after_ut1_tai = after.ut1_minus_utc - after_delta;
    let span = after_tai - before_tai;
    let tai = epoch.tai_seconds();

    let ut1_tai = if span <= 0.0 {
        before_ut1_tai
    } else {
        let fraction = (tai - before_tai) / span;
        if !(0.0..=1.0).contains(&fraction) {
            return Err(TimeError::Ut1OutOfRange);
        }
        before_ut1_tai + fraction * (after_ut1_tai - before_ut1_tai)
    };

    let delta_at = delta_at_at_epoch(epoch, lsk).ok_or(TimeError::Ut1OutOfRange)? as f64;
    Ok(ut1_tai + delta_at)
}

/// Converts a TAI epoch to UT1 seconds past `2000-01-01T00:00:00 UT1`.
///
/// The result is the continuous UT1 time coordinate. It is defined as
///
/// ```text
/// UT1 = TAI + (UT1 - UTC) - (TAI - UTC)
/// ```
///
/// so it is zero at the UT1 instant labelled `2000-01-01T00:00:00`.
/// UT1 is an Earth-rotation scale; it is intentionally not folded into
/// ET or TDB.
pub fn tai_to_ut1(epoch: Epoch, lsk: &LeapSecondTable, eop: &EopTable) -> Result<f64, TimeError> {
    let delta_at = delta_at_at_epoch(epoch, lsk).ok_or(TimeError::Ut1OutOfRange)? as f64;
    let ut1_minus_tai = ut1_minus_utc(epoch, lsk, eop)? - delta_at;
    Ok(epoch.tai_seconds() + ut1_minus_tai)
}

/// Converts UT1 seconds past `2000-01-01T00:00:00 UT1` to a TAI epoch.
///
/// The inverse is a fixed-point solve, because `UT1 - UTC` depends on
/// the epoch itself. It converges in a few iterations because the offset
/// changes slowly.
pub fn ut1_to_tai(ut1: f64, lsk: &LeapSecondTable, eop: &EopTable) -> Result<Epoch, TimeError> {
    if !ut1.is_finite() {
        return Err(TimeError::Ut1OutOfRange);
    }

    let mut tai = initial_tai_guess(ut1, lsk, eop).ok_or(TimeError::Ut1OutOfRange)?;

    for _ in 0..10 {
        let epoch = Epoch::from_tai_seconds(tai);
        let delta_at = delta_at_at_epoch(epoch, lsk).ok_or(TimeError::Ut1OutOfRange)? as f64;
        let ut1_minus_tai = ut1_minus_utc(epoch, lsk, eop)? - delta_at;
        let next = ut1 - ut1_minus_tai;
        if math::abs(next - tai) < 1e-10 {
            tai = next;
            break;
        }
        tai = next;
    }

    let epoch = Epoch::from_tai_seconds(tai);
    let actual = tai_to_ut1(epoch, lsk, eop)?;
    if math::abs(actual - ut1) > 1e-6 {
        return Err(TimeError::Ut1OutOfRange);
    }

    Ok(epoch)
}

/// Clamps the first fixed-point guess to the EOP-supported TAI interval.
fn initial_tai_guess(ut1: f64, lsk: &LeapSecondTable, eop: &EopTable) -> Option<f64> {
    let (first_mjd, last_mjd) = eop.range();
    if !first_mjd.is_finite() || !last_mjd.is_finite() || first_mjd > last_mjd {
        return None;
    }

    let first_tai = tai_at_utc_day_start(mjd_to_jdn(first_mjd), lsk)? as f64;
    let last_tai = tai_at_utc_day_start(mjd_to_jdn(last_mjd), lsk)? as f64 + 86_400.0;
    Some((ut1 + 32.0).clamp(first_tai, last_tai))
}

/// Converts a daily EOP MJD to its Julian Day Number.
fn mjd_to_jdn(mjd: f64) -> i64 {
    J2000_JDN + math::round(mjd - 51_544.0) as i64
}

/// Converts a civil UTC representation to UT1 seconds past
/// `2000-01-01T00:00:00 UT1`.
pub fn utc_to_ut1(
    dt: &UtcDateTime,
    calendar: Calendar,
    lsk: &LeapSecondTable,
    eop: &EopTable,
) -> Result<f64, TimeError> {
    tai_to_ut1(utc_to_tai(dt, calendar, lsk)?, lsk, eop)
}

/// Converts UT1 seconds past `2000-01-01T00:00:00 UT1` to a civil UTC
/// representation.
pub fn ut1_to_utc(
    ut1: f64,
    calendar: Calendar,
    lsk: &LeapSecondTable,
    eop: &EopTable,
) -> Result<UtcDateTime, TimeError> {
    tai_to_utc(ut1_to_tai(ut1, lsk, eop)?, calendar, lsk)
}
