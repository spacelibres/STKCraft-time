//! Parser for the subset of NAIF text kernels needed by the time model.
//!
//! Only the `DELTET/*` variables of a `KPL/LSK` file are interpreted.
//! Unknown variables are ignored so that newer LSKs remain readable.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::calendar;
use crate::error::TimeError;
use crate::leap::{self, LeapEntry, LeapSecondTable};

/// Parses a `KPL/LSK` leap-second kernel.
pub fn parse_lsk(text: &str) -> Result<LeapSecondTable, TimeError> {
    let data = extract_data(text)?;
    let tokens = tokenize(data);

    let mut delta_t_a = leap::DELTA_T_A;
    let mut k = leap::K;
    let mut eb = leap::EB;
    let mut m0 = leap::M0;
    let mut m1 = leap::M1;
    let mut entries: Vec<LeapEntry> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "DELTET/DELTA_T_A" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                delta_t_a = parse_f64(token(&tokens, i)?)?;
                i += 1;
            }
            "DELTET/K" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                k = parse_f64(token(&tokens, i)?)?;
                i += 1;
            }
            "DELTET/EB" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                eb = parse_f64(token(&tokens, i)?)?;
                i += 1;
            }
            "DELTET/M" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                expect(&tokens, i, "(")?;
                i += 1;
                m0 = parse_f64(token(&tokens, i)?)?;
                i += 1;
                m1 = parse_f64(token(&tokens, i)?)?;
                i += 1;
                expect(&tokens, i, ")")?;
                i += 1;
            }
            "DELTET/DELTA_AT" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                expect(&tokens, i, "(")?;
                i += 1;
                while i < tokens.len() && tokens[i] != ")" {
                    while i < tokens.len() && tokens[i] == "," {
                        i += 1;
                    }
                    if i >= tokens.len() || tokens[i] == ")" {
                        break;
                    }

                    let delta_at = parse_i32(token(&tokens, i)?)?;
                    i += 1;

                    while i < tokens.len() && tokens[i] == "," {
                        i += 1;
                    }

                    let jdn = parse_date_token(token(&tokens, i)?)?;
                    i += 1;
                    entries.push(LeapEntry { jdn, delta_at });
                }
                expect(&tokens, i, ")")?;
                i += 1;
            }
            _ => i += 1,
        }
    }

    Ok(LeapSecondTable::new(entries)?.with_constants(delta_t_a, k, eb, m0, m1))
}

/// Parses the bundled `naif0012.tls`.
pub fn builtin() -> Result<LeapSecondTable, TimeError> {
    parse_lsk(include_str!("naif0012.tls"))
}

fn extract_data(text: &str) -> Result<&str, TimeError> {
    const BEGIN: &str = "\\begindata";
    const END: &str = "\\begintext";

    let start = text.find(BEGIN).ok_or(TimeError::LskParse)? + BEGIN.len();
    let rest = &text[start..];
    let end = rest.find(END).unwrap_or(rest.len());
    Ok(&rest[..end])
}

fn tokenize(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();

    for ch in input.chars() {
        match ch {
            '=' | '(' | ')' | ',' => {
                if !cur.is_empty() {
                    out.push(core::mem::take(&mut cur));
                }
                out.push(ch.to_string());
            }
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(core::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }

    if !cur.is_empty() {
        out.push(cur);
    }

    out
}

fn token(tokens: &[String], i: usize) -> Result<&str, TimeError> {
    tokens.get(i).map(String::as_str).ok_or(TimeError::LskParse)
}

fn expect(tokens: &[String], i: usize, expected: &str) -> Result<(), TimeError> {
    if token(tokens, i)? == expected {
        Ok(())
    } else {
        Err(TimeError::LskParse)
    }
}

fn parse_f64(tok: &str) -> Result<f64, TimeError> {
    // NAIF text kernels use Fortran `D` exponents.
    let normalised = tok.replace('D', "E").replace('d', "e");
    normalised.parse::<f64>().map_err(|_| TimeError::LskParse)
}

fn parse_i32(tok: &str) -> Result<i32, TimeError> {
    tok.parse::<i32>().map_err(|_| TimeError::LskParse)
}

fn parse_date_token(tok: &str) -> Result<i64, TimeError> {
    let value = tok.strip_prefix('@').ok_or(TimeError::LskParse)?;
    let mut parts = value.split('-');

    let year = parts
        .next()
        .ok_or(TimeError::LskParse)?
        .parse::<i32>()
        .map_err(|_| TimeError::LskParse)?;
    let month_name = parts.next().ok_or(TimeError::LskParse)?;
    let day = parts
        .next()
        .ok_or(TimeError::LskParse)?
        .parse::<u8>()
        .map_err(|_| TimeError::LskParse)?;

    if parts.next().is_some() {
        return Err(TimeError::LskParse);
    }

    let month = month_from_name(month_name).ok_or(TimeError::LskParse)?;
    Ok(calendar::gregorian_to_jdn(year, month, day))
}

fn month_from_name(name: &str) -> Option<u8> {
    match name.to_ascii_uppercase().as_str() {
        "JAN" => Some(1),
        "FEB" => Some(2),
        "MAR" => Some(3),
        "APR" => Some(4),
        "MAY" => Some(5),
        "JUN" => Some(6),
        "JUL" => Some(7),
        "AUG" => Some(8),
        "SEP" => Some(9),
        "OCT" => Some(10),
        "NOV" => Some(11),
        "DEC" => Some(12),
        _ => None,
    }
}
