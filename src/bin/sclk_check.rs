//! Reads encoded SCLK ticks (one per line) from stdin and prints
//! `ticks,et` using this crate and the bundled LSK.
//!
//! Usage:
//!
//! ```text
//! sclk-check <path-to-sclk-kernel> <spacecraft-id>
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

    let lsk = LeapSecondTable::builtin().expect("bundled naif0012.tls");
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

        match kernel.ticks_to_et(ticks, &lsk) {
            Ok(et) => {
                writeln!(out, "{ticks},{et:.12}").ok();
            }
            Err(error) => {
                eprintln!("conversion error for {value}: {error}");
                process::exit(1);
            }
        }
    }
}
