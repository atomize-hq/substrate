#![allow(unused_crate_dependencies)]

use std::fs;

use agent_drift_sentinel::{
    LiveSessionCoordinator, LiveSessionError, LiveSessionRequest, SchedulerPolicy, WarningPolicy,
};
use camino::Utf8Path;
use tempfile::TempDir;

#[test]
fn real_session_live_coordinator_emits_only_checkpoint_deltas_for_append_only_growth() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, first_rollout_phase()).expect("write first phase");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.clone()),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let first = coordinator.poll_once().expect("first poll");
    assert!(first.reran_pipeline);
    assert!(!first.observations.is_empty());
    let first_latest_cursor = first
        .latest_cursor
        .clone()
        .expect("first poll should establish a cursor");
    assert_eq!(first_latest_cursor.session_id, "session-live");

    let second = coordinator.poll_once().expect("idle poll");
    assert!(!second.reran_pipeline);
    assert!(second.observations.is_empty());
    assert_eq!(second.latest_cursor.as_ref(), Some(&first_latest_cursor));

    fs::write(
        &rollout_path,
        format!("{}{}", first_rollout_phase(), second_rollout_phase()),
    )
    .expect("append second phase");

    let third = coordinator.poll_once().expect("growth poll");
    assert!(third.reran_pipeline);
    assert!(!third.observations.is_empty());
    assert!(third
        .observations
        .iter()
        .all(|observation| observation.event.cursor.session_id == "session-live"));
    assert!(third
        .observations
        .iter()
        .all(|observation| { observation.event.cursor.ordinal > first_latest_cursor.ordinal }));
    assert_eq!(
        coordinator.latest_cursor(),
        third
            .observations
            .last()
            .map(|observation| &observation.event.cursor)
    );
}

#[test]
fn real_session_live_coordinator_keeps_polling_through_sparse_startup_until_analyzable() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, "").expect("write empty rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let empty_poll = coordinator.poll_once().expect("empty rollout should stay pending");
    assert!(empty_poll.reran_pipeline);
    assert_eq!(empty_poll.emitted_checkpoints, 0);
    assert!(empty_poll.observations.is_empty());
    assert!(empty_poll.latest_cursor.is_none());
    assert_eq!(empty_poll.observed_size_bytes, 0);

    fs::write(&rollout_path, session_meta_only_rollout_phase()).expect("write session meta");

    let session_meta_poll = coordinator
        .poll_once()
        .expect("session_meta-only rollout should stay pending");
    assert!(session_meta_poll.reran_pipeline);
    assert_eq!(session_meta_poll.emitted_checkpoints, 0);
    assert!(session_meta_poll.observations.is_empty());
    assert!(session_meta_poll.latest_cursor.is_none());

    fs::write(
        &rollout_path,
        format!(
            "{}{}",
            session_meta_only_rollout_phase(),
            initial_analyzable_growth_phase()
        ),
    )
    .expect("append analyzable growth");

    let analyzable_poll = coordinator
        .poll_once()
        .expect("analyzable rollout should emit checkpoints");
    assert!(analyzable_poll.reran_pipeline);
    assert!(!analyzable_poll.observations.is_empty());
    assert!(analyzable_poll
        .latest_cursor
        .as_ref()
        .is_some_and(|cursor| cursor.session_id == "session-live"));
}

#[test]
fn real_session_live_coordinator_rejects_ambiguous_rollout_artifacts() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-live-a.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout a");
    fs::write(
        rollout_dir.join("rollout-session-live-b.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout b");

    let error = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir: Utf8Path::from_path(temp_dir.path())
                .expect("utf8 temp dir")
                .join("state"),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect_err("ambiguous rollout artifacts should fail");

    assert!(matches!(
        error,
        LiveSessionError::AmbiguousRolloutArtifacts { .. }
    ));
}

fn first_rollout_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:02Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-1\",\"user_instructions\":\"Repo-local rules\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-1\",\"message\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:05Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n"
    )
}

fn second_rollout_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:07Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_complete\",\"turn_id\":\"turn-1\",\"last_agent_message\":\"First phase complete\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:08Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-2\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:09Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-2\",\"user_instructions\":\"Follow the packet acceptance criteria exactly\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:10Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"also prove the live command on the active session and update docs/specs/hybrid-drift-sentinel-implementation-order.md\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:10.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-2\",\"message\":\"also prove the live command on the active session and update docs/specs/hybrid-drift-sentinel-implementation-order.md\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:11Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel live_runtime -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-2\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:12Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-2\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:13Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Second phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:13.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Second phase complete\"}]}}\n"
    )
}

fn session_meta_only_rollout_phase() -> &'static str {
    "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n"
}

fn initial_analyzable_growth_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:02Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-1\",\"user_instructions\":\"Repo-local rules\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-1\",\"message\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:05Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n"
    )
}
