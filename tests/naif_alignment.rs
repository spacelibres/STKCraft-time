//! Alignment tests against the NAIF/SPICE time model.

use stkcraft_time::calendar::{gregorian_to_jdn, julian_to_jdn};
use stkcraft_time::convert::{
    et_to_tai, et_to_utc, gps_to_tai, tai_to_et, tai_to_gps, tai_to_tt, tai_to_utc, tt_to_tai,
    utc_to_et, utc_to_tai,
};
use stkcraft_time::{Calendar, LeapEntry, LeapSecondTable, TimeError, UtcDateTime};

fn lsk() -> LeapSecondTable {
    LeapSecondTable::builtin().expect("bundled naif0012.tls must parse")
}

#[test]
fn lsk_constants_match_naif0012() {
    let table = lsk();
    assert!((table.delta_t_a - 32.184).abs() < 1e-12);
    assert!((table.k - 1.657e-3).abs() < 1e-15);
    assert!((table.eb - 1.671e-2).abs() < 1e-15);
    assert!((table.m0 - 6.239996).abs() < 1e-12);
    assert!((table.m1 - 1.99096871e-7).abs() < 1e-20);
}

#[test]
fn delta_at_matches_lsk_entries() {
    let table = lsk();
    assert_eq!(
        table.delta_at_for_jdn(gregorian_to_jdn(1972, 1, 1)),
        Some(10)
    );
    assert_eq!(
        table.delta_at_for_jdn(gregorian_to_jdn(2016, 12, 31)),
        Some(36)
    );
    assert_eq!(
        table.delta_at_for_jdn(gregorian_to_jdn(2017, 1, 1)),
        Some(37)
    );
    assert_eq!(table.delta_at_for_jdn(gregorian_to_jdn(1960, 1, 1)), None);
}

#[test]
fn leap_second_round_trip() {
    let table = lsk();

    for second in [59.0, 60.0, 60.5, 60.999_999] {
        let dt = UtcDateTime::new(2016, 12, 31, 23, 59, second);
        let tai = utc_to_tai(&dt, Calendar::Gregorian, &table).expect("valid UTC");
        let back = tai_to_utc(tai, Calendar::Gregorian, &table).expect("valid TAI");

        assert_eq!((back.year, back.month, back.day), (2016, 12, 31));
        assert_eq!((back.hour, back.minute), (23, 59));
        assert!(
            (back.second - second).abs() < 1e-6,
            "round trip {second} -> {}",
            back.second
        );
    }
}

#[test]
fn day_after_leap_second() {
    let table = lsk();
    let dt = UtcDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let tai = utc_to_tai(&dt, Calendar::Gregorian, &table).expect("valid UTC");
    let back = tai_to_utc(tai, Calendar::Gregorian, &table).expect("valid TAI");

    assert_eq!((back.year, back.month, back.day), (2017, 1, 1));
    assert_eq!((back.hour, back.minute), (0, 0));
    assert!(back.second.abs() < 1e-9);
}

#[test]
fn true_leap_second_is_rejected() {
    let table = lsk();
    let dt = UtcDateTime::new(2016, 12, 30, 23, 59, 60.0);
    assert_eq!(
        utc_to_tai(&dt, Calendar::Gregorian, &table),
        Err(TimeError::InvalidLeapSecond)
    );
}

/// A table with a negative leap second at the end of 2017: that UTC day
/// is 86399 seconds long and `23:59:59` does not exist.
fn negative_leap_table() -> LeapSecondTable {
    LeapSecondTable::new(vec![
        LeapEntry {
            jdn: gregorian_to_jdn(1972, 1, 1),
            delta_at: 10,
        },
        LeapEntry {
            jdn: gregorian_to_jdn(2017, 1, 1),
            delta_at: 36,
        },
        LeapEntry {
            jdn: gregorian_to_jdn(2018, 1, 1),
            delta_at: 35,
        },
    ])
    .expect("custom LSK must parse")
}

#[test]
fn negative_leap_second_removes_the_last_second_of_the_day() {
    let table = negative_leap_table();
    let day = (2017, 12, 31);

    for second in [59.0, 59.5, 59.999_999] {
        let dt = UtcDateTime::new(day.0, day.1, day.2, 23, 59, second);
        assert_eq!(
            utc_to_tai(&dt, Calendar::Gregorian, &table),
            Err(TimeError::InvalidLeapSecond),
            "23:59:{second} must not exist on a 86399-second day"
        );
    }

    // The last existing second is 23:59:58, exactly one second before
    // midnight; it must not alias the next day's 00:00:00.
    let last = UtcDateTime::new(day.0, day.1, day.2, 23, 59, 58.0);
    let last_tai = utc_to_tai(&last, Calendar::Gregorian, &table).expect("valid UTC");
    let midnight = UtcDateTime::new(2018, 1, 1, 0, 0, 0.0);
    let midnight_tai = utc_to_tai(&midnight, Calendar::Gregorian, &table).expect("valid UTC");

    assert!((midnight_tai.tai_seconds() - last_tai.tai_seconds() - 1.0).abs() < 1e-9);

    let back = tai_to_utc(last_tai, Calendar::Gregorian, &table).expect("valid TAI");
    assert_eq!((back.year, back.month, back.day), day);
    assert_eq!((back.hour, back.minute), (23, 59));
    assert!((back.second - 58.0).abs() < 1e-9);

    let midnight_back = tai_to_utc(midnight_tai, Calendar::Gregorian, &table).expect("valid TAI");
    assert_eq!(
        (midnight_back.year, midnight_back.month, midnight_back.day),
        (2018, 1, 1)
    );
}

#[test]
fn utc_before_lsk_is_rejected() {
    let table = lsk();
    let dt = UtcDateTime::new(1960, 1, 1, 0, 0, 0.0);
    assert_eq!(
        utc_to_tai(&dt, Calendar::Gregorian, &table),
        Err(TimeError::UtcOutOfRange)
    );
}

#[test]
fn et_at_2000_utc_noon_matches_spice() {
    let table = lsk();
    let dt = UtcDateTime::new(2000, 1, 1, 12, 0, 0.0);
    let et = utc_to_et(&dt, Calendar::Gregorian, &table).expect("valid UTC");

    // SPICE str2et("2000-01-01 12:00:00 UTC") is approximately 64.183927 s.
    assert!((et - 64.183_927).abs() < 1e-4, "ET = {et}");

    let back = et_to_utc(et, Calendar::Gregorian, &table).expect("valid ET");
    assert_eq!((back.year, back.month, back.day), (2000, 1, 1));
    assert_eq!((back.hour, back.minute), (12, 0));
    assert!(back.second.abs() < 1e-6);
}

#[test]
fn et_is_zero_at_j2000_tdb() {
    let table = lsk();
    let tai = et_to_tai(0.0, &table);
    let et = tai_to_et(tai, &table);
    assert!(et.abs() < 1e-9, "ET at J2000 is {et}");

    assert_eq!(
        stkcraft_time::format::etcal(0.0, 3),
        "2000 JAN 01 12:00:00.000"
    );
}

#[test]
fn et_round_trip() {
    let table = lsk();
    let et = 12_345.678_9;
    let tai = et_to_tai(et, &table);
    let back = tai_to_et(tai, &table);
    assert!((back - et).abs() < 1e-6, "ET round trip {back}");
}

#[test]
fn tt_round_trip() {
    let table = lsk();
    let tt = 12_345.678;
    let tai = tt_to_tai(tt, &table);
    let back = tai_to_tt(tai, &table);
    assert!((back - tt).abs() < 1e-9, "TT round trip {back}");
}

#[test]
fn tt_and_et_differ_by_the_periodic_term() {
    let table = lsk();
    let tai = tt_to_tai(0.0, &table);
    let et = tai_to_et(tai, &table);
    // The TT/TDB difference is at most about 1.7 ms.
    assert!((et - 0.0).abs() < 0.001_7, "ET = {et}");
}

#[test]
fn gps_epoch_round_trip() {
    let tai = gps_to_tai(0.0);
    assert!(tai_to_gps(tai).abs() < 1e-9);
}

#[test]
fn calendar_switch_is_consistent() {
    assert_eq!(gregorian_to_jdn(1582, 10, 15), 2_299_161);
    assert_eq!(julian_to_jdn(1582, 10, 5), 2_299_161);
    assert_eq!(gregorian_to_jdn(2000, 1, 1), 2_451_545);
}

#[test]
fn iso8601_formatting() {
    let dt = UtcDateTime::new(2000, 1, 1, 12, 0, 0.0);
    assert_eq!(
        stkcraft_time::format::iso8601(&dt, 3),
        "2000-01-01T12:00:00.000Z"
    );
}

#[test]
fn julian_date_helpers() {
    use stkcraft_time::calendar::{calendar_to_jd, jd_to_mjd, mjd_to_jd};

    let jd = calendar_to_jd(2000, 1, 1, 12, 0, 0.0, Calendar::Gregorian).expect("valid date");
    assert!((jd - 2_451_545.0).abs() < 1e-12);
    assert!((jd_to_mjd(jd) - 51_544.5).abs() < 1e-12);
    assert!((mjd_to_jd(51_544.5) - 2_451_545.0).abs() < 1e-12);
}
