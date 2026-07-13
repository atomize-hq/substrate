# Active Spec: R7 Bounded Delegated-Session Support

Canonical authority:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-spec.md`

Status: PLANNED / NOT IMPLEMENTED

The active objective is to add bounded, reciprocal direct parent/child linkage across the existing
`rollout -> compactor -> analyzer -> sentinel` pipeline while preserving separate parent and child
progress/scoring trajectories.

Hard invariants:

- `R6-1` already completed the `dead_end_thrash` progress-aware cutover; do not recreate it.
- Do not reopen `semantic_goal_drift` or `R6-3.X.3` without new failing evidence.
- Never infer child implementation progress, child drift, or child completion from parent
  orchestration alone.
- Only reciprocal structured parent/child linkage authorizes child semantics.
- The compactor owns raw rollout parsing; analyzer and sentinel consume typed contracts.
- Direct children only in the first supported cut; deeper descendants remain explicit residue.
- R8 sentinel interpretation consolidation stays out of R7.

Required planning artifacts:

- `docs/specs/r7/MAP.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-spec.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-plan.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-tasks.md`

Read the canonical spec for commands, interface contracts, testing strategy, boundaries, and success
criteria. If this root mirror and the canonical document ever diverge, the canonical document wins.
