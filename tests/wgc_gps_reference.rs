use stkcraft_time::LeapSecondTable;

#[test]
fn wgc_gps_week_tow_matches_live_reference() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/week-tow-cases.tsv");
    assert_eq!(rows.lines().count(), 355);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let seconds: f64 = input.parse().unwrap();
        let actual = stkcraft_time::wgc_gps::gps_seconds_to_week_tow(seconds, &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}

#[test]
fn wgc_gps_week_tow_rejects_nonfinite_kernel_arithmetic() {
    use stkcraft_time::wgc_gps::{gps_seconds_to_week_tow, WgcGpsError};
    let mut lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    lsk.k = f64::NAN;
    assert_eq!(
        gps_seconds_to_week_tow(0.0, &lsk),
        Err(WgcGpsError::NonFinite)
    );
}

#[test]
fn wgc_gps_week_tow_matches_half_ties_and_large_inputs() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/week-tow-edge-cases.tsv");
    assert_eq!(rows.lines().count(), 39);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let seconds: f64 = input.parse().unwrap();
        let actual = stkcraft_time::wgc_gps::gps_seconds_to_week_tow(seconds, &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}

#[test]
fn wgc_gps_week_tow_matches_decimal_tie_cases() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/decimal-ties.tsv");
    assert_eq!(rows.lines().count(), 7);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual =
            stkcraft_time::wgc_gps::gps_seconds_to_week_tow(input.parse().unwrap(), &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}

#[test]
fn wgc_gps_week_tow_matches_decimal_adjacent_cases() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/decimal-adjacent.tsv");
    assert_eq!(rows.lines().count(), 9);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual =
            stkcraft_time::wgc_gps::gps_seconds_to_week_tow(input.parse().unwrap(), &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}

#[test]
fn wgc_gps_week_tow_matches_seeded_half_grid() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/seeded-half-grid.tsv");
    assert_eq!(rows.lines().count(), 401);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual =
            stkcraft_time::wgc_gps::gps_seconds_to_week_tow(input.parse().unwrap(), &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}

#[test]
fn wgc_gps_week_tow_matches_independent_half_probe() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/independent-half-probe.tsv");
    assert_eq!(rows.lines().count(), 481);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual =
            stkcraft_time::wgc_gps::gps_seconds_to_week_tow(input.parse().unwrap(), &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}

#[test]
fn wgc_gps_week_tow_matches_nonfinite_inputs() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps/nonfinite-inputs.tsv");
    assert_eq!(rows.lines().count(), 4);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let value = match input {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            _ => panic!("unexpected input"),
        };
        let actual = stkcraft_time::wgc_gps::gps_seconds_to_week_tow(value, &lsk).unwrap();
        assert_eq!(actual, expected, "GPS seconds {input}");
    }
}
