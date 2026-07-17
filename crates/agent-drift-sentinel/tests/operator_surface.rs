#![allow(unused_crate_dependencies)]

mod support;

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use agent_drift_analyzer::{
    Checkpoint, ChildWorkVisibility, Confidence, DelegationContext, DelegationTopology, DriftClass,
    DriftState, EvidenceRef, ProgressDimension, ProgressStatus, SessionArchetype,
    SessionArchetypeLabel, SessionProgress, TurnActivityMix, TurnContext, TurnExecutionMode,
};
use agent_drift_sentinel::{
    adjudication::{shape_request, AdjudicationRequest},
    execute,
    operator_surface::{
        present_checkpoint, present_checkpoint_with_previous, render_replay_report,
        warning_fingerprint, CheckpointPosture, CheckpointPresentation, ReplayReport,
    },
    scheduler::{DecisionReason, EvaluationDecision, ReplayScheduler},
    AdjudicationConfig, InputError, ReasoningEffort, ReplayCheckpointBundle, SchedulerPolicy,
    SentinelError, SentinelMode, SentinelRequest, SentinelResult, TriggerClass, WarningPolicy,
};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};
use syn::visit::{self, Visit};

#[test]
fn operator_surface_locks_replay_public_signatures() {
    let _: fn(&SentinelRequest) -> Result<SentinelResult, SentinelError> = execute;
    let _: fn(
        &ReplayCheckpointBundle,
        &[Checkpoint],
        &SchedulerPolicy,
        &WarningPolicy,
    ) -> ReplayReport = render_replay_report;
}

#[test]
fn operator_surface_locks_all_legacy_presentation_facade_signatures() {
    let _: fn(
        &Checkpoint,
        TriggerClass,
        &EvaluationDecision,
        &WarningPolicy,
    ) -> CheckpointPresentation = present_checkpoint;
    let _: fn(
        &Checkpoint,
        Option<&Checkpoint>,
        TriggerClass,
        &EvaluationDecision,
        &WarningPolicy,
    ) -> CheckpointPresentation = present_checkpoint_with_previous;
    let _: fn(&CheckpointPresentation, Option<&str>) -> String =
        CheckpointPresentation::render_console_block;
}

#[test]
fn legacy_facade_zero_evidence_limit_stops_after_an_empty_first_selected_group() {
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let policy = WarningPolicy {
        max_evidence_lines: 0,
        ..WarningPolicy::default()
    };

    let actual = ["v0.2", "v0.3"]
        .into_iter()
        .map(|schema_version| {
            let checkpoint = checkpoint_with_selected_evidence_groups(
                schema_version,
                &[],
                &["later selected evidence must remain hidden"],
            );
            let presentation = present_checkpoint(
                &checkpoint,
                TriggerClass::CheckpointReady,
                &decision,
                &policy,
            );
            (schema_version, presentation.evidence_lines)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![("v0.2", Vec::new()), ("v0.3", Vec::new())],
        "legacy and explicit-state facade paths must preserve the pre-R8 first-group stop"
    );
}

#[test]
fn legacy_facade_zero_evidence_limit_keeps_one_item_from_a_nonempty_first_selected_group() {
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let policy = WarningPolicy {
        max_evidence_lines: 0,
        ..WarningPolicy::default()
    };

    let actual = ["v0.2", "v0.3"]
        .into_iter()
        .map(|schema_version| {
            let checkpoint = checkpoint_with_selected_evidence_groups(
                schema_version,
                &["first selected evidence survives the zero limit"],
                &["later selected evidence stays hidden"],
            );
            let presentation = present_checkpoint(
                &checkpoint,
                TriggerClass::CheckpointReady,
                &decision,
                &policy,
            );
            (schema_version, presentation.evidence_lines)
        })
        .collect::<Vec<_>>();

    for (schema_version, evidence_lines) in actual {
        assert_eq!(
            evidence_lines.len(),
            1,
            "{schema_version} must preserve the pre-R8 zero-limit first-item behavior"
        );
        assert!(evidence_lines[0].contains("first selected evidence survives the zero limit"));
        assert!(!evidence_lines[0].contains("later selected evidence stays hidden"));
    }
}

#[test]
fn legacy_facade_nonzero_evidence_limit_preserves_selected_group_order_and_boundary() {
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let policy = WarningPolicy {
        max_evidence_lines: 3,
        ..WarningPolicy::default()
    };

    for schema_version in ["v0.2", "v0.3"] {
        let checkpoint = checkpoint_with_selected_evidence_groups(
            schema_version,
            &["first selected evidence one", "first selected evidence two"],
            &["later selected evidence one", "later selected evidence two"],
        );
        let evidence_lines = present_checkpoint(
            &checkpoint,
            TriggerClass::CheckpointReady,
            &decision,
            &policy,
        )
        .evidence_lines;

        assert_eq!(
            evidence_lines.len(),
            3,
            "{schema_version} must apply the nonzero limit across selected groups"
        );
        for (index, expected) in [
            "first selected evidence one",
            "first selected evidence two",
            "later selected evidence one",
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                evidence_lines[index].contains(expected),
                "{schema_version} evidence line {index} must preserve selected-group ordering: {evidence_lines:?}"
            );
        }
        assert!(
            evidence_lines
                .iter()
                .all(|line| !line.contains("later selected evidence two")),
            "{schema_version} must stop at the selected-group boundary after reaching the limit"
        );
    }
}

#[test]
fn legacy_facade_preserves_delegation_presence_and_unsupported_total_output() {
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let policy = WarningPolicy::default();
    let delegation = DelegationContext {
        topology: DelegationTopology::MixedOrAmbiguous,
        parent_session_id: Some("parent-session".to_string()),
        child_session_ids: vec!["child-a".to_string(), "child-b".to_string()],
        child_work_visibility: ChildWorkVisibility::Opaque,
        confidence: Confidence::Low,
        markers: vec!["typed marker".to_string()],
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    };

    for version in ["v0.2", "v0.3", "v0.4", "v0.5", "v0.6", "v0.7"] {
        let mut checkpoint = support::checkpoint(
            "session-delegation",
            1,
            0,
            false,
            "continue on the current task frame",
        );
        checkpoint.schema_version = version.to_string();
        checkpoint.delegation = delegation.clone();
        let rendered = present_checkpoint(
            &checkpoint,
            TriggerClass::CheckpointReady,
            &decision,
            &policy,
        )
        .render_console_block(None);
        assert!(
            !rendered.contains("- Delegation:"),
            "{version} must preserve absent delegation presentation"
        );
    }

    let mut v0_8 = support::checkpoint(
        "session-delegation",
        1,
        0,
        false,
        "continue on the current task frame",
    );
    v0_8.schema_version = "v0.8".to_string();
    v0_8.delegation = delegation;
    let v0_8_rendered =
        present_checkpoint(&v0_8, TriggerClass::CheckpointReady, &decision, &policy)
            .render_console_block(None);
    assert!(v0_8_rendered.contains(
        "- Delegation: topology=mixed_or_ambiguous parent=parent-session children=[child-a,child-b] visibility=opaque confidence=low"
    ));

    let mut unsupported = v0_8;
    unsupported.schema_version = "v-next".to_string();
    unsupported.drift_scores[0].class = DriftClass::TruthGroundingGap;
    unsupported.drift_scores[0].state = DriftState::Active;
    unsupported.flagged = false;
    unsupported.drift_scores[0].flagged = false;
    unsupported.drift_scores[0].evidence = vec![EvidenceRef {
        row: unsupported.boundary.start.clone(),
        reason: "historical truth-grounding gap: facade-compatible evidence".to_string(),
    }];
    let mut legacy_equivalent = unsupported.clone();
    legacy_equivalent.schema_version = "v0.2".to_string();
    let legacy_rendered = present_checkpoint(
        &legacy_equivalent,
        TriggerClass::CheckpointReady,
        &decision,
        &policy,
    )
    .render_console_block(None);
    let unsupported_rendered = present_checkpoint(
        &unsupported,
        TriggerClass::CheckpointReady,
        &decision,
        &policy,
    )
    .render_console_block(None);
    assert!(legacy_rendered.contains("- Posture: historical-only"));
    assert!(legacy_rendered.contains(
        "- Evidence: session-delegation.jsonl#1:0 historical truth-grounding gap: facade-compatible evidence"
    ));
    assert_eq!(unsupported_rendered, legacy_rendered);
}

#[test]
fn legacy_facade_is_total_and_deterministic_for_supported_core_invalid_typed_shapes() {
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let policy = WarningPolicy::default();

    let mut empty_required_strings = support::checkpoint(
        "session-empty",
        1,
        0,
        false,
        "continue on the current task frame",
    );
    empty_required_strings.schema_version = "v0.8".to_string();
    empty_required_strings.session_id.clear();
    empty_required_strings.checkpoint_id.clear();
    empty_required_strings.task_frame.objective.clear();
    empty_required_strings.expected_next_step.clear();

    let mut missing_version_optionals = support::checkpoint(
        "session-missing",
        1,
        0,
        false,
        "continue on the current task frame",
    );
    missing_version_optionals.schema_version = "v0.8".to_string();
    missing_version_optionals.turn_context = None;
    missing_version_optionals.session_archetype = None;
    missing_version_optionals.session_progress = None;

    for checkpoint in [empty_required_strings, missing_version_optionals] {
        let first = present_checkpoint(
            &checkpoint,
            TriggerClass::CheckpointReady,
            &decision,
            &policy,
        );
        let second = present_checkpoint(
            &checkpoint,
            TriggerClass::CheckpointReady,
            &decision,
            &policy,
        );

        assert_eq!(first, second, "the public facade must be deterministic");
        assert_eq!(
            first.render_console_block(None),
            second.render_console_block(None),
            "the public facade must produce deterministic bytes"
        );
    }
}

#[test]
fn legacy_facade_ignores_invalid_history_relationships_without_changing_exact_output() {
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let policy = WarningPolicy::default();

    let mut checkpoint = support::checkpoint(
        "session-history",
        2,
        0,
        false,
        "continue on the current task frame",
    );
    checkpoint.drift_scores[0].class = DriftClass::DeadEndThrash;
    checkpoint.drift_scores[0].evidence = vec![EvidenceRef {
        row: checkpoint.boundary.start.clone(),
        reason: "historical repeated failure evidence: facade-compatible history".to_string(),
    }];

    let mut cross_session = support::checkpoint(
        "other-session",
        1,
        80,
        true,
        "continue on the current task frame",
    );
    cross_session.drift_scores[0].class = DriftClass::DeadEndThrash;

    let mut mismatched_schema = support::checkpoint(
        "session-history",
        1,
        80,
        true,
        "continue on the current task frame",
    );
    mismatched_schema.schema_version = "v0.3".to_string();
    mismatched_schema.drift_scores[0].class = DriftClass::DeadEndThrash;

    let expected = concat!(
        "[checkpoint] session-history:0002 @ checkpoint_ready (low)\n",
        "- Objective: /goal Complete replay validation for session-history checkpoint 2\n",
        "- Drift: no flagged drift classes\n",
        "- Posture: recovered\n",
        "- Diagnostics: task_frame_transitioned=false, working_set_changed=false, verification=1/1 (100.00%), evidence_items=1\n",
        "- Expected next step: continue on the current task frame\n",
        "- Evidence: session-history.jsonl#2:0 historical repeated failure evidence: facade-compatible history\n",
        "- Silent reason: checkpoint recorded without a visible warning"
    );

    for previous in [&cross_session, &mismatched_schema] {
        let render = || {
            present_checkpoint_with_previous(
                &checkpoint,
                Some(previous),
                TriggerClass::CheckpointReady,
                &decision,
                &policy,
            )
            .render_console_block(None)
        };

        assert_eq!(render(), expected);
        assert_eq!(render(), expected, "the total facade must be deterministic");
    }
}

#[test]
fn adjudication_shape_request_preserves_exact_legacy_facade_bytes_and_fields() {
    let mut checkpoint = support::checkpoint(
        "session-adjudication",
        1,
        0,
        false,
        "continue on the current task frame",
    );
    checkpoint.schema_version = "v0.8".to_string();
    checkpoint.delegation = DelegationContext {
        topology: DelegationTopology::MixedOrAmbiguous,
        parent_session_id: Some("parent".to_string()),
        child_session_ids: vec!["child-a".to_string()],
        child_work_visibility: ChildWorkVisibility::Opaque,
        confidence: Confidence::Low,
        markers: Vec::new(),
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    };
    let decision = EvaluationDecision {
        evaluate: true,
        visible_warning_allowed: true,
        reason: DecisionReason::InitialCheckpoint,
    };
    let presentation = present_checkpoint(
        &checkpoint,
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );
    let config = AdjudicationConfig {
        enabled: true,
        model: "gpt-5.4-mini".to_string(),
        reasoning_effort: ReasoningEffort::Medium,
        max_evidence_items: 3,
        max_context_chars: usize::MAX,
    };
    let expected_summary = concat!(
        "[checkpoint] session-adjudication:0001 @ checkpoint_ready (low)\n",
        "- Objective: /goal Complete replay validation for session-adjudication checkpoint 1\n",
        "- Drift: no flagged drift classes\n",
        "- Delegation: topology=mixed_or_ambiguous parent=parent children=[child-a] visibility=opaque confidence=low\n",
        "- Diagnostics: task_frame_transitioned=true, working_set_changed=false, verification=1/1 (100.00%), evidence_items=1\n",
        "- Expected next step: continue on the current task frame\n",
        "- Silent reason: checkpoint recorded without a visible warning"
    );
    let expected = AdjudicationRequest {
        model: "gpt-5.4-mini".to_string(),
        reasoning_effort: "medium".to_string(),
        checkpoint_id: "session-adjudication:0001".to_string(),
        session_id: "session-adjudication".to_string(),
        operator_summary: expected_summary.to_string(),
        expected_next_step: "continue on the current task frame".to_string(),
        evidence: Vec::new(),
    };

    assert_eq!(presentation.render_console_block(None), expected_summary);
    assert_eq!(shape_request(&presentation, &config), Some(expected));
}

#[test]
fn replay_infallible_facade_preserves_successful_report_behavior() {
    let fixture = support::ReplayFixture::sample();
    let request = SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    };
    let result = execute(&request).expect("run fallible replay core");
    let checkpoints = result.bundle.checkpoints_after(None);

    let compatibility_report = render_replay_report(
        &result.bundle,
        &checkpoints,
        &request.scheduler_policy,
        &request.warning_policy,
    );

    assert_eq!(compatibility_report, result.report);
}

#[test]
fn replay_failure_rejects_the_complete_set_before_returning_any_effects() {
    let mut checkpoints = support::sample_checkpoints();
    checkpoints[1].expected_next_step = "".to_string();
    let invalid_checkpoint_id = checkpoints[1].checkpoint_id.clone();
    let fixture = support::ReplayFixture::from_checkpoints(checkpoints, support::sample_summary());

    let error = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect_err("one invalid selected checkpoint must reject the entire replay result");

    assert!(matches!(
        error,
        SentinelError::Input(InputError::ContractGap {
            line_number: 2,
            ref schema_version,
            ref field,
            ref reason,
            ..
        }) if schema_version == "v0.2"
            && field == "expected_next_step"
            && reason.contains("non-empty string")
    ));
    assert!(error.to_string().contains(&invalid_checkpoint_id));
}

#[test]
fn replay_fallible_core_interprets_the_complete_set_before_scheduler_construction() {
    let source = include_str!("../src/operator_surface.rs");
    let core_start = source
        .find("pub(crate) fn try_render_replay_report")
        .expect("fallible replay core");
    let core = &source[core_start..];
    let interpretation = core
        .find("interpret_checkpoint")
        .expect("typed interpretation in replay core");
    let scheduler = core
        .find("ReplayScheduler::new")
        .expect("scheduler construction in replay core");
    let presentation = core
        .find("present_interpretation")
        .expect("typed presentation in replay core");

    assert!(interpretation < scheduler);
    assert!(scheduler < presentation);
    assert!(!core[..presentation].contains("present_checkpoint_with_previous"));

    let execute_source = include_str!("../src/lib.rs");
    assert!(execute_source.contains("operator_surface::try_render_replay_report("));
    assert!(!execute_source.contains("operator_surface::render_replay_report("));
}

#[test]
fn operator_surface_renders_evidence_backed_visible_warning_blocks() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("Agent Drift Sentinel Replay"));
    assert!(rendered.contains("Visible warnings:"));
    assert!(rendered.contains(
        "Diagnostics: task_frame_transitioned=true, working_set_changed=false, verification=1/1 (100.00%), evidence_items=2"
    ));
    assert!(rendered.contains("Expected next step: align plan to repo truth"));
    assert!(
        rendered.contains("Evidence: session-alpha.jsonl#1:0 flagged score for session-alpha:1")
    );
}

#[test]
fn operator_surface_renders_diagnostics_for_silent_checkpoints() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("Silent checkpoints:"));
    assert!(rendered.contains(
        "Diagnostics: task_frame_transitioned=false, working_set_changed=false, verification=1/1 (100.00%), evidence_items=2"
    ));
    assert!(rendered.contains("Silent reason: scheduler cooldown deferred replay evaluation"));
}

#[test]
fn operator_surface_replay_flagged_checkpoints_keep_checkpoint_ready_headlines() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let visible = &result.report.visible_warnings[0];

    assert!(visible.checkpoint.flagged);
    assert!(visible.headline.contains("checkpoint_ready"));
    assert!(!visible
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(visible.posture, Some(CheckpointPosture::Active));
}

#[test]
fn operator_surface_renders_unavailable_density_for_zero_command_checkpoints() {
    let mut checkpoints = support::sample_checkpoints();
    checkpoints[0].diagnostics.interval_command_count = 0;
    checkpoints[0]
        .diagnostics
        .interval_verification_command_count = 0;
    let fixture = support::ReplayFixture::from_checkpoints(checkpoints, support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains(
        "Diagnostics: task_frame_transitioned=true, working_set_changed=false, verification=0/0 (unavailable), evidence_items=2"
    ));
}

#[test]
fn operator_surface_renders_compact_turn_context_for_v0_4_checkpoints() {
    let mut visible = checkpoint_with_drift(
        "session-turn-context",
        1,
        DriftClass::TruthGroundingGap,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-turn-context:1"],
    );
    visible.schema_version = "v0.4".to_string();
    visible.turn_context = Some(sample_turn_context(1));

    let mut silent = checkpoint_with_drift(
        "session-turn-context",
        2,
        DriftClass::TruthGroundingGap,
        20,
        false,
        "continue on the current task frame",
        &["historical truth-grounding gap: flagged score for session-turn-context:1"],
    );
    silent.schema_version = "v0.4".to_string();
    silent.turn_context = Some(sample_turn_context(1));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![visible, silent], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains(
        "- Turn context: turn-1 (#1) rows=4 elapsed=12s checkpoints=2 session-prompts=1 mode=autonomous activity[dir=1 asst=1 tool=1 read=1 write=1 verify=0 out=1]"
    ));
}

#[test]
fn operator_surface_renders_compact_archetype_inspection_for_v0_5_checkpoints() {
    let mut visible = checkpoint_with_schema_state(
        "v0.5",
        "session-archetype",
        1,
        DriftClass::TruthGroundingGap,
        DriftState::Active,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-archetype:1"],
    );
    visible.turn_context = Some(sample_turn_context(1));
    visible.session_archetype = Some(sample_session_archetype(
        &visible,
        SessionArchetypeLabel::AutonomousImplementation,
    ));

    let mut silent = checkpoint_with_schema_state(
        "v0.5",
        "session-archetype",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    silent.turn_context = Some(sample_turn_context(2));
    silent.session_archetype = Some(sample_session_archetype(
        &silent,
        SessionArchetypeLabel::VerificationCloseout,
    ));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![visible, silent], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("- Turn context: turn-1 (#1)"));
    assert!(rendered.contains("- Archetype: label=autonomous_implementation confidence=medium"));
    assert!(rendered.contains("- Archetype: label=verification_closeout confidence=medium"));
    assert!(rendered.contains("source edit command strengthened implementation-like evidence"));
    assert!(rendered.contains("+1 more"));
    assert!(rendered.contains("counter[inspection-style command widened the visible search space]"));
    assert!(rendered.contains("- Diagnostics: task_frame_transitioned=true"));
}

#[test]
fn operator_surface_renders_compact_progress_inspection_for_v0_7_checkpoints() {
    let mut visible = checkpoint_with_schema_state(
        "v0.7",
        "session-progress",
        1,
        DriftClass::TruthGroundingGap,
        DriftState::Active,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-progress:1"],
    );
    visible.turn_context = Some(sample_turn_context(1));
    visible.session_archetype = Some(sample_session_archetype(
        &visible,
        SessionArchetypeLabel::AutonomousImplementation,
    ));
    visible.session_progress = Some(sample_session_progress(
        &visible,
        ProgressStatus::Advancing,
        ProgressDimension::ImplementationVerificationWall,
    ));

    let mut silent = checkpoint_with_schema_state(
        "v0.7",
        "session-progress",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    silent.turn_context = Some(sample_turn_context(2));
    silent.session_archetype = Some(sample_session_archetype(
        &silent,
        SessionArchetypeLabel::VerificationCloseout,
    ));
    silent.session_progress = Some(sample_session_progress(
        &silent,
        ProgressStatus::Mixed,
        ProgressDimension::VerificationCloseoutNarrowing,
    ));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![visible, silent], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let visible_render = result.report.visible_warnings[0].render_console_block(None);
    let silent_render = result.report.silent_checkpoints[0].render_console_block(None);

    assert!(visible_render.contains(
        "- Progress: status=advancing dimension=implementation_verification_wall confidence=medium"
    ));
    assert!(silent_render.contains(
        "- Progress: status=mixed dimension=verification_closeout_narrowing confidence=medium"
    ));
    assert!(silent_render.contains("- Posture: recovered"));
    assert!(silent_render
        .contains("Evidence: session-progress.jsonl#2:0 explicit analyzer recovery evidence"));
    assert_progress_line_after_archetype(&visible_render);
    assert_progress_line_after_archetype(&silent_render);
}

#[test]
fn operator_surface_classifies_active_recovered_and_historical_only_posture() {
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![
            checkpoint_with_drift(
                "session-posture",
                1,
                DriftClass::TruthGroundingGap,
                82,
                true,
                "align plan to repo truth",
                &["flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                2,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                3,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
        ],
        support::sample_summary(),
    );
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(
        result.report.visible_warnings[0].posture,
        Some(CheckpointPosture::Active)
    );
    let recovered = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0002")
        .expect("recovered checkpoint");
    assert_eq!(recovered.posture, Some(CheckpointPosture::Recovered));
    assert!(recovered
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-posture:1")));
    let historical_only = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0003")
        .expect("historical-only checkpoint");
    assert_eq!(
        historical_only.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert!(historical_only
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-posture:1")));

    let rendered = result.report.to_console_text();
    assert!(rendered.contains("- Posture: active"));
    assert!(rendered.contains("- Posture: recovered"));
    assert!(rendered.contains("- Posture: historical-only"));
    assert!(
        rendered.contains("historical truth-grounding gap: flagged score for session-posture:1")
    );
}

#[test]
fn operator_surface_preserves_recovered_posture_when_replay_resumes_after_cursor() {
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![
            checkpoint_with_drift(
                "session-posture",
                1,
                DriftClass::TruthGroundingGap,
                82,
                true,
                "align plan to repo truth",
                &["flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                2,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                3,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
        ],
        support::sample_summary(),
    );
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: Some(agent_drift_sentinel::CheckpointCursor {
            session_id: "session-posture".to_string(),
            ordinal: 1,
        }),
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(result.report.visible_warnings.len(), 0);
    let recovered = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0002")
        .expect("recovered checkpoint after cursor");
    assert_eq!(recovered.posture, Some(CheckpointPosture::Recovered));

    let historical_only = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0003")
        .expect("historical-only checkpoint after cursor");
    assert_eq!(
        historical_only.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
}

#[test]
fn operator_surface_prefers_explicit_v0_3_and_v0_4_state_without_previous_checkpoint() {
    for schema_version in ["v0.3", "v0.4"] {
        let recovered = checkpoint_with_schema_state(
            schema_version,
            "session-explicit-state",
            2,
            DriftClass::TruthGroundingGap,
            DriftState::Recovered,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer recovery evidence"],
        );
        let historical_only = checkpoint_with_schema_state(
            schema_version,
            "session-explicit-state",
            3,
            DriftClass::TruthGroundingGap,
            DriftState::HistoricalOnly,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer historical evidence"],
        );
        let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());

        let recovered_decision = scheduler.observe(
            agent_drift_sentinel::CheckpointCursor::from(&recovered),
            TriggerClass::CheckpointReady,
            recovered.flagged,
            Some(&warning_fingerprint(&recovered)),
        );
        let recovered_presentation = present_checkpoint_with_previous(
            &recovered,
            None,
            TriggerClass::CheckpointReady,
            &recovered_decision,
            &WarningPolicy::default(),
        );

        let historical_decision = scheduler.observe(
            agent_drift_sentinel::CheckpointCursor::from(&historical_only),
            TriggerClass::CheckpointReady,
            historical_only.flagged,
            Some(&warning_fingerprint(&historical_only)),
        );
        let historical_presentation = present_checkpoint_with_previous(
            &historical_only,
            None,
            TriggerClass::CheckpointReady,
            &historical_decision,
            &WarningPolicy::default(),
        );

        assert_eq!(
            recovered_presentation.posture,
            Some(CheckpointPosture::Recovered),
            "{schema_version} recovered posture should stay explicit-state backed"
        );
        assert_eq!(
            historical_presentation.posture,
            Some(CheckpointPosture::HistoricalOnly),
            "{schema_version} historical-only posture should stay explicit-state backed"
        );
        assert!(recovered_presentation
            .evidence_lines
            .iter()
            .any(|line| line.contains("explicit analyzer recovery evidence")));
        assert!(historical_presentation
            .evidence_lines
            .iter()
            .any(|line| line.contains("explicit analyzer historical evidence")));
    }
}

#[test]
fn operator_surface_detects_dead_end_historical_verification_evidence() {
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![checkpoint_with_drift(
            "session-history",
            1,
            DriftClass::DeadEndThrash,
            20,
            false,
            "re-run verification deliberately",
            &["historical repeated verification evidence: repeated verification command: cargo test -p agent-drift-sentinel -- --nocapture"],
        )],
        support::sample_summary(),
    );
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(result.report.visible_warnings.len(), 0);
    assert_eq!(result.report.silent_checkpoints.len(), 1);
    assert_eq!(
        result.report.silent_checkpoints[0].posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert!(result.report.silent_checkpoints[0]
        .evidence_lines
        .iter()
        .any(|line| line.contains("historical repeated verification evidence:")));
}

#[test]
fn operator_surface_public_previous_aware_presenter_can_render_recovered_posture() {
    let previous = checkpoint_with_drift(
        "session-public",
        1,
        DriftClass::TruthGroundingGap,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-public:1"],
    );
    let current = checkpoint_with_drift(
        "session-public",
        2,
        DriftClass::TruthGroundingGap,
        20,
        false,
        "continue on the current task frame",
        &["historical truth-grounding gap: flagged score for session-public:1"],
    );
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());
    let decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&current),
        TriggerClass::CheckpointReady,
        current.flagged,
        Some(&warning_fingerprint(&current)),
    );

    let presentation = present_checkpoint_with_previous(
        &current,
        Some(&previous),
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );

    assert_eq!(presentation.posture, Some(CheckpointPosture::Recovered));
    assert!(presentation
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-public:1")));
}

#[test]
fn operator_surface_renders_flagged_semantic_goal_drift_checkpoint_end_to_end() {
    // schema_version must be v0.7 (what the analyzer actually writes for this variant) so this
    // exercises the real explicit-state-backed rendering path, not the legacy v0.2 default.
    let mut checkpoint = checkpoint_with_drift(
        "session-semantic-goal-drift",
        1,
        DriftClass::SemanticGoalDrift,
        80,
        true,
        "confirm the pivot is intentional or return to the anchored goal",
        &[
            "semantic goal drift kickoff anchor: crates_agent_drift_analyzer_src_scoring_mod_rs",
            "semantic goal drift current goal: docs_specs_r6_map_md",
        ],
    );
    checkpoint.schema_version = "v0.7".to_string();
    checkpoint.drift_scores[0].state = DriftState::Active;
    checkpoint.turn_context = Some(sample_turn_context(1));
    checkpoint.session_archetype = Some(sample_session_archetype(
        &checkpoint,
        SessionArchetypeLabel::AutonomousImplementation,
    ));
    checkpoint.session_progress = Some(sample_session_progress(
        &checkpoint,
        ProgressStatus::Advancing,
        ProgressDimension::ImplementationVerificationWall,
    ));
    let fixture =
        support::ReplayFixture::from_checkpoints(vec![checkpoint], support::sample_summary());
    let warning_policy = WarningPolicy {
        max_evidence_lines: 4,
        ..WarningPolicy::default()
    };
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy,
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(result.report.visible_warnings.len(), 1);
    let visible = &result.report.visible_warnings[0];
    assert_eq!(visible.posture, Some(CheckpointPosture::Active));
    assert!(visible
        .evidence_lines
        .iter()
        .any(|line| line.contains("semantic goal drift kickoff anchor:")));
    assert!(visible
        .evidence_lines
        .iter()
        .any(|line| line.contains("semantic goal drift current goal:")));
}

#[test]
fn operator_surface_renders_cofire_rolling_previous_line_in_order_under_default_policy() {
    // The analyzer de-dups the shared current-goal line on co-fire, so a co-firing
    // SemanticGoalDrift checkpoint carries exactly three evidence lines in this order:
    // kickoff current goal, kickoff anchor, rolling previous goal. This test uses the DEFAULT
    // warning policy (max_evidence_lines = 3) — not a widened cap — to prove the informative
    // "rolling ... previous goal:" line (what the goal lurched away from) survives truncation
    // and renders in order, which the old 4-line ordering would have dropped.
    let mut checkpoint = checkpoint_with_drift(
        "session-semantic-goal-drift-rolling",
        3,
        DriftClass::SemanticGoalDrift,
        80,
        true,
        "confirm the pivot is intentional or return to the anchored goal",
        &[
            "semantic goal drift current goal: docs_specs_r6_map_md",
            "semantic goal drift kickoff anchor: crates_agent_drift_analyzer_tests_checkpoints_rs",
            "rolling semantic goal drift previous goal: crates_agent_drift_analyzer_tests_checkpoints_rs",
        ],
    );
    checkpoint.schema_version = "v0.7".to_string();
    checkpoint.drift_scores[0].state = DriftState::Active;
    checkpoint.turn_context = Some(sample_turn_context(3));
    checkpoint.session_archetype = Some(sample_session_archetype(
        &checkpoint,
        SessionArchetypeLabel::AutonomousImplementation,
    ));
    checkpoint.session_progress = Some(sample_session_progress(
        &checkpoint,
        ProgressStatus::Advancing,
        ProgressDimension::ImplementationVerificationWall,
    ));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![checkpoint], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let visible = &result.report.visible_warnings[0];
    assert_eq!(visible.posture, Some(CheckpointPosture::Active));
    assert_eq!(
        visible.evidence_lines.len(),
        3,
        "co-fire renders exactly three de-duped evidence lines under the default cap, got {:?}",
        visible.evidence_lines
    );
    assert!(visible.evidence_lines[0].contains("semantic goal drift current goal:"));
    assert!(visible.evidence_lines[1].contains("semantic goal drift kickoff anchor:"));
    assert!(
        visible.evidence_lines[2].contains("rolling semantic goal drift previous goal:"),
        "the rolling previous-goal line must survive default-policy truncation and render last, got {:?}",
        visible.evidence_lines
    );
    assert!(
        visible
            .evidence_lines
            .iter()
            .all(|line| !line.contains("rolling semantic goal drift current goal:")),
        "the redundant rolling current-goal line is de-duped by the analyzer and must not appear"
    );
}

#[test]
fn operator_surface_semantic_goal_drift_has_no_historical_echo_once_cleared() {
    // This is the sentinel-side half of the contract: given an already-Cleared score (which is
    // all this class's scorer ever produces when not flagged — the analyzer-side guarantee that
    // it never emits DriftStateHint::HistoricalContext is proven separately by
    // agent-drift-analyzer::checkpoint::tests::assign_drift_states_never_promotes_a_cleared_semantic_goal_drift_score_to_historical
    // — this test only proves the sentinel does not add Recovered/HistoricalOnly framing on top
    // of it, even right after a flagged one.
    let previous = checkpoint_with_schema_state(
        "v0.7",
        "session-semantic-goal-drift-echo",
        1,
        DriftClass::SemanticGoalDrift,
        DriftState::Active,
        80,
        true,
        "confirm the pivot is intentional or return to the anchored goal",
        &["semantic goal drift kickoff anchor: crates_agent_drift_analyzer_src_scoring_mod_rs"],
    );
    let current = checkpoint_with_schema_state(
        "v0.7",
        "session-semantic-goal-drift-echo",
        2,
        DriftClass::SemanticGoalDrift,
        DriftState::Cleared,
        0,
        false,
        "continue on the current task frame",
        &[],
    );

    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());
    let decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&current),
        TriggerClass::CheckpointReady,
        current.flagged,
        Some(&warning_fingerprint(&current)),
    );
    let presentation = present_checkpoint_with_previous(
        &current,
        Some(&previous),
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );

    assert_eq!(
        presentation.posture, None,
        "a cleared semantic_goal_drift score must not echo as Recovered/HistoricalOnly"
    );
}

#[test]
fn operator_surface_labels_scheduler_trigger_separately_from_analyzer_posture() {
    let recovered = checkpoint_with_state(
        "session-trigger-label",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    let historical_only = checkpoint_with_state(
        "session-trigger-label",
        3,
        DriftClass::TruthGroundingGap,
        DriftState::HistoricalOnly,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer historical evidence"],
    );
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());

    let recovered_decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&recovered),
        TriggerClass::RepeatedFailure,
        recovered.flagged,
        Some(&warning_fingerprint(&recovered)),
    );
    let recovered_presentation = present_checkpoint_with_previous(
        &recovered,
        None,
        TriggerClass::RepeatedFailure,
        &recovered_decision,
        &WarningPolicy::default(),
    );

    let historical_decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&historical_only),
        TriggerClass::RepeatedFailure,
        historical_only.flagged,
        Some(&warning_fingerprint(&historical_only)),
    );
    let historical_presentation = present_checkpoint_with_previous(
        &historical_only,
        None,
        TriggerClass::RepeatedFailure,
        &historical_decision,
        &WarningPolicy::default(),
    );

    assert!(recovered_presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        recovered_presentation.posture,
        Some(CheckpointPosture::Recovered)
    );
    assert!(historical_presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        historical_presentation.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
}

#[test]
fn operator_surface_ast_policy_locks_schema_predicate_and_facade_call_owners() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let operator_path = manifest_dir.join("src/operator_surface.rs");
    let operator_source = fs::read_to_string(&operator_path).expect("read operator surface source");
    let operator_file = syn::parse_file(&operator_source).expect("parse operator surface source");
    let schema_analysis = analyze_schema_policy(&operator_file);

    assert_eq!(
        schema_analysis.direct_uses,
        vec!["CheckpointPresentation::render_console_block".to_string()]
    );
    assert_eq!(
        schema_analysis.exact_predicates,
        vec!["CheckpointPresentation::render_console_block".to_string()]
    );
    assert!(
        schema_analysis.violations.is_empty(),
        "unclassified schema-version syntax: {:?}",
        schema_analysis.violations
    );

    let mut call_inventory = RenderCallInventory::default();
    for source_path in production_rust_sources(&manifest_dir.join("src")) {
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", source_path.display()));
        let file = syn::parse_file(&source)
            .unwrap_or_else(|error| panic!("parse {}: {error}", source_path.display()));
        let module = source_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("Rust source file stem");
        analyze_render_calls_in_items(module, &file.items, &mut call_inventory);
    }

    let expected_counts = expected_render_call_counts();
    assert_eq!(call_inventory.counts, expected_counts);
    assert_eq!(call_inventory.counts.values().sum::<usize>(), 4);
    assert_eq!(
        call_inventory
            .counts
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "ReplayReport::to_console_text".to_string(),
            "adjudication::shape_request".to_string(),
            "cli::run_live".to_string(),
        ])
    );
    assert!(
        call_inventory.violations.is_empty(),
        "unclassified render-console reference: {:?}",
        call_inventory.violations
    );
}

#[test]
fn ast_policy_self_test_does_not_treat_cfg_not_test_as_test_only() {
    let file = syn::parse_file(
        r#"
        #[cfg(not(test))]
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                if self.checkpoint.schema_version == "v0.8" {}
                if self.checkpoint.schema_version == "v0.8" {}
            }
        }

        #[cfg(not(test))]
        fn run_live(presentation: CheckpointPresentation) {
            presentation.render_console_block(None);
        }
        "#,
    )
    .expect("parse cfg(not(test)) policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec![
            "CheckpointPresentation::render_console_block".to_string(),
            "CheckpointPresentation::render_console_block".to_string(),
        ],
        "cfg(not(test)) schema uses must remain production-visible"
    );
    assert_eq!(
        schema.exact_predicates,
        vec![
            "CheckpointPresentation::render_console_block".to_string(),
            "CheckpointPresentation::render_console_block".to_string(),
        ],
        "the second cfg(not(test)) predicate must fail the exact-one policy"
    );

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("cli", &file.items, &mut calls);
    assert_eq!(
        calls.counts.get("cli::run_live"),
        Some(&1),
        "cfg(not(test)) facade calls must remain production-visible"
    );
}

#[test]
fn ast_policy_self_test_rejects_local_macro_rules_hiding_schema_predicates() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                macro_rules! hidden_schema_predicate {
                    () => {
                        if self.checkpoint.schema_version == "v0.8" {}
                    };
                }
                hidden_schema_predicate!();
            }
        }
        "#,
    )
    .expect("parse local macro policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec![
            concat!(
                "CheckpointPresentation::render_console_block: unparsed hidden_schema_predicate! macro: ",
                "macro is not in the production AST policy allowlist"
            ),
            concat!(
                "CheckpointPresentation::render_console_block::macro macro_rules: unparsed macro_rules! macro: ",
                "macro_rules bodies are not accepted by the production AST policy"
            ),
        ],
        "local macro_rules schema syntax must fail closed"
    );
}

#[test]
fn ast_policy_self_test_propagates_typed_destructured_schema_aliases() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                let ((schema_alias, unrelated),): ((&String, bool),) =
                    ((&self.checkpoint.schema_version, false),);
                if schema_alias == "v0.8" {}
                let _ = unrelated;
            }
        }
        "#,
    )
    .expect("parse typed destructuring policy fixture");

    let schema = analyze_schema_policy(&file);
    assert!(
        schema
            .violations
            .iter()
            .any(|violation| violation
                == "CheckpointPresentation::render_console_block: non-exact schema-version binary predicate"),
        "typed/destructured schema alias predicate must fail closed: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_propagates_schema_aliases_through_nested_control_forms() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                for (schema_alias, unrelated) in
                    [(&self.checkpoint.schema_version, false)]
                {
                    loop {
                        match unrelated {
                            _ if schema_alias == "v0.8" => break,
                            _ => break,
                        }
                    }
                }
            }
        }
        "#,
    )
    .expect("parse nested-control policy fixture");

    let schema = analyze_schema_policy(&file);
    assert!(
        schema.violations.iter().any(|violation| violation
            == "CheckpointPresentation::render_console_block: schema-version match guard"),
        "nested control-flow aliases must not bypass the schema policy: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_propagates_schema_aliases_through_macro_aggregates() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                let is_v0_8 = self.checkpoint.schema_version == "v0.8";
                let decisions = vec![is_v0_8];
                if decisions[0] {}
            }
        }
        "#,
    )
    .expect("parse macro aggregate policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec![
            "CheckpointPresentation::render_console_block: schema-version alias or nesting in if condition"
                .to_string(),
        ],
        "a schema-derived alias must remain classified through an allowlisted macro aggregate and index: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_propagates_direct_schema_through_macro_aggregate_if() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                let decisions = vec![self.checkpoint.schema_version == "v0.8"];
                if decisions[0] {}
            }
        }
        "#,
    )
    .expect("parse direct-schema macro aggregate if policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec![
            "CheckpointPresentation::render_console_block: schema-version alias or nesting in if condition"
                .to_string(),
        ],
        "a direct schema source must remain classified through an allowlisted macro aggregate and index: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_propagates_direct_schema_through_macro_aggregate_let_else() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                let decisions = vec![self.checkpoint.schema_version == "v0.8"];
                let true = decisions[0] else { return; };
            }
        }
        "#,
    )
    .expect("parse direct-schema macro aggregate let-else policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec![
            "CheckpointPresentation::render_console_block: schema-version let-else initializer"
                .to_string(),
        ],
        "a direct schema source must remain classified through an allowlisted macro aggregate and let-else: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_classifies_schema_aliases_in_let_else_control() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                let is_v0_8 = self.checkpoint.schema_version == "v0.8";
                let true = is_v0_8 else { return; };
            }
        }
        "#,
    )
    .expect("parse let-else policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec![
            "CheckpointPresentation::render_console_block: schema-version let-else initializer"
                .to_string(),
        ],
        "schema-derived let-else control must fail closed: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_rejects_a_fifth_call_inside_an_allowed_owner() {
    let file = syn::parse_file(
        r#"
        impl ReplayReport {
            fn to_console_text(&self) {
                fifth.render_console_block(None);
            }
        }
        "#,
    )
    .expect("parse fifth-call policy fixture");

    let mut calls = RenderCallInventory {
        counts: expected_render_call_counts(),
        violations: Vec::new(),
    };
    analyze_render_calls_in_items("operator_surface", &file.items, &mut calls);
    assert_eq!(
        calls.counts,
        BTreeMap::from([
            ("ReplayReport::to_console_text".to_string(), 3usize),
            ("adjudication::shape_request".to_string(), 1usize),
            ("cli::run_live".to_string(), 1usize),
        ]),
        "the fixture must add exactly one fifth call to the otherwise valid inventory"
    );
    assert!(
        !render_call_policy_is_exact(&calls),
        "an allowed owner must not bypass the global exact-count policy"
    );
}

#[test]
fn ast_policy_self_test_normalizes_raw_schema_field_identifiers() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                if self.checkpoint.r#schema_version == "v0.8" {}
            }
        }

        fn hidden_schema_branch(checkpoint: &Checkpoint) {
            if checkpoint.r#schema_version != "v0.8" {}
        }
        "#,
    )
    .expect("parse raw schema-field policy fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec![
            "CheckpointPresentation::render_console_block".to_string(),
            "hidden_schema_branch".to_string(),
        ],
        "raw schema fields must remain visible to the direct-use inventory"
    );
    assert_eq!(
        schema.exact_predicates,
        vec!["CheckpointPresentation::render_console_block".to_string()],
        "the allowed raw field spelling must normalize to the exact v0.8 predicate"
    );
    assert_eq!(
        schema.violations,
        vec![
            "hidden_schema_branch: non-exact schema-version binary predicate".to_string(),
            "hidden_schema_branch: schema-version alias or nesting in if condition".to_string(),
        ],
        "an unauthorized raw schema predicate must fail closed"
    );
}

#[test]
fn ast_policy_self_test_normalizes_raw_render_call_identifiers() {
    let file = syn::parse_file(
        r#"
        fn raw_calls(presentation: CheckpointPresentation) {
            presentation.r#render_console_block(None);
            CheckpointPresentation::r#render_console_block(&presentation, None);
            let escaped = CheckpointPresentation::r#render_console_block;
            let _ = escaped;
        }
        "#,
    )
    .expect("parse raw render-call policy fixture");

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("raw", &file.items, &mut calls);
    assert_eq!(
        calls.counts,
        BTreeMap::from([("raw::raw_calls".to_string(), 2usize)]),
        "raw method and associated-function calls must remain countable"
    );
    assert_eq!(
        calls.violations,
        vec!["raw::raw_calls: unclassified render_console_block path reference".to_string()],
        "an uncalled raw facade reference must fail closed"
    );
}

#[test]
fn ast_policy_self_test_attributes_nested_items_to_their_actual_owners() {
    let file = syn::parse_file(
        r#"
        impl CheckpointPresentation {
            fn render_console_block(&self) {
                fn nested_schema_branch(checkpoint: &Checkpoint) {
                    if checkpoint.schema_version == "v0.8" {}
                }
            }
        }

        impl ReplayReport {
            fn to_console_text(&self) {
                fn nested_render_call(presentation: CheckpointPresentation) {
                    presentation.render_console_block(None);
                }
            }
        }
        "#,
    )
    .expect("parse nested-item ownership fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec!["CheckpointPresentation::render_console_block::nested_schema_branch".to_string()],
        "a nested schema branch must not inherit its enclosing allowed owner"
    );
    assert!(schema.violations.iter().any(|violation| violation
        == "CheckpointPresentation::render_console_block::nested_schema_branch: non-exact schema-version binary predicate"));

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("operator_surface", &file.items, &mut calls);
    assert_eq!(
        calls.counts,
        BTreeMap::from([(
            "ReplayReport::to_console_text::nested_render_call".to_string(),
            1usize,
        )]),
        "a nested facade call must not inherit its enclosing allowed owner"
    );
}

#[test]
fn ast_policy_self_test_rejects_qualified_qself_and_attribute_schema_paths() {
    let file = syn::parse_file(
        r#"
        #[policy(crate::schema_version)]
        fn attribute_schema_path() {}

        fn qualified_schema_path() {
            if crate::schema_version == "v0.8" {}
        }

        fn qself_schema_path() {
            if <Checkpoint as HasSchema>::schema_version == "v0.8" {}
        }
        "#,
    )
    .expect("parse qualified schema-path fixture");

    let schema = analyze_schema_policy(&file);
    assert!(schema
        .violations
        .iter()
        .any(|violation| violation == "attribute_schema_path: schema_version attribute token"));
    for owner in ["qualified_schema_path", "qself_schema_path"] {
        assert!(
            schema
                .violations
                .iter()
                .any(|violation| violation == &format!("{owner}: qualified schema_version path")),
            "{owner} must fail closed: {schema:?}"
        );
    }
}

#[test]
fn ast_policy_self_test_rejects_unknown_macros_in_both_inventories() {
    let file = syn::parse_file(
        r#"
        fn hidden_schema(checkpoint: &Checkpoint) {
            mystery!(checkpoint.schema_version == "v0.8");
            shadow::format!("{}", checkpoint.schema_version);
        }

        fn hidden_render(presentation: CheckpointPresentation) {
            mystery!(presentation.render_console_block(None));
        }
        "#,
    )
    .expect("parse unknown-macro fixture");

    let schema = analyze_schema_policy(&file);
    assert!(schema.violations.iter().any(|violation| violation
        == "hidden_schema: unparsed mystery! macro: macro is not in the production AST policy allowlist"));
    assert!(schema.violations.iter().any(|violation| violation
        == "hidden_schema: unparsed shadow::format! macro: macro is not in the production AST policy allowlist"));

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("unknown", &file.items, &mut calls);
    assert!(calls.violations.iter().any(|violation| violation
        == "unknown::hidden_render: unparsed mystery! macro in render-call inventory: macro is not in the production AST policy allowlist"));
}

#[test]
fn ast_policy_self_test_rejects_schema_predicates_in_trait_const_defaults() {
    let file = syn::parse_file(
        r#"
        struct Checkpoint;

        impl Checkpoint {
            const schema_version: u8 = 8;
        }

        trait SchemaGate {
            const HIDDEN_SCHEMA_PREDICATE: bool = Checkpoint::schema_version == 8;
        }
        "#,
    )
    .expect("parse trait associated-const schema fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec!["trait SchemaGate::const HIDDEN_SCHEMA_PREDICATE".to_string()],
        "a trait associated-const default must retain its actual owner"
    );
    assert!(
        schema.violations.iter().any(|violation| violation
            == "trait SchemaGate::const HIDDEN_SCHEMA_PREDICATE: qualified schema_version path"),
        "a schema predicate in a trait associated-const default must fail closed: {schema:?}"
    );
}

#[test]
fn ast_policy_self_test_rejects_render_references_in_trait_const_defaults() {
    let file = syn::parse_file(
        r#"
        struct CheckpointPresentation;

        impl CheckpointPresentation {
            fn render_console_block(&self, _: Option<&str>) -> String {
                String::new()
            }
        }

        trait RenderReference {
            const HIDDEN_RENDER_REFERENCE: fn(
                &CheckpointPresentation,
                Option<&str>,
            ) -> String = CheckpointPresentation::render_console_block;
        }
        "#,
    )
    .expect("parse trait associated-const render-reference fixture");

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("trait_const", &file.items, &mut calls);
    assert_eq!(
        calls.violations,
        vec![concat!(
            "trait RenderReference::const HIDDEN_RENDER_REFERENCE: ",
            "unclassified render_console_block path reference"
        )],
        "a render function reference in a trait associated-const default must fail closed"
    );
}

#[test]
fn ast_policy_self_test_visits_all_compile_valid_trait_type_surfaces() {
    let source = r#"
        #![allow(dead_code, non_upper_case_globals)]

        struct Checkpoint;

        impl Checkpoint {
            const schema_version: u8 = 8;
        }

        struct CheckpointPresentation;

        impl CheckpointPresentation {
            fn render_console_block(&self) {}
        }

        trait HiddenControls {
            const HIDDEN: [(); {
                let _render = CheckpointPresentation::render_console_block;
                (Checkpoint::schema_version == 8) as usize
            }];

            fn hidden(_: [(); {
                let _render = CheckpointPresentation::render_console_block;
                (Checkpoint::schema_version == 8) as usize
            }]);

            type HiddenType: Into<[(); {
                let _render = CheckpointPresentation::render_console_block;
                (Checkpoint::schema_version == 8) as usize
            }]>;
        }
    "#;
    assert_rust_fixture_compiles("trait_type_surfaces", source);
    let file = syn::parse_file(source).expect("parse compile-valid trait type-surface fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec![
            "trait HiddenControls::const HIDDEN".to_string(),
            "trait HiddenControls::hidden".to_string(),
            "trait HiddenControls::type HiddenType".to_string(),
        ],
        "every trait signature/type surface must retain its actual schema owner"
    );
    assert_eq!(
        schema.violations,
        vec![
            "trait HiddenControls::const HIDDEN: non-exact schema-version binary predicate"
                .to_string(),
            "trait HiddenControls::const HIDDEN: qualified schema_version path".to_string(),
            "trait HiddenControls::hidden: non-exact schema-version binary predicate".to_string(),
            "trait HiddenControls::hidden: qualified schema_version path".to_string(),
            "trait HiddenControls::type HiddenType: non-exact schema-version binary predicate"
                .to_string(),
            "trait HiddenControls::type HiddenType: qualified schema_version path".to_string(),
        ],
        "schema inventory must fail closed across associated-const types, method signatures, and associated types"
    );

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("trait_types", &file.items, &mut calls);
    assert!(calls.counts.is_empty());
    assert_eq!(
        calls.violations,
        vec![
            concat!(
                "trait HiddenControls::const HIDDEN: ",
                "unclassified render_console_block path reference"
            ),
            concat!(
                "trait HiddenControls::hidden: ",
                "unclassified render_console_block path reference"
            ),
            concat!(
                "trait HiddenControls::type HiddenType: ",
                "unclassified render_console_block path reference"
            ),
        ],
        "render inventory must fail closed across associated-const types, method signatures, and associated types"
    );
}

#[test]
fn ast_policy_self_test_visits_compile_valid_trait_const_generic_defaults() {
    let source = r#"
        #![allow(dead_code)]

        struct Checkpoint;

        impl Checkpoint {
            const schema_version: u8 = 8;
        }

        struct CheckpointPresentation;

        impl CheckpointPresentation {
            fn render_console_block(&self) {}
        }

        trait HiddenConstDefault<
            const HIDDEN: usize = {
                let _render = CheckpointPresentation::render_console_block;
                (Checkpoint::schema_version == 8) as usize
            },
        > {}
    "#;
    assert_rust_fixture_compiles("trait_const_generic_default", source);
    let file = syn::parse_file(source).expect("parse trait const-generic default fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec!["trait HiddenConstDefault".to_string()],
        "a trait const-generic default must retain the trait owner"
    );
    assert!(schema
        .violations
        .iter()
        .any(|violation| violation == "trait HiddenConstDefault: qualified schema_version path"));

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("trait_default", &file.items, &mut calls);
    assert_eq!(
        calls.violations,
        vec![concat!(
            "trait HiddenConstDefault: ",
            "unclassified render_console_block path reference"
        )],
        "a render reference in a trait const-generic default must fail closed"
    );
}

#[test]
fn ast_policy_self_test_visits_compile_valid_trait_supertrait_const_arguments() {
    let source = r#"
        #![allow(dead_code)]

        struct Checkpoint;

        impl Checkpoint {
            const schema_version: u8 = 8;
        }

        struct CheckpointPresentation;

        impl CheckpointPresentation {
            fn render_console_block(&self) {}
        }

        trait ConstArgument<const VALUE: usize> {}

        trait HiddenSupertrait: ConstArgument<{
            let _render = CheckpointPresentation::render_console_block;
            (Checkpoint::schema_version == 8) as usize
        }> {}
    "#;
    assert_rust_fixture_compiles("trait_supertrait_const_argument", source);
    let file = syn::parse_file(source).expect("parse trait supertrait const-argument fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec!["trait HiddenSupertrait".to_string()],
        "a supertrait const argument must retain the trait owner"
    );
    assert!(schema
        .violations
        .iter()
        .any(|violation| violation == "trait HiddenSupertrait: qualified schema_version path"));

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("trait_supertrait", &file.items, &mut calls);
    assert_eq!(
        calls.violations,
        vec![concat!(
            "trait HiddenSupertrait: ",
            "unclassified render_console_block path reference"
        )],
        "a render reference in a supertrait const argument must fail closed"
    );
}

#[test]
fn ast_policy_self_test_visits_compile_valid_free_fn_parameter_types() {
    let source = r#"
        #![allow(dead_code)]

        struct Checkpoint;

        impl Checkpoint {
            const schema_version: u8 = 8;
        }

        struct CheckpointPresentation;

        impl CheckpointPresentation {
            fn render_console_block(&self) {}
        }

        fn hidden_parameter(_: [(); {
            let _render = CheckpointPresentation::render_console_block;
            (Checkpoint::schema_version == 8) as usize
        }]) {}
    "#;
    assert_rust_fixture_compiles("free_fn_parameter_type", source);
    let file = syn::parse_file(source).expect("parse free-fn parameter type fixture");

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.direct_uses,
        vec!["hidden_parameter".to_string()],
        "a free-function parameter type must retain the function owner"
    );
    assert!(schema
        .violations
        .iter()
        .any(|violation| violation == "hidden_parameter: qualified schema_version path"));

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("free_fn", &file.items, &mut calls);
    assert_eq!(
        calls.violations,
        vec![concat!(
            "free_fn::hidden_parameter: ",
            "unclassified render_console_block path reference"
        )],
        "a render reference in a free-function parameter type must fail closed"
    );
}

#[test]
fn ast_policy_self_test_visits_all_item_impl_and_foreign_header_surfaces() {
    const MARKER: &str = r#"{
        let _render = CheckpointPresentation::render_console_block;
        (Checkpoint::schema_version == 8) as usize
    }"#;

    let cases = [
        (
            "free function signature",
            "fn free<T: Bound<$M>>(_: [(); $M]) -> [(); $M] where T: Other<$M> { loop {} }",
            "free",
            "headers::free",
            4,
        ),
        (
            "const type",
            "const VALUE: [(); $M] = [];",
            "const VALUE",
            "headers::const VALUE",
            1,
        ),
        (
            "static type",
            "static VALUE: [(); $M] = [];",
            "static VALUE",
            "headers::static VALUE",
            1,
        ),
        (
            "type alias generics, where clause, and type",
            "type Alias<const N: usize = $M, T> where T: Bound<$M> = [(); $M];",
            "type Alias",
            "type Alias",
            3,
        ),
        (
            "struct generics, where clause, and field type",
            "struct Struct<const N: usize = $M, T> where T: Bound<$M> { field: [(); $M] }",
            "struct Struct",
            "struct Struct",
            3,
        ),
        (
            "enum generics, where clause, field type, and discriminant",
            concat!(
                "enum Enum<const N: usize = $M, T> where T: Bound<$M> { ",
                "Field([(); $M]), Discriminant = $M }"
            ),
            "enum Enum",
            "enum Enum",
            4,
        ),
        (
            "union generics, where clause, and field type",
            concat!(
                "union Union<const N: usize = $M, T> where T: Bound<$M> { ",
                "field: ManuallyDrop<[(); $M]> }"
            ),
            "union Union",
            "union Union",
            3,
        ),
        (
            "trait generics, supertraits, and where clause",
            "trait Trait<const N: usize = $M, T>: Bound<$M> where T: Other<$M> {}",
            "trait Trait",
            "trait Trait",
            3,
        ),
        (
            "trait alias generics, bounds, and where clause",
            "trait Alias<const N: usize = $M, T> = Bound<$M> where T: Other<$M>;",
            "trait alias Alias",
            "trait alias Alias",
            3,
        ),
        (
            "impl generics, trait, self type, and where clause",
            concat!(
                "impl<T: Bound<$M>> Trait<$M> for Target<$M> ",
                "where T: Other<$M> {}"
            ),
            "impl <Target<...> as Trait>",
            "impl <Target<...> as Trait>",
            4,
        ),
        (
            "impl function signature",
            concat!(
                "impl Target { fn method<T: Bound<$M>>(_: [(); $M]) -> [(); $M] ",
                "where T: Other<$M> { loop {} } }"
            ),
            "Target::method",
            "Target::method",
            4,
        ),
        (
            "impl const type",
            "impl Target { const VALUE: [(); $M] = []; }",
            "Target::const VALUE",
            "Target::const VALUE",
            1,
        ),
        (
            "impl type generics, where clause, and type",
            concat!(
                "impl Target { type Value<const N: usize = $M, T> = [(); $M] ",
                "where T: Bound<$M>; }"
            ),
            "Target::type Value",
            "Target::type Value",
            3,
        ),
        (
            "foreign function signature",
            concat!(
                "extern \"C\" { fn foreign<T: Bound<$M>>(_: [(); $M]) -> [(); $M] ",
                "where T: Other<$M>; }"
            ),
            "extern block::foreign",
            "extern block::foreign",
            4,
        ),
        (
            "foreign static type",
            "extern \"C\" { static FOREIGN: [(); $M]; }",
            "extern block::static FOREIGN",
            "extern block::static FOREIGN",
            1,
        ),
        (
            "foreign type generics and where clause",
            "extern \"C\" { type Foreign<T: Bound<$M>> where T: Other<$M>; }",
            "extern block::type Foreign",
            "extern block::type Foreign",
            2,
        ),
        (
            "module attribute",
            "#[policy(Checkpoint::schema_version == 8, CheckpointPresentation::render_console_block)] mod nested;",
            "nested",
            "headers::nested",
            1,
        ),
    ];

    for (label, template, schema_owner, render_owner, expected_count) in cases {
        let source = template.replace("$M", MARKER);
        let file = syn::parse_file(&source)
            .unwrap_or_else(|error| panic!("parse {label} header fixture: {error}\n{source}"));

        let schema = analyze_schema_policy(&file);
        assert_eq!(
            schema.direct_uses,
            vec![schema_owner.to_string(); expected_count],
            "schema inventory missed {label}: {schema:?}"
        );

        let mut calls = RenderCallInventory::default();
        analyze_render_calls_in_items("headers", &file.items, &mut calls);
        let expected_violation =
            format!("{render_owner}: unclassified render_console_block path reference");
        assert_eq!(
            calls.violations,
            vec![expected_violation; expected_count],
            "render inventory missed {label}: {calls:?}"
        );
    }
}

#[test]
fn ast_policy_self_test_records_unsupported_trait_item_forms() {
    let mut file = syn::parse_file("trait HiddenControls {}")
        .expect("parse unsupported trait-item fixture shell");
    let syn::Item::Trait(item_trait) = &mut file.items[0] else {
        panic!("fixture shell must contain a trait");
    };
    item_trait.items.push(syn::TraitItem::Verbatim(
        "unsupported trait item"
            .parse()
            .expect("parse verbatim trait-item tokens"),
    ));

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec!["trait HiddenControls: unsupported trait item form: verbatim".to_string()],
        "schema inventory must not silently ignore an unsupported trait item"
    );

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("trait_items", &file.items, &mut calls);
    assert_eq!(
        calls.violations,
        vec!["trait HiddenControls: unsupported trait item form: verbatim".to_string()],
        "render inventory must not silently ignore an unsupported trait item"
    );
}

#[test]
fn ast_policy_self_test_records_unsupported_item_impl_and_foreign_forms() {
    let mut file = syn::parse_file("impl Target {} extern \"C\" {}")
        .expect("parse unsupported item-form fixture shell");
    file.items.insert(
        0,
        syn::Item::Verbatim(
            "unsupported item"
                .parse()
                .expect("parse verbatim item tokens"),
        ),
    );
    let syn::Item::Impl(item_impl) = &mut file.items[1] else {
        panic!("fixture shell must contain an impl");
    };
    item_impl.items.push(syn::ImplItem::Verbatim(
        "unsupported impl item"
            .parse()
            .expect("parse verbatim impl-item tokens"),
    ));
    let syn::Item::ForeignMod(foreign_mod) = &mut file.items[2] else {
        panic!("fixture shell must contain a foreign block");
    };
    foreign_mod.items.push(syn::ForeignItem::Verbatim(
        "unsupported foreign item"
            .parse()
            .expect("parse verbatim foreign-item tokens"),
    ));

    let schema = analyze_schema_policy(&file);
    assert_eq!(
        schema.violations,
        vec![
            "crate: unsupported item form: verbatim".to_string(),
            "extern block: unsupported foreign item form: verbatim".to_string(),
            "impl Target: unsupported impl item form: verbatim".to_string(),
        ],
        "schema inventory must fail explicitly for every unsupported item family"
    );

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("unsupported", &file.items, &mut calls);
    assert_eq!(
        calls.violations,
        vec![
            "unsupported: unsupported item form: verbatim".to_string(),
            "impl Target: unsupported impl item form: verbatim".to_string(),
            "extern block: unsupported foreign item form: verbatim".to_string(),
        ],
        "render inventory must fail explicitly for every unsupported item family"
    );
}

fn assert_rust_fixture_compiles(crate_name: &str, source: &str) {
    let fixture_dir = tempfile::tempdir().expect("create Rust fixture directory");
    let source_path = fixture_dir.path().join("lib.rs");
    let output_path = fixture_dir.path().join("fixture.rlib");
    fs::write(&source_path, source).expect("write Rust fixture source");
    let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("--crate-name")
        .arg(crate_name)
        .arg("--crate-type=lib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(output_path)
        .output()
        .expect("run rustc for fixture");
    assert!(
        output.status.success(),
        "fixture must compile successfully:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ast_policy_self_test_rejects_trait_impl_and_nested_schema_owner_lookalikes() {
    let trait_impl = syn::parse_file(
        r#"
        impl PresentationRenderer for CheckpointPresentation {
            fn render_console_block(&self) {
                if self.checkpoint.schema_version == "v0.8" {}
            }
        }
        "#,
    )
    .expect("parse trait-impl schema owner fixture");
    assert_eq!(
        analyze_schema_policy(&trait_impl).direct_uses,
        vec!["<CheckpointPresentation as PresentationRenderer>::render_console_block".to_string()],
        "a trait impl must not impersonate the allowed inherent method"
    );

    let nested_impl = syn::parse_file(
        r#"
        fn nested_owner() {
            impl CheckpointPresentation {
                fn render_console_block(&self) {
                    if self.checkpoint.schema_version == "v0.8" {}
                }
            }
        }
        "#,
    )
    .expect("parse nested schema owner fixture");
    assert_eq!(
        analyze_schema_policy(&nested_impl).direct_uses,
        vec!["nested_owner::CheckpointPresentation::render_console_block".to_string()],
        "a nested inherent impl must retain its enclosing owner lineage"
    );
}

#[test]
fn ast_policy_self_test_rejects_trait_impl_and_nested_render_owner_lookalikes() {
    let trait_impl = syn::parse_file(
        r#"
        impl ConsoleRenderer for ReplayReport {
            fn to_console_text(&self, presentation: &CheckpointPresentation) {
                presentation.render_console_block(None);
            }
        }
        "#,
    )
    .expect("parse trait-impl render owner fixture");
    let mut trait_calls = RenderCallInventory::default();
    analyze_render_calls_in_items("operator_surface", &trait_impl.items, &mut trait_calls);
    assert_eq!(
        trait_calls.counts,
        BTreeMap::from([(
            "<ReplayReport as ConsoleRenderer>::to_console_text".to_string(),
            1usize,
        )]),
        "a trait impl must not impersonate the allowed inherent method"
    );

    let nested_impl = syn::parse_file(
        r#"
        fn nested_owner() {
            impl ReplayReport {
                fn to_console_text(&self, presentation: &CheckpointPresentation) {
                    presentation.render_console_block(None);
                }
            }
        }
        "#,
    )
    .expect("parse nested render owner fixture");
    let mut nested_calls = RenderCallInventory::default();
    analyze_render_calls_in_items("operator_surface", &nested_impl.items, &mut nested_calls);
    assert_eq!(
        nested_calls.counts,
        BTreeMap::from([(
            "operator_surface::nested_owner::ReplayReport::to_console_text".to_string(),
            1usize,
        )]),
        "a nested inherent impl must retain its enclosing owner lineage"
    );
}

#[test]
fn ast_policy_self_test_rejects_qualified_inherent_schema_owner_lookalike() {
    let file = syn::parse_file(
        r#"
        impl lookalike::CheckpointPresentation {
            fn render_console_block(&self) {
                if self.checkpoint.schema_version == "v0.8" {}
            }
        }
        "#,
    )
    .expect("parse qualified inherent schema owner fixture");

    let schema = analyze_schema_policy(&file);
    let owner = "lookalike::CheckpointPresentation::render_console_block".to_string();
    assert_eq!(
        schema.direct_uses,
        vec![owner.clone()],
        "a qualified inherent impl must retain its full self-type path"
    );
    assert_eq!(
        schema.exact_predicates,
        vec![owner],
        "qualified inherent schema syntax must not be attributed to the allowed top-level owner"
    );
}

#[test]
fn ast_policy_self_test_rejects_qualified_inherent_render_owner_lookalike() {
    let file = syn::parse_file(
        r#"
        impl lookalike::ReplayReport {
            fn to_console_text(&self, presentation: &CheckpointPresentation) {
                presentation.render_console_block(None);
                presentation.render_console_block(None);
            }
        }
        "#,
    )
    .expect("parse qualified inherent render owner fixture");

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("operator_surface", &file.items, &mut calls);
    assert_eq!(
        calls.counts,
        BTreeMap::from([(
            "lookalike::ReplayReport::to_console_text".to_string(),
            2usize,
        )]),
        "a qualified inherent impl must not impersonate the allowed top-level render owner"
    );
}

#[test]
fn ast_policy_self_test_rejects_schema_and_allowlisted_macro_import_aliases() {
    let file = syn::parse_file(
        r#"
        use crate::schema_version as hidden_schema;
        use shadow::injected as format;
        use shadow as anyhow;

        fn hidden_schema_branch() {
            if hidden_schema == "v0.8" {}
            format!("macro body is unavailable to the AST policy");
            anyhow::anyhow!("qualified macro body is also unavailable");
        }
        "#,
    )
    .expect("parse import-alias policy fixture");

    let schema = analyze_schema_policy(&file);
    for expected in [
        "use hidden_schema: schema_version import `crate::schema_version` binds `hidden_schema`",
        "use format: untrusted allowlisted macro import `shadow::injected` binds `format`",
        "use anyhow: untrusted allowlisted macro import `shadow` binds `anyhow`",
    ] {
        assert!(
            schema
                .violations
                .iter()
                .any(|violation| violation == expected),
            "schema inventory must reject {expected}: {schema:?}"
        );
    }

    let mut calls = RenderCallInventory::default();
    analyze_render_calls_in_items("aliases", &file.items, &mut calls);
    for expected in [
        "aliases::use format: untrusted allowlisted macro import `shadow::injected` binds `format`",
        "aliases::use anyhow: untrusted allowlisted macro import `shadow` binds `anyhow`",
    ] {
        assert!(
            calls
                .violations
                .iter()
                .any(|violation| violation == expected),
            "render inventory must reject {expected}: {calls:?}"
        );
    }
}

fn visit_item_header<'ast, V: Visit<'ast>>(visitor: &mut V, item: &'ast syn::Item) {
    for attribute in item_attrs(item) {
        visitor.visit_attribute(attribute);
    }
    match item {
        syn::Item::Const(item) => {
            visitor.visit_generics(&item.generics);
            visitor.visit_type(&item.ty);
        }
        syn::Item::Enum(item) => {
            visitor.visit_generics(&item.generics);
            for variant in &item.variants {
                visitor.visit_variant(variant);
            }
        }
        syn::Item::ExternCrate(_) => {}
        syn::Item::Fn(item) => visitor.visit_signature(&item.sig),
        syn::Item::ForeignMod(item) => visitor.visit_abi(&item.abi),
        syn::Item::Impl(item) => {
            visitor.visit_generics(&item.generics);
            if let Some((_, trait_path, _)) = &item.trait_ {
                visitor.visit_path(trait_path);
            }
            visitor.visit_type(&item.self_ty);
        }
        syn::Item::Macro(_) | syn::Item::Mod(_) => {}
        syn::Item::Static(item) => visitor.visit_type(&item.ty),
        syn::Item::Struct(item) => {
            visitor.visit_generics(&item.generics);
            for field in &item.fields {
                visitor.visit_field(field);
            }
        }
        syn::Item::Trait(item) => {
            visitor.visit_generics(&item.generics);
            for supertrait in &item.supertraits {
                visitor.visit_type_param_bound(supertrait);
            }
        }
        syn::Item::TraitAlias(item) => {
            visitor.visit_generics(&item.generics);
            for bound in &item.bounds {
                visitor.visit_type_param_bound(bound);
            }
        }
        syn::Item::Type(item) => {
            visitor.visit_generics(&item.generics);
            visitor.visit_type(&item.ty);
        }
        syn::Item::Union(item) => {
            visitor.visit_generics(&item.generics);
            for field in &item.fields.named {
                visitor.visit_field(field);
            }
        }
        syn::Item::Use(item) => visitor.visit_use_tree(&item.tree),
        syn::Item::Verbatim(_) => {}
        _ => {}
    }
}

fn visit_impl_item_header<'ast, V: Visit<'ast>>(visitor: &mut V, item: &'ast syn::ImplItem) {
    match item {
        syn::ImplItem::Const(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_generics(&item.generics);
            visitor.visit_type(&item.ty);
        }
        syn::ImplItem::Fn(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_signature(&item.sig);
        }
        syn::ImplItem::Type(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_generics(&item.generics);
            visitor.visit_type(&item.ty);
        }
        syn::ImplItem::Macro(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
        }
        syn::ImplItem::Verbatim(_) => {}
        _ => {}
    }
}

fn visit_trait_item_header<'ast, V: Visit<'ast>>(visitor: &mut V, item: &'ast syn::TraitItem) {
    match item {
        syn::TraitItem::Const(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_generics(&item.generics);
            visitor.visit_type(&item.ty);
        }
        syn::TraitItem::Fn(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_signature(&item.sig);
        }
        syn::TraitItem::Type(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_generics(&item.generics);
            for bound in &item.bounds {
                visitor.visit_type_param_bound(bound);
            }
            if let Some((_, default)) = &item.default {
                visitor.visit_type(default);
            }
        }
        syn::TraitItem::Macro(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
        }
        syn::TraitItem::Verbatim(_) => {}
        _ => {}
    }
}

fn visit_foreign_item_header<'ast, V: Visit<'ast>>(visitor: &mut V, item: &'ast syn::ForeignItem) {
    match item {
        syn::ForeignItem::Fn(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_signature(&item.sig);
        }
        syn::ForeignItem::Static(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_type(&item.ty);
        }
        syn::ForeignItem::Type(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
            visitor.visit_generics(&item.generics);
        }
        syn::ForeignItem::Macro(item) => {
            for attribute in &item.attrs {
                visitor.visit_attribute(attribute);
            }
        }
        syn::ForeignItem::Verbatim(_) => {}
        _ => {}
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SchemaPolicyAnalysis {
    direct_uses: Vec<String>,
    exact_predicates: Vec<String>,
    violations: Vec<String>,
}

fn analyze_schema_policy(file: &syn::File) -> SchemaPolicyAnalysis {
    let mut analysis = SchemaPolicyAnalysis::default();
    analyze_schema_items(&file.items, None, &mut analysis);
    analysis.direct_uses.sort();
    analysis.exact_predicates.sort();
    analysis.violations.sort();
    analysis
}

fn analyze_schema_item_header(owner: &str, item: &syn::Item, analysis: &mut SchemaPolicyAnalysis) {
    let aliases = HashSet::new();
    let mut visitor = SchemaExpressionVisitor {
        owner,
        aliases: &aliases,
        analysis,
    };
    visit_item_header(&mut visitor, item);
}

fn analyze_schema_impl_item_header(
    owner: &str,
    item: &syn::ImplItem,
    analysis: &mut SchemaPolicyAnalysis,
) {
    let aliases = HashSet::new();
    let mut visitor = SchemaExpressionVisitor {
        owner,
        aliases: &aliases,
        analysis,
    };
    visit_impl_item_header(&mut visitor, item);
}

fn analyze_schema_trait_item_header(
    owner: &str,
    item: &syn::TraitItem,
    analysis: &mut SchemaPolicyAnalysis,
) {
    let aliases = HashSet::new();
    let mut visitor = SchemaExpressionVisitor {
        owner,
        aliases: &aliases,
        analysis,
    };
    visit_trait_item_header(&mut visitor, item);
}

fn analyze_schema_foreign_item_header(
    owner: &str,
    item: &syn::ForeignItem,
    analysis: &mut SchemaPolicyAnalysis,
) {
    let aliases = HashSet::new();
    let mut visitor = SchemaExpressionVisitor {
        owner,
        aliases: &aliases,
        analysis,
    };
    visit_foreign_item_header(&mut visitor, item);
}

fn analyze_schema_items(
    items: &[syn::Item],
    module: Option<&str>,
    analysis: &mut SchemaPolicyAnalysis,
) {
    for item in items {
        if has_cfg_test(item_attrs(item)) {
            continue;
        }
        match item {
            syn::Item::Fn(function) => {
                let owner = qualify_owner(module, &identifier_name(&function.sig.ident));
                analyze_schema_item_header(&owner, item, analysis);
                analyze_schema_block(owner, &function.block, analysis);
            }
            syn::Item::Impl(item_impl) => {
                let type_path = type_owner_name(&item_impl.self_ty).unwrap_or_else(|| {
                    analysis.violations.push(format!(
                        "{}: unclassified impl self type",
                        qualify_owner(module, "impl <unclassified self type>")
                    ));
                    "<unclassified self type>".to_string()
                });
                let impl_identity = impl_identity(item_impl, &type_path);
                let impl_owner = qualify_owner(module, &format!("impl {impl_identity}"));
                analyze_schema_item_header(&impl_owner, item, analysis);
                for impl_item in &item_impl.items {
                    match impl_item {
                        syn::ImplItem::Fn(method) if !has_cfg_test(&method.attrs) => {
                            let owner = qualify_owner(
                                module,
                                &format!("{impl_identity}::{}", identifier_name(&method.sig.ident)),
                            );
                            analyze_schema_impl_item_header(&owner, impl_item, analysis);
                            analyze_schema_block(owner, &method.block, analysis);
                        }
                        syn::ImplItem::Const(constant) if !has_cfg_test(&constant.attrs) => {
                            let owner = qualify_owner(
                                module,
                                &format!(
                                    "{impl_identity}::const {}",
                                    identifier_name(&constant.ident)
                                ),
                            );
                            analyze_schema_impl_item_header(&owner, impl_item, analysis);
                            analyze_schema_expression(owner, &constant.expr, analysis);
                        }
                        syn::ImplItem::Type(associated_type)
                            if !has_cfg_test(&associated_type.attrs) =>
                        {
                            let owner = qualify_owner(
                                module,
                                &format!(
                                    "{impl_identity}::type {}",
                                    identifier_name(&associated_type.ident)
                                ),
                            );
                            analyze_schema_impl_item_header(&owner, impl_item, analysis);
                        }
                        syn::ImplItem::Macro(item_macro) if !has_cfg_test(&item_macro.attrs) => {
                            let owner = qualify_owner(
                                module,
                                &format!("{impl_identity}::macro {}", macro_path(&item_macro.mac)),
                            );
                            analyze_schema_impl_item_header(&owner, impl_item, analysis);
                            analyze_schema_macro(owner, &item_macro.mac, analysis);
                        }
                        syn::ImplItem::Verbatim(_) => analysis.violations.push(format!(
                            "{impl_owner}: unsupported impl item form: verbatim"
                        )),
                        syn::ImplItem::Fn(_)
                        | syn::ImplItem::Const(_)
                        | syn::ImplItem::Type(_)
                        | syn::ImplItem::Macro(_) => {}
                        _ => analysis
                            .violations
                            .push(format!("{impl_owner}: unsupported impl item form: unknown")),
                    }
                }
            }
            syn::Item::Const(constant) => {
                let owner = qualify_owner(
                    module,
                    &format!("const {}", identifier_name(&constant.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
                analyze_schema_expression(owner, &constant.expr, analysis);
            }
            syn::Item::Static(static_item) => {
                let owner = qualify_owner(
                    module,
                    &format!("static {}", identifier_name(&static_item.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
                analyze_schema_expression(owner, &static_item.expr, analysis);
            }
            syn::Item::Struct(item_struct) => {
                let owner = qualify_owner(
                    module,
                    &format!("struct {}", identifier_name(&item_struct.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
                if item_struct.fields.iter().any(|field| {
                    field
                        .ident
                        .as_ref()
                        .is_some_and(|identifier| identifier_is(identifier, "schema_version"))
                }) {
                    analysis
                        .violations
                        .push(format!("{owner}: schema_version field definition"));
                }
            }
            syn::Item::Enum(item_enum) => {
                let owner = qualify_owner(
                    module,
                    &format!("enum {}", identifier_name(&item_enum.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
                if item_enum.variants.iter().any(|variant| {
                    variant.fields.iter().any(|field| {
                        field
                            .ident
                            .as_ref()
                            .is_some_and(|identifier| identifier_is(identifier, "schema_version"))
                    })
                }) {
                    analysis
                        .violations
                        .push(format!("{owner}: schema_version field definition"));
                }
            }
            syn::Item::Union(item_union) => {
                let owner = qualify_owner(
                    module,
                    &format!("union {}", identifier_name(&item_union.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
                if item_union.fields.named.iter().any(|field| {
                    field
                        .ident
                        .as_ref()
                        .is_some_and(|identifier| identifier_is(identifier, "schema_version"))
                }) {
                    analysis
                        .violations
                        .push(format!("{owner}: schema_version field definition"));
                }
            }
            syn::Item::Trait(item_trait) => {
                let trait_identity = qualify_owner(
                    module,
                    &format!("trait {}", identifier_name(&item_trait.ident)),
                );
                analyze_schema_item_header(&trait_identity, item, analysis);
                for trait_item in &item_trait.items {
                    match trait_item {
                        syn::TraitItem::Fn(method) if !has_cfg_test(&method.attrs) => {
                            let owner = format!(
                                "{}::{}",
                                trait_identity,
                                identifier_name(&method.sig.ident)
                            );
                            analyze_schema_trait_item_header(&owner, trait_item, analysis);
                            if let Some(default) = &method.default {
                                analyze_schema_block(owner, default, analysis);
                            }
                        }
                        syn::TraitItem::Const(constant) if !has_cfg_test(&constant.attrs) => {
                            let owner = format!(
                                "{}::const {}",
                                trait_identity,
                                identifier_name(&constant.ident)
                            );
                            analyze_schema_trait_item_header(&owner, trait_item, analysis);
                            if let Some((_, default)) = &constant.default {
                                analyze_schema_expression(owner, default, analysis);
                            }
                        }
                        syn::TraitItem::Type(associated_type)
                            if !has_cfg_test(&associated_type.attrs) =>
                        {
                            let owner = format!(
                                "{}::type {}",
                                trait_identity,
                                identifier_name(&associated_type.ident)
                            );
                            analyze_schema_trait_item_header(&owner, trait_item, analysis);
                        }
                        syn::TraitItem::Macro(item_macro) if !has_cfg_test(&item_macro.attrs) => {
                            let owner = format!(
                                "{}::macro {}",
                                trait_identity,
                                macro_path(&item_macro.mac)
                            );
                            analyze_schema_trait_item_header(&owner, trait_item, analysis);
                            analyze_schema_macro(owner, &item_macro.mac, analysis);
                        }
                        syn::TraitItem::Verbatim(_) => analysis.violations.push(format!(
                            "{trait_identity}: unsupported trait item form: verbatim"
                        )),
                        syn::TraitItem::Fn(_)
                        | syn::TraitItem::Const(_)
                        | syn::TraitItem::Type(_)
                        | syn::TraitItem::Macro(_) => {}
                        _ => analysis.violations.push(format!(
                            "{trait_identity}: unsupported trait item form: unknown"
                        )),
                    }
                }
            }
            syn::Item::TraitAlias(item_alias) => {
                let owner = qualify_owner(
                    module,
                    &format!("trait alias {}", identifier_name(&item_alias.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
            }
            syn::Item::Type(item_type) => {
                let owner = qualify_owner(
                    module,
                    &format!("type {}", identifier_name(&item_type.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
            }
            syn::Item::ForeignMod(foreign_mod) => {
                let foreign_owner = qualify_owner(module, "extern block");
                analyze_schema_item_header(&foreign_owner, item, analysis);
                for foreign_item in &foreign_mod.items {
                    match foreign_item {
                        syn::ForeignItem::Fn(function) if !has_cfg_test(&function.attrs) => {
                            let owner = format!(
                                "{foreign_owner}::{}",
                                identifier_name(&function.sig.ident)
                            );
                            analyze_schema_foreign_item_header(&owner, foreign_item, analysis);
                        }
                        syn::ForeignItem::Static(static_item)
                            if !has_cfg_test(&static_item.attrs) =>
                        {
                            let owner = format!(
                                "{foreign_owner}::static {}",
                                identifier_name(&static_item.ident)
                            );
                            analyze_schema_foreign_item_header(&owner, foreign_item, analysis);
                        }
                        syn::ForeignItem::Type(foreign_type)
                            if !has_cfg_test(&foreign_type.attrs) =>
                        {
                            let owner = format!(
                                "{foreign_owner}::type {}",
                                identifier_name(&foreign_type.ident)
                            );
                            analyze_schema_foreign_item_header(&owner, foreign_item, analysis);
                        }
                        syn::ForeignItem::Macro(item_macro) if !has_cfg_test(&item_macro.attrs) => {
                            let owner =
                                format!("{foreign_owner}::macro {}", macro_path(&item_macro.mac));
                            analyze_schema_foreign_item_header(&owner, foreign_item, analysis);
                            analyze_schema_macro(owner, &item_macro.mac, analysis);
                        }
                        syn::ForeignItem::Verbatim(_) => analysis.violations.push(format!(
                            "{foreign_owner}: unsupported foreign item form: verbatim"
                        )),
                        syn::ForeignItem::Fn(_)
                        | syn::ForeignItem::Static(_)
                        | syn::ForeignItem::Type(_)
                        | syn::ForeignItem::Macro(_) => {}
                        _ => analysis.violations.push(format!(
                            "{foreign_owner}: unsupported foreign item form: unknown"
                        )),
                    }
                }
            }
            syn::Item::ExternCrate(extern_crate) => {
                let owner = qualify_owner(
                    module,
                    &format!("extern crate {}", identifier_name(&extern_crate.ident)),
                );
                analyze_schema_item_header(&owner, item, analysis);
            }
            syn::Item::Macro(item_macro) => {
                let owner =
                    qualify_owner(module, &format!("macro {}", macro_path(&item_macro.mac)));
                analyze_schema_item_header(&owner, item, analysis);
                analyze_schema_macro(owner, &item_macro.mac, analysis);
            }
            syn::Item::Mod(item_mod) => {
                let nested_module = module.map_or_else(
                    || identifier_name(&item_mod.ident),
                    |module| format!("{module}::{}", identifier_name(&item_mod.ident)),
                );
                analyze_schema_item_header(&nested_module, item, analysis);
                if let Some((_, nested)) = &item_mod.content {
                    analyze_schema_items(nested, Some(&nested_module), analysis);
                }
            }
            syn::Item::Use(item_use) => {
                analyze_schema_imports(module, item_use, analysis);
            }
            syn::Item::Verbatim(_) => analysis.violations.push(format!(
                "{}: unsupported item form: verbatim",
                module.unwrap_or("crate")
            )),
            _ => analysis.violations.push(format!(
                "{}: unsupported item form: unknown",
                module.unwrap_or("crate")
            )),
        }
    }
}

fn analyze_schema_attributes(
    owner: &str,
    attributes: &[syn::Attribute],
    analysis: &mut SchemaPolicyAnalysis,
) {
    for attribute in attributes {
        let contains_schema = match &attribute.meta {
            syn::Meta::Path(path) => path_last_is(path, "schema_version"),
            syn::Meta::NameValue(name_value) => expression_has_direct_schema(&name_value.value),
            syn::Meta::List(list) => {
                let parser =
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
                match parser.parse2(list.tokens.clone()) {
                    Ok(arguments) => arguments.iter().any(expression_has_direct_schema),
                    Err(error) => {
                        analysis.violations.push(format!(
                            "{owner}: unparsed {} attribute tokens: {error}",
                            macro_path_from_path(&list.path)
                        ));
                        false
                    }
                }
            }
        };
        if contains_schema {
            analysis.direct_uses.push(owner.to_string());
            analysis
                .violations
                .push(format!("{owner}: schema_version attribute token"));
        }
    }
}

fn analyze_schema_expression(
    owner: String,
    expression: &syn::Expr,
    analysis: &mut SchemaPolicyAnalysis,
) {
    let aliases = HashSet::new();
    SchemaExpressionVisitor {
        owner: &owner,
        aliases: &aliases,
        analysis,
    }
    .visit_expr(expression);
}

fn analyze_schema_macro(
    owner: String,
    expression: &syn::Macro,
    analysis: &mut SchemaPolicyAnalysis,
) {
    let aliases = HashSet::new();
    SchemaExpressionVisitor {
        owner: &owner,
        aliases: &aliases,
        analysis,
    }
    .inspect_macro(expression);
}

fn analyze_schema_block(owner: String, block: &syn::Block, analysis: &mut SchemaPolicyAnalysis) {
    let mut bindings = BindingCollector::default();
    bindings.visit_block(block);
    let aliases = bindings.schema_aliases();
    let mut visitor = SchemaExpressionVisitor {
        owner: &owner,
        aliases: &aliases,
        analysis,
    };
    visitor.visit_block(block);
}

#[derive(Default)]
struct BindingCollector {
    assignments: Vec<(String, bool, HashSet<String>)>,
}

impl BindingCollector {
    fn schema_aliases(&self) -> HashSet<String> {
        let mut aliases = HashSet::new();
        loop {
            let mut changed = false;
            for (name, has_direct_schema, dependencies) in &self.assignments {
                if (*has_direct_schema || !dependencies.is_disjoint(&aliases))
                    && aliases.insert(name.clone())
                {
                    changed = true;
                }
            }
            if !changed {
                return aliases;
            }
        }
    }

    fn record_pattern(&mut self, pattern: &syn::Pat, expression: &syn::Expr) {
        let direct_schema = expression_has_direct_schema(expression);
        let dependencies = expression_identifiers(expression);
        for name in pattern_identifiers(pattern) {
            self.assignments
                .push((name, direct_schema, dependencies.clone()));
        }
    }

    fn record_assignment(&mut self, target: &syn::Expr, expression: &syn::Expr) {
        let direct_schema = expression_has_direct_schema(expression);
        let dependencies = expression_identifiers(expression);
        for name in assignment_identifiers(target) {
            self.assignments
                .push((name, direct_schema, dependencies.clone()));
        }
    }
}

impl<'ast> Visit<'ast> for BindingCollector {
    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        if !matches!(statement, syn::Stmt::Item(_)) {
            visit::visit_stmt(self, statement);
        }
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.record_pattern(&local.pat, &init.expr);
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_assign(&mut self, assignment: &'ast syn::ExprAssign) {
        self.record_assignment(&assignment.left, &assignment.right);
        visit::visit_expr_assign(self, assignment);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.record_pattern(&expression.pat, &expression.expr);
        visit::visit_expr_for_loop(self, expression);
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.record_pattern(&expression.pat, &expression.expr);
        visit::visit_expr_let(self, expression);
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        for arm in &expression.arms {
            self.record_pattern(&arm.pat, &expression.expr);
        }
        visit::visit_expr_match(self, expression);
    }
}

fn pattern_identifiers(pattern: &syn::Pat) -> HashSet<String> {
    #[derive(Default)]
    struct Collector(HashSet<String>);

    impl<'ast> Visit<'ast> for Collector {
        fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
            self.0.insert(identifier_name(&pattern.ident));
            visit::visit_pat_ident(self, pattern);
        }
    }

    let mut collector = Collector::default();
    collector.visit_pat(pattern);
    collector.0
}

fn assignment_identifiers(target: &syn::Expr) -> HashSet<String> {
    #[derive(Default)]
    struct Collector(HashSet<String>);

    impl<'ast> Visit<'ast> for Collector {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if let Some(identifier) = path.path.get_ident() {
                self.0.insert(identifier_name(identifier));
            }
            visit::visit_expr_path(self, path);
        }
    }

    let mut collector = Collector::default();
    collector.visit_expr(target);
    collector.0
}

enum ParsedMacroArguments {
    Expressions(Vec<syn::Expr>),
    Matches(Box<MatchesMacroArguments>),
}

struct MatchesMacroArguments {
    expression: syn::Expr,
    pattern: syn::Pat,
    guard: Option<syn::Expr>,
}

impl Parse for MatchesMacroArguments {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expression = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let pattern = syn::Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(syn::Token![if]) {
            input.parse::<syn::Token![if]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("unexpected tokens after matches! arguments"));
        }
        Ok(Self {
            expression,
            pattern,
            guard,
        })
    }
}

fn parse_macro_arguments(expression: &syn::Macro) -> syn::Result<ParsedMacroArguments> {
    let macro_name = macro_path(expression);
    if macro_name == "macro_rules" {
        return Err(syn::Error::new_spanned(
            &expression.path,
            "macro_rules bodies are not accepted by the production AST policy",
        ));
    }
    if macro_name == "matches" {
        return syn::parse2(expression.tokens.clone())
            .map(Box::new)
            .map(ParsedMacroArguments::Matches);
    }
    if !matches!(
        macro_name.as_str(),
        "anyhow::anyhow" | "bail" | "format" | "println" | "vec" | "writeln"
    ) {
        return Err(syn::Error::new_spanned(
            &expression.path,
            "macro is not in the production AST policy allowlist",
        ));
    }

    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    parser
        .parse2(expression.tokens.clone())
        .map(|expressions| ParsedMacroArguments::Expressions(expressions.into_iter().collect()))
}

fn macro_path(expression: &syn::Macro) -> String {
    macro_path_from_path(&expression.path)
}

fn macro_path_from_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| identifier_name(&segment.ident))
        .collect::<Vec<_>>()
        .join("::")
}

fn qualify_owner(lineage: Option<&str>, owner: &str) -> String {
    lineage.map_or_else(
        || owner.to_string(),
        |lineage| format!("{lineage}::{owner}"),
    )
}

fn impl_identity(item_impl: &syn::ItemImpl, type_path: &str) -> String {
    item_impl.trait_.as_ref().map_or_else(
        || type_path.to_string(),
        |(_, trait_path, _)| format!("<{type_path} as {}>", macro_path_from_path(trait_path)),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportBinding {
    source: String,
    binding: Option<String>,
}

fn import_bindings(item_use: &syn::ItemUse) -> Vec<ImportBinding> {
    fn collect(tree: &syn::UseTree, prefix: &[String], bindings: &mut Vec<ImportBinding>) {
        match tree {
            syn::UseTree::Path(path) => {
                let mut nested = prefix.to_vec();
                nested.push(identifier_name(&path.ident));
                collect(&path.tree, &nested, bindings);
            }
            syn::UseTree::Name(name) => {
                let identifier = identifier_name(&name.ident);
                let (source, binding) = if identifier == "self" {
                    (
                        prefix.join("::"),
                        prefix.last().cloned().unwrap_or(identifier),
                    )
                } else {
                    let mut source = prefix.to_vec();
                    source.push(identifier.clone());
                    (source.join("::"), identifier)
                };
                bindings.push(ImportBinding {
                    source,
                    binding: Some(binding),
                });
            }
            syn::UseTree::Rename(rename) => {
                let identifier = identifier_name(&rename.ident);
                let source = if identifier == "self" {
                    prefix.join("::")
                } else {
                    let mut source = prefix.to_vec();
                    source.push(identifier);
                    source.join("::")
                };
                bindings.push(ImportBinding {
                    source,
                    binding: Some(identifier_name(&rename.rename)),
                });
            }
            syn::UseTree::Glob(_) => bindings.push(ImportBinding {
                source: format!("{}::*", prefix.join("::")),
                binding: None,
            }),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    collect(tree, prefix, bindings);
                }
            }
        }
    }

    let mut bindings = Vec::new();
    collect(&item_use.tree, &[], &mut bindings);
    bindings
}

fn import_owner(lineage: Option<&str>, binding: &ImportBinding) -> String {
    let label = binding
        .binding
        .as_deref()
        .map_or_else(|| binding.source.as_str(), |binding| binding);
    qualify_owner(lineage, &format!("use {label}"))
}

fn untrusted_allowlisted_macro_import(binding: &ImportBinding) -> bool {
    let Some(name) = binding.binding.as_deref() else {
        return false;
    };
    match name {
        "anyhow" => binding.source != "anyhow",
        "bail" => binding.source != "anyhow::bail",
        "format" | "matches" | "println" | "vec" | "writeln" => true,
        _ => false,
    }
}

fn analyze_schema_imports(
    module: Option<&str>,
    item_use: &syn::ItemUse,
    analysis: &mut SchemaPolicyAnalysis,
) {
    for binding in import_bindings(item_use) {
        let owner = import_owner(module, &binding);
        analyze_schema_attributes(&owner, &item_use.attrs, analysis);
        let Some(name) = binding.binding.as_deref() else {
            analysis.violations.push(format!(
                "{owner}: unclassified glob import `{}` can obscure schema or macro provenance",
                binding.source
            ));
            continue;
        };
        if name == "schema_version"
            || binding
                .source
                .split("::")
                .any(|segment| segment == "schema_version")
        {
            analysis.direct_uses.push(owner.clone());
            analysis.violations.push(format!(
                "{owner}: schema_version import `{}` binds `{name}`",
                binding.source
            ));
        }
        if untrusted_allowlisted_macro_import(&binding) {
            analysis.violations.push(format!(
                "{owner}: untrusted allowlisted macro import `{}` binds `{name}`",
                binding.source
            ));
        }
    }
}

fn analyze_render_imports(
    module: &str,
    item_use: &syn::ItemUse,
    inventory: &mut RenderCallInventory,
) {
    for binding in import_bindings(item_use) {
        let owner = import_owner(Some(module), &binding);
        for attribute in &item_use.attrs {
            RenderCallVisitor {
                owner: owner.clone(),
                inventory,
            }
            .visit_attribute(attribute);
        }
        let Some(name) = binding.binding.as_deref() else {
            inventory.violations.push(format!(
                "{owner}: unclassified glob import `{}` can obscure macro provenance",
                binding.source
            ));
            continue;
        };
        if untrusted_allowlisted_macro_import(&binding) {
            inventory.violations.push(format!(
                "{owner}: untrusted allowlisted macro import `{}` binds `{name}`",
                binding.source
            ));
        }
    }
}

struct SchemaExpressionVisitor<'a> {
    owner: &'a str,
    aliases: &'a HashSet<String>,
    analysis: &'a mut SchemaPolicyAnalysis,
}

impl SchemaExpressionVisitor<'_> {
    fn violation(&mut self, kind: &str) {
        self.analysis
            .violations
            .push(format!("{}: {kind}", self.owner));
    }

    fn visit_parsed_expression(&mut self, expression: &syn::Expr) {
        SchemaExpressionVisitor {
            owner: self.owner,
            aliases: self.aliases,
            analysis: self.analysis,
        }
        .visit_expr(expression);
    }

    fn visit_parsed_pattern(&mut self, pattern: &syn::Pat) {
        SchemaExpressionVisitor {
            owner: self.owner,
            aliases: self.aliases,
            analysis: self.analysis,
        }
        .visit_pat(pattern);
    }

    fn inspect_macro(&mut self, expression: &syn::Macro) {
        match parse_macro_arguments(expression) {
            Ok(ParsedMacroArguments::Expressions(arguments)) => {
                for argument in &arguments {
                    self.visit_parsed_expression(argument);
                }
            }
            Ok(ParsedMacroArguments::Matches(arguments)) => {
                if expression_depends_on_schema(&arguments.expression, self.aliases) {
                    self.violation("schema-version matches! predicate");
                }
                if arguments
                    .guard
                    .as_ref()
                    .is_some_and(|guard| expression_depends_on_schema(guard, self.aliases))
                {
                    self.violation("schema-version matches! guard");
                }
                self.visit_parsed_expression(&arguments.expression);
                self.visit_parsed_pattern(&arguments.pattern);
                if let Some(guard) = &arguments.guard {
                    self.visit_parsed_expression(guard);
                }
            }
            Err(error) => self.violation(&format!(
                "unparsed {}! macro: {error}",
                macro_path(expression)
            )),
        }
    }
}

impl<'ast> Visit<'ast> for SchemaExpressionVisitor<'_> {
    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        if let syn::Stmt::Item(item) = statement {
            analyze_schema_items(std::slice::from_ref(item), Some(self.owner), self.analysis);
        } else {
            visit::visit_stmt(self, statement);
        }
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        analyze_schema_attributes(self.owner, std::slice::from_ref(attribute), self.analysis);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if local.init.as_ref().is_some_and(|init| {
            init.diverge.is_some() && expression_depends_on_schema(&init.expr, self.aliases)
        }) {
            self.violation("schema-version let-else initializer");
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
        if member_is(&field.member, "schema_version") {
            self.analysis.direct_uses.push(self.owner.to_string());
        }
        visit::visit_expr_field(self, field);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path_last_is(&path.path, "schema_version") {
            self.analysis.direct_uses.push(self.owner.to_string());
            if path.qself.is_none() && path_is_ident(&path.path, "schema_version") {
                self.violation("bare schema_version path");
            } else {
                self.violation("qualified schema_version path");
            }
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
        if expression_depends_on_schema(&syn::Expr::Binary(binary.clone()), self.aliases) {
            if is_exact_v0_8_schema_predicate(binary) {
                self.analysis.exact_predicates.push(self.owner.to_string());
            } else {
                self.violation("non-exact schema-version binary predicate");
            }
        }
        visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        if expression_depends_on_schema(&expression.cond, self.aliases)
            && !matches!(expression.cond.as_ref(), syn::Expr::Binary(binary) if is_exact_v0_8_schema_predicate(binary))
        {
            self.violation("schema-version alias or nesting in if condition");
        }
        visit::visit_expr_if(self, expression);
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        if expression_depends_on_schema(&expression.cond, self.aliases) {
            self.violation("schema-version while condition");
        }
        visit::visit_expr_while(self, expression);
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        if expression_depends_on_schema(&expression.expr, self.aliases) {
            self.violation("schema-version for-loop iterator");
        }
        visit::visit_expr_for_loop(self, expression);
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        if expression_depends_on_schema(&expression.expr, self.aliases) {
            self.violation("schema-version let predicate");
        }
        visit::visit_expr_let(self, expression);
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        if expression_depends_on_schema(&expression.expr, self.aliases) {
            self.violation("schema-version match scrutinee");
        }
        for arm in &expression.arms {
            if arm
                .guard
                .as_ref()
                .is_some_and(|(_, guard)| expression_depends_on_schema(guard, self.aliases))
            {
                self.violation("schema-version match guard");
            }
        }
        visit::visit_expr_match(self, expression);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if expression_depends_on_schema(&syn::Expr::Call(expression.clone()), self.aliases) {
            self.violation("schema-version function-call dependency");
        }
        visit::visit_expr_call(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if expression_depends_on_schema(&syn::Expr::MethodCall(expression.clone()), self.aliases) {
            self.violation("schema-version method-call dependency");
        }
        visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        if expression
            .fields
            .iter()
            .any(|field| member_is(&field.member, "schema_version"))
        {
            self.violation("schema_version struct field construction");
        }
        visit::visit_expr_struct(self, expression);
    }

    fn visit_pat_struct(&mut self, pattern: &'ast syn::PatStruct) {
        if pattern
            .fields
            .iter()
            .any(|field| member_is(&field.member, "schema_version"))
        {
            self.violation("schema_version struct pattern");
        }
        visit::visit_pat_struct(self, pattern);
    }

    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        if identifier_is(&pattern.ident, "schema_version") {
            self.violation("bare schema_version pattern binding");
        }
        visit::visit_pat_ident(self, pattern);
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        self.inspect_macro(expression);
    }
}

fn expression_has_direct_schema(expression: &syn::Expr) -> bool {
    #[derive(Default)]
    struct Finder(bool);
    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
            if member_is(&field.member, "schema_version") {
                self.0 = true;
            }
            visit::visit_expr_field(self, field);
        }

        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if path_last_is(&path.path, "schema_version") {
                self.0 = true;
            }
            visit::visit_expr_path(self, path);
        }

        fn visit_macro(&mut self, expression: &'ast syn::Macro) {
            match parse_macro_arguments(expression) {
                Ok(ParsedMacroArguments::Expressions(arguments)) => {
                    for argument in &arguments {
                        self.visit_expr(argument);
                    }
                }
                Ok(ParsedMacroArguments::Matches(arguments)) => {
                    self.visit_expr(&arguments.expression);
                    if let Some(guard) = &arguments.guard {
                        self.visit_expr(guard);
                    }
                }
                Err(_) => {}
            }
        }
    }
    let mut finder = Finder::default();
    finder.visit_expr(expression);
    finder.0
}

fn expression_identifiers(expression: &syn::Expr) -> HashSet<String> {
    #[derive(Default)]
    struct Collector(HashSet<String>);
    impl<'ast> Visit<'ast> for Collector {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if let Some(identifier) = path.path.segments.last().map(|segment| &segment.ident) {
                self.0.insert(identifier_name(identifier));
            }
            visit::visit_expr_path(self, path);
        }

        fn visit_macro(&mut self, expression: &'ast syn::Macro) {
            match parse_macro_arguments(expression) {
                Ok(ParsedMacroArguments::Expressions(arguments)) => {
                    for argument in &arguments {
                        self.visit_expr(argument);
                    }
                }
                Ok(ParsedMacroArguments::Matches(arguments)) => {
                    self.visit_expr(&arguments.expression);
                    if let Some(guard) = &arguments.guard {
                        self.visit_expr(guard);
                    }
                }
                Err(_) => {}
            }
        }
    }
    let mut collector = Collector::default();
    collector.visit_expr(expression);
    collector.0
}

fn expression_depends_on_schema(expression: &syn::Expr, aliases: &HashSet<String>) -> bool {
    expression_has_direct_schema(expression)
        || !expression_identifiers(expression).is_disjoint(aliases)
}

fn is_exact_v0_8_schema_predicate(binary: &syn::ExprBinary) -> bool {
    matches!(binary.op, syn::BinOp::Eq(_))
        && is_self_checkpoint_schema_version(&binary.left)
        && matches!(
            binary.right.as_ref(),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(literal),
                ..
            }) if literal.value() == "v0.8"
        )
}

fn is_self_checkpoint_schema_version(expression: &syn::Expr) -> bool {
    let syn::Expr::Field(schema) = expression else {
        return false;
    };
    if !member_is(&schema.member, "schema_version") {
        return false;
    }
    let syn::Expr::Field(checkpoint) = schema.base.as_ref() else {
        return false;
    };
    member_is(&checkpoint.member, "checkpoint")
        && matches!(
            checkpoint.base.as_ref(),
            syn::Expr::Path(path) if path_is_ident(&path.path, "self")
        )
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RenderCallInventory {
    counts: BTreeMap<String, usize>,
    violations: Vec<String>,
}

fn expected_render_call_counts() -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("ReplayReport::to_console_text".to_string(), 2usize),
        ("adjudication::shape_request".to_string(), 1usize),
        ("cli::run_live".to_string(), 1usize),
    ])
}

fn render_call_policy_is_exact(inventory: &RenderCallInventory) -> bool {
    inventory.counts == expected_render_call_counts()
        && inventory.counts.values().sum::<usize>() == 4
        && inventory.violations.is_empty()
}

fn analyze_render_item_header(owner: &str, item: &syn::Item, inventory: &mut RenderCallInventory) {
    let mut visitor = RenderCallVisitor {
        owner: owner.to_string(),
        inventory,
    };
    visit_item_header(&mut visitor, item);
}

fn analyze_render_impl_item_header(
    owner: &str,
    item: &syn::ImplItem,
    inventory: &mut RenderCallInventory,
) {
    let mut visitor = RenderCallVisitor {
        owner: owner.to_string(),
        inventory,
    };
    visit_impl_item_header(&mut visitor, item);
}

fn analyze_render_trait_item_header(
    owner: &str,
    item: &syn::TraitItem,
    inventory: &mut RenderCallInventory,
) {
    let mut visitor = RenderCallVisitor {
        owner: owner.to_string(),
        inventory,
    };
    visit_trait_item_header(&mut visitor, item);
}

fn analyze_render_foreign_item_header(
    owner: &str,
    item: &syn::ForeignItem,
    inventory: &mut RenderCallInventory,
) {
    let mut visitor = RenderCallVisitor {
        owner: owner.to_string(),
        inventory,
    };
    visit_foreign_item_header(&mut visitor, item);
}

fn analyze_render_calls_in_items(
    module: &str,
    items: &[syn::Item],
    inventory: &mut RenderCallInventory,
) {
    analyze_render_calls_in_items_with_lineage(module, items, false, inventory);
}

fn analyze_render_calls_in_items_with_lineage(
    module: &str,
    items: &[syn::Item],
    has_enclosing_lineage: bool,
    inventory: &mut RenderCallInventory,
) {
    for item in items {
        if has_cfg_test(item_attrs(item)) {
            continue;
        }
        let lineage = has_enclosing_lineage.then_some(module);
        match item {
            syn::Item::Fn(function) => {
                let owner = format!("{module}::{}", identifier_name(&function.sig.ident));
                analyze_render_item_header(&owner, item, inventory);
                RenderCallVisitor { owner, inventory }.visit_block(&function.block);
            }
            syn::Item::Impl(item_impl) => {
                let type_path = type_owner_name(&item_impl.self_ty).unwrap_or_else(|| {
                    let owner = qualify_owner(lineage, "impl <unclassified self type>");
                    inventory
                        .violations
                        .push(format!("{owner}: unclassified impl self type"));
                    "<unclassified self type>".to_string()
                });
                let impl_identity = impl_identity(item_impl, &type_path);
                let impl_owner = qualify_owner(lineage, &format!("impl {impl_identity}"));
                analyze_render_item_header(&impl_owner, item, inventory);
                for impl_item in &item_impl.items {
                    match impl_item {
                        syn::ImplItem::Fn(method) if !has_cfg_test(&method.attrs) => {
                            let owner = qualify_owner(
                                lineage,
                                &format!("{impl_identity}::{}", identifier_name(&method.sig.ident)),
                            );
                            analyze_render_impl_item_header(&owner, impl_item, inventory);
                            RenderCallVisitor { owner, inventory }.visit_block(&method.block);
                        }
                        syn::ImplItem::Const(constant) if !has_cfg_test(&constant.attrs) => {
                            let owner = qualify_owner(
                                lineage,
                                &format!(
                                    "{impl_identity}::const {}",
                                    identifier_name(&constant.ident)
                                ),
                            );
                            analyze_render_impl_item_header(&owner, impl_item, inventory);
                            RenderCallVisitor { owner, inventory }.visit_expr(&constant.expr);
                        }
                        syn::ImplItem::Type(associated_type)
                            if !has_cfg_test(&associated_type.attrs) =>
                        {
                            let owner = qualify_owner(
                                lineage,
                                &format!(
                                    "{impl_identity}::type {}",
                                    identifier_name(&associated_type.ident)
                                ),
                            );
                            analyze_render_impl_item_header(&owner, impl_item, inventory);
                        }
                        syn::ImplItem::Macro(item_macro) if !has_cfg_test(&item_macro.attrs) => {
                            let owner = qualify_owner(
                                lineage,
                                &format!("{impl_identity}::macro {}", macro_path(&item_macro.mac)),
                            );
                            analyze_render_impl_item_header(&owner, impl_item, inventory);
                            RenderCallVisitor { owner, inventory }.inspect_macro(&item_macro.mac);
                        }
                        syn::ImplItem::Verbatim(_) => inventory.violations.push(format!(
                            "{impl_owner}: unsupported impl item form: verbatim"
                        )),
                        syn::ImplItem::Fn(_)
                        | syn::ImplItem::Const(_)
                        | syn::ImplItem::Type(_)
                        | syn::ImplItem::Macro(_) => {}
                        _ => inventory
                            .violations
                            .push(format!("{impl_owner}: unsupported impl item form: unknown")),
                    }
                }
            }
            syn::Item::Const(constant) => {
                let owner = format!("{module}::const {}", identifier_name(&constant.ident));
                analyze_render_item_header(&owner, item, inventory);
                RenderCallVisitor { owner, inventory }.visit_expr(&constant.expr);
            }
            syn::Item::Static(static_item) => {
                let owner = format!("{module}::static {}", identifier_name(&static_item.ident));
                analyze_render_item_header(&owner, item, inventory);
                RenderCallVisitor { owner, inventory }.visit_expr(&static_item.expr);
            }
            syn::Item::Struct(item_struct) => {
                let owner = qualify_owner(
                    lineage,
                    &format!("struct {}", identifier_name(&item_struct.ident)),
                );
                analyze_render_item_header(&owner, item, inventory);
            }
            syn::Item::Enum(item_enum) => {
                let owner = qualify_owner(
                    lineage,
                    &format!("enum {}", identifier_name(&item_enum.ident)),
                );
                analyze_render_item_header(&owner, item, inventory);
            }
            syn::Item::Union(item_union) => {
                let owner = qualify_owner(
                    lineage,
                    &format!("union {}", identifier_name(&item_union.ident)),
                );
                analyze_render_item_header(&owner, item, inventory);
            }
            syn::Item::Trait(item_trait) => {
                let trait_identity = qualify_owner(
                    lineage,
                    &format!("trait {}", identifier_name(&item_trait.ident)),
                );
                analyze_render_item_header(&trait_identity, item, inventory);
                for trait_item in &item_trait.items {
                    match trait_item {
                        syn::TraitItem::Fn(method) if !has_cfg_test(&method.attrs) => {
                            let owner = format!(
                                "{}::{}",
                                trait_identity,
                                identifier_name(&method.sig.ident)
                            );
                            analyze_render_trait_item_header(&owner, trait_item, inventory);
                            if let Some(default) = &method.default {
                                RenderCallVisitor { owner, inventory }.visit_block(default);
                            }
                        }
                        syn::TraitItem::Const(constant) if !has_cfg_test(&constant.attrs) => {
                            let owner = format!(
                                "{}::const {}",
                                trait_identity,
                                identifier_name(&constant.ident)
                            );
                            analyze_render_trait_item_header(&owner, trait_item, inventory);
                            if let Some((_, default)) = &constant.default {
                                RenderCallVisitor { owner, inventory }.visit_expr(default);
                            }
                        }
                        syn::TraitItem::Type(associated_type)
                            if !has_cfg_test(&associated_type.attrs) =>
                        {
                            let owner = format!(
                                "{}::type {}",
                                trait_identity,
                                identifier_name(&associated_type.ident)
                            );
                            analyze_render_trait_item_header(&owner, trait_item, inventory);
                        }
                        syn::TraitItem::Macro(item_macro) if !has_cfg_test(&item_macro.attrs) => {
                            let owner = format!(
                                "{}::macro {}",
                                trait_identity,
                                macro_path(&item_macro.mac)
                            );
                            analyze_render_trait_item_header(&owner, trait_item, inventory);
                            RenderCallVisitor { owner, inventory }.inspect_macro(&item_macro.mac);
                        }
                        syn::TraitItem::Verbatim(_) => inventory.violations.push(format!(
                            "{trait_identity}: unsupported trait item form: verbatim"
                        )),
                        syn::TraitItem::Fn(_)
                        | syn::TraitItem::Const(_)
                        | syn::TraitItem::Type(_)
                        | syn::TraitItem::Macro(_) => {}
                        _ => inventory.violations.push(format!(
                            "{trait_identity}: unsupported trait item form: unknown"
                        )),
                    }
                }
            }
            syn::Item::TraitAlias(item_alias) => {
                let owner = qualify_owner(
                    lineage,
                    &format!("trait alias {}", identifier_name(&item_alias.ident)),
                );
                analyze_render_item_header(&owner, item, inventory);
            }
            syn::Item::Type(item_type) => {
                let owner = qualify_owner(
                    lineage,
                    &format!("type {}", identifier_name(&item_type.ident)),
                );
                analyze_render_item_header(&owner, item, inventory);
            }
            syn::Item::ForeignMod(foreign_mod) => {
                let foreign_owner = qualify_owner(lineage, "extern block");
                analyze_render_item_header(&foreign_owner, item, inventory);
                for foreign_item in &foreign_mod.items {
                    match foreign_item {
                        syn::ForeignItem::Fn(function) if !has_cfg_test(&function.attrs) => {
                            let owner = format!(
                                "{foreign_owner}::{}",
                                identifier_name(&function.sig.ident)
                            );
                            analyze_render_foreign_item_header(&owner, foreign_item, inventory);
                        }
                        syn::ForeignItem::Static(static_item)
                            if !has_cfg_test(&static_item.attrs) =>
                        {
                            let owner = format!(
                                "{foreign_owner}::static {}",
                                identifier_name(&static_item.ident)
                            );
                            analyze_render_foreign_item_header(&owner, foreign_item, inventory);
                        }
                        syn::ForeignItem::Type(foreign_type)
                            if !has_cfg_test(&foreign_type.attrs) =>
                        {
                            let owner = format!(
                                "{foreign_owner}::type {}",
                                identifier_name(&foreign_type.ident)
                            );
                            analyze_render_foreign_item_header(&owner, foreign_item, inventory);
                        }
                        syn::ForeignItem::Macro(item_macro) if !has_cfg_test(&item_macro.attrs) => {
                            let owner =
                                format!("{foreign_owner}::macro {}", macro_path(&item_macro.mac));
                            analyze_render_foreign_item_header(&owner, foreign_item, inventory);
                            RenderCallVisitor { owner, inventory }.inspect_macro(&item_macro.mac);
                        }
                        syn::ForeignItem::Verbatim(_) => inventory.violations.push(format!(
                            "{foreign_owner}: unsupported foreign item form: verbatim"
                        )),
                        syn::ForeignItem::Fn(_)
                        | syn::ForeignItem::Static(_)
                        | syn::ForeignItem::Type(_)
                        | syn::ForeignItem::Macro(_) => {}
                        _ => inventory.violations.push(format!(
                            "{foreign_owner}: unsupported foreign item form: unknown"
                        )),
                    }
                }
            }
            syn::Item::ExternCrate(extern_crate) => {
                let owner = qualify_owner(
                    lineage,
                    &format!("extern crate {}", identifier_name(&extern_crate.ident)),
                );
                analyze_render_item_header(&owner, item, inventory);
            }
            syn::Item::Macro(item_macro) => {
                let owner = format!("{module}::macro {}", macro_path(&item_macro.mac));
                analyze_render_item_header(&owner, item, inventory);
                RenderCallVisitor { owner, inventory }.inspect_macro(&item_macro.mac);
            }
            syn::Item::Mod(item_mod) => {
                let nested_module = format!("{module}::{}", identifier_name(&item_mod.ident));
                analyze_render_item_header(&nested_module, item, inventory);
                if let Some((_, nested)) = &item_mod.content {
                    analyze_render_calls_in_items_with_lineage(
                        &nested_module,
                        nested,
                        true,
                        inventory,
                    );
                }
            }
            syn::Item::Use(item_use) => {
                analyze_render_imports(module, item_use, inventory);
            }
            syn::Item::Verbatim(_) => inventory
                .violations
                .push(format!("{module}: unsupported item form: verbatim")),
            _ => inventory
                .violations
                .push(format!("{module}: unsupported item form: unknown")),
        }
    }
}

struct RenderCallVisitor<'a> {
    owner: String,
    inventory: &'a mut RenderCallInventory,
}

impl RenderCallVisitor<'_> {
    fn record(&mut self) {
        *self.inventory.counts.entry(self.owner.clone()).or_insert(0) += 1;
    }

    fn visit_parsed_expression(&mut self, expression: &syn::Expr) {
        RenderCallVisitor {
            owner: self.owner.clone(),
            inventory: self.inventory,
        }
        .visit_expr(expression);
    }

    fn inspect_macro(&mut self, expression: &syn::Macro) {
        match parse_macro_arguments(expression) {
            Ok(ParsedMacroArguments::Expressions(arguments)) => {
                for argument in &arguments {
                    self.visit_parsed_expression(argument);
                }
            }
            Ok(ParsedMacroArguments::Matches(arguments)) => {
                self.visit_parsed_expression(&arguments.expression);
                if let Some(guard) = &arguments.guard {
                    self.visit_parsed_expression(guard);
                }
            }
            Err(error) => self.inventory.violations.push(format!(
                "{}: unparsed {}! macro in render-call inventory: {error}",
                self.owner,
                macro_path(expression)
            )),
        }
    }
}

impl<'ast> Visit<'ast> for RenderCallVisitor<'_> {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        match &attribute.meta {
            syn::Meta::Path(path) => {
                if path_last_is(path, "render_console_block") {
                    self.inventory.violations.push(format!(
                        "{}: render_console_block attribute path reference",
                        self.owner
                    ));
                }
            }
            syn::Meta::NameValue(name_value) => self.visit_expr(&name_value.value),
            syn::Meta::List(list) => {
                let parser =
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
                match parser.parse2(list.tokens.clone()) {
                    Ok(arguments) => {
                        for argument in &arguments {
                            self.visit_expr(argument);
                        }
                    }
                    Err(error) => {
                        let tokens = list.tokens.to_string();
                        if tokens
                            .split(|character: char| {
                                !character.is_alphanumeric() && character != '_'
                            })
                            .any(|token| token == "render_console_block")
                        {
                            self.inventory.violations.push(format!(
                                "{}: unparsed {} attribute render_console_block tokens: {error}",
                                self.owner,
                                macro_path_from_path(&list.path)
                            ));
                        }
                    }
                }
            }
        }
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        if let syn::Stmt::Item(item) = statement {
            analyze_render_calls_in_items_with_lineage(
                &self.owner,
                std::slice::from_ref(item),
                true,
                self.inventory,
            );
        } else {
            visit::visit_stmt(self, statement);
        }
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        if identifier_is(&expression.method, "render_console_block") {
            self.record();
        }
        visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = expression.func.as_ref() {
            if path
                .path
                .segments
                .last()
                .is_some_and(|segment| identifier_is(&segment.ident, "render_console_block"))
            {
                self.record();
                for argument in &expression.args {
                    self.visit_expr(argument);
                }
                return;
            }
        }
        visit::visit_expr_call(self, expression);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path
            .path
            .segments
            .last()
            .is_some_and(|segment| identifier_is(&segment.ident, "render_console_block"))
        {
            self.inventory.violations.push(format!(
                "{}: unclassified render_console_block path reference",
                self.owner
            ));
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        self.inspect_macro(expression);
    }
}

fn production_rust_sources(root: &Path) -> Vec<PathBuf> {
    fn collect(directory: &Path, sources: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()));
        for entry in entries {
            let path = entry.expect("source directory entry").path();
            if path.is_dir() {
                collect(&path, sources);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                sources.push(path);
            }
        }
    }

    let mut sources = Vec::new();
    collect(root, &mut sources);
    sources.sort();
    sources
}

fn type_owner_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Array(array) => {
            type_owner_name(&array.elem).map(|element| format!("[{element}; const]"))
        }
        syn::Type::Group(group) => type_owner_name(&group.elem),
        syn::Type::Infer(_) => Some("_".to_string()),
        syn::Type::Never(_) => Some("!".to_string()),
        syn::Type::Paren(paren) => type_owner_name(&paren.elem).map(|inner| format!("({inner})")),
        syn::Type::Path(path) if path.qself.is_none() => {
            let identity = path
                .path
                .segments
                .iter()
                .map(|segment| {
                    let arguments = match &segment.arguments {
                        syn::PathArguments::None => "",
                        syn::PathArguments::AngleBracketed(_) => "<...>",
                        syn::PathArguments::Parenthesized(_) => "(...)",
                    };
                    format!("{}{arguments}", identifier_name(&segment.ident))
                })
                .collect::<Vec<_>>()
                .join("::");
            Some(if path.path.leading_colon.is_some() {
                format!("::{identity}")
            } else {
                identity
            })
        }
        syn::Type::Ptr(pointer) => type_owner_name(&pointer.elem).map(|element| {
            let mutability = if pointer.mutability.is_some() {
                "mut"
            } else {
                "const"
            };
            format!("*{mutability} {element}")
        }),
        syn::Type::Reference(reference) => type_owner_name(&reference.elem).map(|element| {
            let mutability = if reference.mutability.is_some() {
                "mut "
            } else {
                ""
            };
            format!("&{mutability}{element}")
        }),
        syn::Type::Slice(slice) => {
            type_owner_name(&slice.elem).map(|element| format!("[{element}]"))
        }
        syn::Type::Tuple(tuple) => {
            let elements = tuple
                .elems
                .iter()
                .map(type_owner_name)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("({})", elements.join(", ")))
        }
        _ => None,
    }
}

fn member_is(member: &syn::Member, expected: &str) -> bool {
    matches!(member, syn::Member::Named(identifier) if identifier_is(identifier, expected))
}

fn identifier_is(identifier: &syn::Ident, expected: &str) -> bool {
    identifier.unraw() == expected
}

fn identifier_name(identifier: &syn::Ident) -> String {
    identifier.unraw().to_string()
}

fn path_is_ident(path: &syn::Path, expected: &str) -> bool {
    path.get_ident()
        .is_some_and(|identifier| identifier_is(identifier, expected))
}

fn path_last_is(path: &syn::Path, expected: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| identifier_is(&segment.ident, expected))
}

fn has_cfg_test(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        path_is_ident(attribute.path(), "cfg")
            && attribute.parse_args::<syn::Meta>().is_ok_and(
                |meta| matches!(meta, syn::Meta::Path(path) if path_is_ident(&path, "test")),
            )
    })
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(item) => &item.attrs,
        syn::Item::Enum(item) => &item.attrs,
        syn::Item::ExternCrate(item) => &item.attrs,
        syn::Item::Fn(item) => &item.attrs,
        syn::Item::ForeignMod(item) => &item.attrs,
        syn::Item::Impl(item) => &item.attrs,
        syn::Item::Macro(item) => &item.attrs,
        syn::Item::Mod(item) => &item.attrs,
        syn::Item::Static(item) => &item.attrs,
        syn::Item::Struct(item) => &item.attrs,
        syn::Item::Trait(item) => &item.attrs,
        syn::Item::TraitAlias(item) => &item.attrs,
        syn::Item::Type(item) => &item.attrs,
        syn::Item::Union(item) => &item.attrs,
        syn::Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

fn checkpoint_with_selected_evidence_groups(
    schema_version: &str,
    first_group_reasons: &[&str],
    later_group_reasons: &[&str],
) -> Checkpoint {
    let mut checkpoint = support::checkpoint(
        "session-evidence-groups",
        1,
        80,
        true,
        "preserve selected evidence group ordering",
    );
    checkpoint.schema_version = schema_version.to_string();

    let evidence = |reasons: &[&str], historical: bool| {
        reasons
            .iter()
            .map(|reason| EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason: if historical {
                    format!("historical repeated failure evidence: {reason}")
                } else {
                    (*reason).to_string()
                },
            })
            .collect::<Vec<_>>()
    };

    let mut first_score = checkpoint.drift_scores[0].clone();
    first_score.class = DriftClass::WrongPlanBranch;
    first_score.state = DriftState::Active;
    first_score.flagged = true;
    first_score.evidence = evidence(first_group_reasons, false);

    let mut later_score = first_score.clone();
    later_score.class = DriftClass::DeadEndThrash;
    later_score.state = if schema_version == "v0.2" {
        DriftState::Cleared
    } else {
        DriftState::Recovered
    };
    later_score.flagged = false;
    later_score.evidence = evidence(later_group_reasons, schema_version == "v0.2");

    checkpoint.diagnostics.evidence_item_count =
        first_score.evidence.len() + later_score.evidence.len();
    checkpoint.drift_scores = vec![later_score, first_score];
    checkpoint
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_with_drift(
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.flagged = flagged;
    checkpoint.drift_scores[0].class = class;
    checkpoint.drift_scores[0].raw_score = raw_score;
    checkpoint.drift_scores[0].flagged = flagged;
    checkpoint.drift_scores[0].evidence = evidence_reasons
        .iter()
        .map(|reason| EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: (*reason).to_string(),
        })
        .collect();
    checkpoint.diagnostics.evidence_item_count = checkpoint.drift_scores[0].evidence.len();
    checkpoint
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_with_state(
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    state: DriftState,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> Checkpoint {
    checkpoint_with_schema_state(
        "v0.4",
        session_id,
        ordinal,
        class,
        state,
        raw_score,
        flagged,
        expected_next_step,
        evidence_reasons,
    )
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_with_schema_state(
    schema_version: &str,
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    state: DriftState,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = schema_version.to_string();
    if schema_version == "v0.4" {
        checkpoint.turn_context = Some(sample_turn_context(ordinal));
    }
    checkpoint.flagged = flagged;
    checkpoint.drift_scores[0].class = class;
    checkpoint.drift_scores[0].state = state;
    checkpoint.drift_scores[0].raw_score = raw_score;
    checkpoint.drift_scores[0].flagged = flagged;
    checkpoint.drift_scores[0].evidence = evidence_reasons
        .iter()
        .map(|reason| EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: (*reason).to_string(),
        })
        .collect();
    checkpoint.diagnostics.evidence_item_count = checkpoint.drift_scores[0].evidence.len();
    checkpoint
}

fn sample_turn_context(turn_ordinal: usize) -> TurnContext {
    TurnContext {
        turn_id: Some(format!("turn-{turn_ordinal}")),
        turn_ordinal,
        rows_since_turn_start: 4,
        seconds_since_turn_start: Some(12),
        checkpoints_in_turn: 2,
        prompts_observed_in_session: turn_ordinal,
        execution_mode: TurnExecutionMode::Autonomous,
        activity_mix: TurnActivityMix {
            directive_row_count: 1,
            assistant_message_count: 1,
            tool_call_count: 1,
            read_like_command_count: 1,
            write_like_command_count: 1,
            verification_like_command_count: 0,
            tool_output_count: 1,
        },
    }
}

fn sample_session_archetype(
    checkpoint: &Checkpoint,
    label: SessionArchetypeLabel,
) -> SessionArchetype {
    SessionArchetype {
        label,
        confidence: Confidence::Medium,
        supporting_evidence: vec![
            EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason:
                    "stable working set plus source edits supported concentrated implementation"
                        .to_string(),
            },
            EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason: "source edit command strengthened implementation-like evidence"
                    .to_string(),
            },
            EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason: "local verification against source scope also supported implementation follow-through"
                    .to_string(),
            },
        ],
        counter_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "inspection-style command widened the visible search space".to_string(),
        }],
    }
}

fn sample_session_progress(
    checkpoint: &Checkpoint,
    status: ProgressStatus,
    dimension: ProgressDimension,
) -> SessionProgress {
    SessionProgress {
        status,
        dimension,
        confidence: Confidence::Medium,
        signals: Vec::new(),
        supporting_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "progress evidence stayed focused on the active verification target"
                .to_string(),
        }],
        counter_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "one branch of the session remained open while verification narrowed"
                .to_string(),
        }],
    }
}

fn assert_progress_line_after_archetype(rendered: &str) {
    let lines = rendered.lines().collect::<Vec<_>>();
    let archetype_index = lines
        .iter()
        .position(|line| line.starts_with("- Archetype: "))
        .expect("archetype line");
    let progress_index = lines
        .iter()
        .position(|line| line.starts_with("- Progress: "))
        .expect("progress line");

    assert_eq!(progress_index, archetype_index + 1);
}
