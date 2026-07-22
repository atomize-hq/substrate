use anyhow as _;
use blake3 as _;
use clap as _;
use codex as _;
use serde as _;
use serde_json::{self, json, Value};
use std::fs;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use walkdir as _;

use agent_session_compactor::dedupe::dedupe_rows_exact;
use agent_session_compactor::ingest::{
    extract_rollout_linkage_metadata, ingest_rollout_file, IngestError, RolloutFormat,
};
use agent_session_compactor::normalize::{normalize_rollout_file, CompactionKind};
use camino::{Utf8Path, Utf8PathBuf};

#[test]
fn current_native_adapter_uses_input_and_preserves_typed_output_order_and_types() {
    let rollout = ingest_fixture("commands-and-typed-output.jsonl");

    assert_eq!(rollout.format, RolloutFormat::CurrentNativeV2);
    assert_eq!(rollout.session_id.as_deref(), Some("thread-current"));
    assert!(rollout.parse_failures.is_empty());

    let rows = normalize_rollout_file(&rollout);
    let tool_call = rows
        .iter()
        .find(|row| row.kind == CompactionKind::ToolCall)
        .expect("current native tool call");
    assert_eq!(
        tool_call.text,
        "{\"cmd\":\"cargo test -p agent-session-compactor\"}"
    );
    assert_eq!(tool_call.turn_id.as_deref(), Some("turn-current"));
    assert_eq!(
        identity(tool_call),
        json!({
            "call_id": "call-current",
            "name": "shell_command",
            "type": "custom_tool_call",
        })
    );

    let outputs = rows
        .iter()
        .filter(|row| row.kind == CompactionKind::ToolOutput)
        .collect::<Vec<_>>();
    assert_eq!(outputs.len(), 3);
    assert_eq!(
        outputs
            .iter()
            .map(|row| row.row_ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(outputs[0].text, "first segment");
    assert_eq!(outputs[2].text, "second segment");
    let image: Value = serde_json::from_str(&outputs[1].text).expect("typed image projection");
    assert_eq!(
        image,
        json!({
            "detail": "low",
            "image_url": "data:image/png;base64,c2FuaXRpemVk",
            "type": "input_image",
        })
    );
    for (index, segment_type) in ["input_text", "input_image", "input_text"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(outputs[index].turn_id.as_deref(), Some("turn-current"));
        assert_eq!(
            identity(outputs[index]),
            json!({
                "call_id": "call-current",
                "name": "shell_command",
                "segment_index": index,
                "segment_type": segment_type,
                "type": "custom_tool_call_output",
            })
        );
    }
}

#[test]
fn current_native_adapter_maps_direct_thread_delegation_activity_and_agent_message() {
    let rollout = ingest_fixture("delegation-and-agent-message.jsonl");
    let linkage = extract_rollout_linkage_metadata(&rollout);

    assert_eq!(rollout.format, RolloutFormat::CurrentNativeV2);
    assert_eq!(linkage.parent_spawn_results.len(), 1);
    let spawn = &linkage.parent_spawn_results[0];
    assert_eq!(spawn.parent_session_id, "thread-parent");
    assert_eq!(spawn.child_session_id, "thread-child");
    assert_eq!(spawn.call_id, "call-spawn");
    assert_eq!(spawn.spawn_call_provenance.line_number, 2);
    assert_eq!(spawn.spawn_result_provenance.line_number, 3);

    let rows = normalize_rollout_file(&rollout);
    let activity = rows
        .iter()
        .find(|row| row.kind == CompactionKind::Status && row.text.contains("sub_agent_activity"))
        .expect("typed sub-agent activity");
    assert_eq!(
        identity(activity),
        json!({
            "agent_thread_id": "thread-child",
            "event_id": "activity-1",
            "kind": "interacted",
            "type": "sub_agent_activity",
        })
    );

    let delegated_message = rows
        .iter()
        .find(|row| row.text == "Delegated update")
        .expect("typed delegated agent message");
    assert_eq!(delegated_message.kind, CompactionKind::AssistantMessage);
    assert_eq!(delegated_message.turn_id.as_deref(), Some("turn-delegate"));
    assert_eq!(
        identity(delegated_message),
        json!({
            "author": "thread-child",
            "content_types": ["input_text"],
            "recipient": "thread-parent",
            "type": "agent_message",
        })
    );
}

#[test]
fn current_native_adapter_fails_closed_on_malformed_and_unsupported_events() {
    let rollout = ingest_fixture("malformed-and-unsupported.jsonl");

    assert_eq!(rollout.format, RolloutFormat::CurrentNativeV2);
    assert_eq!(rollout.parse_failures.len(), 2);
    assert!(rollout.parse_failures[0]
        .error
        .contains("input is required"));
    assert!(rollout.parse_failures[1]
        .error
        .contains("unsupported segment type"));
    assert_eq!(
        rollout.parse_failures[0].turn_id.as_deref(),
        Some("turn-malformed")
    );
    assert_eq!(
        rollout.parse_failures[1].turn_id.as_deref(),
        Some("turn-malformed")
    );

    let rows = normalize_rollout_file(&rollout);
    let errors = rows
        .iter()
        .filter(|row| row.kind == CompactionKind::Error)
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 2);
    assert!(errors
        .iter()
        .all(|row| row.turn_id.as_deref() == Some("turn-malformed")));
    let unsupported = rows
        .iter()
        .find(|row| row.kind == CompactionKind::Unknown)
        .expect("unsupported native event remains explicit");
    assert_eq!(unsupported.turn_id.as_deref(), Some("turn-malformed"));
    assert!(unsupported.text.contains("future_native_event"));
    assert_eq!(
        unsupported.dedupe_identity.as_deref(),
        Some("current_native:unsupported:event_msg:future_native_event")
    );
}

#[test]
fn current_native_adapter_rejects_unknown_explicit_format_version() {
    let temp_dir = TempDir::new().expect("temp dir");
    let path = temp_dir.path().join("rollout-unsupported-version.jsonl");
    fs::write(
        &path,
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread\",\"multi_agent_version\":\"v3\"}}\n",
    )
    .expect("write unsupported version fixture");
    let path = Utf8Path::from_path(&path).expect("utf8 path");

    let error = ingest_rollout_file(path).expect_err("unknown explicit version must fail closed");
    assert!(matches!(
        error,
        IngestError::UnsupportedFormat { reason, .. }
            if reason.contains("unsupported session_meta.multi_agent_version")
    ));
}

#[test]
fn current_native_adapter_validates_format_markers_across_the_entire_stream() {
    let unsupported = ingest_text(
        "rollout-v2-then-v3.jsonl",
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread\",\"multi_agent_version\":\"v2\"}}\n",
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread\",\"multi_agent_version\":\"v3\"}}\n",
        ),
    )
    .expect_err("a later unsupported marker must fail the whole stream");
    assert!(matches!(
        unsupported,
        IngestError::UnsupportedFormat { reason, .. }
            if reason.contains("unsupported session_meta.multi_agent_version")
    ));

    let conflict = ingest_text(
        "rollout-v1-then-v2.jsonl",
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread\",\"multi_agent_version\":\"v1\"}}\n",
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread\",\"multi_agent_version\":\"v2\"}}\n",
        ),
    )
    .expect_err("conflicting supported markers must fail the whole stream");
    assert!(matches!(
        conflict,
        IngestError::UnsupportedFormat { reason, .. }
            if reason.contains("conflicting rollout format markers")
    ));
}

#[test]
fn malformed_current_native_records_keep_local_turn_scope_through_dedupe() {
    let rollout = ingest_text(
        "rollout-malformed-turns.jsonl",
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"thread\",\"multi_agent_version\":\"v2\"}}\n",
            "{\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-ambient\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"custom_tool_call\",\"name\":\"shell_command\",\"call_id\":\"call-a\",\"internal_chat_message_metadata_passthrough\":{\"turn_id\":\"turn-a\"}}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"custom_tool_call\",\"name\":\"shell_command\",\"call_id\":\"call-b\",\"internal_chat_message_metadata_passthrough\":{\"turn_id\":\"turn-b\"}}}\n",
        ),
    )
    .expect("ingest malformed current-native records");

    assert_eq!(rollout.parse_failures.len(), 2);
    assert_eq!(
        rollout
            .parse_failures
            .iter()
            .map(|failure| failure.turn_id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("turn-a"), Some("turn-b")]
    );

    let rows = normalize_rollout_file(&rollout);
    let errors = rows
        .iter()
        .filter(|row| row.kind == CompactionKind::Error)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        errors
            .iter()
            .map(|row| row.turn_id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("turn-a"), Some("turn-b")]
    );

    let deduped = dedupe_rows_exact(&errors);
    assert_eq!(deduped.archival_rows.len(), 2);
    assert_eq!(deduped.compact_rows.len(), 2);
    assert!(deduped.dedupe_groups.is_empty());
}

#[test]
fn current_native_linkage_requires_a_parent_origin_preceding_spawn_begin() {
    let cases = [
        (
            "end-only",
            "{\"type\":\"event_msg\",\"payload\":{\"type\":\"collab_agent_spawn_end\",\"call_id\":\"call-spawn\",\"sender_thread_id\":\"thread-parent\",\"new_thread_id\":\"thread-child\"}}\n",
        ),
        (
            "wrong-sender",
            concat!(
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"collab_agent_spawn_begin\",\"call_id\":\"call-spawn\",\"sender_thread_id\":\"thread-other\",\"prompt\":\"sanitized\"}}\n",
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"collab_agent_spawn_end\",\"call_id\":\"call-spawn\",\"sender_thread_id\":\"thread-parent\",\"new_thread_id\":\"thread-child\"}}\n",
            ),
        ),
        (
            "begin-after-end",
            concat!(
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"collab_agent_spawn_end\",\"call_id\":\"call-spawn\",\"sender_thread_id\":\"thread-parent\",\"new_thread_id\":\"thread-child\"}}\n",
                "{\"type\":\"event_msg\",\"payload\":{\"type\":\"collab_agent_spawn_begin\",\"call_id\":\"call-spawn\",\"sender_thread_id\":\"thread-parent\",\"prompt\":\"sanitized\"}}\n",
            ),
        ),
    ];

    for (name, events) in cases {
        let text = format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-parent\",\"multi_agent_version\":\"v2\"}}}}\n{events}"
        );
        let rollout = ingest_text(&format!("rollout-{name}.jsonl"), &text)
            .expect("ingest invalid direct linkage witness");
        assert!(
            extract_rollout_linkage_metadata(&rollout)
                .parent_spawn_results
                .is_empty(),
            "{name} must not fabricate a parent spawn result"
        );
    }
}

fn ingest_fixture(name: &str) -> agent_session_compactor::IngestedRolloutFile {
    let path = fixture_path(name);
    ingest_rollout_file(&path).expect("ingest current native fixture")
}

fn fixture_path(name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/current_native")
        .join(name)
}

fn ingest_text(
    name: &str,
    text: &str,
) -> Result<agent_session_compactor::IngestedRolloutFile, IngestError> {
    let temp_dir = TempDir::new().expect("temp dir");
    let path = temp_dir.path().join(name);
    fs::write(&path, text).expect("write current-native witness");
    ingest_rollout_file(Utf8Path::from_path(&path).expect("utf8 path"))
}

fn identity(row: &agent_session_compactor::CompactionRow) -> Value {
    serde_json::from_str(
        row.dedupe_identity
            .as_deref()
            .expect("row should carry explicit native identity"),
    )
    .expect("native identity is canonical JSON")
}
