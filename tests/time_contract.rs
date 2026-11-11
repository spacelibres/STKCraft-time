//! Explicit model contracts, checked conversion, and independent reference data.

use stkcraft_time::contract::{CalculationProfile, ScientificContext, TimeContext};
use stkcraft_time::{Calendar, Epoch, LeapSecondSource, LeapSecondTable, UtcDateTime};

#[test]
fn default_context_is_naif_and_preserves_cspice_vectors() {
    assert_eq!(CalculationProfile::default(), CalculationProfile::NaifSpice);
    let lsk = LeapSecondTable::builtin().unwrap();
    let context = TimeContext::new(&lsk);
    for row in include_str!("golden/utc_to_et.csv").lines().skip(1) {
        let (utc, expected) = row.split_once(',').unwrap();
        let parts: Vec<_> = utc.split(['-', 'T', ':']).collect();
        let dt = UtcDateTime::new(
            parts[0].parse().unwrap(),
            parts[1].parse().unwrap(),
            parts[2].parse().unwrap(),
            parts[3].parse().unwrap(),
            parts[4].parse().unwrap(),
            parts[5].parse().unwrap(),
        );
        let et = context.utc_to_et(&dt, Calendar::Gregorian).unwrap();
        assert!(
            (et.seconds() - expected.parse::<f64>().unwrap()).abs() < 1e-6,
            "{row}"
        );
    }
    assert_eq!(context.metadata().leap_entry_dates, lsk.coverage());
    assert_eq!(context.metadata().leap_policy, lsk.policy());
}

#[test]
fn typed_et_inverse_returns_errors_instead_of_panicking() {
    use stkcraft_time::contract::NaifEt;
    let lsk = LeapSecondTable::builtin().unwrap();
    let context = TimeContext::new(&lsk);
    assert!(NaifEt::from_seconds(f64::NAN).is_err());
    assert!(NaifEt::from_seconds(f64::INFINITY).is_err());
    assert!(context
        .to_epoch(NaifEt::from_seconds(1e30).unwrap())
        .is_err());
    let epoch = Epoch::from_parts(43_167, 816_000_000);
    let et = context.to_et(epoch).unwrap();
    assert!(
        (context.to_epoch(et).unwrap() - epoch)
            .total_nanoseconds()
            .abs()
            <= 1
    );
}

#[test]
fn scientific_tt_does_not_inherit_modified_naif_constants() {
    let lsk = LeapSecondTable::builtin()
        .unwrap()
        .with_constants(33.0, 0.0, 0.0, 0.0, 0.0);
    let epoch = Epoch::from_parts(43_167, 816_000_000);
    assert!((TimeContext::new(&lsk).to_et(epoch).unwrap().seconds() - 0.816).abs() < 1e-10);
    let scientific = ScientificContext::new();
    assert!(scientific.tt_seconds(epoch).unwrap().abs() < 1e-10);
    let result = scientific.to_tdb(epoch).unwrap();
    assert_eq!(result.profile(), CalculationProfile::IauScientific);
    // PyERFA 2.0.1.5 dtdb(2451545, 0, 0, 0, 0, 0), independently evaluated.
    assert!((result.seconds() - (-9.930719894379447e-5)).abs() < 5e-9);
}

#[test]
fn scientific_model_bounds_and_invalid_parameters_are_enforced() {
    let scientific = ScientificContext::new();
    assert!(scientific.to_tdb(Epoch::from_parts(i64::MAX, 0)).is_err());
    assert!(scientific
        .to_tdb_at_observer(
            Epoch::from_parts(43_167, 816_000_000),
            f64::NAN,
            0.0,
            0.0,
            0.0
        )
        .is_err());
}

#[test]
fn naif_context_does_not_silently_select_historical_sofa_utc() {
    let lsk = LeapSecondTable::builtin()
        .unwrap()
        .with_source(LeapSecondSource::Sofa);
    let context = TimeContext::new(&lsk);
    assert!(context
        .utc_to_et(
            &UtcDateTime::new(1965, 1, 1, 0, 0, 0.0),
            Calendar::Gregorian
        )
        .is_err());
}

#[test]
fn scientific_context_matches_independent_erfa_epoch_vectors() {
    let context = ScientificContext::new();
    let mut count = 0;
    for row in include_str!("golden/tcg_tcb.csv").lines() {
        if row.starts_with('#') || row.starts_with("tai,") || row.is_empty() {
            continue;
        }
        let columns: Vec<_> = row.split(',').collect();
        let fields: Vec<_> = columns[0].split(['-', 'T', ':']).collect();
        let jdn = stkcraft_time::calendar::calendar_to_jdn(
            fields[0].parse().unwrap(),
            fields[1].parse().unwrap(),
            fields[2].parse().unwrap(),
            Calendar::Gregorian,
        )
        .unwrap();
        let hour: i64 = fields[3].parse().unwrap();
        let minute: i64 = fields[4].parse().unwrap();
        let second: f64 = fields[5].parse().unwrap();
        let epoch = Epoch::from_parts(
            (jdn - 2_451_545) * 86400 + hour * 3600 + minute * 60 + second.floor() as i64,
            ((second - second.floor()) * 1e9).round() as i64,
        );
        let expected_tt: f64 = columns[1].parse().unwrap();
        let expected_tdb: f64 = columns[2].parse().unwrap();
        // The output is absolute binary64 seconds; 250 ns bounds its
        // representation spacing over this already committed reference grid.
        assert!(
            (context.tt_seconds(epoch).unwrap() - expected_tt).abs() <= 2.5e-7,
            "{row}"
        );
        assert!(
            (context.to_tdb(epoch).unwrap().seconds() - expected_tdb).abs() <= 2.5e-7,
            "{row}"
        );
        count += 1;
    }
    assert!(count >= 950);
}

#[test]
fn scientific_context_enforces_integer_epoch_domain_at_nanosecond_edges() {
    let context = ScientificContext::new();
    for (jd, direction) in [(2_433_283_i64, -1), (2_469_808_i64, 1)] {
        // TT midnight at each published endpoint, expressed in internal TAI.
        let boundary = Epoch::from_parts((jd - 2_451_545) * 86400 - 33, 816_000_000);
        assert!(context.to_tdb(boundary).is_ok());
        assert!(context
            .to_tdb(boundary + stkcraft_time::Duration::from_parts(0, direction))
            .is_err());
    }
}

#[test]
fn checked_civil_output_never_wraps_years() {
    use stkcraft_time::contract::NaifEt;
    let lsk = LeapSecondTable::builtin().unwrap();
    for calendar in [Calendar::Gregorian, Calendar::Julian, Calendar::Mixed] {
        assert!(TimeContext::new(&lsk)
            .et_to_utc(NaifEt::from_seconds(1e18).unwrap(), calendar)
            .is_err());
    }
}

#[test]
fn fractional_utc_and_numeric_tai_follow_their_cspice_entry_points() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let utc = UtcDateTime::new(2024, 4, 8, 18, 0, 0.123456789);
    let epoch = stkcraft_time::convert::utc_to_tai(&utc, Calendar::Gregorian, &lsk).unwrap();
    // Independent CSPICE N0067 str2et("2024-04-08T18:00:00.123456789"),
    // using naif0012.tls. This case exposed millimetre geometry differences.
    let expected = 765_871_269.309_107_2_f64;
    assert_eq!(
        TimeContext::new(&lsk)
            .utc_to_et(&utc, Calendar::Gregorian)
            .unwrap()
            .seconds(),
        expected
    );
    // CSPICE UNITIM for the explicitly supplied numeric TAI input has one
    // different bit. This is input quantization, not a different TDB model.
    let numeric_tai_reference = 765_871_269.309_107_3_f64;
    assert_eq!(
        TimeContext::new(&lsk).to_et(epoch).unwrap().seconds(),
        numeric_tai_reference
    );
}

#[test]
fn checked_utc_contract_matches_pinned_cspice_input_arithmetic() {
    let lsk = LeapSecondTable::builtin().unwrap();
    let context = TimeContext::new(&lsk);
    let mut count = 0;
    for line in include_str!("golden/contract_utc.csv").lines() {
        if line.starts_with('#') || line.starts_with("year,") || line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split(',').collect();
        let utc = UtcDateTime::new(
            fields[0].parse().unwrap(),
            fields[1].parse().unwrap(),
            fields[2].parse().unwrap(),
            fields[3].parse().unwrap(),
            fields[4].parse().unwrap(),
            fields[5].parse().unwrap(),
        );
        let expected = u64::from_str_radix(fields[7], 16).unwrap();
        assert_eq!(
            context
                .utc_to_et(&utc, Calendar::Gregorian)
                .unwrap()
                .seconds()
                .to_bits(),
            expected,
            "{line}"
        );
        count += 1;
    }
    assert!(count >= 910);
}

#[test]
fn frozen_kernel_policy_uses_only_its_effective_leap_entries() {
    use stkcraft_time::LeapSecondPolicy;
    let lsk = LeapSecondTable::builtin().unwrap();
    let jdn = stkcraft_time::calendar::gregorian_to_jdn(2016, 1, 1);
    let frozen = lsk
        .clone()
        .with_policy(LeapSecondPolicy::FrozenAfter { jdn });
    let effective = LeapSecondTable::new(
        lsk.entries()
            .iter()
            .copied()
            .filter(|entry| entry.jdn <= jdn)
            .collect(),
    )
    .unwrap();
    for utc in [
        UtcDateTime::new(2016, 12, 31, 23, 59, 59.123456789),
        UtcDateTime::new(2024, 4, 8, 18, 0, 0.123456789),
    ] {
        assert_eq!(
            TimeContext::new(&frozen)
                .utc_to_et(&utc, Calendar::Gregorian)
                .unwrap(),
            TimeContext::new(&effective)
                .utc_to_et(&utc, Calendar::Gregorian)
                .unwrap()
        );
    }
}

#[test]
fn checked_contract_rejects_unrepresentable_public_table_anchors() {
    use stkcraft_time::{contract::NaifEt, LeapEntry};
    let jdn = i64::MIN.div_euclid(86400) + 1 + 2_451_545;
    let lsk = LeapSecondTable::new(vec![LeapEntry { jdn, delta_at: 10 }]).unwrap();
    let context = TimeContext::new(&lsk);
    assert!(context
        .utc_to_et(
            &UtcDateTime::new(2024, 1, 1, 0, 0, 0.0),
            Calendar::Gregorian
        )
        .is_err());
    assert!(context
        .et_to_utc(NaifEt::from_seconds(-2e18).unwrap(), Calendar::Gregorian)
        .is_err());
}
