# world-disabled-diagnostics — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
- Slice specs: see `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/world-disabled-diagnostics/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist).

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
      "slices": ["WDD0", "WDD1", "WDD2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "The feature has 3 slices (WDD0..WDD2), which is below the default min_triads_per_checkpoint=4; a single end-of-feature checkpoint is appropriate. CP1 is placed at the contract + validation completion seam (WDD2) so cross-platform CI validates the final, stabilized operator-facing surfaces (disabled/skipped diagnostics posture, JSON additive status fields, and smoke coverage)."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 — WDD0..WDD2 (end-of-feature)

Why this boundary is code-grounded:
- This feature is intentionally small (3 slices); the CI checkpoint standard allows a single checkpoint when total slices `< min_triads_per_checkpoint`.
- CP1 is the “contract + validation completion seam”: by the end of `WDD2`, the work should include the decision register selections, additive JSON schema spec, and runnable validation artifacts (tests + smoke scripts). See:
  - `pre-planning/spec_manifest.md` (“Required spec documents” + slice intent)
  - `pre-planning/minimal_spec_draft.md` (“Draft slice skeleton”)

What surfaces are stabilized by CP1 (from `pre-planning/spec_manifest.md`):
- Effective `world.enabled` resolution inside diagnostics (WDD0)
- Disabled-aware world/world-deps diagnostics posture (skip probes when disabled; fail-closed when enabled) (WDD1)
- Operator contract + deterministic copy constraints + additive JSON schema spec + validation coverage (WDD2)

What risk CP1 reduces (from `pre-planning/impact_map.md`):
- Prevents cross-platform drift for `substrate health` / `substrate shim doctor` text + `--json` surfaces when `world.enabled=false` (disabled/skipped must be explicit and non-error).
- Catches OS-specific build/test failures from shell builtin changes (`crates/shell/**`) via compile parity.
- Validates behavior + docs coherence against overlapping packs (json-mode / world-deps provisioning) by running the final end-to-end gates at the seam where the contract and schema are locked.

## Follow-ups

Required to make this plan mechanically valid once slice tasks are created:
- In `tasks.json`:
  - Create triad tasks for `WDD0..WDD2` (including the final `*-integ` tasks required by planning validators).
  - Create `CP1-ci-checkpoint` (type `ops`) and wire dependencies:
    - `CP1-ci-checkpoint` MUST depend on `WDD2-integ-core`.
    - The next checkpoint group (if any are added later) MUST depend on `CP1-ci-checkpoint` per the standard.
  - Ensure `meta.checkpoint_boundaries` matches this plan (`["WDD2"]`).
- Create the checkpoint kickoff prompt referenced by `tasks.json`:
  - `docs/project_management/packs/draft/world-disabled-diagnostics/kickoff_prompts/CP1-ci-checkpoint.md`
- Then run (must pass):
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-disabled-diagnostics"`

If slice scope expands:
- If `WDD1` grows platform-specific complexity (platform guards, Windows/WSL-specific paths), consider splitting into:
  - CP1 after `WDD1` (compile parity + `ci_testing: quick`; feature smoke optional if smoke scripts land later), and
  - CP2 after `WDD2` (full gates including feature smoke).

