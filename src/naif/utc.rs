//! N0067 TTRANS TDB-to-JDUTC arithmetic, without an integer Epoch detour.
use crate::{contract::ContractError, math, LeapSecondTable};

const SECONDS_PER_DAY: f64 = 86400.0;
const HALF_DAY: f64 = 43200.0;
const DAY_NUMBER_2000: i32 = 730119;
const JULIAN_DAY_1_AD: f64 = 1721425.5;
const MAX_LEAP_ENTRIES: usize = 140;

pub(crate) fn julian_utc(et: f64, lsk: &LeapSecondTable) -> Result<f64, ContractError> {
    let count = lsk.entries().len().min(MAX_LEAP_ENTRIES);
    if count == 0 {
        return Err(ContractError::InvalidNaifLeapSeconds);
    }
    if [lsk.delta_t_a, lsk.k, lsk.eb, lsk.m0, lsk.m1]
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err(ContractError::NonFinite);
    }
    // GDPOOL in N0067 TTRANS requests 280 values. Preserve that native
    // selection before validating its paired TAI/day tables.
    let mut times = [0.0; MAX_LEAP_ENTRIES * 2];
    let mut days = [0_i32; MAX_LEAP_ENTRIES * 2];
    let mut previous_delta = lsk.entries()[0].delta_at as f64 - 1.0;
    for (index, entry) in lsk.entries().iter().take(count).enumerate() {
        let formal = (entry.jdn - crate::calendar::J2000_JDN) as f64 * SECONDS_PER_DAY - HALF_DAY;
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
    let tai = super::et_to_tt(et, lsk) - lsk.delta_t_a;
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
