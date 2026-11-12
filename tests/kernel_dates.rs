//! Native N0067 kernel-date literals.
//!
//! The expected numbers are the native WebGeocalc results recorded for the
//! STKCraft-Geocalc frame-kernel capture
//! (`docs/review-2026-11-12/frame-kernel-errors/time-kernel-dates.tsv`).
//! Every numeric expectation is the exact IEEE-754 bit pattern; no
//! tolerance is applied.
//!
//! Scope: these tests cover the eleven literals in that capture. Broader
//! TPARSE coverage is not claimed; anything outside the documented forms
//! must fail with `KernelDateUnsupported`, never with a missing-kernel
//! error and never with a substitute epoch.

use stkcraft_time::kernel_date::parse_kernel_date;
use stkcraft_time::TimeError;

/// Literal, expected seconds past J2000 as an IEEE-754 bit pattern.
const NATIVE_VALUES: [(&str, u64); 8] = [
    ("@2000-JAN-01-12:00:00", 0x0000_0000_0000_0000),
    ("@2000-JAN-01", 0xc0e5_1800_0000_0000),
    ("@31-JAN-1987", 0xc1b8_4cad_c000_0000),
    ("@feb/4/1987", 0xc1b8_4767_c000_0000),
    ("@March-7-1987-3:10:39.221", 0xc1b8_1e5c_90c7_6c8b),
    ("@2000-FEB-29-23:59:59.999", 0x4153_9c4f_ffef_9db2),
    ("@1965-JUL-01", 0xc1d0_39a7_f000_0000),
    ("@2001-FEB-29", 0x4181_7d26_0000_0000),
];

/// Literals native SPICE rejects with `SPICE(BADTIMESPEC)`.
const NATIVE_REJECTIONS: [&str; 3] = ["@2000-060", "@", "@NOT_A_DATE"];

#[test]
fn native_kernel_date_bits_match_exactly() {
    for (text, expected) in NATIVE_VALUES {
        let date = parse_kernel_date(text).unwrap_or_else(|error| panic!("{text}: {error}"));
        assert_eq!(
            date.seconds_past_j2000.to_bits(),
            expected,
            "{text} produced {}",
            date.seconds_past_j2000
        );
    }
}

#[test]
fn leap_day_and_midnight_normalize_to_calendar_fields() {
    let leap = parse_kernel_date("@2000-FEB-29-23:59:59.999").unwrap();
    assert_eq!(
        (leap.year, leap.month, leap.day, leap.hour, leap.minute),
        (2000, 2, 29, 23, 59)
    );
    assert_eq!(leap.second, 59.999);

    let midnight = parse_kernel_date("@2000-JAN-01").unwrap();
    assert_eq!((midnight.year, midnight.month, midnight.day), (2000, 1, 1));
    assert_eq!(
        (midnight.hour, midnight.minute, midnight.second),
        (0, 0, 0.0)
    );
}

#[test]
fn day_past_the_end_of_the_month_rolls_forward() {
    let rolled = parse_kernel_date("@2001-FEB-29").unwrap();

    assert_eq!((rolled.year, rolled.month, rolled.day), (2001, 3, 1));
    assert_eq!(rolled.seconds_past_j2000, 36_676_800.0);
}

#[test]
fn historical_literal_needs_no_leap_second_kernel() {
    // 1965 is before the first LSK entry; a kernel date constant is plain
    // calendar arithmetic and must not be gated by UTC coverage.
    let historical = parse_kernel_date("@1965-JUL-01").unwrap();

    assert_eq!(
        (historical.year, historical.month, historical.day),
        (1965, 7, 1)
    );
    assert_eq!(historical.seconds_past_j2000, -1_088_856_000.0);
}

#[test]
fn native_rejections_are_explicit_and_never_missing_constants() {
    for text in NATIVE_REJECTIONS {
        let error = parse_kernel_date(text).unwrap_err();
        assert_eq!(
            error,
            TimeError::KernelDateUnsupported,
            "{text} returned {error:?}"
        );

        // A present literal must never surface as an absent kernel variable.
        assert!(!format!("{error}").contains("missing"), "{text}");
    }
}

#[test]
fn malformed_literals_are_rejected_not_approximated() {
    for text in [
        "@19-JAN-87",            // two-digit year
        "@2000-060",             // ordinal day of year
        "@2000-JAN",             // no day
        "@2000-JAN-01-12",       // time field without a colon
        "@2000-JAN-01-24:00:00", // hour out of range
        "@2000-JAN-01-12:60:00", // minute out of range
        "@2000-JAN-01-12:00:60", // second out of range
        "@2000-FEB-00",          // day zero
        "@2000-FEB-32",          // day beyond any month
        "@2000-JAN-01T12:00:00", // wrong separator for time
        "@2000-01/01",           // mixed separators
        "@01-2000-02",           // numeric year in the middle
        "@2000-JAN-01-12:00:.",  // empty fractional second
        "2000-JAN-01",           // missing `@`
        "@ 2000-JAN-01",         // leading space
    ] {
        assert_eq!(
            parse_kernel_date(text).unwrap_err(),
            TimeError::KernelDateUnsupported,
            "{text} was accepted"
        );
    }
}

#[test]
fn numeric_month_order_matches_native_convention() {
    let year_first = parse_kernel_date("@1990-01-02").unwrap();
    let year_last = parse_kernel_date("@01/02/1990").unwrap();

    assert_eq!(
        (year_first.year, year_first.month, year_first.day),
        (1990, 1, 2)
    );
    assert_eq!(
        (year_last.year, year_last.month, year_last.day),
        (1990, 1, 2)
    );
}
