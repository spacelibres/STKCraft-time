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
