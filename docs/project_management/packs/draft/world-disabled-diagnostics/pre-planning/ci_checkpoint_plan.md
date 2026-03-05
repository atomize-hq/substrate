# world-disabled-diagnostics — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/world-disabled-diagnostics/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: `tasks.json` `meta.checkpoint_boundaries` MUST list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: this plan is mechanically validated against `tasks.json` once slice tasks exist; keep the JSON `slices` list and the `tasks.json` `meta.checkpoint_boundaries` in sync.

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
      "rationale": "Single end-of-feature checkpoint after WDD2. Total slices (3) is below defaults.min=4, so one checkpoint is acceptable; CP1 validates the end-to-end disabled/skipped diagnostics contract (text + JSON), the \"no probes when disabled\" boundary, docs alignment, and cross-platform parity across Linux/macOS/Windows."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`WDD0..WDD2`) — disabled/skipped diagnostics contract completion seam

Why this boundary is code-grounded:
- Total slices for this pack is 3 (`WDD0..WDD2`); the CI checkpoint standard allows a single checkpoint when total slices `< defaults.min_triads_per_checkpoint`.
- CP1 is the “contract completion seam” for this feature: by the end of `WDD2`, the operator-facing behavior for `substrate health` and `substrate shim doctor` (text + `--json`) should be coherent and testable end-to-end, including the disabled/skipped posture, the enabled-but-broken posture, and docs alignment. See:
  - `pre-planning/minimal_spec_draft.md` (cross-cutting invariants: effective-config precedence, degrade vs fail-visible posture, additive-only JSON)
  - `pre-planning/spec_manifest.md` (owned contract surfaces and required docs)
  - `pre-planning/impact_map.md` (touch set + contradiction risks)

What surfaces are stabilized at CP1 (from `pre-planning/spec_manifest.md` + `pre-planning/impact_map.md`):
- Operator-facing contract + exit-code posture: `contract.md` (disabled vs needs-attention semantics; deterministic copy; exit-code mapping).
- JSON contract (additive-only; machine-detectable disabled/skipped): `world-disabled-diagnostics-json-schema-spec.md` (field paths + enum spellings + emission rules).
- Operator documentation: `docs/USAGE.md` (examples/field explanations aligned to shipped behavior).
- Slice behavior + acceptance criteria:
  - `slices/WDD0/WDD0-spec.md` (effective-config resolution seam + exit-code posture for config errors).
  - `slices/WDD1/WDD1-spec.md` (shim doctor disabled/skipped statuses + omission rules + no-probes boundary).
  - `slices/WDD2/WDD2-spec.md` (health summary behavior + docs alignment).
- Cross-platform behavior validation artifacts: `manual_testing_playbook.md` and `smoke/{linux,macos,windows}-smoke.*`.

What risk CP1 reduces (from `pre-planning/impact_map.md`):
- Prevents “primary dev platform is green but other platforms fail” drift for touched crates/files (notably `crates/shell/src/builtins/health.rs` and `crates/shell/src/builtins/shim_doctor/**`) via compile parity + CI testing.
- Validates the cross-cutting invariants under real platform conditions (Linux/macOS/Windows): disabled mode emits explicit disabled/skipped statuses and does not probe; enabled mode does not mask backend failures.
- Catches JSON contract drift early by combining feature smoke (behavior platforms) with full CI testing at the seam where field placement and enum spellings should be stabilized.

## Follow-ups

Before running:
- `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-disabled-diagnostics"`

If planning later adds additional checkpoints:
- Wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.
