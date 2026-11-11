//! Reads numeric inputs and prints lossless `input,output` pairs.
//! The default maps SCLK ticks to ET; --inverse maps ET to ticks.
//!
//! Usage:
//!
//! ```text
//! sclk-check <path-to-sclk-kernel> <spacecraft-id> [--lsk path] [--inverse]
//! ```
//!
//! Used by `tools/validate_sclk.py`.

use std::io::{self, BufRead, Write};
use std::process;

use stkcraft_time::sclk::SclkKernel;
use stkcraft_time::LeapSecondTable;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("usage: sclk-check <path-to-sclk-kernel> <spacecraft-id>");
            process::exit(2);
        }
    };
    let spacecraft_id: i32 = match args.next().and_then(|value| value.parse().ok()) {
        Some(id) => id,
        None => {
            eprintln!("usage: sclk-check <path-to-sclk-kernel> <spacecraft-id>");
            process::exit(2);
        }
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            process::exit(1);
        }
    };

    let kernel = match SclkKernel::parse(&text, spacecraft_id) {
        Ok(kernel) => kernel,
        Err(error) => {
            eprintln!("cannot parse {path}: {error}");
            process::exit(1);
        }
    };

    let mut lsk = LeapSecondTable::builtin().expect("bundled naif0012.tls");
    let mut inverse = false;
    while let Some(option) = args.next() {
        match option.as_str() {
            "--inverse" => inverse = true,
            "--lsk" => {
                let selected = args
                    .next()
                    .and_then(|path| std::fs::read_to_string(path).ok())
                    .and_then(|text| LeapSecondTable::from_lsk(&text).ok());
                lsk = selected.unwrap_or_else(|| {
                    eprintln!("cannot load selected leap-second kernel");
                    process::exit(2);
                });
            }
            _ => {
                eprintln!("unknown option: {option}");
                process::exit(2);
            }
        }
    }
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in io::stdin().lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("stdin error: {error}");
                process::exit(1);
            }
        };

        let value = line.trim();
        if value.is_empty() {
            continue;
        }

        let ticks: f64 = match value.parse() {
            Ok(ticks) => ticks,
            Err(_) => {
                eprintln!("invalid tick value: {value}");
                process::exit(1);
            }
        };

        let result = if inverse {
            kernel.et_to_ticks(ticks, &lsk)
        } else {
            kernel.ticks_to_et(ticks, &lsk)
        };
        match result {
            Ok(et) => {
                writeln!(out, "{ticks},{et}").ok();
            }
            Err(error) => {
                eprintln!("conversion error for {value}: {error}");
                process::exit(1);
            }
        }
    }
}
