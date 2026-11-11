//! RFC 3339, RFC 2822 and C89-style formatting tests.

use stkcraft_time::calendar::Calendar;
use stkcraft_time::format::{
    etcal, iso8601, iso8601_with, rfc2822, rfc3339, strftime, strftime_with, FormatOptions,
};
use stkcraft_time::{LeapSecondTable, UtcDateTime};

#[test]
fn rfc3339_formatting() {
    let dt = UtcDateTime::new(2000, 2, 29, 14, 57, 29.000_000_037);
    assert_eq!(rfc3339(&dt, 9), "2000-02-29T14:57:29.000000037+00:00");
}

#[test]
fn rfc2822_formatting() {
    let dt = UtcDateTime::new(2000, 2, 29, 14, 57, 29.0);
    assert_eq!(rfc2822(&dt), "Tue, 29 Feb 2000 14:57:29 +0000");
}

#[test]
fn c89_tokens() {
    let dt = UtcDateTime::new(2000, 2, 29, 14, 57, 29.5);

    assert_eq!(
        strftime(&dt, "%Y-%m-%d %H:%M:%S").unwrap(),
        "2000-02-29 14:57:29"
    );
    assert_eq!(strftime(&dt, "%j").unwrap(), "060");
    assert_eq!(strftime(&dt, "%a %b %e %Y").unwrap(), "Tue Feb 29 2000");
    assert_eq!(
        strftime(&dt, "%A, %B %d, %Y").unwrap(),
        "Tuesday, February 29, 2000"
    );
    assert_eq!(strftime(&dt, "%f").unwrap(), "500000000");
    assert_eq!(strftime(&dt, "100%%").unwrap(), "100%");
}

#[test]
fn leap_second_seconds_field() {
    let dt = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
    assert_eq!(
        strftime(&dt, "%Y-%m-%dT%H:%M:%S.%f").unwrap(),
        "2016-12-31T23:59:60.500000000"
    );
}

#[test]
fn invalid_token_is_rejected() {
    let dt = UtcDateTime::new(2000, 2, 29, 14, 57, 29.0);
    assert!(strftime(&dt, "%Q").is_err());
}

#[test]
fn rounding_carries_to_minute_and_day() {
    let dt = UtcDateTime::new(2026, 1, 1, 23, 59, 9.9999);
    assert_eq!(iso8601(&dt, 3), "2026-01-01T23:59:10.000Z");

    let dt = UtcDateTime::new(2026, 1, 1, 23, 59, 59.9999);
    assert_eq!(iso8601(&dt, 3), "2026-01-02T00:00:00.000Z");

    let leap = UtcDateTime::new(2016, 12, 31, 23, 59, 60.9999);
    assert_eq!(iso8601(&leap, 3), "2017-01-01T00:00:00.000Z");

    let almost_midnight = UtcDateTime::new(2026, 1, 1, 23, 59, 59.999_999_999_9);
    assert_eq!(
        strftime(&almost_midnight, "%H:%M:%S.%f").unwrap(),
        "00:00:00.000000000"
    );
}

#[test]
fn precision_is_clamped_to_nanoseconds() {
    let dt = UtcDateTime::new(2000, 1, 1, 12, 0, 0.0);
    assert_eq!(iso8601(&dt, 100), "2000-01-01T12:00:00.000000000Z");
}

#[test]
fn rounding_onto_a_leap_second_uses_the_day_length() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let options = FormatOptions::new(3).with_leap_seconds(&lsk);

    // 2016-12-31 is 86 401 s long, so this rounds to the leap second and
    // not to the following day.
    let dt = UtcDateTime::new(2016, 12, 31, 23, 59, 59.9999);
    assert_eq!(iso8601_with(&dt, &options), "2016-12-31T23:59:60.000Z");
    assert_eq!(
        strftime_with(&dt, "%Y-%m-%dT%H:%M:%S", &options).unwrap(),
        "2016-12-31T23:59:60"
    );

    // The plain formatter documents a 86 400-second day.
    assert_eq!(iso8601(&dt, 3), "2017-01-01T00:00:00.000Z");

    // The next day is normal, so the same instant carries into the day.
    let normal = UtcDateTime::new(2017, 1, 1, 23, 59, 59.9999);
    assert_eq!(iso8601_with(&normal, &options), "2017-01-02T00:00:00.000Z");
}

#[test]
fn day_carry_follows_the_selected_calendar() {
    // 1500 is a Julian leap year but not a Gregorian one.
    let dt = UtcDateTime::new(1500, 2, 28, 23, 59, 59.9999);
    let julian = FormatOptions::new(3).with_calendar(Calendar::Julian);

    assert_eq!(iso8601_with(&dt, &julian), "1500-02-29T00:00:00.000Z");
    assert_eq!(iso8601(&dt, 3), "1500-03-01T00:00:00.000Z");

    let leap_day = UtcDateTime::new(1500, 2, 29, 12, 0, 0.0);
    assert_eq!(strftime_with(&leap_day, "%j", &julian).unwrap(), "060");
}

#[test]
fn etcal_preserves_microseconds() {
    assert_eq!(etcal(0.000_001, 9), "2000 JAN 01 12:00:00.000001000");
    assert_eq!(etcal(-0.000_001, 9), "2000 JAN 01 11:59:59.999999000");
    assert_eq!(etcal(86_400.000_001, 9), "2000 JAN 02 12:00:00.000001000");
    assert_eq!(etcal(f64::NAN, 9), "INVALID ET");
}
