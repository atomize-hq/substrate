# Workstream triage

## Triage decision
- Pack mode: strict pack (`tasks.json.meta.slice_spec_version = 2`).
- Pack-derived lift v1: `lift_score = 80`, `estimated_slices = 7`, `confidence = low`.
- Dominant triggers: `likely_split:crates_touched>2`, `likely_split:lift_score>24`, `likely_split:touch_files_sum>12`, `split_required:estimated_slices>3`.
- Decision: expand the draft two-slice skeleton to four slices and run five pack-internal planning workstreams.
- Reason: the impact map spans contract/schema docs, policy evaluation semantics, trace publication, cross-platform validation, and `tasks.json` checkpoint wiring. One writer on `tasks.json` remains mandatory.

## Slice skeleton recommendations
- KEEP `ITPS0` as the contract and schema lock slice.
- SPLIT `ITPS1` into three follow-on slices:
- `ITPS1` — policy evaluation ordering, deny taxonomy, and explain-surface closure.
- `ITPS2` — telemetry publication and compatibility closure.
- `ITPS3` — manual validation, CI checkpoint alignment, and promotion closure.
- ACCEPTED slice order for full planning: `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3`.
- Draft slice order from `minimal_spec_draft.md`: `ITPS0`, `ITPS1`.
- Prefix decision: keep `ITPS` unchanged.
- No merge action is needed.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "ITPS",
  "accepted_slice_order": ["ITPS0", "ITPS1", "ITPS2", "ITPS3"],
  "draft_slice_order": ["ITPS0", "ITPS1"],
  "pws": [
    {
      "id": "ITPS-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0042 remains the semantic owner for tuple vocabulary",
        "The policy inspection surface stays on substrate policy current show --explain"
      ],
      "owns": [
        "contract.md",
        "tuple-policy-schema-spec.md",
        "slices/ITPS0/ITPS0-spec.md"
      ]
    },
    {
      "id": "ITPS-PWS-runtime_fail_early",
      "role": "runtime_fail_early",
      "depends_on": ["ITPS-PWS-contract"],
      "assumes": [
        "backend ids remain adapter gates rather than tuple surrogates",
        "fail_closed routing stays in the ordered evaluation path"
      ],
      "owns": [
        "policy-spec.md",
        "decision_register.md",
        "slices/ITPS1/ITPS1-spec.md"
      ]
    },
    {
      "id": "ITPS-PWS-implementation_seams",
      "role": "implementation",
      "depends_on": ["ITPS-PWS-contract", "ITPS-PWS-runtime_fail_early"],
      "assumes": [
        "broker, shell, and trace seams stay split in the final slice inventory",
        "telemetry publication reuses identity_tuple and placement_posture"
      ],
      "owns": [
        "slices/ITPS2/ITPS2-spec.md",
        "slices/ITPS3/ITPS3-spec.md"
      ]
    },
    {
      "id": "ITPS-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": ["ITPS-PWS-contract", "ITPS-PWS-runtime_fail_early"],
      "assumes": [
        "manual validation keeps Codex auth paths as validation-only examples",
        "cross-platform parity remains Linux plus macOS plus Windows"
      ],
      "owns": [
        "telemetry-spec.md",
        "compatibility-spec.md",
        "manual_testing_playbook.md"
      ]
    },
    {
      "id": "ITPS-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "ITPS-PWS-contract",
        "ITPS-PWS-runtime_fail_early",
        "ITPS-PWS-implementation_seams",
        "ITPS-PWS-docs_validation"
      ],
      "assumes": [
        "CI checkpoint cadence stays as one final cross-platform gate unless full planning records a new boundary",
        "The accepted slice order remains stable once planning starts"
      ],
      "owns": [
        "plan.md",
        "tasks.json",
        "session_log.md",
        "kickoff_prompts/",
        "slices/ITPS0/kickoff_prompts/",
        "slices/ITPS1/kickoff_prompts/",
        "slices/ITPS2/kickoff_prompts/",
        "slices/ITPS3/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

### ITPS-PWS-contract — Contract and schema lock
- Goal: lock the additive `llm.constraints.*` contract, naming grammar, precedence, and explain-surface wording before downstream slice drafting.
- Owned surfaces: `contract.md`, `tuple-policy-schema-spec.md`, `slices/ITPS0/ITPS0-spec.md`.
- Dependencies: none.
- Proposed slices/triads: `ITPS0` triad covering contract text, schema grammar, and operator-facing examples.

### ITPS-PWS-runtime_fail_early — Policy evaluation and deny-order lock
- Goal: lock the ordered evaluation flow, deny taxonomy, fail-closed posture, and decision-register entries that other planning lanes reuse verbatim.
- Owned surfaces: `policy-spec.md`, `decision_register.md`, `slices/ITPS1/ITPS1-spec.md`.
- Dependencies: `ITPS-PWS-contract`.
- Proposed slices/triads: `ITPS1` triad covering backend gate ordering, tuple-axis narrowing, explain payloads, and deny reasons.

### ITPS-PWS-implementation_seams — Execution seam split for telemetry and validation
- Goal: convert the high-churn tail of the current `ITPS1` draft into discrete execution seams so full planning does not pack telemetry and validation into one oversized slice.
- Owned surfaces: `slices/ITPS2/ITPS2-spec.md`, `slices/ITPS3/ITPS3-spec.md`.
- Dependencies: `ITPS-PWS-contract`, `ITPS-PWS-runtime_fail_early`.
- Proposed slices/triads:
- `ITPS2` triad for telemetry publication and compatibility closure.
- `ITPS3` triad for validation closure and promotion packaging.

### ITPS-PWS-docs_validation — Telemetry, compatibility, and manual validation closure
- Goal: lock the reusable doc surfaces that carry telemetry field names, additive rollout rules, and deterministic manual review steps.
- Owned surfaces: `telemetry-spec.md`, `compatibility-spec.md`, `manual_testing_playbook.md`.
- Dependencies: `ITPS-PWS-contract`, `ITPS-PWS-runtime_fail_early`.
- Proposed slices/triads:
- `ITPS2` doc triad content for trace publication and compatibility text.
- `ITPS3` doc triad content for validation matrix and example coverage.

### ITPS-PWS-tasks_checkpoints — Single writer for plan and task wiring
- Goal: convert the accepted slice order and dependency graph into `plan.md`, `tasks.json`, checkpoint wiring, kickoff prompts, and session logging with one writer.
- Owned surfaces: `plan.md`, `tasks.json`, `session_log.md`, `kickoff_prompts/`, `slices/ITPS0/kickoff_prompts/`, `slices/ITPS1/kickoff_prompts/`, `slices/ITPS2/kickoff_prompts/`, `slices/ITPS3/kickoff_prompts/`.
- Dependencies: `ITPS-PWS-contract`, `ITPS-PWS-runtime_fail_early`, `ITPS-PWS-implementation_seams`, `ITPS-PWS-docs_validation`.
- Proposed slices/triads: one planning triad per accepted slice plus one final checkpoint task after `ITPS3`.

## Sequencing and gates
- Gate 1: land `ITPS-PWS-contract` first. Full planning inherits the locked tuple vocabulary, precedence, and explain-surface contract from this lane.
- Gate 2: land `ITPS-PWS-runtime_fail_early` next. Telemetry, compatibility, validation, and slice-spec drafting depend on its deny taxonomy and ordered evaluation flow.
- Gate 3: run `ITPS-PWS-implementation_seams` and `ITPS-PWS-docs_validation` in parallel after Gate 2.
- Gate 4: run `ITPS-PWS-tasks_checkpoints` last as the single writer for `tasks.json`, `plan.md`, session log, and kickoff prompts.
- CI checkpoint implication: replace the draft checkpoint slice list in `pre-planning/ci_checkpoint_plan.md` with `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3` during full planning, then keep one final `CP1-ci-checkpoint` boundary after `ITPS3` unless full planning records a second cross-platform checkpoint explicitly.

## Risks and unknowns
- High-churn seam: the current `ITPS1` draft mixes policy ordering, trace publication, compatibility text, validation wiring, and cross-platform checkpoint logic. The split above removes that seam collision.
- High-churn seam: operator inspection wording is already inconsistent across ADR-0043, the implemented ADR-0027 pack, and the policy CLI surface. `ITPS-PWS-contract` needs a single winner text before `tasks.json` lands.
- Unknown requiring follow-up: `pm-lift-intake` failed on `ADR-0043` because `risk.unknowns_high` is a boolean instead of an integer in the embedded lift vector.
- Unknown requiring follow-up: the pack-derived lift score is high and confidence is low. Full planning needs explicit touch-surface counts inside slice specs so later checkpoint and task wiring uses stable scope instead of narrative estimates.

## Evidence links
- Canonical inputs:
- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/ci_checkpoint_plan.md`
- Stable sentinels:
- `logs/spec-manifest/last_message.md`
- `logs/impact-map/last_message.md`
- `logs/min-spec-draft/last_message.md`
- `logs/CI-checkpoint/last_message.md`
- Lift evidence:
- `logs/workstream-triage/pm_lift_pack.txt`
- `logs/workstream-triage/pm_lift_pack.json`
- `logs/workstream-triage/pm_lift_intake.txt`
- `logs/workstream-triage/pm_lift_intake.json`

## Follow-ups
- Repair the `ADR-0043` lift vector so `pm-lift-intake` emits valid JSON during the next refinement pass.
- Update the checkpoint plan’s machine-readable slice list during full planning to match the accepted slice order.
- Record the accepted slice split in `plan.md` and `tasks.json` exactly as `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3`.
