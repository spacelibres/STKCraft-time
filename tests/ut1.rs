//! UT1 and IERS Earth Orientation Parameter tests.
//!
//! These tests use a synthetic EOP table whose values are chosen to make
//! the leap-second step exactly `1.0 s`, so the interpolation behaviour
//! can be asserted to the microsecond. The parsing test embeds three
//! real lines from `finals2000A.all`, including a two-digit month/day
//! row that whitespace splitting would misalign.

use stkcraft_time::calendar::Calendar;
use stkcraft_time::convert::{
    tai_to_ut1, ut1_minus_utc, ut1_to_tai, ut1_to_utc, utc_to_tai, utc_to_ut1,
};
use stkcraft_time::eop::{EopSample, EopSeries, EopTable};
use stkcraft_time::{LeapSecondTable, TimeError, UtcDateTime};

/// Three real `finals2000A.all` rows.
///
/// The third row is `1973-01-10`. Its year, month and day are written as
/// `73 110`, so a whitespace split yields `["73", "110", "41692.00", ...]`
/// and every later column shifts by one. A correct parser must still
/// read `UT1 - UTC = 0.7829389`.
const FINALS_SNIPPET: &str = "\
73 1 2 41684.00 I  0.120733 0.009786  0.136966 0.015902  I 0.8084178 0.0002710  0.0000 0.1916  P    -0.766    0.199    -0.720    0.300
73 1 3 41685.00 I  0.118980 0.011039  0.135656 0.013616  I 0.8056163 0.0002710  3.5563 0.1916  P    -0.751    0.199    -0.701    0.300
73 110 41692.00 I  0.106616 0.011233  0.126675 0.007850  I 0.7829389 0.0002710  3.6924 0.2189  P    -0.822    0.199    -0.574    0.300
";

/// A table around the 2017-01-01 positive leap second.
///
/// MJD 57753 is 2016-12-31 and MJD 57754 is 2017-01-01. The values
/// differ by `1.0 s`, which is exactly the leap-second step; the residual
/// rotation-rate term is therefore zero.
fn leap_second_eop() -> EopTable {
    EopTable::new(vec![
        EopSample {
            mjd: 57_753.0,
            ut1_minus_utc: -0.4089,
            observed: true,
        },
        EopSample {
            mjd: 57_754.0,
            ut1_minus_utc: 0.5911,
            observed: true,
        },
        EopSample {
            mjd: 57_755.0,
            ut1_minus_utc: 0.5907,
            observed: true,
        },
    ])
    .unwrap()
}

fn uniform_eop() -> EopTable {
    EopTable::new(vec![
        EopSample {
            mjd: 51_544.0,
            ut1_minus_utc: -0.20,
            observed: true,
        },
        EopSample {
            mjd: 51_545.0,
            ut1_minus_utc: -0.21,
            observed: true,
        },
        EopSample {
            mjd: 51_546.0,
            ut1_minus_utc: -0.22,
            observed: true,
        },
    ])
    .unwrap()
}

#[test]
fn linear_interpolation_endpoints_and_midpoint() {
    let table = uniform_eop();
    assert_eq!(table.len(), 3);

    assert!((table.ut1_minus_utc(51_544.0).unwrap() + 0.20).abs() < 1e-15);
    assert!((table.ut1_minus_utc(51_545.5).unwrap() + 0.215).abs() < 1e-12);
    assert!((table.ut1_minus_utc(51_546.0).unwrap() + 0.22).abs() < 1e-15);
}

#[test]
fn out_of_range_lookup_is_an_error() {
    let table = uniform_eop();
    assert_eq!(table.ut1_minus_utc(51_543.0), Err(TimeError::Ut1OutOfRange));
    assert_eq!(table.ut1_minus_utc(51_547.0), Err(TimeError::Ut1OutOfRange));
}

#[test]
fn non_finite_samples_are_rejected() {
    let nan_mjd = EopTable::new(vec![EopSample {
        mjd: f64::NAN,
        ut1_minus_utc: 0.0,
        observed: true,
    }]);
    assert_eq!(nan_mjd.unwrap_err(), TimeError::EopParse);

    let infinite_ut1 = EopTable::new(vec![EopSample {
        mjd: 51_544.0,
        ut1_minus_utc: f64::INFINITY,
        observed: true,
    }]);
    assert_eq!(infinite_ut1.unwrap_err(), TimeError::EopParse);
}

#[test]
fn parser_rejects_non_finite_values() {
    let text = format!("{}\n", sample_line("NaN", "0.1", ""));
    assert_eq!(
        EopTable::from_finals2000a(&text).unwrap_err(),
        TimeError::EopParse
    );
}

#[test]
fn parses_fixed_columns_even_when_month_and_day_merge() {
    let table = EopTable::from_finals2000a(FINALS_SNIPPET).unwrap();
    assert_eq!(table.len(), 3);

    assert!((table.ut1_minus_utc(41_684.0).unwrap() - 0.808_417_8).abs() < 1e-12);
    assert!((table.ut1_minus_utc(41_685.0).unwrap() - 0.805_616_3).abs() < 1e-12);
    assert!((table.ut1_minus_utc(41_692.0).unwrap() - 0.782_938_9).abs() < 1e-12);

    assert!(table.samples().iter().all(|sample| sample.observed));
}

#[test]
fn leap_second_step_is_not_smeared() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let eop = leap_second_eop();

    let noon = UtcDateTime::new(2016, 12, 31, 12, 0, 0.0);
    let noon_tai = utc_to_tai(&noon, Calendar::Gregorian, &lsk).unwrap();
    let aware = ut1_minus_utc(noon_tai, &lsk, &eop).unwrap();

    // The leap-aware value stays on the pre-leap level all day.
    assert!((aware + 0.4089).abs() < 1e-6, "leap-aware value {aware}");

    // A naive raw interpolation would place the value halfway between
    // the two daily samples, which is off by half a second.
    let naive = eop.ut1_minus_utc(57_753.5).unwrap();
    assert!((naive - 0.0911).abs() < 1e-6, "naive value {naive}");
    assert!((naive - aware - 0.5).abs() < 1e-6);
}

#[test]
fn leap_second_instant_keeps_the_previous_value() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let eop = leap_second_eop();

    let leap = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
    let leap_tai = utc_to_tai(&leap, Calendar::Gregorian, &lsk).unwrap();
    let during = ut1_minus_utc(leap_tai, &lsk, &eop).unwrap();
    assert!(
        (during + 0.4089).abs() < 1e-6,
        "during leap second {during}"
    );

    let after = UtcDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let after_tai = utc_to_tai(&after, Calendar::Gregorian, &lsk).unwrap();
    let just_after = ut1_minus_utc(after_tai, &lsk, &eop).unwrap();
    assert!(
        (just_after - 0.5911).abs() < 1e-9,
        "just after {just_after}"
    );

    // The step is exactly one second at the day boundary.
    assert!((just_after - during - 1.0).abs() < 1e-5);
}

#[test]
fn ut1_round_trip_stays_within_one_nanosecond() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let eop = leap_second_eop();

    let dt = UtcDateTime::new(2016, 12, 31, 19, 30, 15.25);
    let tai = utc_to_tai(&dt, Calendar::Gregorian, &lsk).unwrap();

    let ut1 = tai_to_ut1(tai, &lsk, &eop).unwrap();
    let back = ut1_to_tai(ut1, &lsk, &eop).unwrap();
    assert!((back.tai_seconds() - tai.tai_seconds()).abs() < 2e-9);

    let dt_back = ut1_to_utc(ut1, Calendar::Gregorian, &lsk, &eop).unwrap();
    assert_eq!((dt_back.year, dt_back.month, dt_back.day), (2016, 12, 31));
    assert!((utc_to_ut1(&dt, Calendar::Gregorian, &lsk, &eop).unwrap() - ut1).abs() < 1e-9);
}

#[test]
fn epoch_outside_the_table_returns_out_of_range() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let eop = leap_second_eop();

    let dt = UtcDateTime::new(2020, 1, 1, 0, 0, 0.0);
    let tai = utc_to_tai(&dt, Calendar::Gregorian, &lsk).unwrap();

    assert_eq!(
        ut1_minus_utc(tai, &lsk, &eop),
        Err(TimeError::Ut1OutOfRange)
    );
    assert_eq!(tai_to_ut1(tai, &lsk, &eop), Err(TimeError::Ut1OutOfRange));
}

#[test]
fn observed_only_filters_predictions() {
    let table = EopTable::new(vec![
        EopSample {
            mjd: 1.0,
            ut1_minus_utc: 0.1,
            observed: true,
        },
        EopSample {
            mjd: 2.0,
            ut1_minus_utc: 0.2,
            observed: false,
        },
        EopSample {
            mjd: 3.0,
            ut1_minus_utc: 0.3,
            observed: true,
        },
    ])
    .unwrap();

    let observed = table.observed_only().unwrap();
    assert_eq!(observed.len(), 2);
    assert!(observed.samples().iter().all(|sample| sample.observed));
}

/// Builds a `finals2000A.all` line with the fields this parser reads.
fn sample_line(mjd: &str, bulletin_a: &str, bulletin_b: &str) -> String {
    let mut chars = vec![' '; 187];
    let fields = [
        (6usize, mjd),
        (57, "I"),
        (58, bulletin_a),
        (156, bulletin_b),
    ];
    for (start, text) in fields {
        for (offset, ch) in text.chars().enumerate() {
            chars[start + offset] = ch;
        }
    }
    chars.into_iter().collect()
}

#[test]
fn selects_bulletin_a_b_or_final() {
    let text = format!("{}\n", sample_line("42144.00", "0.4236357", ".4289000"));

    let a = EopTable::from_finals2000a_series(&text, EopSeries::BulletinA).unwrap();
    let b = EopTable::from_finals2000a_series(&text, EopSeries::BulletinB).unwrap();
    let final_preferred =
        EopTable::from_finals2000a_series(&text, EopSeries::FinalPreferred).unwrap();

    assert!((a.ut1_minus_utc(42_144.0).unwrap() - 0.423_635_7).abs() < 1e-12);
    assert!((b.ut1_minus_utc(42_144.0).unwrap() - 0.428_9).abs() < 1e-12);
    // Bulletin B is preferred when present.
    assert!((final_preferred.ut1_minus_utc(42_144.0).unwrap() - 0.428_9).abs() < 1e-12);
}

#[test]
fn final_preferred_falls_back_to_bulletin_a() {
    let text = format!("{}\n", sample_line("60000.00", "0.0836168", ""));
    let table = EopTable::from_finals2000a_series(&text, EopSeries::FinalPreferred).unwrap();
    assert!((table.ut1_minus_utc(60_000.0).unwrap() - 0.083_616_8).abs() < 1e-12);
}
