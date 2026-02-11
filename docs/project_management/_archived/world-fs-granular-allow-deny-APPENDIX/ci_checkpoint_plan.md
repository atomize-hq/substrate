# CI Checkpoint Plan — world-fs-granular-allow-deny-appendix

This plan bounds cross-platform CI dispatch checkpoints for this feature.

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
      "slices": ["WFGADAX0", "WFGADAX1"],
      "gates": {
        "compile_parity": true,
        "ci_testing": "skip",
        "feature_smoke": { "runner": "self-hosted", "platform": "behavior" }
      },
      "rationale": "Checkpoint after schema and routing changes because they affect parsing and routing posture."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WFGADAX2", "WFGADAX3"],
      "gates": {
        "compile_parity": true,
        "ci_testing": "skip",
        "feature_smoke": { "runner": "self-hosted", "platform": "behavior" }
      },
      "rationale": "Checkpoint after REPL behavior changes because they affect interactive workflow safety boundaries."
    }
  ]
}
```
