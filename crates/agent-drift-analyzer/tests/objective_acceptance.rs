#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    context::extract_objective, ObjectiveClass, ObjectiveConstraintKind, ObjectiveIntent,
    ObjectiveRole, ObjectiveSectionKind, ObjectiveSourceKind, ObjectiveTargetKind,
    RequestedDeliverableKind,
};
use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use support::{
    sorted_entry_names, ObjectiveAcceptanceCorpus, OBJECTIVE_ACCEPTANCE_FAMILIES,
    OBJECTIVE_ACCEPTANCE_FAMILY_README,
};
use tempfile::TempDir;

#[test]
fn objective_acceptance_fixture_root_contains_only_readme_and_family_dirs() {
    let corpus = ObjectiveAcceptanceCorpus::load();
    let mut expected_root_entries = OBJECTIVE_ACCEPTANCE_FAMILIES
        .iter()
        .map(|family| family.dir_name.to_owned())
        .collect::<Vec<_>>();
    expected_root_entries.push("README.md".to_owned());
    expected_root_entries.sort();

    assert_eq!(
        sorted_entry_names(corpus.root().as_std_path()),
        expected_root_entries,
        "Packet SO-4.1 must keep the objective acceptance root bounded to README.md plus the committed family directories"
    );
}

#[test]
fn objective_acceptance_family_dirs_keep_so_5_1_locked_cases_and_other_families_placeholder_only() {
    let corpus = ObjectiveAcceptanceCorpus::load();
    let placeholder_only_entries = vec![OBJECTIVE_ACCEPTANCE_FAMILY_README.to_owned()];

    for family in OBJECTIVE_ACCEPTANCE_FAMILIES {
        let family_dir = corpus.family_dir(family);
        let actual_entries = sorted_entry_names(family_dir.as_std_path());
        if family.dir_name == "locked-acceptance" {
            assert_eq!(
                actual_entries,
                vec![
                    OBJECTIVE_ACCEPTANCE_FAMILY_README.to_owned(),
                    "instruction-surface-agents-skill-update".to_owned(),
                    "instruction-surface-available-skills-review".to_owned(),
                    "wdap0-integ-linux-kickoff".to_owned(),
                    "wdap0-integ-macos-kickoff".to_owned(),
                ],
                "Packets SO-5.1 and SO-5.2 must seed the locked acceptance family with the WDAP kickoff cases plus preserved instruction-surface controls only"
            );
            assert_eq!(
                corpus
                    .case_paths(family)
                    .into_iter()
                    .map(|case| case.case_id)
                    .collect::<Vec<_>>(),
                vec![
                    "instruction-surface-agents-skill-update".to_owned(),
                    "instruction-surface-available-skills-review".to_owned(),
                    "wdap0-integ-linux-kickoff".to_owned(),
                    "wdap0-integ-macos-kickoff".to_owned(),
                ],
                "Packets SO-5.1 and SO-5.2 must keep the committed locked acceptance case list deterministic"
            );
        } else {
            assert_eq!(
                actual_entries,
                placeholder_only_entries,
                "objective acceptance family {} must stay placeholder-only until its own SO-5 follow-on packet lands",
                family.dir_name
            );
            assert!(
                corpus.case_paths(family).is_empty(),
                "objective acceptance family {} must not report committed case directories before its follow-on SO-5 packet lands",
                family.dir_name
            );
        }
    }
}

#[test]
fn objective_acceptance_readme_documents_the_family_contract() {
    let readme = std::fs::read_to_string(
        ObjectiveAcceptanceCorpus::load()
            .root()
            .join("README.md")
            .as_std_path(),
    )
    .expect("read objective acceptance README");

    for family in OBJECTIVE_ACCEPTANCE_FAMILIES {
        assert!(
            readme.contains(family.dir_name),
            "objective acceptance README must document family {}",
            family.dir_name
        );
    }
    for required_fragment in [
        "instruction-surface-agents-skill-update",
        "instruction-surface-available-skills-review",
        "wdap0-integ-linux-kickoff",
        "wdap0-integ-macos-kickoff",
        "AGENTS.md",
        "<skill>",
        "Available skills",
        "design-set/",
        "stretch-external/",
        "remain placeholder-only",
    ] {
        assert!(
            readme.contains(required_fragment),
            "objective acceptance README must document SO-5.2 locked acceptance seeding detail `{required_fragment}`"
        );
    }
}

#[test]
fn objective_acceptance_readme_documents_expected_shape_contract() {
    let readme = fs::read_to_string(
        ObjectiveAcceptanceCorpus::load()
            .root()
            .join("README.md")
            .as_std_path(),
    )
    .expect("read objective acceptance README");

    for required_fragment in [
        "raw.json",
        "expected.json",
        "role_spans",
        "exact_ref",
        "field_evidence",
        "forbidden_role_promotions",
        "compatibility_rendering",
        "comparison_key",
        "required_unknown_fields",
    ] {
        assert!(
            readme.contains(required_fragment),
            "objective acceptance README must document expected-shape field `{required_fragment}`"
        );
    }
}

#[test]
fn objective_acceptance_expected_shape_contract_validates_structured_correctness() {
    let corpus = SyntheticObjectiveAcceptanceCorpus::new();

    corpus.write_case(
        "design-set",
        "instruction_surface_review_contract",
        serde_json::json!({
            "rows": [
                {
                    "kind": "user_message",
                    "user_message_role": "prompt",
                    "text": "Filesystem sandboxing defines which files can be read or written. Approval policy is currently never.\n\n/goal Determine whether the AGENTS.md instruction block and <skill> section should change."
                }
            ]
        }),
        serde_json::json!({
            "case_id": "instruction_surface_review_contract",
            "objective_class": "task_statement",
            "primary_intent": "review",
            "target": {
                "kind": "skill_or_instruction_surface",
                "display_contains": ["AGENTS.md", "<skill>"]
            },
            "compatibility_rendering": {
                "acceptable_any_of": [
                    "/goal Determine whether the AGENTS.md instruction block and <skill> section should change."
                ],
                "comparison_key": "review|skill_or_instruction_surface|agents_md|instruction_block|skill"
            },
            "forbidden_unknown_fields": ["target"]
        }),
    );

    corpus.write_case(
        "locked-acceptance",
        "scope_over_checklist_contract",
        serde_json::json!({
            "rows": [
                {
                    "kind": "user_message",
                    "user_message_role": "prompt",
                    "text": "Read first:\n- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md\n- crates/agent-drift-analyzer/src/context/objective.rs\n\n## Scope\nTeach objective extraction to distinguish mission/scope, checklist, verification, constraints, deliverables, context, boilerplate, and tooling-instruction sections without widening into full field assembly yet.\n\n## Checklist\n- Run this task on a linux machine.\n- Inspect objective.rs before editing.\n\n## Verification\n- cargo test -p agent-drift-analyzer checkpoints -- --nocapture\n\n## Return with\n- changed files\n- residual risk"
                }
            ]
        }),
        serde_json::json!({
            "case_id": "scope_over_checklist_contract",
            "objective_class": "task_statement",
            "primary_intent": "other_task",
            "verification_commands": [
                "cargo test -p agent-drift-analyzer checkpoints -- --nocapture"
            ],
            "role_spans": [
                {
                    "role": "goal",
                    "source_kind": "user_prompt",
                    "section_kind": "scope",
                    "excerpt_contains": "Teach objective extraction to distinguish mission/scope",
                    "exact_ref": {
                        "source_file_suffix": "scope_over_checklist_contract.jsonl",
                        "event_index": 0,
                        "row_ordinal": 0
                    }
                },
                {
                    "role": "verification",
                    "source_kind": "user_prompt",
                    "section_kind": "verification",
                    "excerpt_contains": "cargo test -p agent-drift-analyzer checkpoints -- --nocapture",
                    "exact_ref": {
                        "source_file_suffix": "scope_over_checklist_contract.jsonl",
                        "event_index": 0,
                        "row_ordinal": 0
                    }
                }
            ],
            "forbidden_role_promotions": [
                {
                    "forbidden_role": "goal",
                    "source_kind": "user_prompt",
                    "section_kind": "checklist",
                    "excerpt_contains": "Run this task on a linux machine.",
                    "exact_ref": {
                        "source_file_suffix": "scope_over_checklist_contract.jsonl",
                        "event_index": 0,
                        "row_ordinal": 0
                    }
                }
            ],
            "compatibility_rendering": {
                "acceptable_any_of": [
                    "Teach objective extraction to distinguish mission/scope, checklist, verification, constraints, deliverables, context, boilerplate, and tooling-instruction sections without widening into full field assembly yet."
                ]
            }
        }),
    );

    corpus.write_case(
        "stretch-external",
        "explicit_file_target_grounding_contract",
        serde_json::json!({
            "rows": [
                {
                    "kind": "user_message",
                    "user_message_role": "prompt",
                    "text": "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
                }
            ]
        }),
        serde_json::json!({
            "case_id": "explicit_file_target_grounding_contract",
            "objective_class": "task_statement",
            "primary_intent": "review",
            "target": {
                "kind": "file_or_directory",
                "display_contains": ["crates/agent-drift-analyzer/src/context/objective.rs"]
            },
            "field_evidence": {
                "target": [
                    {
                        "role": "goal",
                        "excerpt_contains": "crates/agent-drift-analyzer/src/context/objective.rs",
                        "exact_ref": {
                            "source_file_suffix": "explicit_file_target_grounding_contract.jsonl",
                            "event_index": 0,
                            "row_ordinal": 0
                        }
                    }
                ]
            },
            "compatibility_rendering": {
                "acceptable_any_of": [
                    "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
                ]
            },
            "forbidden_unknown_fields": ["target"]
        }),
    );

    corpus.write_case(
        "design-set",
        "unknown_target_contract",
        serde_json::json!({
            "rows": [
                {
                    "kind": "user_message",
                    "user_message_role": "prompt",
                    "text": "Look at the stuff above and make it better."
                }
            ]
        }),
        serde_json::json!({
            "case_id": "unknown_target_contract",
            "objective_class": "not_task_statement",
            "primary_intent": "other_task",
            "required_unknown_fields": ["target"]
        }),
    );

    corpus.write_case(
        "locked-acceptance",
        "constraint_and_deliverable_grounding_contract",
        serde_json::json!({
            "rows": [
                {
                    "kind": "user_message",
                    "user_message_role": "prompt",
                    "text": "Review SO-2.1, do not change code, identify brittle gaps, and return concrete packet fixes."
                }
            ]
        }),
        serde_json::json!({
            "case_id": "constraint_and_deliverable_grounding_contract",
            "objective_class": "task_statement",
            "primary_intent": "review",
            "constraints": [
                {
                    "constraint_kind": "no_code",
                    "display_contains": "Do not change code"
                }
            ],
            "deliverables": [
                {
                    "deliverable_kind": "other_deliverable",
                    "display_contains": "Return concrete packet fixes"
                }
            ],
            "field_evidence": {
                "constraints": [
                    [
                        {
                            "role": "constraint",
                            "source_kind": "user_prompt",
                            "excerpt_contains": "Do not change code",
                            "exact_ref": {
                                "source_file_suffix": "constraint_and_deliverable_grounding_contract.jsonl",
                                "event_index": 0,
                                "row_ordinal": 0
                            }
                        }
                    ]
                ],
                "deliverables": [
                    [
                        {
                            "role": "goal",
                            "source_kind": "user_prompt",
                            "excerpt_contains": "Return concrete packet fixes",
                            "exact_ref": {
                                "source_file_suffix": "constraint_and_deliverable_grounding_contract.jsonl",
                                "event_index": 0,
                                "row_ordinal": 0
                            }
                        }
                    ]
                ]
            }
        }),
    );

    validate_objective_acceptance_corpus(corpus.root());
}

#[test]
fn objective_acceptance_committed_locked_cases_validate_current_corpus() {
    let corpus = ObjectiveAcceptanceCorpus::load();
    validate_objective_acceptance_corpus(corpus.root());
}

#[derive(Debug, Deserialize)]
struct ObjectiveAcceptanceRawFixture {
    rows: Vec<ObjectiveAcceptanceRawRow>,
}

#[derive(Debug, Deserialize)]
struct ObjectiveAcceptanceRawRow {
    kind: CompactionKind,
    #[serde(default)]
    user_message_role: Option<UserMessageRole>,
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedObjectiveAcceptance {
    case_id: String,
    objective_class: ObjectiveClass,
    primary_intent: ObjectiveIntent,
    #[serde(default)]
    target: Option<ExpectedTarget>,
    #[serde(default)]
    constraints: Vec<ExpectedConstraint>,
    #[serde(default)]
    success_conditions: Vec<ExpectedDisplayField>,
    #[serde(default)]
    deliverables: Vec<ExpectedDeliverable>,
    #[serde(default)]
    verification_commands: Vec<String>,
    #[serde(default)]
    role_spans: Vec<ExpectedRoleSpan>,
    #[serde(default)]
    field_evidence: ExpectedFieldEvidence,
    #[serde(default)]
    forbidden_role_promotions: Vec<ExpectedForbiddenRolePromotion>,
    #[serde(default)]
    compatibility_rendering: Option<ExpectedCompatibilityRendering>,
    #[serde(default)]
    required_unknown_fields: Vec<String>,
    #[serde(default)]
    forbidden_unknown_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedTarget {
    kind: ObjectiveTargetKind,
    #[serde(default)]
    display_contains: Vec<String>,
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    symbols: Vec<String>,
    #[serde(default)]
    named_artifacts: Vec<String>,
    #[serde(default)]
    workspace_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedConstraint {
    constraint_kind: ObjectiveConstraintKind,
    display_contains: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDisplayField {
    display_contains: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDeliverable {
    deliverable_kind: RequestedDeliverableKind,
    display_contains: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedRoleSpan {
    role: ObjectiveRole,
    #[serde(default)]
    source_kind: Option<ObjectiveSourceKind>,
    #[serde(default)]
    section_kind: Option<ObjectiveSectionKind>,
    excerpt_contains: String,
    #[serde(default)]
    exact_ref: Option<ExpectedSpanRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedSpanRef {
    #[serde(default)]
    source_file_suffix: Option<String>,
    #[serde(default)]
    event_index: Option<usize>,
    #[serde(default)]
    row_ordinal: Option<usize>,
    #[serde(default)]
    section_index: Option<usize>,
    #[serde(default)]
    clause_index: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedFieldEvidence {
    #[serde(default)]
    target: Vec<ExpectedRoleSpan>,
    #[serde(default)]
    constraints: Vec<Vec<ExpectedRoleSpan>>,
    #[serde(default)]
    success_conditions: Vec<Vec<ExpectedRoleSpan>>,
    #[serde(default)]
    deliverables: Vec<Vec<ExpectedRoleSpan>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedForbiddenRolePromotion {
    forbidden_role: ObjectiveRole,
    #[serde(default)]
    source_kind: Option<ObjectiveSourceKind>,
    #[serde(default)]
    section_kind: Option<ObjectiveSectionKind>,
    excerpt_contains: String,
    #[serde(default)]
    exact_ref: Option<ExpectedSpanRef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedCompatibilityRendering {
    acceptable_any_of: Vec<String>,
    #[serde(default)]
    comparison_key: Option<String>,
}

struct SyntheticObjectiveAcceptanceCorpus {
    _temp_dir: TempDir,
    root: Utf8PathBuf,
}

impl SyntheticObjectiveAcceptanceCorpus {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("temp objective acceptance corpus");
        let root = Utf8PathBuf::from_path_buf(temp_dir.path().join("objective_acceptance"))
            .expect("utf8 objective acceptance root");
        fs::create_dir_all(root.as_std_path()).expect("create objective acceptance root");
        fs::write(
            root.join("README.md").as_std_path(),
            "synthetic SO-4.2 contract corpus\n",
        )
        .expect("write synthetic README");
        for family in OBJECTIVE_ACCEPTANCE_FAMILIES {
            let family_dir = root.join(family.dir_name);
            fs::create_dir_all(family_dir.as_std_path()).expect("create family dir");
            fs::write(
                family_dir
                    .join(OBJECTIVE_ACCEPTANCE_FAMILY_README)
                    .as_std_path(),
                "synthetic family placeholder\n",
            )
            .expect("write family README");
        }
        Self {
            _temp_dir: temp_dir,
            root,
        }
    }

    fn root(&self) -> &Utf8Path {
        &self.root
    }

    fn write_case(
        &self,
        family_dir_name: &str,
        case_id: &str,
        raw_json: serde_json::Value,
        expected_json: serde_json::Value,
    ) {
        let case_dir = self.root.join(family_dir_name).join(case_id);
        fs::create_dir_all(case_dir.as_std_path()).expect("create case dir");
        fs::write(
            case_dir.join("raw.json").as_std_path(),
            serde_json::to_string_pretty(&raw_json).expect("serialize raw fixture"),
        )
        .expect("write raw fixture");
        fs::write(
            case_dir.join("expected.json").as_std_path(),
            serde_json::to_string_pretty(&expected_json).expect("serialize expected fixture"),
        )
        .expect("write expected fixture");
    }
}

#[derive(Debug)]
struct ObjectiveAcceptanceCaseFixture {
    family_dir_name: String,
    case_id: String,
    raw_json: Utf8PathBuf,
    expected_json: Utf8PathBuf,
}

fn validate_objective_acceptance_corpus(root: &Utf8Path) {
    let cases = objective_acceptance_case_fixtures(root);
    assert!(
        !cases.is_empty(),
        "SO-4.2 contract validation requires at least one committed or synthetic case"
    );
    for case in cases {
        validate_objective_acceptance_case(&case);
    }
}

fn objective_acceptance_case_fixtures(root: &Utf8Path) -> Vec<ObjectiveAcceptanceCaseFixture> {
    let mut fixtures = Vec::new();
    for family in OBJECTIVE_ACCEPTANCE_FAMILIES {
        let family_dir = root.join(family.dir_name);
        let mut case_dirs = fs::read_dir(family_dir.as_std_path())
            .unwrap_or_else(|error| {
                panic!("read objective acceptance family {}: {error}", family_dir)
            })
            .filter_map(|entry| {
                let entry = entry.unwrap_or_else(|error| {
                    panic!("read objective acceptance entry in {}: {error}", family_dir)
                });
                let path = entry.path();
                if !path.is_dir() {
                    return None;
                }
                let case_id = entry.file_name().into_string().unwrap_or_else(|name| {
                    panic!(
                        "objective acceptance case name {} must be UTF-8",
                        std::path::Path::new(&name).display()
                    )
                });
                let dir = Utf8PathBuf::from_path_buf(path).unwrap_or_else(|path| {
                    panic!("objective acceptance case path {:?} must be UTF-8", path)
                });
                Some(ObjectiveAcceptanceCaseFixture {
                    family_dir_name: family.dir_name.to_string(),
                    raw_json: dir.join("raw.json"),
                    expected_json: dir.join("expected.json"),
                    case_id,
                })
            })
            .collect::<Vec<_>>();
        case_dirs.sort_by(|left, right| left.case_id.cmp(&right.case_id));
        fixtures.extend(case_dirs);
    }
    fixtures
}

fn validate_objective_acceptance_case(case: &ObjectiveAcceptanceCaseFixture) {
    assert!(
        case.raw_json.is_file(),
        "objective acceptance case {}/{} must include raw.json",
        case.family_dir_name,
        case.case_id
    );
    assert!(
        case.expected_json.is_file(),
        "objective acceptance case {}/{} must include expected.json",
        case.family_dir_name,
        case.case_id
    );

    let raw: ObjectiveAcceptanceRawFixture = read_json(&case.raw_json);
    let expected: ExpectedObjectiveAcceptance = read_json(&case.expected_json);
    assert_eq!(
        expected.case_id, case.case_id,
        "objective acceptance expected.json case_id must match its directory name"
    );

    let summary = extract_objective(&rows_from_raw_fixture(&case.case_id, &raw));
    let structured = summary
        .structured
        .as_ref()
        .expect("objective acceptance contract requires structured output");

    assert_eq!(
        structured.objective_class, expected.objective_class,
        "objective acceptance case {} must keep objective_class honest",
        case.case_id
    );
    assert_eq!(
        structured.primary_intent, expected.primary_intent,
        "objective acceptance case {} must keep primary_intent honest",
        case.case_id
    );
    assert_eq!(
        summary.verification_commands, expected.verification_commands,
        "objective acceptance case {} must keep verification_commands grounded and deterministic",
        case.case_id
    );

    if let Some(target) = &expected.target {
        let actual_target = structured
            .target
            .as_ref()
            .expect("objective acceptance target expectation requires a concrete target");
        assert_eq!(actual_target.kind, target.kind);
        for fragment in &target.display_contains {
            assert!(
                actual_target.display.contains(fragment),
                "objective acceptance case {} target display must contain `{fragment}`, got `{}`",
                case.case_id,
                actual_target.display
            );
        }
        for expected_path in &target.paths {
            assert!(
                actual_target.paths.iter().any(|path| path == expected_path),
                "objective acceptance case {} target paths must contain `{expected_path}`; got {:?}",
                case.case_id,
                actual_target.paths
            );
        }
        for symbol in &target.symbols {
            assert!(
                actual_target.symbols.iter().any(|actual| actual == symbol),
                "objective acceptance case {} target symbols must contain `{symbol}`; got {:?}",
                case.case_id,
                actual_target.symbols
            );
        }
        for artifact in &target.named_artifacts {
            assert!(
                actual_target
                    .named_artifacts
                    .iter()
                    .any(|actual| actual == artifact),
                "objective acceptance case {} target named_artifacts must contain `{artifact}`; got {:?}",
                case.case_id,
                actual_target.named_artifacts
            );
        }
        for workspace_ref in &target.workspace_refs {
            assert!(
                actual_target
                    .workspace_refs
                    .iter()
                    .any(|actual| actual == workspace_ref),
                "objective acceptance case {} target workspace_refs must contain `{workspace_ref}`; got {:?}",
                case.case_id,
                actual_target.workspace_refs
            );
        }
        for span in &expected.field_evidence.target {
            assert_span_matches(
                &actual_target.evidence,
                span,
                &format!("objective acceptance case {} target evidence", case.case_id),
            );
        }
    }
    if !expected.field_evidence.constraints.is_empty() {
        assert_eq!(
            expected.field_evidence.constraints.len(),
            expected.constraints.len(),
            "objective acceptance case {} constraint field_evidence must align 1:1 with expected constraints",
            case.case_id
        );
    }
    if !expected.field_evidence.success_conditions.is_empty() {
        assert_eq!(
            expected.field_evidence.success_conditions.len(),
            expected.success_conditions.len(),
            "objective acceptance case {} success-condition field_evidence must align 1:1 with expected success_conditions",
            case.case_id
        );
    }
    if !expected.field_evidence.deliverables.is_empty() {
        assert_eq!(
            expected.field_evidence.deliverables.len(),
            expected.deliverables.len(),
            "objective acceptance case {} deliverable field_evidence must align 1:1 with expected deliverables",
            case.case_id
        );
    }

    for (constraint_index, expected_constraint) in expected.constraints.iter().enumerate() {
        let actual_constraint = structured.constraints.iter().find(|constraint| {
            constraint.constraint_kind == expected_constraint.constraint_kind
                && constraint
                    .display
                    .contains(&expected_constraint.display_contains)
        });
        assert!(
            actual_constraint.is_some(),
            "objective acceptance case {} missing constraint kind {:?} containing `{}`; got {:?}",
            case.case_id,
            expected_constraint.constraint_kind,
            expected_constraint.display_contains,
            structured
                .constraints
                .iter()
                .map(|constraint| (&constraint.constraint_kind, &constraint.display))
                .collect::<Vec<_>>()
        );
        if let Some(field_evidence) = expected.field_evidence.constraints.get(constraint_index) {
            let actual_constraint = actual_constraint.expect("constraint matched above");
            for span in field_evidence {
                assert_span_matches(
                    &actual_constraint.evidence,
                    span,
                    &format!(
                        "objective acceptance case {} constraint {} evidence",
                        case.case_id, constraint_index
                    ),
                );
            }
        }
    }

    for (success_condition_index, expected_success_condition) in
        expected.success_conditions.iter().enumerate()
    {
        let actual_success_condition = structured.success_conditions.iter().find(|condition| {
            condition
                .display
                .contains(&expected_success_condition.display_contains)
        });
        assert!(
            actual_success_condition.is_some(),
            "objective acceptance case {} missing success condition containing `{}`; got {:?}",
            case.case_id,
            expected_success_condition.display_contains,
            structured
                .success_conditions
                .iter()
                .map(|condition| condition.display.as_str())
                .collect::<Vec<_>>()
        );
        if let Some(field_evidence) = expected
            .field_evidence
            .success_conditions
            .get(success_condition_index)
        {
            let actual_success_condition =
                actual_success_condition.expect("success condition matched above");
            for span in field_evidence {
                assert_span_matches(
                    &actual_success_condition.evidence,
                    span,
                    &format!(
                        "objective acceptance case {} success condition {} evidence",
                        case.case_id, success_condition_index
                    ),
                );
            }
        }
    }

    for (deliverable_index, expected_deliverable) in expected.deliverables.iter().enumerate() {
        let actual_deliverable = structured.deliverables.iter().find(|deliverable| {
            deliverable.deliverable_kind == expected_deliverable.deliverable_kind
                && deliverable
                    .display
                    .contains(&expected_deliverable.display_contains)
        });
        assert!(
            actual_deliverable.is_some(),
            "objective acceptance case {} missing deliverable kind {:?} containing `{}`; got {:?}",
            case.case_id,
            expected_deliverable.deliverable_kind,
            expected_deliverable.display_contains,
            structured
                .deliverables
                .iter()
                .map(|deliverable| (&deliverable.deliverable_kind, &deliverable.display))
                .collect::<Vec<_>>()
        );
        if let Some(field_evidence) = expected.field_evidence.deliverables.get(deliverable_index) {
            let actual_deliverable = actual_deliverable.expect("deliverable matched above");
            for span in field_evidence {
                assert_span_matches(
                    &actual_deliverable.evidence,
                    span,
                    &format!(
                        "objective acceptance case {} deliverable {} evidence",
                        case.case_id, deliverable_index
                    ),
                );
            }
        }
    }

    for span in &expected.role_spans {
        assert_span_matches(
            &structured.evidence_spans,
            span,
            &format!("objective acceptance case {} role span", case.case_id),
        );
    }

    for forbidden_promotion in &expected.forbidden_role_promotions {
        assert!(
            !structured.evidence_spans.iter().any(|span| {
                span.role == forbidden_promotion.forbidden_role
                    && forbidden_promotion
                        .source_kind
                        .is_none_or(|source_kind| span.source_kind == source_kind)
                    && forbidden_promotion
                        .section_kind
                        .is_none_or(|section_kind| span.section_kind == section_kind)
                    && span.excerpt.contains(&forbidden_promotion.excerpt_contains)
                    && forbidden_promotion
                        .exact_ref
                        .as_ref()
                        .is_none_or(|exact_ref| exact_ref_matches(span, exact_ref))
            }),
            "objective acceptance case {} must not promote `{}` to {:?}",
            case.case_id,
            forbidden_promotion.excerpt_contains,
            forbidden_promotion.forbidden_role
        );
    }

    if let Some(compatibility) = &expected.compatibility_rendering {
        assert!(
            compatibility
                .acceptable_any_of
                .iter()
                .any(|candidate| candidate == &summary.text),
            "objective acceptance case {} compatibility rendering must be one of {:?}, got `{}`",
            case.case_id,
            compatibility.acceptable_any_of,
            summary.text
        );
        if let Some(comparison_key) = &compatibility.comparison_key {
            assert_eq!(
                summary.comparison_key, *comparison_key,
                "objective acceptance case {} comparison_key must stay semantic",
                case.case_id
            );
        }
    }

    let actual_unknown_fields = structured
        .unknowns
        .iter()
        .map(|unknown| unknown.field_name.as_str())
        .collect::<Vec<_>>();
    for required_unknown in &expected.required_unknown_fields {
        assert!(
            actual_unknown_fields
                .iter()
                .any(|field_name| field_name == required_unknown),
            "objective acceptance case {} must keep unknown field `{required_unknown}`; got {:?}",
            case.case_id,
            actual_unknown_fields
        );
        if required_unknown == "target" {
            assert!(
                structured.target.is_none(),
                "objective acceptance case {} must not promote an unknown target into a concrete target",
                case.case_id
            );
        }
    }
    for forbidden_unknown in &expected.forbidden_unknown_fields {
        assert!(
            actual_unknown_fields
                .iter()
                .all(|field_name| field_name != forbidden_unknown),
            "objective acceptance case {} must not leave `{forbidden_unknown}` unknown; got {:?}",
            case.case_id,
            actual_unknown_fields
        );
    }
}

fn rows_from_raw_fixture(case_id: &str, raw: &ObjectiveAcceptanceRawFixture) -> Vec<CompactionRow> {
    let source_file = Utf8PathBuf::from(format!("/fixtures/objective_acceptance/{case_id}.jsonl"));
    raw.rows
        .iter()
        .enumerate()
        .map(|(index, row)| CompactionRow {
            source_file: source_file.clone(),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some(format!("session-{case_id}")),
            turn_id: Some("turn-001".to_string()),
            event_index: index,
            line_number: index + 1,
            row_ordinal: 0,
            timestamp: None,
            kind: row.kind,
            user_message_role: row.user_message_role,
            dedupe_identity: None,
            text: row.text.clone(),
            canonical_text: row.text.clone(),
            text_hash_hex: format!("hash-{case_id}-{index}"),
        })
        .collect()
}

fn assert_span_matches(
    actual_spans: &[agent_drift_analyzer::ObjectiveEvidenceSpan],
    expected: &ExpectedRoleSpan,
    label: &str,
) {
    assert!(
        actual_spans.iter().any(|span| span_matches(span, expected)),
        "{label} must include role {:?} with excerpt containing `{}`; got {:?}",
        expected.role,
        expected.excerpt_contains,
        actual_spans
            .iter()
            .map(|span| {
                (
                    span.role,
                    span.source_kind,
                    span.section_kind,
                    span.excerpt.as_str(),
                    span.section_index,
                    span.clause_index,
                )
            })
            .collect::<Vec<_>>()
    );
}

fn span_matches(
    actual: &agent_drift_analyzer::ObjectiveEvidenceSpan,
    expected: &ExpectedRoleSpan,
) -> bool {
    actual.role == expected.role
        && expected
            .source_kind
            .is_none_or(|source_kind| actual.source_kind == source_kind)
        && expected
            .section_kind
            .is_none_or(|section_kind| actual.section_kind == section_kind)
        && actual.excerpt.contains(&expected.excerpt_contains)
        && expected
            .exact_ref
            .as_ref()
            .is_none_or(|exact_ref| exact_ref_matches(actual, exact_ref))
}

fn exact_ref_matches(
    actual: &agent_drift_analyzer::ObjectiveEvidenceSpan,
    expected: &ExpectedSpanRef,
) -> bool {
    expected
        .source_file_suffix
        .as_ref()
        .is_none_or(|suffix| actual.row.source_file.as_str().ends_with(suffix))
        && expected
            .event_index
            .is_none_or(|event_index| actual.row.event_index == event_index)
        && expected
            .row_ordinal
            .is_none_or(|row_ordinal| actual.row.row_ordinal == row_ordinal)
        && expected
            .section_index
            .is_none_or(|section_index| actual.section_index == Some(section_index))
        && expected
            .clause_index
            .is_none_or(|clause_index| actual.clause_index == Some(clause_index))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Utf8Path) -> T {
    serde_json::from_str(
        &fs::read_to_string(path.as_std_path())
            .unwrap_or_else(|error| panic!("read json fixture {}: {error}", path)),
    )
    .unwrap_or_else(|error| panic!("parse json fixture {}: {error}", path))
}
