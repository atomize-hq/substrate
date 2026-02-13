# world-deps-packages-bundles-contract — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-deps-packages-bundles-contract`
- `docs/project_management/next/world-deps-packages-bundles-contract/impact_map.md`
- `docs/project_management/next/world-deps-packages-bundles-contract/spec_manifest.md`
- Slice specs: `docs/project_management/next/world-deps-packages-bundles-contract/*-spec*.md`
- Required platforms (authoritative):
  - Behavior smoke platforms: `tasks.json` → `meta.behavior_platforms_required` (`linux`, `macos`)
  - CI parity platforms: `tasks.json` → `meta.ci_parity_platforms_required` (`linux`, `macos`)
  - WSL coverage: `tasks.json` → `meta.wsl_required=true` (bundled into Linux smoke via `RUN_WSL=1`)

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted).

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
      "slices": ["WDP0", "WDP1", "WDP2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after the inventory+enabled+status seam: merged available inventory, enabled patch editing, and world-backed applied status. This stabilizes the operator read-only surfaces before any install mutations are shipped."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WDP3", "WDP4", "WDP5"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Final checkpoint after install/sync mutation surfaces: dry-run planning, script installs + wrapper generation, apt installs, and replacement completeness. This is a safety-critical surface and requires full CI testing."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 — inventory + enabled + applied status seam
- Code-grounded boundary:
  - Completes the operator read-only workflow: discover inventory → enable via patches → observe applied status.
- Stabilized surfaces:
  - `docs/project_management/next/world_deps_packages_bundles_contract.md` (CLI contract for list/show/add/remove/reset)
  - `docs/project_management/next/world-deps-packages-bundles-contract/platform-parity-spec.md` (backend-unavailable posture)
- Risk reduced:
  - Prevents platform drift in basic UX and exit codes before installs begin mutating the world.

### CP2 — install/sync mutation seam
- Code-grounded boundary:
  - Completes the mutation workflow: plan (`--dry-run`) → apply prefix scripts/wrappers → apply apt installs and sync enabled list.
- Stabilized surfaces:
  - `docs/project_management/next/world_deps_packages_bundles_contract.md` (install/sync ordering and exit codes)
  - `docs/project_management/next/world-deps-packages-bundles-contract/manual_testing_playbook.md` (smoke parity with operator workflow)
- Risk reduced:
  - Ensures the highest-risk surfaces (apt + prefix installers + legacy removal) are validated across platforms at one bounded checkpoint.
