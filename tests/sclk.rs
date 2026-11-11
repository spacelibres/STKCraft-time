//! SCLK type-1 parsing and conversion tests.
//!
//! The kernel here is synthetic and hand-checkable. The real cross-check
//! against CSPICE `sct2e` is `tools/validate_sclk.py`.

use stkcraft_time::sclk::{SclkKernel, SclkTimeSystem};
use stkcraft_time::LeapSecondTable;

const SYNTH: &str = r#"
KPL/SCLK

\begindata

SCLK_DATA_TYPE_999       = ( 1 )
SCLK01_TIME_SYSTEM_999   = ( 1 )
SCLK01_N_FIELDS_999      = ( 2 )
SCLK01_MODULI_999        = ( 1000 100 )
SCLK01_OFFSETS_999       = ( 0 0 )
SCLK01_OUTPUT_DELIM_999  = ( 2 )
SCLK01_COEFFICIENTS_999  = (
    0.0000000000000E+00     0.0000000000000E+00     1.0000000000000E+00
    1.0000000000000E+05     1.0000000000000E+03     1.0000000000000E+00
)

\begintext
"#;

const OFFSET_SCLK: &str = r#"
KPL/SCLK

\begindata

SCLK_DATA_TYPE_999       = ( 1 )
SCLK01_TIME_SYSTEM_999   = ( 1 )
SCLK01_N_FIELDS_999      = ( 3 )
SCLK01_MODULI_999        = ( 65536 60 800 )
SCLK01_OFFSETS_999       = ( 0 0 1 )
SCLK01_OUTPUT_DELIM_999  = ( 2 )
SCLK01_COEFFICIENTS_999  = (
    0.0000000000000E+00     0.0000000000000E+00     2.8800000000000E+03
)

\begintext
"#;

#[test]
fn parses_the_kernel_for_either_id_sign() {
    for id in [999, -999] {
        let kernel = SclkKernel::parse(SYNTH, id).unwrap();
        assert_eq!(kernel.n_fields(), 2);
        assert_eq!(kernel.moduli(), &[1000, 100]);
        assert_eq!(kernel.offsets(), &[0, 0]);
        assert_eq!(kernel.time_system(), SclkTimeSystem::Tdb);
        assert_eq!(kernel.data_type(), 1);
        assert!((kernel.ticks_per_msb() - 100.0).abs() < 1e-12);
    }
}

#[test]
fn fields_and_ticks_are_inverse() {
    let kernel = SclkKernel::parse(SYNTH, -999).unwrap();

    // ticks = f0 * 100 + f1.
    assert!((kernel.ticks_from_fields(&[123, 45]).unwrap() - 12_345.0).abs() < 1e-9);
    assert_eq!(kernel.fields_from_ticks(12_345.0).unwrap(), vec![123, 45]);
}

#[test]
fn parallel_seconds_follows_the_piecewise_linear_map() {
    let kernel = SclkKernel::parse(SYNTH, -999).unwrap();

    // First segment: rate 1 per MSB count, 100 ticks per MSB count.
    assert!((kernel.parallel_seconds(0.0).unwrap() - 0.0).abs() < 1e-12);
    assert!((kernel.parallel_seconds(12_345.0).unwrap() - 123.45).abs() < 1e-9);
    assert!((kernel.parallel_seconds(100_200.0).unwrap() - 1002.0).abs() < 1e-9);

    // Inverse.
    assert!((kernel.ticks_from_parallel(123.45).unwrap() - 12_345.0).abs() < 1e-6);
    assert!((kernel.ticks_from_parallel(1002.0).unwrap() - 100_200.0).abs() < 1e-6);
}

#[test]
fn clock_strings_round_trip() {
    let kernel = SclkKernel::parse(SYNTH, -999).unwrap();

    assert_eq!(kernel.parse_string("123:45").unwrap(), vec![123, 45]);
    assert_eq!(kernel.format_string(&[123, 45]).unwrap(), "123:45");
}

#[test]
fn tdb_clock_maps_ticks_directly_to_et() {
    let kernel = SclkKernel::parse(SYNTH, -999).unwrap();
    let lsk = LeapSecondTable::builtin().unwrap();

    let et = kernel.ticks_to_et(12_345.0, &lsk).unwrap();
    assert!((et - 123.45).abs() < 1e-9);

    let ticks = kernel.et_to_ticks(et, &lsk).unwrap();
    assert!((ticks - 12_345.0).abs() < 1e-6);
}

#[test]
fn nonzero_offsets_follow_naif_semantics() {
    let kernel = SclkKernel::parse(OFFSET_SCLK, -999).unwrap();

    // Voyager-like third field: the first displayed count is 1, while the
    // encoded counter starts at 0.
    assert_eq!(kernel.ticks_from_fields(&[0, 0, 1]).unwrap(), 0.0);
    assert_eq!(kernel.fields_from_ticks(0.0).unwrap(), vec![0, 0, 1]);

    assert_eq!(kernel.ticks_from_fields(&[0, 0, 2]).unwrap(), 1.0);
    assert_eq!(kernel.fields_from_ticks(1.0).unwrap(), vec![0, 0, 2]);

    assert_eq!(kernel.ticks_from_fields(&[0, 0, 800]).unwrap(), 799.0);
    assert_eq!(kernel.fields_from_ticks(799.0).unwrap(), vec![0, 0, 800]);
}

#[test]
fn rejects_zero_rate_and_unsorted_coefficients() {
    let zero_rate = SYNTH.replace(
        "0.0000000000000E+00     0.0000000000000E+00     1.0000000000000E+00",
        "0.0000000000000E+00     0.0000000000000E+00     0.0000000000000E+00",
    );
    assert!(SclkKernel::parse(&zero_rate, -999).is_err());

    let unsorted = SYNTH.replace(
        "1.0000000000000E+05     1.0000000000000E+03     1.0000000000000E+00",
        "0.0000000000000E+00     1.0000000000000E+03     1.0000000000000E+00",
    );
    assert!(SclkKernel::parse(&unsorted, -999).is_err());
}

#[test]
fn partition_aware_strings_are_rejected_but_ticks_still_work() {
    let partitioned = SYNTH.replace(
        "\\begintext",
        "SCLK_PARTITION_START_999 = ( 10.0 )\nSCLK_PARTITION_END_999 = ( 100.0 )\n\n\\begintext",
    );
    let kernel = SclkKernel::parse(&partitioned, -999).unwrap();
    let lsk = LeapSecondTable::builtin().unwrap();

    assert_eq!(
        kernel.parse_string("123:45"),
        Err(stkcraft_time::TimeError::SclkPartitionsUnsupported)
    );
    assert_eq!(
        kernel.format_string(&[123, 45]),
        Err(stkcraft_time::TimeError::SclkPartitionsUnsupported)
    );
    assert_eq!(
        kernel.ticks_from_fields(&[123, 45]),
        Err(stkcraft_time::TimeError::SclkPartitionsUnsupported)
    );

    // Encoded ticks remain usable even when partition-aware strings are not.
    assert!(kernel.ticks_to_et(12_345.0, &lsk).is_ok());
}

#[test]
fn minimum_clock_id_does_not_panic() {
    assert!(SclkKernel::parse(SYNTH, i32::MIN).is_err());
}

#[test]
fn non_finite_numeric_inputs_are_rejected() {
    let kernel = SclkKernel::parse(OFFSET_SCLK, -999).unwrap();

    assert!(kernel.parallel_seconds(f64::NAN).is_err());
    assert!(kernel.parallel_seconds(f64::INFINITY).is_err());
    assert!(kernel.ticks_from_parallel(f64::NAN).is_err());
    assert!(kernel.fields_from_ticks(f64::NAN).is_err());
    assert!(kernel.fields_from_ticks(0.5).is_err());
}
