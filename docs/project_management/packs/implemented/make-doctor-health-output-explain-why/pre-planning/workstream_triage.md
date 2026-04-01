# make-doctor-health-output-explain-why — workstream triage (pre-planning)

This artifact proposes **pack-internal planning workstreams (PWS)** to parallelize full planning for this pack (contracts + slice specs + tasks/checkpoints). These are **not** umbrella initiative workstreams.

## Quick context (inputs that shape triage)

- Slice prefix (source of truth): `DHO` (from `pre-planning/minimal_spec_draft.md`)
- Draft slice skeleton (starting point):
  - `DHO0` — doctor text output: correct disable attribution
  - `DHO1` — doctor/health JSON additions + health attribution surface
- Cross-platform requirement (source of truth): `tasks.json meta.behavior_platforms_required` = `linux`, `macos`, `windows`
- CI checkpoint plan (source of truth): `pre-planning/ci_checkpoint_plan.md`
  - CP1: `["DHO0"]` (`ci_testing=quick`)
  - CP2: `["DHO1"]` (`ci_testing=full`)

## Work lift signals (evidence for parallelism + boundaries)

- Pack-derived lift (impact-map-derived; strict pack `meta.slice_spec_version=2`):
  - `lift_score=84`, `estimated_slices=7`, `confidence=low`
  - Triggers include: `split_required:estimated_slices>3`, `likely_split:touch_files_sum>12`, `likely_split:lift_score>24`
  - Evidence: `logs/workstream-triage/pm_lift_pack.txt` + `logs/workstream-triage/pm_lift_pack.json`
- Discovery-time (ADR/intake-derived) lift:
  - `lift_score=7`, `estimated_slices=1`, `confidence=low` (missing touch/test inputs)
  - Evidence: `logs/workstream-triage/pm_lift_intake.txt` + `logs/workstream-triage/pm_lift_intake.json`

Interpretation:
- The impact-map lift is dominated by the authored Touch Set (docs + prompts + smoke scripts + tests), so it is most useful here as a **signal to parallelize planning** and to keep seams explicit (contracts/schema/tests/tasks), not as a mandate to inflate the behavior slice count immediately.

## Proposed planning workstreams (PWS)

### DHO-PWS-contract

- Goal:
  - Resolve ADR-0037 contradictions into one deterministic contract for “why world is disabled” attribution (strings + precedence + redaction).
  - Produce stable vocabulary reused by text + JSON surfaces.
- Owns surfaces (authoritative docs / outputs):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`
- Owns key seams (by reference; must match existing behavior):
  - Precedence model from `pre-planning/minimal_spec_draft.md` (including workspace-gating of env override).
  - Redaction invariants (tokenized paths; env-value suppression) from `pre-planning/minimal_spec_draft.md` and `pre-planning/spec_manifest.md`.
- Depends on:
  - None (first ordering gate).
- Proposed slices/triads to create during full planning:
  - Define AC IDs and wording constraints consumed by:
    - `DHO0-(code|test|integ)` (doctor text attribution)
    - `DHO1-(code|test|integ)` (JSON + health parity)

### DHO-PWS-schema_inventory

- Goal:
  - Inventory current JSON payload shapes for:
    - `substrate host doctor --json`
    - `substrate world doctor --json`
    - `substrate health --json`
  - Pin exact additive field placement + names + enums + omit/emit rules for:
    - `world_disable_reason`
    - `world_disable_source`
- Owns surfaces (authoritative docs / outputs):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
- Depends on:
  - `DHO-PWS-contract` (enum values + redaction rules + “source unknown” posture).
- Proposed slices/triads to create during full planning:
  - Provide schema anchors + examples referenced by `DHO1-spec.md` and `DHO1-(test|integ)` tasks.

### DHO-PWS-implementation_seams

- Goal:
  - Define a single, reusable attribution seam that can be shared across doctor + health, while matching effective config resolution:
    - implement strategy selection (DR-0001) without duplicating precedence logic,
    - ensure tokenized `path_display` and env redaction invariants,
    - ensure cross-platform parity for text + JSON emission.
- Owns surfaces (authoritative planning outputs):
  - Slice specs:
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/DHO0-spec.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/DHO1-spec.md`
  - Implementation seam description (to be reflected in slice specs and, later, code):
    - new module: `crates/shell/src/execution/world_disable_attribution.rs`
- Depends on:
  - `DHO-PWS-contract`
  - `DHO-PWS-schema_inventory` (for JSON field placement/naming used in DHO1).
- Proposed slices/triads to create during full planning:
  - `DHO0-(code|test|integ)` scoped to doctor **text** attribution only (no JSON additions).
  - `DHO1-(code|test|integ)` scoped to additive doctor/health JSON fields + health parity.

### DHO-PWS-tests_ci

- Goal:
  - Author the validation artifacts needed to make the contract provable across Linux/macOS/Windows:
    - unit tests for attribution classifier,
    - integration tests for doctor + health outputs (text + JSON),
    - manual testing playbook + smoke scripts aligned to CI checkpoint gates.
- Owns surfaces (authoritative docs / outputs):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/manual_testing_playbook.md`
  - Smoke scripts:
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/linux-smoke.sh`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/macos-smoke.sh`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/windows-smoke.ps1`
  - Test touch surfaces to be specified + asserted (by reference):
    - `crates/shell/tests/world_disable_attribution.rs` (new)
    - existing test extensions likely in `crates/shell/tests/doctor_scopes_ds0.rs`, `crates/shell/tests/shim_health.rs`, `crates/shell/tests/shim_doctor.rs`
- Depends on:
  - `DHO-PWS-contract` (exact strings + redaction invariants)
  - `DHO-PWS-schema_inventory` (exact JSON names/placement)
  - `DHO-PWS-implementation_seams` (AC IDs in slice specs; seam boundaries)
- Proposed slices/triads to create during full planning:
  - Bind slice-spec AC IDs to:
    - `DHO0-test` / `DHO0-integ`
    - `DHO1-test` / `DHO1-integ`
  - Ensure smoke steps map to CI gates in `pre-planning/ci_checkpoint_plan.md` (CP1 validates text attribution; CP2 validates JSON + health parity).

### DHO-PWS-tasks_checkpoints

- Goal:
  - Populate the schema v4 task graph + kickoff prompts so automation and checkpointing can run mechanically.
  - Keep task deps aligned with the checkpoint plan (CP1 after DHO0 integ; CP2 after DHO1 integ).
- Owns surfaces (authoritative docs / outputs):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json` (populate `tasks[]`; keep `meta.checkpoint_boundaries` aligned to checkpoint plan)
  - Kickoff prompts (must exist on disk; paths listed in impact map):
    - `slices/DHO0/kickoff_prompts/DHO0-(code|test|integ).md`
    - `slices/DHO1/kickoff_prompts/DHO1-(code|test|integ).md`
  - Runbook + audit:
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/plan.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/session_log.md`
  - Cross-pack sequencing touch:
    - `docs/project_management/packs/sequencing.json` (per `pre-planning/impact_map.md`)
- Depends on:
  - `DHO-PWS-implementation_seams` (slice specs + AC IDs; stable slice boundaries)
  - `DHO-PWS-tests_ci` (so tasks/checkpoints encode required evidence)
- Proposed slices/triads to create during full planning:
  - Minimal triad spine:
    - `DHO0-code` → `DHO0-test` → `DHO0-integ`
    - `DHO1-code` → `DHO1-test` → `DHO1-integ`
  - Checkpoint task wiring (per `pre-planning/ci_checkpoint_plan.md` Follow-ups):
    - add `CP1-ci-checkpoint` depends on `DHO0-integ-core`
    - add `CP2-ci-checkpoint` depends on `DHO1-integ-core`
    - wire `DHO1-(code|test)` to depend on `CP1-ci-checkpoint`

## Sequencing + gates (hard ordering constraints)

- Gate 1 — contract truth must land before slice specs are treated as final:
  - DR-0001 + precedence + redaction invariants must be decided in `decision_register.md` / `contract.md`.
  - Rationale: misattribution is worse than generic messaging; contract must define “source unknown” fail-open posture.
- Gate 2 — JSON contract must land before DHO1 AC IDs and tests:
  - Field names, placement, and enum set must be locked in `doctor-health-output-attribution-schema-spec.md` before writing DHO1 acceptance criteria.
- Gate 3 — task graph must follow checkpoint plan:
  - CP1/CP2 ordering is already fixed by `pre-planning/ci_checkpoint_plan.md` and `tasks.json meta.checkpoint_boundaries=["DHO0","DHO1"]`.
  - If slice skeleton changes (see below), update `pre-planning/ci_checkpoint_plan.md` first, then update `tasks.json meta.checkpoint_boundaries`.

## Risks + unknowns (follow-ups to resolve during full planning)

- ADR-0037 internal contradiction (must be reconciled via DR-0001):
  - Recommendation vs Decision Summary re: provenance-based vs heuristic attribution.
- JSON contract ambiguity (DR-0002 + schema spec):
  - Confirm existing “world enabled” JSON key name(s) per command, and define the emit/omit gate for the new fields.
  - Decide whether `world_disable_reason=default` is actually emittable; if not, remove it from the enum set.
  - Pin where the health JSON fields live (top-level vs nested) without breaking existing consumers.
- High-churn seams (consider explicit boundaries to reduce rework):
  - `crates/shell/src/builtins/health.rs` and `crates/shell/src/builtins/shim_doctor/report.rs` overlap with queued packs noted in `pre-planning/impact_map.md` (world-disabled-diagnostics, json-mode, provisioning packs).
  - `docs/project_management/packs/sequencing.json` is a shared coordination artifact (expect concurrent edits).

## Slice skeleton recommendations (explicit)

- Recommendation: **NO CHANGE** to the current draft slice skeleton (`DHO0`, `DHO1`).
  - Rationale:
    - Matches the behavior seams defined in `pre-planning/minimal_spec_draft.md` (text attribution seam first, then JSON + health seam).
    - Aligns with the already-authored CI checkpoint partition in `pre-planning/ci_checkpoint_plan.md` (CP1 after DHO0, CP2 after DHO1).
    - Pack-lift split triggers are primarily driven by planning scaffolding file creation in the Touch Set; address that by parallel planning workstreams + a richer task graph, not by inflating behavior slices prematurely.
- Contingency (only if DHO1 becomes too broad during slice-spec authoring):
  - `SPLIT DHO1` → `DHO1` (doctor JSON additions only) + `DHO2` (health text/JSON attribution + shim-doctor plumbing).
  - If executed, must also update:
    - `pre-planning/ci_checkpoint_plan.md` (checkpoint slices + rationale)
    - `tasks.json meta.checkpoint_boundaries` (e.g., `["DHO0","DHO2"]`)

## Evidence links (step completion sentinels)

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/logs/CI-checkpoint/last_message.md`

## Canonical artifacts referenced

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json`

