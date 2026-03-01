# world-disabled-diagnostics — workstream triage (pre-planning)

This artifact proposes **pack-internal planning workstreams (PWS)** to parallelize full planning for:
- Feature dir: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`

## Lift + sizing signal (advisory)

Pack strictness:
- `tasks.json meta.slice_spec_version = 2` (strict pack; pack-derived lift is meaningful)

Discovery-time lift (ADR; low confidence):
- `lift_score=10`, `estimated_slices=1`, `confidence=low`

Pack-derived lift (from `pre-planning/impact_map.md` Touch Set):
- `lift_score=105`, `estimated_slices=9`, `confidence=low`
- Derived touch counts (explicit files): `create=23`, `edit=16`
- Triggers include: `likely_split:lift_score>24`, `likely_split:touch_files_sum>12`, `split_required:estimated_slices>3`

Interpretation for planning:
- Treat this as a **parallelization + “split/justify” gate** signal (per D9 in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`), not an instruction to rewrite slice IDs during pre-planning.

## Proposed planning workstreams (PWS)

### WDD-PWS-schema_inventory

- Goal: inventory current JSON/text surfaces so additive-field decisions don’t accidentally break compatibility.
- Owns surfaces:
  - Existing JSON shapes + legacy error fields for `substrate shim doctor --json` and `substrate health --json`
  - Collision scan against `world_doctor` status vocabulary and `json-mode` envelope keys
- Dependencies: none.
- Slices/triads enabled:
  - Unblocks DR-0001/2 and final ACs in `WDD1`/`WDD2`

### WDD-PWS-contract

- Goal: lock the operator-facing contract and the three blocking Decision Register entries.
- Owns surfaces:
  - `contract.md` (precedence, disabled/skipped truth table, exit posture)
  - `decision_register.md` (DR-0001/2/3)
- Dependencies:
  - Requires `WDD-PWS-schema_inventory` before finalizing DR-0001/2.
- Slices/triads enabled:
  - Provides authoritative inputs to `WDD0`/`WDD1`/`WDD2` slice specs and to docs updates.

### WDD-PWS-slice_spec_wdd0

- Goal: author `WDD0` slice spec around threading the **effective** `world.enabled` value into diagnostics reporting.
- Owns surfaces:
  - `slices/WDD0/WDD0-spec.md`
- Dependencies:
  - Needs the contract-level precedence statement (from `WDD-PWS-contract`).
- Slices/triads to create in full planning:
  - `WDD0-code`, `WDD0-test`, `WDD0-integ` (+ kickoff prompts)

### WDD-PWS-slice_spec_wdd1

- Goal: author `WDD1` slice spec for disabled/skipped classification + “skip probes when disabled” behavior.
- Owns surfaces:
  - `slices/WDD1/WDD1-spec.md`
- Dependencies:
  - Needs DR-0001/2/3 decisions (from `WDD-PWS-contract`) before final ACs.
  - Needs field placement constraints (from `WDD-PWS-schema_inventory`).
- Slices/triads to create in full planning:
  - `WDD1-code`, `WDD1-test`, `WDD1-integ` (+ kickoff prompts)

### WDD-PWS-slice_spec_wdd2

- Goal: author `WDD2` slice spec to make validation deterministic and runnable (tests + manual + smoke).
- Owns surfaces:
  - `slices/WDD2/WDD2-spec.md`
  - Acceptance criteria tying integration assertions to DR-selected enum spellings and legacy-field behavior
- Dependencies:
  - Needs DR-0001/2/3 decisions (from `WDD-PWS-contract`).
  - Needs baseline `--json` inventory (from `WDD-PWS-schema_inventory`).
- Slices/triads to create in full planning:
  - `WDD2-code`, `WDD2-test`, `WDD2-integ` (+ kickoff prompts)

### WDD-PWS-docs_validation

- Goal: bound and plan doc updates implied by the Touch Set so operator guidance stays coherent across packs.
- Owns surfaces (per `pre-planning/impact_map.md` Touch Set):
  - `docs/USAGE.md`
  - `docs/CONFIGURATION.md`
  - `docs/COMMANDS.md`
  - `docs/INSTALLATION.md`
  - `docs/cross-platform/wsl_world_troubleshooting.md`
- Dependencies:
  - Needs deterministic copy constraints + enum spellings (DR-0001/3).
- Slices/triads enabled:
  - Typically lands inside `WDD2-*` (or is split into a follow-up if lift-driven shaping requires).

### WDD-PWS-tasks_checkpoints

- Goal: wire the planning pack to execution primitives (checkpoints + triads + prompts) without inventing scope.
- Owns surfaces:
  - `plan.md`
  - `tasks.json` (triads, prompt paths, `meta.checkpoint_boundaries`)
  - kickoff prompts under `slices/WDD*/kickoff_prompts/`
  - `kickoff_prompts/CP1-ci-checkpoint.md` (once `CP1` is an ops task)
- Dependencies:
  - Needs the accepted slice skeleton (baseline `WDD0..WDD2`, plus any split/deferral decisions).
  - Needs slice specs so tasks can reference AC IDs (no freeform acceptance).
- Slices/triads to create in full planning:
  - `WDD0-*`, `WDD1-*`, `WDD2-*`, plus `CP1-ci-checkpoint` (ops task) per `pre-planning/ci_checkpoint_plan.md`.

## Sequencing + gates (full planning)

Hard ordering constraints:
1) **Lift-driven shaping gate (early)**: decide how to respond to pack-derived lift (`estimated_slices=9`):
   - Keep baseline `WDD0..WDD2` and record explicit justification (D9), or
   - Split/deferral: move doc-heavy Touch Set items into a follow-up (or split ADR) to reduce per-item lift.
2) **DR gate before final slice ACs**: DR-0001/2/3 must land before `WDD1`/`WDD2` specs are considered “final”.
3) **Schema inventory before schema spec**: inventory current `--json` shapes before specifying additive fields.
4) **Checkpoint plan before task boundaries**: `tasks.json meta.checkpoint_boundaries` must match the checkpoint plan (currently `["WDD2"]`).

CI checkpoint implications:
- Current pre-planning plan is a single end-of-feature checkpoint `CP1` over `WDD0..WDD2` with compile parity + feature smoke + full CI testing.
- If slice scope expands (especially platform-specific complexity), revisit checkpoint grouping per `pre-planning/ci_checkpoint_plan.md`.

## Risks + unknowns (planning-time)

- Blocking decisions (must be resolved in full planning): DR-0001/2/3 (field paths/enums, legacy-field disabled/skipped behavior, deterministic copy).
- Cross-queue conflicts to respect (from `pre-planning/impact_map.md`):
  - ADR-0003’s `SUBSTRATE_WORLD` mental model vs current `SUBSTRATE_OVERRIDE_WORLD` contract (WDD must stay on current resolver/contract).
  - `world-deps-*` provisioning packs touching `health` remediation guidance must branch on disabled/skipped status fields and avoid provisioning guidance when disabled.
  - `json-mode` must preserve WDD additive fields; DR-0001 should avoid ambiguous collisions with envelope keys.

## Evidence links

Sentinels:
- `docs/project_management/packs/draft/world-disabled-diagnostics/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/logs/CI-checkpoint/last_message.md`

Canonical pre-planning artifacts relied on:
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`

## Slice skeleton recommendations

Baseline (from `pre-planning/minimal_spec_draft.md`): `WDD0`, `WDD1`, `WDD2`.

Recommendation (pre-planning): **NO CHANGE** to the baseline skeleton yet.
- Full planning MUST record an explicit justification for not splitting despite pack-derived lift triggers (D9), or else shape/split scope (prefer deferring doc-heavy Touch Set items to a follow-up rather than adding >3 slices to a single ADR).
