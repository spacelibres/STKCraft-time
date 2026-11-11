//! Parser for the subset of NAIF text kernels needed by the time model.
//!
//! Only the `DELTET/*` variables of a `KPL/LSK` file are interpreted.
//! Unknown variables are ignored so that newer LSKs remain readable.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::calendar;
use crate::error::TimeError;
use crate::leap::{LeapEntry, LeapSecondTable};

/// Parses a `KPL/LSK` leap-second kernel.
///
/// Requires `DELTET/DELTA_T_A`, `K`, `EB`, `M` and `DELTA_AT` explicitly.
/// Missing constants are rejected rather than replaced by nominal values.
pub fn parse_lsk(text: &str) -> Result<LeapSecondTable, TimeError> {
    let data = extract_data(text)?;
    let tokens = tokenize(&data);

    let mut delta_t_a = None;
    let mut k = None;
    let mut eb = None;
    let mut m = None;
    let mut entries: Vec<LeapEntry> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "DELTET/DELTA_T_A" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                delta_t_a = Some(parse_f64(token(&tokens, i)?)?);
                i += 1;
            }
            "DELTET/K" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                k = Some(parse_f64(token(&tokens, i)?)?);
                i += 1;
            }
            "DELTET/EB" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                eb = Some(parse_f64(token(&tokens, i)?)?);
                i += 1;
            }
            "DELTET/M" => {
                i += 1;
                expect(&tokens, i, "=")?;
                i += 1;
                expect(&tokens, i, "(")?;
                i += 1;
                let m0 = parse_f64(token(&tokens, i)?)?;
                i += 1;
                let m1 = parse_f64(token(&tokens, i)?)?;
                m = Some((m0, m1));
                i += 1;
                expect(&tokens, i, ")")?;
                i += 1;
            }
            "DELTET/DELTA_AT" => {
                i += 1;
                // NAIF text kernels distinguish assignment from
                // accumulation: `=` replaces the previous value and `+=`
                // appends to it. A later block may restate the whole table
                // or extend an earlier one.
                let replace = match (tokens.get(i), tokens.get(i + 1)) {
                    (Some(op), Some(eq)) if op == "+" && eq == "=" => {
                        i += 2;
                        false
                    }
                    (Some(op), _) if op == "=" => {
                        i += 1;
                        true
                    }
                    _ => return Err(TimeError::LskParse),
                };
                expect(&tokens, i, "(")?;
                i += 1;
                let mut parsed = Vec::new();
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
                    parsed.push(LeapEntry { jdn, delta_at });
                }
                expect(&tokens, i, ")")?;
                i += 1;

                if replace {
                    entries = parsed;
                } else {
                    entries.extend(parsed);
                }
            }
            _ => i += 1,
        }
    }

    let delta_t_a = delta_t_a.ok_or(TimeError::LskMissing("DELTET/DELTA_T_A"))?;
    let k = k.ok_or(TimeError::LskMissing("DELTET/K"))?;
    let eb = eb.ok_or(TimeError::LskMissing("DELTET/EB"))?;
    let (m0, m1) = m.ok_or(TimeError::LskMissing("DELTET/M"))?;
    Ok(LeapSecondTable::new(entries)?.with_constants(delta_t_a, k, eb, m0, m1))
}

/// Parses the bundled `naif0012.tls`.
pub fn builtin() -> Result<LeapSecondTable, TimeError> {
    parse_lsk(include_str!("naif0012.tls"))
}

fn extract_data(text: &str) -> Result<String, TimeError> {
    const BEGIN: &str = "\\begindata";
    const END: &str = "\\begintext";

    let mut data = String::new();
    let mut rest = text;
    let mut found = false;

    while let Some(start) = rest.find(BEGIN) {
        found = true;
        let after = &rest[start + BEGIN.len()..];
        let end = after.find(END).unwrap_or(after.len());
        data.push_str(&after[..end]);
        data.push('\n');
        if end == after.len() {
            break;
        }
        rest = &after[end + END.len()..];
    }

    if found {
        Ok(data)
    } else {
        Err(TimeError::LskParse)
    }
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
    crate::naif::parse_f64(tok).ok_or(TimeError::LskParse)
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
    calendar::calendar_to_jdn(year, month, day, calendar::Calendar::Gregorian)
        .map_err(|_| TimeError::LskParse)
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
