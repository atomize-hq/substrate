# world-disabled-reason-attribution — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: `tasks.json` `meta.checkpoint_boundaries` MUST list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice triads (`*-integ`) yet, so this plan is not mechanically validated yet; the slice list below is derived from the draft slice skeleton in `pre-planning/minimal_spec_draft.md`.

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["WDRA0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single end-of-feature checkpoint after WDRA0. Total slices (1) is below defaults.min=4, so one checkpoint is acceptable; CP1 validates the final, stabilized replay UX attribution + additive replay strategy telemetry + tests/docs, and catches cross-platform drift/redaction issues early."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`WDRA0..WDRA0`) — end-to-end replay attribution UX + additive telemetry seam

Why this boundary is code-grounded:
- This pre-planning slice skeleton has a single slice (`WDRA0`); the CI checkpoint standard allows a single checkpoint when total slices `< defaults.min_triads_per_checkpoint`.
- CP1 is the “contract completion seam” for this feature: replay stderr attribution + telemetry + validation coverage + doc updates compose a coherent operator-facing change set. See:
  - `pre-planning/spec_manifest.md` (owned contract surfaces and required docs)
  - `pre-planning/impact_map.md` (touch set and contradiction risks)

What surfaces are stabilized at CP1 (from `pre-planning/spec_manifest.md` + `pre-planning/minimal_spec_draft.md`):
- Replay operator-facing contract: `contract.md` (exact stderr templates, redaction rules, attribution selection and fallback)
- Replay trace/telemetry contract: `telemetry-spec.md` (additive replay_strategy fields + absence semantics + redaction)
- Slice behavior + acceptance criteria: `slices/WDRA0/WDRA0-spec.md` (disable-source winner matrix + redaction assertions + “no routing/selection semantics changes”)
- Docs that must be kept coherent with the above authoritative contracts:
  - `docs/REPLAY.md`
  - `docs/TRACE.md`

What risk CP1 reduces (from `pre-planning/impact_map.md`):
- Prevents cross-platform drift in build/tests from touching `crates/shell/**` and `crates/replay/**` via compile parity + CI testing.
- Validates the misattribution boundary and redaction constraints (no absolute host paths; no env value leaks beyond fixed tokens) across Linux/macOS/Windows.
- Catches trace schema/doc churn regressions by running full CI testing at the seam where telemetry field placement and doc examples should be finalized.

## Follow-ups

This plan is not mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint ops tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-disabled-reason-attribution"`

…complete these wiring steps:

1) Confirm slice ids and ordering
   - Ensure the final slice ids in `tasks.json` match the accepted slice ids (expect `WDRA0` unless planning splits/merges).
   - Ensure this plan’s JSON `slices` list matches the deterministic slice order from `tasks.json`.

2) Confirm `tasks.json` checkpoint boundary metadata (schema v4 cross-platform)
   - Ensure `meta.checkpoint_boundaries = ["WDRA0"]` matches the checkpoint group boundary (the last slice in each checkpoint group).
   - If the slice skeleton changes, update `pre-planning/ci_checkpoint_plan.md` first, then update `meta.checkpoint_boundaries`.

3) Add checkpoint task + kickoff prompt + deps
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["WDRA0-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/world-disabled-reason-attribution/kickoff_prompts/CP1-ci-checkpoint.md`

4) If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.

