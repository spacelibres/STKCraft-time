use stkcraft_time::sclk::SclkKernel;

#[test]
fn native_mro_partitioned_clock_strings() {
    let kernel = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let mut decode_count = 0;
    let mut encode_count = 0;
    for row in include_str!("reference/sclk-partition/native.tsv")
        .lines()
        .skip(1)
    {
        let mut columns = row.split('\t');
        let operation = columns.next().unwrap();
        let input = columns.next().unwrap();
        let expected = columns.next().unwrap();
        assert!(columns.next().is_none());
        let actual = match operation {
            "D" => {
                decode_count += 1;
                let ticks = f64::from_bits(u64::from_str_radix(input, 16).unwrap());
                match kernel.decode_native_sclk(ticks) {
                    Ok(value) => format!("S:{value}"),
                    Err(error) => format!("E:{}", error.spice_short()),
                }
            }
            "E" => {
                encode_count += 1;
                match kernel.encode_native_sclk(input) {
                    Ok(value) => format!("T:{:016x}", value.to_bits()),
                    Err(error) => format!("E:{}", error.spice_short()),
                }
            }
            other => panic!("unknown operation {other}"),
        };
        assert_eq!(actual, expected, "{row}");
    }
    assert_eq!((decode_count, encode_count), (224, 111));
}

#[test]
fn native_mro_clock_numeric_and_delimiter_grammar() {
    let kernel = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/extra.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let input = input.replace("\\t", "\t");
        let actual = match kernel.encode_native_sclk(&input) {
            Ok(value) => format!("T:{:016x}", value.to_bits()),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{input:?}");
        count += 1;
    }
    assert_eq!(count, 33);
}

#[test]
fn native_mro_partitioned_clock_et_round_trip() {
    use stkcraft_time::sclk::{SclkClock, SclkMap};
    use stkcraft_time::LeapSecondTable;
    let kernel = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    let clock = SclkClock {
        kernel: &kernel,
        lsk: &lsk,
    };
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/et.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = row.split('\t').collect();
        let et = f64::from_bits(u64::from_str_radix(fields[1], 16).unwrap());
        assert_eq!(
            clock.sclk_to_et(fields[0]).unwrap().to_bits(),
            et.to_bits(),
            "{row}"
        );
        assert_eq!(clock.et_to_sclk(et).unwrap(), fields[2], "{row}");
        count += 1;
    }
    assert_eq!(count, 94);
}

#[test]
fn native_reset_overlap_and_field_offsets() {
    let kernel =
        SclkKernel::parse(include_str!("kernels/synthetic_partition_reset.tsc"), -999).unwrap();
    let (mut decode_count, mut encode_count) = (0, 0);
    for row in include_str!("reference/sclk-partition/reset.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = row.split('\t').collect();
        let actual = if fields[0] == "D" {
            decode_count += 1;
            let ticks = f64::from_bits(u64::from_str_radix(fields[1], 16).unwrap());
            match kernel.decode_native_sclk(ticks) {
                Ok(value) => format!("S:{value}"),
                Err(error) => format!("E:{}", error.spice_short()),
            }
        } else {
            encode_count += 1;
            match kernel.encode_native_sclk(fields[1]) {
                Ok(value) => format!("T:{:016x}", value.to_bits()),
                Err(error) => format!("E:{}", error.spice_short()),
            }
        };
        assert_eq!(actual, fields[2], "{row}");
    }
    assert_eq!((decode_count, encode_count), (22, 16));
}

#[test]
fn native_mro_seeded_clock_string_grammar() {
    use stkcraft_time::sclk::SclkStringContext;
    let kernel = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let mut context = SclkStringContext::default();
    let mut mismatches = Vec::new();
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/fuzz.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual = match kernel.encode_native_sclk_with_context(input, &mut context) {
            Ok(value) => format!("T:{:016x}", value.to_bits()),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        if actual != expected {
            mismatches.push(format!("{input:?}: {expected} != {actual}"));
        }
        count += 1;
    }
    assert_eq!(count, 3000);
    assert!(
        mismatches.is_empty(),
        "{} mismatches: {:?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(12)]
    );
}

#[test]
fn native_synthetic_partition_and_delimiter_grid() {
    use stkcraft_time::sclk::SclkStringContext;
    let text = include_str!("reference/sclk-partition/synthetic-grid.tsc");
    let mut context = SclkStringContext::default();
    let mut mismatches = Vec::new();
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/synthetic-grid.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = row.split('\t').collect();
        let kernel = SclkKernel::parse(text, -fields[0].parse::<i32>().unwrap()).unwrap();
        let actual = if fields[1] == "D" {
            let ticks = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
            match kernel.decode_native_sclk(ticks) {
                Ok(value) => format!("S:{value}"),
                Err(error) => format!("E:{}", error.spice_short()),
            }
        } else {
            match kernel.encode_native_sclk_with_context(fields[2], &mut context) {
                Ok(value) => format!("T:{:016x}", value.to_bits()),
                Err(error) => format!("E:{}", error.spice_short()),
            }
        };
        if actual != fields[3] {
            mismatches.push(format!("{row}: {actual}"));
        }
        count += 1;
    }
    assert_eq!(count, 170);
    assert!(
        mismatches.is_empty(),
        "{} mismatches: {:?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(10)]
    );
}

#[test]
fn native_component_error_history_is_preserved() {
    use stkcraft_time::sclk::SclkStringContext;
    let kernel = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let mut context = SclkStringContext::default();
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/history.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual = match kernel.encode_native_sclk_with_context(input, &mut context) {
            Ok(value) => format!("T:{:016x}", value.to_bits()),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{row}");
        count += 1;
    }
    assert_eq!(count, 6);
}

#[test]
fn native_sclk_sign_nul_and_partition_capacity() {
    let base = include_str!("kernels/synthetic_partition_reset.tsc");
    let negative = base.replace(
        "SCLK01_OFFSETS_999          = ( 0 0 1 )",
        "SCLK01_OFFSETS_999          = ( 0 -1 1 )",
    );
    let negative = SclkKernel::parse(&negative, -999).unwrap();
    let ordinary = SclkKernel::parse(base, -999).unwrap();
    let zeros = vec!["0"; 10000].join(" ");
    let ones = vec!["1"; 10000].join(" ");
    let large = base
        .replace(
            "SCLK_PARTITION_START_999    = ( 6000 0 6000 )",
            &format!("SCLK_PARTITION_START_999 = ( {zeros} )"),
        )
        .replace(
            "SCLK_PARTITION_END_999      = ( 12000 6000 12000 )",
            &format!("SCLK_PARTITION_END_999 = ( {ones} )"),
        );
    let large = SclkKernel::parse(&large, -999).unwrap();
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/special.tsv")
        .lines()
        .skip(1)
    {
        let (case, expected) = row.split_once('\t').unwrap();
        let actual = match case {
            "negative-offset" => match negative.decode_native_sclk(0.0) {
                Ok(value) => format!("S:{value}"),
                Err(error) => format!("E:{}", error.spice_short()),
            },
            "many:decode" => match large.decode_native_sclk(0.0) {
                Ok(value) => format!("S:{value}"),
                Err(error) => format!("E:{}", error.spice_short()),
            },
            "many:encode" => match large.encode_native_sclk("1/0") {
                Ok(value) => format!("T:{:016x}", value.to_bits()),
                Err(error) => format!("E:{}", error.spice_short()),
            },
            _ => match ordinary
                .encode_native_sclk(&case.strip_prefix("nul:").unwrap().replace("\\0", "\0"))
            {
                Ok(value) => format!("T:{:016x}", value.to_bits()),
                Err(error) => format!("E:{}", error.spice_short()),
            },
        };
        assert_eq!(actual, expected, "{case}");
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn native_truncated_multibyte_component_preserves_error_history() {
    use stkcraft_time::sclk::SclkStringContext;
    let kernel = SclkKernel::parse(include_str!("kernels/MRO_SCLKSCET.00120.tsc"), -74).unwrap();
    let mut context = SclkStringContext::default();
    let mut count = 0;
    for row in include_str!("reference/sclk-partition/utf8-history.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let actual = match kernel.encode_native_sclk_with_context(input, &mut context) {
            Ok(value) => format!("T:{:016x}", value.to_bits()),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{input:?}");
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn native_sclk_discrete_and_fractional_partition_boundaries() {
    assert_eq!(
        include_str!("reference/sclk-partition/et-boundaries.tsv")
            .lines()
            .count(),
        13
    );
    use stkcraft_time::sclk::{SclkClock, SclkMap};
    use stkcraft_time::LeapSecondTable;
    let reset =
        SclkKernel::parse(include_str!("kernels/synthetic_partition_reset.tsc"), -999).unwrap();
    let grid = SclkKernel::parse(
        include_str!("reference/sclk-partition/synthetic-grid.tsc"),
        -901,
    )
    .unwrap();
    let lsk = LeapSecondTable::from_lsk(include_str!("../src/naif0012.tls")).unwrap();
    for row in include_str!("reference/sclk-partition/et-boundaries.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = row.split('\t').collect();
        let input = f64::from_bits(u64::from_str_radix(fields[2], 16).unwrap());
        let actual = match fields[0] {
            "D" => SclkClock {
                kernel: &reset,
                lsk: &lsk,
            }
            .et_to_sclk(input)
            .map(|v| format!("S:{v}")),
            "T" => grid
                .ticks_to_et(input, &lsk)
                .map(|v| format!("F:{:016x}", v.to_bits())),
            _ => panic!("unknown operation"),
        };
        if fields[3].starts_with("E:") {
            assert!(actual.is_err(), "{row}: {actual:?}");
        } else {
            assert_eq!(actual.unwrap(), fields[3], "{row}");
        }
    }
}

#[test]
fn native_sclk_large_component_formatting() {
    assert_eq!(
        include_str!("reference/sclk-partition/large-field.tsv")
            .lines()
            .count(),
        6009
    );
    let kernel = SclkKernel::parse(
        include_str!("reference/sclk-partition/large-field.tsc"),
        -902,
    )
    .unwrap();
    for row in include_str!("reference/sclk-partition/large-field.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let ticks = f64::from_bits(u64::from_str_radix(input, 16).unwrap());
        let actual = match kernel.decode_native_sclk(ticks) {
            Ok(value) => format!("S:{value}"),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{row}");
    }
}

#[test]
fn native_sclk_large_offset_formatting() {
    assert_eq!(
        include_str!("reference/sclk-partition/large-offset.tsv")
            .lines()
            .count(),
        1005
    );
    let kernel = SclkKernel::parse(
        include_str!("reference/sclk-partition/large-offset.tsc"),
        -902,
    )
    .unwrap();
    for row in include_str!("reference/sclk-partition/large-offset.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let ticks = f64::from_bits(u64::from_str_radix(input, 16).unwrap());
        let actual = match kernel.decode_native_sclk(ticks) {
            Ok(value) => format!("S:{value}"),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{row}");
    }
}

#[test]
fn native_sclk_extreme_field_buffer_behavior() {
    assert_eq!(
        include_str!("reference/sclk-partition/extreme-field.tsv")
            .lines()
            .count(),
        17
    );
    let kernel = SclkKernel::parse(
        include_str!("reference/sclk-partition/extreme-field.tsc"),
        -902,
    )
    .unwrap();
    for row in include_str!("reference/sclk-partition/extreme-field.tsv")
        .lines()
        .skip(1)
    {
        let (input, expected) = row.split_once('\t').unwrap();
        let ticks = f64::from_bits(u64::from_str_radix(input, 16).unwrap());
        let actual = match kernel.decode_native_sclk(ticks) {
            Ok(value) => format!("S:{value}"),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{row}");
    }
}

#[test]
fn native_sclk_rejects_eleven_fields() {
    assert_eq!(
        include_str!("reference/sclk-partition/eleven-fields.tsv")
            .lines()
            .count(),
        3
    );
    let kernel = SclkKernel::parse(
        include_str!("reference/sclk-partition/eleven-fields.tsc"),
        -902,
    )
    .unwrap();
    for row in include_str!("reference/sclk-partition/eleven-fields.tsv")
        .lines()
        .skip(1)
    {
        let (op, expected) = row.split_once('\t').unwrap();
        let actual = match op {
            "D" => kernel
                .decode_native_sclk(0.0)
                .map(|value| format!("S:{value}")),
            "E" => kernel
                .encode_native_sclk("1/0")
                .map(|value| format!("T:{value}")),
            _ => panic!("unknown operation"),
        };
        assert_eq!(
            actual.unwrap_err().spice_short(),
            expected.strip_prefix("E:").unwrap(),
            "{row}"
        );
    }
}

#[test]
fn native_eleven_field_validation_precedes_partition_errors() {
    assert_eq!(
        include_str!("reference/sclk-partition/eleven-precedence.tsv")
            .lines()
            .count(),
        25
    );
    let source = include_str!("reference/sclk-partition/eleven-fields.tsc");
    let starts = "SCLK_PARTITION_START_902=(10.0 0.0)";
    let ends = "SCLK_PARTITION_END_902=(99.0 99.0)";
    let mut current_mode = "";
    let mut kernel = None;
    for row in include_str!("reference/sclk-partition/eleven-precedence.tsv")
        .lines()
        .skip(1)
    {
        let columns: Vec<_> = row.split('\t').collect();
        if columns[0] != current_mode {
            current_mode = columns[0];
            let text = match current_mode {
                "normal" => source.to_owned(),
                "missing" => source.replace(starts, "").replace(ends, ""),
                "capacity" => source
                    .replace(
                        starts,
                        &format!("SCLK_PARTITION_START_902=({})", "0 ".repeat(10000)),
                    )
                    .replace(
                        ends,
                        &format!("SCLK_PARTITION_END_902=({})", "1 ".repeat(10000)),
                    ),
                _ => panic!("unknown mode"),
            };
            kernel = Some(SclkKernel::parse(&text, -902).unwrap());
        }
        let kernel = kernel.as_ref().unwrap();
        let actual = match columns[1] {
            "D" => kernel.decode_native_sclk(0.0).map(|_| ()),
            "E" => kernel.encode_native_sclk(columns[2]).map(|_| ()),
            _ => panic!("unknown operation"),
        };
        assert_eq!(
            actual.unwrap_err().spice_short(),
            columns[3].strip_prefix("E:").unwrap(),
            "{row}"
        );
    }
}

#[test]
fn native_sclk_validates_delimiter_metadata() {
    assert_eq!(
        include_str!("reference/sclk-partition/delimiter-validation.tsv")
            .lines()
            .count(),
        19
    );
    let source = include_str!("reference/sclk-partition/large-field.tsc");
    let declaration = "SCLK01_OUTPUT_DELIM_902=(1)";
    for row in include_str!("reference/sclk-partition/delimiter-validation.tsv")
        .lines()
        .skip(1)
    {
        let columns: Vec<_> = row.split('\t').collect();
        let replacement = if columns[0] == "missing" {
            String::new()
        } else {
            format!("SCLK01_OUTPUT_DELIM_902=({})", columns[0])
        };
        let kernel = SclkKernel::parse(&source.replace(declaration, &replacement), -902).unwrap();
        let actual = match columns[1] {
            "D" => kernel
                .decode_native_sclk(0.0)
                .map(|value| format!("S:{value}")),
            "E" => kernel
                .encode_native_sclk("1/0")
                .map(|value| format!("T:{value:.1}")),
            _ => panic!("unknown operation"),
        };
        let actual = match actual {
            Ok(value) => value,
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, columns[2], "{row}");
    }
}

#[test]
fn native_sclk_metadata_validation_precedence() {
    let source = include_str!("reference/sclk-partition/large-field.tsc");
    let eleven = include_str!("reference/sclk-partition/eleven-fields.tsc");
    let delimiter = "SCLK01_OUTPUT_DELIM_902=(1)";
    let starts = "SCLK_PARTITION_START_902=(0.0)";
    let ends = "SCLK_PARTITION_END_902=(10000000000000000.0)";
    assert_eq!(
        include_str!("reference/sclk-partition/metadata-precedence.tsv")
            .lines()
            .count(),
        21
    );
    let mut current_mode = "";
    let mut kernel = None;
    for row in include_str!("reference/sclk-partition/metadata-precedence.tsv")
        .lines()
        .skip(1)
    {
        let columns: Vec<_> = row.split('\t').collect();
        if columns[0] != current_mode {
            current_mode = columns[0];
            let text = match current_mode {
                "fields_and_delim_missing" => eleven.replace(delimiter, ""),
                "delim_invalid_partitions_missing" => source
                    .replace(delimiter, "SCLK01_OUTPUT_DELIM_902=(6)")
                    .replace(starts, "")
                    .replace(ends, ""),
                "delim_missing_partitions_missing" => source
                    .replace(delimiter, "")
                    .replace(starts, "")
                    .replace(ends, ""),
                "delim_invalid_capacity" => source
                    .replace(delimiter, "SCLK01_OUTPUT_DELIM_902=(6)")
                    .replace(
                        starts,
                        &format!("SCLK_PARTITION_START_902=({})", "0 ".repeat(10000)),
                    )
                    .replace(
                        ends,
                        &format!("SCLK_PARTITION_END_902=({})", "1 ".repeat(10000)),
                    ),
                _ => panic!("unknown mode"),
            };
            kernel = Some(SclkKernel::parse(&text, -902).unwrap());
        }
        let kernel = kernel.as_ref().unwrap();
        let actual = match columns[1] {
            "D" => kernel.decode_native_sclk(0.0).map(|_| ()),
            "E" => kernel.encode_native_sclk(columns[2]).map(|_| ()),
            _ => panic!("unknown operation"),
        };
        assert_eq!(
            actual.unwrap_err().spice_short(),
            columns[3].strip_prefix("E:").unwrap(),
            "{row}"
        );
    }
}

#[test]
fn native_sclk_field_width_near_powers_of_ten() {
    let source = include_str!("reference/sclk-partition/large-field.tsc");
    let rows = include_str!("reference/sclk-partition/width-boundaries.tsv");
    assert_eq!(rows.lines().count(), 81);
    for row in rows.lines().skip(1) {
        let (modulus, expected) = row.split_once('\t').unwrap();
        let text = source.replace(
            "SCLK01_MODULI_902=(10000000000000000)",
            &format!("SCLK01_MODULI_902=({modulus})"),
        );
        let kernel = SclkKernel::parse(&text, -902).unwrap();
        let actual = match kernel.decode_native_sclk(0.0) {
            Ok(value) => format!("S:{value}"),
            Err(error) => format!("E:{}", error.spice_short()),
        };
        assert_eq!(actual, expected, "{row}");
    }
}
