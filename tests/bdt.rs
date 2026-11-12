use stkcraft_time::{
    bdt::*, contract::TimeContext, convert, Bdt, Calendar, Epoch, LeapSecondTable, TimeError,
    UtcDateTime,
};

#[test]
fn independent_erfa_utc_tai_vectors_match_exactly() {
    // Reproduce with tools/gen_bdt_golden.py, independently of this crate.
    let lsk = LeapSecondTable::builtin().unwrap();
    let mut count = 0;
    for line in include_str!("golden/bdt.csv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .skip(1)
    {
        let fields: Vec<&str> = line.split(',').collect();
        let utc = UtcDateTime::new(
            fields[0].parse().unwrap(),
            fields[1].parse().unwrap(),
            fields[2].parse().unwrap(),
            fields[3].parse().unwrap(),
            fields[4].parse().unwrap(),
            fields[5].parse().unwrap(),
        );
        let expected =
            Bdt::checked_from_parts(fields[6].parse().unwrap(), fields[7].parse().unwrap())
                .unwrap();
        assert_eq!(
            Bdt::from_utc(&utc, Calendar::Gregorian, &lsk).unwrap(),
            expected,
            "{line}"
        );
        assert_eq!(
            expected.to_utc(Calendar::Gregorian, &lsk).unwrap(),
            utc,
            "{line}"
        );
        count += 1;
    }
    assert_eq!(count, 28);
}

#[test]
fn origin_and_fractional_parts_are_exact() {
    let zero = Bdt::checked_from_parts(0, 0).unwrap();
    assert_eq!(zero.to_epoch().unwrap(), Epoch::from_parts(189_388_833, 0));
    assert_eq!(Bdt::from_epoch(zero.to_epoch().unwrap()).unwrap(), zero);
    let negative = Bdt::checked_from_parts(0, -1).unwrap();
    assert_eq!((negative.sec(), negative.nanos()), (-1, 999_999_999));
    assert_eq!(negative.total_nanoseconds(), -1);
    assert_eq!(
        negative.to_epoch().unwrap(),
        Epoch::from_parts(189_388_832, 999_999_999)
    );
    assert_eq!(Bdt::checked_from_nanoseconds(-1), Some(negative));
}

#[test]
fn checked_ranges_reject_unrepresentable_values() {
    assert!(Bdt::checked_from_parts(i64::MAX, 1_000_000_000).is_none());
    assert!(Bdt::checked_from_parts(i64::MIN, -1).is_none());
    assert!(Bdt::checked_from_nanoseconds(i128::MAX).is_none());
    assert!(Bdt::checked_from_nanoseconds(i128::MIN).is_none());
    assert!(matches!(
        Bdt::checked_from_parts(i64::MAX, 0).unwrap().to_epoch(),
        Err(TimeError::Overflow)
    ));
    assert!(matches!(
        Bdt::from_epoch(Epoch::from_parts(i64::MIN, 0)),
        Err(TimeError::Overflow)
    ));
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        i64::MAX as f64,
        i64::MIN as f64 - 2048.0,
    ] {
        assert!(Bdt::checked_from_seconds(value).is_none());
    }
    let half = Bdt::checked_from_seconds(-0.5).unwrap();
    assert_eq!(half.total_nanoseconds(), -500_000_000);
    assert_eq!(half.seconds(), -0.5);
    assert_eq!(Bdt::checked_from_seconds(0.999_999_999_6).unwrap().sec(), 1);
}

#[test]
fn week_coordinates_use_euclidean_unbounded_weeks() {
    let before = Bdt::checked_from_nanoseconds(-1).unwrap();
    assert_eq!(before.week_nanoseconds(), (-1, NANOSECONDS_PER_WEEK - 1));
    assert_eq!(
        Bdt::from_week_nanoseconds(-1, NANOSECONDS_PER_WEEK - 1).unwrap(),
        before
    );
    assert_eq!(
        Bdt::from_week_nanoseconds(8192, 0)
            .unwrap()
            .week_nanoseconds(),
        (8192, 0)
    );
    assert!(Bdt::from_week_nanoseconds(0, NANOSECONDS_PER_WEEK).is_err());
    assert!(Bdt::from_week_nanoseconds(i64::MAX, 0).is_err());
    assert!(Bdt::from_week_nanoseconds(i64::MIN, 0).is_err());
}

#[test]
fn utc_leap_seconds_follow_the_supplied_kernel() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let origin = UtcDateTime::new(2006, 1, 1, 0, 0, 0.0);
    assert_eq!(
        Bdt::from_utc(&origin, Calendar::Gregorian, &lsk)
            .unwrap()
            .sec(),
        0
    );
    let mut previous = None;
    for utc in [
        UtcDateTime::new(2016, 12, 31, 23, 59, 59.0),
        UtcDateTime::new(2016, 12, 31, 23, 59, 60.0),
        UtcDateTime::new(2017, 1, 1, 0, 0, 0.0),
    ] {
        let bdt = Bdt::from_utc(&utc, Calendar::Gregorian, &lsk).unwrap();
        assert_eq!(bdt.to_utc(Calendar::Gregorian, &lsk).unwrap(), utc);
        if let Some(prev) = previous {
            assert_eq!(bdt.total_nanoseconds() - prev, 1_000_000_000);
        }
        previous = Some(bdt.total_nanoseconds());
    }
    let invalid = UtcDateTime::new(2017, 1, 2, 23, 59, 60.0);
    assert!(Bdt::from_utc(&invalid, Calendar::Gregorian, &lsk).is_err());
    assert!(Bdt::from_utc(
        &UtcDateTime::new(1900, 1, 1, 0, 0, 0.0),
        Calendar::Gregorian,
        &lsk
    )
    .is_err());
}

#[test]
fn et_delegates_to_the_existing_naif_contract() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let context = TimeContext::new(&lsk);
    let value = Bdt::checked_from_parts(123_456_789, 123_456_789).unwrap();
    let et = value.to_et(context).unwrap();
    assert_eq!(et, context.to_et(value.to_epoch().unwrap()).unwrap());
    assert_eq!(
        Bdt::from_et(et, context).unwrap(),
        Bdt::from_epoch(context.to_epoch(et).unwrap()).unwrap()
    );
    let utc = UtcDateTime::new(2020, 1, 1, 0, 0, 0.0);
    let tai = convert::utc_to_tai(&utc, Calendar::Gregorian, &lsk).unwrap();
    let bdt = Bdt::from_epoch(tai).unwrap();
    // The different GPS and BDT origins contribute 1356 weeks in addition to 14 seconds.
    assert_eq!(
        convert::tai_to_gps(tai) - bdt.seconds(),
        1356.0 * 604_800.0 + 14.0
    );
}

#[test]
fn utc_helpers_respect_kernel_changes_and_frozen_policy() {
    use stkcraft_time::{calendar::gregorian_to_jdn, LeapEntry, LeapSecondPolicy};
    let lsk = LeapSecondTable::builtin().unwrap();
    let mut entries = lsk.entries().to_vec();
    entries.push(LeapEntry {
        jdn: gregorian_to_jdn(2027, 1, 1),
        delta_at: 38,
    });
    let future = LeapSecondTable::new(entries).unwrap();
    let frozen = future
        .clone()
        .with_policy(LeapSecondPolicy::frozen_after(2026, 1, 1));
    let utc = UtcDateTime::new(2027, 1, 1, 0, 0, 0.0);
    let reference = Bdt::from_utc(&utc, Calendar::Gregorian, &lsk).unwrap();
    let changed = Bdt::from_utc(&utc, Calendar::Gregorian, &future).unwrap();
    assert_eq!(
        changed.total_nanoseconds() - reference.total_nanoseconds(),
        1_000_000_000
    );
    assert_eq!(
        Bdt::from_utc(&utc, Calendar::Gregorian, &frozen).unwrap(),
        reference
    );
    assert_eq!(changed.to_utc(Calendar::Gregorian, &future).unwrap(), utc);
    assert_eq!(reference.to_utc(Calendar::Gregorian, &frozen).unwrap(), utc);
}

#[test]
fn representable_intersection_and_extreme_week_parts_are_exact() {
    for value in [
        Bdt::checked_from_parts(i64::MIN, 0).unwrap(),
        Bdt::checked_from_parts(i64::MAX - ORIGIN_TAI_SECONDS, 999_999_999).unwrap(),
    ] {
        assert_eq!(Bdt::from_epoch(value.to_epoch().unwrap()).unwrap(), value);
        let (week, nanos) = value.week_nanoseconds();
        assert_eq!(Bdt::from_week_nanoseconds(week, nanos).unwrap(), value);
    }
    for value in [
        Bdt::checked_from_parts(i64::MIN, 0).unwrap(),
        Bdt::checked_from_parts(i64::MAX, 999_999_999).unwrap(),
    ] {
        let (week, nanos) = value.week_nanoseconds();
        assert_eq!(Bdt::from_week_nanoseconds(week, nanos).unwrap(), value);
    }
}

#[test]
fn utc_conversion_rejects_days_outside_the_calendar_year_range() {
    use stkcraft_time::{calendar::calendar_to_jdn, Duration, LeapEntry};
    let lsk = LeapSecondTable::builtin().unwrap();
    for calendar in [Calendar::Gregorian, Calendar::Julian, Calendar::Mixed] {
        let maximum = UtcDateTime::new(i32::MAX, 12, 31, 23, 59, 59.5);
        let value = Bdt::from_utc(&maximum, calendar, &lsk).unwrap();
        assert_eq!(value.to_utc(calendar, &lsk).unwrap(), maximum);
        let next = Bdt::from_epoch(
            value
                .to_epoch()
                .unwrap()
                .checked_add(Duration::from_parts(1, 0))
                .unwrap(),
        )
        .unwrap();
        assert!(matches!(
            next.to_utc(calendar, &lsk),
            Err(TimeError::Overflow)
        ));

        // A custom table may cover the earliest i32 year. Keep that date
        // valid while rejecting the day immediately before it.
        let minimum_day = calendar_to_jdn(i32::MIN, 1, 1, calendar).unwrap();
        let ancient = LeapSecondTable::new(vec![LeapEntry {
            jdn: minimum_day - 1,
            delta_at: 0,
        }])
        .unwrap();
        let minimum = UtcDateTime::new(i32::MIN, 1, 1, 0, 0, 0.5);
        // Civil UTC construction deliberately retains its pre-1972 model
        // coverage rules. Supply this custom table's exact epoch directly.
        let value = Bdt::from_epoch(Epoch::from_parts(
            (minimum_day - stkcraft_time::calendar::J2000_JDN) * 86_400,
            500_000_000,
        ))
        .unwrap();
        assert_eq!(value.to_utc(calendar, &ancient).unwrap(), minimum);
        let previous = Bdt::from_epoch(
            value
                .to_epoch()
                .unwrap()
                .checked_sub(Duration::from_parts(1, 0))
                .unwrap(),
        )
        .unwrap();
        assert!(matches!(
            previous.to_utc(calendar, &ancient),
            Err(TimeError::Overflow)
        ));
    }
}

#[test]
fn extreme_utc_conversion_returns_an_error_without_panicking() {
    use stkcraft_time::{calendar::J2000_JDN, LeapEntry};
    let lsk = LeapSecondTable::builtin().unwrap();
    for calendar in [Calendar::Gregorian, Calendar::Julian] {
        let latest = Bdt::from_epoch(Epoch::from_parts(i64::MAX, 0)).unwrap();
        assert!(matches!(
            latest.to_utc(calendar, &lsk),
            Err(TimeError::Overflow)
        ));
        assert!(Bdt::checked_from_parts(i64::MIN, 0)
            .unwrap()
            .to_utc(calendar, &lsk)
            .is_err());
        // The checked UTC day helper can fail due to offset subtraction,
        // even after a valid custom table's first TAI instant.
        let custom = LeapSecondTable::new(vec![LeapEntry {
            jdn: J2000_JDN,
            delta_at: i32::MIN,
        }])
        .unwrap();
        assert!(matches!(
            latest.to_utc(calendar, &custom),
            Err(TimeError::Overflow)
        ));
    }
}

#[test]
fn utc_range_guard_preserves_sofa_historical_conversion() {
    use stkcraft_time::LeapSecondSource;
    let lsk = LeapSecondTable::builtin()
        .unwrap()
        .with_source(LeapSecondSource::Sofa);
    for utc in [
        UtcDateTime::new(1960, 1, 1, 0, 0, 0.0),
        UtcDateTime::new(1965, 7, 1, 12, 0, 0.5),
    ] {
        let expected_epoch = convert::utc_to_tai(&utc, Calendar::Gregorian, &lsk).unwrap();
        let bdt = Bdt::from_epoch(expected_epoch).unwrap();
        let expected_utc = convert::tai_to_utc(expected_epoch, Calendar::Gregorian, &lsk).unwrap();
        assert_eq!(bdt.to_utc(Calendar::Gregorian, &lsk).unwrap(), expected_utc);
        assert_eq!(Bdt::from_utc(&utc, Calendar::Gregorian, &lsk).unwrap(), bdt);
    }
}

#[test]
fn from_utc_rejects_active_unrepresentable_kernel_jumps() {
    use stkcraft_time::{calendar::gregorian_to_jdn, LeapEntry};
    let first = gregorian_to_jdn(1972, 1, 1);
    let boundary = gregorian_to_jdn(2200, 1, 1);
    for (current, next) in [(i32::MIN, i32::MAX), (i32::MAX, i32::MIN)] {
        // Widely separated entries keep their TAI instants increasing, so
        // both tables are accepted by the existing kernel constructor.
        let lsk = LeapSecondTable::new(vec![
            LeapEntry {
                jdn: first,
                delta_at: current,
            },
            LeapEntry {
                jdn: boundary,
                delta_at: next,
            },
        ])
        .unwrap();
        let utc = UtcDateTime::new(2199, 12, 31, 23, 59, 59.0);
        assert!(matches!(
            Bdt::from_utc(&utc, Calendar::Gregorian, &lsk),
            Err(TimeError::Overflow)
        ));
    }
}

#[test]
fn to_utc_rejects_active_unrepresentable_kernel_jumps() {
    use stkcraft_time::{calendar::gregorian_to_jdn, LeapEntry};
    let first = gregorian_to_jdn(1972, 1, 1);
    let boundary = gregorian_to_jdn(2200, 1, 1);
    for (current, next) in [(i32::MIN, i32::MAX), (i32::MAX, i32::MIN)] {
        let lsk = LeapSecondTable::new(vec![
            LeapEntry {
                jdn: first,
                delta_at: current,
            },
            LeapEntry {
                jdn: boundary,
                delta_at: next,
            },
        ])
        .unwrap();
        let epoch = Epoch::from_parts(
            (boundary - stkcraft_time::calendar::J2000_JDN) * 86_400 + current as i64,
            0,
        );
        let bdt = Bdt::from_epoch(epoch).unwrap();
        assert!(matches!(
            bdt.to_utc(Calendar::Gregorian, &lsk),
            Err(TimeError::Overflow)
        ));
    }
}

#[test]
fn utc_jump_guard_ignores_frozen_entries_and_historical_sofa_route() {
    use stkcraft_time::{
        calendar::gregorian_to_jdn, LeapEntry, LeapSecondPolicy, LeapSecondSource,
    };
    let first = gregorian_to_jdn(1972, 1, 1);
    let boundary = gregorian_to_jdn(2200, 1, 1);
    let lsk = LeapSecondTable::new(vec![
        LeapEntry {
            jdn: first,
            delta_at: i32::MIN,
        },
        LeapEntry {
            jdn: boundary,
            delta_at: i32::MAX,
        },
    ])
    .unwrap();
    let frozen = lsk
        .clone()
        .with_policy(LeapSecondPolicy::frozen_after(2199, 1, 1));
    let utc = UtcDateTime::new(2199, 12, 31, 23, 59, 59.0);
    let bdt = Bdt::from_utc(&utc, Calendar::Gregorian, &frozen).unwrap();
    assert_eq!(bdt.to_utc(Calendar::Gregorian, &frozen).unwrap(), utc);
    // An artificial future jump must not interfere with the separate
    // historical SOFA drift model before the first TAI table instant.
    let historical = LeapSecondTable::new(vec![
        LeapEntry {
            jdn: first,
            delta_at: i32::MAX,
        },
        LeapEntry {
            jdn: boundary,
            delta_at: i32::MIN,
        },
    ])
    .unwrap()
    .with_source(LeapSecondSource::Sofa);
    let utc = UtcDateTime::new(1960, 1, 1, 0, 0, 0.0);
    let bdt = Bdt::from_utc(&utc, Calendar::Gregorian, &historical).unwrap();
    assert_eq!(bdt.to_utc(Calendar::Gregorian, &historical).unwrap(), utc);
}
