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
