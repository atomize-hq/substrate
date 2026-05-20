use agent_api_types::{SharedWorldOwnerAction, SharedWorldOwnerSpec};
use serde_json::json;
use substrate_shell::execution::repl_persistent_session::PersistentSessionClientCore;

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
    shared_world: serde_json::Value,
) -> serde_json::Value {
    json!({
        "type": "ready",
        "session_nonce": "0123456789abcdef0123456789abcdef",
        "world_id": "wld_test",
        "cwd": "/",
        "protocol_version": protocol_version,
        "shared_world": shared_world,
    })
}

fn exit_frame(code: i32) -> serde_json::Value {
    json!({
        "type": "exit",
        "code": code,
    })
}

fn stdout_frame(data_b64: &str) -> serde_json::Value {
    json!({
        "type": "stdout",
        "data_b64": data_b64,
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

fn shared_world_binding(
    world_id: &str,
    orchestration_session_id: &str,
    world_generation: u64,
) -> serde_json::Value {
    json!({
        "orchestration_session_id": orchestration_session_id,
        "world_id": world_id,
        "world_generation": world_generation,
        "binding_state": "active",
    })
}

#[test]
fn persistent_session_client_v1_fail_closed_core_contract() {
    // Version mismatch must fail closed.
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    let err = client.on_server_frame(ready_frame(2)).unwrap_err();
    assert!(err.is_fatal());
    assert!(client.on_server_frame(stdout_frame("AA==")).is_err());

    // Ready(version=1) accepted; unknown frames must fail closed.
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1)).unwrap();
    let err = client
        .on_server_frame(json!({"type":"future_frame_type","hello":"world"}))
        .unwrap_err();
    assert!(err.is_fatal());

    // No pipelining: attempting to mark a second exec while one is in-flight is fatal.
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1)).unwrap();
    client
        .note_exec_in_flight(1, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .unwrap();
    assert!(client
        .note_exec_in_flight(2, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        .is_err());

    // (seq, token_hex) mismatches are fatal protocol errors.
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1)).unwrap();

    let awaited = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mismatched = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    client.note_exec_in_flight(1, awaited).unwrap();

    let err = client
        .on_server_frame(command_complete_frame(2, awaited))
        .unwrap_err();
    assert!(err.is_fatal());

    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1)).unwrap();
    client.note_exec_in_flight(1, awaited).unwrap();
    let err = client
        .on_server_frame(command_complete_frame(1, mismatched))
        .unwrap_err();
    assert!(err.is_fatal());
    let msg = err.to_string();
    assert!(!msg.contains(awaited) && !msg.contains(mismatched));

    // Exit is fatal unless shutdown has been initiated.
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1)).unwrap();
    assert!(client.on_server_frame(exit_frame(0)).is_err());

    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent();
    client.on_server_frame(ready_frame(1)).unwrap();
    client.note_shutdown_initiated();
    client.on_server_frame(exit_frame(0)).unwrap();
}

#[test]
fn persistent_session_client_v1_accepts_shared_world_attach_create_ready_proof() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent_with_shared_world(Some(SharedWorldOwnerSpec {
        orchestration_session_id: "orch-test".to_string(),
        action: SharedWorldOwnerAction::AttachOrCreate,
    }));

    client
        .on_server_frame(ready_frame_with_shared_world(
            1,
            shared_world_binding("wld_test", "orch-test", 0),
        ))
        .expect("matching attach/create proof should be accepted");
}

#[test]
fn persistent_session_client_v1_accepts_replacement_ready_when_generation_advances() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent_with_shared_world(Some(SharedWorldOwnerSpec {
        orchestration_session_id: "orch-test".to_string(),
        action: SharedWorldOwnerAction::ReplaceExpectedGeneration {
            expected_generation: 41,
            reason: "test replacement".to_string(),
        },
    }));

    client
        .on_server_frame(ready_frame_with_shared_world(
            1,
            shared_world_binding("wld_test", "orch-test", 42),
        ))
        .expect("replacement proof should require a strictly newer world generation");
}

#[test]
fn persistent_session_client_v1_rejects_invalid_shared_world_ready_proof() {
    let mut client = PersistentSessionClientCore::new();
    client.note_start_session_sent_with_shared_world(Some(SharedWorldOwnerSpec {
        orchestration_session_id: "orch-test".to_string(),
        action: SharedWorldOwnerAction::AttachOrCreate,
    }));

    let err = client
        .on_server_frame(ready_frame_with_shared_world(
            1,
            shared_world_binding("wld_other", "orch-test", 0),
        ))
        .expect_err("world_id mismatch must fail closed");
    assert!(err.is_fatal());
    assert!(err
        .to_string()
        .contains("ready.shared_world.world_id mismatch"));
}
