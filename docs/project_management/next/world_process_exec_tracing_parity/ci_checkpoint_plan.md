# world_process_exec_tracing_parity — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world_process_exec_tracing_parity`
- `docs/project_management/next/world_process_exec_tracing_parity/impact_map.md`
- `docs/project_management/next/world_process_exec_tracing_parity/spec_manifest.md`
- Slice specs:
  - `docs/project_management/next/world_process_exec_tracing_parity/WPEP0-spec.md`
  - `docs/project_management/next/world_process_exec_tracing_parity/WPEP1-spec.md`
  - `docs/project_management/next/world_process_exec_tracing_parity/WPEP2-spec.md`
  - `docs/project_management/next/world_process_exec_tracing_parity/WPEP3-spec.md`

## Operator rules
- This plan is authoritative for CI cadence.
- Any change to slices or platform requirements MUST update:
  1) this file,
  2) `tasks.json` meta.checkpoint_boundaries, and
  3) checkpoint tasks (`CP*-ci-checkpoint`).

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 1,
    "max_triads_per_checkpoint": 3
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["WPEP0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after the trace correctness + joinability foundation is merged. This boundary is a contract seam: downstream high-volume process telemetry must not land on top of incorrect span parentage or non-joinable IDs."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WPEP1", "WPEP2", "WPEP3"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after process telemetry protocol+persistence, Linux capture, and argv/env redaction hardening are merged. This boundary is a high-risk platform seam: it touches world-agent transport, trace persistence, Linux ptrace/caps behavior, and redaction correctness."
    }
  ]
}
```

## Human-readable rationale

CP1:
- Prevents shipping additional trace volume on top of span hierarchy corruption and missing join keys.

CP2:
- Ensures API transport, persistence, and Linux capture are validated together with smoke coverage.
