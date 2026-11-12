//! Native FURNSH/GDPOOL literal grammar, without modified inputs or tolerances.
use stkcraft_time::{kernel_date::parse_kernel_date, TimeError};

#[test]
fn native_kernel_date_grammar_and_bits_exact() {
    let mut failures = Vec::new();
    let (mut accepted, mut rejected) = (0, 0);
    for row in include_str!("reference/kernel-dates/native.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = row.split('\t').collect();
        let literal = fields[0];
        let result = parse_kernel_date(literal);
        if fields[1] == "OK" {
            accepted += 1;
            let hex = fields[2].strip_prefix("N|1|").unwrap();
            let expected = u64::from_str_radix(hex, 16).unwrap();
            match result {
                Ok(date) if date.seconds_past_j2000.to_bits() == expected => {}
                Ok(date) => failures.push(format!(
                    "{literal}: {:016x}; native {hex}",
                    date.seconds_past_j2000.to_bits()
                )),
                Err(error) => failures.push(format!("{literal}: {error}; native {hex}")),
            }
        } else {
            rejected += 1;
            if !matches!(result, Err(TimeError::KernelDateUnsupported)) {
                failures.push(format!("{literal}: {result:?}; native {}", fields[1]));
            }
        }
    }
    assert_eq!((accepted, rejected), (345, 71));
    assert!(
        failures.is_empty(),
        "{} native mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
