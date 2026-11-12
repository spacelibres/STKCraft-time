//! The crate version and every published reference must agree exactly.
//!
//! A patch release touches `Cargo.toml`, the README, the API guide, the
//! citation metadata and the issue template. These checks fail the build
//! when one of them is left behind, instead of shipping a page that still
//! advertises the previous version.

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Collects every current major/minor series version occurrence in `text`.
///
/// Only the crate's own version series is matched, so unrelated decimal
/// constants in the documents cannot produce a false positive.
fn crate_versions(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = text;
    let (series, _) = VERSION.rsplit_once('.').expect("semantic version");
    let prefix = format!("{series}.");

    while let Some(index) = rest.find(&prefix) {
        let candidate = &rest[index..];
        let digits = candidate[prefix.len()..]
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_digit())
            .last()
            .map(|(offset, ch)| offset + ch.len_utf8())
            .unwrap_or(0);

        if digits > 0 {
            found.push(&candidate[..prefix.len() + digits]);
        }
        rest = &candidate[prefix.len()..];
    }

    found
}

#[test]
fn documentation_versions_match_the_crate_version() {
    let files: [(&str, &str); 4] = [
        ("README.md", include_str!("../README.md")),
        ("docs/API.md", include_str!("../docs/API.md")),
        ("CITATION.cff", include_str!("../CITATION.cff")),
        (
            ".github/ISSUE_TEMPLATE/bug_report.yml",
            include_str!("../.github/ISSUE_TEMPLATE/bug_report.yml"),
        ),
    ];

    for (name, text) in files {
        let found = crate_versions(text);
        assert!(!found.is_empty(), "{name} has no crate version reference");

        for version in found {
            assert_eq!(
                version, VERSION,
                "{name} references {version}, but the crate version is {VERSION}"
            );
        }
    }
}

#[test]
fn install_snippets_pin_the_exact_release() {
    let readme = include_str!("../README.md");

    let plain = format!("stkcraft-time = \"{VERSION}\"");
    let with_features = format!("stkcraft-time = {{ version = \"{VERSION}\"");
    let tag = format!("tag = \"v{VERSION}\"");
    let crate_line = format!("`stkcraft-time` {VERSION}");

    assert!(readme.contains(&plain), "README misses `{plain}`");
    assert!(
        readme.contains(&with_features),
        "README misses `{with_features}`"
    );
    assert!(readme.contains(&tag), "README misses `{tag}`");
    assert!(readme.contains(&crate_line), "README misses `{crate_line}`");
}

#[test]
fn api_guide_installs_the_exact_release() {
    let guide = include_str!("../docs/API.md");

    let plain = format!("stkcraft-time = \"{VERSION}\"");
    let with_features = format!("version = \"{VERSION}\", default-features = false");
    let tag = format!("tag = \"v{VERSION}\"");

    assert!(guide.contains(&plain), "API guide misses `{plain}`");
    assert!(
        guide.contains(&with_features),
        "API guide misses `{with_features}`"
    );
    assert!(guide.contains(&tag), "API guide misses `{tag}`");
}
