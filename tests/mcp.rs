//! MCP protocol tests. Run with `cargo test --features mcp`.

#![cfg(feature = "mcp")]

use serde_json::{json, Value};

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
            "name": "utc_to_tai",
            "arguments": {
                "year": 2016,
                "month": 12,
                "day": 30,
                "hour": 23,
                "minute": 59,
                "second": 60.0
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
fn notifications_do_not_receive_responses() {
    assert!(stkcraft_time::mcp::handle_message(
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
    )
    .is_none());
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
