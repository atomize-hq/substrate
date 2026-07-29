#![allow(unused_crate_dependencies)]

use std::collections::BTreeSet;
use std::fs;
use std::thread;
use std::time::Duration;

use agent_session_compactor::{
    ingest_rollout_file, BoundedClosureCompactor, BoundedClosureError, BoundedClosureRequest,
    CompactorError, DelegationLinkState, RolloutFormat,
};
use anyhow as _;
use blake3 as _;
use camino::{Utf8Path, Utf8PathBuf};
use clap as _;
use codex as _;
use serde as _;
use serde_json::{json, Value};
use tempfile::TempDir;
use thiserror as _;
use time::macros::datetime;
use walkdir as _;

#[test]
fn bounded_closure_indexes_cross_format_linkage_before_decoding_selected_sources() {
    let home = TestCodexHome::new();
    let root = home.write(
        "rollout-root.jsonl",
        &legacy_root_rollout("root", "child", "shared event body"),
    );
    let child = home.write(
        "rollout-child.jsonl",
        &current_child_rollout("root", "child", "shared event body", false),
    );
    let unrelated = home.write(
        "rollout-unrelated.jsonl",
        &current_session_rollout("unrelated", "shared event body", false),
    );

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&home.request("root"))
        .expect("prepare bounded closure");
    let snapshot = prepared.snapshot();
    assert!(snapshot.root_identity_validated);
    assert_eq!(snapshot.root_source.source_file, root);
    assert_eq!(snapshot.root_source.format, RolloutFormat::Legacy);
    assert!(snapshot.startup_readiness.has_session_activity);
    assert!(snapshot.startup_readiness.has_literal_directive_text);
    assert!(snapshot.startup_readiness.has_path_hint);
    assert!(snapshot.startup_readiness.has_parseable_tool_call_arguments);

    let selected_ids = snapshot
        .selected_sources
        .iter()
        .filter_map(|source| source.session_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected_ids,
        BTreeSet::from(["child".to_string(), "root".to_string()])
    );
    let child_source = snapshot
        .selected_sources
        .iter()
        .find(|source| source.session_id.as_deref() == Some("child"))
        .expect("selected current-native child");
    assert_eq!(child_source.format, RolloutFormat::CurrentNativeV2);
    assert!(!snapshot
        .selected_sources
        .iter()
        .any(|source| source.source_file == unrelated));

    let result = compactor
        .compact(
            prepared,
            &home.output("cross-format"),
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect("compact bounded closure");
    assert_eq!(
        result
            .decoded_source_files
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([root, child])
    );
    assert!(!result.decoded_source_files.contains(&unrelated));
    assert_eq!(
        result.run_result.manifest.session_ids,
        vec!["child".to_string(), "root".to_string()]
    );
    assert_eq!(result.run_result.manifest.delegation_links.len(), 1);
    assert_eq!(
        result.run_result.manifest.delegation_links[0].state,
        DelegationLinkState::Verified
    );
}

#[test]
fn current_native_parent_envelope_selects_reciprocal_legacy_child() {
    let home = TestCodexHome::new();
    let root = home.write(
        "rollout-current-root.jsonl",
        &current_root_rollout("root", "child"),
    );
    let child = home.write(
        "rollout-legacy-child.jsonl",
        &legacy_child_rollout("root", "child"),
    );

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&home.request("root"))
        .expect("prepare current-native parent closure");
    assert_eq!(
        prepared.snapshot().root_source.format,
        RolloutFormat::CurrentNativeV2
    );
    assert_eq!(prepared.snapshot().selected_sources.len(), 2);
    let result = compactor
        .compact(
            prepared,
            &home.output("current-parent"),
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect("compact current-native parent closure");
    assert_eq!(
        result
            .decoded_source_files
            .into_iter()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([root, child])
    );
    assert_eq!(result.run_result.manifest.delegation_links.len(), 1);
    assert_eq!(
        result.run_result.manifest.delegation_links[0].state,
        DelegationLinkState::Verified
    );
}

#[test]
fn malformed_unrelated_body_is_excluded_without_full_decode() {
    let home = TestCodexHome::new();
    let root = home.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "root body"),
    );
    let unrelated = home.write(
        "rollout-unrelated.jsonl",
        &current_session_rollout("unrelated", "ignored", true),
    );
    let directly_ingested = ingest_rollout_file(&unrelated).expect("ingest malformed source");
    assert_eq!(directly_ingested.parse_failures.len(), 1);

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&home.request("root"))
        .expect("unrelated malformed body remains excludable");
    assert_eq!(prepared.snapshot().selected_sources.len(), 1);
    let result = compactor
        .compact(
            prepared,
            &home.output("excluded-malformed"),
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect("compact selected root only");
    assert_eq!(result.decoded_source_files, vec![root]);
    assert!(!result.decoded_source_files.contains(&unrelated));
}

#[test]
fn malformed_selected_payload_fails_closed() {
    let home = TestCodexHome::new();
    home.write(
        "rollout-root.jsonl",
        &legacy_root_rollout("root", "child", "root body"),
    );
    let child = home.write(
        "rollout-child.jsonl",
        &current_child_rollout("root", "child", "selected body", true),
    );

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&home.request("root"))
        .expect("linkage envelope selects malformed child");
    let error = compactor
        .compact(
            prepared,
            &home.output("selected-malformed"),
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect_err("selected malformed body must fail closed");
    assert!(matches!(
        error,
        CompactorError::BoundedClosure(BoundedClosureError::SelectedPayloadMalformed {
            path,
            ..
        }) if path == child
    ));
}

#[test]
fn newly_created_linked_source_invalidates_a_prepared_closure() {
    let home = TestCodexHome::new();
    home.write(
        "rollout-root.jsonl",
        &legacy_root_rollout("root", "child", "root body"),
    );

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&home.request("root"))
        .expect("prepare root before linked child exists");
    assert_eq!(prepared.snapshot().selected_sources.len(), 1);

    home.write(
        "rollout-child.jsonl",
        &current_child_rollout("root", "child", "child body", false),
    );
    let error = compactor
        .compact(
            prepared,
            &home.output("stale-linked-source"),
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect_err("newly discovered linked source must invalidate stale preparation");
    assert!(matches!(
        error,
        CompactorError::BoundedClosure(
            BoundedClosureError::DiscoveredSourceStateChanged { codex_home }
        ) if codex_home == home.codex_home
    ));

    let refreshed = compactor
        .prepare(&home.request("root"))
        .expect("fresh preparation includes the new linked child");
    assert_eq!(
        refreshed
            .snapshot()
            .selected_sources
            .iter()
            .filter_map(|source| source.session_id.as_deref())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["child", "root"])
    );
}

#[test]
fn newly_created_duplicate_root_identity_invalidates_a_prepared_closure() {
    let home = TestCodexHome::new();
    home.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "root body"),
    );

    let mut compactor = BoundedClosureCompactor::default();
    let prepared = compactor
        .prepare(&home.request("root"))
        .expect("prepare unique root identity");

    let duplicate = home.write(
        "rollout-root-duplicate.jsonl",
        &current_session_rollout("root", "duplicate root", false),
    );
    let error = compactor
        .compact(
            prepared,
            &home.output("stale-duplicate-root"),
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect_err("new duplicate root identity must invalidate stale preparation");
    assert!(matches!(
        error,
        CompactorError::BoundedClosure(
            BoundedClosureError::DiscoveredSourceStateChanged { codex_home }
        ) if codex_home == home.codex_home
    ));

    let error = compactor
        .prepare(&home.request("root"))
        .expect_err("fresh preparation must expose duplicate root identity");
    assert!(matches!(
        error,
        BoundedClosureError::Discovery(
            agent_session_compactor::DiscoveryError::AmbiguousLinkedSession {
                session_id,
                source_files,
            }
        ) if session_id == "root" && source_files.contains(&duplicate)
    ));
}

#[test]
fn insufficient_or_ambiguous_envelopes_remain_explicit_errors() {
    let missing_identity = TestCodexHome::new();
    missing_identity.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "root body"),
    );
    let unidentified = missing_identity.write(
        "rollout-unidentified.jsonl",
        "{\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-x\"}}\n",
    );
    let mut compactor = BoundedClosureCompactor::default();
    let error = compactor
        .prepare(&missing_identity.request("root"))
        .expect_err("non-empty identity-free source is indeterminate");
    assert!(matches!(
        error,
        BoundedClosureError::IndeterminateSourceIdentity { path } if path == unidentified
    ));

    let ambiguous = TestCodexHome::new();
    ambiguous.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "root body"),
    );
    let broken = ambiguous.write("rollout-broken.jsonl", "{not-json}\n");
    let mut compactor = BoundedClosureCompactor::default();
    let error = compactor
        .prepare(&ambiguous.request("root"))
        .expect_err("envelope-free event must remain explicit");
    assert!(matches!(
        error,
        BoundedClosureError::IndeterminateEnvelope { path, .. } if path == broken
    ));

    let malformed_origin = TestCodexHome::new();
    malformed_origin.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "root body"),
    );
    let malformed_source = malformed_origin.write(
        "rollout-other.jsonl",
        &encode_jsonl(vec![json!({
            "type": "session_meta",
            "payload": {
                "id": "other",
                "multi_agent_version": "v2",
                "source": []
            }
        })]),
    );
    let mut compactor = BoundedClosureCompactor::default();
    let error = compactor
        .prepare(&malformed_origin.request("root"))
        .expect_err("ambiguous current-native child origin must remain explicit");
    assert!(matches!(
        error,
        BoundedClosureError::IndeterminateEnvelope { path, .. } if path == malformed_source
    ));
}

#[test]
fn cache_reuses_unchanged_indexes_and_invalidates_on_append_and_root_change() {
    let home = TestCodexHome::new();
    let root = home.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "alpha"),
    );
    let unrelated = home.write(
        "rollout-other.jsonl",
        &legacy_session_rollout("other", "other body"),
    );
    let mut compactor = BoundedClosureCompactor::default();

    let first = compactor
        .prepare(&home.request("root"))
        .expect("first prepare");
    assert_eq!(first.cache_stats().rebuilt_source_index_count, 2);
    assert_eq!(first.cache_stats().reused_source_index_count, 0);
    assert!(!first.cache_stats().closure_selection_reused);
    let first_revision = first.snapshot().revision.clone();
    let first_state_token = first.state_token().to_string();
    let first_root_high_water = first.snapshot().root_source.high_water_mark;

    let unchanged = compactor
        .prepare(&home.request("root"))
        .expect("unchanged prepare");
    assert_eq!(unchanged.cache_stats().rebuilt_source_index_count, 0);
    assert_eq!(unchanged.cache_stats().reused_source_index_count, 2);
    assert!(unchanged.cache_stats().closure_selection_reused);
    assert_eq!(unchanged.snapshot().revision, first_revision);
    assert_eq!(unchanged.state_token(), first_state_token.as_str());

    fs::OpenOptions::new()
        .append(true)
        .open(&unrelated)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(b"{\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\"}}\n")
        })
        .expect("append unrelated history");
    let appended = compactor
        .prepare(&home.request("root"))
        .expect("prepare after unrelated append");
    assert_eq!(appended.cache_stats().rebuilt_source_index_count, 1);
    assert_eq!(appended.cache_stats().reused_source_index_count, 1);
    assert!(!appended.cache_stats().closure_selection_reused);
    assert_eq!(appended.snapshot().revision, first_revision);
    assert_ne!(appended.state_token(), first_state_token.as_str());
    assert_eq!(
        appended.snapshot().root_source.high_water_mark,
        first_root_high_water
    );

    let other_root = compactor
        .prepare(&home.request("other"))
        .expect("change closure root");
    assert_eq!(other_root.cache_stats().rebuilt_source_index_count, 0);
    assert_eq!(other_root.cache_stats().reused_source_index_count, 2);
    assert!(!other_root.cache_stats().closure_selection_reused);
    assert_eq!(other_root.snapshot().root_source.source_file, unrelated);

    fs::OpenOptions::new()
        .append(true)
        .open(&root)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(b"{\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\"}}\n")
        })
        .expect("append selected root history");
    let selected_append = compactor
        .prepare(&home.request("root"))
        .expect("prepare after selected append");
    assert_eq!(selected_append.cache_stats().rebuilt_source_index_count, 1);
    assert_eq!(selected_append.cache_stats().reused_source_index_count, 1);
    assert!(!selected_append.cache_stats().closure_selection_reused);
    assert!(selected_append.snapshot().root_source.high_water_mark > first_root_high_water);
    assert_ne!(selected_append.snapshot().revision, first_revision);
}

#[test]
fn cache_generation_invalidates_same_length_rewrite() {
    let home = TestCodexHome::new();
    let root = home.write(
        "rollout-root.jsonl",
        &legacy_session_rollout("root", "alpha"),
    );
    let mut compactor = BoundedClosureCompactor::default();
    let first = compactor
        .prepare(&home.request("root"))
        .expect("first prepare");
    let first_generation = first.snapshot().root_source.generation;
    let first_high_water = first.snapshot().root_source.high_water_mark;
    let first_state_token = first.state_token().to_string();

    thread::sleep(Duration::from_millis(1100));
    let rewritten = legacy_session_rollout("root", "omega");
    assert_eq!(rewritten.len() as u64, first_high_water);
    fs::write(&root, rewritten).expect("same-length rewrite");

    let second = compactor
        .prepare(&home.request("root"))
        .expect("prepare rewritten root");
    assert_eq!(second.cache_stats().rebuilt_source_index_count, 1);
    assert!(!second.cache_stats().closure_selection_reused);
    assert_eq!(
        second.snapshot().root_source.high_water_mark,
        first_high_water
    );
    assert_ne!(second.snapshot().root_source.generation, first_generation);
    assert_ne!(second.state_token(), first_state_token.as_str());
}

#[test]
fn reordered_and_appended_global_history_preserves_selected_order_and_replay_rows() {
    let first_home = TestCodexHome::new();
    first_home.write(
        "rollout-z-root.jsonl",
        &legacy_root_rollout("root", "child", "root body"),
    );
    first_home.write(
        "rollout-a-child.jsonl",
        &current_child_rollout("root", "child", "child body", false),
    );
    let unrelated = first_home.write(
        "rollout-m-unrelated.jsonl",
        &legacy_session_rollout("unrelated", "unrelated body"),
    );

    let second_home = TestCodexHome::new();
    second_home.write(
        "rollout-m-unrelated.jsonl",
        &legacy_session_rollout("unrelated", "unrelated body"),
    );
    second_home.write(
        "rollout-a-child.jsonl",
        &current_child_rollout("root", "child", "child body", false),
    );
    second_home.write(
        "rollout-z-root.jsonl",
        &legacy_root_rollout("root", "child", "root body"),
    );

    let mut first_compactor = BoundedClosureCompactor::default();
    let first_prepared = first_compactor
        .prepare(&first_home.request("root"))
        .expect("first closure");
    let first_revision = first_prepared.snapshot().revision.clone();
    let first_names = selected_file_names(first_prepared.snapshot());
    let first_output = first_home.output("first-order");
    first_compactor
        .compact(
            first_prepared,
            &first_output,
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect("first compaction");

    let mut second_compactor = BoundedClosureCompactor::default();
    let second_prepared = second_compactor
        .prepare(&second_home.request("root"))
        .expect("second closure");
    assert_eq!(selected_file_names(second_prepared.snapshot()), first_names);
    let second_output = second_home.output("second-order");
    second_compactor
        .compact(
            second_prepared,
            &second_output,
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect("second compaction");
    assert_eq!(
        fs::read(first_output.join("rows.archival.jsonl")).expect("first archival rows"),
        fs::read(second_output.join("rows.archival.jsonl")).expect("second archival rows")
    );
    assert_eq!(
        fs::read(first_output.join("rows.compact.jsonl")).expect("first compact rows"),
        fs::read(second_output.join("rows.compact.jsonl")).expect("second compact rows")
    );

    fs::OpenOptions::new()
        .append(true)
        .open(unrelated)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(b"{\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\"}}\n")
        })
        .expect("append unrelated history");
    let appended_prepared = first_compactor
        .prepare(&first_home.request("root"))
        .expect("closure after unrelated append");
    assert_eq!(appended_prepared.snapshot().revision, first_revision);
    assert_eq!(
        selected_file_names(appended_prepared.snapshot()),
        first_names
    );
    let appended_output = first_home.output("appended-unrelated");
    first_compactor
        .compact(
            appended_prepared,
            &appended_output,
            Some(datetime!(2026-07-28 12:00:00 UTC)),
        )
        .expect("compaction after unrelated append");
    assert_eq!(
        fs::read(first_output.join("rows.archival.jsonl")).expect("initial archival rows"),
        fs::read(appended_output.join("rows.archival.jsonl")).expect("appended archival rows")
    );
    assert_eq!(
        fs::read(first_output.join("rows.compact.jsonl")).expect("initial compact rows"),
        fs::read(appended_output.join("rows.compact.jsonl")).expect("appended compact rows")
    );
}

struct TestCodexHome {
    _temp_dir: TempDir,
    codex_home: Utf8PathBuf,
    sessions_dir: Utf8PathBuf,
}

impl TestCodexHome {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let codex_home = root.join(".codex");
        let sessions_dir = codex_home.join("sessions/2026/07/28");
        fs::create_dir_all(&sessions_dir).expect("create sessions dir");
        Self {
            _temp_dir: temp_dir,
            codex_home,
            sessions_dir,
        }
    }

    fn write(&self, file_name: &str, contents: &str) -> Utf8PathBuf {
        let path = self.sessions_dir.join(file_name);
        fs::write(&path, contents).expect("write rollout");
        path
    }

    fn request(&self, root_session_id: &str) -> BoundedClosureRequest {
        BoundedClosureRequest {
            codex_home: Some(self.codex_home.clone()),
            root_session_id: root_session_id.to_string(),
        }
    }

    fn output(&self, name: &str) -> Utf8PathBuf {
        self.codex_home.join("outputs").join(name)
    }
}

fn selected_file_names(snapshot: &agent_session_compactor::BoundedClosureSnapshot) -> Vec<String> {
    snapshot
        .selected_sources
        .iter()
        .map(|source| {
            source
                .source_file
                .file_name()
                .expect("selected source file name")
                .to_string()
        })
        .collect()
}

fn legacy_session_rollout(session_id: &str, body: &str) -> String {
    encode_jsonl(vec![
        json!({
            "timestamp": "2026-07-28T12:00:00Z",
            "type": "session_meta",
            "payload": {
                "id": session_id,
                "base_instructions": { "text": "Base instructions" }
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:01Z",
            "type": "event_msg",
            "payload": { "type": "task_started", "turn_id": "turn-1" }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:02Z",
            "type": "turn_context",
            "payload": {
                "turn_id": "turn-1",
                "user_instructions": "Inspect crates/agent-session-compactor/src/lib.rs"
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:03Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": body }]
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:04Z",
            "type": "response_item",
            "payload": {
                "type": "function_call",
                "name": "functions.shell_command",
                "arguments": json!({
                    "command": "cargo test -p agent-session-compactor",
                    "workdir": "/repo"
                })
                .to_string(),
                "call_id": "call-tool"
            }
        }),
    ])
}

fn legacy_root_rollout(root_session_id: &str, child_session_id: &str, body: &str) -> String {
    let mut rollout = legacy_session_rollout(root_session_id, body);
    rollout.push_str(&encode_jsonl(vec![
        json!({
            "timestamp": "2026-07-28T12:00:05Z",
            "type": "response_item",
            "payload": {
                "type": "function_call",
                "name": "spawn_agent",
                "call_id": "call-spawn",
                "arguments": "{}"
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:06Z",
            "type": "response_item",
            "payload": {
                "type": "function_call_output",
                "call_id": "call-spawn",
                "output": json!({ "agent_id": child_session_id }).to_string()
            }
        }),
    ]));
    rollout
}

fn current_session_rollout(session_id: &str, body: &str, malformed_body: bool) -> String {
    let content = if malformed_body {
        Value::String("malformed-content".to_string())
    } else {
        json!([{ "type": "input_text", "text": body }])
    };
    encode_jsonl(vec![
        json!({
            "timestamp": "2026-07-28T12:00:00Z",
            "type": "session_meta",
            "payload": { "id": session_id, "multi_agent_version": "v2" }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:01Z",
            "type": "event_msg",
            "payload": { "type": "task_started", "turn_id": "turn-1" }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:02Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": content,
                "internal_chat_message_metadata_passthrough": { "turn_id": "turn-1" }
            }
        }),
    ])
}

fn current_child_rollout(
    parent_session_id: &str,
    child_session_id: &str,
    body: &str,
    malformed_body: bool,
) -> String {
    let content = if malformed_body {
        Value::String("malformed-content".to_string())
    } else {
        json!([{ "type": "input_text", "text": body }])
    };
    encode_jsonl(vec![
        json!({
            "timestamp": "2026-07-28T12:00:00Z",
            "type": "session_meta",
            "payload": {
                "id": child_session_id,
                "multi_agent_version": "v2",
                "source": {
                    "subagent": {
                        "thread_spawn": {
                            "parent_thread_id": parent_session_id,
                            "depth": 1,
                            "agent_nickname": "child",
                            "agent_role": "default"
                        }
                    }
                }
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:01Z",
            "type": "event_msg",
            "payload": { "type": "task_started", "turn_id": "turn-child" }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:02Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": content,
                "internal_chat_message_metadata_passthrough": { "turn_id": "turn-child" }
            }
        }),
    ])
}

fn legacy_child_rollout(parent_session_id: &str, child_session_id: &str) -> String {
    encode_jsonl(vec![
        json!({
            "timestamp": "2026-07-28T12:00:00Z",
            "type": "session_meta",
            "payload": {
                "id": child_session_id,
                "source": {
                    "subagent": {
                        "thread_spawn": {
                            "parent_thread_id": parent_session_id,
                            "depth": 1,
                            "agent_nickname": "",
                            "agent_role": ""
                        }
                    }
                }
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:01Z",
            "type": "event_msg",
            "payload": { "type": "task_started", "turn_id": "turn-child" }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:02Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "legacy child body" }]
            }
        }),
    ])
}

fn current_root_rollout(root_session_id: &str, child_session_id: &str) -> String {
    encode_jsonl(vec![
        json!({
            "timestamp": "2026-07-28T12:00:00Z",
            "type": "session_meta",
            "payload": {
                "id": root_session_id,
                "multi_agent_version": "v2",
                "base_instructions": { "text": "Base instructions" }
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:01Z",
            "type": "event_msg",
            "payload": {
                "type": "collab_agent_spawn_begin",
                "call_id": "call-current-spawn",
                "sender_thread_id": root_session_id,
                "prompt": ""
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:02Z",
            "type": "event_msg",
            "payload": {
                "type": "collab_agent_spawn_end",
                "call_id": "call-current-spawn",
                "sender_thread_id": root_session_id,
                "new_thread_id": child_session_id
            }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:03Z",
            "type": "event_msg",
            "payload": { "type": "task_started", "turn_id": "turn-root" }
        }),
        json!({
            "timestamp": "2026-07-28T12:00:04Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "current root body" }],
                "internal_chat_message_metadata_passthrough": { "turn_id": "turn-root" }
            }
        }),
    ])
}

fn encode_jsonl(records: Vec<Value>) -> String {
    let mut encoded = String::new();
    for record in records {
        encoded.push_str(&record.to_string());
        encoded.push('\n');
    }
    encoded
}
