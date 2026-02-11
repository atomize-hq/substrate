# world-fs-granular-allow-deny — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/_archived/world-fs-granular-allow-deny`
- `docs/project_management/_archived/world-fs-granular-allow-deny/impact_map.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/spec_manifest.md`
- Slice specs: `docs/project_management/_archived/world-fs-granular-allow-deny/*-spec*.md`

## Operator rules
- This plan is authoritative for CI cadence.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4 cross-platform automation packs: `tasks.json` `meta.checkpoint_boundaries` MUST list the last slice in each checkpoint group (this is mechanically validated).

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
      "slices": ["WFGAD0", "WFGAD1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after the initial breaking broker/schema + snapshot/protocol acceptance work is landed. This boundary is a contract completion seam: the system must reject legacy policy shapes and PolicySnapshotV1 deterministically on all supported build platforms."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WFGAD2", "WFGAD3"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after host emission + helper env contract and deny masking semantics are implemented. This boundary is a subsystem seam: it crosses shell↔world-agent↔world integration touchpoints and is expected to surface platform-specific build and lint issues."
    },
    {
      "id": "CP3",
      "task_id": "CP3-ci-checkpoint",
      "slices": ["WFGAD4", "WFGAD5"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after discover/read semantics and strict lockdown are implemented. This boundary is a high-risk security seam: changes in low-level Linux enforcement and request handling often introduce accidental platform build regressions that must be caught before feature closeout."
    }
  ]
}
```

## Human-readable rationale

CP1:
- Stabilizes the breaking policy schema and request/response snapshot model so all downstream slices can assume V2 inputs and hard-error behavior.

CP2:
- Stabilizes helper env plumbing and deny masking semantics so later security/visibility work has a consistent execution path.

CP3:
- Stabilizes the remaining high-risk enforcement semantics (discover/read and strict lockdown) before feature completion.
