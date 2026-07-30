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
                actual_variants.insert(variant.to_string());
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
            "{\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-p7-01-main\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"custom_tool_call\",\"name\":\"shell_command\",\"call_id\":\"call-p7-01-build\",\"input\":\"{}\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"custom_tool_call_output\",\"call_id\":\"call-p7-01-build\",\"output\":[{\"type\":\"input_text\",\"text\":\"ok\"}]}}\n",
        ),
    )
    .expect("write synthetic raw stream");

    let matrix = load_matrix().expect("load P7 matrix");
    let mut implemented = matrix.clone();
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
