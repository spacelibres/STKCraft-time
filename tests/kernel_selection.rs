//! Kernel selection.
//!
//! The same UTC instant converts differently under different
//! leap-second kernels. This is the library equivalent of the "Kernel
//! selection" control in the JPL WebGeocalc Time Conversion tool. No
//! result is produced by guessing: dates outside a kernel's coverage are
//! rejected, and the post-kernel behaviour is NAIF's documented one.

use stkcraft_time::calendar::{gregorian_to_jdn, Calendar};
use stkcraft_time::convert::{tai_to_et, utc_to_tai};
use stkcraft_time::{LeapEntry, LeapSecondTable, TimeError, UtcDateTime};

/// A table truncated to the leap-second history of an earlier kernel.
fn truncated(cutoff_jdn: i64) -> LeapSecondTable {
    let full = LeapSecondTable::builtin().unwrap();
    let entries: Vec<LeapEntry> = full
        .entries()
        .iter()
        .copied()
        .filter(|entry| entry.jdn < cutoff_jdn)
        .collect();
    LeapSecondTable::new(entries).unwrap()
}

/// The built-in kernel with every entry on or after 2017-01-01 removed.
fn kernel_without_2017() -> LeapSecondTable {
    truncated(gregorian_to_jdn(2017, 1, 1))
}

/// Maps a measured kernel name to the table with the same leap history.
fn table_for_kernel(name: &str) -> LeapSecondTable {
    match name {
        "naif0012" => LeapSecondTable::builtin().unwrap(),
        "naif0011" => truncated(gregorian_to_jdn(2017, 1, 1)),
        "naif0010" => truncated(gregorian_to_jdn(2015, 7, 1)),
        other => panic!("unknown kernel in golden CSV: {other}"),
    }
}

/// Parses `YYYY-MM-DDTHH:MM:SS[.fraction]` for the golden rows only.
fn parse_utc(text: &str) -> UtcDateTime {
    let (date, time) = text.split_once('T').expect("ISO date-time");
    let mut d = date.split('-');
    let year: i32 = d.next().unwrap().parse().unwrap();
    let month: u8 = d.next().unwrap().parse().unwrap();
    let day: u8 = d.next().unwrap().parse().unwrap();

    let mut t = time.split(':');
    let hour: u8 = t.next().unwrap().parse().unwrap();
    let minute: u8 = t.next().unwrap().parse().unwrap();
    let second: f64 = t.next().unwrap().parse().unwrap();

    UtcDateTime::new(year, month, day, hour, minute, second)
}

#[test]
fn same_utc_different_kernel_different_tai() {
    let full = LeapSecondTable::builtin().unwrap();
    let old = kernel_without_2017();

    let dt = UtcDateTime::new(2017, 6, 1, 0, 0, 0.0);
    let tai_full = utc_to_tai(&dt, Calendar::Gregorian, &full).unwrap();
    let tai_old = utc_to_tai(&dt, Calendar::Gregorian, &old).unwrap();

    assert_eq!(
        tai_full.sec() - tai_old.sec(),
        1,
        "the 2017 leap second must be the only difference"
    );
}

#[test]
fn builtin_kernel_reports_its_coverage() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let (first, last) = lsk.coverage();

    assert_eq!(first, gregorian_to_jdn(1972, 1, 1));
    assert_eq!(last, gregorian_to_jdn(2017, 1, 1));
}

#[test]
fn from_lsk_builds_the_supplied_kernel() {
    let text = include_str!("../src/naif0012.tls");
    let parsed = LeapSecondTable::from_lsk(text).unwrap();
    let builtin = LeapSecondTable::builtin().unwrap();

    assert_eq!(parsed.entries(), builtin.entries());
    assert_eq!(parsed.coverage(), builtin.coverage());
}

#[test]
fn utc_before_the_kernel_is_rejected_not_guessed() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let before = UtcDateTime::new(1960, 1, 1, 0, 0, 0.0);

    assert!(utc_to_tai(&before, Calendar::Gregorian, &lsk).is_err());
}

#[test]
fn utc_after_the_kernel_holds_the_last_value() {
    let lsk = LeapSecondTable::builtin().unwrap();

    let at_last = UtcDateTime::new(2017, 1, 1, 0, 0, 0.0);
    let later = UtcDateTime::new(2017, 1, 1, 12, 0, 0.0);

    let a = utc_to_tai(&at_last, Calendar::Gregorian, &lsk).unwrap();
    let b = utc_to_tai(&later, Calendar::Gregorian, &lsk).unwrap();

    // Twelve hours later, TAI - UTC is unchanged: no leap second is
    // fabricated beyond what the kernel contains.
    assert_eq!(b.sec() - a.sec(), 43_200);
}

/// Checks this crate against values measured from the live WGC2 API.
///
/// `tools/wgc2_kernel_sweep.py` produced the CSV with the same UTC
/// epochs converted to TDB under three leap-second kernels. The values
/// are rounded by WGC2 to about six decimals, so the tolerance is 5e-7 s.
///
/// Rows that use a `23:59:60` label with a kernel that does not declare
/// that leap second are expected to be rejected: `SPICE` documents that
/// an undeclared leap second is an error, and this crate does not invent
/// one. WGC2 returned a number for those inputs; that lenient
/// normalisation is deliberately not reproduced.
#[test]
fn crate_matches_measured_wgc2_kernel_values() {
    let csv = include_str!("golden/wgc2_kernel_selection.csv");
    let mut compared = 0;

    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("kernel,") {
            continue;
        }

        let mut fields = line.split(',');
        let kernel = fields.next().unwrap();
        let utc = fields.next().unwrap();
        let expected: f64 = fields.next().unwrap().parse().unwrap();

        let table = table_for_kernel(kernel);
        let dt = parse_utc(utc);
        let result = utc_to_tai(&dt, Calendar::Gregorian, &table);

        if utc.contains("23:59:60") && kernel != "naif0012" {
            assert_eq!(
                result,
                Err(TimeError::InvalidLeapSecond),
                "{kernel} {utc} must reject an undeclared leap second"
            );
            continue;
        }

        let tai = result.unwrap_or_else(|error| panic!("{kernel} {utc}: {error:?}"));
        let et = tai_to_et(tai, &table);
        assert!(
            (et - expected).abs() <= 5e-7,
            "{kernel} {utc}: crate {et} vs measured WGC2 {expected}"
        );
        compared += 1;
    }

    assert_eq!(compared, 10, "expected ten comparable rows");
}
