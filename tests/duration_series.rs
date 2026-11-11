//! Exact `Duration` arithmetic and `TimeSeries` tests.

use stkcraft_time::{Duration, Epoch, TimeSeries};

#[test]
fn duration_is_exact() {
    let twelve_minutes = Duration::from_parts(12 * 60, 0);
    let total = twelve_minutes + Duration::from_parts(0, 35);

    assert_eq!(total.sec(), 720);
    assert_eq!(total.nanos(), 35);
    assert_eq!(format!("{total}"), "720.000000035 s");
}

#[test]
fn negative_duration_display() {
    let negative = Duration::from_parts(-2, 500_000_000);
    assert!((negative.seconds_f64() + 1.5).abs() < 1e-12);
    assert_eq!(format!("{negative}"), "-1.500000000 s");
}

#[test]
fn rounding_is_integer_based() {
    let value = Duration::from_parts(720, 35);
    let minute = Duration::from_parts(60, 0);

    assert_eq!(value.round(minute), Duration::from_parts(720, 0));
    assert_eq!(value.floor(minute), Duration::from_parts(720, 0));
    assert_eq!(value.ceil(minute), Duration::from_parts(780, 0));

    let one_and_a_half = Duration::from_parts(1, 500_000_000);
    let second = Duration::SECOND;
    assert_eq!(one_and_a_half.floor(second), Duration::from_parts(1, 0));
    assert_eq!(one_and_a_half.ceil(second), Duration::from_parts(2, 0));
    assert_eq!(one_and_a_half.round(second), Duration::from_parts(2, 0));
}

#[test]
fn duration_covers_65536_centuries() {
    const JULIAN_CENTURY_SECONDS: i64 = 3_155_760_000;
    let edge = Duration::from_parts(65_536 * JULIAN_CENTURY_SECONDS, 999_999_999);
    assert_eq!(edge.sec(), 65_536 * JULIAN_CENTURY_SECONDS);
    assert_eq!(edge.nanos(), 999_999_999);
}

#[test]
fn epoch_arithmetic() {
    let start = Epoch::from_parts(0, 0);
    let later = start + Duration::from_parts(5, 250_000_000);
    let delta = later - start;

    assert_eq!(delta, Duration::from_parts(5, 250_000_000));
    assert_eq!(later - Duration::from_parts(5, 250_000_000), start);
}

#[test]
fn time_series_inclusive_has_seven_steps() {
    let start = Epoch::from_parts(0, 0);
    let end = start + Duration::from_parts(12 * 3_600, 0);
    let step = Duration::from_parts(2 * 3_600, 0);

    assert_eq!(TimeSeries::inclusive(start, end, step).count(), 7);
    assert_eq!(TimeSeries::exclusive(start, end, step).count(), 6);
}

#[test]
fn checked_duration_accepts_representable_extremes() {
    let a = Duration::from_parts(i64::MIN, 500_000_000);

    assert_eq!(
        a.checked_neg(),
        Some(Duration::from_parts(i64::MAX, 500_000_000))
    );
    assert_eq!(a.checked_sub(a), Some(Duration::ZERO));
    assert_eq!(
        a.checked_add(Duration::from_parts(-1, 500_000_000)),
        Some(Duration::from_parts(i64::MIN, 0))
    );
}

#[test]
fn invalid_float_durations_are_rejected() {
    assert!(Duration::checked_from_seconds_f64(f64::NAN).is_none());
    assert!(Duration::checked_from_seconds_f64(f64::INFINITY).is_none());
    assert!(Duration::checked_from_seconds_f64(1e300).is_none());
}

#[test]
fn epoch_extremes_do_not_wrap() {
    assert!(Epoch::checked_from_parts(i64::MAX, 1_000_000_000).is_none());
    assert!(Epoch::checked_from_parts(i64::MIN, -1).is_none());
    assert_eq!(Epoch::from_parts(i64::MAX, 999_999_999).sec(), i64::MAX);

    assert!(Epoch::checked_from_tai_seconds(f64::NAN).is_none());
    assert!(Epoch::checked_from_tai_seconds(f64::INFINITY).is_none());

    let epoch = Epoch::from_parts(i64::MAX, 0);
    assert!(epoch.checked_add(Duration::SECOND).is_none());
    assert!(epoch
        .checked_sub(Duration::from_parts(i64::MIN, 0))
        .is_none());
}

#[test]
fn time_series_stops_at_last_representable_epoch() {
    let end = Epoch::from_parts(i64::MAX, 0);
    let mut inclusive = TimeSeries::inclusive(end, end, Duration::SECOND);

    assert_eq!(inclusive.next(), Some(end));
    assert_eq!(inclusive.next(), None);

    let mut exclusive = TimeSeries::exclusive(end, end, Duration::SECOND);
    assert_eq!(exclusive.next(), None);
}
