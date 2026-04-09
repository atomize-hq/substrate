# substrate-gateway-boundary-and-runtime-ownership — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- Orchestration branch: `feat/substrate-gateway-boundary-and-runtime-ownership`
- Planning surfaces in scope:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `slices/SGBRO0/SGBRO0-spec.md` through `slices/SGBRO4/SGBRO4-spec.md`
  - `kickoff_prompts/CP1-ci-checkpoint.md`
  - `slices/SGBRO*/kickoff_prompts/`

## Goal
- Reconcile the planning pack to the accepted five-slice spine `SGBRO0` through `SGBRO4`.
- Lock the single checkpoint boundary at `CP1` after `SGBRO4`.
- Keep the task graph validator-backed by real kickoff prompt paths and slice-spec files.

## Guardrails
- Specs remain the source of truth for slice intent and acceptance criteria.
- Planning docs are edited only on the orchestration branch.
- Do not widen this slice into S1/S2 docs alignment or S99 closeout work.
- Do not invent new runtime, schema, policy, or operator semantics here.

## Accepted slice ordering
- `SGBRO0`: boundary ownership and contract authority lock.
- `SGBRO1`: status-schema and `client_wiring.*` lock.
- `SGBRO2`: policy-evaluation and trust-boundary lock.
- `SGBRO3`: typed runtime and parity lock.
- `SGBRO4`: docs-validation, task-graph, and checkpoint lock-in.

## Checkpoint rule
- `CP1-ci-checkpoint` runs once, after `SGBRO4` is complete.
- The checkpoint plan, task graph, and feature-level checkpoint prompt must all name `SGBRO4` as the boundary slice.

## Validation discipline
- First check: `validate_tasks_json.py`
- Second check: `validate_slice_specs.py`
- Third check: `validate_ci_checkpoint_plan.py`
- Record the validator results in `session_log.md` and summarize them in `quality_gate_report.md`.

## Change boundary
- This slice is limited to planning/task/checkpoint lock-in.
- Support artifacts required by the populated task graph are in scope.
- No repository docs from earlier slices are changed here.
