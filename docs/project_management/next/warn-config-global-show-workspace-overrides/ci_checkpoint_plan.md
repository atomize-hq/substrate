# warn-config-global-show-workspace-overrides — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/warn-config-global-show-workspace-overrides/`
- `docs/project_management/next/warn-config-global-show-workspace-overrides/impact_map.md`
- `docs/project_management/next/warn-config-global-show-workspace-overrides/spec_manifest.md`
- Slice specs: `docs/project_management/next/warn-config-global-show-workspace-overrides/*-spec*.md`

## Operator rules
- This plan is authoritative for CI cadence.
- If slice grouping changes, update this file and then update `tasks.json`.
- For schema v4 cross-platform automation packs, `tasks.json` `meta.checkpoint_boundaries` lists the last slice in each checkpoint group.

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 2,
    "max_triads_per_checkpoint": 4
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["C0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single-slice feature: CP1 validates the C0 core integration branch before any platform-fix work. The checkpoint is the seam where the new stderr note contract is validated across Linux/macOS/Windows."
    }
  ]
}
```

## Human-readable rationale (required)

CP1 is the only checkpoint for this feature because the feature has one slice (`C0`).

Boundary rationale:
- The checkpoint validates the end-to-end CLI contract for `substrate config global show` (stderr-only note, stdout invariants, unchanged exit codes) across Linux/macOS/Windows.
- The checkpoint is executed after `C0-integ-core` is green on the primary platform, and before any platform-fix tasks are started.

