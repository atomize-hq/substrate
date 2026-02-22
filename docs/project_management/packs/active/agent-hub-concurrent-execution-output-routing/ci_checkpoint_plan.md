# agent-hub-concurrent-execution-output-routing — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:

- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing`
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/impact_map.md`
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/spec_manifest.md`
- Slice specs:
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR0-spec.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-spec.md`

## Operator rules

- This plan is authoritative for cross-platform CI cadence.
- Any change to slices or platform requirements MUST update:
  1. this file,
  2. `tasks.json` `meta.checkpoint_boundaries`, and
  3. checkpoint tasks (`CP*-ci-checkpoint`).

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
      "slices": ["OR0", "OR1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single bounded checkpoint after OR1-integ-core. OR0+OR1 together form one end-to-end contract seam (event envelope + trace persistence + PTY non-injection + buffering + deterministic warning records). Cross-platform validation runs once the entire seam is merged in a single checkpoint to avoid redundant multi-OS dispatch."
    }
  ]
}
```

## Human-readable rationale

CP1:

- Validates that the end-to-end interactive routing seam is correct across platforms after the contract is fully implemented.
- Avoids dispatching cross-platform CI during OR0, because OR0 alone does not exercise PTY passthrough routing and would produce redundant parity runs.
