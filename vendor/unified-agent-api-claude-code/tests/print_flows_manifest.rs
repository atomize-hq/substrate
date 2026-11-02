mod support_paths;

use std::{collections::BTreeMap, fs};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PrintFlowsManifest {
    schema_version: u32,
    flows: Vec<PrintFlowEntry>,
}

#[derive(Debug, Deserialize)]
struct PrintFlowEntry {
    id: String,
    examples: Vec<String>,
}

#[test]
fn print_flows_manifest_covers_required_flows() {
    let examples_dir = support_paths::claude_code_examples_dir();
    let manifest_path = examples_dir.join("print_flows_manifest.json");
    let bytes = fs::read(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let manifest: PrintFlowsManifest = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {e}", manifest_path.display()));
    assert_eq!(manifest.schema_version, 1);

    let required = [
        "print_stream_json",
        "print_include_partial_messages",
        "print_session_id",
        "multi_turn_resume",
        "multi_turn_fork_session",
        "multi_turn_continue",
    ];

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for flow in manifest.flows {
        assert!(!flow.id.trim().is_empty(), "flow id must be non-empty");
        assert!(
            !flow.examples.is_empty(),
            "print flows manifest entry for {} must list at least one example",
            flow.id
        );
        map.insert(flow.id, flow.examples);
    }

    let mut missing = Vec::new();
    for id in required {
        if !map.contains_key(id) {
            missing.push(id);
        }
    }
    assert!(
        missing.is_empty(),
        "missing required print-flow examples: {missing:?}"
    );

    for (id, examples) in map {
        for example in examples {
            let file = examples_dir.join(format!("{example}.rs"));
            assert!(
                file.is_file(),
                "print flows manifest entry for {id} references missing file {}",
                file.display()
            );
        }
    }
}
