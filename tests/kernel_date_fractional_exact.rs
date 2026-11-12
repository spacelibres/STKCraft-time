//! Independent N0067 FURNSH/GDPOOL counterexamples from GeoCalc issue #1.
use stkcraft_time::kernel_date::parse_kernel_date;

#[test]
fn native_fractional_kernel_dates_preserve_arithmetic_order() {
    let values = [
        ("@2000-JAN-1-23:59:59.999999999", 0x40e5_17ff_ffff_ff77_u64),
        ("@2000-JAN-01-3:10:39.221", 0xc0df_0431_db22_d0e5),
        ("@2000-JAN-01-23:59:59.999999", 0x40e5_17ff_fffd_e721),
        ("@01-JAN-2000-3:10:39.221", 0xc0df_0431_db22_d0e5),
        ("@01-JAN-2000-23:59:59.999999", 0x40e5_17ff_fffd_e721),
        ("@JAN-01-2000-3:10:39.221", 0xc0df_0431_db22_d0e5),
        ("@JAN-01-2000-23:59:59.999999", 0x40e5_17ff_fffd_e721),
    ];
    let mut failures = Vec::new();
    for (literal, expected) in values {
        let actual = parse_kernel_date(literal)
            .unwrap()
            .seconds_past_j2000
            .to_bits();
        if actual != expected {
            failures.push(format!(
                "{literal}: actual {actual:016x}; native {expected:016x}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} native mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
