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
fn mcp_sclk_to_et_reports_unrepresentable_tai_without_panicking() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/reference/sclk-partition/overflow-rate.tsc"
    );
    for clock in ["1/001", "1/002"] {
        let response = request(json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"sclk_to_et","arguments":{"sclk_path":path,"spacecraft_id":-777,"clock_string":clock}}}));
        assert_eq!(response["result"]["isError"], true, "{response}");
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("overflow"),
            "{response}"
        );
    }
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
        "wgc_week_tow_to_et",
        "wgc_gps_seconds_to_et",
        "wgc_et_to_gps_seconds",
        "wgc_gps_seconds_to_week_tow",
        "wgc_et_to_calendar",
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

    // Only 23:59 may run past 59 s on a leap-second day.
    let response = request(json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "tools/call",
        "params": {
            "name": "format_utc",
            "arguments": {
                "year": 2016,
                "month": 12,
                "day": 31,
                "hour": 12,
                "minute": 34,
                "second": 59.9999,
                "precision": 3
            }
        }
    }));

    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    let value: Value = serde_json::from_str(text).expect("tool result JSON");
    assert_eq!(value["text"], "2016-12-31T12:35:00.000Z");
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
fn mro_partitioned_sclk_strings_match_native_et() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/kernels/MRO_SCLKSCET.00120.tsc"
    );
    let et = f64::from_bits(0xc1c2cfa30668755d);
    let response = request(json!({
        "jsonrpc": "2.0", "id": 41, "method": "tools/call",
        "params": { "name": "sclk_to_et", "arguments": {
            "sclk_path": path, "spacecraft_id": -74, "clock_string": "1/0000000000.000"
        }}
    }));
    assert!(response["result"]["isError"].is_null(), "{response}");
    let value: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(value["et"].as_f64().unwrap().to_bits(), et.to_bits());
    let response = request(json!({
        "jsonrpc": "2.0", "id": 42, "method": "tools/call",
        "params": { "name": "et_to_sclk", "arguments": {
            "sclk_path": path, "spacecraft_id": -74, "et": et
        }}
    }));
    assert!(response["result"]["isError"].is_null(), "{response}");
    let value: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(value["clock_string"], "1/0000000000.000");
}

#[test]
fn mcp_session_preserves_native_sclk_component_history() {
    use stkcraft_time::sclk::SclkStringContext;
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/kernels/MRO_SCLKSCET.00120.tsc"
    );
    let mut context = SclkStringContext::default();
    for (clock_string, error_expected) in [
        ("1/0.0", false),
        ("1/.1", false),
        ("1/UNKNOWN", true),
        ("1/.1", true),
        ("1/0.0", false),
        ("1/.1", false),
    ] {
        let call = json!({"jsonrpc":"2.0", "id":1, "method":"tools/call", "params":{
            "name":"sclk_to_et", "arguments":{"sclk_path":path,"spacecraft_id":-74,"clock_string":clock_string}
        }});
        let response =
            stkcraft_time::mcp::handle_message_with_sclk_context(&call.to_string(), &mut context)
                .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(
            response["result"]["isError"].as_bool().unwrap_or(false),
            error_expected,
            "{clock_string}: {response}"
        );
    }
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

#[test]
fn incomplete_lsk_is_a_tool_error() {
    let path = std::env::temp_dir().join(format!("stkcraft-missing-k-{}.tls", std::process::id()));
    let text = include_str!("../src/naif0012.tls")
        .lines()
        .filter(|line| !line.trim_start().starts_with("DELTET/K "))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, text).unwrap();
    let response = request(json!({
        "jsonrpc": "2.0", "id": 11, "method": "tools/call",
        "params": { "name": "utc_to_et", "arguments": {
            "lsk_path": path.to_str().unwrap(),
            "year": 2000, "month": 1, "day": 1,
            "hour": 12, "minute": 0, "second": 0.0
        }}
    }));
    std::fs::remove_file(path).unwrap();
    assert_eq!(response["result"]["isError"], true);
    assert!(response["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("DELTET/K"));
}

#[test]
fn partitioned_clock_mcp_preserves_discrete_boundary() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/kernels/synthetic_partition_reset.tsc"
    );
    let response = request(
        json!({"jsonrpc":"2.0", "id": 43, "method":"tools/call", "params":{
            "name":"et_to_sclk", "arguments":{"sclk_path":path,"spacecraft_id":-999,"et":-0.00008}
        }}),
    );
    assert!(response["result"]["isError"].is_null(), "{response}");
    let value: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(value["clock_string"], "1/001:00:001");
    assert!(value["ticks"].is_null());
}

#[test]
fn mcp_partitioned_et_to_sclk_reports_native_metadata_error() {
    let path = std::env::temp_dir().join(format!(
        "stkcraft-sclk-missing-delimiter-{}.tsc",
        std::process::id()
    ));
    let text = include_str!("kernels/synthetic_partition_reset.tsc")
        .lines()
        .filter(|line| !line.contains("SCLK01_OUTPUT_DELIM_999"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, text).unwrap();
    let response = request(
        json!({"jsonrpc":"2.0","id":99,"method":"tools/call","params":{
            "name":"et_to_sclk","arguments":{"sclk_path":path.to_str().unwrap(),"spacecraft_id":-999,"et":0.0}
        }}),
    );
    std::fs::remove_file(path).unwrap();
    assert_eq!(response["result"]["isError"], true, "{response}");
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("SPICE(KERNELVARNOTFOUND)"),
        "{response}"
    );
}

#[test]
fn mcp_partitioned_et_to_sclk_reports_native_range_error() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/kernels/synthetic_partition_reset.tsc"
    );
    let et = f64::from_bits(0xbf1a36e2eb1c432d);
    let response = request(
        json!({"jsonrpc":"2.0","id":100,"method":"tools/call","params":{
            "name":"et_to_sclk","arguments":{"sclk_path":path,"spacecraft_id":-999,"et":et}
        }}),
    );
    assert_eq!(response["result"]["isError"], true, "{response}");
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("SPICE(VALUEOUTOFRANGE)"),
        "{response}"
    );
}

#[test]
fn mcp_sclk_string_before_first_coefficient_reports_native_range_error() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/reference/sclk-partition/coefficient-range.tsc"
    );
    let response = request(
        json!({"jsonrpc":"2.0","id":101,"method":"tools/call","params":{
            "name":"sclk_to_et","arguments":{"sclk_path":path,"spacecraft_id":-999,"clock_string":"1/001:00:001"}
        }}),
    );
    assert_eq!(response["result"]["isError"], true, "{response}");
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("SPICE(VALUEOUTOFRANGE)"),
        "{response}"
    );
}

#[test]
fn mcp_single_zero_start_partition_matches_native_clock_strings() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/reference/sclk-partition/single-zero-start.tsc"
    );
    for (et, expected) in [
        (0.0, "1/000:00:001"),
        (0.0001, "1/000:00:002"),
        (1.0, "1/001:00:001"),
        (2.0, "1/002:00:001"),
    ] {
        let response = request(
            json!({"jsonrpc":"2.0","id":102,"method":"tools/call","params":{
                "name":"et_to_sclk","arguments":{"sclk_path":path,"spacecraft_id":-999,"et":et}
            }}),
        );
        assert!(response["result"]["isError"].is_null(), "{response}");
        let value: Value =
            serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap())
                .unwrap();
        assert_eq!(value["clock_string"], expected, "{response}");
    }
}

#[test]
fn mcp_wgc_week_tow_and_gps_seconds_match_nasa_cells() {
    for row in include_str!("reference/wgc-gps-input/week-seconds-seeded.tsv")
        .lines()
        .skip(1)
    {
        let (input, bits) = row.split_once('\t').unwrap();
        let first = request(
            json!({"jsonrpc":"2.0","id":110,"method":"tools/call","params":{
                "name":"wgc_week_tow_to_et","arguments":{"week_tow":input}
            }}),
        );
        assert!(first["result"]["isError"].is_null(), "{first}");
        let first_value: Value =
            serde_json::from_str(first["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        let et = first_value["et"].as_f64().unwrap();
        let second = request(
            json!({"jsonrpc":"2.0","id":111,"method":"tools/call","params":{
                "name":"wgc_et_to_gps_seconds","arguments":{"et":et}
            }}),
        );
        assert!(second["result"]["isError"].is_null(), "{second}");
        let second_value: Value =
            serde_json::from_str(second["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(
            second_value["gps_seconds"].as_f64().unwrap().to_bits(),
            u64::from_str_radix(bits, 16).unwrap(),
            "{row}"
        );
    }
}

#[test]
fn mcp_wgc_calendar_formats_match_nasa() {
    for (system, rows) in [
        (
            "UTC",
            include_str!("reference/wgc-gps-input/utc-ymd-epochs.tsv"),
        ),
        (
            "TDB",
            include_str!("reference/wgc-gps-input/tdb-ymd-epochs.tsv"),
        ),
        (
            "TT",
            include_str!("reference/wgc-gps-input/tt-ymd-epochs.tsv"),
        ),
        (
            "GPS",
            include_str!("reference/wgc-gps-input/gps-ymd-epochs.tsv"),
        ),
    ] {
        for row in rows.lines().skip(1) {
            let (et, expected) = row.split_once('\t').unwrap();
            let response = request(
                json!({"jsonrpc":"2.0","id":112,"method":"tools/call","params":{
                    "name":"wgc_et_to_calendar","arguments":{"et":et.parse::<f64>().unwrap(),"system":system,"format":"ymd"}
                }}),
            );
            assert!(response["result"]["isError"].is_null(), "{response}");
            let value: Value =
                serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap())
                    .unwrap();
            assert_eq!(value["calendar"], expected, "{row}");
        }
    }
}

#[test]
fn mcp_wgc_seconds_and_error_paths_are_explicit() {
    let call = |name: &str, arguments: Value| {
        request(
            json!({"jsonrpc":"2.0","id":113,"method":"tools/call","params":{"name":name,"arguments":arguments}}),
        )
    };
    let et = call("wgc_gps_seconds_to_et", json!({"gps_seconds":0.0}));
    assert!(et["result"]["isError"].is_null(), "{et}");
    let et: Value =
        serde_json::from_str(et["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    let display = call("wgc_et_to_gps_seconds", json!({"et":et["et"]}));
    let display: Value =
        serde_json::from_str(display["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(display["gps_seconds"], 0.0);
    let week = call("wgc_gps_seconds_to_week_tow", json!({"gps_seconds":0.0}));
    let week: Value =
        serde_json::from_str(week["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(week["week_tow"], "0, 0.000000");
    for (name, args) in [
        ("wgc_week_tow_to_et", json!({"week_tow":"1,2"})),
        (
            "wgc_et_to_calendar",
            json!({"et":0.0,"system":"UTC","format":"custom"}),
        ),
        (
            "wgc_et_to_calendar",
            json!({"et":0.0,"system":"unknown","format":"ymd"}),
        ),
    ] {
        assert_eq!(call(name, args)["result"]["isError"], true, "{name}");
    }
}

#[test]
fn mcp_native_clock_handles_finite_et_with_indeterminate_ticks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/reference/sclk-partition/nan-finite-et");
    for (name, expected) in [
        ("single", "E:SPICE(VALUEOUTOFRANGE)"),
        ("threefield", "S:2/000:00:000"),
        ("space", "S:2/00 00"),
    ] {
        let path = root.join(format!("{name}.tsc"));
        let response = request(
            json!({"jsonrpc":"2.0","id":115,"method":"tools/call","params":{
                "name":"et_to_sclk","arguments":{"sclk_path":path.to_str().unwrap(),"spacecraft_id":-777,"et":0.0}
            }}),
        );
        if let Some(error) = expected.strip_prefix("E:") {
            assert_eq!(response["result"]["isError"], true, "{response}");
            assert!(response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains(error));
        } else {
            assert!(response["result"]["isError"].is_null(), "{response}");
            let value: Value =
                serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap())
                    .unwrap();
            assert_eq!(value["clock_string"], expected.strip_prefix("S:").unwrap());
            assert!(value["ticks"].is_null());
            assert!(value["fields"].is_null());
        }
    }
}
