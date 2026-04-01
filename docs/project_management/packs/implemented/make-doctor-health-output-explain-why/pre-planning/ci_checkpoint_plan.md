# make-doctor-health-output-explain-why — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/minimal_spec_draft.md`
- Slice specs: see `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative):
  - Behavior smoke platforms: `tasks.json` → `meta.behavior_platforms_required` (`linux`, `macos`, `windows`)
  - CI parity platforms: `tasks.json` → `meta.ci_parity_platforms_required` (`linux`, `macos`, `windows`)

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted).

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
      "slices": ["DHO0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after the first contract-complete seam: doctor text output attribution (DHO0) is stabilized across platforms before the additive JSON + health attribution surfaces (DHO1) are introduced."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["DHO1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Final checkpoint after the additive JSON contract + health attribution surface is complete (DHO1). This validates the end-to-end operator-facing outputs (text + JSON) across required platforms before feature completion."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 — doctor text attribution seam

- Code-grounded boundary:
  - Completes the “why world disabled” attribution seam for doctor text output (copy + precedence + redaction invariants) before introducing the additive JSON surfaces and health parity behavior in DHO1.
- Stabilized surfaces (from `spec_manifest.md`):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` (doctor/health text attribution wording + precedence + redaction invariants)
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/DHO0-spec.md`
- Risk reduced (from `impact_map.md`):
  - Detects platform-specific drift in doctor output and redaction invariants early (Linux/macOS/Windows).
  - Prevents misattribution due to precedence edge cases (workspace patch vs override env gating) before JSON fields and health surfaces reuse the same attribution logic.

### CP2 — JSON + health attribution seam

- Code-grounded boundary:
  - Completes the additive JSON contract and the health attribution surface, which must stay coherent with doctor and with the cross-cutting precedence/redaction invariants.
- Stabilized surfaces (from `spec_manifest.md`):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/DHO1-spec.md`
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
- Risk reduced (from `impact_map.md`):
  - Validates additive JSON fields and health parity across required platforms and reduces collision risk with queued JSON/health packs listed in `pre-planning/impact_map.md`.

## Follow-ups

### Make this mechanically valid (required once slice tasks exist)

This pack does not define slice triad tasks yet (`tasks.json` `tasks[]` is empty), so checkpoint plan linting against `tasks.json` is not runnable yet.

Before running planning lint:
1) Ensure slice ids in `tasks.json` match `DHO0`, `DHO1` (or update this plan to match the accepted ids).
2) Add triad tasks for each slice (`DHO*-code`, `DHO*-test`, `DHO*-integ-core`, `DHO*-integ`) and their kickoff prompts.
3) Add ops checkpoint tasks `CP1-ci-checkpoint` and `CP2-ci-checkpoint`:
   - `CP1-ci-checkpoint` depends on `DHO0-integ-core`
   - `CP2-ci-checkpoint` depends on `DHO1-integ-core`
4) Gate wiring:
   - `DHO1-code` and `DHO1-test` must `depends_on` `CP1-ci-checkpoint`
5) Keep `tasks.json` `meta.checkpoint_boundaries` equal to the checkpoint boundaries in this plan (currently `["DHO0","DHO1"]`).
6) Then run (must pass):
   - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/make-doctor-health-output-explain-why"`

