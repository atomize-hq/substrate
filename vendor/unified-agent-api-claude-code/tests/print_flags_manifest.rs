mod support_paths;

use std::{collections::BTreeMap, fs};

use claude_code::wrapper_coverage_manifest::{wrapper_coverage_manifest, CoverageLevel};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PrintFlagsManifest {
    schema_version: u32,
    flags: Vec<PrintFlagEntry>,
}

#[derive(Debug, Deserialize)]
struct PrintFlagEntry {
    key: String,
    examples: Vec<String>,
}

#[test]
fn print_flags_manifest_covers_all_explicit_root_flags() {
    let examples_dir = support_paths::claude_code_examples_dir();
    let manifest_path = examples_dir.join("print_flags_manifest.json");
    let bytes = fs::read(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let manifest: PrintFlagsManifest = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {e}", manifest_path.display()));
    assert_eq!(manifest.schema_version, 1);

    let wrapper = wrapper_coverage_manifest();
    let root = wrapper
        .coverage
        .iter()
        .find(|cmd| cmd.path.is_empty())
        .expect("root wrapper command present");
    let root_flags = root.flags.as_ref().expect("root flags present");

    let mut required: Vec<String> = Vec::new();
    for f in root_flags {
        if f.level == CoverageLevel::Explicit {
            required.push(f.key.clone());
        }
    }

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in manifest.flags {
        assert!(
            !entry.key.trim().is_empty(),
            "print flags manifest key must be non-empty"
        );
        assert!(
            !entry.examples.is_empty(),
            "print flags manifest entry for {} must list at least one example",
            entry.key
        );
        map.insert(entry.key, entry.examples);
    }

    let mut missing = Vec::new();
    for k in &required {
        if !map.contains_key(k) {
            missing.push(k);
        }
    }
    assert!(
        missing.is_empty(),
        "missing example coverage for explicit root flags: {missing:?}"
    );

    let required_set: std::collections::BTreeSet<String> = required.into_iter().collect();
    let mut unknown = Vec::new();
    for k in map.keys() {
        if !required_set.contains(k) {
            unknown.push(k.clone());
        }
    }
    assert!(
        unknown.is_empty(),
        "print flags manifest contains unknown keys (not explicit root flags): {unknown:?}"
    );

    for (key, examples) in map {
        for example in examples {
            let file = examples_dir.join(format!("{example}.rs"));
            assert!(
                file.is_file(),
                "print flags manifest entry for {key} references missing file {}",
                file.display()
            );
        }
    }
}
