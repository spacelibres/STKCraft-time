//! Reads UTC calendar strings (one per line) from stdin and prints
//! `utc,et` using this crate.
//!
//! Used by `tools/wgc2_compare.py`.

use std::io::{self, BufRead};
use std::process;

use stkcraft_time::convert::utc_to_et;
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

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
    let lsk = LeapSecondTable::builtin().expect("bundled naif0012.tls");

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

        let dt = match parse_utc(value) {
            Some(dt) => dt,
            None => {
                eprintln!("invalid UTC string: {value}");
                process::exit(1);
            }
        };

        match utc_to_et(&dt, Calendar::Gregorian, &lsk) {
            Ok(et) => println!("{value},{et:.12}"),
            Err(error) => {
                eprintln!("conversion error for {value}: {error}");
                process::exit(1);
            }
        }
    }
}
