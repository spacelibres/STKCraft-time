//! RFC 3339, RFC 2822 and C89-style formatting tests.

use stkcraft_time::format::{etcal, iso8601, rfc2822, rfc3339, strftime};
use stkcraft_time::UtcDateTime;

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
fn etcal_preserves_microseconds() {
    assert_eq!(etcal(0.000_001, 9), "2000 JAN 01 12:00:00.000001000");
    assert_eq!(etcal(-0.000_001, 9), "2000 JAN 01 11:59:59.999999000");
    assert_eq!(etcal(86_400.000_001, 9), "2000 JAN 02 12:00:00.000001000");
    assert_eq!(etcal(f64::NAN, 9), "INVALID ET");
}
