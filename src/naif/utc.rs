//! N0067 TTRANS TDB-to-JDUTC arithmetic, without an integer Epoch detour.
use crate::{contract::ContractError, math, LeapEntry};

const SECONDS_PER_DAY: f64 = 86400.0;
const HALF_DAY: f64 = 43200.0;
const DAY_NUMBER_2000: i32 = 730119;
const JULIAN_DAY_1_AD: f64 = 1721425.5;
const MAX_LEAP_ENTRIES: usize = 140;

pub(crate) fn julian_utc(
    et: f64,
    entries: &[LeapEntry],
    [delta_t_a, k, eb, m0, m1]: [f64; 5],
) -> Result<f64, ContractError> {
    let (mut day, mut seconds) = day_seconds(et, entries, [delta_t_a, k, eb, m0, m1])?;
    // A formal JDUTC stands still within a positive leap second in
    // TTRANS. The comparison is strictly greater than, not >=.
    if seconds > SECONDS_PER_DAY {
        day = day.checked_add(1).ok_or(ContractError::EpochOutOfRange)?;
        seconds = 0.0;
    }
    let result = JULIAN_DAY_1_AD + day as f64 + seconds / SECONDS_PER_DAY;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(ContractError::NonFinite)
    }
}

/// TTRANS non-formal calendar day number and seconds since midnight.
pub(crate) fn day_seconds(
    et: f64,
    entries: &[LeapEntry],
    [delta_t_a, k, eb, m0, m1]: [f64; 5],
) -> Result<(i32, f64), ContractError> {
    let tdt = super::et_to_tt_parts(et, [k, eb, m0, m1]);
    day_seconds_from_tdt(tdt, entries, [delta_t_a, k, eb, m0, m1])
}

pub(crate) fn day_seconds_from_tdt(
    tdt: f64,
    entries: &[LeapEntry],
    [delta_t_a, k, eb, m0, m1]: [f64; 5],
) -> Result<(i32, f64), ContractError> {
    let count = entries.len().min(MAX_LEAP_ENTRIES);
    if count == 0 {
        return Err(ContractError::InvalidNaifLeapSeconds);
    }
    if [delta_t_a, k, eb, m0, m1]
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err(ContractError::NonFinite);
    }
    // GDPOOL in N0067 TTRANS requests 280 values. Preserve that native
    // selection before validating its paired TAI/day tables.
    let mut times = [0.0; MAX_LEAP_ENTRIES * 2];
    let mut days = [0_i32; MAX_LEAP_ENTRIES * 2];
    let mut previous_delta = entries[0].delta_at as f64 - 1.0;
    for (index, entry) in entries.iter().take(count).enumerate() {
        let elapsed_days = entry
            .jdn
            .checked_sub(crate::calendar::J2000_JDN)
            .ok_or(ContractError::EpochOutOfRange)?;
        let formal = elapsed_days as f64 * SECONDS_PER_DAY - HALF_DAY;
        let delta = entry.delta_at as f64;
        times[2 * index] = formal - SECONDS_PER_DAY + previous_delta;
        times[2 * index + 1] = formal + delta;
        let day = integer((formal + HALF_DAY) / SECONDS_PER_DAY)?
            .checked_add(DAY_NUMBER_2000)
            .ok_or(ContractError::EpochOutOfRange)?;
        days[2 * index] = day.checked_sub(1).ok_or(ContractError::EpochOutOfRange)?;
        days[2 * index + 1] = day;
        previous_delta = delta;
    }
    let times = &times[..count * 2];
    let days = &days[..count * 2];
    if times.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ContractError::InvalidNaifLeapSeconds);
    }
    let tai = tdt - delta_t_a;
    if !tai.is_finite() {
        return Err(ContractError::NonFinite);
    }
    let (mut day, mut seconds) = locate(tai, times, days)?;
    // Native TTRANS re-enters TAI near a day boundary. Reconstructing
    // this value can round; do not remove this apparently redundant path.
    if seconds > SECONDS_PER_DAY - 1.0 || seconds < 0.0 {
        let anchor = days.partition_point(|value| *value <= day).max(1) - 1;
        let elapsed_days = day
            .checked_sub(days[anchor])
            .ok_or(ContractError::EpochOutOfRange)?;
        seconds += elapsed_days as f64 * SECONDS_PER_DAY;
        let tai = times[anchor] + seconds;
        (day, seconds) = locate(tai, times, days)?;
    }
    Ok((day, seconds))
}

fn locate(tai: f64, times: &[f64], days: &[i32]) -> Result<(i32, f64), ContractError> {
    let position = times.partition_point(|value| *value <= tai);
    if position % 2 == 1 {
        Ok((days[position - 1], tai - times[position - 1]))
    } else {
        let anchor = position.max(1) - 1;
        let (quotient, remainder) = remainder(tai - times[anchor]);
        let day = days[anchor]
            .checked_add(integer(quotient)?)
            .ok_or(ContractError::EpochOutOfRange)?;
        Ok((day, remainder))
    }
}

/// Reverse the TTRANS YMD-to-TDT path for a validated native UTC table.
pub(crate) fn tdt_from_day_seconds(
    mut day: i32,
    mut seconds: f64,
    entries: &[LeapEntry],
    delta_t_a: f64,
) -> Result<f64, ContractError> {
    let count = entries.len().min(MAX_LEAP_ENTRIES);
    if count == 0 {
        return Err(ContractError::InvalidNaifLeapSeconds);
    }
    let mut times = [0.0; MAX_LEAP_ENTRIES * 2];
    let mut days = [0_i32; MAX_LEAP_ENTRIES * 2];
    let mut previous_delta = entries[0].delta_at as f64 - 1.0;
    for (index, entry) in entries.iter().take(count).enumerate() {
        let elapsed = entry
            .jdn
            .checked_sub(crate::calendar::J2000_JDN)
            .ok_or(ContractError::EpochOutOfRange)?;
        let formal = elapsed as f64 * SECONDS_PER_DAY - HALF_DAY;
        let delta = entry.delta_at as f64;
        times[2 * index] = formal - SECONDS_PER_DAY + previous_delta;
        times[2 * index + 1] = formal + delta;
        let reference_day = integer((formal + HALF_DAY) / SECONDS_PER_DAY)?
            .checked_add(DAY_NUMBER_2000)
            .ok_or(ContractError::EpochOutOfRange)?;
        days[2 * index] = reference_day
            .checked_sub(1)
            .ok_or(ContractError::EpochOutOfRange)?;
        days[2 * index + 1] = reference_day;
        previous_delta = delta;
    }
    let times = &times[..count * 2];
    let days = &days[..count * 2];
    if times.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ContractError::InvalidNaifLeapSeconds);
    }
    if seconds > SECONDS_PER_DAY - 1.0 || seconds < 0.0 {
        let anchor = days.partition_point(|value| *value <= day).max(1) - 1;
        let elapsed = day
            .checked_sub(days[anchor])
            .ok_or(ContractError::EpochOutOfRange)?;
        seconds += elapsed as f64 * SECONDS_PER_DAY;
        (day, seconds) = locate(times[anchor] + seconds, times, days)?;
    }
    let anchor = days.partition_point(|value| *value <= day).max(1) - 1;
    let elapsed = day
        .checked_sub(days[anchor])
        .ok_or(ContractError::EpochOutOfRange)?;
    seconds += elapsed as f64 * SECONDS_PER_DAY;
    Ok((times[anchor] + seconds) + delta_t_a)
}

fn remainder(value: f64) -> (f64, f64) {
    // RMAIND uses f2c D_INT (truncate toward zero), followed by a
    // negative-remainder correction. A floating remainder operator or
    // an initial floor can change boundary rounding.
    let ratio = value / SECONDS_PER_DAY;
    let mut quotient = if ratio > 0.0 {
        math::floor(ratio)
    } else {
        -math::floor(-ratio)
    };
    let mut remainder = value - quotient * SECONDS_PER_DAY;
    if remainder < 0.0 {
        quotient += -1.0;
        remainder += SECONDS_PER_DAY;
    }
    (quotient, remainder)
}

fn integer(value: f64) -> Result<i32, ContractError> {
    if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
        Err(ContractError::EpochOutOfRange)
    } else {
        Ok(value as i32)
    }
}
