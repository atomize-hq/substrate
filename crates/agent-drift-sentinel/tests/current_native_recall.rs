#![allow(unused_crate_dependencies)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::Command;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, AnalyzeResult, Checkpoint, DriftClass, DriftState,
};
use agent_drift_sentinel::{
    CheckpointCursor, LiveCheckpointEvent, LiveRuntime, LiveRuntimeError, LiveSessionCoordinator,
    LiveSessionRequest, SchedulerPolicy, TriggerClass, WarningPolicy,
};
use agent_session_compactor::{
    BoundedClosureCompactor, BoundedClosureError, BoundedClosureRequest, CompactorError,
    RolloutFormat,
};
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
    let allowed_terminal_owners: &[&str] = match case.terminal_boundary.as_str() {
        "public-live checkpoint" => &["public-live"],
        "analyzer scoring" => &["scoring"],
        "analyzer delegation semantics" => &["delegation-semantics"],
        "analyzer input" => &["analyzer-input"],
        "closure selection" => &["closure-selection"],
        "closure verification" => &["closure-verification"],
        "public-live validation" => &["public-live-validation"],
        "canonical projection" => &["canonical-projection", "parity-comparison"],
        _ => return Err(case_error(case, "unknown terminal_boundary")),
    };
    if !allowed_terminal_owners.iter().any(|required| {
        case.diagnostic_owners
            .iter()
            .any(|actual| actual == required)
    }) {
        return Err(case_error(
            case,
            &format!(
                "terminal boundary requires one of these diagnostic owners: {}",
                allowed_terminal_owners.join(",")
            ),
        ));
    }
    let expected_path = fixture_root().join(&case.fixture_dir).join("expected.json");
    let expected_source = fs::read_to_string(&expected_path).map_err(|error| {
        case_error(
            case,
            &format!("cannot read expected projection {expected_path}: {error}"),
        )
    })?;
    let expected_fixture: Value = serde_json::from_str(&expected_source).map_err(|error| {
        case_error(
            case,
            &format!("cannot parse expected projection {expected_path}: {error}"),
        )
    })?;
    if case.expected != expected_fixture {
        return Err(case_error(
            case,
            "matrix expected projection does not match expected.json",
        ));
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

    let mut expected_mismatch = matrix.clone();
    expected_mismatch.cases[0].expected["public_trigger"] = Value::String("other".to_string());
    let error =
        validate_matrix(&expected_mismatch).expect_err("matrix expected mutation must fail");
    assert!(error.contains("P7-01"));
    assert!(error.contains("matrix expected projection does not match expected.json"));
    assert!(error.contains("owner=scoring,public-live"));

    let mut owner_mismatch = matrix.clone();
    owner_mismatch.cases[6].diagnostic_owners = vec!["objective-path-extraction".to_string()];
    let error =
        validate_matrix(&owner_mismatch).expect_err("P7-07 terminal owner mutation must fail");
    assert!(error.contains("P7-07"));
    assert!(error.contains("terminal boundary requires"));
    assert!(error.contains("owner=objective-path-extraction"));

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

fn recursively_validate_privacy(root: &Utf8Path) -> Result<(), String> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let metadata =
            fs::metadata(&path).map_err(|error| privacy_error(&path, &error.to_string()))?;
        if metadata.is_dir() {
            let mut children = fs::read_dir(&path)
                .map_err(|error| privacy_error(&path, &error.to_string()))?
                .map(|entry| {
                    entry
                        .map(|entry| Utf8PathBuf::from_path_buf(entry.path()))
                        .map_err(|error| error.to_string())
                        .and_then(|path| {
                            path.map_err(|path| format!("non-UTF-8 fixture path {path:?}"))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            children.sort();
            pending.extend(children.into_iter().rev());
            continue;
        }

        let source =
            fs::read_to_string(&path).map_err(|error| privacy_error(&path, &error.to_string()))?;
        validate_forbidden_markers(&path, &source)?;
        match path.extension() {
            Some("json") => {
                let value: Value = serde_json::from_str(&source)
                    .map_err(|error| privacy_error(&path, &format!("invalid JSON: {error}")))?;
                validate_json_identifiers(&path, &value, &mut Vec::new())?;
            }
            Some("jsonl") => {
                for (index, line) in source.lines().enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let value: Value = serde_json::from_str(line).map_err(|error| {
                        privacy_error(&path, &format!("line {} invalid JSON: {error}", index + 1))
                    })?;
                    validate_json_identifiers(&path, &value, &mut Vec::new())?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_forbidden_markers(path: &Utf8Path, source: &str) -> Result<(), String> {
    for marker in [
        "/Users/",
        "/home/",
        "/private/",
        "C:\\Users\\",
        "__Active_Code",
        "spensermcconnell",
        "atomize-hq",
        "ghp_",
        "github_pat_",
        "AKIA",
        "sk-proj-",
    ] {
        if source.contains(marker) {
            return Err(privacy_error(path, &format!("forbidden marker {marker:?}")));
        }
    }
    Ok(())
}

fn privacy_error(path: &Utf8Path, message: &str) -> String {
    let case_id = path
        .components()
        .map(|component| component.as_str())
        .find(|component| {
            component.len() == 5
                && component.starts_with("p7-")
                && component[3..].bytes().all(|byte| byte.is_ascii_digit())
        })
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| "P7-MATRIX".to_string());
    format!("{case_id} privacy scan [owner=harness] file={path}: {message}")
}

fn validate_json_identifiers(
    path: &Utf8Path,
    value: &Value,
    key_path: &mut Vec<String>,
) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                key_path.push(key.clone());
                validate_json_identifiers(path, child, key_path)?;
                key_path.pop();
            }
        }
        Value::Array(values) => {
            for child in values {
                validate_json_identifiers(path, child, key_path)?;
            }
        }
        Value::String(value) => {
            if let Some(kind) = identifier_kind(key_path, value) {
                validate_identifier(path, key_path, kind, value)?;
            }
            if is_path_field(key_path) {
                validate_path_value(path, key_path, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn identifier_kind<'a>(key_path: &[String], value: &'a str) -> Option<&'a str> {
    let key = key_path.last().map(String::as_str).unwrap_or_default();
    if key == "turn_id" || value.starts_with("turn-") {
        Some("turn")
    } else if key == "call_id" || value.starts_with("call-") {
        Some("call")
    } else if key == "event_id" || value.starts_with("event-") {
        Some("event")
    } else if key == "agent_id" || key == "agent_thread_id" || value.starts_with("agent-") {
        Some("agent")
    } else if key == "repository_id" || key == "repo_id" || value.starts_with("repo-") {
        Some("repo")
    } else if key.ends_with("session_id")
        || value.starts_with("session-")
        || (key == "id" && key_path.iter().any(|component| component == "session_meta"))
    {
        Some("session")
    } else {
        None
    }
}

fn validate_identifier(
    path: &Utf8Path,
    key_path: &[String],
    kind: &str,
    value: &str,
) -> Result<(), String> {
    let expected_prefix = format!("{kind}-p7-");
    let suffix = value.strip_prefix(&expected_prefix).ok_or_else(|| {
        privacy_error(
            path,
            &format!(
                "{}={value:?} violates {kind} synthetic grammar",
                key_path.join(".")
            ),
        )
    })?;
    let (case_number, tail) = suffix.split_once('-').ok_or_else(|| {
        privacy_error(
            path,
            &format!(
                "{}={value:?} violates {kind} synthetic grammar",
                key_path.join(".")
            ),
        )
    })?;
    let valid_case_number = case_number.len() == 2
        && case_number.bytes().all(|byte| byte.is_ascii_digit())
        && matches!(case_number.parse::<u8>(), Ok(1..=19));
    let valid_tail = !tail.is_empty()
        && !tail.starts_with('-')
        && !tail.ends_with('-')
        && tail
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !valid_case_number || !valid_tail {
        return Err(privacy_error(
            path,
            &format!(
                "{}={value:?} violates {kind} synthetic grammar",
                key_path.join(".")
            ),
        ));
    }
    Ok(())
}

fn is_path_field(key_path: &[String]) -> bool {
    let key = key_path.last().map(String::as_str).unwrap_or_default();
    key == "path" || key == "file_path" || key == "repository_path" || key.ends_with("_root")
}

fn validate_path_value(path: &Utf8Path, key_path: &[String], value: &str) -> Result<(), String> {
    let candidate = Utf8Path::new(value);
    let valid = !candidate.is_absolute()
        && !value.contains('\\')
        && !candidate
            .components()
            .any(|component| component.as_str() == "..")
        && (value.starts_with("workspace-p7-")
            || value.starts_with("raw/")
            || value.starts_with("legacy/")
            || value.starts_with("current-native/")
            || value.starts_with("bundle/")
            || value.starts_with("crates/")
            || value.starts_with("docs/")
            || value.starts_with("scripts/"));
    if !valid {
        return Err(privacy_error(
            path,
            &format!(
                "{}={value:?} violates path synthetic grammar",
                key_path.join(".")
            ),
        ));
    }
    Ok(())
}

fn validate_implemented_case_provenance(
    root: &Utf8Path,
    matrix: &RecallMatrix,
) -> Result<(), String> {
    for case in matrix.cases.iter().filter(|case| case.implemented) {
        let case_root = root.join(&case.fixture_dir);
        let mut actual_adapters = BTreeSet::new();
        let mut actual_variants = BTreeSet::new();
        for logical_source in &case.logical_source_files {
            let source_path = case_root.join(logical_source);
            if !source_path.is_file() {
                return Err(case_error(
                    case,
                    &format!("declared source {logical_source:?} is missing"),
                ));
            }
            match source_path.extension() {
                Some("jsonl") => collect_jsonl_provenance(
                    case,
                    logical_source,
                    &source_path,
                    &mut actual_adapters,
                    &mut actual_variants,
                )?,
                Some("json") if logical_source.ends_with("manifest.json") => {
                    actual_adapters.insert("BundleV0_2".to_string());
                    actual_variants.insert("manifest".to_string());
                    let value: Value = serde_json::from_str(
                        &fs::read_to_string(&source_path)
                            .map_err(|error| case_error(case, &error.to_string()))?,
                    )
                    .map_err(|error| case_error(case, &error.to_string()))?;
                    if value["delegation_links"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|link| link["state"] == "verified")
                    {
                        actual_variants.insert("verified_link".to_string());
                    }
                }
                Some("json") if logical_source.ends_with("live-event.json") => {
                    actual_adapters.insert("PublicLive".to_string());
                    let value: Value = serde_json::from_str(
                        &fs::read_to_string(&source_path)
                            .map_err(|error| case_error(case, &error.to_string()))?,
                    )
                    .map_err(|error| case_error(case, &error.to_string()))?;
                    let variant = value
                        .get("trigger")
                        .and_then(Value::as_str)
                        .ok_or_else(|| case_error(case, "live event must declare trigger"))?;
                    actual_variants.insert(variant.to_string());
                }
                _ => {
                    return Err(case_error(
                        case,
                        &format!("unsupported declared source {logical_source:?}"),
                    ));
                }
            }
        }
        let expected_adapters = case
            .adapter_classes
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if actual_adapters != expected_adapters {
            return Err(case_error(
                case,
                &format!(
                    "adapter provenance mismatch: declared={expected_adapters:?} actual={actual_adapters:?}"
                ),
            ));
        }
        let expected_variants = case.event_variants.iter().cloned().collect::<BTreeSet<_>>();
        if actual_variants != expected_variants {
            return Err(case_error(
                case,
                &format!(
                    "event variant provenance mismatch: declared={expected_variants:?} actual={actual_variants:?}"
                ),
            ));
        }
    }
    Ok(())
}

fn collect_jsonl_provenance(
    case: &RecallCase,
    logical_source: &str,
    source_path: &Utf8Path,
    actual_adapters: &mut BTreeSet<String>,
    actual_variants: &mut BTreeSet<String>,
) -> Result<(), String> {
    let source =
        fs::read_to_string(source_path).map_err(|error| case_error(case, &error.to_string()))?;
    let mut has_current_native_marker = false;
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).map_err(|error| {
            case_error(
                case,
                &format!(
                    "{} line {} is invalid JSON: {error}",
                    logical_source,
                    index + 1
                ),
            )
        })?;
        let outer = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| case_error(case, "raw record must declare type"))?;
        if outer == "session_meta"
            && value
                .pointer("/payload/multi_agent_version")
                .and_then(Value::as_str)
                == Some("v2")
        {
            has_current_native_marker = true;
        }
        match outer {
            "response_item" | "event_msg" => {
                let variant = value
                    .pointer("/payload/type")
                    .and_then(Value::as_str)
                    .ok_or_else(|| case_error(case, "typed record must declare payload.type"))?;
                if outer == "response_item"
                    && value
                        .pointer("/payload/content")
                        .is_some_and(|content| !content.is_array())
                {
                    actual_variants.insert("malformed_response_item".to_string());
                } else {
                    actual_variants.insert(variant.to_string());
                }
            }
            other => {
                actual_variants.insert(other.to_string());
            }
        }
    }

    if logical_source.starts_with("legacy/") {
        actual_adapters.insert("Legacy".to_string());
        actual_variants.insert("legacy_event".to_string());
    } else if has_current_native_marker {
        actual_adapters.insert("CurrentNativeV2".to_string());
    } else {
        return Err(case_error(
            case,
            &format!("{logical_source:?} lacks multi_agent_version=v2"),
        ));
    }
    Ok(())
}

#[test]
fn current_native_recall_privacy_and_provenance_are_recursive() {
    let root = fixture_root();
    recursively_validate_privacy(&root).expect("validate tracked P7 privacy");
    let matrix = load_matrix().expect("load P7 matrix");
    validate_implemented_case_provenance(&root, &matrix)
        .expect("validate implemented P7 provenance");
}

#[test]
fn current_native_recall_validation_receipt_is_privacy_safe() {
    let path = Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-validation.md");
    recursively_validate_privacy(&path).expect("validate committed P7 receipt privacy");
}

#[test]
fn current_native_recall_privacy_identifier_mutations_fail_closed() {
    let path = Utf8Path::new("fixtures/privacy-mutation.json");
    let mutations = [
        ("session", serde_json::json!({"session_id": "real-session"})),
        ("turn", serde_json::json!({"turn_id": "turn-real"})),
        ("call", serde_json::json!({"call_id": "call-real"})),
        ("event", serde_json::json!({"event_id": "event-real"})),
        ("agent", serde_json::json!({"agent_id": "agent-real"})),
        (
            "repo",
            serde_json::json!({"repository_id": "repository-real"}),
        ),
        ("path", serde_json::json!({"path": "../private/source.rs"})),
    ];
    for (kind, value) in mutations {
        let error = validate_json_identifiers(path, &value, &mut Vec::new())
            .expect_err("invalid synthetic identifier must fail");
        assert!(error.contains(kind), "{kind} mutation: {error}");
        assert!(error.contains("owner=harness"), "{kind} mutation: {error}");
    }

    let marker_error = validate_forbidden_markers(
        path,
        r#"{"path":"/Users/private/project","token":"sk-proj-secret"}"#,
    )
    .expect_err("private marker must fail");
    assert!(marker_error.contains("/Users/"));
    assert!(marker_error.contains("owner=harness"));
}

#[test]
fn current_native_recall_adapter_and_variant_mutations_fail_closed() {
    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("UTF-8 temp path");
    let case_root = root.join("p7-01/raw");
    fs::create_dir_all(&case_root).expect("create synthetic case");
    fs::write(
        case_root.join("root.jsonl"),
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-p7-01-root\",\"multi_agent_version\":\"v2\"}}\n",
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-p7-01-main\"}}\n",
            "{\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-p7-01-main\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"Synthetic prompt\"}],\"internal_chat_message_metadata_passthrough\":{\"turn_id\":\"turn-p7-01-main\"}}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"custom_tool_call\",\"name\":\"shell_command\",\"call_id\":\"call-p7-01-build\",\"input\":\"{}\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"custom_tool_call_output\",\"call_id\":\"call-p7-01-build\",\"output\":[{\"type\":\"input_text\",\"text\":\"ok\"}]}}\n",
        ),
    )
    .expect("write synthetic raw stream");

    let matrix = load_matrix().expect("load P7 matrix");
    let mut implemented = matrix.clone();
    for case in &mut implemented.cases {
        case.implemented = false;
    }
    implemented.cases[0].implemented = true;
    validate_implemented_case_provenance(root, &implemented)
        .expect("matching adapter and variants pass");

    let mut adapter_mismatch = implemented.clone();
    adapter_mismatch.cases[0].adapter_classes = vec!["Legacy".to_string()];
    let error = validate_implemented_case_provenance(root, &adapter_mismatch)
        .expect_err("adapter mismatch must fail");
    assert!(error.contains("P7-01"));
    assert!(error.contains("adapter provenance mismatch"));
    assert!(error.contains("owner=scoring,public-live"));

    let mut variant_mismatch = implemented;
    variant_mismatch.cases[0].event_variants.pop();
    let error = validate_implemented_case_provenance(root, &variant_mismatch)
        .expect_err("variant mismatch must fail");
    assert!(error.contains("P7-01"));
    assert!(error.contains("event variant provenance mismatch"));
    assert!(error.contains("owner=scoring,public-live"));
}

#[test]
fn p7_01_typed_semantic_goal_drift_reaches_public_live() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-01")
        .expect("P7-01 matrix entry");

    assert!(case.implemented, "P7-01 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-01 expected"),
    )
    .expect("parse P7-01 expected");
    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/29");
    fs::create_dir_all(&rollout_dir).expect("create P7-01 rollout directory");
    fs::copy(
        case_root.join("raw/root.jsonl"),
        rollout_dir.join("rollout-session-p7-01-root.jsonl"),
    )
    .expect("materialize P7-01 raw stream");

    let state_dir = temp_root.join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-p7-01-root".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create P7-01 live coordinator");
    let poll = coordinator.poll_once().expect("run P7-01 production path");
    assert!(poll.reran_pipeline);
    assert!(poll.emitted_checkpoints >= 3);
    assert!(!poll.observations.is_empty());
    assert!(poll.observations.iter().all(|observation| {
        observation.event.trigger == TriggerClass::CheckpointReady
            && observation.event.cursor.session_id == "session-p7-01-root"
    }));

    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(state_dir.join("compactor/manifest.json"))
            .expect("read P7-01 manifest"),
    )
    .expect("parse P7-01 manifest");
    assert_eq!(
        manifest["schema_version"],
        expected["bundle_schema_version"]
    );
    assert_eq!(
        manifest["session_ids"],
        serde_json::json!(["session-p7-01-root"])
    );
    assert_eq!(
        manifest["files"][0]["turns"],
        serde_json::json!(["turn-p7-01-one", "turn-p7-01-three", "turn-p7-01-two"])
    );

    let compact_rows = fs::read_to_string(state_dir.join("compactor/rows.compact.jsonl"))
        .expect("read P7-01 compact rows")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("parse P7-01 compact row"))
        .collect::<Vec<_>>();
    let first_call_outputs = compact_rows
        .iter()
        .filter(|row| row["kind"] == "tool_output")
        .filter_map(|row| {
            row["dedupe_identity"]
                .as_str()
                .and_then(|identity| serde_json::from_str::<Value>(identity).ok())
        })
        .filter(|identity| identity["call_id"] == "call-p7-01-one")
        .collect::<Vec<_>>();
    assert_eq!(first_call_outputs.len(), 2);
    assert_eq!(first_call_outputs[0]["segment_index"], 0);
    assert_eq!(first_call_outputs[1]["segment_index"], 1);
    assert!(first_call_outputs
        .iter()
        .all(|identity| identity["segment_type"] == "input_text"));

    let final_checkpoint = poll
        .observations
        .iter()
        .filter_map(|observation| observation.event.checkpoint.as_ref())
        .max_by_key(|checkpoint| checkpoint.ordinal)
        .expect("P7-01 final checkpoint");
    let semantic = final_checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::SemanticGoalDrift)
        .expect("P7-01 semantic-goal-drift score");
    assert_eq!(semantic.state, DriftState::Active);
    assert!(semantic.flagged);
    assert_eq!(
        semantic.raw_score,
        expected["semantic_goal_drift"]["raw_score"]
    );
    let target = final_checkpoint
        .structured_objective
        .as_ref()
        .and_then(|objective| objective.target.as_ref())
        .expect("P7-01 final structured target");
    assert_eq!(target.display, expected["final_target_display"]);
    for prefix in expected["required_reason_prefixes"]
        .as_array()
        .expect("P7-01 evidence prefixes")
    {
        let prefix = prefix.as_str().expect("P7-01 evidence prefix");
        assert!(
            semantic
                .evidence
                .iter()
                .any(|evidence| evidence.reason.starts_with(prefix)),
            "P7-01 missing semantic evidence prefix {prefix:?}"
        );
    }
    assert_eq!(
        serde_json::to_value(
            final_checkpoint
                .session_progress
                .as_ref()
                .expect("P7-01 progress")
        )
        .expect("serialize P7-01 progress"),
        expected["session_progress"],
        "P7-01 must retain the exact progress projection"
    );
    let final_observation = poll.observations.last().expect("P7-01 final observation");
    assert_eq!(
        final_observation.event.trigger,
        TriggerClass::CheckpointReady
    );
    assert_eq!(
        final_observation.snapshot.processed_events,
        poll.observations.len()
    );
}

#[test]
fn p7_02_positive_legacy_current_native_parity() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-02")
        .expect("P7-02 matrix entry");

    assert!(case.implemented, "P7-02 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-02 expected"),
    )
    .expect("parse P7-02 expected");
    let legacy =
        run_single_source_pipeline(&case_root.join("legacy/root.jsonl"), "session-p7-02-legacy");
    let current_native = run_single_source_pipeline(
        &case_root.join("current-native/root.jsonl"),
        "session-p7-02-native",
    );
    assert_eq!(legacy.format, RolloutFormat::Legacy);
    assert_eq!(current_native.format, RolloutFormat::CurrentNativeV2);
    assert_eq!(legacy.manifest["schema_version"], "v0.2");
    assert_eq!(current_native.manifest["schema_version"], "v0.2");

    let legacy_projection = canonical_semantic_projection(&legacy.result, "session-p7-02-legacy");
    let native_projection =
        canonical_semantic_projection(&current_native.result, "session-p7-02-native");
    assert_eq!(
        legacy_projection, native_projection,
        "P7-02 canonical semantic projection must ignore adapter-local and unstable manifest fields"
    );
    let final_projection = native_projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-02 final canonical checkpoint");
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    assert_eq!(
        final_projection["target_display"],
        expected["final_target_display"]
    );
}

struct PipelineRun {
    _temp_dir: tempfile::TempDir,
    format: RolloutFormat,
    manifest: Value,
    result: AnalyzeResult,
}

fn run_single_source_pipeline(source: &Utf8Path, session_id: &str) -> PipelineRun {
    let temp_dir = tempfile::TempDir::new().expect("pipeline temp dir");
    let root = Utf8Path::from_path(temp_dir.path()).expect("UTF-8 pipeline temp root");
    let codex_home = root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/29");
    fs::create_dir_all(&rollout_dir).expect("create pipeline rollout directory");
    fs::copy(
        source,
        rollout_dir.join(format!("rollout-{session_id}.jsonl")),
    )
    .expect("materialize pipeline source");

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&BoundedClosureRequest {
            codex_home: Some(codex_home),
            root_session_id: session_id.to_string(),
        })
        .expect("prepare single-source closure");
    let format = prepared.snapshot().root_source.format;
    let compactor_dir = root.join("compactor");
    compactor
        .compact(prepared, &compactor_dir, None)
        .expect("compact single-source closure");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(compactor_dir.join("manifest.json")).expect("read compactor manifest"),
    )
    .expect("parse compactor manifest");
    let analyzer_dir = root.join("analyzer");
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: compactor_dir,
        output_dir: analyzer_dir,
    })
    .expect("analyze single-source bundle");

    PipelineRun {
        _temp_dir: temp_dir,
        format,
        manifest,
        result,
    }
}

fn canonical_semantic_projection(result: &AnalyzeResult, session_id: &str) -> Value {
    let session = result
        .sessions
        .iter()
        .find(|session| session.session_id == session_id)
        .expect("canonical projection session");
    Value::Array(
        session
            .checkpoints
            .iter()
            .map(|checkpoint| {
                let objective = checkpoint.structured_objective.as_ref();
                let semantic = checkpoint
                    .drift_scores
                    .iter()
                    .find(|score| score.class == DriftClass::SemanticGoalDrift)
                    .expect("canonical semantic-goal-drift score");
                serde_json::json!({
                    "ordinal": checkpoint.ordinal,
                    "target_display": checkpoint
                        .structured_objective
                        .as_ref()
                        .and_then(|objective| objective.target.as_ref())
                        .map(|target| target.display.as_str()),
                    "working_set_paths": checkpoint.task_frame.working_set_paths,
                    "scorer_eligibility_witness": {
                        "objective_class": objective.map(|objective| objective.objective_class),
                        "objective_confidence": objective.map(|objective| objective.confidence),
                        "unknown_count": objective.map(|objective| objective.unknowns.len()),
                        "semantic_score_confidence": semantic.confidence,
                    },
                    "semantic_goal_drift": {
                        "state": semantic.state,
                        "flagged": semantic.flagged,
                        "raw_score": semantic.raw_score,
                    },
                    "semantic_reasons": semantic
                        .evidence
                        .iter()
                        .map(|evidence| evidence.reason.as_str())
                        .collect::<Vec<_>>(),
                    "session_progress": checkpoint.session_progress,
                })
            })
            .collect(),
    )
}

fn assert_scorer_eligible_projection(checkpoint: &Value, label: &str) {
    let witness = &checkpoint["scorer_eligibility_witness"];
    assert_eq!(
        witness["objective_class"], "task_statement",
        "{label} must expose an eligible task statement: {witness}"
    );
    assert!(
        matches!(
            witness["objective_confidence"].as_str(),
            Some("medium" | "high")
        ),
        "{label} must expose medium-or-higher objective confidence: {witness}"
    );
    assert_eq!(
        witness["unknown_count"], 0,
        "{label} must expose an objective without unknowns"
    );
    assert!(
        matches!(
            witness["semantic_score_confidence"].as_str(),
            Some("medium" | "high")
        ),
        "{label} must prove that semantic scoring passed its eligibility gate: {witness}"
    );
}

#[test]
fn p7_03_semantic_alignment_is_conservative() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-03")
        .expect("P7-03 matrix entry");

    assert!(case.implemented, "P7-03 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-03 expected"),
    )
    .expect("parse P7-03 expected");
    let run = run_single_source_pipeline(&case_root.join("raw/root.jsonl"), "session-p7-03-root");
    assert_eq!(run.format, RolloutFormat::CurrentNativeV2);
    let projection = canonical_semantic_projection(&run.result, "session-p7-03-root");
    for (index, checkpoint) in projection
        .as_array()
        .expect("P7-03 canonical checkpoints")
        .iter()
        .filter(|checkpoint| checkpoint["target_display"].is_string())
        .enumerate()
    {
        assert_scorer_eligible_projection(checkpoint, &format!("P7-03 checkpoint {}", index + 1));
    }
    let trajectory_targets = projection
        .as_array()
        .expect("P7-03 canonical checkpoints")
        .iter()
        .filter_map(|checkpoint| checkpoint["target_display"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        trajectory_targets,
        expected["trajectory_target_displays"]
            .as_array()
            .expect("P7-03 expected target trajectory")
            .iter()
            .map(|target| target.as_str().expect("P7-03 expected target"))
            .collect::<Vec<_>>(),
        "P7-03 must compare repeated, eligible aligned objectives"
    );
    let final_projection = projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-03 final canonical checkpoint");
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    assert_eq!(
        final_projection["target_display"],
        expected["final_target_display"]
    );
    assert_eq!(
        final_projection["semantic_reasons"], expected["semantic_reasons"],
        "P7-03 NoClaim must be an exact cleared score with no semantic evidence"
    );
    assert_eq!(expected["claim"], "NoClaim");
}

#[test]
fn p7_04_path_narrowing_preserves_progress() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-04")
        .expect("P7-04 matrix entry");

    assert!(case.implemented, "P7-04 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-04 expected"),
    )
    .expect("parse P7-04 expected");
    let run = run_single_source_pipeline(&case_root.join("raw/root.jsonl"), "session-p7-04-root");
    let projection = canonical_semantic_projection(&run.result, "session-p7-04-root");
    for (index, checkpoint) in projection
        .as_array()
        .expect("P7-04 canonical checkpoints")
        .iter()
        .filter(|checkpoint| checkpoint["target_display"].is_string())
        .enumerate()
    {
        assert_scorer_eligible_projection(checkpoint, &format!("P7-04 checkpoint {}", index + 1));
    }
    let trajectory_targets = projection
        .as_array()
        .expect("P7-04 canonical checkpoints")
        .iter()
        .filter_map(|checkpoint| checkpoint["target_display"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        trajectory_targets,
        expected["trajectory_target_displays"]
            .as_array()
            .expect("P7-04 expected target trajectory")
            .iter()
            .map(|target| target.as_str().expect("P7-04 expected target"))
            .collect::<Vec<_>>(),
        "P7-04 must exercise a real directory-to-contained-file trajectory"
    );
    assert_eq!(
        trajectory_targets.first().copied(),
        expected["initial_target_display"].as_str()
    );
    let initial_target = trajectory_targets.first().expect("P7-04 initial target");
    let narrowed_target = trajectory_targets.last().expect("P7-04 narrowed target");
    assert!(
        narrowed_target.starts_with(&format!("{initial_target}/")),
        "P7-04 final target must be a contained member of the initial target"
    );
    let final_projection = projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-04 final canonical checkpoint");
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    assert_eq!(
        final_projection["target_display"],
        expected["final_target_display"]
    );
    assert_eq!(
        final_projection["session_progress"],
        expected["session_progress"]
    );
    assert_eq!(expected["relation"], "narrowing");
    assert_eq!(expected["claim"], "NoClaim");
}

#[test]
fn p7_05_sanctioned_replan_is_suppressed_for_the_declared_reason() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-05")
        .expect("P7-05 matrix entry");

    assert!(case.implemented, "P7-05 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-05 expected"),
    )
    .expect("parse P7-05 expected");
    let run = run_single_source_pipeline(&case_root.join("raw/root.jsonl"), "session-p7-05-root");
    let projection = canonical_semantic_projection(&run.result, "session-p7-05-root");
    let trajectory_targets = projection
        .as_array()
        .expect("P7-05 canonical checkpoints")
        .iter()
        .filter_map(|checkpoint| checkpoint["target_display"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        trajectory_targets,
        expected["trajectory_target_displays"]
            .as_array()
            .expect("P7-05 expected target trajectory")
            .iter()
            .map(|target| target.as_str().expect("P7-05 expected target"))
            .collect::<Vec<_>>(),
        "P7-05 must contain an otherwise disjoint eligible goal pivot"
    );
    let final_projection = projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-05 final canonical checkpoint");
    assert_scorer_eligible_projection(final_projection, "P7-05 sanctioned checkpoint");
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    assert_eq!(
        final_projection["target_display"],
        expected["structured_target_display"]
    );
    let working_set_paths = final_projection["working_set_paths"]
        .as_array()
        .expect("P7-05 working-set paths");
    for path in expected["replan_working_set_paths"]
        .as_array()
        .expect("P7-05 replan working-set paths")
    {
        assert!(
            working_set_paths.contains(path),
            "the sanctioned replan path {path:?} must remain visible"
        );
    }
    let compactor_dir = Utf8Path::new(
        run.manifest["output_dir"]
            .as_str()
            .expect("P7-05 compactor output directory"),
    );
    let replan_row = fs::read_to_string(compactor_dir.join("rows.compact.jsonl"))
        .expect("read P7-05 compact rows")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("parse P7-05 compact row"))
        .find(|row| {
            row["kind"] == "user_message"
                && row["text"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("/goal Replan:"))
        })
        .expect("P7-05 explicit replan row");
    assert_eq!(replan_row["user_message_role"], "steer");
    assert_eq!(
        final_projection["semantic_reasons"],
        expected["semantic_reasons"]
    );

    let source = fs::read_to_string(case_root.join("raw/root.jsonl"))
        .expect("read P7-05 source for counterfactual");
    assert_eq!(
        source.matches("Replan:").count(),
        1,
        "P7-05 must have exactly one sanctioned-replan marker"
    );
    let counterfactual_dir = tempfile::TempDir::new().expect("P7-05 counterfactual temp dir");
    let counterfactual_path =
        Utf8Path::from_path(counterfactual_dir.path()).expect("P7-05 UTF-8 counterfactual path");
    let counterfactual_source = counterfactual_path.join("root.jsonl");
    fs::write(&counterfactual_source, source.replacen("Replan:", "", 1))
        .expect("write P7-05 counterfactual source");
    let counterfactual = run_single_source_pipeline(&counterfactual_source, "session-p7-05-root");
    let counterfactual_projection =
        canonical_semantic_projection(&counterfactual.result, "session-p7-05-root");
    let counterfactual_final = counterfactual_projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-05 counterfactual final checkpoint");
    assert_scorer_eligible_projection(counterfactual_final, "P7-05 counterfactual checkpoint");
    assert_eq!(
        final_projection["scorer_eligibility_witness"],
        counterfactual_final["scorer_eligibility_witness"],
        "removing only the sanction marker must retain the same eligible structured goal"
    );
    assert_eq!(
        counterfactual_final["target_display"], expected["structured_target_display"],
        "removing only the sanction marker must retain the same eligible pivot"
    );
    assert_eq!(
        counterfactual_final["semantic_goal_drift"], expected["counterfactual_without_sanction"],
        "removing only the sanction marker must restore the positive drift claim"
    );
    assert_eq!(expected["suppression"], "sanctioned_replan");
    assert_eq!(expected["claim"], "NoClaim");
}

#[test]
fn p7_06_zero_test_execution_cannot_claim_clean_verification() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-06")
        .expect("P7-06 matrix entry");

    assert!(case.implemented, "P7-06 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-06 expected"),
    )
    .expect("parse P7-06 expected");
    let run = run_single_source_pipeline(&case_root.join("raw/root.jsonl"), "session-p7-06-root");
    assert_eq!(run.format, RolloutFormat::CurrentNativeV2);

    let compactor_dir = Utf8Path::new(
        run.manifest["output_dir"]
            .as_str()
            .expect("P7-06 compactor output directory"),
    );
    let zero_test_row = fs::read_to_string(compactor_dir.join("rows.compact.jsonl"))
        .expect("read P7-06 compact rows")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("parse P7-06 compact row"))
        .find(|row| {
            row["kind"] == "tool_output"
                && row["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("running 0 tests"))
        })
        .expect("P7-06 zero-test tool output");
    let zero_test_identity: Value = serde_json::from_str(
        zero_test_row["dedupe_identity"]
            .as_str()
            .expect("P7-06 typed tool-output identity"),
    )
    .expect("parse P7-06 typed tool-output identity");
    assert_eq!(zero_test_identity["call_id"], "call-p7-06-zero");
    assert_eq!(zero_test_identity["segment_type"], "input_text");
    assert!(zero_test_row["text"]
        .as_str()
        .expect("P7-06 zero-test text")
        .contains(
            expected["zero_test_output"]
                .as_str()
                .expect("P7-06 expected zero-test output")
        ));

    let projection = canonical_semantic_projection(&run.result, "session-p7-06-root");
    let final_projection = projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-06 final canonical checkpoint");
    let progress = &final_projection["session_progress"];
    assert_eq!(progress, &expected["session_progress"]);
    for forbidden in expected["forbidden_signal_codes"]
        .as_array()
        .expect("P7-06 forbidden signal codes")
    {
        let forbidden = forbidden.as_str().expect("P7-06 signal code");
        assert!(
            !progress["signals"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|signal| signal["code"] == forbidden),
            "zero-test execution must not emit {forbidden}: {progress}"
        );
    }
    assert_eq!(expected["tests_executed"], 0);
    assert_eq!(expected["claim"], "NoClaim");
}

#[test]
fn p7_07_directive_path_cannot_establish_authority() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-07")
        .expect("P7-07 matrix entry");

    assert!(case.implemented, "P7-07 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-07 expected"),
    )
    .expect("parse P7-07 expected");
    let run = run_single_source_pipeline(&case_root.join("raw/root.jsonl"), "session-p7-07-root");
    assert_eq!(run.format, RolloutFormat::CurrentNativeV2);

    let session = run
        .result
        .sessions
        .iter()
        .find(|session| session.session_id == "session-p7-07-root")
        .expect("P7-07 analyzed session");
    let authoritative_paths = session
        .context
        .truth_artifacts
        .iter()
        .filter(|artifact| {
            matches!(
                artifact.source.as_str(),
                "objective_literal" | "directive_literal"
            )
        })
        .map(|artifact| artifact.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        authoritative_paths,
        expected["authoritative_paths"]
            .as_array()
            .expect("P7-07 authoritative paths")
            .iter()
            .map(|path| path.as_str().expect("P7-07 authoritative path"))
            .collect::<Vec<_>>()
    );
    assert!(session.context.truth_artifacts.iter().any(|artifact| {
        artifact.path == expected["directive_path"]
            && artifact.source == "control_directive_literal"
    }));

    let projection = canonical_semantic_projection(&run.result, "session-p7-07-root");
    let final_projection = projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-07 final canonical checkpoint");
    assert_eq!(
        final_projection["working_set_paths"],
        expected["working_set_paths"]
    );
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    let final_checkpoint = session
        .checkpoints
        .last()
        .expect("P7-07 final analyzer checkpoint");
    let wrong_plan = final_checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::WrongPlanBranch)
        .expect("P7-07 wrong-plan score");
    assert_eq!(
        serde_json::json!({
            "state": wrong_plan.state,
            "flagged": wrong_plan.flagged,
            "raw_score": wrong_plan.raw_score,
        }),
        expected["wrong_plan_branch"]
    );
    assert_eq!(expected["directive_authoritative"], false);
    assert_eq!(expected["claim"], "wrong_plan_branch");
}

#[test]
fn p7_08_lexical_collision_preserves_real_pivot() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-08")
        .expect("P7-08 matrix entry");

    assert!(case.implemented, "P7-08 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-08 expected"),
    )
    .expect("parse P7-08 expected");
    let run = run_single_source_pipeline(&case_root.join("raw/root.jsonl"), "session-p7-08-root");
    assert_eq!(run.format, RolloutFormat::CurrentNativeV2);

    let projection = canonical_semantic_projection(&run.result, "session-p7-08-root");
    let checkpoints = projection.as_array().expect("P7-08 canonical checkpoints");
    assert!(checkpoints
        .iter()
        .take(checkpoints.len() - 1)
        .any(|checkpoint| {
            checkpoint["target_display"] == expected["penultimate_target_display"]
                && checkpoint["semantic_goal_drift"]["flagged"] == false
        }));
    let final_projection = checkpoints
        .last()
        .expect("P7-08 final canonical checkpoint");
    assert_eq!(
        final_projection["target_display"],
        expected["final_target_display"]
    );
    assert_eq!(
        final_projection["working_set_paths"],
        expected["final_working_set_paths"]
    );
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    for prefix in expected["required_reason_prefixes"]
        .as_array()
        .expect("P7-08 evidence prefixes")
    {
        let prefix = prefix.as_str().expect("P7-08 evidence prefix");
        assert!(
            final_projection["semantic_reasons"]
                .as_array()
                .expect("P7-08 semantic reasons")
                .iter()
                .any(|reason| reason
                    .as_str()
                    .is_some_and(|reason| reason.starts_with(prefix))),
            "P7-08 missing semantic evidence prefix {prefix:?}"
        );
    }
    assert_eq!(expected["path_relation"], "unrelated");
}

#[test]
fn p7_09_unrelated_session_is_excluded_from_public_live() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-09")
        .expect("P7-09 matrix entry");

    assert!(case.implemented, "P7-09 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-09 expected"),
    )
    .expect("parse P7-09 expected");
    let run_live = |include_unrelated: bool| {
        let temp_dir = tempfile::TempDir::new().expect("P7-09 temp dir");
        let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7-09 UTF-8 temp root");
        let codex_home = temp_root.join(".codex");
        let rollout_dir = codex_home.join("sessions/2026/07/29");
        fs::create_dir_all(&rollout_dir).expect("create P7-09 rollout directory");
        fs::copy(
            case_root.join("raw/root.jsonl"),
            rollout_dir.join("rollout-session-p7-09-root.jsonl"),
        )
        .expect("materialize P7-09 selected source");
        if include_unrelated {
            fs::copy(
                case_root.join("raw/unrelated.jsonl"),
                rollout_dir.join("rollout-session-p7-09-unrelated.jsonl"),
            )
            .expect("materialize P7-09 unrelated source");
        }

        let state_dir = temp_root.join("state");
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home),
                session_id: "session-p7-09-root".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create P7-09 live coordinator");
        let poll = coordinator.poll_once().expect("run P7-09 public-live path");
        assert!(poll.reran_pipeline);
        let observation = poll.observations.last().expect("P7-09 public observation");
        let checkpoint = observation
            .event
            .checkpoint
            .as_ref()
            .expect("P7-09 public checkpoint");
        let semantic = checkpoint
            .drift_scores
            .iter()
            .find(|score| score.class == DriftClass::SemanticGoalDrift)
            .expect("P7-09 semantic score");
        assert_eq!(
            observation.event.trigger,
            TriggerClass::CheckpointReady,
            "P7-09 public observation must remain checkpoint-owned"
        );
        let public_projection = serde_json::json!({
            "trigger": "checkpoint_ready",
            "target_display": checkpoint
                .structured_objective
                .as_ref()
                .and_then(|objective| objective.target.as_ref())
                .map(|target| target.display.as_str()),
            "semantic_goal_drift": {
                "state": semantic.state,
                "flagged": semantic.flagged,
                "raw_score": semantic.raw_score,
            },
        });
        let manifest: Value = serde_json::from_str(
            &fs::read_to_string(state_dir.join("compactor/manifest.json"))
                .expect("read P7-09 manifest"),
        )
        .expect("parse P7-09 manifest");
        let compact_rows = fs::read_to_string(state_dir.join("compactor/rows.compact.jsonl"))
            .expect("read P7-09 compact rows");
        (manifest, public_projection, compact_rows)
    };

    let (selected_manifest, selected_projection, _) = run_live(false);
    let (isolated_manifest, isolated_projection, isolated_rows) = run_live(true);
    assert_eq!(
        selected_manifest["session_ids"],
        expected["selected_sessions"]
    );
    assert_eq!(
        isolated_manifest["session_ids"],
        expected["selected_sessions"]
    );
    assert_eq!(
        selected_projection, isolated_projection,
        "unrelated rollout evidence must not alter the public-live result"
    );
    assert_eq!(isolated_projection["trigger"], expected["public_trigger"]);
    assert_eq!(
        isolated_projection["target_display"],
        expected["final_target_display"]
    );
    assert_eq!(
        isolated_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    for excluded in expected["excluded_sessions"]
        .as_array()
        .expect("P7-09 excluded sessions")
    {
        assert!(
            !isolated_rows.contains(excluded.as_str().expect("P7-09 excluded session")),
            "P7-09 unrelated session residue reached compact rows"
        );
    }
}

#[test]
fn p7_10_typed_delegation_survives_production_bundle() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-10")
        .expect("P7-10 matrix entry");

    assert!(case.implemented, "P7-10 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-10 expected"),
    )
    .expect("parse P7-10 expected");
    let temp_dir = tempfile::TempDir::new().expect("P7-10 temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7-10 UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/29");
    fs::create_dir_all(&rollout_dir).expect("create P7-10 rollout directory");
    for session in ["root", "child", "grandchild"] {
        fs::copy(
            case_root.join(format!("raw/{session}.jsonl")),
            rollout_dir.join(format!("rollout-session-p7-10-{session}.jsonl")),
        )
        .expect("materialize P7-10 source");
    }

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&BoundedClosureRequest {
            codex_home: Some(codex_home),
            root_session_id: "session-p7-10-root".to_string(),
        })
        .expect("prepare P7-10 direct closure");
    let compactor_dir = temp_root.join("compactor");
    compactor
        .compact(prepared, &compactor_dir, None)
        .expect("compact P7-10 direct closure");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(compactor_dir.join("manifest.json")).expect("read P7-10 manifest"),
    )
    .expect("parse P7-10 manifest");
    assert_eq!(manifest["schema_version"], "v0.2");
    assert_eq!(manifest["session_ids"], expected["selected_sessions"]);
    let registry = manifest["files"]
        .as_array()
        .expect("P7-10 file registry")
        .iter()
        .map(|file| {
            (
                file["session_id"].as_str().expect("P7-10 registry session"),
                file["turns"].clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        registry,
        BTreeMap::from([
            (
                "session-p7-10-child",
                serde_json::json!(["turn-p7-10-child"]),
            ),
            ("session-p7-10-root", serde_json::json!(["turn-p7-10-root"]),),
        ])
    );
    let links = manifest["delegation_links"]
        .as_array()
        .expect("P7-10 delegation links");
    assert_eq!(links.len(), 2, "P7-10 links: {links:?}");
    let verified_link = links
        .iter()
        .find(|link| link["state"] == "verified")
        .expect("P7-10 verified direct link");
    for field in [
        "parent_session_id",
        "child_session_id",
        "child_origin_parent_session_id",
        "depth",
        "state",
    ] {
        assert_eq!(
            verified_link[field], expected["delegation_link"][field],
            "P7-10 delegation link field {field}"
        );
    }
    let deeper_residue = links
        .iter()
        .find(|link| link["state"] == "deeper_residue")
        .expect("P7-10 typed deeper residue");
    for field in [
        "parent_session_id",
        "child_session_id",
        "child_origin_parent_session_id",
        "depth",
        "state",
    ] {
        assert_eq!(
            deeper_residue[field], expected["deeper_residue"][field],
            "P7-10 deeper-residue field {field}"
        );
    }

    let compact_rows = fs::read_to_string(compactor_dir.join("rows.compact.jsonl"))
        .expect("read P7-10 compact rows")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("parse P7-10 compact row"))
        .collect::<Vec<_>>();
    let activity = compact_rows
        .iter()
        .find(|row| {
            row["kind"] == "status"
                && row["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("sub_agent_activity"))
        })
        .expect("P7-10 typed sub-agent activity");
    let activity_identity: Value = serde_json::from_str(
        activity["dedupe_identity"]
            .as_str()
            .expect("P7-10 activity identity"),
    )
    .expect("parse P7-10 activity identity");
    assert_eq!(activity_identity, expected["activity_identity"]);
    let agent_message = compact_rows
        .iter()
        .find(|row| row["text"] == "Delegated parser seam is bounded.")
        .expect("P7-10 typed agent message");
    let agent_message_identity: Value = serde_json::from_str(
        agent_message["dedupe_identity"]
            .as_str()
            .expect("P7-10 agent-message identity"),
    )
    .expect("parse P7-10 agent-message identity");
    assert_eq!(agent_message_identity, expected["agent_message_identity"]);

    let analyzer_dir = temp_root.join("analyzer");
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: compactor_dir,
        output_dir: analyzer_dir,
    })
    .expect("analyze P7-10 production-generated bundle");
    assert_eq!(
        result
            .sessions
            .iter()
            .map(|session| session.session_id.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["session-p7-10-child", "session-p7-10-root"])
    );
    for (session_id, expected_key) in [
        ("session-p7-10-root", "root_delegation"),
        ("session-p7-10-child", "child_delegation"),
    ] {
        let checkpoint = result
            .sessions
            .iter()
            .find(|session| session.session_id == session_id)
            .and_then(|session| session.checkpoints.last())
            .expect("P7-10 final delegated checkpoint");
        let projection = serde_json::json!({
            "topology": checkpoint.delegation.topology,
            "parent_session_id": checkpoint.delegation.parent_session_id,
            "child_session_ids": checkpoint.delegation.child_session_ids,
            "child_work_visibility": checkpoint.delegation.child_work_visibility,
            "confidence": checkpoint.delegation.confidence,
        });
        assert_eq!(
            projection, expected[expected_key],
            "P7-10 analyzer delegation projection for {session_id}"
        );
    }
    for excluded in expected["excluded_sessions"]
        .as_array()
        .expect("P7-10 excluded sessions")
    {
        let excluded = excluded.as_str().expect("P7-10 excluded session");
        assert!(
            !manifest["session_ids"]
                .as_array()
                .expect("P7-10 selected sessions")
                .iter()
                .any(|session| session == excluded),
            "P7-10 must not select a transitive delegation session"
        );
    }
}

#[test]
fn p7_11_verified_metadata_only_child_is_accepted() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-11")
        .expect("P7-11 matrix entry");

    assert!(case.implemented, "P7-11 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-11 expected"),
    )
    .expect("parse P7-11 expected");
    let temp_dir = tempfile::TempDir::new().expect("P7-11 temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7-11 UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/29");
    fs::create_dir_all(&rollout_dir).expect("create P7-11 rollout directory");
    for session in ["root", "child"] {
        fs::copy(
            case_root.join(format!("raw/{session}.jsonl")),
            rollout_dir.join(format!("rollout-session-p7-11-{session}.jsonl")),
        )
        .expect("materialize P7-11 source");
    }

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&BoundedClosureRequest {
            codex_home: Some(codex_home),
            root_session_id: "session-p7-11-root".to_string(),
        })
        .expect("prepare P7-11 direct closure");
    let compactor_dir = temp_root.join("compactor");
    compactor
        .compact(prepared, &compactor_dir, None)
        .expect("compact P7-11 direct closure");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(compactor_dir.join("manifest.json")).expect("read P7-11 manifest"),
    )
    .expect("parse P7-11 manifest");
    assert_eq!(manifest["session_ids"], expected["selected_sessions"]);
    let child_file = manifest["files"]
        .as_array()
        .expect("P7-11 file registry")
        .iter()
        .find(|file| file["session_id"] == expected["child_session_id"])
        .expect("P7-11 metadata-only child registry entry");
    assert_eq!(child_file["turns"], expected["child_file_turns"]);
    let child_file_id = child_file["id"].as_u64().expect("P7-11 child file ID");
    for artifact in ["rows.archival.jsonl", "rows.compact.jsonl"] {
        let child_rows = fs::read_to_string(compactor_dir.join(artifact))
            .expect("read P7-11 rows")
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("parse P7-11 row"))
            .filter(|row| row["source_file_id"] == child_file_id)
            .count();
        assert_eq!(
            child_rows,
            expected["child_rows"]
                .as_u64()
                .expect("P7-11 expected child rows") as usize
        );
    }
    let link = manifest["delegation_links"]
        .as_array()
        .and_then(|links| links.first())
        .expect("P7-11 verified delegation link");
    assert_eq!(link["state"], expected["link_state"]);
    assert_eq!(link["parent_session_id"], expected["parent_session_id"]);
    assert_eq!(link["child_session_id"], expected["child_session_id"]);

    let loaded = agent_drift_analyzer::input::load_bundle(&compactor_dir)
        .expect("P7-11 analyzer accepts metadata-only child");
    let child = loaded
        .sessions
        .iter()
        .find(|session| session.session_id == "session-p7-11-child")
        .expect("P7-11 analyzer child session");
    assert!(child.archival_rows.is_empty());
    assert!(child.compact_rows.is_empty());
    assert_eq!(
        loaded.delegation_graph.by_session_id["session-p7-11-root"].child_links[0].child_session_id,
        "session-p7-11-child"
    );
    assert_eq!(expected["accepted"], true);
}

#[test]
fn p7_12_missing_verified_child_is_rejected() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-12")
        .expect("P7-12 matrix entry");

    assert!(case.implemented, "P7-12 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-12 expected"),
    )
    .expect("parse P7-12 expected");
    let error = agent_drift_analyzer::input::load_bundle(&case_root.join("bundle"))
        .expect_err("P7-12 missing verified child must be rejected");
    match &error {
        agent_drift_analyzer::InputError::VerifiedDelegationSessionMissing {
            parent_session_id,
            child_session_id,
            missing_session_id,
        } => {
            assert_eq!(parent_session_id, &expected["parent_session_id"]);
            assert_eq!(child_session_id, &expected["child_session_id"]);
            assert_eq!(missing_session_id, &expected["missing_session_id"]);
        }
        other => panic!("P7-12 wrong analyzer input error: {other}"),
    }
    assert_eq!(error.to_string(), expected["error_message"]);
    assert_eq!(
        expected["error_category"],
        "verified_delegation_session_missing"
    );
    assert_eq!(expected["accepted"], false);
}

#[test]
fn p7_13_malformed_unrelated_source_is_excluded() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-13")
        .expect("P7-13 matrix entry");

    assert!(case.implemented, "P7-13 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-13 expected"),
    )
    .expect("parse P7-13 expected");
    let temp_dir = tempfile::TempDir::new().expect("P7-13 temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7-13 UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/29");
    fs::create_dir_all(&rollout_dir).expect("create P7-13 rollout directory");
    fs::copy(
        case_root.join("raw/root.jsonl"),
        rollout_dir.join("rollout-session-p7-13-root.jsonl"),
    )
    .expect("materialize P7-13 root source");
    fs::copy(
        case_root.join("raw/malformed-unrelated.jsonl"),
        rollout_dir.join("rollout-session-p7-13-unrelated.jsonl"),
    )
    .expect("materialize P7-13 malformed unrelated source");

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&BoundedClosureRequest {
            codex_home: Some(codex_home),
            root_session_id: "session-p7-13-root".to_string(),
        })
        .expect("P7-13 selection must ignore malformed unrelated body");
    let compactor_dir = temp_root.join("compactor");
    compactor
        .compact(prepared, &compactor_dir, None)
        .expect("compact P7-13 selected closure");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(compactor_dir.join("manifest.json")).expect("read P7-13 manifest"),
    )
    .expect("parse P7-13 manifest");
    assert_eq!(manifest["session_ids"], expected["selected_sessions"]);
    let compact_rows = fs::read_to_string(compactor_dir.join("rows.compact.jsonl"))
        .expect("read P7-13 compact rows");
    assert!(
        !compact_rows.contains(
            expected["malformed_session_id"]
                .as_str()
                .expect("P7-13 malformed session")
        ),
        "P7-13 unrelated malformed residue must not be decoded into selected rows"
    );

    let analyzer_dir = temp_root.join("analyzer");
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: compactor_dir,
        output_dir: analyzer_dir,
    })
    .expect("analyze P7-13 selected bundle");
    let projection = canonical_semantic_projection(&result, "session-p7-13-root");
    let final_projection = projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-13 final canonical checkpoint");
    assert_eq!(
        final_projection["target_display"],
        expected["final_target_display"]
    );
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    assert_eq!(expected["malformed_source_selected"], false);
    assert_eq!(expected["selected_pipeline"], "success");
}

#[test]
fn p7_14_malformed_selected_source_is_rejected() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-14")
        .expect("P7-14 matrix entry");

    assert!(case.implemented, "P7-14 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-14 expected"),
    )
    .expect("parse P7-14 expected");
    let temp_dir = tempfile::TempDir::new().expect("P7-14 temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7-14 UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/30");
    fs::create_dir_all(&rollout_dir).expect("create P7-14 rollout directory");
    let selected_path = rollout_dir.join("rollout-session-p7-14-root.jsonl");
    fs::copy(case_root.join("raw/root.jsonl"), &selected_path)
        .expect("materialize P7-14 selected source");

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&BoundedClosureRequest {
            codex_home: Some(codex_home),
            root_session_id: "session-p7-14-root".to_string(),
        })
        .expect("P7-14 envelope selection succeeds before full decode");
    let compactor_dir = temp_root.join("compactor");
    let error = compactor
        .compact(prepared, &compactor_dir, None)
        .expect_err("P7-14 selected malformed body must fail closed");
    match error {
        CompactorError::BoundedClosure(BoundedClosureError::SelectedPayloadMalformed {
            path,
            failures,
        }) => {
            assert_eq!(path, selected_path);
            assert_eq!(
                failures.len(),
                expected["failure_count"]
                    .as_u64()
                    .expect("P7-14 failure count") as usize
            );
            assert_eq!(failures[0], expected["failure"]);
        }
        other => panic!("P7-14 wrong closure error: {other}"),
    }
    assert!(
        !compactor_dir.exists(),
        "P7-14 must not publish an analyzer bundle"
    );
    assert_eq!(expected["error_category"], "selected_payload_malformed");
    assert_eq!(expected["selected_pipeline"], "rejected");
    assert_eq!(expected["analyzer_started"], false);
}

#[test]
fn p7_15_incomplete_bundle_v0_2_is_rejected() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-15")
        .expect("P7-15 matrix entry");

    assert!(case.implemented, "P7-15 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-15 expected"),
    )
    .expect("parse P7-15 expected");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("bundle/manifest.json")).expect("read P7-15 manifest"),
    )
    .expect("parse P7-15 manifest");
    assert_eq!(manifest["schema_version"], expected["bundle_schema"]);

    let error = agent_drift_analyzer::input::load_bundle(&case_root.join("bundle"))
        .expect_err("P7-15 incomplete bundle-v0.2 must be rejected");
    match &error {
        agent_drift_analyzer::InputError::MissingArtifact { path } => {
            assert_eq!(
                path.file_name(),
                Some(
                    expected["missing_artifact"]
                        .as_str()
                        .expect("P7-15 missing artifact")
                )
            );
        }
        other => panic!("P7-15 wrong analyzer input error: {other}"),
    }
    assert_eq!(expected["error_category"], "missing_artifact");
    assert_eq!(expected["accepted"], false);
}

#[test]
fn p7_16_invalid_public_live_event_does_not_mutate_runtime() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-16")
        .expect("P7-16 matrix entry");

    assert!(case.implemented, "P7-16 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let fixture: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("live-event.json")).expect("read P7-16 live event"),
    )
    .expect("parse P7-16 live event");
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-16 expected"),
    )
    .expect("parse P7-16 expected");
    assert_eq!(fixture["trigger"], "checkpoint_ready");
    assert!(fixture["checkpoint"].is_null());

    let event = LiveCheckpointEvent {
        emission_ordinal: fixture["emission_ordinal"]
            .as_u64()
            .expect("P7-16 emission ordinal") as usize,
        cursor: CheckpointCursor {
            session_id: fixture["cursor"]["session_id"]
                .as_str()
                .expect("P7-16 cursor session")
                .to_string(),
            ordinal: fixture["cursor"]["ordinal"]
                .as_u64()
                .expect("P7-16 cursor ordinal") as usize,
        },
        trigger: TriggerClass::CheckpointReady,
        checkpoint: None,
        source_label: fixture["source_label"].as_str().map(str::to_string),
    };
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let before = runtime.snapshot();
    assert_eq!(before.processed_events, expected["processed_events_before"]);
    let error = runtime
        .observe(event)
        .expect_err("P7-16 missing checkpoint payload must be rejected");
    assert!(matches!(
        error,
        LiveRuntimeError::MissingCheckpointPayload {
            emission_ordinal: 1
        }
    ));
    let after = runtime.snapshot();
    assert_eq!(after, before);
    assert_eq!(after.processed_events, expected["processed_events_after"]);
    assert_eq!(expected["error_category"], "missing_checkpoint_payload");
    assert_eq!(expected["state_mutated"], false);
    assert_eq!(expected["accepted"], false);
}

#[test]
fn p7_17_aligned_legacy_current_native_parity() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-17")
        .expect("P7-17 matrix entry");

    assert!(case.implemented, "P7-17 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-17 expected"),
    )
    .expect("parse P7-17 expected");
    let legacy =
        run_single_source_pipeline(&case_root.join("legacy/root.jsonl"), "session-p7-17-legacy");
    let current_native = run_single_source_pipeline(
        &case_root.join("current-native/root.jsonl"),
        "session-p7-17-native",
    );
    assert_eq!(legacy.format, RolloutFormat::Legacy);
    assert_eq!(current_native.format, RolloutFormat::CurrentNativeV2);

    let legacy_projection = canonical_semantic_projection(&legacy.result, "session-p7-17-legacy");
    let current_native_projection =
        canonical_semantic_projection(&current_native.result, "session-p7-17-native");
    assert_eq!(
        legacy_projection, current_native_projection,
        "P7-17 aligned canonical projections must match exactly"
    );
    let final_projection = current_native_projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-17 final canonical checkpoint");
    assert_eq!(
        final_projection["target_display"],
        expected["final_target_display"]
    );
    assert_eq!(
        final_projection["semantic_goal_drift"],
        expected["semantic_goal_drift"]
    );
    assert_eq!(expected["canonical_projection"], "equal");
    assert_eq!(expected["claim"], "NoClaim");
}

#[test]
fn p7_18_zero_test_legacy_current_native_parity() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-18")
        .expect("P7-18 matrix entry");

    assert!(case.implemented, "P7-18 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-18 expected"),
    )
    .expect("parse P7-18 expected");
    let legacy =
        run_single_source_pipeline(&case_root.join("legacy/root.jsonl"), "session-p7-18-legacy");
    let current_native = run_single_source_pipeline(
        &case_root.join("current-native/root.jsonl"),
        "session-p7-18-native",
    );
    assert_eq!(legacy.format, RolloutFormat::Legacy);
    assert_eq!(current_native.format, RolloutFormat::CurrentNativeV2);

    let legacy_projection = canonical_semantic_projection(&legacy.result, "session-p7-18-legacy");
    let current_native_projection =
        canonical_semantic_projection(&current_native.result, "session-p7-18-native");
    assert_eq!(
        legacy_projection, current_native_projection,
        "P7-18 zero-test canonical projections must match exactly"
    );
    let final_projection = current_native_projection
        .as_array()
        .and_then(|checkpoints| checkpoints.last())
        .expect("P7-18 final canonical checkpoint");
    assert_eq!(
        final_projection["session_progress"],
        expected["session_progress"]
    );
    for forbidden in expected["forbidden_signal_codes"]
        .as_array()
        .expect("P7-18 forbidden signal codes")
    {
        let forbidden = forbidden.as_str().expect("P7-18 signal code");
        assert!(
            !final_projection["session_progress"]["signals"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|signal| signal["code"] == forbidden),
            "P7-18 zero-test parity must not emit {forbidden}"
        );
    }
    assert_eq!(expected["canonical_projection"], "equal");
    assert_eq!(expected["claim"], "NoClaim");
}

fn canonical_typed_output_text(text: &str) -> String {
    fn sort_json(value: Value) -> Value {
        match value {
            Value::Array(values) => Value::Array(values.into_iter().map(sort_json).collect()),
            Value::Object(values) => Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, sort_json(value)))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            other => other,
        }
    }

    serde_json::from_str(text)
        .map(sort_json)
        .and_then(|value| serde_json::to_string(&value))
        .unwrap_or_else(|_| text.to_string())
}

fn run_p7_19_variant(case_root: &Utf8Path, reverse_creation: bool, warm_cache: bool) -> Value {
    let temp_dir = tempfile::TempDir::new().expect("P7-19 temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7-19 UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/07/30");
    fs::create_dir_all(&rollout_dir).expect("create P7-19 rollout directory");
    let mut sources = [
        ("root", "rollout-session-p7-19-root.jsonl", "raw/root.jsonl"),
        (
            "unrelated",
            "rollout-session-p7-19-unrelated.jsonl",
            "raw/unrelated.jsonl",
        ),
    ];
    if reverse_creation {
        sources.reverse();
    }
    for (label, destination, source) in sources {
        fs::copy(case_root.join(source), rollout_dir.join(destination))
            .unwrap_or_else(|error| panic!("materialize P7-19 {label} source: {error}"));
    }

    let mut compactor = BoundedClosureCompactor::default();
    if warm_cache {
        let prepared = compactor
            .prepare(&BoundedClosureRequest {
                codex_home: Some(codex_home.clone()),
                root_session_id: "session-p7-19-root".to_string(),
            })
            .expect("prepare P7-19 warm-up closure");
        compactor
            .compact(prepared, &temp_root.join("warm-up"), None)
            .expect("compact P7-19 warm-up closure");
    }
    let prepared = compactor
        .prepare(&BoundedClosureRequest {
            codex_home: Some(codex_home),
            root_session_id: "session-p7-19-root".to_string(),
        })
        .expect("prepare P7-19 measured closure");
    let compactor_dir = temp_root.join("compactor");
    compactor
        .compact(prepared, &compactor_dir, None)
        .expect("compact P7-19 measured closure");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(compactor_dir.join("manifest.json")).expect("read P7-19 manifest"),
    )
    .expect("parse P7-19 manifest");
    let typed_output_rows = fs::read_to_string(compactor_dir.join("rows.compact.jsonl"))
        .expect("read P7-19 compact rows")
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("parse P7-19 compact row"))
        .filter(|row| {
            row["kind"] == "tool_output"
                && row["dedupe_identity"].as_str().is_some_and(|identity| {
                    serde_json::from_str::<Value>(identity)
                        .is_ok_and(|identity| identity["call_id"] == "call-p7-19-typed")
                })
        })
        .map(|row| {
            serde_json::json!({
                "source_file_id": row["source_file_id"],
                "turn_id_ref": row["turn_id_ref"],
                "event_index": row["event_index"],
                "row_ordinal": row["row_ordinal"],
                "dedupe_identity": serde_json::from_str::<Value>(
                    row["dedupe_identity"].as_str().expect("P7-19 typed identity")
                )
                .expect("parse P7-19 typed identity"),
                "text": canonical_typed_output_text(
                    row["text"].as_str().expect("P7-19 typed text")
                ),
            })
        })
        .collect::<Vec<_>>();

    let analyzer_dir = temp_root.join("analyzer");
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: compactor_dir,
        output_dir: analyzer_dir,
    })
    .expect("analyze P7-19 bundle");
    serde_json::json!({
        "schema_version": manifest["schema_version"],
        "session_ids": manifest["session_ids"],
        "files": manifest["files"]
            .as_array()
            .expect("P7-19 file registry")
            .iter()
            .map(|file| serde_json::json!({
                "id": file["id"],
                "session_id": file["session_id"],
                "turns": file["turns"],
            }))
            .collect::<Vec<_>>(),
        "typed_output_rows": typed_output_rows,
        "semantic_projection": canonical_semantic_projection(&result, "session-p7-19-root"),
    })
}

#[test]
fn p7_19_typed_output_is_deterministic_across_varied_runs() {
    let matrix = load_matrix().expect("load P7 matrix");
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == "P7-19")
        .expect("P7-19 matrix entry");

    assert!(case.implemented, "P7-19 must be implemented before it runs");

    let case_root = fixture_root().join(&case.fixture_dir);
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(case_root.join("expected.json")).expect("read P7-19 expected"),
    )
    .expect("parse P7-19 expected");
    let cold_forward = run_p7_19_variant(&case_root, false, false);
    let cold_reverse = run_p7_19_variant(&case_root, true, false);
    let warm_reverse = run_p7_19_variant(&case_root, true, true);
    assert_eq!(cold_forward, cold_reverse);
    assert_eq!(cold_forward, warm_reverse);
    assert_eq!(cold_forward["session_ids"], expected["selected_sessions"]);

    let typed_rows = cold_forward["typed_output_rows"]
        .as_array()
        .expect("P7-19 typed output rows");
    assert_eq!(
        typed_rows
            .iter()
            .map(|row| row["dedupe_identity"]["segment_type"].clone())
            .collect::<Vec<_>>(),
        expected["typed_segment_types"]
            .as_array()
            .expect("P7-19 segment types")
            .clone()
    );
    assert_eq!(
        typed_rows
            .iter()
            .map(|row| row["dedupe_identity"]["segment_index"].clone())
            .collect::<Vec<_>>(),
        expected["typed_segment_order"]
            .as_array()
            .expect("P7-19 segment order")
            .clone()
    );
    assert_eq!(
        typed_rows
            .iter()
            .map(|row| row["text"].clone())
            .collect::<Vec<_>>(),
        expected["typed_segment_text"]
            .as_array()
            .expect("P7-19 segment text")
            .clone()
    );
    assert_eq!(expected["varied_runs_equal"], true);
}

#[test]
fn current_native_recall_whole_wall_is_complete_bounded_and_deterministic() {
    let matrix = load_matrix().expect("load P7 matrix");
    validate_matrix(&matrix).expect("validate P7 matrix");
    let directories = tracked_case_directories().expect("list P7 fixture directories");
    validate_case_inventory(&matrix, &directories).expect("validate P7 fixture inventory");

    assert_eq!(matrix.cases.len(), PLANNED_CASE_IDS.len());
    assert!(
        (15..=25).contains(&matrix.cases.len()),
        "P7 case count must remain inside the approved 15-25 envelope"
    );
    assert!(
        matrix.cases.len() <= 22,
        "P7 case counts above the soft cap of 22 require a written distinct-contract explanation"
    );
    assert!(
        matrix.cases.iter().all(|case| case.implemented),
        "all P7 cases must be implemented before the whole wall runs"
    );

    let covered_contracts = matrix
        .cases
        .iter()
        .flat_map(|case| case.contract_families.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let required_contracts = BTreeSet::from([
        "native_identity",
        "typed_shape",
        "semantic_positive",
        "semantic_conservative",
        "verification_execution",
        "path_authority",
        "lexical_path_identity",
        "session_isolation",
        "delegation",
        "closure_malformed",
        "analyzer_malformed",
        "public_live",
        "compatibility",
    ]);
    assert_eq!(
        required_contracts
            .difference(&covered_contracts)
            .copied()
            .collect::<Vec<_>>(),
        Vec::<&str>::new(),
        "P7 matrix is missing required contract families"
    );

    validate_historical_references(&matrix).expect("validate P7 historical references");

    let cold_forward = run_canonical_recall_wall(&matrix, false, false);
    let warm_reverse = run_canonical_recall_wall(&matrix, true, true);
    let cold_bytes = serde_json::to_vec(&cold_forward).expect("serialize cold P7 wall");
    let warm_bytes = serde_json::to_vec(&warm_reverse).expect("serialize warm P7 wall");
    assert_eq!(
        cold_bytes, warm_bytes,
        "P7 canonical wall must be byte-identical across temporary roots, source order, and cache state"
    );
    assert_eq!(
        cold_forward
            .as_array()
            .expect("P7 canonical wall cases")
            .len(),
        PLANNED_CASE_IDS.len()
    );
    for case_id in ["P7-01", "P7-09"] {
        let case = cold_forward
            .as_array()
            .expect("P7 canonical wall cases")
            .iter()
            .find(|case| case["case_id"] == case_id)
            .unwrap_or_else(|| panic!("{case_id} canonical wall entry"));
        let roots = case["projection"]
            .as_array()
            .unwrap_or_else(|| panic!("{case_id} canonical root projections"));
        assert!(
            roots
                .iter()
                .all(|root| root["public_live"]["reran_pipeline"] == true),
            "{case_id} must execute the measured public-live coordinator"
        );
        assert!(
            roots.iter().all(|root| {
                root["public_live"]["observations"]
                    .as_array()
                    .is_some_and(|observations| {
                        !observations.is_empty()
                            && observations
                                .iter()
                                .all(|observation| observation["trigger"] == "checkpoint_ready")
                    })
            }),
            "{case_id} must project public checkpoint-ready observations"
        );
    }
}

#[derive(Debug)]
struct WallSource {
    path: Utf8PathBuf,
    session_id: String,
    is_child: bool,
}

fn run_canonical_recall_wall(
    matrix: &RecallMatrix,
    reverse_sources: bool,
    warm_closure: bool,
) -> Value {
    let mut cases = matrix.cases.iter().collect::<Vec<_>>();
    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Value::Array(
        cases
            .into_iter()
            .map(|case| {
                let projection = if case
                    .adapter_classes
                    .iter()
                    .any(|adapter| adapter == "BundleV0_2")
                {
                    run_canonical_bundle_case(case)
                } else if case
                    .adapter_classes
                    .iter()
                    .any(|adapter| adapter == "PublicLive")
                {
                    run_canonical_public_live_case(case)
                } else {
                    run_canonical_raw_case(case, reverse_sources, warm_closure)
                };
                serde_json::json!({
                    "case_id": case.case_id,
                    "terminal_boundary": case.terminal_boundary,
                    "projection": projection,
                })
            })
            .collect(),
    )
}

fn run_canonical_raw_case(case: &RecallCase, reverse_sources: bool, warm_closure: bool) -> Value {
    let case_root = fixture_root().join(&case.fixture_dir);
    let mut sources = case
        .logical_source_files
        .iter()
        .map(|logical| wall_source(&case_root.join(logical)))
        .collect::<Vec<_>>();
    if reverse_sources {
        sources.reverse();
    }

    let temp_dir = tempfile::TempDir::new().expect("P7 whole-wall temp dir");
    let temp_root = Utf8Path::from_path(temp_dir.path()).expect("P7 whole-wall UTF-8 temp root");
    let codex_home = temp_root.join(".codex");
    let rollout_dir = codex_home.join("sessions/2030/01/01");
    fs::create_dir_all(&rollout_dir).expect("create P7 whole-wall rollout directory");
    for source in &sources {
        fs::copy(
            &source.path,
            rollout_dir.join(format!("rollout-{}.jsonl", source.session_id)),
        )
        .unwrap_or_else(|error| {
            panic!(
                "{} failed to materialize {}: {error}",
                case.case_id, source.session_id
            )
        });
    }

    let source_ids = sources
        .iter()
        .map(|source| source.session_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut roots = case
        .selected_direct_closure
        .iter()
        .filter(|session_id| {
            source_ids.contains(session_id.as_str())
                && sources
                    .iter()
                    .any(|source| source.session_id == **session_id && !source.is_child)
        })
        .cloned()
        .collect::<Vec<_>>();
    roots.sort();
    assert!(
        !roots.is_empty(),
        "{} has no materialized root session",
        case.case_id
    );

    Value::Array(
        roots
            .into_iter()
            .map(|root_session_id| {
                run_canonical_root(
                    case,
                    &codex_home,
                    temp_root,
                    &root_session_id,
                    warm_closure,
                    case.terminal_boundary == "public-live checkpoint",
                )
            })
            .collect(),
    )
}

fn wall_source(path: &Utf8Path) -> WallSource {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read P7 whole-wall source {path}: {error}"));
    let session_meta = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<Value>(line)
                .unwrap_or_else(|error| panic!("parse P7 whole-wall source {path}: {error}"))
        })
        .find(|row| row["type"] == "session_meta")
        .unwrap_or_else(|| panic!("P7 whole-wall source {path} has no session_meta"));
    let session_id = session_meta["payload"]["session_id"]
        .as_str()
        .unwrap_or_else(|| panic!("P7 whole-wall source {path} has no session identity"))
        .to_string();
    let is_child = session_meta
        .pointer("/payload/source/subagent/thread_spawn/parent_thread_id")
        .is_some_and(Value::is_string);
    WallSource {
        path: path.to_path_buf(),
        session_id,
        is_child,
    }
}

fn run_canonical_root(
    case: &RecallCase,
    codex_home: &Utf8Path,
    temp_root: &Utf8Path,
    root_session_id: &str,
    warm_closure: bool,
    public_live: bool,
) -> Value {
    let request = BoundedClosureRequest {
        codex_home: Some(codex_home.to_path_buf()),
        root_session_id: root_session_id.to_string(),
    };
    let mut compactor = BoundedClosureCompactor::default();
    if warm_closure {
        let prepared = compactor.prepare(&request).unwrap_or_else(|error| {
            panic!(
                "{} failed to warm closure for {root_session_id}: {error}",
                case.case_id
            )
        });
        if case.case_id != "P7-14" {
            compactor
                .compact(
                    prepared,
                    &temp_root.join(format!("warm-{root_session_id}")),
                    None,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "{} failed to compact warm closure for {root_session_id}: {error}",
                        case.case_id
                    )
                });
        }
    }

    if public_live {
        return canonical_public_live_root(case, codex_home, temp_root, root_session_id);
    }

    let prepared = compactor.prepare(&request).unwrap_or_else(|error| {
        panic!(
            "{} failed to prepare measured closure for {root_session_id}: {error}",
            case.case_id
        )
    });
    let output_dir = temp_root.join(format!("measured-{root_session_id}"));
    match compactor.compact(prepared, &output_dir, None) {
        Ok(_) => canonical_successful_root(root_session_id, &output_dir, temp_root),
        Err(CompactorError::BoundedClosure(BoundedClosureError::SelectedPayloadMalformed {
            failures,
            ..
        })) => serde_json::json!({
            "root_session_id": root_session_id,
            "status": "error",
            "error": "selected_payload_malformed",
            "failures": failures,
        }),
        Err(other) => panic!(
            "{} unexpected closure failure for {root_session_id}: {other}",
            case.case_id
        ),
    }
}

fn canonical_public_live_root(
    case: &RecallCase,
    codex_home: &Utf8Path,
    temp_root: &Utf8Path,
    root_session_id: &str,
) -> Value {
    let state_dir = temp_root.join(format!("live-{root_session_id}"));
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.to_path_buf()),
            session_id: root_session_id.to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .unwrap_or_else(|error| {
        panic!(
            "{} failed to create live coordinator for {root_session_id}: {error}",
            case.case_id
        )
    });
    let poll = coordinator.poll_once().unwrap_or_else(|error| {
        panic!(
            "{} failed public-live poll for {root_session_id}: {error}",
            case.case_id
        )
    });
    assert!(
        poll.reran_pipeline,
        "{} public-live measured poll did not run",
        case.case_id
    );
    assert_eq!(
        poll.emitted_checkpoints,
        poll.observations.len(),
        "{} public-live emission count mismatch",
        case.case_id
    );

    let compactor_dir = state_dir.join("compactor");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(compactor_dir.join("manifest.json"))
            .expect("read P7 public-live whole-wall manifest"),
    )
    .expect("parse P7 public-live whole-wall manifest");
    let bundle = canonical_bundle_projection(&manifest, &compactor_dir);
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: compactor_dir,
        output_dir: temp_root.join(format!("live-analyzer-{root_session_id}")),
    })
    .expect("analyze P7 public-live whole-wall bundle");
    let observations = poll
        .observations
        .iter()
        .map(|observation| {
            serde_json::json!({
                "emission_ordinal": observation.event.emission_ordinal,
                "cursor": {
                    "session_id": observation.event.cursor.session_id,
                    "ordinal": observation.event.cursor.ordinal,
                },
                "trigger": trigger_label(observation.event.trigger),
                "checkpoint": observation.event.checkpoint.as_ref().map(canonical_checkpoint_projection),
                "decision": {
                    "evaluate": observation.decision.evaluate,
                    "visible_warning_allowed": observation.decision.visible_warning_allowed,
                    "reason": format!("{:?}", observation.decision.reason),
                },
                "runtime_snapshot": canonical_runtime_snapshot(&observation.snapshot),
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "root_session_id": root_session_id,
        "status": "success",
        "bundle": bundle,
        "analyzer": canonical_analyzer_projection(&result),
        "public_live": {
            "reran_pipeline": poll.reran_pipeline,
            "emitted_checkpoints": poll.emitted_checkpoints,
            "latest_cursor": poll.latest_cursor.map(|cursor| serde_json::json!({
                "session_id": cursor.session_id,
                "ordinal": cursor.ordinal,
            })),
            "observations": observations,
            "runtime_snapshot": canonical_runtime_snapshot(&coordinator.runtime_snapshot()),
        },
    })
}

fn canonical_successful_root(
    root_session_id: &str,
    output_dir: &Utf8Path,
    temp_root: &Utf8Path,
) -> Value {
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(output_dir.join("manifest.json")).expect("read P7 whole-wall manifest"),
    )
    .expect("parse P7 whole-wall manifest");
    let bundle = canonical_bundle_projection(&manifest, output_dir);
    let analyzer_dir = temp_root.join(format!("analyzer-{root_session_id}"));
    let result = analyze_bundle(&AnalyzeRequest {
        input_dir: output_dir.to_path_buf(),
        output_dir: analyzer_dir,
    })
    .expect("analyze P7 whole-wall bundle");
    serde_json::json!({
        "root_session_id": root_session_id,
        "status": "success",
        "bundle": bundle,
        "analyzer": canonical_analyzer_projection(&result),
    })
}

fn canonical_bundle_projection(manifest: &Value, output_dir: &Utf8Path) -> Value {
    let mut files = manifest["files"]
        .as_array()
        .expect("P7 whole-wall file registry")
        .iter()
        .map(|file| {
            serde_json::json!({
                "session_id": file["session_id"],
                "turns": file["turns"],
            })
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|file| serde_json::to_vec(file).expect("serialize P7 file projection"));

    let registry = manifest["files"]
        .as_array()
        .expect("P7 whole-wall file registry")
        .iter()
        .map(|file| {
            (
                file["id"].as_u64().expect("P7 whole-wall source file ID"),
                (
                    file["session_id"].clone(),
                    file["turns"].as_array().cloned().unwrap_or_default(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let compact_rows = fs::read_to_string(output_dir.join("rows.compact.jsonl"))
        .expect("read P7 whole-wall compact rows")
        .lines()
        .map(|line| {
            let row: Value = serde_json::from_str(line).expect("parse P7 whole-wall compact row");
            let source_file_id = row["source_file_id"]
                .as_u64()
                .expect("P7 whole-wall row source file ID");
            let (session_id, turns) = registry
                .get(&source_file_id)
                .expect("P7 whole-wall row registry entry");
            let turn_id = row["turn_id_ref"]
                .as_u64()
                .and_then(|turn_ref| turns.get(turn_ref as usize))
                .cloned()
                .unwrap_or(Value::Null);
            serde_json::json!({
                "source_session_id": session_id,
                "source_kind": row["source_kind"],
                "turn_id": turn_id,
                "event_index": row["event_index"],
                "row_ordinal": row["row_ordinal"],
                "kind": row["kind"],
                "user_message_role": row["user_message_role"],
                "dedupe_identity": row["dedupe_identity"],
                "text": row["text"],
                "text_hash_hex": row["text_hash_hex"],
            })
        })
        .collect::<Vec<_>>();

    let mut delegation_links = manifest["delegation_links"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|link| {
            serde_json::json!({
                "parent_session_id": link["parent_session_id"],
                "child_session_id": link["child_session_id"],
                "child_origin_parent_session_id": link["child_origin_parent_session_id"],
                "depth": link["depth"],
                "state": link["state"],
                "parent_evidence": canonical_link_evidence(&link["parent_evidence"]),
                "child_evidence": canonical_link_evidence(&link["child_evidence"]),
            })
        })
        .collect::<Vec<_>>();
    delegation_links
        .sort_by_key(|link| serde_json::to_vec(link).expect("serialize P7 link projection"));

    serde_json::json!({
        "schema_version": manifest["schema_version"],
        "discovered_file_count": manifest["discovered_file_count"],
        "archival_row_count": manifest["archival_row_count"],
        "compact_row_count": manifest["compact_row_count"],
        "dedupe_group_count": manifest["dedupe_group_count"],
        "session_ids": manifest["session_ids"],
        "files": files,
        "delegation_links": delegation_links,
        "compact_rows": compact_rows,
    })
}

fn canonical_link_evidence(evidence: &Value) -> Vec<Value> {
    evidence
        .as_array()
        .into_iter()
        .flatten()
        .map(|entry| {
            serde_json::json!({
                "line_number": entry["line_number"],
                "event_index": entry["event_index"],
            })
        })
        .collect()
}

fn canonical_analyzer_projection(result: &AnalyzeResult) -> Value {
    let mut sessions = result.sessions.iter().collect::<Vec<_>>();
    sessions.sort_by(|left, right| left.session_id.cmp(&right.session_id));
    Value::Array(
        sessions
            .into_iter()
            .map(|session| {
                serde_json::json!({
                    "session_id": session.session_id,
                    "checkpoints": session.checkpoints
                        .iter()
                        .map(canonical_checkpoint_projection)
                        .collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}

fn canonical_checkpoint_projection(checkpoint: &Checkpoint) -> Value {
    serde_json::json!({
        "ordinal": checkpoint.ordinal,
        "target_display": checkpoint.structured_objective
            .as_ref()
            .and_then(|objective| objective.target.as_ref())
            .map(|target| target.display.as_str()),
        "objective": checkpoint.task_frame.objective,
        "working_set_paths": checkpoint.task_frame.working_set_paths,
        "verification_commands": checkpoint.task_frame.verification_commands,
        "delegation": {
            "topology": checkpoint.delegation.topology,
            "parent_session_id": checkpoint.delegation.parent_session_id,
            "child_session_ids": checkpoint.delegation.child_session_ids,
            "child_work_visibility": checkpoint.delegation.child_work_visibility,
            "confidence": checkpoint.delegation.confidence,
            "markers": checkpoint.delegation.markers,
        },
        "drift_scores": checkpoint.drift_scores.iter().map(|score| {
            serde_json::json!({
                "class": score.class,
                "state": score.state,
                "raw_score": score.raw_score,
                "confidence": score.confidence,
                "flagged": score.flagged,
                "reasons": score.evidence.iter()
                    .map(|evidence| evidence.reason.as_str())
                    .collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "session_progress": checkpoint.session_progress.as_ref().map(|progress| {
            serde_json::json!({
                "status": progress.status,
                "dimension": progress.dimension,
                "confidence": progress.confidence,
                "signals": progress.signals.iter().map(|signal| {
                    serde_json::json!({
                        "code": signal.code,
                        "polarity": signal.polarity,
                        "strength": signal.strength,
                        "summary": signal.summary,
                        "before": signal.before,
                        "after": signal.after,
                    })
                }).collect::<Vec<_>>(),
            })
        }),
    })
}

fn trigger_label(trigger: TriggerClass) -> &'static str {
    match trigger {
        TriggerClass::CheckpointReady => "checkpoint_ready",
        TriggerClass::Heartbeat => "heartbeat",
        TriggerClass::RepeatedFailure => "repeated_failure",
        TriggerClass::ManualReview => "manual_review",
    }
}

fn canonical_runtime_snapshot(snapshot: &agent_drift_sentinel::LiveRuntimeSnapshot) -> Value {
    serde_json::json!({
        "latest_cursor": snapshot.latest_cursor.as_ref().map(|cursor| serde_json::json!({
            "session_id": cursor.session_id,
            "ordinal": cursor.ordinal,
        })),
        "latest_checkpoint_id": snapshot.latest_checkpoint_id,
        "last_trigger": snapshot.last_trigger.map(trigger_label),
        "processed_events": snapshot.processed_events,
        "scheduler_state": {
            "last_evaluated": snapshot.scheduler_state.last_evaluated.as_ref().map(|cursor| serde_json::json!({
                "session_id": cursor.session_id,
                "ordinal": cursor.ordinal,
            })),
            "checkpoints_since_last_evaluation": snapshot.scheduler_state.checkpoints_since_last_evaluation,
            "last_visible_warning_fingerprint": snapshot.scheduler_state.last_visible_warning_fingerprint,
            "checkpoints_since_last_visible_warning": snapshot.scheduler_state.checkpoints_since_last_visible_warning,
            "consecutive_flagged_checkpoints": snapshot.scheduler_state.consecutive_flagged_checkpoints,
        },
    })
}

fn run_canonical_bundle_case(case: &RecallCase) -> Value {
    let bundle_dir = fixture_root().join(&case.fixture_dir).join("bundle");
    let error = agent_drift_analyzer::input::load_bundle(&bundle_dir)
        .expect_err("P7 whole-wall malformed bundle must be rejected");
    match error {
        agent_drift_analyzer::InputError::VerifiedDelegationSessionMissing {
            parent_session_id,
            child_session_id,
            missing_session_id,
        } => serde_json::json!({
            "status": "error",
            "error": "verified_delegation_session_missing",
            "parent_session_id": parent_session_id,
            "child_session_id": child_session_id,
            "missing_session_id": missing_session_id,
        }),
        agent_drift_analyzer::InputError::MissingArtifact { path } => serde_json::json!({
            "status": "error",
            "error": "missing_artifact",
            "missing_artifact": path.file_name(),
        }),
        other => panic!("{} unexpected analyzer input error: {other}", case.case_id),
    }
}

fn run_canonical_public_live_case(case: &RecallCase) -> Value {
    let fixture: Value = serde_json::from_str(
        &fs::read_to_string(
            fixture_root()
                .join(&case.fixture_dir)
                .join("live-event.json"),
        )
        .expect("read P7 whole-wall public-live fixture"),
    )
    .expect("parse P7 whole-wall public-live fixture");
    let event = LiveCheckpointEvent {
        emission_ordinal: fixture["emission_ordinal"]
            .as_u64()
            .expect("P7 whole-wall emission ordinal") as usize,
        cursor: CheckpointCursor {
            session_id: fixture["cursor"]["session_id"]
                .as_str()
                .expect("P7 whole-wall cursor session")
                .to_string(),
            ordinal: fixture["cursor"]["ordinal"]
                .as_u64()
                .expect("P7 whole-wall cursor ordinal") as usize,
        },
        trigger: TriggerClass::CheckpointReady,
        checkpoint: None,
        source_label: fixture["source_label"].as_str().map(str::to_string),
    };
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let before = runtime.snapshot();
    let error = runtime
        .observe(event)
        .expect_err("P7 whole-wall public-live fixture must be rejected");
    let emission_ordinal = match error {
        LiveRuntimeError::MissingCheckpointPayload { emission_ordinal } => emission_ordinal,
        other => panic!("{} unexpected public-live error: {other}", case.case_id),
    };
    let after = runtime.snapshot();
    serde_json::json!({
        "status": "error",
        "error": "missing_checkpoint_payload",
        "emission_ordinal": emission_ordinal,
        "state_mutated": before != after,
        "processed_events_before": before.processed_events,
        "processed_events_after": after.processed_events,
    })
}

fn validate_historical_references(matrix: &RecallMatrix) -> Result<(), String> {
    let repo_root = Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Utf8Path::parent)
        .ok_or_else(|| {
            "P7 historical references [owner=harness] cannot resolve repo root".to_string()
        })?;
    for case in &matrix.cases {
        for reference in &case.historical_references {
            let path = repo_root.join(reference);
            if !path.is_file() {
                return Err(case_error(
                    case,
                    &format!("historical reference does not resolve: {reference}"),
                ));
            }
            let tracked = Command::new("git")
                .args([
                    "-C",
                    repo_root.as_str(),
                    "ls-files",
                    "--error-unmatch",
                    "--",
                ])
                .arg(reference)
                .output()
                .map_err(|error| {
                    case_error(
                        case,
                        &format!("cannot inspect historical reference {reference}: {error}"),
                    )
                })?;
            if !tracked.status.success() {
                return Err(case_error(
                    case,
                    &format!("historical reference is not tracked: {reference}"),
                ));
            }
        }
    }
    Ok(())
}
