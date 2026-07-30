#![allow(unused_crate_dependencies)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use serde_json::Value;

const PLANNED_CASE_IDS: [&str; 19] = [
    "P7-01", "P7-02", "P7-03", "P7-04", "P7-05", "P7-06", "P7-07", "P7-08", "P7-09", "P7-10",
    "P7-11", "P7-12", "P7-13", "P7-14", "P7-15", "P7-16", "P7-17", "P7-18", "P7-19",
];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecallMatrix {
    schema_version: u32,
    planned_count: usize,
    cases: Vec<RecallCase>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecallCase {
    case_id: String,
    title: String,
    implemented: bool,
    fixture_dir: String,
    adapter_classes: Vec<String>,
    event_variants: Vec<String>,
    logical_source_files: Vec<String>,
    selected_direct_closure: Vec<String>,
    semantic_family: String,
    contract_families: Vec<String>,
    parity_mode: String,
    parity_group: Option<String>,
    terminal_boundary: String,
    diagnostic_owners: Vec<String>,
    expected: Value,
    historical_references: Vec<String>,
}

fn fixture_root() -> Utf8PathBuf {
    Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/current_native_recall")
}

fn load_matrix() -> Result<RecallMatrix, String> {
    let path = fixture_root().join("matrix.json");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("P7 matrix inventory [owner=harness] {path}: {error}"))?;
    serde_json::from_str(&source)
        .map_err(|error| format!("P7 matrix schema [owner=harness] {path}: {error}"))
}

fn tracked_case_directories() -> Result<BTreeSet<String>, String> {
    let root = fixture_root();
    let entries = fs::read_dir(&root)
        .map_err(|error| format!("P7 matrix inventory [owner=harness] {root}: {error}"))?;
    let mut directories = BTreeSet::new();
    for entry in entries {
        let entry = entry
            .map_err(|error| format!("P7 matrix inventory [owner=harness] {root}: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("P7 matrix inventory [owner=harness] {root}: {error}"))?;
        if file_type.is_dir() {
            directories.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(directories)
}

fn validate_matrix(matrix: &RecallMatrix) -> Result<(), String> {
    if matrix.schema_version != 1 {
        return Err(format!(
            "P7 matrix schema [owner=harness] expected schema_version=1, got {}",
            matrix.schema_version
        ));
    }
    if matrix.planned_count != PLANNED_CASE_IDS.len() {
        return Err(format!(
            "P7 matrix inventory [owner=harness] expected planned_count={}, got {}",
            PLANNED_CASE_IDS.len(),
            matrix.planned_count
        ));
    }

    let planned = PLANNED_CASE_IDS.into_iter().collect::<BTreeSet<_>>();
    let mut by_id = BTreeMap::new();
    for case in &matrix.cases {
        if !planned.contains(case.case_id.as_str()) {
            return Err(case_error(
                case,
                &format!("unknown case_id {}", case.case_id),
            ));
        }
        if by_id.insert(case.case_id.as_str(), case).is_some() {
            return Err(case_error(case, "duplicate matrix entry"));
        }
        validate_case_metadata(case)?;
    }

    for case_id in PLANNED_CASE_IDS {
        if !by_id.contains_key(case_id) {
            return Err(format!(
                "{case_id} failed at matrix inventory [owner=harness]: missing matrix entry"
            ));
        }
    }
    if matrix.cases.len() != matrix.planned_count {
        return Err(format!(
            "P7 matrix inventory [owner=harness] expected {} cases, got {}",
            matrix.planned_count,
            matrix.cases.len()
        ));
    }
    Ok(())
}

fn validate_case_metadata(case: &RecallCase) -> Result<(), String> {
    for (field, value) in [
        ("title", case.title.as_str()),
        ("fixture_dir", case.fixture_dir.as_str()),
        ("semantic_family", case.semantic_family.as_str()),
        ("parity_mode", case.parity_mode.as_str()),
        ("terminal_boundary", case.terminal_boundary.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(case_error(case, &format!("{field} must not be empty")));
        }
    }
    for (field, values) in [
        ("adapter_classes", &case.adapter_classes),
        ("event_variants", &case.event_variants),
        ("logical_source_files", &case.logical_source_files),
        ("selected_direct_closure", &case.selected_direct_closure),
        ("contract_families", &case.contract_families),
        ("diagnostic_owners", &case.diagnostic_owners),
        ("historical_references", &case.historical_references),
    ] {
        if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
            return Err(case_error(
                case,
                &format!("{field} must contain non-empty values"),
            ));
        }
    }
    if !matches!(
        case.parity_mode.as_str(),
        "equivalent" | "current_native_only" | "not_applicable"
    ) {
        return Err(case_error(case, "unknown parity_mode"));
    }
    if case.parity_mode == "equivalent"
        && (case.parity_group.as_deref().is_none_or(str::is_empty)
            || !case.adapter_classes.iter().any(|value| value == "Legacy")
            || !case
                .adapter_classes
                .iter()
                .any(|value| value == "CurrentNativeV2"))
    {
        return Err(case_error(
            case,
            "equivalent parity requires a group plus Legacy and CurrentNativeV2",
        ));
    }
    if !case.expected.is_object()
        || case
            .expected
            .as_object()
            .is_none_or(serde_json::Map::is_empty)
    {
        return Err(case_error(case, "expected must be a non-empty object"));
    }
    Ok(())
}

fn validate_case_inventory(
    matrix: &RecallMatrix,
    directories: &BTreeSet<String>,
) -> Result<(), String> {
    let declared = matrix
        .cases
        .iter()
        .map(|case| case.fixture_dir.as_str())
        .collect::<BTreeSet<_>>();
    for directory in directories {
        if !declared.contains(directory.as_str()) {
            return Err(format!(
                "{directory} failed at matrix inventory [owner=harness]: extra fixture directory"
            ));
        }
    }
    for case in &matrix.cases {
        let exists = directories.contains(&case.fixture_dir);
        if case.implemented && !exists {
            return Err(case_error(
                case,
                "implemented case is missing its fixture directory",
            ));
        }
        if !case.implemented && exists {
            return Err(case_error(
                case,
                "fixture directory exists before the case is marked implemented",
            ));
        }
    }
    Ok(())
}

fn case_error(case: &RecallCase, message: &str) -> String {
    format!(
        "{} failed at {} [owner={}]: {message}",
        case.case_id,
        case.terminal_boundary,
        case.diagnostic_owners.join(",")
    )
}

#[test]
fn current_native_recall_matrix_is_complete_and_tracked_inventory_is_valid() {
    let matrix = load_matrix().expect("load P7 matrix");
    validate_matrix(&matrix).expect("validate P7 matrix");
    let directories = tracked_case_directories().expect("list P7 fixture directories");
    validate_case_inventory(&matrix, &directories).expect("validate P7 fixture inventory");
}

#[test]
fn current_native_recall_inventory_negative_controls_fail_closed() {
    let matrix = load_matrix().expect("load P7 matrix");

    let mut missing = matrix.clone();
    missing.cases.retain(|case| case.case_id != "P7-19");
    let error = validate_matrix(&missing).expect_err("missing P7-19 must fail");
    assert!(error.contains("P7-19"));
    assert!(error.contains("matrix inventory"));
    assert!(error.contains("owner=harness"));

    let mut duplicate = matrix.clone();
    duplicate.cases.push(duplicate.cases[0].clone());
    let error = validate_matrix(&duplicate).expect_err("duplicate P7-01 must fail");
    assert!(error.contains("P7-01"));
    assert!(error.contains("public-live checkpoint"));
    assert!(error.contains("owner=scoring,public-live"));

    let mut unknown = matrix.clone();
    let last = unknown.cases.last_mut().expect("P7-19");
    last.case_id = "P7-99".to_string();
    let error = validate_matrix(&unknown).expect_err("unknown P7-99 must fail");
    assert!(error.contains("P7-99"));
    assert!(error.contains("canonical projection"));
    assert!(error.contains("owner=ingest,canonical-projection"));

    let extra = BTreeSet::from(["p7-99".to_string()]);
    let error = validate_case_inventory(&matrix, &extra).expect_err("extra directory must fail");
    assert!(error.contains("p7-99"));
    assert!(error.contains("matrix inventory"));
    assert!(error.contains("owner=harness"));

    let mut missing_fixture = matrix.clone();
    missing_fixture.cases[0].implemented = true;
    let error = validate_case_inventory(&missing_fixture, &BTreeSet::new())
        .expect_err("implemented P7-01 without a directory must fail");
    assert!(error.contains("P7-01"));
    assert!(error.contains("public-live checkpoint"));
    assert!(error.contains("owner=scoring,public-live"));
}
