//! Independent native TTRANS outputs with original inputs, no tolerance.
use stkcraft_time::{
    contract::{NaifEt, TimeContext},
    LeapEntry, LeapSecondTable,
};
fn number(text: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(text, 16).unwrap())
}

#[test]
fn native_jdutc_constructible_profiles_and_boundaries_exact() {
    let mut lsk = None;
    let mut current = "";
    let mut failures = Vec::new();
    let (mut profiles, mut numeric, mut errors, mut constructor_rejections) = (0, 0, 0, 0);
    for row in include_str!("reference/native-jdutc/native.tsv").lines() {
        let f: Vec<_> = row.split('|').collect();
        match f[0] {
            "C" => {
                assert_eq!(f.last(), Some(&"OK"));
                let count = f[2].parse::<usize>().unwrap();
                assert_eq!(f.len(), 9 + count * 2);
                let entries = (0..count)
                    .map(|i| LeapEntry {
                        jdn: f[8 + 2 * i].parse().unwrap(),
                        delta_at: f[9 + 2 * i].parse().unwrap(),
                    })
                    .collect();
                let mut table = LeapSecondTable::new(entries);
                if let Ok(table) = &mut table {
                    table.delta_t_a = number(f[3]);
                    table.k = number(f[4]);
                    table.eb = number(f[5]);
                    table.m0 = number(f[6]);
                    table.m1 = number(f[7]);
                }
                lsk = Some(table);
                current = f[1];
                profiles += 1;
            }
            "R" => {
                assert_eq!(f[1], current);
                match lsk.as_ref().unwrap() {
                    Ok(table) => {
                        let result = TimeContext::new(table)
                            .et_to_spice_julian_utc(NaifEt::from_seconds(number(f[2])).unwrap());
                        let actual = match result {
                            Ok(value) => {
                                numeric += 1;
                                format!("N|{:016x}", value.days().to_bits())
                            }
                            Err(error) => {
                                errors += 1;
                                format!("E|{error}")
                            }
                        };
                        if actual != f[3..].join("|") {
                            failures.push(format!("{row}: actual {actual}"));
                        }
                    }
                    Err(error) => {
                        // This malformed table cannot enter the new API.
                        // Preserve every native error record and report the
                        // existing constructor's distinct error contract;
                        // do not claim native short-symbol equality here.
                        assert_eq!(current, "invalid_tai_order");
                        assert!(matches!(error, stkcraft_time::TimeError::LskParse));
                        assert_eq!(f[3..].join("|"), "E|SPICE(BADLEAPSECONDS)");
                        constructor_rejections += 1;
                    }
                }
            }
            other => panic!("Unknown native row: {other}"),
        }
    }
    assert_eq!(
        (profiles, numeric, errors, constructor_rejections),
        (13, 18493, 185, 185)
    );
    assert!(
        failures.is_empty(),
        "{} native mismatches:\n{}",
        failures.len(),
        failures
            .iter()
            .take(16)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn native_jdutc_original_lsk_historical_gap_exact() {
    let lsk =
        LeapSecondTable::from_lsk(include_str!("reference/native-jdutc/naif0012.tls")).unwrap();
    let mut count = 0;
    for row in include_str!("reference/native-jdutc/original-lsk.tsv").lines() {
        let f: Vec<_> = row.split('|').collect();
        let result = TimeContext::new(&lsk)
            .et_to_spice_julian_utc(NaifEt::from_seconds(number(f[0])).unwrap())
            .unwrap();
        assert_eq!(result.days().to_bits(), number(f[1]).to_bits(), "{row}");
        if count < 9 {
            assert!(
                TimeContext::new(&lsk)
                    .et_to_utc(
                        NaifEt::from_seconds(number(f[0])).unwrap(),
                        stkcraft_time::Calendar::Gregorian
                    )
                    .is_err(),
                "Legacy coverage changed: {row}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 13);
}

#[test]
fn native_jdutc_requires_explicit_raw_lsk_model() {
    use stkcraft_time::{contract::ContractError, LeapSecondPolicy, LeapSecondSource};
    let table = LeapSecondTable::builtin().unwrap();
    let et = NaifEt::from_seconds(-1e9).unwrap();
    for lsk in [
        table.clone().with_source(LeapSecondSource::Sofa),
        table
            .clone()
            .with_policy(LeapSecondPolicy::FrozenAfter { jdn: 2451545 }),
    ] {
        assert!(matches!(
            TimeContext::new(&lsk).et_to_spice_julian_utc(et),
            Err(ContractError::NaifKernelModelRequired)
        ));
    }
    let mut invalid = table;
    invalid.k = f64::NAN;
    assert!(matches!(
        TimeContext::new(&invalid).et_to_spice_julian_utc(et),
        Err(ContractError::NonFinite)
    ));
}
