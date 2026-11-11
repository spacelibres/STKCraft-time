//! Pre-1972 UTC drift model (SOFA `iauDat`) and the 1972 boundary.

use stkcraft_time::calendar::gregorian_to_jdn;
use stkcraft_time::convert::{tai_to_utc, utc_to_tai};
use stkcraft_time::leap::{sofa_delta_at, LeapSecondSource};
use stkcraft_time::{Calendar, LeapSecondTable, TimeError, UtcDateTime};

fn sofa() -> LeapSecondTable {
    LeapSecondTable::builtin()
        .expect("bundled naif0012.tls must parse")
        .with_source(LeapSecondSource::Sofa)
}

#[test]
fn sofa_drift_values() {
    // MJD 36934 = 1960-01-01
    let value = sofa_delta_at(36_934.0).expect("1960 is covered");
    assert!((value - 0.943_482).abs() < 1e-9, "1960 dAT = {value}");

    // MJD 37300 = 1961-01-01; the new row takes effect on that day.
    let value = sofa_delta_at(37_300.0).expect("1961 is covered");
    assert!((value - 1.422_818).abs() < 1e-9, "1961 dAT = {value}");

    // MJD 39887 = 1968-02-01
    let value = sofa_delta_at(39_887.0).expect("1968 is covered");
    let expected = 4.213_170 + (39_887.0 - 39_126.0) * 0.002_592;
    assert!((value - expected).abs() < 1e-9, "1968 dAT = {value}");

    // Before 1960 the SOFA model has no valid value.
    assert!(sofa_delta_at(36_000.0).is_none());
}

#[test]
fn integer_leap_seconds_take_over_at_1972() {
    let table = sofa();
    let jdn = gregorian_to_jdn(1972, 1, 1);
    assert_eq!(table.delta_at_f64(jdn, 0.0), Some(10.0));
}

#[test]
fn pre_1972_round_trip() {
    let table = sofa();

    let cases = [
        (1960, 1, 1, 0, 0, 0.0),
        (1961, 1, 1, 0, 0, 0.0),
        (1968, 2, 1, 12, 0, 0.0),
        (1971, 12, 31, 23, 59, 59.0),
    ];

    for (year, month, day, hour, minute, second) in cases {
        let dt = UtcDateTime::new(year, month, day, hour, minute, second);
        let tai = utc_to_tai(&dt, Calendar::Gregorian, &table).expect("valid UTC");
        let back = tai_to_utc(tai, Calendar::Gregorian, &table).expect("valid TAI");

        assert_eq!((back.year, back.month, back.day), (year, month, day));
        assert_eq!((back.hour, back.minute), (hour, minute));
        assert!(
            (back.second - second).abs() < 1e-6,
            "{year}-{month}-{day}: {} vs {second}",
            back.second
        );
    }
}

#[test]
fn iers_only_rejects_pre_1972() {
    let table = LeapSecondTable::builtin().expect("bundled LSK must parse");
    let dt = UtcDateTime::new(1960, 1, 1, 0, 0, 0.0);

    assert_eq!(
        utc_to_tai(&dt, Calendar::Gregorian, &table),
        Err(TimeError::UtcOutOfRange)
    );
}

#[test]
fn ordinary_pre_1972_day_has_no_leap_second() {
    let table = sofa();
    let dt = UtcDateTime::new(1960, 12, 30, 23, 59, 60.0);

    assert_eq!(
        utc_to_tai(&dt, Calendar::Gregorian, &table),
        Err(TimeError::InvalidLeapSecond)
    );
}

#[test]
fn boundary_1972_is_clean() {
    let table = sofa();
    let dt = UtcDateTime::new(1972, 1, 1, 0, 0, 0.0);

    let tai = utc_to_tai(&dt, Calendar::Gregorian, &table).expect("valid UTC");
    let back = tai_to_utc(tai, Calendar::Gregorian, &table).expect("valid TAI");

    assert_eq!((back.year, back.month, back.day), (1972, 1, 1));
    assert_eq!((back.hour, back.minute), (0, 0));
    assert!(back.second.abs() < 1e-9);
}

#[test]
fn historical_utc_matches_erfa_golden_vectors() {
    let table = sofa();
    for line in include_str!("golden/pre_1972.csv").lines() {
        if line.starts_with('#') || line.starts_with("year,") {
            continue;
        }
        let columns: Vec<_> = line.split(',').collect();
        let utc = UtcDateTime::new(
            columns[0].parse().unwrap(),
            columns[1].parse().unwrap(),
            columns[2].parse().unwrap(),
            columns[3].parse().unwrap(),
            columns[4].parse().unwrap(),
            columns[5].parse().unwrap(),
        );
        let actual = utc_to_tai(&utc, Calendar::Gregorian, &table);
        if columns[6] == "0" {
            assert!(
                matches!(actual, Err(TimeError::InvalidLeapSecond)),
                "{line}: {actual:?}"
            );
            continue;
        }
        let expected = stkcraft_time::Epoch::from_parts(
            columns[7].parse().unwrap(),
            columns[8].parse().unwrap(),
        );
        let actual = actual.unwrap_or_else(|err| panic!("{line}: {err:?}"));
        assert!(
            (actual - expected).total_nanoseconds().abs() <= 2,
            "{line}: {actual:?} vs {expected:?}"
        );
        let back = tai_to_utc(expected, Calendar::Gregorian, &table).unwrap();
        assert_eq!(
            (back.year, back.month, back.day, back.hour, back.minute),
            (utc.year, utc.month, utc.day, utc.hour, utc.minute),
            "{line}: {back:?}"
        );
        assert!((back.second - utc.second).abs() <= 2e-9, "{line}: {back:?}");
    }
}

#[test]
fn historical_utc_round_trip_preserves_nanoseconds() {
    let table = sofa();
    for second in [0.000000001, 0.123456789, 59.987654321] {
        for (year, month, day, hour, minute) in [
            (1960, 1, 1, 0, 0),
            (1961, 7, 31, 12, 34),
            (1968, 1, 31, 0, 0),
            (1971, 12, 31, 23, 59),
        ] {
            let utc = UtcDateTime::new(year, month, day, hour, minute, second);
            let epoch = utc_to_tai(&utc, Calendar::Gregorian, &table).unwrap();
            let back = tai_to_utc(epoch, Calendar::Gregorian, &table).unwrap();
            assert!((back.second - second).abs() <= 1e-9, "{utc:?} -> {back:?}");
            assert_eq!(
                utc_to_tai(&back, Calendar::Gregorian, &table).unwrap(),
                epoch
            );
        }
    }
}

#[test]
fn historical_mini_leaps_are_used_when_rounding_calendar_output() {
    use stkcraft_time::format::{iso8601_with, FormatOptions};
    let table = sofa();
    let options = FormatOptions::new(2).with_leap_seconds(&table);
    assert_eq!(
        iso8601_with(&UtcDateTime::new(1971, 12, 31, 23, 59, 59.999), &options),
        "1971-12-31T23:59:60.00Z"
    );
    assert_eq!(
        iso8601_with(&UtcDateTime::new(1961, 7, 31, 23, 59, 59.949), &options),
        "1961-08-01T00:00:00.00Z"
    );
    assert_eq!(
        iso8601_with(&UtcDateTime::new(1968, 1, 31, 23, 59, 59.899), &options),
        "1968-02-01T00:00:00.00Z"
    );
}

#[test]
fn historical_inverse_never_returns_a_removed_label_at_midnight() {
    let table = sofa();
    for (year, month, day) in [(1961, 8, 1), (1968, 2, 1), (1972, 1, 1)] {
        let midnight = utc_to_tai(
            &UtcDateTime::new(year, month, day, 0, 0, 0.0),
            Calendar::Gregorian,
            &table,
        )
        .unwrap();
        for offset in -10..=10 {
            let epoch = midnight + stkcraft_time::Duration::from_parts(0, offset);
            if let Ok(utc) = tai_to_utc(epoch, Calendar::Gregorian, &table) {
                assert!(
                    utc_to_tai(&utc, Calendar::Gregorian, &table).is_ok(),
                    "{epoch:?}: inverse returned removed label {utc:?}"
                );
                if offset >= 0 {
                    assert_eq!((utc.year, utc.month, utc.day), (year, month, day));
                }
            }
        }
    }
}

#[test]
fn historical_formatting_carries_at_rounded_mini_leap_endpoint() {
    use stkcraft_time::format::{iso8601_with, FormatOptions};
    let table = sofa();
    let options = FormatOptions::new(9).with_leap_seconds(&table);
    for (year, month, day, second, expected) in [
        (
            1960,
            12,
            31,
            60.0049999996,
            "1961-01-01T00:00:00.000000000Z",
        ),
        (
            1963,
            10,
            31,
            60.0999999996,
            "1963-11-01T00:00:00.000000000Z",
        ),
        (1964, 3, 31, 60.0999999996, "1964-04-01T00:00:00.000000000Z"),
    ] {
        let utc = UtcDateTime::new(year, month, day, 23, 59, second);
        assert!(utc_to_tai(&utc, Calendar::Gregorian, &table).is_ok());
        assert_eq!(iso8601_with(&utc, &options), expected);
    }
}
