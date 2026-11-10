//! Validation of the Fairhead & Bretagnon TDB series against ERFA.
//!
//! `tools/gen_dtdb_golden.py` samples ERFA `dtdb` every 30 days over
//! 1950-2050. The port must agree to a few nanoseconds; ERFA states the
//! series itself is good to about 3 ns in this interval.

use stkcraft_time::dtdb::dtdb_minus_tt;

#[test]
fn matches_erfa_series_over_1950_to_2050() {
    let csv = include_str!("golden/dtdb.csv");
    let mut worst: f64 = 0.0;
    let mut worst_jd = 0.0;
    let mut checked = 0;

    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt,") {
            continue;
        }

        let (jd_text, value_text) = line.split_once(',').unwrap();
        let jd: f64 = jd_text.parse().unwrap();
        let expected: f64 = value_text.parse().unwrap();

        let difference = (dtdb_minus_tt(jd) - expected).abs();
        if difference > worst {
            worst = difference;
            worst_jd = jd;
        }
        checked += 1;
    }

    assert!(checked > 1000, "expected a dense grid, got {checked}");
    // The series is ±3 ns; allow 5 ns for f64 evaluation differences.
    assert!(
        worst < 5e-9,
        "worst difference {worst:e} s at JD {worst_jd}"
    );

    println!("vs ERFA dtdb over {checked} samples: worst {worst:.3e} s (JD {worst_jd})");
}
