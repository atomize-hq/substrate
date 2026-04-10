# SGBRO4-spec - lock docs-validation, task graph, and checkpoint planning

## Behavior delta (single)
- Existing: the final slice is the intended checkpoint-boundary lane, but its core merge, checkpoint, platform-fix, and final aggregation structure is not yet normalized from one authoritative slice spec.
- New: `SGBRO4` becomes the boundary slice that owns the prompt/spec/task wiring for checkpoint execution and final docs-validation lock-in.
- Why: the planning pack needs one explicit boundary slice to keep `CP1`, platform-fix tasks, and final aggregation deterministic.

## Scope
- Lock the planning-support artifacts that validate the accepted five-slice spine.
- Keep `plan.md`, `tasks.json`, `session_log.md`, `quality_gate_report.md`, and `pre-planning/ci_checkpoint_plan.md` aligned.
- Keep the feature-level checkpoint prompt and per-slice prompt paths coherent.

## Behavior (authoritative)

### Boundary-slice checkpoint model
- `SGBRO4` is the only checkpoint-boundary slice in the planning pack.
- `CP1-ci-checkpoint` is a feature-level ops task that depends on `SGBRO4-integ-core`.
- `SGBRO4-integ` is the final aggregator that waits for `SGBRO4-integ-core` and the three platform-fix tasks.
- All populated `SGBRO4` task entries point at real kickoff prompt paths.

## Acceptance criteria
- AC-SGBRO4-01: every populated task in `tasks.json` has a real kickoff prompt path.
- AC-SGBRO4-02: `CP1-ci-checkpoint` is defined at the feature level and depends on `SGBRO4-integ-core`.
- AC-SGBRO4-03: the checkpoint plan, task graph, and quality gate report all agree that `SGBRO4` is the boundary slice.

## Out of scope
- Contract wording.
- Schema wording.
- Policy wording.
- Runtime-parity wording.
