# world-disabled-diagnostics — workstream triage (pre-planning)

## Evidence (authoritative inputs + sentinels)

- Canonical artifacts:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `tasks.json`
- Lift outputs (captured under logs):
  - Pack-derived: `logs/workstream-triage/pm_lift_pack.txt` and `logs/workstream-triage/pm_lift_pack.json`
  - Intake/ADR-derived: `logs/workstream-triage/pm_lift_intake_ADR-0036.txt` and `logs/workstream-triage/pm_lift_intake_ADR-0036.json`
- Step completion sentinels:
  - `logs/spec-manifest/last_message.md`
  - `logs/impact-map/last_message.md`
  - `logs/min-spec-draft/last_message.md`
  - `logs/CI-checkpoint/last_message.md`

## Work Lift v1 signals (sizing + split heuristics)

- Intake-derived (ADR-0036): `lift_score=10`, `estimated_slices=1`, `confidence=low`.
- Pack-derived (strict pack; `tasks.json.meta.slice_spec_version=2`): `lift_score=56`, `estimated_slices=5`, `confidence=low`.
  - Key triggers include:
    - `likely_split:lift_score>24`
    - `likely_split:touch_files_sum>12`
    - `split_required:estimated_slices>3`
  - Impact-map-derived touch counts (evidence): `create=12`, `edit=8`, `prefix_entries=0` (see `pm_lift_pack.json` → `derived.impact_map_touch_counts`).

Interpretation:
- Treat the pack-derived estimate as a strong “split/parallelize” signal (even if the ADR remains a single behavior delta). Prefer smaller slices over a single large slice that risks exceeding the per-task context budget.

## Proposed Planning Workstreams (PWS)

PWS are **pack-internal planning workstreams** (not umbrella `WS-YYYYMM-*`). During full planning, each PWS is intended to be a single-writer for its owned surfaces.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 1,
  "slice_prefix": "WDD",
  "pws": [
    {
      "id": "WDD-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "External authoritative inputs remain canonical: docs/reference/env/contract.md, docs/CONFIGURATION.md, EXIT_CODE_TAXONOMY"
      ],
      "owns": [
        "contract.md",
        "decision_register.md",
        "world-disabled-diagnostics-json-schema-spec.md"
      ]
    },
    {
      "id": "WDD-PWS-implementation_seams",
      "role": "implementation_seams",
      "depends_on": [],
      "assumes": [
        "Slice specs reference DR-0001/2/3 (avoid duplicating schema tokens and copy constraints across multiple docs)"
      ],
      "owns": [
        "slices/WDD0/WDD0-spec.md",
        "slices/WDD1/WDD1-spec.md",
        "slices/WDD2/WDD2-spec.md"
      ]
    },
    {
      "id": "WDD-PWS-tests_ci",
      "role": "tests_ci",
      "depends_on": [
        "WDD-PWS-contract",
        "WDD-PWS-implementation_seams"
      ],
      "assumes": [
        "CI checkpoint cadence follows pre-planning/ci_checkpoint_plan.md (CP1 compile_parity + feature_smoke + ci_testing=full)"
      ],
      "owns": [
        "manual_testing_playbook.md",
        "smoke/"
      ]
    },
    {
      "id": "WDD-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "WDD-PWS-contract",
        "WDD-PWS-implementation_seams",
        "WDD-PWS-tests_ci"
      ],
      "assumes": [
        "tasks.json meta.execution_gates remains unset/false unless explicitly enabled during planning",
        "If the slice skeleton remains single-slice, unused WDD1/WDD2 kickoff prompt directories remain empty"
      ],
      "owns": [
        "tasks.json",
        "session_log.md",
        "plan.md",
        "quality_gate_report.md",
        "pre-planning/ci_checkpoint_plan.md",
        "kickoff_prompts/",
        "slices/WDD0/kickoff_prompts/",
        "slices/WDD1/kickoff_prompts/",
        "slices/WDD2/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

### WDD-PWS-contract — Contract + decision surfaces

- Goal: lock deterministic, testable operator + JSON contracts for disabled vs enabled-but-broken behavior, without redefining external authoritative inputs.
- Owned surfaces (tracked, pack-local):
  - `contract.md`, `decision_register.md`, `world-disabled-diagnostics-json-schema-spec.md`
- Dependencies: none.
- Produces for downstream slices/triads:
  - DR-0001/2/3 selections (JSON field paths + enum spellings; legacy error-field behavior; deterministic copy contract)
  - Exit-code posture for `health` / `shim doctor` (taxonomy-referenced)

### WDD-PWS-implementation_seams — Slice specs + seam boundaries

- Goal: define slice boundaries + acceptance criteria seams so execution triads are small, deterministic, and do not require reading the entire pack.
- Owned surfaces:
  - `slices/…/*-spec.md` (see index)
- Dependencies: none (author specs to reference DR entries; avoid hardcoding final schema/copy tokens).
- Produces for downstream slices/triads:
  - singular, testable “skip probes when disabled” operational boundary
  - deterministic behavior for effective-config resolution errors (and required exit-code mapping)

### WDD-PWS-tests_ci — Manual + smoke validation artifacts

- Goal: create deterministic cross-platform validation evidence aligned to the contract + slice AC.
- Owned surfaces:
  - `manual_testing_playbook.md`, `smoke/`
- Dependencies: `WDD-PWS-contract`, `WDD-PWS-implementation_seams`.
- Produces for downstream slices/triads:
  - disabled/enabled-but-broken manual cases with explicit expected assertions (text + JSON + exit-code)
  - smoke scripts aligned to `pre-planning/ci_checkpoint_plan.md` gates

### WDD-PWS-tasks_checkpoints — tasks.json + checkpoint wiring (single writer)

- Goal: translate the plan into execution-ready tasks/checkpoints with safe ownership boundaries.
- Owned surfaces (constraints):
  - `tasks.json` (single writer; must incorporate contract + slice spec + validation artifacts)
  - `session_log.md`
  - kickoff prompt directories: `kickoff_prompts/`, `slices/*/kickoff_prompts/`
  - orchestration runbook + QA gates: `plan.md`, `quality_gate_report.md`, `pre-planning/ci_checkpoint_plan.md`
- Dependencies: `WDD-PWS-contract`, `WDD-PWS-implementation_seams`, `WDD-PWS-tests_ci`.
- Produces:
  - per-slice triads (`WDD*-code`, `WDD*-test`, `WDD*-integ-*`) + checkpoint task (`CP1-ci-checkpoint`)
  - `tasks.json` metadata alignment: `meta.checkpoint_boundaries` must match `pre-planning/ci_checkpoint_plan.md`

## Sequencing + gates

- Hard ordering constraints:
  - DR selections (DR-0001/2/3) must land before tests/playbook assertions are made deterministic.
  - Slice specs must define the “skip probes” boundary before implementation task decomposition (prevents masked probes when disabled).
  - `tasks.json` wiring is the endcap: it must incorporate the finalized contract + slice AC + validation artifacts.
- CI checkpoint implications:
  - `pre-planning/ci_checkpoint_plan.md` currently defines a single end-of-feature checkpoint (CP1) with gates:
    - compile parity (linux/macos/windows)
    - feature smoke (linux/macos/windows)
    - CI testing: full
- Cross-queue sequencing constraints (from `pre-planning/impact_map.md`):
  - WDD must land before attribution work that touches the same surfaces (e.g., `make-doctor-health-output-explain-why`) and must be preserved by any JSON reshaping work (`json-mode`).

## Risks + unknowns (to resolve during full planning)

- Effective-config resolution failures: must define deterministic behavior and exit codes (no silent misclassification).
- DR-0001/2/3 are required for determinism:
  - JSON field paths + enum spellings
  - legacy error-field behavior under disabled/skipped
  - deterministic operator copy requirements
- “Skip probes” boundary must be operationally defined (forbidden calls/paths) and proven by tests.
- Shared-file churn risk: `crates/shell/src/builtins/health.rs` and `crates/shell/src/builtins/shim_doctor/report.rs` are explicit cross-queue overlap surfaces.

## Slice skeleton recommendations (required)

- Baseline draft slice skeleton in `pre-planning/minimal_spec_draft.md`: `WDD0` only.
- Recommendation (lift/impact-driven): split into 3 smaller slices to reduce per-slice context + de-risk shared-surface churn.
  - SPLIT `WDD0` → `WDD0`, `WDD1`, `WDD2`:
    - `WDD0` — effective-config resolution + shared plumbing for disabled/skipped gating
    - `WDD1` — `substrate shim doctor` disabled/skipped UX + JSON + tests
    - `WDD2` — `substrate health` disabled/skipped UX + JSON + `docs/USAGE.md` update + tests
  - Follow-up if adopted:
    - Update `pre-planning/impact_map.md` Touch Set (add `slices/WDD1/WDD1-spec.md`, `slices/WDD2/WDD2-spec.md` and any additional touched files).
    - Update `pre-planning/ci_checkpoint_plan.md` machine plan `slices` list and set the checkpoint boundary to the last slice (likely `WDD2`).
    - Update `pre-planning/spec_manifest.md` “Slice IDs (canonical)” list to include `WDD1`/`WDD2`.

## Follow-ups

- Repo hygiene: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` is referenced as canonical by multiple standards, but does not exist at repo root in this checkout (only found at `docs/project_management/_archived/misc/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`).
