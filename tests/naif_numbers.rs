//! Regression for a decimal token in ESA's official JUICE SCLK kernel.

use stkcraft_time::SclkKernel;

#[test]
fn coefficient_boundary_matches_naif_decimal_parsing() {
    // Token from juice_step_250412_v01.tsc. Expected bits were obtained
    // with CSPICE N0067 PRSDP through SpiceyPy, not a Rust calculation.
    let text = r#"KPL/SCLK
\begindata
SCLK01_N_FIELDS_999 = ( 1 )
SCLK01_MODULI_999 = ( 100000000000000 )
SCLK01_COEFFICIENTS_999 = (
    4.9260090394032E+13 0 1
)
\begintext
"#;
    let kernel = SclkKernel::parse(text, -999).unwrap();
    assert_eq!(kernel.coefficients()[0].ticks.to_bits(), 0x42c666a11842d801);
}
