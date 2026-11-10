//! Point-by-point comparison against CSPICE golden vectors.
//!
//! The CSV is generated offline by `tools/gen_golden.py` using
//! `spiceypy`; CSPICE is not a build or runtime dependency.

use std::fs;

use stkcraft_time::convert::utc_to_et;
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

fn parse_utc(value: &str) -> UtcDateTime {
    let (date, time) = value.split_once('T').expect("date/time separator");

    let mut date_parts = date.split('-');
    let year = date_parts.next().unwrap().parse().unwrap();
    let month = date_parts.next().unwrap().parse().unwrap();
    let day = date_parts.next().unwrap().parse().unwrap();

    let mut time_parts = time.split(':');
    let hour = time_parts.next().unwrap().parse().unwrap();
    let minute = time_parts.next().unwrap().parse().unwrap();
    let second = time_parts.next().unwrap().parse().unwrap();

    UtcDateTime::new(year, month, day, hour, minute, second)
}

#[test]
fn golden_utc_to_et_matches_spice() {
    let lsk = LeapSecondTable::builtin().expect("bundled naif0012.tls must parse");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/utc_to_et.csv");
    let csv = fs::read_to_string(path).expect("golden CSV must exist");

    let mut count = 0;
    let mut max_error = 0.0_f64;

    for line in csv.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let (utc, et_text) = line.split_once(',').expect("CSV row");
        let expected: f64 = et_text.parse().expect("golden ET");
        let dt = parse_utc(utc);
        let actual = utc_to_et(&dt, Calendar::Gregorian, &lsk).expect("valid UTC");

        let error = (actual - expected).abs();
        max_error = max_error.max(error);
        count += 1;

        assert!(
            error < 1e-6,
            "{utc}: CSPICE = {expected}, ours = {actual}, error = {error:e} s"
        );
    }

    assert!(
        count >= 80,
        "expected at least 80 golden vectors, got {count}"
    );
    println!("max ET error: {max_error:e} s over {count} vectors");
}
