#![allow(unused_crate_dependencies)]

use std::fs;

use agent_drift_sentinel::{
    LiveSessionCoordinator, LiveSessionError, LiveSessionRequest, SchedulerPolicy, WarningPolicy,
};
use camino::Utf8Path;
use serde_json::{json, Value};
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
fn real_session_live_coordinator_restores_progress_for_restart_idle_decision() {
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
    let first_latest_cursor = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create first coordinator");

        let first = coordinator.poll_once().expect("first poll");
        assert!(first.reran_pipeline);
        assert!(first.emitted_checkpoints > 0);
        let first_latest_cursor = first
            .latest_cursor
            .clone()
            .expect("first poll should establish a cursor");
        let next_emission_ordinal = first
            .observations
            .last()
            .expect("first poll should emit at least one observation")
            .event
            .emission_ordinal
            + 1;
        let persisted_state = read_persisted_state(&state_dir);
        assert_eq!(persisted_state["schema_version"].as_u64(), Some(3));
        assert_eq!(
            persisted_state["root_session_id"].as_str(),
            Some("session-live")
        );
        assert_eq!(
            persisted_state["progress"]["last_observed_size_bytes"].as_u64(),
            Some(first.observed_size_bytes)
        );
        assert!(persisted_state["progress"]["pending_observed_size_bytes"].is_null());
        assert_eq!(
            persisted_state["progress"]["last_delivered_cursors"]["session-live"]["session_id"]
                .as_str(),
            Some("session-live")
        );
        assert_eq!(
            persisted_state["progress"]["last_delivered_cursors"]["session-live"]["ordinal"]
                .as_u64(),
            Some(first_latest_cursor.ordinal as u64)
        );
        assert_eq!(
            persisted_state["progress"]["next_emission_ordinal"].as_u64(),
            Some(next_emission_ordinal as u64)
        );
        first_latest_cursor
    };

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.clone()),
            session_id: "session-live".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create restarted coordinator");

    let resumed_idle = restarted.poll_once().expect("restart poll");
    assert!(!resumed_idle.reran_pipeline);
    assert_eq!(
        resumed_idle.observed_size_bytes as usize,
        first_rollout_phase().len()
    );
    assert_eq!(resumed_idle.emitted_checkpoints, 0);
    assert!(resumed_idle.observations.is_empty());
    assert_eq!(
        resumed_idle.latest_cursor.as_ref(),
        Some(&first_latest_cursor)
    );
}

#[test]
fn real_session_live_coordinator_replays_pending_growth_after_interrupted_poll_restart() {
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
    let first_phase = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create coordinator");

        coordinator.poll_once().expect("first poll")
    };

    let helper_state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("helper-state");
    fs::create_dir_all(&helper_state_dir).expect("create helper state dir");
    fs::copy(
        state_dir.join("live-session-state.json").as_std_path(),
        helper_state_dir
            .join("live-session-state.json")
            .as_std_path(),
    )
    .expect("copy first persisted state");

    let full_rollout = format!(
        "{}{}{}",
        first_rollout_phase(),
        second_rollout_phase(),
        third_rollout_phase()
    );
    fs::write(&rollout_path, &full_rollout).expect("write full rollout");

    let growth_poll = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: helper_state_dir,
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create helper coordinator");

        coordinator.poll_once().expect("growth poll")
    };
    assert!(
        growth_poll.observations.len() >= 2,
        "fixture should create multiple fresh observations for interrupted-poll recovery"
    );

    let partially_delivered = growth_poll
        .observations
        .first()
        .expect("growth poll should emit observations");
    let interrupted_state = json!({
        "schema_version": 2,
        "session_id": "session-live",
        "progress": {
            "last_observed_size_bytes": first_phase.observed_size_bytes,
            "pending_observed_size_bytes": full_rollout.len(),
            "last_delivered_cursor": {
                "session_id": partially_delivered.event.cursor.session_id,
                "ordinal": partially_delivered.event.cursor.ordinal,
            },
            "next_emission_ordinal": partially_delivered.event.emission_ordinal + 1,
        }
    });
    write_persisted_state(&state_dir, &interrupted_state);

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create restarted coordinator");

    let resumed = restarted.poll_once().expect("resumed poll");
    assert!(resumed.reran_pipeline);
    assert_eq!(
        resumed.emitted_checkpoints,
        growth_poll.observations.len() - 1
    );
    assert_eq!(
        resumed
            .observations
            .first()
            .expect("resumed poll should emit remaining observations")
            .event
            .emission_ordinal,
        partially_delivered.event.emission_ordinal + 1
    );
    assert_eq!(resumed.latest_cursor, growth_poll.latest_cursor);

    let persisted_state = read_persisted_state(&state_dir);
    assert_eq!(
        persisted_state["progress"]["last_observed_size_bytes"].as_u64(),
        Some(full_rollout.len() as u64)
    );
    assert!(persisted_state["progress"]["pending_observed_size_bytes"].is_null());
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

    let empty_poll = coordinator
        .poll_once()
        .expect("empty rollout should stay pending");
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
fn real_session_live_coordinator_surfaces_posture_in_live_presentations() {
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
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let first_poll = coordinator.poll_once().expect("first poll");
    let posture_observation = first_poll
        .observations
        .iter()
        .find(|observation| observation.presentation.posture.is_some())
        .expect("real-session live seam should surface posture-bearing presentations");

    assert!(posture_observation
        .presentation
        .render_console_block(None)
        .contains("- Posture: "));
}

#[test]
fn real_session_live_coordinator_upgrades_valid_legacy_schema_v1_state_to_v3_progress() {
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
    let first_latest_cursor = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create coordinator");

        coordinator
            .poll_once()
            .expect("first poll")
            .latest_cursor
            .expect("first poll should establish a cursor")
    };

    let legacy_state = json!({
        "schema_version": 1,
        "session_id": "session-live",
        "last_delivered_cursor": {
            "session_id": first_latest_cursor.session_id,
            "ordinal": first_latest_cursor.ordinal,
        }
    });
    write_persisted_state(&state_dir, &legacy_state);

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create restarted coordinator");
    assert_eq!(restarted.latest_cursor(), Some(&first_latest_cursor));

    let restored = restarted.poll_once().expect("legacy restored poll");
    assert!(restored.reran_pipeline);
    assert_eq!(restored.emitted_checkpoints, 0);
    assert!(restored.observations.is_empty());
    assert_eq!(restored.latest_cursor.as_ref(), Some(&first_latest_cursor));

    let upgraded_state = read_persisted_state(&state_dir);
    assert_eq!(upgraded_state["schema_version"].as_u64(), Some(3));
    assert_eq!(
        upgraded_state["root_session_id"].as_str(),
        Some("session-live")
    );
    assert_eq!(
        upgraded_state["progress"]["last_observed_size_bytes"].as_u64(),
        Some(first_rollout_phase().len() as u64)
    );
    assert!(upgraded_state["progress"]["pending_observed_size_bytes"].is_null());
    assert_eq!(
        upgraded_state["progress"]["last_delivered_cursors"]["session-live"]["session_id"].as_str(),
        Some("session-live")
    );
    assert_eq!(
        upgraded_state["progress"]["last_delivered_cursors"]["session-live"]["ordinal"].as_u64(),
        Some(first_latest_cursor.ordinal as u64)
    );
    assert_eq!(
        upgraded_state["progress"]["next_emission_ordinal"].as_u64(),
        Some(1)
    );
}

#[test]
fn real_session_live_coordinator_rejects_invalid_persisted_cursor_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    fs::create_dir_all(&state_dir).expect("create state dir");
    fs::write(
        state_dir.join("live-session-state.json"),
        "{\"schema_version\":1,\"session_id\":\"other-session\",\"last_delivered_cursor\":{\"session_id\":\"other-session\",\"ordinal\":6}}\n",
    )
    .expect("write invalid state");

    let error = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect_err("mismatched persisted state should fail");

    assert!(error.to_string().contains("persisted live session state"));
}

#[test]
fn real_session_live_coordinator_rejects_persisted_cursor_outside_verified_closure() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    write_persisted_state(
        &state_dir,
        &json!({
            "schema_version": 3,
            "root_session_id": "session-live",
            "progress": {
                "last_observed_size_bytes": null,
                "pending_observed_size_bytes": null,
                "last_delivered_cursors": {
                    "session-unexpected": {
                        "session_id": "session-unexpected",
                        "ordinal": 1
                    }
                },
                "next_emission_ordinal": 2
            }
        }),
    );

    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("persisted cursor is structurally valid before closure validation");

    let error = coordinator
        .poll_once()
        .expect_err("cursor outside analyzer-owned closure should fail closed");
    assert!(matches!(
        error,
        LiveSessionError::UnexpectedCheckpointSessions { .. }
    ));
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

#[test]
fn real_session_live_coordinator_accepts_verified_children_and_advances_each_session_independently()
{
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");

    let root_session_id = "session-live";
    let first_child_session_id = "session-child-a";
    let second_child_session_id = "session-child-b";
    let root_rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    let first_child_rollout_path = rollout_dir.join("rollout-session-child-a.jsonl");
    let second_child_rollout_path = rollout_dir.join("rollout-session-child-b.jsonl");
    fs::write(
        &root_rollout_path,
        linked_root_rollout(
            root_session_id,
            &[first_child_session_id, second_child_session_id],
        ),
    )
    .expect("write linked root rollout");
    fs::write(
        &first_child_rollout_path,
        linked_child_rollout(
            root_session_id,
            first_child_session_id,
            first_rollout_phase(),
        ),
    )
    .expect("write first linked child rollout");
    fs::write(
        &second_child_rollout_path,
        linked_child_rollout(
            root_session_id,
            second_child_session_id,
            first_rollout_phase(),
        ),
    )
    .expect("write second linked child rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: root_session_id.to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create linked coordinator");

    let first = coordinator.poll_once().expect("first linked poll");
    assert!(first.reran_pipeline);
    assert!(first
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == root_session_id));
    assert!(first
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == first_child_session_id));
    assert!(first
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == second_child_session_id));
    let first_root_cursor = first
        .observations
        .iter()
        .rev()
        .find(|observation| observation.event.cursor.session_id == root_session_id)
        .map(|observation| observation.event.cursor.clone())
        .expect("root cursor");
    let first_child_cursor = first
        .observations
        .iter()
        .rev()
        .find(|observation| observation.event.cursor.session_id == first_child_session_id)
        .map(|observation| observation.event.cursor.clone())
        .expect("first child cursor");
    let second_child_cursor = first
        .observations
        .iter()
        .rev()
        .find(|observation| observation.event.cursor.session_id == second_child_session_id)
        .map(|observation| observation.event.cursor.clone())
        .expect("second child cursor");
    assert_eq!(first.latest_cursor.as_ref(), Some(&first_root_cursor));

    fs::write(
        &second_child_rollout_path,
        linked_child_rollout(
            root_session_id,
            second_child_session_id,
            &format!("{}{}", first_rollout_phase(), second_rollout_phase()),
        ),
    )
    .expect("append child-only growth");

    let child_growth = coordinator.poll_once().expect("child growth poll");
    assert!(child_growth.reran_pipeline);
    assert!(!child_growth.observations.is_empty());
    assert!(child_growth.observations.iter().all(|observation| {
        observation.event.cursor.session_id == second_child_session_id
            && observation.event.cursor.ordinal > second_child_cursor.ordinal
    }));
    assert_eq!(
        child_growth.latest_cursor.as_ref(),
        Some(&first_root_cursor)
    );

    let persisted_state = read_persisted_state(&state_dir);
    assert_eq!(persisted_state["schema_version"].as_u64(), Some(3));
    assert_eq!(
        persisted_state["root_session_id"].as_str(),
        Some(root_session_id)
    );
    assert_eq!(
        persisted_state["progress"]["last_delivered_cursors"][root_session_id]["ordinal"].as_u64(),
        Some(first_root_cursor.ordinal as u64)
    );
    assert_eq!(
        persisted_state["progress"]["last_delivered_cursors"][first_child_session_id]["ordinal"]
            .as_u64(),
        Some(first_child_cursor.ordinal as u64)
    );
    assert!(
        persisted_state["progress"]["last_delivered_cursors"][second_child_session_id]["ordinal"]
            .as_u64()
            > Some(second_child_cursor.ordinal as u64)
    );
}

#[test]
fn real_session_live_coordinator_discovers_verified_child_that_appears_after_root_poll() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");

    let root_session_id = "session-live";
    let child_session_id = "session-child-late";
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        linked_root_rollout(root_session_id, &[child_session_id]),
    )
    .expect("write root rollout before child artifact exists");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.clone()),
            session_id: root_session_id.to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let root_only = coordinator.poll_once().expect("root-only poll");
    assert!(root_only.reran_pipeline);
    assert!(root_only
        .observations
        .iter()
        .all(|observation| observation.event.cursor.session_id == root_session_id));
    drop(coordinator);

    fs::write(
        rollout_dir.join("rollout-session-child-late.jsonl"),
        linked_child_rollout(root_session_id, child_session_id, first_rollout_phase()),
    )
    .expect("write late child rollout");

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: root_session_id.to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("restart coordinator while linked closure is pending");
    let child_discovered = restarted.poll_once().expect("late child poll");
    assert!(child_discovered.reran_pipeline);
    assert!(child_discovered
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == child_session_id));
}

fn linked_root_rollout(root_session_id: &str, child_session_ids: &[&str]) -> String {
    let mut rollout = first_rollout_phase().replace("session-live", root_session_id);
    for (index, child_session_id) in child_session_ids.iter().enumerate() {
        rollout.push_str(&format!(
            "{{\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call\",\"name\":\"spawn_agent\",\"call_id\":\"call-spawn-child-{index}\"}}}}\n{{\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call_output\",\"call_id\":\"call-spawn-child-{index}\",\"output\":\"{{\\\"agent_id\\\":\\\"{child_session_id}\\\"}}\"}}}}\n"
        ));
    }
    rollout
}

fn linked_child_rollout(root_session_id: &str, child_session_id: &str, rollout: &str) -> String {
    let source = json!({
        "subagent": {
            "thread_spawn": {
                "parent_thread_id": root_session_id,
                "depth": 1,
                "agent_nickname": "agent-child",
                "agent_role": "default"
            }
        }
    });
    rollout
        .replace("session-live", child_session_id)
        .lines()
        .map(|line| {
            let mut value: Value = serde_json::from_str(line).expect("parse rollout line");
            if value["type"] == "session_meta" {
                value["payload"]["source"] = source.clone();
            }
            format!(
                "{}\n",
                serde_json::to_string(&value).expect("encode rollout line")
            )
        })
        .collect()
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

fn third_rollout_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:14Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_complete\",\"turn_id\":\"turn-2\",\"last_agent_message\":\"Second phase complete\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:15Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-3\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:16Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-3\",\"user_instructions\":\"Keep the restart continuity seam bounded to real_session_live.rs\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:17Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"only fix the flagged review issue in real_session_live.rs and preserve the packet boundary\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:17.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-3\",\"message\":\"only fix the flagged review issue in real_session_live.rs and preserve the packet boundary\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:18Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel real_session_live -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-3\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:19Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-3\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:20Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Third phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:20.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Third phase complete\"}]}}\n"
    )
}

fn read_persisted_state(state_dir: &Utf8Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(state_dir.join("live-session-state.json"))
            .expect("read persisted state"),
    )
    .expect("parse persisted state json")
}

fn write_persisted_state(state_dir: &Utf8Path, state: &Value) {
    fs::create_dir_all(state_dir).expect("create state dir");
    fs::write(
        state_dir.join("live-session-state.json"),
        serde_json::to_vec_pretty(state).expect("encode persisted state"),
    )
    .expect("write persisted state");
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
