//! Reads UTC calendar strings (one per line) from stdin and prints
//! `utc,ut1_minus_utc` using this crate and an IERS EOP file.
//!
//! Usage:
//!
//! ```text
//! ut1-check <path-to-finals2000A.all>
//! ```
//!
//! Used by `tools/validate_ut1.py`.

use std::io::{self, BufRead, Write};
use std::process;

use stkcraft_time::calendar::Calendar;
use stkcraft_time::convert::{ut1_minus_utc, utc_to_tai};
use stkcraft_time::eop::{EopSeries, EopTable};
use stkcraft_time::{LeapSecondTable, UtcDateTime};

fn parse_utc(value: &str) -> Option<UtcDateTime> {
    let (date, time) = value.split_once('T')?;

    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse().ok()?;
    let month = date_parts.next()?.parse().ok()?;
    let day = date_parts.next()?.parse().ok()?;

    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse().ok()?;
    let minute = time_parts.next()?.parse().ok()?;
    let second = time_parts.next()?.parse().ok()?;

    Some(UtcDateTime::new(year, month, day, hour, minute, second))
}

fn main() {
    let path = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: ut1-check <path-to-finals2000A.all> [a|b|final]");
            process::exit(2);
        }
    };
    let series = match std::env::args().nth(2).as_deref() {
        None | Some("a") => EopSeries::BulletinA,
        Some("b") => EopSeries::BulletinB,
        Some("final") => EopSeries::FinalPreferred,
        Some(other) => {
            eprintln!("unknown series {other}; use a, b or final");
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

    let eop = match EopTable::from_finals2000a_series(&text, series) {
        Ok(eop) => eop,
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

        match parse_utc(value) {
            None => {
                writeln!(out, "{value},PARSE_ERR").ok();
            }
            Some(dt) => {
                let result = utc_to_tai(&dt, Calendar::Gregorian, &lsk)
                    .and_then(|tai| ut1_minus_utc(tai, &lsk, &eop));
                match result {
                    Ok(ut1) => {
                        writeln!(out, "{value},{ut1:.9}").ok();
                    }
                    Err(error) => {
                        writeln!(out, "{value},ERR:{error}").ok();
                    }
                }
            }
        }
    }
}
