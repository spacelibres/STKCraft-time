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
fn pre_1972_has_no_leap_second() {
    let table = sofa();
    let dt = UtcDateTime::new(1960, 12, 31, 23, 59, 60.0);

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
