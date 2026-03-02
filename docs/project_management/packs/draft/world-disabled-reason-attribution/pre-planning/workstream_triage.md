# Workstream triage — world-disabled-reason-attribution

Goal: propose pack-internal **Planning Workstreams (PWS)** and sequencing gates for full planning.

## Inputs used (authoritative)

- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/ci_checkpoint_plan.md`
- `tasks.json`

## Work lift evidence (advisory-only)

- Intake/ADR lift (ADR-0038):
  - `logs/workstream-triage/pm_lift_intake.txt`: `lift_score=14`, `estimated_slices=2`, `confidence=low` (missing QA + touch inputs).
- Pack-derived lift (strict pack; `tasks.json.meta.slice_spec_version=2`):
  - Current status: **blocked**. `make pm-lift-pack ...` fails because `pre-planning/impact_map.md` declares a non-existent path as an **Edit** touch:
    - `logs/workstream-triage/pm_lift_pack.stderr.log`
    - Failure: `crates/shell/tests/world_disable_attribution.rs` (impact_map.md:38)

## Slice skeleton (source of truth)

- Slice prefix (from `pre-planning/minimal_spec_draft.md`): `WDRA`
- Draft slice skeleton baseline: `WDRA0` only

## Planning workstreams (PWS)

### WDRA-PWS-contract — replay operator-facing contract

- Goal: author the feature-local operator contract for replay verbose stderr attribution (strings/templates + redaction + “no behavior change” invariants), **reusing ADR-0037 tokens/precedence verbatim**.
- Depends on: (none) — but must not contradict `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`.
- Owns (planning artifacts):
  - `contract.md`
- Proposed slices/triads to plan:
  - `WDRA0` acceptance criteria inputs for replay stderr surfaces (origin summary + host-mode warning) + attribution boundary rules.

### WDRA-PWS-schema_inventory — telemetry + trace contract shape

- Goal: inventory the replay trace surface(s) and pin the additive `replay_strategy` schema changes and redaction/absence semantics (aligned with ADR-0037 fields).
- Depends on:
  - Assumes `WDRA-PWS-contract` has established the replay stderr wording constraints (so telemetry mirrors, not diverges).
- Owns (planning artifacts):
  - `telemetry-spec.md`
- Proposed slices/triads to plan:
  - `WDRA0` telemetry acceptance criteria inputs (`world_disable_reason` / `world_disable_source` emission conditions + absence semantics).

### WDRA-PWS-implementation_seams — touch set validity + seam boundaries

- Goal: make the Touch Set mechanically valid and stable enough to drive lift-derived split decisions (and to reduce churn in slice/task specs).
- Depends on: (none)
- Owns (planning artifacts):
  - `pre-planning/impact_map.md`
  - `pre-planning/spec_manifest.md` (only if doc location/surface→doc mapping needs correction during full planning)
- Proposed slices/triads to plan:
  - Default: keep `WDRA0` as a single slice unless lift/split triggers clearly indicate otherwise.

### WDRA-PWS-slice_spec_wdra0 — slice spec authoring (`WDRA0`)

- Goal: author the vertical slice spec with a deterministic disable-source winner matrix (CLI/env/workspace/global) + redaction assertions + “no routing/selection semantics changes”.
- Depends on (hard):
  - `WDRA-PWS-contract`
  - `WDRA-PWS-schema_inventory`
- Assumes (soft ordering):
  - `WDRA-PWS-implementation_seams` has stabilized the Touch Set (to avoid churn).
- Owns (planning artifacts):
  - `slices/WDRA0/WDRA0-spec.md`
- Proposed slices/triads to plan:
  - `WDRA0` triad: code (shell + replay), tests (replay integration coverage), docs sync tasks (REPLAY/TRACE) referenced but owned outside the pack.

### WDRA-PWS-tasks_checkpoints — tasks.json + sequencing + checkpoint wiring

- Goal: write the executable task graph and checkpoint wiring aligned to the accepted slice skeleton and CI cadence.
- Depends on (hard):
  - `WDRA-PWS-slice_spec_wdra0`
- Owns (planning artifacts):
  - `tasks.json` (**single writer**)
  - `plan.md`
  - `decision_register.md`
  - `pre-planning/ci_checkpoint_plan.md` (update only if slice skeleton changes or checkpoint wiring needs correction)
- Proposed slices/triads to plan:
  - `WDRA0-*` triad tasks (code/test/integ) + `CP1-ci-checkpoint` ops task + kickoff prompt wiring (per CI checkpoint plan follow-ups).

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 1,
  "slice_prefix": "WDRA",
  "pws": [
    {
      "id": "WDRA-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": ["ADR-0037 contract is authoritative"],
      "owns": ["contract.md"]
    },
    {
      "id": "WDRA-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [],
      "assumes": ["WDRA-PWS-contract drafts replay stderr constraints first", "ADR-0037 fields reused verbatim"],
      "owns": ["telemetry-spec.md"]
    },
    {
      "id": "WDRA-PWS-implementation_seams",
      "role": "implementation_seams",
      "depends_on": [],
      "assumes": ["Fix Touch Set validation so pm-lift-pack can run"],
      "owns": ["pre-planning/impact_map.md", "pre-planning/spec_manifest.md"]
    },
    {
      "id": "WDRA-PWS-slice_spec_wdra0",
      "role": "slice_spec",
      "depends_on": ["WDRA-PWS-contract", "WDRA-PWS-schema_inventory"],
      "assumes": ["WDRA-PWS-implementation_seams reduces churn by stabilizing touch set first"],
      "owns": ["slices/WDRA0/WDRA0-spec.md"]
    },
    {
      "id": "WDRA-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": ["WDRA-PWS-slice_spec_wdra0"],
      "assumes": ["pre-planning/ci_checkpoint_plan.md remains single-checkpoint unless slice skeleton changes"],
      "owns": ["tasks.json", "plan.md", "decision_register.md", "pre-planning/ci_checkpoint_plan.md"]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Sequencing + gates (full planning)

Hard gates:

1) **ADR-0037 is the authority** for disable attribution: precedence + tokens + structured field names MUST be reused verbatim (no replay-local taxonomy).
2) **Unblock pack-derived lift**:
   - Fix `pre-planning/impact_map.md` so `make pm-lift-pack ...` succeeds (currently blocked by an “Edit” touch referencing a non-existent path).
3) **Lock DR-0002 (telemetry strategy)** in `decision_register.md` before finalizing `telemetry-spec.md` and `WDRA0` ACs (avoid schema churn).

Recommended order (low churn):

- `contract.md` + `telemetry-spec.md` draft in parallel → `slices/WDRA0/WDRA0-spec.md` → `tasks.json` + checkpoint wiring.

## Slice skeleton recommendations (required)

- RECOMMENDATION: **NO CHANGE** — keep the current single-slice skeleton (`WDRA0` only).
- Revisit only after pack-derived lift is unblocked; if lift triggers indicate an ADR/slice split, propose it explicitly as `SPLIT WDRA0 -> WDRA0 + WDRA1` (UX vs telemetry/docs seams).

## Risks + unknowns (for full planning)

- **Impact map validity**: `pm-lift-pack` (and downstream lift-based split guidance) is currently blocked by a Touch Set entry that cannot be validated against the repo.
- **Precedence contradiction risk**: ADR-0038 text historically listed env override before workspace patch; ADR-0037 + env contract require “workspace overrides env override”. Full planning must reconcile and update ADR-0038 accordingly.
- **Attribution boundary correctness**: replay host-mode reasons must distinguish “world disabled by effective config” vs “host due to replay opt-out / recorded-origin / platform limitations” to avoid misattribution.

## Evidence links (completion sentinels)

- `logs/spec-manifest/last_message.md`
- `logs/impact-map/last_message.md`
- `logs/min-spec-draft/last_message.md`
- `logs/CI-checkpoint/last_message.md`

