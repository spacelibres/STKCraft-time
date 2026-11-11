//! Kernel parser validation tests.
//!
//! These cover the semantic checks that binary-search and interpolation code
//! depends on: finite constants, valid dates, coefficient ordering and rates.

use stkcraft_time::lsk::parse_lsk;
use stkcraft_time::sclk::SclkKernel;
use stkcraft_time::TimeError;

const BAD_NAN: &str = r#"
KPL/LSK

\begindata
DELTET/K = NaN
DELTET/DELTA_AT = ( 10 @1972-JAN-01 )
\begintext
"#;

const MULTI_BLOCK_SCLK: &str = r#"
KPL/SCLK

\begindata
SCLK_DATA_TYPE_999       = ( 1 )
SCLK01_N_FIELDS_999      = ( 2 )
SCLK01_MODULI_999        = ( 100 10 )
\begintext

\begindata
SCLK01_OFFSETS_999       = ( 0 0 )
SCLK01_COEFFICIENTS_999  = ( 0 0 1 )
\begintext
"#;

#[test]
fn lsk_rejects_non_finite_constants() {
    assert_eq!(parse_lsk(BAD_NAN).unwrap_err(), TimeError::LskParse);
}

#[test]
fn lsk_rejects_invalid_calendar_dates() {
    let text = BAD_NAN
        .replace("NaN", "0.001657")
        .replace("JAN-01", "FEB-31");

    assert_eq!(parse_lsk(&text).unwrap_err(), TimeError::LskParse);
}

#[test]
fn lsk_reads_all_data_blocks() {
    let text = r#"
KPL/LSK

\begindata
DELTET/K = 0.001657
\begintext

\begindata
DELTET/DELTA_AT = ( 10 @1972-JAN-01, 37 @2017-JAN-01 )
\begintext
"#;

    let table = parse_lsk(text).unwrap();
    assert_eq!(table.coverage(), (2_441_318, 2_457_755));
}

#[test]
fn sclk_reads_all_data_blocks() {
    let kernel = SclkKernel::parse(MULTI_BLOCK_SCLK, -999).unwrap();
    assert_eq!(kernel.n_fields(), 2);
    assert_eq!(kernel.ticks_from_fields(&[1, 2]).unwrap(), 12.0);
}

#[test]
fn sclk_rejects_non_finite_coefficients() {
    let text = MULTI_BLOCK_SCLK.replace("0 0 1", "0 0 NaN");
    assert!(SclkKernel::parse(&text, -999).is_err());
}
