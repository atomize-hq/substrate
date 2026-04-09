# llm_and_agent_config_policy_surface — CI checkpoint plan

This file defines when cross-platform validation gates run for this feature.

Standard:

- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/active/llm_and_agent_config_policy_surface`
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/impact_map.md`
- `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md`
- Slice specs:
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`

## Operator rules

- This plan is authoritative for CI cadence.
- Any change to slices or platform requirements MUST update:
  1. this file,
  2. `tasks.json` meta.checkpoint_boundaries, and
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
      "slices": ["LACP0", "LACP1"],
      "gates": {
        "compile_parity": false,
        "feature_smoke": true,
        "ci_testing": "skip"
      },
      "rationale": "Single checkpoint at Phase 3 completion. This boundary is a contract seam: strict schema, dotted updates, and agent inventory/overlay enforcement must be validated on behavior platforms before downstream LLM/agent features consume the surfaces."
    }
  ]
}
```

## Human-readable rationale

CP1:

- Stabilizes the operator-facing config/policy surface (new key families + strictness + `--explain`) and the agent inventory file format and overlay restriction rules.
- Runs feature smoke on Linux and macOS only (Windows is out of scope for this Planning Pack).
