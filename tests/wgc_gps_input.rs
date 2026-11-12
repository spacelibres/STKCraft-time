use stkcraft_time::{wgc_gps, LeapSecondTable};

#[test]
fn week_tow_input_matches_nasa_seeded_display() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps-input/seeded-week-tow.tsv");
    assert_eq!(rows.lines().count(), 201);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let seconds = wgc_gps::parse_week_tow(input).unwrap();
        assert_eq!(
            wgc_gps::gps_seconds_to_week_tow(seconds, &lsk).unwrap(),
            expected,
            "{input}"
        );
    }
}

#[test]
fn week_tow_input_rejects_nasa_invalid_separators() {
    for input in [
        "1,2",
        "1 , 2",
        "1,\t2",
        "2147483648, 0",
        "1.5, 0",
        "\u{a0}1, 2",
        "1, \u{a0}2",
        "1, 2, ",
        "1, 2,3",
    ] {
        assert!(wgc_gps::parse_week_tow(input).is_err(), "{input:?}");
    }
}

#[test]
fn week_tow_input_accepts_observed_java_numbers() {
    for (input, expected) in [
        ("1, 0x1.0p2", 604804.0_f64),
        ("1, 1d", 604801.0),
        ("1, 1f", 604801.0),
        ("1, 2, 3", 604802.0),
        ("-1,  2", -604798.0),
    ] {
        assert_eq!(
            wgc_gps::parse_week_tow(input).unwrap().to_bits(),
            expected.to_bits(),
            "{input}"
        );
    }
    for input in ["1, nan", "1, inf", "1, -infinity", "1, 1_0", "1, 0x1"] {
        assert!(wgc_gps::parse_week_tow(input).is_err(), "{input}");
    }
    for input in ["1, NaN", "1, +NaN", "1, -NaN"] {
        assert!(wgc_gps::parse_week_tow(input).unwrap().is_nan(), "{input}");
    }
}

#[test]
fn hexadecimal_parser_matches_python_binary64_reference() {
    let rows = include_str!("reference/wgc-gps-input/hex-python.tsv");
    assert_eq!(rows.lines().count(), 5001);
    for row in rows.lines().skip(1) {
        let (input, expected) = row.split_once('\t').unwrap();
        let expected = u64::from_str_radix(expected, 16).unwrap();
        assert_eq!(
            wgc_gps::parse_week_tow(input).unwrap().to_bits(),
            expected,
            "{input}"
        );
    }
}

#[test]
fn utc_calendar_doy_matches_nasa_leap_and_day_boundaries() {
    use stkcraft_time::{calendar::Calendar, contract::TimeContext, epoch::UtcDateTime};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let context = TimeContext::new(&lsk);
    let cases = [
        (
            UtcDateTime::new(1980, 1, 6, 0, 0, 0.0),
            "1980-006/00:00:00.000000 UTC",
        ),
        (
            UtcDateTime::new(2000, 1, 1, 12, 0, 0.0),
            "2000-001/12:00:00.000000 UTC",
        ),
        (
            UtcDateTime::new(2016, 12, 31, 23, 59, 59.0),
            "2016-366/23:59:59.000000 UTC",
        ),
        (
            UtcDateTime::new(2016, 12, 31, 23, 59, 60.0),
            "2016-366/23:59:60.000000 UTC",
        ),
        (
            UtcDateTime::new(2017, 1, 1, 0, 0, 0.0),
            "2017-001/00:00:00.000000 UTC",
        ),
        (
            UtcDateTime::new(2024, 2, 29, 23, 59, 59.999999),
            "2024-060/23:59:59.999999 UTC",
        ),
    ];
    for (utc, expected) in cases {
        let et = context.utc_to_et(&utc, Calendar::Gregorian).unwrap();
        assert_eq!(wgc_gps::et_to_utc_calendar_doy(et, &lsk).unwrap(), expected);
    }
}

#[test]
fn utc_calendar_doy_matches_nasa_seeded_et_samples() {
    use stkcraft_time::contract::NaifEt;
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps-input/utc-doy-seeded.tsv");
    assert_eq!(rows.lines().count(), 241);
    for row in rows.lines().skip(1) {
        let (et, expected) = row.split_once('\t').unwrap();
        let et = NaifEt::from_seconds(et.parse().unwrap()).unwrap();
        assert_eq!(
            wgc_gps::et_to_utc_calendar_doy(et, &lsk).unwrap(),
            expected,
            "{row}"
        );
    }
}

#[test]
fn dynamical_calendar_doy_matches_nasa_seeded_systems() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    for (system, rows) in [
        (
            WgcCalendarSystem::Tdb,
            include_str!("reference/wgc-gps-input/tdb-doy-seeded.tsv"),
        ),
        (
            WgcCalendarSystem::Tt,
            include_str!("reference/wgc-gps-input/tt-doy-seeded.tsv"),
        ),
        (
            WgcCalendarSystem::Gps,
            include_str!("reference/wgc-gps-input/gps-doy-seeded.tsv"),
        ),
    ] {
        assert_eq!(rows.lines().count(), 241);
        for row in rows.lines().skip(1) {
            let (et, expected) = row.split_once('\t').unwrap();
            let et = NaifEt::from_seconds(et.parse().unwrap()).unwrap();
            assert_eq!(
                wgc_gps::et_to_calendar_doy(et, system, &lsk).unwrap(),
                expected,
                "{row}"
            );
        }
    }
}

#[test]
fn calendar_doy_matches_nasa_broad_epochs() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    for (system, rows) in [
        (
            WgcCalendarSystem::Utc,
            include_str!("reference/wgc-gps-input/utc-doy-epochs.tsv"),
        ),
        (
            WgcCalendarSystem::Tdb,
            include_str!("reference/wgc-gps-input/tdb-doy-epochs.tsv"),
        ),
        (
            WgcCalendarSystem::Tt,
            include_str!("reference/wgc-gps-input/tt-doy-epochs.tsv"),
        ),
        (
            WgcCalendarSystem::Gps,
            include_str!("reference/wgc-gps-input/gps-doy-epochs.tsv"),
        ),
    ] {
        assert_eq!(rows.lines().count(), 262);
        for row in rows.lines().skip(1) {
            let (et, expected) = row.split_once('\t').unwrap();
            let et = NaifEt::from_seconds(et.parse().unwrap()).unwrap();
            assert_eq!(
                wgc_gps::et_to_calendar_doy(et, system, &lsk).unwrap(),
                expected,
                "{system:?}: {row}"
            );
        }
    }
}

#[test]
fn calendar_doy_matches_cspice_timout_seeded_binary64() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps-input/cspice-doy-seeded.tsv");
    assert_eq!(rows.lines().count(), 12001);
    for row in rows.lines().skip(1) {
        let mut fields = row.split('\t');
        let system = match fields.next().unwrap() {
            "UTC" => WgcCalendarSystem::Utc,
            "TDB" => WgcCalendarSystem::Tdb,
            "TT" => WgcCalendarSystem::Tt,
            "GPS" => WgcCalendarSystem::Gps,
            _ => panic!("invalid system"),
        };
        let et = f64::from_bits(u64::from_str_radix(fields.next().unwrap(), 16).unwrap());
        let expected = fields.next().unwrap();
        assert_eq!(
            wgc_gps::et_to_calendar_doy(NaifEt::from_seconds(et).unwrap(), system, &lsk).unwrap(),
            expected,
            "{row}"
        );
    }
}

#[test]
fn calendar_doy_matches_nasa_near_all_lsk_boundaries() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    for (system, rows) in [
        (
            WgcCalendarSystem::Utc,
            include_str!("reference/wgc-gps-input/utc-doy-leaps.tsv"),
        ),
        (
            WgcCalendarSystem::Tdb,
            include_str!("reference/wgc-gps-input/tdb-doy-leaps.tsv"),
        ),
        (
            WgcCalendarSystem::Tt,
            include_str!("reference/wgc-gps-input/tt-doy-leaps.tsv"),
        ),
        (
            WgcCalendarSystem::Gps,
            include_str!("reference/wgc-gps-input/gps-doy-leaps.tsv"),
        ),
    ] {
        assert_eq!(rows.lines().count(), 337);
        for row in rows.lines().skip(1) {
            let (et, expected) = row.split_once('\t').unwrap();
            let et = NaifEt::from_seconds(et.parse().unwrap()).unwrap();
            assert_eq!(
                wgc_gps::et_to_calendar_doy(et, system, &lsk).unwrap(),
                expected,
                "{system:?}: {row}"
            );
        }
    }
}

#[test]
fn week_tow_grammar_matches_nasa_matrix() {
    let rows = include_str!("reference/wgc-gps-input/grammar-matrix-bytes.tsv");
    assert_eq!(rows.lines().count(), 60);
    let mut mismatches = Vec::new();
    for row in rows.lines().skip(1) {
        let (hex, accepted) = row.split_once('\t').unwrap();
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let input = core::str::from_utf8(&bytes).unwrap();
        if wgc_gps::parse_week_tow(input).is_ok() != (accepted == "1") {
            mismatches.push(input.to_owned());
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:?}");
}

#[test]
fn gps_week_input_to_numeric_seconds_matches_nasa_display_bits() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps-input/week-seconds-seeded.tsv");
    assert_eq!(rows.lines().count(), 201);
    for row in rows.lines().skip(1) {
        let (input, bits) = row.split_once('\t').unwrap();
        let et = wgc_gps::week_tow_to_et(input, &lsk).unwrap();
        assert_eq!(
            wgc_gps::et_to_gps_display_seconds(et, &lsk)
                .unwrap()
                .to_bits(),
            u64::from_str_radix(bits, 16).unwrap(),
            "{row}"
        );
    }
}

#[test]
fn gps_numeric_input_and_display_match_nasa_bitwise() {
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let mut errors = Vec::new();
    for (rows, count) in [
        (
            include_str!("reference/wgc-gps-input/gps-seconds-numeric.tsv"),
            330,
        ),
        (
            include_str!("reference/wgc-gps-input/gps-numeric-rounding-grid.tsv"),
            1200,
        ),
        (
            include_str!("reference/wgc-gps-input/gps-signed-zero.tsv"),
            10,
        ),
    ] {
        assert_eq!(rows.lines().count(), count + 1);
        for row in rows.lines().skip(1) {
            let (input, bits) = row.split_once('\t').unwrap();
            let et = wgc_gps::gps_seconds_to_et(input.parse().unwrap(), &lsk).unwrap();
            let actual = wgc_gps::et_to_gps_display_seconds(et, &lsk)
                .unwrap()
                .to_bits();
            if actual != u64::from_str_radix(bits, 16).unwrap() {
                errors.push((input, actual, bits));
            }
        }
    }
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn calendar_ymd_matches_nasa_broad_epochs() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    for (system, rows) in [
        (
            WgcCalendarSystem::Utc,
            include_str!("reference/wgc-gps-input/utc-ymd-epochs.tsv"),
        ),
        (
            WgcCalendarSystem::Tdb,
            include_str!("reference/wgc-gps-input/tdb-ymd-epochs.tsv"),
        ),
        (
            WgcCalendarSystem::Tt,
            include_str!("reference/wgc-gps-input/tt-ymd-epochs.tsv"),
        ),
        (
            WgcCalendarSystem::Gps,
            include_str!("reference/wgc-gps-input/gps-ymd-epochs.tsv"),
        ),
    ] {
        assert_eq!(rows.lines().count(), 262);
        for row in rows.lines().skip(1) {
            let (et, expected) = row.split_once('\t').unwrap();
            assert_eq!(
                wgc_gps::et_to_calendar_ymd(
                    NaifEt::from_seconds(et.parse().unwrap()).unwrap(),
                    system,
                    &lsk
                )
                .unwrap(),
                expected,
                "{system:?}: {row}"
            );
        }
    }
}

#[test]
fn calendar_formats_match_cspice_with_altered_lsk() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk =
        LeapSecondTable::from_lsk(include_str!("reference/wgc-gps-input/altered-lsk.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps-input/altered-lsk-doy-ymd.tsv");
    assert_eq!(rows.lines().count(), 3201);
    let mut errors = Vec::new();
    for row in rows.lines().skip(1) {
        let f: Vec<_> = row.split('\t').collect();
        let system = match f[1] {
            "UTC" => WgcCalendarSystem::Utc,
            "TDB" => WgcCalendarSystem::Tdb,
            "TT" => WgcCalendarSystem::Tt,
            "GPS" => WgcCalendarSystem::Gps,
            _ => panic!("unknown system"),
        };
        let et =
            NaifEt::from_seconds(f64::from_bits(u64::from_str_radix(f[2], 16).unwrap())).unwrap();
        let actual = if f[0] == "DOY" {
            wgc_gps::et_to_calendar_doy(et, system, &lsk)
        } else {
            wgc_gps::et_to_calendar_ymd(et, system, &lsk)
        }
        .unwrap();
        if actual != f[3] {
            errors.push(format!("{row}: {actual}"));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn week_tow_unicode_week_digits_match_nasa() {
    let rows = include_str!("reference/wgc-gps-input/grammar-unicode.tsv");
    assert_eq!(rows.lines().count(), 414);
    let mut errors = Vec::new();
    for row in rows.lines().skip(1) {
        let f: Vec<_> = row.split('\t').collect();
        let bytes = (0..f[0].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&f[0][i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let input = core::str::from_utf8(&bytes).unwrap();
        let parsed = wgc_gps::parse_week_tow(input);
        let good = if f[1] == "1" {
            parsed
                .map(|n| n.to_bits() == u64::from_str_radix(f[2], 16).unwrap())
                .unwrap_or(false)
        } else {
            parsed.is_err()
        };
        if !good {
            errors.push(format!("{input:?}: {parsed:?}"));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn calendar_wide_epoch_components_and_year_match_independent_cspice() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let rows = include_str!("reference/wgc-gps-input/review-wide.tsv");
    assert_eq!(rows.lines().count(), 20491);
    for row in rows.lines().skip(1) {
        let f: Vec<_> = row.split('\t').collect();
        let system = match f[0] {
            "UTC" => WgcCalendarSystem::Utc,
            "TDB" => WgcCalendarSystem::Tdb,
            "TT" => WgcCalendarSystem::Tt,
            "TDT" => WgcCalendarSystem::Tdt,
            "GPS" => WgcCalendarSystem::Gps,
            _ => panic!("unknown system"),
        };
        let et =
            NaifEt::from_seconds(f64::from_bits(u64::from_str_radix(f[2], 16).unwrap())).unwrap();
        let text = if f[1] == "DOY" {
            wgc_gps::et_to_calendar_doy(et, system, &lsk)
        } else {
            wgc_gps::et_to_calendar_ymd(et, system, &lsk)
        }
        .unwrap();
        let (year, suffix) = text.split_once('-').unwrap();
        let year_value = if let Some(n) = year.strip_suffix(" BC") {
            1 - n.parse::<i32>().unwrap()
        } else {
            year.strip_suffix(" AD")
                .unwrap_or(year)
                .parse::<i32>()
                .unwrap()
        };
        assert_eq!(year_value, f[3].parse::<i32>().unwrap(), "{row}");
        assert_eq!(suffix, f[4], "{row}");
    }
}

#[test]
fn utc_calendar_matches_cspice_with_altered_leap_seconds() {
    use stkcraft_time::{contract::NaifEt, wgc_gps::WgcCalendarSystem};
    let base = include_str!("../src/naif0012.tls");
    let offsets = [35, 36, 38, 40, 0, -100];
    let tables: Vec<_> = offsets
        .iter()
        .map(|n| {
            LeapSecondTable::from_lsk(
                &base.replace("37,   @2017-JAN-1", &format!("{n}, @2017-JAN-1")),
            )
            .unwrap()
        })
        .collect();
    let rows = include_str!("reference/wgc-gps-input/review-altered-leaps.tsv");
    assert_eq!(rows.lines().count(), 49249);
    for row in rows.lines().skip(1) {
        let f: Vec<_> = row.split('\t').collect();
        let offset = f[0].parse::<i32>().unwrap();
        let lsk = &tables[offsets.iter().position(|n| *n == offset).unwrap()];
        let et =
            NaifEt::from_seconds(f64::from_bits(u64::from_str_radix(f[2], 16).unwrap())).unwrap();
        let text = if f[1] == "DOY" {
            wgc_gps::et_to_calendar_doy(et, WgcCalendarSystem::Utc, lsk)
        } else {
            wgc_gps::et_to_calendar_ymd(et, WgcCalendarSystem::Utc, lsk)
        }
        .unwrap();
        assert_eq!(text, f[3], "{row}");
    }
}
