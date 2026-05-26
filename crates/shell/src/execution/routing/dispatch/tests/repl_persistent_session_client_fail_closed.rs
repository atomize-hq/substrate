//! C2-test: fail-closed host-side persistent session client protocol handling.
//!
//! Reference:
//! - docs/internals/repl/persistent_session.md

use serde_json::json;
use world_api::{
    SharedWorldBindingSnapshot, SharedWorldBindingState, SharedWorldOwnerAction,
    SharedWorldOwnerSpec,
};

use crate::execution::repl_persistent_session::{
    PersistentSessionClientCore, PersistentSessionProtocolError,
};

fn ready_frame(protocol_version: u32) -> serde_json::Value {
    json!({
        "type": "ready",
        "session_nonce": "0123456789abcdef0123456789abcdef",
        "world_id": "wld_test",
        "cwd": "/",
        "protocol_version": protocol_version,
    })
}

fn ready_frame_with_shared_world(
    protocol_version: u32,
    shared_world: Option<SharedWorldBindingSnapshot>,
) -> serde_json::Value {
    let mut frame = ready_frame(protocol_version);
    if let Some(shared_world) = shared_world {
        frame["shared_world"] = serde_json::to_value(shared_world).expect("serialize shared_world");
    }
    frame
}

fn exit_frame(code: i32) -> serde_json::Value {
    json!({
        "type": "exit",
        "code": code,
    })
}

fn command_complete_frame(seq: u64, token_hex: &str) -> serde_json::Value {
    json!({
        "type": "command_complete",
        "seq": seq,
        "token_hex": token_hex,
        "exit": 0,
        "cwd": "/",
    })
}

#[test]
fn ready_protocol_version_mismatch_is_fatal_and_latches_fail_closed() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();

    let err = client
        .on_server_frame(ready_frame(2))
        .expect_err("ready.protocol_version != 1 must fail closed");
    assert!(
        err.is_fatal(),
        "protocol mismatch should be fatal (fail-closed): {err:#}"
    );

    let followup = client.on_server_frame(json!({"type":"stdout","data_b64":""}));
    assert!(
        followup.is_err(),
        "client must remain fail-closed after fatal protocol error"
    );
}

#[test]
fn unknown_server_frame_type_is_fatal_protocol_error() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();

    client
        .on_server_frame(ready_frame(1))
        .expect("ready(protocol_version=1) accepted");

    let err = client
        .on_server_frame(json!({"type":"future_frame_type","hello":"world"}))
        .expect_err("unknown frame types must fail closed under v1");
    assert!(err.is_fatal(), "unknown frame must be fatal: {err:#}");
}

#[test]
fn command_complete_seq_mismatch_is_fatal_protocol_error() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client
        .on_server_frame(ready_frame(1))
        .expect("ready accepted");

    client
        .note_exec_in_flight(1, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .expect("mark exec in-flight");

    let err = client
        .on_server_frame(command_complete_frame(
            2,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ))
        .expect_err("seq mismatch must fail closed");
    assert!(err.is_fatal(), "seq mismatch must be fatal: {err:#}");
}

#[test]
fn command_complete_token_mismatch_is_fatal_and_redacts_token() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client
        .on_server_frame(ready_frame(1))
        .expect("ready accepted");

    let awaited = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mismatched = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    client
        .note_exec_in_flight(1, awaited)
        .expect("mark exec in-flight");

    let err = client
        .on_server_frame(command_complete_frame(1, mismatched))
        .expect_err("token mismatch must fail closed");
    assert!(err.is_fatal(), "token mismatch must be fatal: {err:#}");

    let msg = err.to_string();
    assert!(
        !msg.contains(awaited) && !msg.contains(mismatched),
        "error messages must not include full token_hex (R-019): {msg}"
    );
}

#[test]
fn unexpected_exit_is_fatal_protocol_error() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client
        .on_server_frame(ready_frame(1))
        .expect("ready accepted");

    let err = client
        .on_server_frame(exit_frame(0))
        .expect_err("unexpected exit must fail closed");
    assert!(err.is_fatal(), "unexpected exit must be fatal: {err:#}");
}

#[test]
fn exit_after_shutdown_is_expected_and_not_an_error() -> Result<(), PersistentSessionProtocolError>
{
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1))?;

    client.note_shutdown_initiated();
    client.on_server_frame(exit_frame(0))?;
    Ok(())
}

#[test]
fn ready_missing_world_id_is_fatal_protocol_error() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();

    let err = client
        .on_server_frame(json!({
            "type": "ready",
            "session_nonce": "0123456789abcdef0123456789abcdef",
            "cwd": "/",
            "protocol_version": 1,
        }))
        .expect_err("ready.world_id is required");
    assert!(err.is_fatal(), "missing world_id must be fatal: {err:#}");
}

#[test]
fn ready_empty_world_id_is_fatal_protocol_error() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();

    let err = client
        .on_server_frame(json!({
            "type": "ready",
            "session_nonce": "0123456789abcdef0123456789abcdef",
            "world_id": "",
            "cwd": "/",
            "protocol_version": 1,
        }))
        .expect_err("ready.world_id must be non-empty");
    assert!(err.is_fatal(), "empty world_id must be fatal: {err:#}");
}

#[test]
fn ready_missing_shared_world_for_explicit_owner_request_is_fatal() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent_with_shared_world(Some(SharedWorldOwnerSpec {
        orchestration_session_id: "orch-test".to_string(),
        action: SharedWorldOwnerAction::AttachOrCreate,
    }));

    let err = client
        .on_server_frame(ready_frame_with_shared_world(1, None))
        .expect_err("explicit owner-bound start_session must require echoed proof");
    assert!(
        err.is_fatal(),
        "missing shared_world must be fatal: {err:#}"
    );
}

#[test]
fn ready_unexpected_shared_world_for_generic_request_is_fatal() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();

    let err = client
        .on_server_frame(ready_frame_with_shared_world(
            1,
            Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch-test".to_string(),
                world_id: "wld_test".to_string(),
                world_generation: 0,
                binding_state: SharedWorldBindingState::Active,
            }),
        ))
        .expect_err("generic start_session must reject unexpected shared_world proof");
    assert!(
        err.is_fatal(),
        "unexpected shared_world must be fatal: {err:#}"
    );
}

#[test]
fn ready_replace_expected_generation_requires_advanced_generation() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent_with_shared_world(Some(SharedWorldOwnerSpec {
        orchestration_session_id: "orch-test".to_string(),
        action: SharedWorldOwnerAction::ReplaceExpectedGeneration {
            expected_generation: 4,
            reason: "restart".to_string(),
        },
    }));

    let err = client
        .on_server_frame(ready_frame_with_shared_world(
            1,
            Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch-test".to_string(),
                world_id: "wld_test".to_string(),
                world_generation: 4,
                binding_state: SharedWorldBindingState::Active,
            }),
        ))
        .expect_err("replace requests must reject stale echoed generations");
    assert!(err.is_fatal(), "stale generation must be fatal: {err:#}");
}
