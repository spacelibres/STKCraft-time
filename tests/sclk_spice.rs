//! Independent CSPICE references, including TDT/ET rounding counterexamples.
use stkcraft_time::{sclk::SclkKernel, LeapSecondTable};

fn check(direction: &str) {
    let lsk = LeapSecondTable::builtin().unwrap();
    let mro = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let juice = SclkKernel::parse(include_str!("kernels/juice_step_250412_v01.tsc"), -28).unwrap();
    let mut count = 0;
    for line in include_str!("golden/sclk.csv")
        .lines()
        .filter(|x| !x.starts_with('#'))
        .skip(1)
    {
        let fields: Vec<_> = line.split(',').collect();
        if fields[1] != direction {
            continue;
        }
        let clock = if fields[0] == "-74" { &mro } else { &juice };
        let input = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
        let expected = u64::from_str_radix(fields[3], 16).unwrap();
        let actual = if direction == "forward" {
            clock.ticks_to_et(input, &lsk)
        } else {
            clock.et_to_ticks(input, &lsk)
        }
        .unwrap();
        assert_eq!(actual.to_bits(), expected, "{line}: actual {actual:?}");
        count += 1;
    }
    assert!(count > 2000, "incomplete {direction} fixture: {count}");
}

#[test]
fn tdt_clock_forward_matches_cspice_bits() {
    check("forward");
}

#[test]
fn tdt_clock_inverse_matches_cspice_bits() {
    check("inverse");
}
