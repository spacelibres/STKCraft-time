//! Minimal Model Context Protocol (MCP) server over stdio.
//!
//! Enabled by the `mcp` feature. The server exposes the crate's time-scale
//! conversions as MCP tools so an agent can call them without writing Rust
//! glue code. See `docs/MCP.md` for the tool list and client setup.
//!
//! The transport is newline-delimited JSON-RPC 2.0 on stdin/stdout. Logs
//! belong on stderr; stdout carries protocol messages only.

use std::fs;
use std::io::{self, BufRead, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde_json::{json, Value};

use crate::calendar::{jdn_to_calendar, Calendar};
use crate::convert::{
    et_to_utc, gps_to_tai, tai_to_gps, tai_to_tt, tai_to_utc, ut1_to_utc, utc_to_et, utc_to_tai,
    utc_to_ut1,
};
use crate::eop::{EopSeries, EopTable};
use crate::epoch::{Epoch, UtcDateTime};
use crate::format::{self, FormatOptions};
use crate::leap::LeapSecondTable;
use crate::sclk::{SclkKernel, SclkStringContext};

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const MAX_MESSAGE_BYTES: usize = 1_048_576;
const MAX_PRECISION: usize = 9;

/// Runs the MCP server on stdin/stdout until EOF.
pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut sclk_context = SclkStringContext::default();

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(response) = handle_message_with_sclk_context(line, &mut sclk_context) {
            writeln!(out, "{response}")?;
            out.flush()?;
        }
    }

    Ok(())
}

/// Handles one JSON-RPC message. Returns `None` for notifications.
pub fn handle_message(message: &str) -> Option<String> {
    handle_message_with_sclk_context(message, &mut SclkStringContext::default())
}

/// Handle one JSON-RPC message with a persistent NAIF SCLK parser history.
pub fn handle_message_with_sclk_context(
    message: &str,
    sclk_context: &mut SclkStringContext,
) -> Option<String> {
    if message.len() > MAX_MESSAGE_BYTES {
        return Some(error_response(Value::Null, -32600, "message too large").to_string());
    }

    let request: Value = match serde_json::from_str(message) {
        Ok(value) => value,
        Err(error) => {
            return Some(
                error_response(Value::Null, -32700, &format!("parse error: {error}")).to_string(),
            )
        }
    };

    if !request.is_object() {
        return Some(
            error_response(Value::Null, -32600, "invalid request: expected an object").to_string(),
        );
    }

    let id = request.get("id").cloned();
    let method = match request.get("method").and_then(Value::as_str) {
        Some(method) => method,
        None => {
            if let Some(id) = id {
                return Some(
                    error_response(id, -32600, "invalid request: missing method").to_string(),
                );
            }
            return None;
        }
    };

    // JSON-RPC notifications carry no id and must not receive a response.
    let id = id?;

    match request.get("jsonrpc").and_then(Value::as_str) {
        Some("2.0") => {}
        Some(_) => {
            return Some(
                error_response(id, -32600, "invalid request: jsonrpc must be \"2.0\"").to_string(),
            )
        }
        None => {
            return Some(error_response(id, -32600, "invalid request: missing jsonrpc").to_string())
        }
    }

    let params = request.get("params").cloned().unwrap_or(Value::Null);

    match method {
        "initialize" => Some(
            success_response(
                id,
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "stkcraft-time",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "instructions": "Convert between UTC, TAI, TT, ET/TDB, GPS, UT1 and spacecraft clock ticks. The tools read LSK, SCLK and EOP files from local paths; omit lsk_path to use the bundled naif0012.tls."
                }),
            )
            .to_string(),
        ),
        "ping" => Some(success_response(id, json!({})).to_string()),
        "tools/list" => {
            Some(success_response(id, json!({ "tools": tool_definitions() })).to_string())
        }
        "tools/call" => {
            let name = match params.get("name").and_then(Value::as_str) {
                Some(name) => name,
                None => {
                    return Some(
                        error_response(id, -32602, "tools/call requires a `name`").to_string(),
                    )
                }
            };
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let outcome = catch_unwind(AssertUnwindSafe(|| call_tool(name, &arguments, sclk_context)));
            let result = match outcome {
                Ok(Ok(value)) => {
                    let text = match serde_json::to_string_pretty(&value) {
                        Ok(text) => text,
                        Err(_) => value.to_string(),
                    };
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": text
                            }
                        ]
                    })
                }
                Ok(Err(error)) => json!({
                    "content": [
                        {
                            "type": "text",
                            "text": error
                        }
                    ],
                    "isError": true
                }),
                Err(_) => json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "tool panicked"
                        }
                    ],
                    "isError": true
                }),
            };

            Some(success_response(id, result).to_string())
        }
        other => {
            Some(error_response(id, -32601, &format!("method not found: {other}")).to_string())
        }
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "lsk_info",
            "description": "Report leap-second kernel coverage, entries, source and policy. Uses the bundled naif0012.tls unless lsk_path is supplied.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" }
                }
            }
        },
        {
            "name": "delta_at",
            "description": "Return TAI-UTC in whole seconds for a UTC date. Supply jdn, or year/month/day.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "jdn": { "type": "integer", "description": "Julian Day Number; overrides year/month/day" },
                    "year": { "type": "integer" },
                    "month": { "type": "integer", "minimum": 1, "maximum": 12 },
                    "day": { "type": "integer", "minimum": 1, "maximum": 31 },
                    "calendar": { "type": "string", "enum": ["gregorian", "julian", "mixed"] },
                    "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" }
                }
            }
        },
        {
            "name": "utc_to_tai",
            "description": "Convert a civil UTC date and time to TAI. Rejects 23:59:60 without a positive leap second and 23:59:59 on a shortened day.",
            "inputSchema": utc_schema()
        },
        {
            "name": "tai_to_utc",
            "description": "Convert TAI seconds past 2000-01-01T00:00:00 TAI to civil UTC.",
            "inputSchema": epoch_schema()
        },
        {
            "name": "utc_to_et",
            "description": "Convert civil UTC to NAIF ET (TDB seconds past J2000).",
            "inputSchema": utc_schema()
        },
        {
            "name": "et_to_utc",
            "description": "Convert NAIF ET (TDB seconds past J2000) to civil UTC.",
            "inputSchema": et_schema()
        },
        {
            "name": "tai_to_tt",
            "description": "Convert TAI seconds past 2000-01-01T00:00:00 TAI to TT seconds past J2000 TT.",
            "inputSchema": epoch_schema()
        },
        {
            "name": "tai_to_gps",
            "description": "Convert a TAI epoch to GPS seconds past the GPS epoch (1980-01-06T00:00:00 UTC).",
            "inputSchema": tai_seconds_schema()
        },
        {
            "name": "gps_to_tai",
            "description": "Convert GPS seconds past the GPS epoch to a TAI epoch.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "gps_seconds": { "type": "number", "description": "GPS seconds past 1980-01-06T00:00:00 UTC" }
                },
                "required": ["gps_seconds"]
            }
        },
        {
            "name": "utc_to_ut1",
            "description": "Convert civil UTC to UT1 seconds past 2000-01-01T00:00:00 UT1 using an IERS EOP file.",
            "inputSchema": utc_eop_schema()
        },
        {
            "name": "ut1_to_utc",
            "description": "Convert UT1 seconds past 2000-01-01T00:00:00 UT1 to civil UTC using an IERS EOP file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ut1_seconds": { "type": "number" },
                    "calendar": { "type": "string", "enum": ["gregorian", "julian", "mixed"] },
                    "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" },
                    "eop_path": { "type": "string", "description": "Path to finals2000A.all" },
                    "eop_series": { "type": "string", "enum": ["a", "b", "final"] },
                    "observed_only": { "type": "boolean" }
                },
                "required": ["ut1_seconds", "eop_path"]
            }
        },
        {
            "name": "sclk_to_et",
            "description": "Convert spacecraft clock ticks or a clock string to ET and TAI using a type-1 SCLK kernel.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sclk_path": { "type": "string", "description": "Path to a KPL/SCLK kernel" },
                    "spacecraft_id": { "type": "integer", "description": "NAIF clock id, either sign" },
                    "ticks": { "type": "number", "description": "Encoded tick count" },
                    "clock_string": { "type": "string", "description": "Formatted clock string, e.g. `123:45`" },
                    "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" }
                },
                "required": ["sclk_path", "spacecraft_id"]
            }
        },
        {
            "name": "et_to_sclk",
            "description": "Convert ET to spacecraft clock ticks, fields and a formatted clock string.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sclk_path": { "type": "string", "description": "Path to a KPL/SCLK kernel" },
                    "spacecraft_id": { "type": "integer", "description": "NAIF clock id, either sign" },
                    "et": { "type": "number", "description": "TDB seconds past J2000" },
                    "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" }
                },
                "required": ["sclk_path", "spacecraft_id", "et"]
            }
        },
        {
            "name": "format_utc",
            "description": "Format a civil UTC date/time as ISO 8601, RFC 3339, RFC 2822, SPICE calendar or strftime. Rounding uses the leap-second table, so 23:59:59.9999 on a leap-second day becomes 23:59:60.000 instead of the next day.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "year": { "type": "integer" },
                    "month": { "type": "integer", "minimum": 1, "maximum": 12 },
                    "day": { "type": "integer", "minimum": 1, "maximum": 31 },
                    "hour": { "type": "integer", "minimum": 0, "maximum": 23 },
                    "minute": { "type": "integer", "minimum": 0, "maximum": 59 },
                    "second": { "type": "number", "minimum": 0, "exclusiveMaximum": 61 },
                    "calendar": { "type": "string", "enum": ["gregorian", "julian", "mixed"], "description": "Calendar used for day carries; default gregorian" },
                    "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel; omit for the bundled naif0012.tls" },
                    "kind": { "type": "string", "enum": ["iso8601", "rfc3339", "rfc2822", "spice", "strftime"] },
                    "precision": { "type": "integer", "minimum": 0, "maximum": 9, "description": "Fractional digits for ISO/RFC 3339/SPICE" },
                    "pattern": { "type": "string", "description": "strftime pattern when kind=strftime" }
                },
                "required": ["year", "month", "day", "hour", "minute", "second"]
            }
        },
        {
            "name": "etcal",
            "description": "Format ET as a SPICE calendar TDB string without a leap-second kernel.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "et": { "type": "number" },
                    "precision": { "type": "integer", "minimum": 0, "maximum": 9 }
                },
                "required": ["et"]
            }
        }
    ])
}

fn utc_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "year": { "type": "integer", "description": "Astronomical year" },
            "month": { "type": "integer", "minimum": 1, "maximum": 12 },
            "day": { "type": "integer", "minimum": 1, "maximum": 31 },
            "hour": { "type": "integer", "minimum": 0, "maximum": 23 },
            "minute": { "type": "integer", "minimum": 0, "maximum": 59 },
            "second": {
                "type": "number",
                "minimum": 0,
                "exclusiveMaximum": 61,
                "description": "Seconds including fraction; 60.x is legal on a positive leap-second day"
            },
            "calendar": { "type": "string", "enum": ["gregorian", "julian", "mixed"] },
            "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel; omit for the bundled naif0012.tls" }
        },
        "required": ["year", "month", "day", "hour", "minute", "second"]
    })
}

fn utc_eop_schema() -> Value {
    let mut schema = utc_schema();
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        properties.insert(
            "eop_path".to_string(),
            json!({ "type": "string", "description": "Path to finals2000A.all" }),
        );
        properties.insert(
            "eop_series".to_string(),
            json!({ "type": "string", "enum": ["a", "b", "final"] }),
        );
        properties.insert("observed_only".to_string(), json!({ "type": "boolean" }));
    }
    if let Some(required) = schema.get_mut("required").and_then(Value::as_array_mut) {
        required.push(json!("eop_path"));
    }
    schema
}

fn epoch_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "tai_seconds": {
                "type": "number",
                "description": "TAI seconds past 2000-01-01T00:00:00 TAI"
            },
            "sec": {
                "type": "integer",
                "description": "Whole TAI seconds; use with nanos instead of tai_seconds"
            },
            "nanos": {
                "type": "integer",
                "minimum": 0,
                "maximum": 999999999,
                "description": "Nanoseconds for sec"
            },
            "calendar": { "type": "string", "enum": ["gregorian", "julian", "mixed"] },
            "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" }
        }
    })
}

fn tai_seconds_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "tai_seconds": {
                "type": "number",
                "description": "TAI seconds past 2000-01-01T00:00:00 TAI"
            },
            "sec": {
                "type": "integer",
                "description": "Whole TAI seconds; use with nanos instead of tai_seconds"
            },
            "nanos": {
                "type": "integer",
                "minimum": 0,
                "maximum": 999999999,
                "description": "Nanoseconds for sec"
            }
        }
    })
}

fn et_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "et": {
                "type": "number",
                "description": "NAIF ET: TDB seconds past J2000"
            },
            "calendar": { "type": "string", "enum": ["gregorian", "julian", "mixed"] },
            "lsk_path": { "type": "string", "description": "Path to a KPL/LSK kernel" }
        },
        "required": ["et"]
    })
}

fn call_tool(
    name: &str,
    args: &Value,
    sclk_context: &mut SclkStringContext,
) -> Result<Value, String> {
    match name {
        "lsk_info" => {
            let lsk = load_lsk(args)?;
            let (first_jdn, last_jdn) = lsk.coverage();
            let entries: Vec<Value> = lsk
                .entries()
                .iter()
                .map(|entry| json!({ "jdn": entry.jdn, "delta_at": entry.delta_at }))
                .collect();
            Ok(json!({
                "coverage": {
                    "first_jdn": first_jdn,
                    "last_jdn": last_jdn,
                    "first_date": date_json(first_jdn),
                    "last_date": date_json(last_jdn)
                },
                "entries": entries,
                "source": format!("{:?}", lsk.source()),
                "policy": format!("{:?}", lsk.policy())
            }))
        }
        "delta_at" => {
            let lsk = load_lsk(args)?;
            let jdn = date_jdn(args)?;
            Ok(json!({
                "jdn": jdn,
                "date": date_json(jdn),
                "delta_at": lsk.delta_at_for_jdn(jdn)
            }))
        }
        "utc_to_tai" => {
            let lsk = load_lsk(args)?;
            let utc = utc_arg(args)?;
            let calendar = calendar_arg(args)?;
            let tai = utc_to_tai(&utc, calendar, &lsk).map_err(|error| error.to_string())?;
            Ok(json!({
                "utc": utc_json(&utc, &lsk, calendar),
                "tai": epoch_json(tai)
            }))
        }
        "tai_to_utc" => {
            let lsk = load_lsk(args)?;
            let tai = epoch_arg(args)?;
            let calendar = calendar_arg(args)?;
            let utc = tai_to_utc(tai, calendar, &lsk).map_err(|error| error.to_string())?;
            Ok(json!({
                "tai": epoch_json(tai),
                "utc": utc_json(&utc, &lsk, calendar)
            }))
        }
        "utc_to_et" => {
            let lsk = load_lsk(args)?;
            let utc = utc_arg(args)?;
            let calendar = calendar_arg(args)?;
            let et = utc_to_et(&utc, calendar, &lsk).map_err(|error| error.to_string())?;
            Ok(json!({
                "utc": utc_json(&utc, &lsk, calendar),
                "et": et
            }))
        }
        "et_to_utc" => {
            let lsk = load_lsk(args)?;
            let et = as_f64(args, "et")?;
            let calendar = calendar_arg(args)?;
            let utc = et_to_utc(et, calendar, &lsk).map_err(|error| error.to_string())?;
            Ok(json!({
                "et": et,
                "utc": utc_json(&utc, &lsk, calendar)
            }))
        }
        "tai_to_tt" => {
            let lsk = load_lsk(args)?;
            let tai = epoch_arg(args)?;
            Ok(json!({
                "tai": epoch_json(tai),
                "tt": tai_to_tt(tai, &lsk)
            }))
        }
        "tai_to_gps" => {
            let tai = epoch_arg(args)?;
            Ok(json!({
                "tai": epoch_json(tai),
                "gps_seconds": tai_to_gps(tai)
            }))
        }
        "gps_to_tai" => {
            let gps = as_f64(args, "gps_seconds")?;
            let tai = gps_to_tai(gps);
            Ok(json!({
                "gps_seconds": gps,
                "tai": epoch_json(tai)
            }))
        }
        "utc_to_ut1" => {
            let lsk = load_lsk(args)?;
            let eop = load_eop(args)?;
            let utc = utc_arg(args)?;
            let calendar = calendar_arg(args)?;
            let ut1 = utc_to_ut1(&utc, calendar, &lsk, &eop).map_err(|error| error.to_string())?;
            Ok(json!({
                "utc": utc_json(&utc, &lsk, calendar),
                "ut1_seconds": ut1
            }))
        }
        "ut1_to_utc" => {
            let lsk = load_lsk(args)?;
            let eop = load_eop(args)?;
            let ut1 = as_f64(args, "ut1_seconds")?;
            let calendar = calendar_arg(args)?;
            let utc = ut1_to_utc(ut1, calendar, &lsk, &eop).map_err(|error| error.to_string())?;
            Ok(json!({
                "ut1_seconds": ut1,
                "utc": utc_json(&utc, &lsk, calendar)
            }))
        }
        "sclk_to_et" => {
            let lsk = load_lsk(args)?;
            let sclk = load_sclk(args)?;
            let ticks = sclk_ticks(args, &sclk, sclk_context)?;
            let et = sclk
                .ticks_to_et(ticks, &lsk)
                .map_err(|error| error.to_string())?;
            let tai = sclk
                .ticks_to_tai(ticks, &lsk)
                .map_err(|error| error.to_string())?;
            let fields = fields_if_integral(&sclk, ticks);
            Ok(json!({
                "ticks": ticks,
                "fields": fields,
                "et": et,
                "tai": epoch_json(tai)
            }))
        }
        "et_to_sclk" => {
            let lsk = load_lsk(args)?;
            let sclk = load_sclk(args)?;
            let et = as_f64(args, "et")?;
            let starts = sclk.partitions().0;
            let partitioned = starts.len() > 1 || starts.first().is_some_and(|&v| v != 0.0);
            let continuous = sclk.et_to_ticks(et, &lsk);
            let discrete = if partitioned {
                Some(
                    sclk.et_to_discrete_ticks(et, &lsk)
                        .map_err(|error| error.to_string())?,
                )
            } else {
                None
            };
            let ticks = match continuous {
                Ok(value) => Some(value),
                Err(_) if discrete.is_some() => None,
                Err(error) => return Err(error.to_string()),
            };
            let fields = ticks.and_then(|value| fields_if_integral(&sclk, value));
            let clock_string = if let Some(discrete) = discrete {
                sclk.decode_native_sclk(discrete).ok()
            } else {
                match &fields {
                    Some(fields) => sclk.format_string(fields).ok(),
                    None => None,
                }
            };
            Ok(json!({
                "et": et,
                "ticks": ticks,
                "fields": fields,
                "clock_string": clock_string
            }))
        }
        "format_utc" => {
            let utc = utc_arg(args)?;
            let precision = precision_arg(args)?;
            let calendar = calendar_arg(args)?;
            let lsk = load_lsk(args)?;
            let options = FormatOptions::new(precision)
                .with_calendar(calendar)
                .with_leap_seconds(&lsk);
            let kind = opt_str(args, "kind")
                .unwrap_or("iso8601")
                .to_ascii_lowercase();
            let text = match kind.as_str() {
                "iso8601" => format::iso8601_with(&utc, &options),
                "rfc3339" => format::rfc3339_with(&utc, &options),
                "rfc2822" => format::rfc2822(&utc),
                "spice" | "spice_calendar" => format::spice_calendar_with(&utc, &options),
                "strftime" => {
                    let pattern = as_str(args, "pattern")?;
                    format::strftime_with(&utc, pattern, &options)
                        .map_err(|error| error.to_string())?
                }
                other => return Err(format!("unknown format kind `{other}`")),
            };
            Ok(json!({
                "utc": utc_json(&utc, &lsk, calendar),
                "text": text
            }))
        }
        "etcal" => {
            let et = as_f64(args, "et")?;
            let precision = precision_arg(args)?;
            Ok(json!({
                "et": et,
                "calendar": format::etcal(et, precision)
            }))
        }
        other => Err(format!("unknown tool `{other}`")),
    }
}

fn arg<'a>(args: &'a Value, name: &str) -> Result<&'a Value, String> {
    args.get(name)
        .ok_or_else(|| format!("missing argument `{name}`"))
}

fn as_str<'a>(args: &'a Value, name: &str) -> Result<&'a str, String> {
    arg(args, name)?
        .as_str()
        .ok_or_else(|| format!("`{name}` must be a string"))
}

fn opt_str<'a>(args: &'a Value, name: &str) -> Option<&'a str> {
    args.get(name).and_then(Value::as_str)
}

fn as_f64(args: &Value, name: &str) -> Result<f64, String> {
    arg(args, name)?
        .as_f64()
        .ok_or_else(|| format!("`{name}` must be a number"))
}

fn as_i64(args: &Value, name: &str) -> Result<i64, String> {
    arg(args, name)?
        .as_i64()
        .ok_or_else(|| format!("`{name}` must be an integer"))
}

fn as_i32(args: &Value, name: &str) -> Result<i32, String> {
    let value = as_i64(args, name)?;
    i32::try_from(value).map_err(|_| format!("`{name}` is out of range"))
}

fn as_u8(args: &Value, name: &str) -> Result<u8, String> {
    let value = as_i64(args, name)?;
    u8::try_from(value).map_err(|_| format!("`{name}` is out of range"))
}

fn precision_arg(args: &Value) -> Result<usize, String> {
    match args.get("precision") {
        None => Ok(3),
        Some(value) => {
            let precision = value
                .as_u64()
                .ok_or_else(|| "`precision` must be a non-negative integer".to_string())?;
            if precision > MAX_PRECISION as u64 {
                return Err(format!("`precision` must be between 0 and {MAX_PRECISION}"));
            }
            Ok(precision as usize)
        }
    }
}

fn epoch_arg(args: &Value) -> Result<Epoch, String> {
    if let Some(value) = args.get("tai_seconds") {
        let seconds = value
            .as_f64()
            .ok_or_else(|| "`tai_seconds` must be a number".to_string())?;
        return Epoch::checked_from_tai_seconds(seconds)
            .ok_or_else(|| "`tai_seconds` is non-finite or out of range".to_string());
    }

    let sec = as_i64(args, "sec")?;
    let nanos = match args.get("nanos") {
        Some(value) => value
            .as_i64()
            .ok_or_else(|| "`nanos` must be an integer".to_string())?,
        None => 0,
    };

    Epoch::checked_from_parts(sec, nanos)
        .ok_or_else(|| "`sec`/`nanos` are out of range".to_string())
}

fn fields_if_integral(sclk: &SclkKernel, ticks: f64) -> Option<Vec<i64>> {
    if !ticks.is_finite() {
        return None;
    }
    let rounded = ticks.round();
    if (ticks - rounded).abs() > 1e-6 {
        return None;
    }
    sclk.fields_from_ticks(rounded).ok()
}

fn calendar_arg(args: &Value) -> Result<Calendar, String> {
    match opt_str(args, "calendar")
        .unwrap_or("gregorian")
        .to_ascii_lowercase()
        .as_str()
    {
        "gregorian" => Ok(Calendar::Gregorian),
        "julian" => Ok(Calendar::Julian),
        "mixed" => Ok(Calendar::Mixed),
        other => Err(format!("unknown calendar `{other}`")),
    }
}

fn utc_arg(args: &Value) -> Result<UtcDateTime, String> {
    Ok(UtcDateTime::new(
        as_i32(args, "year")?,
        as_u8(args, "month")?,
        as_u8(args, "day")?,
        as_u8(args, "hour")?,
        as_u8(args, "minute")?,
        as_f64(args, "second")?,
    ))
}

fn date_jdn(args: &Value) -> Result<i64, String> {
    if let Some(jdn) = args.get("jdn").and_then(Value::as_i64) {
        return Ok(jdn);
    }

    crate::calendar::calendar_to_jdn(
        as_i32(args, "year")?,
        as_u8(args, "month")?,
        as_u8(args, "day")?,
        calendar_arg(args)?,
    )
    .map_err(|error| error.to_string())
}

fn date_json(jdn: i64) -> Value {
    let (year, month, day) = jdn_to_calendar(jdn, Calendar::Gregorian);
    json!({
        "year": year,
        "month": month,
        "day": day
    })
}

fn epoch_json(epoch: Epoch) -> Value {
    json!({
        "tai_seconds": epoch.tai_seconds(),
        "sec": epoch.sec(),
        "nanos": epoch.nanos()
    })
}

fn utc_json(dt: &UtcDateTime, lsk: &LeapSecondTable, calendar: Calendar) -> Value {
    let options = FormatOptions::new(9)
        .with_calendar(calendar)
        .with_leap_seconds(lsk);
    json!({
        "year": dt.year,
        "month": dt.month,
        "day": dt.day,
        "hour": dt.hour,
        "minute": dt.minute,
        "second": dt.second,
        "iso8601": format::iso8601_with(dt, &options)
    })
}

fn load_lsk(args: &Value) -> Result<LeapSecondTable, String> {
    match opt_str(args, "lsk_path") {
        Some(path) => {
            let text = fs::read_to_string(path)
                .map_err(|error| format!("cannot read LSK `{path}`: {error}"))?;
            LeapSecondTable::from_lsk(&text)
                .map_err(|error| format!("cannot parse LSK `{path}`: {error}"))
        }
        None => LeapSecondTable::builtin().map_err(|error| format!("bundled LSK: {error}")),
    }
}

fn load_eop(args: &Value) -> Result<EopTable, String> {
    let path = as_str(args, "eop_path")?;
    let text =
        fs::read_to_string(path).map_err(|error| format!("cannot read EOP `{path}`: {error}"))?;

    let series = match opt_str(args, "eop_series")
        .unwrap_or("a")
        .to_ascii_lowercase()
        .as_str()
    {
        "a" | "bulletin_a" => EopSeries::BulletinA,
        "b" | "bulletin_b" => EopSeries::BulletinB,
        "final" | "final_preferred" => EopSeries::FinalPreferred,
        other => return Err(format!("unknown eop_series `{other}`")),
    };

    let table = EopTable::from_finals2000a_series(&text, series)
        .map_err(|error| format!("cannot parse EOP `{path}`: {error}"))?;

    if args
        .get("observed_only")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        table
            .observed_only()
            .map_err(|error| format!("cannot filter EOP `{path}`: {error}"))
    } else {
        Ok(table)
    }
}

fn load_sclk(args: &Value) -> Result<SclkKernel, String> {
    let path = as_str(args, "sclk_path")?;
    let spacecraft_id = as_i32(args, "spacecraft_id")?;
    let text =
        fs::read_to_string(path).map_err(|error| format!("cannot read SCLK `{path}`: {error}"))?;
    SclkKernel::parse(&text, spacecraft_id)
        .map_err(|error| format!("cannot parse SCLK `{path}`: {error}"))
}

fn sclk_ticks(
    args: &Value,
    sclk: &SclkKernel,
    context: &mut SclkStringContext,
) -> Result<f64, String> {
    if let Some(ticks) = args.get("ticks").and_then(Value::as_f64) {
        return Ok(ticks);
    }

    if let Some(text) = opt_str(args, "clock_string") {
        let starts = sclk.partitions().0;
        if !starts.is_empty() {
            return sclk
                .encode_native_sclk_with_context(text, context)
                .map_err(|error| error.to_string());
        }
        let fields = sclk.parse_string(text).map_err(|error| error.to_string())?;
        return sclk
            .ticks_from_fields(&fields)
            .map_err(|error| error.to_string());
    }

    Err("provide either `ticks` or `clock_string`".to_string())
}
