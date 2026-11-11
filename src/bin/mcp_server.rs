//! Newline-delimited JSON-RPC MCP server over stdin/stdout.
//!
//! Build and run:
//!
//! ```sh
//! cargo run --features mcp --bin stkcraft-mcp
//! ```
//!
//! See `docs/MCP.md` for the tool list and client configuration.

fn main() {
    if let Err(error) = stkcraft_time::mcp::run_stdio() {
        eprintln!("stkcraft-mcp: {error}");
        std::process::exit(1);
    }
}
