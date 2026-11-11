# `stkcraft-mcp` — MCP server

`stkcraft-mcp` exposes the `stkcraft-time` API as a
[Model Context Protocol](https://modelcontextprotocol.io/) server over
stdio. It is optional and requires the `mcp` feature (which enables `std`
and `serde_json`).

## Build and run

```sh
cargo build --release --features mcp --bin stkcraft-mcp
cargo run   --release --features mcp --bin stkcraft-mcp
```

The process speaks newline-delimited JSON-RPC 2.0 on stdin/stdout. Logs
and errors go to stderr; stdout is reserved for protocol messages.

## Client configuration

Point your MCP client at the built binary:

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

During development, `cargo run --release` works as well:

```json
{
  "mcpServers": {
    "stkcraft-time": {
      "command": "cargo",
      "args": [
        "run",
        "--release",
        "--features", "mcp",
        "--bin", "stkcraft-mcp",
        "--manifest-path", "/absolute/path/to/STKCraft-time/Cargo.toml"
      ]
    }
  }
}
```

## Protocol

Supported requests:

| Method | Purpose |
|---|---|
| `initialize` | Negotiates the protocol and reports the `tools` capability. |
| `tools/list` | Returns the tool definitions and JSON input schemas. |
| `tools/call` | Invokes one tool. Library errors are returned as `isError: true` content. |
| `ping` | Liveness check. |

Notifications such as `notifications/initialized` receive no response,
as required by JSON-RPC 2.0. The server advertises only MCP protocol
version `2024-11-05`; a request for another version receives that
supported version in the `initialize` result. Invalid requests, including
a missing `method`, receive a `-32600` error, and oversized messages are
rejected.

## Tools

All tools accept an optional `lsk_path`. When it is omitted, the server
uses the bundled `naif0012.tls` snapshot (last leap second `2017-01-01`).
UTC tools accept `calendar`: `gregorian` (default), `julian`, or `mixed`.

| Tool | Required arguments | Returns |
|---|---|---|
| `lsk_info` | — (`lsk_path` optional) | Coverage dates/JDNs, entries, source and policy. |
| `delta_at` | `jdn`, or `year`/`month`/`day` | `TAI - UTC` in whole seconds. |
| `utc_to_tai` | UTC fields | TAI seconds, sec/nanos, ISO 8601. |
| `tai_to_utc` | `tai_seconds`, or `sec` + `nanos` | Civil UTC fields and ISO 8601. |
| `utc_to_et` | UTC fields | NAIF ET. |
| `et_to_utc` | `et` | Civil UTC fields and ISO 8601. |
| `tai_to_tt` | `tai_seconds`, or `sec` + `nanos` | TT seconds past J2000 TT. |
| `tai_to_gps` | `tai_seconds`, or `sec` + `nanos` | GPS seconds past the GPS epoch. |
| `gps_to_tai` | `gps_seconds` | TAI epoch. |
| `utc_to_ut1` | UTC fields, `eop_path` | UT1 seconds past `2000-01-01T00:00:00 UT1`. |
| `ut1_to_utc` | `ut1_seconds`, `eop_path` | Civil UTC fields and ISO 8601. |
| `sclk_to_et` | `sclk_path`, `spacecraft_id`, and `ticks` or `clock_string` | Ticks, optional fields, ET and TAI. |
| `et_to_sclk` | `sclk_path`, `spacecraft_id`, `et` | Ticks, optional fields and clock string. |
| `format_utc` | UTC fields, optional `kind`, `precision`, `pattern` | Formatted string. |
| `etcal` | `et`, optional `precision` | SPICE calendar string. |

EOP tools also accept `eop_series` (`a`, `b`, `final`) and
`observed_only` (boolean). `sclk_path` must be a type-1 SCLK kernel.

`precision` is limited to `0..=9`. `tai_seconds` may be replaced by an
integer `sec` plus `nanos` for exact epoch exchange. SCLK tools accept
continuous ticks; `fields` and `clock_string` are `null` when the tick
value is not integral.

### `utc_to_et` example

Request:

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

Response content (pretty-printed here; the server returns it as one text
block):

```json
{
  "utc": {
    "year": 2016,
    "month": 12,
    "day": 31,
    "hour": 23,
    "minute": 59,
    "second": 60.5,
    "iso8601": "2016-12-31T23:59:60.500000000Z"
  },
  "et": 536500868.6839298
}
```

### `sclk_to_et` example

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "sclk_to_et",
    "arguments": {
      "sclk_path": "/path/to/MRO_SCLKSCET.00001.tsc",
      "spacecraft_id": -74,
      "clock_string": "123:45"
    }
  }
}
```

## Behaviour and limits

- The server reads only the files named by the caller. It performs no
  network I/O.
- UTC before the first LSK entry returns a tool error
  (`UtcOutOfRange`); it is never guessed.
- `23:59:60.x` without a positive leap second, and `23:59:59.x` on a
  shortened day, return `InvalidLeapSecond`.
- Non-finite EOP samples are rejected with `EopParse`.
- UT1 requires bracketing EOP samples around the UTC day; the last
  sample day is not usable as an epoch boundary.
- SCLK support is limited to type-1 clocks. Partitions are parsed but not
  applied; field/string conversion returns
  `SclkPartitionsUnsupported` for partitioned clocks, while encoded
  ticks↔ET/TAI remain available.
- Tool arguments are JSON numbers/strings. To pass a leap-second label,
  use `"second": 60.5`; do not pass a formatted string as a number.

## Tests

```sh
cargo test --features mcp
cargo clippy --all-targets --features mcp -- -D warnings
```
