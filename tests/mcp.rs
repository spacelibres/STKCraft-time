//! MCP protocol tests. Run with `cargo test --features mcp`.

#![cfg(feature = "mcp")]

use serde_json::{json, Value};

const SYNTH_SCLK: &str = r#"
KPL/SCLK

\begindata

SCLK_DATA_TYPE_999       = ( 1 )
SCLK01_TIME_SYSTEM_999   = ( 1 )
SCLK01_N_FIELDS_999      = ( 2 )
SCLK01_MODULI_999        = ( 1000 100 )
SCLK01_OFFSETS_999       = ( 0 0 )
SCLK01_OUTPUT_DELIM_999  = ( 2 )
SCLK01_COEFFICIENTS_999  = (
    0.0000000000000E+00     0.0000000000000E+00     1.0000000000000E+00
    1.0000000000000E+05     1.0000000000000E+03     1.0000000000000E+00
)

\begintext
"#;

fn request(value: Value) -> Value {
    let line = value.to_string();
    let response = stkcraft_time::mcp::handle_message(&line).expect("request must have a response");
    serde_json::from_str(&response).expect("response must be valid JSON")
}

#[test]
fn initialize_reports_tools_capability() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0" }
        }
    }));

    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert_eq!(response["result"]["serverInfo"]["name"], "stkcraft-time");
}

#[test]
fn tools_list_contains_the_documented_tools() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    }));

    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools must be an array");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();

    for expected in [
        "lsk_info",
        "utc_to_tai",
        "utc_to_et",
        "tai_to_utc",
        "tai_to_gps",
        "utc_to_ut1",
        "sclk_to_et",
        "format_utc",
    ] {
        assert!(names.contains(&expected), "missing tool {expected}");
    }
}

#[test]
fn utc_to_et_matches_the_known_leap_second() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "utc_to_et",
            "arguments": {
                "year": 2016,
                "month": 12,
                "day": 31,
                "hour": 23,
                "minute": 59,
                "second": 60.5
            }
        }
    }));

    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    let value: Value = serde_json::from_str(text).expect("tool result JSON");
    let et = value["et"].as_f64().expect("et number");

    assert!((et - 536_500_868.683_93).abs() < 1e-6, "ET = {et}");
}

#[test]
fn invalid_leap_second_is_reported_as_a_tool_error() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "utc_to_et",
            "arguments": {
                "year": 2016,
                "month": 12,
                "day": 30,
                "hour": 23,
                "minute": 59,
                "second": 60.5
            }
        }
    }));

    assert_eq!(response["result"]["isError"], true);
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    assert!(text.contains("leap-second"), "error text: {text}");
}

#[test]
fn format_utc_rounds_onto_a_leap_second() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 41,
        "method": "tools/call",
        "params": {
            "name": "format_utc",
            "arguments": {
                "year": 2016,
                "month": 12,
                "day": 31,
                "hour": 23,
                "minute": 59,
                "second": 59.9999,
                "precision": 3
            }
        }
    }));

    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    let value: Value = serde_json::from_str(text).expect("tool result JSON");

    assert_eq!(value["text"], "2016-12-31T23:59:60.000Z");
    assert_eq!(value["utc"]["iso8601"], "2016-12-31T23:59:59.999900000Z");
}

#[test]
fn notifications_do_not_receive_responses() {
    assert!(stkcraft_time::mcp::handle_message(
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
    )
    .is_none());
}

#[test]
fn invalid_request_without_method_returns_an_error() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 42
    }));

    assert_eq!(response["error"]["code"], -32600);
}

#[test]
fn initialize_uses_the_supported_protocol_version() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "initialize",
        "params": {
            "protocolVersion": "9999-01-01",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0" }
        }
    }));

    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
}

#[test]
fn precision_is_limited() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "format_utc",
            "arguments": {
                "year": 2026,
                "month": 1,
                "day": 1,
                "hour": 0,
                "minute": 0,
                "second": 0,
                "precision": 18446744073709551615u64
            }
        }
    }));

    assert_eq!(response["result"]["isError"], true);
}

#[test]
fn tai_to_utc_accepts_sec_and_nanos() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "tools/call",
        "params": {
            "name": "tai_to_utc",
            "arguments": { "sec": 0, "nanos": 0 }
        }
    }));

    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    let value: Value = serde_json::from_str(text).expect("tool result JSON");

    assert_eq!(value["tai"]["sec"], 0);
    assert_eq!(value["tai"]["nanos"], 0);
    assert!(value["utc"]["iso8601"].is_string());
}

#[test]
fn continuous_sclk_ticks_are_accepted() {
    let path = std::env::temp_dir().join(format!("stkcraft-mcp-test-{}.tsc", std::process::id()));
    std::fs::write(&path, SYNTH_SCLK).expect("write synthetic SCLK");

    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "sclk_to_et",
            "arguments": {
                "sclk_path": path.to_str().expect("temp path"),
                "spacecraft_id": -999,
                "ticks": 0.5
            }
        }
    }));

    let _ = std::fs::remove_file(&path);

    assert!(response["result"]["isError"].is_null());
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    let value: Value = serde_json::from_str(text).expect("tool result JSON");

    assert!(value["fields"].is_null());
    assert!(value["et"].is_number());
    assert!(value["tai"]["sec"].is_number());
}

#[test]
fn unknown_methods_return_json_rpc_errors() {
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "no/such/method"
    }));

    assert_eq!(response["error"]["code"], -32601);
}
