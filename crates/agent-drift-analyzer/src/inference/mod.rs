use std::collections::BTreeSet;

use agent_session_compactor::{CompactionKind, CompactionRow};
use camino::Utf8Path;
use serde_json::Value;

use crate::checkpoint::{Confidence, EvidenceRef, TaskFrame};
use crate::context::{row_text_is_focusable, ContextPack};
use crate::input::{extract_path_hints, parse_tool_payload, BundleSession};

const EXPLICIT_DELEGATION_MARKERS: [&str; 4] =
    ["multi_agent_v1", "spawn_agent", "wait_agent", "close_agent"];
const OPAQUE_CHILD_CONTEXT_SIGNALS: [&str; 3] =
    ["child session id", "child rollout", "separate rollout"];
const EXPLICIT_CHILD_LINK_SIGNALS: [&str; 4] = [
    "child session id",
    "child rollout",
    "subagent",
    "spawned agent",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DelegationContext {
    pub topology: Option<DelegationTopology>,
    pub child_work_visibility: Option<ChildWorkVisibility>,
    pub confidence: Option<Confidence>,
    pub markers: Vec<String>,
    pub supporting_evidence: Vec<EvidenceRef>,
    pub counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum DelegationTopology {
    SingleAgent,
    DelegatingParent,
    DelegatedChild,
    MixedOrAmbiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ChildWorkVisibility {
    None,
    Partial,
    Opaque,
}

pub fn infer_task_frame(context: &ContextPack) -> TaskFrame {
    let truth_artifacts = context
        .truth_artifacts
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let working_set_paths = context
        .working_set_paths
        .iter()
        .map(|path| path.path.clone())
        .collect::<Vec<_>>();
    let tools = context.tools.iter().map(|tool| tool.name.clone()).collect();
    let counter_evidence = infer_counter_evidence(context, &truth_artifacts);
    let confidence = infer_confidence(context, &counter_evidence);

    TaskFrame {
        objective: context.objective.text.clone(),
        confidence,
        truth_artifacts,
        working_set_paths,
        tools,
        command_families: context.command_families.clone(),
        verification_commands: context.objective.verification_commands.clone(),
        supporting_evidence: context.supporting_evidence.clone(),
        counter_evidence,
    }
}

pub(crate) fn infer_delegation_context(
    session: &BundleSession,
    context: &ContextPack,
) -> DelegationContext {
    let marker_evidence = collect_marker_evidence(delegation_rows(session));
    let context_signal_evidence = collect_context_signal_evidence(
        session,
        context,
        &session.session_id,
        !marker_evidence.markers.is_empty(),
    );
    let supporting_evidence = dedupe_evidence(
        marker_evidence.evidence.iter().chain(
            context_signal_evidence
                .iter()
                .map(|evidence| &evidence.evidence),
        ),
    );
    let counter_evidence =
        infer_delegation_counter_evidence(context, &marker_evidence, &context_signal_evidence);

    DelegationContext {
        topology: None,
        child_work_visibility: None,
        confidence: None,
        markers: marker_evidence.markers,
        supporting_evidence,
        counter_evidence,
    }
}

fn infer_counter_evidence(context: &ContextPack, truth_artifacts: &[String]) -> Vec<EvidenceRef> {
    let truth_set = truth_artifacts.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();
    for command in &context.command_observations {
        for path in &command.paths {
            if !truth_set.contains(path) {
                for item in &command.evidence {
                    let candidate = EvidenceRef {
                        row: item.row.clone(),
                        reason: format!(
                            "observed path outside explicit truth artifact set: {path}"
                        ),
                    };
                    let key = format!(
                        "{}:{}:{}:{}",
                        candidate.row.source_file,
                        candidate.row.event_index,
                        candidate.row.row_ordinal,
                        candidate.reason
                    );
                    if seen.insert(key) {
                        evidence.push(candidate);
                    }
                }
            }
        }
    }
    evidence
}

fn infer_confidence(context: &ContextPack, counter_evidence: &[EvidenceRef]) -> Confidence {
    let has_objective = !context.objective.text.trim().is_empty()
        && context.objective.text != "No objective row available";
    let has_truth = !context.truth_artifacts.is_empty();
    let has_working_set = !context.working_set_paths.is_empty();
    let has_commands = !context.command_observations.is_empty();

    match (
        has_objective,
        has_truth || has_working_set,
        has_commands,
        counter_evidence.len(),
    ) {
        (true, true, true, 0..=2) => Confidence::High,
        (true, true, _, _) | (true, _, true, _) => Confidence::Medium,
        _ => Confidence::Low,
    }
}

fn delegation_rows(session: &BundleSession) -> impl Iterator<Item = &CompactionRow> + '_ {
    session
        .archival_rows
        .iter()
        .chain(session.compact_rows.iter())
}

#[derive(Debug, Default)]
struct MarkerEvidence {
    markers: Vec<String>,
    evidence: Vec<EvidenceRef>,
}

fn collect_marker_evidence<'a>(
    rows: impl IntoIterator<Item = &'a CompactionRow>,
) -> MarkerEvidence {
    let mut seen_markers = BTreeSet::new();
    let mut seen_evidence = BTreeSet::new();
    let mut markers = Vec::new();
    let mut evidence = Vec::new();

    for row in rows {
        let Some(marker) = explicit_marker_from_row(row) else {
            continue;
        };

        if seen_markers.insert(marker) {
            markers.push(marker.to_string());
        }

        let candidate = EvidenceRef {
            row: agent_session_compactor::RowRef::from_row(row),
            reason: format!("delegation marker: {marker}"),
        };
        let key = evidence_key(&candidate);
        if seen_evidence.insert(key) {
            evidence.push(candidate);
        }
    }

    MarkerEvidence { markers, evidence }
}

#[derive(Debug, Clone)]
struct ContextSignalEvidence {
    category: ContextSignalCategory,
    visibility: Option<ChildWorkVisibility>,
    evidence: EvidenceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextSignalCategory {
    Visibility,
}

fn collect_context_signal_evidence(
    session: &BundleSession,
    context: &ContextPack,
    session_id: &str,
    has_explicit_markers: bool,
) -> Vec<ContextSignalEvidence> {
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();

    if !has_explicit_markers {
        return evidence;
    }

    for candidate in collect_row_signal_evidence(session, session_id) {
        let key = evidence_key(&candidate.evidence);
        if seen.insert(key) {
            evidence.push(candidate);
        }
    }

    for candidate in collect_directive_signal_evidence(session, context, session_id) {
        let key = evidence_key(&candidate.evidence);
        if seen.insert(key) {
            evidence.push(candidate);
        }
    }

    evidence
}

fn collect_row_signal_evidence(
    session: &BundleSession,
    session_id: &str,
) -> Vec<ContextSignalEvidence> {
    let mut evidence = Vec::new();
    for row in delegation_rows(session) {
        let Some(surface) = visibility_surface(row) else {
            continue;
        };

        let lowered_surface = surface.to_ascii_lowercase();
        let has_explicit_child_link = EXPLICIT_CHILD_LINK_SIGNALS
            .iter()
            .any(|signal| lowered_surface.contains(signal));
        let separate_child_rollouts = collect_separate_child_rollout_paths(&surface, session_id);
        let separate_child_session_ids = collect_separate_child_session_ids(&surface, session_id);

        for (path, child_session_id) in &separate_child_rollouts {
            if has_explicit_child_link {
                evidence.push(ContextSignalEvidence {
                    category: ContextSignalCategory::Visibility,
                    visibility: Some(ChildWorkVisibility::Partial),
                    evidence: EvidenceRef {
                        row: agent_session_compactor::RowRef::from_row(row),
                        reason: format!(
                            "delegation child rollout surface links child/subagent work: {path} (session {child_session_id})"
                        ),
                    },
                });
            }
        }

        let has_opaque_signal = OPAQUE_CHILD_CONTEXT_SIGNALS
            .iter()
            .any(|signal| lowered_surface.contains(signal));
        if has_explicit_child_link
            && has_opaque_signal
            && (!separate_child_rollouts.is_empty() || !separate_child_session_ids.is_empty())
        {
            let reason = if let Some(child_session_id) = separate_child_session_ids.first() {
                format!(
                    "delegation row surface references separate child session id: {child_session_id}"
                )
            } else {
                "delegation row surface references anchored child rollout".to_string()
            };
            evidence.push(ContextSignalEvidence {
                category: ContextSignalCategory::Visibility,
                visibility: Some(ChildWorkVisibility::Opaque),
                evidence: EvidenceRef {
                    row: agent_session_compactor::RowRef::from_row(row),
                    reason,
                },
            });
        }
    }
    evidence
}

fn collect_directive_signal_evidence(
    session: &BundleSession,
    context: &ContextPack,
    session_id: &str,
) -> Vec<ContextSignalEvidence> {
    let mut evidence = Vec::new();

    for row in delegation_rows(session).filter(|row| {
        row_text_is_focusable(row)
            && matches!(
                row.kind,
                CompactionKind::UserMessage
                    | CompactionKind::DeveloperMessage
                    | CompactionKind::SystemMessage
            )
    }) {
        let lowered = row.text.to_ascii_lowercase();
        let mentions_child_surface = EXPLICIT_CHILD_LINK_SIGNALS
            .iter()
            .chain(OPAQUE_CHILD_CONTEXT_SIGNALS.iter())
            .any(|signal| lowered.contains(signal));
        if !mentions_child_surface {
            continue;
        }

        let separate_child_rollouts = extract_path_hints(&row.text)
            .into_iter()
            .filter_map(|path| {
                let child_session_id =
                    separate_child_rollout_session_id(&path, session_id)?.to_string();
                Some((path, child_session_id))
            })
            .collect::<Vec<_>>();

        for (path, child_session_id) in &separate_child_rollouts {
            let already_observed_with_delegation_visibility =
                context.command_observations.iter().any(|command| {
                    command_observation_has_child_visibility_evidence(command, path, session_id)
                });
            if already_observed_with_delegation_visibility {
                continue;
            }

            evidence.push(ContextSignalEvidence {
                category: ContextSignalCategory::Visibility,
                visibility: Some(ChildWorkVisibility::Opaque),
                evidence: EvidenceRef {
                    row: agent_session_compactor::RowRef::from_row(row),
                    reason: format!(
                        "delegation directive surface references separate child rollout: {path} (session {child_session_id})"
                    ),
                },
            });
        }

        if separate_child_rollouts.is_empty() {
            for child_session_id in collect_separate_child_session_ids(&row.text, session_id) {
                evidence.push(ContextSignalEvidence {
                    category: ContextSignalCategory::Visibility,
                    visibility: Some(ChildWorkVisibility::Opaque),
                    evidence: EvidenceRef {
                        row: agent_session_compactor::RowRef::from_row(row),
                        reason: format!(
                            "delegation directive surface references separate child session id: {child_session_id}"
                        ),
                    },
                });
            }
        }
    }

    evidence
}

fn command_observation_has_child_visibility_evidence(
    command: &crate::context::CommandObservation,
    path: &str,
    session_id: &str,
) -> bool {
    if !command.paths.iter().any(|candidate| candidate == path) {
        return false;
    }

    if separate_child_rollout_session_id(path, session_id).is_none() {
        return false;
    }

    let lowered = command.raw_command.to_ascii_lowercase();
    EXPLICIT_CHILD_LINK_SIGNALS
        .iter()
        .any(|signal| lowered.contains(signal))
}

fn visibility_surface(row: &CompactionRow) -> Option<String> {
    match row.kind {
        CompactionKind::ToolCall => {
            let payload = parse_tool_payload(&row.text);
            let surface = payload
                .as_ref()
                .and_then(|value| {
                    value
                        .get("command")
                        .or_else(|| value.get("cmd"))
                        .and_then(Value::as_str)
                })
                .unwrap_or_else(|| row.text.as_str());
            Some(surface.to_string())
        }
        CompactionKind::ToolOutput => Some(row.text.clone()),
        _ => None,
    }
}

fn dedupe_evidence<'a>(items: impl Iterator<Item = &'a EvidenceRef>) -> Vec<EvidenceRef> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for evidence in items {
        let key = evidence_key(evidence);
        if seen.insert(key) {
            deduped.push(evidence.clone());
        }
    }
    deduped
}

fn explicit_marker_from_row(row: &CompactionRow) -> Option<&'static str> {
    let tool_name = tool_name_from_row(row)?;
    EXPLICIT_DELEGATION_MARKERS
        .iter()
        .copied()
        .find(|marker| *marker == tool_name)
}

fn tool_name_from_row(row: &CompactionRow) -> Option<String> {
    row.dedupe_identity
        .as_deref()
        .and_then(|identity| serde_json::from_str::<Value>(identity).ok())
        .and_then(|value| {
            value
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn has_partial_child_signal(context_signal_evidence: &[ContextSignalEvidence]) -> bool {
    context_signal_evidence
        .iter()
        .any(|evidence| evidence.visibility == Some(ChildWorkVisibility::Partial))
}

fn has_visibility_signal(context_signal_evidence: &[ContextSignalEvidence]) -> bool {
    context_signal_evidence
        .iter()
        .any(|evidence| evidence.category == ContextSignalCategory::Visibility)
}

fn separate_child_rollout_session_id<'a>(path: &'a str, session_id: &str) -> Option<&'a str> {
    if !path.contains(".codex/sessions/") {
        return None;
    }

    let file_name = Utf8Path::new(path).file_name()?;
    let stem = file_name.strip_suffix(".jsonl")?;
    let rollout_name = stem.strip_prefix("rollout-")?;
    let child_session_id = rollout_session_id_suffix(rollout_name)?;

    (child_session_id != session_id).then_some(child_session_id)
}

fn rollout_session_id_suffix(rollout_name: &str) -> Option<&str> {
    let hyphen_offsets = rollout_name
        .match_indices('-')
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    if hyphen_offsets.len() < 5 {
        return None;
    }

    let split_index = hyphen_offsets[hyphen_offsets.len() - 5] + 1;
    let candidate = rollout_name.get(split_index..)?;
    let candidate = candidate.split('-').collect::<Vec<_>>();
    if !is_uuid_like_segments(&candidate) {
        return None;
    }

    rollout_name.get(split_index..)
}

fn collect_separate_child_rollout_paths(surface: &str, session_id: &str) -> Vec<(String, String)> {
    extract_path_hints(surface)
        .into_iter()
        .filter_map(|path| {
            let child_session_id =
                separate_child_rollout_session_id(&path, session_id)?.to_string();
            Some((path, child_session_id))
        })
        .collect()
}

fn collect_separate_child_session_ids(surface: &str, session_id: &str) -> Vec<String> {
    if !surface.to_ascii_lowercase().contains("child session id") {
        return Vec::new();
    }

    let mut session_ids = BTreeSet::new();
    for token in surface.split(|ch: char| !ch.is_ascii_hexdigit() && ch != '-') {
        if token == session_id || token.is_empty() {
            continue;
        }
        let segments = token.split('-').collect::<Vec<_>>();
        if is_uuid_like_segments(&segments) {
            session_ids.insert(token.to_string());
        }
    }
    session_ids.into_iter().collect()
}

fn is_uuid_like_segments(segments: &[&str]) -> bool {
    matches!(
        segments,
        [a, b, c, d, e]
            if a.len() == 8
                && b.len() == 4
                && c.len() == 4
                && d.len() == 4
                && e.len() == 12
                && segments.iter().all(|segment| segment.chars().all(is_ascii_hex))
    )
}

fn is_ascii_hex(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

fn infer_delegation_counter_evidence(
    context: &ContextPack,
    marker_evidence: &MarkerEvidence,
    context_signal_evidence: &[ContextSignalEvidence],
) -> Vec<EvidenceRef> {
    if marker_evidence.markers.is_empty() || has_partial_child_signal(context_signal_evidence) {
        return Vec::new();
    }

    let reason = if has_visibility_signal(context_signal_evidence) {
        format!(
            "explicit delegation markers remained child-opaque; no child-visible command evidence appeared across command families: {}",
            summarize_command_families(&context.command_families)
        )
    } else {
        format!(
            "explicit delegation markers lacked child visibility/context evidence across command families: {}",
            summarize_command_families(&context.command_families)
        )
    };

    let candidates = marker_evidence
        .evidence
        .iter()
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: reason.clone(),
        })
        .collect::<Vec<_>>();
    dedupe_evidence(candidates.iter())
}

fn summarize_command_families(command_families: &[String]) -> String {
    if command_families.is_empty() {
        "none observed".to_string()
    } else {
        command_families
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn evidence_key(evidence: &EvidenceRef) -> String {
    format!(
        "{}:{}:{}:{}",
        evidence.row.source_file,
        evidence.row.event_index,
        evidence.row.row_ordinal,
        evidence.reason
    )
}

#[cfg(test)]
mod tests {
    use agent_session_compactor::{CompactionKind, SourceKind};
    use camino::Utf8PathBuf;

    use crate::context::assemble_context;

    use super::*;

    #[test]
    fn delegation_ignores_archival_marker_docs_and_generic_rollout_session_commands() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![row(
                CompactionKind::Unknown,
                "{\"name\":\"multi_agent_v1\"} {\"name\":\"spawn_agent\"}",
            )],
            compact_rows: vec![tool_call(
                "functions.shell_command",
                "{\"command\":\"rg -n 'rollout-.*session' docs/specs\",\"workdir\":\"/repo\"}",
            )],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation.markers.is_empty());
        assert!(delegation.supporting_evidence.is_empty());
    }

    #[test]
    fn delegation_requires_explicit_markers_before_harvesting_context_signals() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![row(
                CompactionKind::AssistantMessage,
                "Child session id 019e-test lives in a separate rollout file after the spawned agent completed work.",
            )],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation.markers.is_empty());
        assert!(delegation.supporting_evidence.is_empty());
    }

    #[test]
    fn delegation_does_not_use_quoted_prose_as_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::AssistantMessage,
                    "Quoted review text: \"Child session id 019e-test lives in a separate rollout file.\"",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(!delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence.reason.contains("context row anchors")));
    }

    #[test]
    fn delegation_harvests_child_rollout_visibility_from_command_surfaces() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert!(delegation.counter_evidence.is_empty());
    }

    #[test]
    fn delegation_caps_opaque_parent_confidence_despite_multi_agent_markers() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("multi_agent_v1", "{\"agent_type\":\"worker\"}"),
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert_eq!(
            delegation.markers,
            vec!["multi_agent_v1".to_string(), "spawn_agent".to_string()]
        );
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_harvests_opaque_child_rollout_from_directive_path_surface() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                row(
                    CompactionKind::UserMessage,
                    "/goal Inspect the spawned agent child rollout at /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl before checkpoint analysis.",
                ),
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("delegation directive surface references separate child rollout")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_combines_archival_markers_with_compact_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![tool_call("spawn_agent", "{\"agent_type\":\"worker\"}")],
            compact_rows: vec![tool_call(
                "functions.shell_command",
                "{\"command\":\"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
            )],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert!(delegation.counter_evidence.is_empty());
    }

    #[test]
    fn delegation_does_not_promote_unrelated_rollout_reads_to_partial_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_harvests_archival_child_visibility_from_raw_tool_rows() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::ToolOutput,
                    "Observed child rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl while inspecting the spawned subagent session.",
                ),
            ],
            compact_rows: Vec::new(),
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert!(delegation.counter_evidence.is_empty());
    }

    #[test]
    fn delegation_does_not_promote_anchored_prompt_bullets_to_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::UserMessage,
                    "- Child session id 019e-test lives in a separate rollout file after the spawned agent completed work.",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("context row anchors")));
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("lacked child visibility/context evidence"));
    }

    #[test]
    fn delegation_does_not_promote_copied_spec_text_to_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::AssistantMessage,
                    "Child rollout files / child session ids only justify partial or opaque visibility in R3.75; they do not authorize parent/child stitching in this packet.",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("context row anchors")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_does_not_promote_copied_spec_grep_tool_output_to_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::ToolOutput,
                    "12:Child rollout files / child session ids only justify partial or opaque visibility in R3.75; they do not authorize parent/child stitching in this packet.",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("anchored child rollout")));
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("separate child session id")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_harvests_directive_path_evidence_independent_of_compaction_placement() {
        let directive = "/goal Inspect the spawned agent child rollout at /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl before checkpoint analysis.";
        let archival_session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![row(CompactionKind::UserMessage, directive)],
            compact_rows: vec![tool_call("spawn_agent", "{\"agent_type\":\"worker\"}")],
        };
        let compact_session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                row(CompactionKind::UserMessage, directive),
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
            ],
        };

        let archival_delegation =
            infer_delegation_context(&archival_session, &assemble_context(&archival_session));
        let compact_delegation =
            infer_delegation_context(&compact_session, &assemble_context(&compact_session));

        let archival_reasons = archival_delegation
            .supporting_evidence
            .iter()
            .map(|evidence| evidence.reason.as_str())
            .filter(|reason| {
                reason.contains("delegation directive surface references separate child rollout")
            })
            .collect::<Vec<_>>();
        let compact_reasons = compact_delegation
            .supporting_evidence
            .iter()
            .map(|evidence| evidence.reason.as_str())
            .filter(|reason| {
                reason.contains("delegation directive surface references separate child rollout")
            })
            .collect::<Vec<_>>();

        assert_eq!(archival_reasons, compact_reasons);
        assert_eq!(
            archival_delegation.counter_evidence.len(),
            compact_delegation.counter_evidence.len()
        );
    }

    #[test]
    fn delegation_keeps_directive_child_visibility_when_generic_read_repeats_path() {
        let directive = "/goal Inspect the spawned agent child rollout at /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl before checkpoint analysis.";
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![row(CompactionKind::UserMessage, directive)],
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };

        let delegation = infer_delegation_context(&session, &assemble_context(&session));

        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence
                .reason
                .contains("delegation directive surface references separate child rollout")));
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("remained child-opaque"));
    }

    #[test]
    fn delegation_harvests_directive_child_session_id_without_rollout_path() {
        let directive = "/goal Inspect the spawned agent handoff before checkpoint analysis; child session id 019ea111-1111-7111-8111-111111111111 remains in a separate rollout file.";
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![row(CompactionKind::UserMessage, directive)],
            compact_rows: vec![tool_call("spawn_agent", "{\"agent_type\":\"worker\"}")],
        };

        let delegation = infer_delegation_context(&session, &assemble_context(&session));

        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence.reason.contains(
                "delegation directive surface references separate child session id: 019ea111-1111-7111-8111-111111111111"
            )));
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("remained child-opaque"));
    }

    #[test]
    fn delegation_does_not_treat_current_parent_rollout_as_child_visible() {
        let session = BundleSession {
            session_id: "019ea000-0000-7000-8000-000000000000".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea000-0000-7000-8000-000000000000.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_does_not_treat_fixture_rollout_as_child_visible() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' crates/agent-drift-analyzer/tests/fixtures/acceptance/sample/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence
                .reason
                .contains("child rollout surface links child/subagent work")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_marks_missing_child_visibility_as_counter_evidence() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![tool_call("spawn_agent", "{\"agent_type\":\"worker\"}")],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert!(delegation.topology.is_none());
        assert!(delegation.child_work_visibility.is_none());
        assert!(delegation.confidence.is_none());
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("lacked child visibility/context evidence"));
    }

    #[test]
    fn delegation_degrades_non_spawn_markers_to_mixed_or_ambiguous() {
        for marker in ["multi_agent_v1", "wait_agent", "close_agent"] {
            let session = BundleSession {
                session_id: format!("session-{marker}"),
                archival_rows: Vec::new(),
                compact_rows: vec![tool_call(marker, "{\"agent_type\":\"worker\"}")],
            };
            let context = assemble_context(&session);

            let delegation = infer_delegation_context(&session, &context);

            assert!(delegation.topology.is_none());
            assert!(delegation.child_work_visibility.is_none());
            assert!(delegation.confidence.is_none());
            assert_eq!(delegation.markers, vec![marker.to_string()]);
            assert_eq!(delegation.counter_evidence.len(), 1);
        }
    }

    fn row(kind: CompactionKind, text: &str) -> CompactionRow {
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 0,
            line_number: 1,
            row_ordinal: 0,
            timestamp: None,
            kind,
            user_message_role: None,
            dedupe_identity: None,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("{kind:?}-{}", text.len()),
        }
    }

    fn tool_call(tool_name: &str, text: &str) -> CompactionRow {
        let mut row = row(CompactionKind::ToolCall, text);
        row.dedupe_identity = Some(format!(
            "{{\"call_id\":\"call-1\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
        ));
        row
    }
}
