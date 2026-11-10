//! Reads UTC calendar strings (one per line) from stdin and prints
//! `utc,et,tdb_iau` using this crate.
//!
//! `et` is the NAIF `ET` model used for SPICE interoperability; `tdb_iau`
//! is the IAU Fairhead & Bretagnon series. Their difference is the
//! approximation error of the NAIF model. Used by
//! `tools/wgc2_compare.py`.

use std::io::{self, BufRead};
use std::process;

use stkcraft_time::calendar::Calendar;
use stkcraft_time::convert::{tai_to_et, utc_to_tai};
use stkcraft_time::dtdb::tai_to_tdb_iau;
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

        match utc_to_tai(&dt, Calendar::Gregorian, &lsk) {
            Ok(tai) => {
                let et = tai_to_et(tai, &lsk);
                let tdb = tai_to_tdb_iau(tai, &lsk);
                println!("{value},{et:.12},{tdb:.12},{:.6}", (et - tdb) * 1e6);
            }
            Err(error) => {
                eprintln!("conversion error for {value}: {error}");
                process::exit(1);
            }
        }
    }
}
