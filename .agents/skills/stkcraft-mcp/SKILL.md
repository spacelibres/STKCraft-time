---
name: stkcraft-mcp
description: Use when running, configuring, testing or extending the stkcraft-mcp stdio MCP server that exposes UTC/TAI/TT/ET/GPS/UT1/SCLK conversions as tools.
---

# stkcraft-mcp

The MCP server is optional and lives behind the `mcp` feature:

```sh
cargo build --release --features mcp --bin stkcraft-mcp
cargo run   --release --features mcp --bin stkcraft-mcp
```

It speaks newline-delimited JSON-RPC 2.0 on stdin/stdout. Never write logs
to stdout; use stderr.

## Client configuration

```json
{
  "mcpServers": {
    "stkcraft-time": {
      "command": "/absolute/path/to/target/release/stkcraft-mcp",
      "args": []
    }
  }
}
```

## Protocol flow

1. Client sends `initialize` with `protocolVersion`, `capabilities` and
   `clientInfo`.
2. Server responds with the negotiated version, `capabilities.tools` and
   `serverInfo`.
3. Client sends `notifications/initialized` (no response).
4. Client calls `tools/list` and `tools/call`.

`handle_message` in `src/mcp.rs` returns `None` for notifications.
Library errors from a tool are returned as `isError: true` text content,
not as JSON-RPC errors. Invalid requests, including a missing `method`,
return `-32600`. The server advertises only protocol version
`2024-11-05`. Tool panics are caught per request and returned as
`isError`, so one bad call does not stop the server.

## Tools

| Tool | Required arguments |
|---|---|
| `lsk_info` | — |
| `delta_at` | `jdn`, or `year`/`month`/`day` |
| `utc_to_tai` | UTC fields |
| `tai_to_utc` | `tai_seconds` |
| `utc_to_et` | UTC fields |
| `et_to_utc` | `et` |
| `tai_to_tt` | `tai_seconds` |
| `tai_to_gps` | `tai_seconds` |
| `gps_to_tai` | `gps_seconds` |
| `utc_to_ut1` | UTC fields, `eop_path` |
| `ut1_to_utc` | `ut1_seconds`, `eop_path` |
| `sclk_to_et` | `sclk_path`, `spacecraft_id`, `ticks` or `clock_string` |
| `et_to_sclk` | `sclk_path`, `spacecraft_id`, `et` |
| `format_utc` | UTC fields, optional `kind`/`precision`/`pattern` |
| `etcal` | `et` |

UTC tools accept `calendar` = `gregorian` (default), `julian` or `mixed`.
All tools accept an optional `lsk_path`; omit it to use the bundled
`naif0012.tls`. EOP tools also accept `eop_series` (`a`, `b`, `final`) and
`observed_only`. `precision` is limited to `0..=9`. TAI tools accept
`tai_seconds`, or exact `sec` + `nanos`. SCLK tools accept continuous
ticks; `fields` and `clock_string` are `null` for non-integral ticks.

## Example call

```json
{
  "jsonrpc": "2.0",
  "id": 1,
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
}
```

The result is one text content block containing pretty-printed JSON with
`utc` and `et`.

## Extending the server

1. Add the tool schema in `tool_definitions()` in `src/mcp.rs`.
2. Add the dispatch arm in `call_tool`.
3. Reuse `load_lsk`, `load_eop`, `load_sclk`, `utc_arg`, `calendar_arg`
   and the `*_json` helpers instead of duplicating parsing.
4. Add a protocol test in `tests/mcp.rs`.
5. Run:

```sh
cargo test --features mcp
cargo clippy --all-targets --features mcp -- -D warnings
```

## Security and limits

- Reads only caller-supplied local paths; no network I/O.
- Only type-1 SCLK kernels are supported.
- `23:59:60.x` without a positive leap second and `23:59:59.x` on a
  shortened day are rejected.
- Non-finite EOP samples are rejected with `EopParse`.
- UT1 needs bracketing EOP samples around the UTC day; the last sample day is not usable as an epoch boundary.

See `docs/MCP.md` for full schemas and examples.
