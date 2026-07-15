use std::collections::BTreeSet;
use std::fs;

use agent_session_compactor::ingest::ingest_rollout_file;
use anyhow as _;
use blake3 as _;
use camino::{Utf8Path, Utf8PathBuf};
use clap as _;
use codex as _;
use serde as _;
use serde_json::{Map, Value};
use tempfile as _;
use thiserror as _;
use time as _;
use walkdir as _;

const CASES: &[(&str, &[&str])] = &[
    (
        "reciprocal",
        &["verified:session-parent-alpha:session-child-alpha"],
    ),
    (
        "parent_only",
        &["parent_only:session-parent-bravo:session-child-bravo"],
    ),
    (
        "child_only",
        &["child_only:session-parent-charlie:session-child-charlie"],
    ),
    (
        "conflict",
        &["conflicting:session-parent-delta:session-parent-other:session-child-delta"],
    ),
    (
        "multi_child",
        &[
            "verified:session-parent-echo:session-child-echo-one",
            "verified:session-parent-echo:session-child-echo-two",
        ],
    ),
    (
        "nested_depth",
        &["nested_residue:session-parent-foxtrot:session-child-foxtrot:depth=2"],
    ),
    ("single_agent", &["single_agent:session-single-golf"]),
];

const FORBIDDEN_CONTENT: &[&str] = &[
    "/users/",
    "/home/",
    ".codex",
    "bearer ",
    "sk-",
    "ghp_",
    "password",
    "credential",
    "authorization",
    "api_key",
];

#[derive(Debug)]
struct SpawnClaim {
    parent_id: String,
    child_id: String,
}

#[derive(Debug)]
struct ChildOrigin {
    child_id: String,
    parent_id: String,
    depth: u64,
}

#[test]
fn delegation_link_fixtures_parse_and_cover_the_bounded_matrix() {
    for (case_name, expected) in CASES {
        let files = fixture_files(case_name);
        for path in &files {
            let ingested = ingest_rollout_file(path).expect("fixture should be readable");
            assert!(
                ingested.parse_failures.is_empty(),
                "{path} should parse without failures: {:?}",
                ingested.parse_failures
            );
            assert_eq!(
                ingested.records.len(),
                read_rows(path).len(),
                "every nonempty fixture line should parse in {path}"
            );
        }

        assert_eq!(
            derive_outcomes(&files),
            expected.iter().map(|value| value.to_string()).collect(),
            "fixture case {case_name} should preserve its declared bounded semantics"
        );
    }
}

#[test]
fn delegation_link_fixtures_keep_only_allowlisted_sanitized_fields() {
    for (case_name, _) in CASES {
        for path in fixture_files(case_name) {
            let content = fs::read_to_string(&path).expect("read fixture");
            let lowercase = content.to_ascii_lowercase();
            for forbidden in FORBIDDEN_CONTENT {
                assert!(
                    !lowercase.contains(forbidden),
                    "{path} contains forbidden private-content marker {forbidden:?}"
                );
            }
            for row in read_rows(&path) {
                assert_allowed_row(&row);
            }
        }
    }
}

fn fixture_files(case_name: &str) -> Vec<Utf8PathBuf> {
    let mut files = fs::read_dir(fixture_root().join(case_name))
        .expect("read fixture case")
        .map(|entry| {
            Utf8PathBuf::from_path_buf(entry.expect("fixture entry").path())
                .expect("fixture path should be UTF-8")
        })
        .filter(|path| path.extension() == Some("jsonl"))
        .collect::<Vec<_>>();
    files.sort();
    assert!(
        !files.is_empty(),
        "fixture case {case_name} must not be empty"
    );
    files
}

fn fixture_root() -> Utf8PathBuf {
    Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/delegation_links")
}

fn read_rows(path: &Utf8Path) -> Vec<Value> {
    fs::read_to_string(path)
        .expect("read fixture")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("fixture JSON row"))
        .collect()
}

fn derive_outcomes(files: &[Utf8PathBuf]) -> BTreeSet<String> {
    let mut session_ids = BTreeSet::new();
    let mut claims = Vec::new();
    let mut origins = Vec::new();

    for path in files {
        let rows = read_rows(path);
        let session_id = text_at(
            rows.iter()
                .find(|row| text_at(row, "/type") == "session_meta")
                .expect("fixture file has session_meta"),
            "/payload/id",
        )
        .to_string();
        session_ids.insert(session_id.clone());

        if let Some(spawn) = rows
            .iter()
            .find_map(|row| row.pointer("/payload/source/subagent/thread_spawn"))
        {
            origins.push(ChildOrigin {
                child_id: session_id.clone(),
                parent_id: text_at(spawn, "/parent_thread_id").to_string(),
                depth: spawn
                    .get("depth")
                    .and_then(Value::as_u64)
                    .expect("child origin depth"),
            });
        }

        let spawn_calls = rows
            .iter()
            .filter(|row| {
                text_at(row, "/payload/type") == "function_call"
                    && text_at(row, "/payload/name") == "spawn_agent"
            })
            .map(|row| text_at(row, "/payload/call_id").to_string())
            .collect::<BTreeSet<_>>();
        for row in rows.iter().filter(|row| {
            row.pointer("/payload/type").and_then(Value::as_str) == Some("function_call_output")
        }) {
            if !spawn_calls.contains(text_at(row, "/payload/call_id")) {
                continue;
            }
            let output: Value =
                serde_json::from_str(text_at(row, "/payload/output")).expect("spawn output JSON");
            claims.push(SpawnClaim {
                parent_id: session_id.clone(),
                child_id: text_at(&output, "/agent_id").to_string(),
            });
        }
    }

    let mut outcomes = BTreeSet::new();
    for claim in &claims {
        match origins
            .iter()
            .find(|origin| origin.child_id == claim.child_id)
        {
            None => {
                outcomes.insert(format!(
                    "parent_only:{}:{}",
                    claim.parent_id, claim.child_id
                ));
            }
            Some(origin) if origin.depth != 1 => {
                outcomes.insert(format!(
                    "nested_residue:{}:{}:depth={}",
                    claim.parent_id, claim.child_id, origin.depth
                ));
            }
            Some(origin) if origin.parent_id == claim.parent_id => {
                outcomes.insert(format!("verified:{}:{}", claim.parent_id, claim.child_id));
            }
            Some(origin) => {
                outcomes.insert(format!(
                    "conflicting:{}:{}:{}",
                    claim.parent_id, origin.parent_id, claim.child_id
                ));
            }
        }
    }
    for origin in &origins {
        if !claims.iter().any(|claim| claim.child_id == origin.child_id) {
            outcomes.insert(format!(
                "child_only:{}:{}",
                origin.parent_id, origin.child_id
            ));
        }
    }
    if claims.is_empty() && origins.is_empty() {
        outcomes.extend(
            session_ids
                .into_iter()
                .map(|session_id| format!("single_agent:{session_id}")),
        );
    }
    outcomes
}

fn assert_allowed_row(row: &Value) {
    exact_keys(object(row), &["payload", "type"], "rollout row");
    let payload = object(row.get("payload").expect("payload"));
    match text_at(row, "/type") {
        "session_meta" => assert_allowed_session_meta(payload),
        "response_item" => assert_allowed_response_item(payload),
        other => panic!("unexpected rollout row type: {other}"),
    }
}

fn assert_allowed_session_meta(payload: &Map<String, Value>) {
    safe_token(text(payload, "id"), "session-");
    let Some(source) = payload.get("source") else {
        exact_keys(payload, &["id"], "ordinary session_meta payload");
        return;
    };

    exact_keys(payload, &["id", "source"], "child session_meta payload");
    let source = object(source);
    exact_keys(source, &["subagent"], "session source");
    let subagent = object(source.get("subagent").expect("subagent source"));
    exact_keys(subagent, &["thread_spawn"], "subagent source");
    let spawn = object(subagent.get("thread_spawn").expect("thread_spawn"));
    assert!(
        spawn.keys().all(
            |key| ["agent_nickname", "agent_role", "depth", "parent_thread_id"]
                .contains(&key.as_str())
        ),
        "unexpected child-origin field: {:?}",
        spawn.keys().collect::<Vec<_>>()
    );
    assert!(spawn.contains_key("depth"));
    safe_token(text(spawn, "parent_thread_id"), "session-");
    if let Some(nickname) = spawn.get("agent_nickname") {
        safe_token(nickname.as_str().expect("nickname string"), "agent-");
    }
    if let Some(role) = spawn.get("agent_role") {
        assert_eq!(role.as_str(), Some("default"));
    }
}

fn assert_allowed_response_item(payload: &Map<String, Value>) {
    match text(payload, "type") {
        "function_call" => {
            exact_keys(payload, &["call_id", "name", "type"], "spawn call");
            assert_eq!(text(payload, "name"), "spawn_agent");
            safe_token(text(payload, "call_id"), "call-");
        }
        "function_call_output" => {
            exact_keys(payload, &["call_id", "output", "type"], "spawn output row");
            safe_token(text(payload, "call_id"), "call-");
            let output: Value =
                serde_json::from_str(text(payload, "output")).expect("spawn output JSON");
            exact_keys(object(&output), &["agent_id"], "spawn output");
            safe_token(text(object(&output), "agent_id"), "session-");
        }
        other => panic!("unexpected response item: {other}"),
    }
}

fn safe_token(value: &str, prefix: &str) {
    assert!(
        value.starts_with(prefix),
        "{value:?} must start with {prefix:?}"
    );
    assert!(value.len() <= 64, "sanitized token is unexpectedly long");
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "sanitized token contains private or non-generic characters: {value:?}"
    );
}

fn exact_keys(object: &Map<String, Value>, expected: &[&str], context: &str) {
    assert_eq!(
        object.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected.iter().copied().collect::<BTreeSet<_>>(),
        "{context} must contain only the linkage allowlist"
    );
}

fn text_at<'a>(value: &'a Value, pointer: &str) -> &'a str {
    value.pointer(pointer).and_then(Value::as_str).unwrap_or("")
}

fn text<'a>(object: &'a Map<String, Value>, key: &str) -> &'a str {
    object.get(key).and_then(Value::as_str).unwrap_or("")
}

fn object(value: &Value) -> &Map<String, Value> {
    value.as_object().expect("JSON object")
}
