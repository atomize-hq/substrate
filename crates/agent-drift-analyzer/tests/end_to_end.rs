#![allow(unused_crate_dependencies)]

mod support;

use std::collections::BTreeMap;
use std::fs;

use agent_drift_analyzer::AnalyzeRequest;
use agent_session_compactor::CompactionRow;
use camino::Utf8PathBuf;
use support::{load_sample_bundle, read_checkpoints, BundleFixture};

#[test]
fn end_to_end_analysis_is_stable_across_reruns() {
    let fixture = BundleFixture::sample();
    let request = agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    };

    let first = agent_drift_analyzer::analyze_bundle(&request).expect("first analysis");
    let first_checkpoints = fs::read_to_string(&first.checkpoints_path).expect("first checkpoints");
    let first_summary = fs::read_to_string(&first.summary_path).expect("first summary");

    let second = agent_drift_analyzer::analyze_bundle(&request).expect("second analysis");
    assert_eq!(
        first_checkpoints,
        fs::read_to_string(&second.checkpoints_path).expect("second checkpoints")
    );
    assert_eq!(
        first_summary,
        fs::read_to_string(&second.summary_path).expect("second summary")
    );

    let checkpoints = read_checkpoints(&second.checkpoints_path);
    assert_eq!(checkpoints[0].schema_version, "v0.8");
    assert_eq!(
        checkpoints[0].delegation,
        agent_drift_analyzer::DelegationContext::default()
    );
    assert!(checkpoints[0].session_archetype.is_some());
    assert!(checkpoints[0].session_progress.is_some());
    assert!(checkpoints[0].turn_context.is_some());
    assert!(!checkpoints[0].expected_next_step.is_empty());
    assert!(first_summary.contains(
        "- Turn-context overview: `turn-001 (#1); checkpoints in turn 1-2; modes verification_heavy -> autonomous`"
    ));
    assert!(first_summary.contains(
        "  turn: `turn-001 (#1) rows=13 checkpoints=2 session-prompts=1 mode=autonomous activity[dir=2 asst=1 tool=8 read=2 write=2 verify=4 out=2]`"
    ));
    assert!(first_summary.contains(
        "  delegation: `topology=single_agent visibility=none confidence=high markers=none support[none] counter[none]`"
    ));
}

#[test]
fn end_to_end_analysis_is_stable_for_many_short_conversational_turns() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
        match row.event_index {
            10 => {
                row.kind = agent_session_compactor::CompactionKind::AssistantMessage;
                row.text = "I am summarizing the next patch step.".to_string();
                row.dedupe_identity = None;
            }
            11 => {
                row.kind = agent_session_compactor::CompactionKind::DeveloperMessage;
                row.text = "Stay inside Packet R3-5 only.".to_string();
                row.dedupe_identity = None;
            }
            12 => {
                row.kind = agent_session_compactor::CompactionKind::AssistantMessage;
                row.text = "Waiting for the next instruction.".to_string();
                row.dedupe_identity = None;
            }
            _ => {}
        }
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let request = AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    };

    let first = agent_drift_analyzer::analyze_bundle(&request).expect("first conversational run");
    let first_checkpoints = fs::read_to_string(&first.checkpoints_path).expect("first checkpoints");
    let first_summary = fs::read_to_string(&first.summary_path).expect("first summary");

    let second = agent_drift_analyzer::analyze_bundle(&request).expect("second conversational run");
    assert_eq!(
        first_checkpoints,
        fs::read_to_string(&second.checkpoints_path).expect("second checkpoints")
    );
    assert_eq!(
        first_summary,
        fs::read_to_string(&second.summary_path).expect("second summary")
    );

    assert!(first_summary.contains("Turns observed: `2`"));
    assert!(first_summary.contains(
        "- Turn-context overview: `turn-001 (#1) -> turn-002 (#2); checkpoints in turn 1; modes verification_heavy -> conversational`"
    ));
    assert!(first_summary.contains(
        "  turn: `turn-002 (#2) rows=4 checkpoints=1 session-prompts=1 mode=conversational activity[dir=1 asst=3 tool=0 read=0 write=0 verify=0 out=0]`"
    ));
}

#[test]
fn end_to_end_summary_surfaces_compact_session_archetype_inspection() {
    let fixture = BundleFixture::sample();
    let request = AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    };

    let result = agent_drift_analyzer::analyze_bundle(&request).expect("analyze sample bundle");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert!(summary.contains("  archetype: `label="));
    for checkpoint in checkpoints {
        let archetype = checkpoint
            .session_archetype
            .as_ref()
            .expect("session archetype");
        assert!(summary.contains(&format!(
            "  archetype: `label={} confidence={}",
            format_label(archetype.label),
            format_confidence(archetype.confidence)
        )));
    }
}

#[test]
fn end_to_end_summary_surfaces_progress_distribution_and_checkpoint_lines() {
    let fixture = BundleFixture::sample();
    let request = AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    };

    let result = agent_drift_analyzer::analyze_bundle(&request).expect("analyze sample bundle");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert!(summary.contains("Progress status distribution: `"));
    assert!(summary.contains("Progress dimension distribution: `"));
    assert!(summary.contains("- Progress status distribution: `"));
    assert!(summary.contains("- Progress dimension distribution: `"));
    assert_eq!(
        summary.matches("  progress: `status=").count(),
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.session_progress.is_some())
            .count()
    );
    assert!(summary.contains("  archetype: `label="));
    assert!(summary.contains("  delegation: `topology="));
    assert!(summary.contains("  turn: `turn-001"));
}

#[test]
fn end_to_end_reruns_preserve_identical_turn_context_and_ordinal_stability() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
        match row.event_index {
            10 => {
                row.kind = agent_session_compactor::CompactionKind::AssistantMessage;
                row.text = "I am summarizing the next patch step.".to_string();
                row.dedupe_identity = None;
            }
            11 => {
                row.kind = agent_session_compactor::CompactionKind::DeveloperMessage;
                row.text = "Stay inside Packet R3-6 only.".to_string();
                row.dedupe_identity = None;
            }
            12 => {
                row.kind = agent_session_compactor::CompactionKind::AssistantMessage;
                row.text = "Waiting for the next instruction.".to_string();
                row.dedupe_identity = None;
            }
            _ => {}
        }
    }

    let mut beta_archival_rows = bundle.archival_rows.clone();
    let mut beta_compact_rows = bundle.compact_rows.clone();
    scope_rows_to_session(
        &mut beta_archival_rows,
        "session-beta",
        "/tmp/session-beta/rollout.jsonl",
    );
    scope_rows_to_session(
        &mut beta_compact_rows,
        "session-beta",
        "/tmp/session-beta/rollout.jsonl",
    );

    let mut archival_rows = bundle.archival_rows;
    archival_rows.extend(beta_archival_rows);
    let mut compact_rows = bundle.compact_rows;
    compact_rows.extend(beta_compact_rows);

    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, bundle.dedupe_groups);
    let request = AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    };

    let first = agent_drift_analyzer::analyze_bundle(&request).expect("first conversational run");
    let second = agent_drift_analyzer::analyze_bundle(&request).expect("second conversational run");

    let first_checkpoints = read_checkpoints(&first.checkpoints_path);
    let second_checkpoints = read_checkpoints(&second.checkpoints_path);

    assert_eq!(first_checkpoints.len(), 4);
    let first_artifact_shape = first_checkpoints
        .iter()
        .map(|checkpoint| {
            (
                checkpoint.session_id.clone(),
                checkpoint.ordinal,
                checkpoint.turn_context.clone(),
            )
        })
        .collect::<Vec<_>>();
    let second_artifact_shape = second_checkpoints
        .iter()
        .map(|checkpoint| {
            (
                checkpoint.session_id.clone(),
                checkpoint.ordinal,
                checkpoint.turn_context.clone(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(first_artifact_shape, second_artifact_shape);
    assert_eq!(
        session_ordinals(&first_checkpoints),
        BTreeMap::from([
            ("session-alpha".to_string(), vec![1, 2]),
            ("session-beta".to_string(), vec![1, 2]),
        ])
    );
    for session_id in ["session-alpha", "session-beta"] {
        let session_checkpoints = first_checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.session_id == session_id)
            .collect::<Vec<_>>();
        assert_eq!(session_checkpoints.len(), 2);
        assert_eq!(
            session_checkpoints[1]
                .turn_context
                .as_ref()
                .and_then(|turn| turn.turn_id.as_deref()),
            Some("turn-002")
        );
        assert_eq!(
            session_checkpoints[1]
                .turn_context
                .as_ref()
                .map(|turn| turn.turn_ordinal),
            Some(2)
        );
    }
}

fn scope_rows_to_session(rows: &mut [CompactionRow], session_id: &str, source_file: &str) {
    for row in rows {
        row.session_id = Some(session_id.to_string());
        row.source_file = Utf8PathBuf::from(source_file);
    }
}

fn session_ordinals(
    checkpoints: &[agent_drift_analyzer::Checkpoint],
) -> BTreeMap<String, Vec<usize>> {
    let mut ordinals = BTreeMap::new();
    for checkpoint in checkpoints {
        ordinals
            .entry(checkpoint.session_id.clone())
            .or_insert_with(Vec::new)
            .push(checkpoint.ordinal);
    }
    ordinals
}

fn format_label(label: agent_drift_analyzer::SessionArchetypeLabel) -> &'static str {
    match label {
        agent_drift_analyzer::SessionArchetypeLabel::Troubleshooting => "troubleshooting",
        agent_drift_analyzer::SessionArchetypeLabel::Planning => "planning",
        agent_drift_analyzer::SessionArchetypeLabel::AutonomousImplementation => {
            "autonomous_implementation"
        }
        agent_drift_analyzer::SessionArchetypeLabel::VerificationCloseout => {
            "verification_closeout"
        }
    }
}

fn format_confidence(confidence: agent_drift_analyzer::Confidence) -> &'static str {
    match confidence {
        agent_drift_analyzer::Confidence::Low => "low",
        agent_drift_analyzer::Confidence::Medium => "medium",
        agent_drift_analyzer::Confidence::High => "high",
    }
}
