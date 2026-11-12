//! Every native raw-table result, including deferred malformed-table errors.
use stkcraft_time::{
    contract::{NaifDeltet, NaifEt, NaifUtcContext},
    LeapEntry,
};

fn number(text: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(text, 16).unwrap())
}

fn native_result(entries: &[LeapEntry], constants: [f64; 5], et: f64) -> String {
    let [delta_t_a, k, eb, m0, m1] = constants;
    let context = NaifUtcContext::from_raw_parts(
        entries,
        NaifDeltet {
            delta_t_a,
            k,
            eb,
            m0,
            m1,
        },
    )
    .expect("Every captured native pool assignment succeeded");
    match context.et_to_spice_julian_utc(NaifEt::from_seconds(et).unwrap()) {
        Ok(value) => format!("N|{:016x}", value.days().to_bits()),
        Err(error) => format!("E|{error}"),
    }
}

#[test]
fn raw_native_jdutc_preserves_all_values_errors_order_and_capacity() {
    let mut entries = Vec::new();
    let mut constants = [0.; 5];
    let mut current = "";
    let (mut profiles, mut numeric, mut errors, mut mismatches) = (0, 0, 0, 0);
    let mut examples = Vec::new();
    for source in [
        include_str!("reference/native-jdutc/native.tsv"),
        include_str!("reference/native-jdutc/raw.tsv"),
    ] {
        for row in source.lines() {
            let f: Vec<_> = row.split('|').collect();
            match f[0] {
                "C" => {
                    assert_eq!(f.last(), Some(&"OK"));
                    let count = f[2].parse::<usize>().unwrap();
                    assert_eq!(f.len(), 9 + count * 2);
                    entries = (0..count)
                        .map(|i| LeapEntry {
                            jdn: f[8 + 2 * i].parse().unwrap(),
                            delta_at: f[9 + 2 * i].parse().unwrap(),
                        })
                        .collect();
                    constants = core::array::from_fn(|i| number(f[3 + i]));
                    current = f[1];
                    profiles += 1;
                }
                "R" => {
                    assert_eq!(f[1], current);
                    assert_eq!(f.len(), 5);
                    match f[3] {
                        "N" => numeric += 1,
                        "E" => errors += 1,
                        other => panic!("Unknown native result: {other}"),
                    }
                    let actual = native_result(&entries, constants, number(f[2]));
                    let expected = f[3..].join("|");
                    if actual != expected {
                        mismatches += 1;
                        if examples.len() < 12 {
                            examples.push(format!("{row}: actual {actual}"));
                        }
                    }
                }
                other => panic!("Unknown native row: {other}"),
            }
        }
    }
    assert_eq!((profiles, numeric, errors), (17, 22292, 4539));
    assert_eq!(mismatches, 0, "{}", examples.join("\n"));
}

#[test]
fn raw_context_preserves_constructor_separation_and_checked_domain() {
    use stkcraft_time::{contract::ContractError, LeapSecondTable, TimeError};
    let constants = NaifDeltet {
        delta_t_a: 32.184,
        k: 0.001657,
        eb: 0.01671,
        m0: 6.239996,
        m1: 1.99096871e-7,
    };
    let invalid = [
        LeapEntry {
            jdn: 2441318,
            delta_at: 100000,
        },
        LeapEntry {
            jdn: 2441319,
            delta_at: 0,
        },
    ];
    assert!(matches!(
        LeapSecondTable::new(invalid.to_vec()),
        Err(TimeError::LskParse)
    ));
    let context = NaifUtcContext::from_raw_parts(&invalid, constants).unwrap();
    assert!(matches!(
        context.et_to_spice_julian_utc(NaifEt::from_seconds(0.).unwrap()),
        Err(ContractError::InvalidNaifLeapSeconds)
    ));
    assert!(matches!(
        NaifUtcContext::from_raw_parts(&[], constants),
        Err(ContractError::Time(TimeError::LskMissing(
            "DELTET/DELTA_AT"
        )))
    ));
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..5 {
            let mut values = [
                constants.delta_t_a,
                constants.k,
                constants.eb,
                constants.m0,
                constants.m1,
            ];
            values[index] = value;
            let [delta_t_a, k, eb, m0, m1] = values;
            assert!(matches!(
                NaifUtcContext::from_raw_parts(
                    &invalid,
                    NaifDeltet {
                        delta_t_a,
                        k,
                        eb,
                        m0,
                        m1
                    }
                ),
                Err(ContractError::NonFinite)
            ));
        }
    }
    for jdn in [i64::MIN, i64::MAX] {
        let entries = [LeapEntry { jdn, delta_at: 10 }];
        let context = NaifUtcContext::from_raw_parts(&entries, constants).unwrap();
        assert!(matches!(
            context.et_to_spice_julian_utc(NaifEt::from_seconds(0.).unwrap()),
            Err(ContractError::EpochOutOfRange)
        ));
    }
}
