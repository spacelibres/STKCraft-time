//! RFC 3339, RFC 2822 and C89-style formatting tests.

use stkcraft_time::format::{rfc2822, rfc3339, strftime};
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
