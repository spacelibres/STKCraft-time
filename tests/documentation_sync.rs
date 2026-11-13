//! Guard against previously observed misleading capability and kernel claims.

#[test]
fn documented_binding_and_mcp_capabilities_match_the_current_contract() {
    let bindings = include_str!("../docs/BINDINGS.md");
    assert!(bindings.contains("JavaScript and Python both expose"));
    assert!(!bindings.contains("JavaScript binding does not yet expose"));

    let mcp = include_str!("../docs/MCP.md");
    assert!(mcp.contains("WGC GPS and calendar tools accept an optional `lsk_path`"));
    assert!(!mcp.contains("GPS tools and `etcal` need no leap-second kernel"));
}
