//! Keeps the public API index in `docs/API.md` complete.
//!
//! Every `pub` item in `src/` must appear in the index, and every name in
//! the index must still exist. The check is textual on purpose: it needs no
//! extra dependency, it runs in `no_std`-agnostic `cargo test`, and it fails
//! with the exact names that drifted.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const INDEX_HEADING: &str = "## 14. Public API index";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Collects `pub` declarations from a Rust source fragment.
///
/// `pub(crate)` does not match: the prefix requires a space after `pub`.
/// Test-only modules are cut off by the caller before parsing.
fn public_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for line in text.lines() {
        let line = line.trim_start();
        if line.starts_with("//") || line.starts_with("///") {
            continue;
        }

        let rest = match line.strip_prefix("pub ") {
            Some(rest) => rest,
            None => continue,
        };

        let rest = ["fn ", "struct ", "enum ", "const ", "type ", "trait "]
            .iter()
            .find_map(|keyword| rest.strip_prefix(keyword));
        let rest = match rest {
            Some(rest) => rest,
            None => continue,
        };

        let end = rest
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .unwrap_or(rest.len());
        if end > 0 {
            names.insert(rest[..end].to_string());
        }
    }

    names
}

/// Reads library Rust files recursively, excluding binaries and test modules.
fn source_names() -> BTreeSet<String> {
    let dir = manifest_dir().join("src");
    let binaries = dir.join("bin");
    let mut names = BTreeSet::new();
    let mut directories = vec![dir];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).expect("library directory must be readable") {
            let path = entry.expect("src entry").path();
            if path.is_dir() {
                if path != binaries {
                    directories.push(path);
                }
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("source file must be readable");
            let text = match text.find("#[cfg(test)]") {
                Some(index) => &text[..index],
                None => text.as_str(),
            };
            names.extend(public_names(text));
        }
    }

    names
}

fn index_names() -> BTreeSet<String> {
    let path = manifest_dir().join("docs").join("API.md");
    let text = fs::read_to_string(&path).expect("docs/API.md must be readable");
    let index = text
        .find(INDEX_HEADING)
        .expect("docs/API.md must contain the public API index heading");
    public_names(&text[index..])
}

fn report(label: &str, names: &BTreeSet<String>) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n{label} ({}):", names.len()));
    for name in names {
        out.push_str("\n  ");
        out.push_str(name);
    }
    out
}

#[test]
fn api_index_lists_every_public_item() {
    let source = source_names();
    let index = index_names();

    assert!(
        !source.is_empty(),
        "no public items were parsed from {}",
        manifest_dir().join("src").display()
    );

    let missing: BTreeSet<_> = source.difference(&index).cloned().collect();
    let stale: BTreeSet<_> = index.difference(&source).cloned().collect();

    assert!(
        missing.is_empty() && stale.is_empty(),
        "docs/API.md index is out of sync with the source.{}{}",
        report("missing from docs/API.md", &missing),
        report("listed in docs/API.md but absent from src", &stale)
    );
}

#[test]
fn parser_ignores_crate_private_items() {
    let parsed = public_names(
        "pub fn exported() {}\n\
         pub(crate) fn crate_private() {}\n\
         fn module_private() {}\n\
         pub struct Exported;\n",
    );

    assert!(parsed.contains("exported"));
    assert!(parsed.contains("Exported"));
    assert!(!parsed.contains("crate_private"));
    assert!(!parsed.contains("module_private"));
}

#[test]
fn index_file_is_present_in_the_package() {
    let docs = manifest_dir().join("docs").join("API.md");
    assert!(
        Path::new(&docs).is_file(),
        "{} must be committed",
        docs.display()
    );
}
