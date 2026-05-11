# ORCH_PLAN: Execute PLAN.md For Host Bootstrap Readiness And Clean-Detach Parking

Live workspace branch: `feat/host-orchestrator-durable-session`  
Recorded branch in [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md): `feat/host-orchestrator-durable-session`  
Authoritative execution branch for this run: `feat/host-orchestrator-durable-session`  
Plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Source SOW: [llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)  
ADR anchor: [docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Prior orchestration reference: [llm-last-mile/ORCH_PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-22.md)  
Execution type: fresh orchestration controller for the slice that corrects host bootstrap readiness, clean-detach parking, and real bootstrap-path regression proof; parent-only gates, parent-only integration, parent-only final authority  
Live workspace root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

This document is the execution controller for the current [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md). It replaces the stale slice-23 orchestration plan and is authoritative for the host bootstrap readiness and clean-detach parking correction run.

This run is complete only if the same merged tree proves all of the runtime truths frozen by `PLAN.md`:

1. `substrate agent start --backend <backend_id> --prompt ...` succeeds when bootstrap establishes valid resumable host continuity and the bootstrap control stream then ends cleanly.
2. That clean exit normalizes to `parked_resumable` or `awaiting_attention`, never `invalidated`, `failed`, or any other terminal state, when the persisted continuity contract is satisfied.
3. `substrate agent turn --session <id> --backend <backend_id> --prompt ...` succeeds against the exact parked session created by that bootstrap path.
4. `substrate agent reattach --session <id>` succeeds against that same parked session without submitting a prompt.
5. Truly broken bootstrap still fails closed as `runtime_start_failed`.
6. The post-`Accepted` public bridge still guarantees explicit `Completed` or `Failed`.

The honest concurrency cap is exactly `2`, and there is only one real parallel window. A third concurrent worker lane would be dishonest because the decisive hotspots collapse back into the same seams:

- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) and [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) must share one frozen readiness/continuity seam.
- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) must stay coordinated around the clean-detach lifecycle path.
- [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) is the late closeout hotspot and stays parent-owned.

The two-worker window remains honest only because `task/m24-p1-parent-contract-freeze-and-readiness-seam` is not just a recording gate. It must leave behind a minimal compile-stable seam scaffold in the accepted main-tree launch base so Worker B can compile against a frozen interface without editing Worker A owned files.

Authoritative run shape:

1. parent initializes the run and freezes the slice contract
2. parent records the contract freeze and writes a narrow compile-stable readiness seam scaffold into the accepted launch base
3. one parallel window opens with exactly two workers
   - Worker A: shared continuity helper plus readiness classification
   - Worker B: bootstrap teardown plus public command lifecycle consumption
4. parent accepts or quarantines lanes
5. parent integrates accepted work, owns the real bootstrap-path regression suite, and finishes public command closeout
6. parent updates docs only after merged code truth is proven
7. parent runs the validation wall, records scope drift, and closes the run

Judgment call on lane shape:

- The suggested "public command + integration/regression closeout" lane is not launched as a third worker lane.
- Reason: the public command surface is not isolated enough from the lifecycle lane, and the real regression suite plus docs are explicit parent-owned hotspots.
- Result: this run uses two worker lanes and a parent-only late closeout phase. That is the highest honest parallelism for this slice.

## Hard Guards

These are run-stopping invariants.

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/host-orchestrator-durable-session`.
2. The parent is the only integrator, the only approval authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-24-host-bootstrap-readiness/**`.
3. `substrate agent start`, `substrate agent turn`, and `substrate agent reattach` keep their current public grammar. No new verbs, no selector broadening, no fuzzy routing, and no "latest session" fallback are allowed.
4. The durable authority remains the persisted orchestration session plus participant truth. Bootstrap success must stop depending solely on attached-live process liveness.
5. Valid detached host continuity is accepted only when all frozen `PLAN.md` contract points hold:
   - session remains non-terminal
   - `active_session_handle_id` still points at the authoritative host participant
   - `attached_participant_id == null`
   - posture is `parked_resumable` when `pending_inbox_count == 0` or `awaiting_attention` when `pending_inbox_count > 0`
   - authoritative host participant remains `resume_eligible == true`
   - authoritative host participant has `attached_client_present == false`
   - required `uaa_session_id` exists when the backend resume path requires it
   - no session or participant field marks the session invalidated, failed, stopped, or otherwise terminal
6. Clean bootstrap control-stream end after continuity exists must park or attention-normalize; it must not invalidate solely because the attached bootstrap stream ended.
7. Broken startup still fails closed when resumability proof is incomplete or contradictory.
8. The post-`Accepted` public bridge invariant remains frozen: after `Accepted`, the request terminates only with `Completed` or `Failed`.
9. Detached-world follow-up remains fail closed. Parked-host logic must not widen world follow-up semantics.
10. `state` remains the lifecycle machine. `posture` remains the explicit persisted attachability and attention summary. No read path may reconstruct posture from owner-process liveness alone.
11. Docs are late and parent-owned. No worker edits docs, ADR text, README truth, or gap-matrix truth before the integrated tree proves the behavior.
12. No worker edits `.runs/**`.
13. If a worker needs to widen beyond its frozen file surface, the lane stops and returns to the parent gate; it does not self-expand.
14. If any frozen runtime contract point becomes disputed during implementation, the run stops and the parent writes `blocked.json`.

Stop the run immediately and write `blocked.json` if any of these occur:

1. The readiness seam cannot be frozen narrowly enough for Worker B to compile against it without editing Worker A owned files.
2. A lane requires a new public selector or a new public lifecycle verb.
3. A lane requires detached-world follow-up success without `reattach`.
4. A lane requires invalidation to remain the clean-exit behavior for a continuity-valid host bootstrap.
5. A lane requires weakening the `Accepted -> Completed|Failed` contract.
6. A lane touches docs or `.runs/**`.
7. The merged tree cannot prove the exact manual CLI bootstrap path required by `PLAN.md`.

## Fresh Worktrees And Branches

Fresh run worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness/continuity-readiness`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness/bootstrap-teardown-lifecycle`

Worker branches:

- `codex/feat-host-orchestrator-durable-session-m24-continuity-readiness`
- `codex/feat-host-orchestrator-durable-session-m24-bootstrap-teardown-lifecycle`

Exact setup commands, to run only after `task/m24-p1-parent-contract-freeze-and-readiness-seam` is accepted and the main checkout `HEAD` is the frozen launch base:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness/continuity-readiness \
  -b codex/feat-host-orchestrator-durable-session-m24-continuity-readiness \
  HEAD

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-24-host-bootstrap-readiness/bootstrap-teardown-lifecycle \
  -b codex/feat-host-orchestrator-durable-session-m24-bootstrap-teardown-lifecycle \
  HEAD
```

Parent integration surface:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/host-orchestrator-durable-session`
- no separate parent integration worktree
- parent integrates in the main checkout only

## Parent-Owned Run-State Surface

Canonical run path:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-24-host-bootstrap-readiness/`

Required top-level artifacts:

- `run-state.json`
- `task-ledger.json`
- `session-log.md`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `impact/`
- `validation/`
- `quarantine/`
- `gates/`
- `sentinels/`
- `qa/`
- `blocked.json` on blocked termination only
- `closeout.md` on successful completion only

Required per-task roots:

- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-p0-parent-run-init-and-source-lock/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-p1-parent-contract-freeze-and-readiness-seam/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-g1-parent-window-a-launch-gate/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-a1-worker-continuity-readiness-implementation/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-a2-worker-continuity-readiness-validation-and-handoff/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-b1-worker-bootstrap-teardown-lifecycle-implementation/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-b2-worker-bootstrap-teardown-lifecycle-validation-and-handoff/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-g2-parent-integration-gate/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-p2-parent-integration-and-public-command-closeout/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-g3-parent-docs-launch-gate/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-p3-parent-docs-and-validation-wall/`
- `.runs/plan-24-host-bootstrap-readiness/tasks/task-m24-p4-parent-final-closeout/`

Each task directory must contain:

- `task.json`
- `commands.txt`
- `summary.md`

Each gate directory must also contain:

- `gate-checklist.md`
- `gate-result.json`

Each worker task directory must also contain:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Validation artifact roots:

- `.runs/plan-24-host-bootstrap-readiness/validation/lane-a/`
- `.runs/plan-24-host-bootstrap-readiness/validation/lane-b/`
- `.runs/plan-24-host-bootstrap-readiness/validation/integration/`
- `.runs/plan-24-host-bootstrap-readiness/validation/final/`
- `.runs/plan-24-host-bootstrap-readiness/validation/validation-wall.md`
- `.runs/plan-24-host-bootstrap-readiness/validation/final/shim-doctor.json`
- `.runs/plan-24-host-bootstrap-readiness/validation/final/health.json`
- `.runs/plan-24-host-bootstrap-readiness/validation/final/world-doctor.json` or `.runs/plan-24-host-bootstrap-readiness/validation/final/world-doctor-rationale.md`

Required external QA artifact:

- `~/.gstack/projects/<slug>/` test-plan artifact frozen from `PLAN.md`

Required sentinels:

- `01--task-m24-p0-parent-run-init-and-source-lock.ok`
- `02--task-m24-p1-parent-contract-freeze-and-readiness-seam.ok`
- `03--task-m24-g1-parent-window-a-launch-gate.ok`
- `04--task-m24-a2-worker-continuity-readiness-validation-and-handoff.ok`
- `05--task-m24-b2-worker-bootstrap-teardown-lifecycle-validation-and-handoff.ok`
- `06--task-m24-g2-parent-integration-gate.ok`
- `07--task-m24-p2-parent-integration-and-public-command-closeout.ok`
- `08--task-m24-g3-parent-docs-launch-gate.ok`
- `09--task-m24-p3-parent-docs-and-validation-wall.ok`
- `10--task-m24-p4-parent-final-closeout.ok`

`run-state.json` is the authoritative parent ledger and must track:

- `run_id`
- `plan_source`
- `authoritative_branch`
- `live_workspace_root`
- `worktree_root`
- `current_phase`
- `current_task_id`
- `gate_status`
- `task_status`
- `lane_ownership`
- `accepted_sentinels`
- `merge_order`
- `quarantined_outputs`
- `impact_artifact_paths`
- `validation_status`
- `blocked_state`
- `closeout_path`

## Frozen Runtime Contract For This Run

`contract-freeze.json` must freeze these exact truths before workers launch:

1. One shared notion of valid detached host continuity is authoritative for both readiness and clean-exit classification.
2. Readiness may be satisfied by:
   - attached-live ownership, or
   - valid detached host continuity under the frozen persisted contract.
3. Clean bootstrap stream end must route through one shared park-vs-fail decision path, not one decision in `async_repl.rs` and another in completion handling.
4. `parked_resumable` and `awaiting_attention` normalization is driven by persisted session truth and `pending_inbox_count`, not attachment-liveness heuristics.
5. `turn` stays exact `(session, backend)` prompt-taking resume only.
6. `reattach` stays attached-owner recovery only.
7. Detached-world follow-up remains fail closed.
8. Broken bootstrap remains `runtime_start_failed`.

Product runtime-state surfaces under test:

- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/leases/<participant_id>.lease`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/inbox/<item_id>.json`

`contract-freeze.json` must also record:

- the frozen helper/classifier seam that Worker B is allowed to consume
- the exact compile-stable seam scaffold written by `p1` into the accepted launch-base tree
- the exact Worker A and Worker B file boundaries
- the stop conditions that force `blocked.json`
- the required final command wall from `PLAN.md`

The parent must distinguish and record three different outputs from `p1`:

1. contract freeze artifact: the written runtime contract in `contract-freeze.json`
2. compile seam prep artifact: the narrow code scaffold added to the accepted launch base so both workers can compile honestly in parallel
3. worker implementation surface: the post-launch code areas that Worker A owns for actual readiness behavior after the scaffold exists

## Workstream Mapping And Concurrency

This run has one real parallel window, and it opens only after the parent freezes the contract and lands the compile-stable seam scaffold in the accepted launch base.

| PLAN.md workstream | Orchestration tasks | Ownership shape |
| --- | --- | --- |
| Freeze the shared continuity contract and launch seam | `task/m24-p1-parent-contract-freeze-and-readiness-seam`, `task/m24-a1-worker-continuity-readiness-implementation`, `task/m24-a2-worker-continuity-readiness-validation-and-handoff` | Parent records the contract and lands the narrow compile-stable seam scaffold; Worker A then owns readiness implementation inside that scaffold surface. |
| Rewrite bootstrap teardown classification | `task/m24-b1-worker-bootstrap-teardown-lifecycle-implementation`, `task/m24-b2-worker-bootstrap-teardown-lifecycle-validation-and-handoff` | Worker B owns clean-detach parking and lifecycle consumption against the accepted `p1` launch seam, without editing Worker A owned files. |
| Align public command behavior and prove the real bootstrap path | `task/m24-p2-parent-integration-and-public-command-closeout` | Parent-only. This is where the regression suite, late public command closeout, and cross-lane glue belong. |
| Docs closeout and validation wall | `task/m24-g3-parent-docs-launch-gate`, `task/m24-p3-parent-docs-and-validation-wall`, `task/m24-p4-parent-final-closeout` | Parent-only and late by contract. |

Concurrency rules:

1. Worker cap is exactly `2` until `task/m24-g2-parent-integration-gate` completes.
2. `p0` and `p1` must finish before any worker starts.
3. `g1` must be green before any worktree is created.
4. Window A is the only concurrent window:
   - Worker A on continuity/readiness
   - Worker B on bootstrap teardown/lifecycle
5. There is no third worker lane for public command closeout.
6. Docs and the real bootstrap-path public control suite stay parent-owned and late.
7. The validation wall runs once, on the final merged tree only.

Why integration order is frozen as A then B then parent closeout:

1. `p1` creates the compile-stable launch seam that both workers branch from.
2. Worker A owns the shared readiness and continuity implementation behind that seam.
3. Worker B is allowed to consume the frozen seam but not reopen it.
4. Integrating Worker A first minimizes rework if the scaffold needs final parent tightening.
5. Parent closeout depends on the accepted merged behavior, not branch-local assumptions.

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m24-p0-parent-run-init-and-source-lock` | parent | — | main checkout / `feat/host-orchestrator-durable-session` | run scaffold, source lock, initial artifact tree, GitNexus preflight record |
| `task/m24-p1-parent-contract-freeze-and-readiness-seam` | parent | `p0` | main checkout / `feat/host-orchestrator-durable-session` | frozen runtime contract, compile-stable launch seam scaffold, lane boundaries, merge order |
| `task/m24-g1-parent-window-a-launch-gate` | parent | `p1` | main checkout / `feat/host-orchestrator-durable-session` | launch decision and worker worktree creation |
| `task/m24-a1-worker-continuity-readiness-implementation` | Worker A | `g1` | continuity-readiness / `codex/feat-host-orchestrator-durable-session-m24-continuity-readiness` | shared continuity helper, readiness predicate, read-side posture alignment in owned files |
| `task/m24-a2-worker-continuity-readiness-validation-and-handoff` | Worker A | `a1` | same | validated report, patch, impact record, lane evidence |
| `task/m24-b1-worker-bootstrap-teardown-lifecycle-implementation` | Worker B | `g1` | bootstrap-teardown-lifecycle / `codex/feat-host-orchestrator-durable-session-m24-bootstrap-teardown-lifecycle` | clean-detach park-vs-fail lifecycle behavior and caller-side consumption in owned files |
| `task/m24-b2-worker-bootstrap-teardown-lifecycle-validation-and-handoff` | Worker B | `b1` | same | validated report, patch, impact record, lane evidence |
| `task/m24-g2-parent-integration-gate` | parent | `a2`, `b2` | main checkout / `feat/host-orchestrator-durable-session` | accept, reject, or quarantine each lane |
| `task/m24-p2-parent-integration-and-public-command-closeout` | parent | `g2` | main checkout / `feat/host-orchestrator-durable-session` | merged tree, late public command closeout, real bootstrap-path regressions, cross-lane glue |
| `task/m24-g3-parent-docs-launch-gate` | parent | `p2` | main checkout / `feat/host-orchestrator-durable-session` | explicit approval to update docs and repo-truth artifacts |
| `task/m24-p3-parent-docs-and-validation-wall` | parent | `g3` | main checkout / `feat/host-orchestrator-durable-session` | late docs truth plus required command wall and manual CLI evidence |
| `task/m24-p4-parent-final-closeout` | parent | `p3` | main checkout / `feat/host-orchestrator-durable-session` | `gitnexus_detect_changes()`, final scope record, closeout artifact, run completion |

## Frozen Ownership Boundaries

Parent-only in `p0`, `p1`, `g1`, `g2`, `p2`, `g3`, `p3`, `p4`:

- `.runs/plan-24-host-bootstrap-readiness/**`
- [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) as read-only input
- [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md) as the controller reference
- [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- docs touched in late closeout only

Parent-owned contract-freeze and compile-seam-prep surface during `p1`:

- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) for the narrow compile-stable seam scaffold and for recording the frozen helper/classifier touchpoints in `contract-freeze.json`
- [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) for the narrow compile-stable seam scaffold and for recording the frozen readiness touchpoints in `contract-freeze.json`

Meaning of `p1` ownership:

1. parent may add only the minimum code scaffold needed to make the accepted launch base compile-stable for both workers
2. parent does not finish the readiness implementation during `p1`
3. after `g1`, Worker A owns continued implementation inside the scaffold surface
4. Worker B may compile against and call into the scaffold surface but may not edit it

Worker A owned after `g1`:

- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- inline tests local to those files

Worker B owned after `g1`:

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- caller-only consumption of the compile-stable seam scaffold frozen in `p1`

Explicit prohibitions:

1. Worker A may not edit `async_repl.rs`, `agents_cmd.rs`, integration tests, or docs.
2. Worker B may not edit `state_store.rs`, `control.rs`, integration tests, or docs.
3. Neither worker may edit the run-state artifacts.
4. If Worker B discovers that it needs `state_store.rs` or `control.rs` drift beyond the `p1` scaffold seam, it stops and hands the issue back to the parent.

## Task Graph And Control Points

### `task/m24-p0-parent-run-init-and-source-lock`

Parent only.

Required actions:

1. create `.runs/plan-24-host-bootstrap-readiness/` and all task roots
2. write initial `run-state.json`, `task-ledger.json`, `lane-ownership.json`, and `validation/validation-wall.md`
3. lock the required source inputs:
   - [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
   - [AGENTS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENTS.md)
   - current [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)
   - [llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)
   - [docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
   - [llm-last-mile/ORCH_PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-22.md)
4. inventory the owned symbol surfaces for the future code run
5. refresh GitNexus if the index is stale
6. run parent-owned GitNexus impact analysis for the symbols the parent may edit during `p1`
7. write the QA-facing test-plan artifact required by `PLAN.md` under `~/.gstack/projects/<slug>/`

Deliverables:

- `impact/preflight/`
- `.runs/plan-24-host-bootstrap-readiness/qa/test-plan-artifact-path.txt`
- source-lock entry in `session-log.md`
- `01--task-m24-p0-parent-run-init-and-source-lock.ok`

### `task/m24-p1-parent-contract-freeze-and-readiness-seam`

Parent only.

Purpose:

- freeze the runtime contract from `PLAN.md`
- write the minimum compile-stable seam scaffold into the main tree before concurrency starts
- freeze the readiness seam before concurrency starts
- freeze the exact lane boundaries so the two worker window is honest

Required actions:

1. confirm the parent-owned GitNexus impact record exists for every `p1` symbol edit before changing code
2. write `contract-freeze.json` with the detached continuity contract, park-vs-fail contract, exact command wall, stop conditions, and a precise description of the compile-stable seam scaffold
3. record the external QA artifact path from `p0` in the run ledger
4. add the narrow compile-stable seam scaffold to the accepted main-tree launch base in the smallest honest touch surface needed for both workers to compile in parallel
5. write `merge-order.json`
6. record the exact touchpoints Worker A owns in `state_store.rs` and `control.rs` after launch
7. record the exact touchpoints Worker B may consume but not edit
8. record the parent-only late closeout surfaces
9. record the accepted launch-base tree identifier in `run-state.json`

Acceptance:

- the compile-stable seam scaffold exists in code on the accepted launch base, not just in prose
- the readiness seam is narrow enough that Worker B does not need to edit Worker A owned files
- the public command and regression closeout surfaces are explicitly parent-owned
- parent-owned GitNexus impact artifacts exist for the actual `p1` symbol edits
- `02--task-m24-p1-parent-contract-freeze-and-readiness-seam.ok`

### `task/m24-g1-parent-window-a-launch-gate`

Parent only.

Purpose:

- verify the contract freeze and compile-stable seam scaffold are both honest
- create worktrees from the accepted post-`p1` launch-base tree
- launch the only concurrent window

Gate must reject launch if:

1. the accepted `p1` launch base does not actually contain the compile-stable seam scaffold described in `contract-freeze.json`
2. Worker B still depends on non-frozen `state_store.rs` or `control.rs` edits
3. lane boundaries are not narrow enough to prevent merge churn
4. the parent would need to start integration tests before both lanes return

Acceptance:

- worker worktrees created from the accepted post-`p1` launch-base tree
- `03--task-m24-g1-parent-window-a-launch-gate.ok`

### `task/m24-a1-worker-continuity-readiness-implementation`

Worker A only.

Scope:

1. implement the shared detached continuity helper
2. update readiness classification to accept attached-live or valid detached continuity
3. preserve required `uaa_session_id` gating on the detached path too
4. align read-side posture classification in the owned readiness surface
5. keep the bridge invariant explicit inside the owned control surface when needed

Before first edit:

1. run GitNexus impact analysis for owned symbols
2. record the blast radius in `impact-analysis.md`
3. stop and escalate if any required edit is `HIGH` or `CRITICAL` risk

### `task/m24-a2-worker-continuity-readiness-validation-and-handoff`

Worker A only.

Minimum lane-A validation before return:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
```

If the current tree requires narrower or different cargo filters, the worker must run the narrowest equivalent commands that execute all new readiness and continuity tests and record the exact substitutions in `commands.txt`.

Required handoff artifacts:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Acceptance target for parent:

- Worker A stayed inside owned files
- readiness now accepts valid detached continuity and still rejects incomplete resumability
- Worker A did not touch lifecycle files, integration tests, or docs

### `task/m24-b1-worker-bootstrap-teardown-lifecycle-implementation`

Worker B only.

Scope:

1. route clean bootstrap stream end through one shared park-vs-fail decision using the compile-stable seam scaffold frozen in `p1`
2. align event-task and completion-task teardown outcomes
3. keep broken bootstrap as explicit failure
4. align `run_start`, `run_turn`, and `run_reattach` caller behavior only as needed to consume the corrected lifecycle truth
5. preserve detached-world fail-closed behavior

Before first edit:

1. run GitNexus impact analysis for owned symbols
2. record the blast radius in `impact-analysis.md`
3. stop and escalate if any required edit is `HIGH` or `CRITICAL` risk

### `task/m24-b2-worker-bootstrap-teardown-lifecycle-validation-and-handoff`

Worker B only.

Minimum lane-B validation before return:

```bash
cargo test -p shell async_repl -- --nocapture
```

If the lane touches `agents_cmd.rs`, the worker must also run the narrowest equivalent caller-surface command that exercises the changed `run_start(...)`, `run_turn(...)`, or `run_reattach(...)` behavior and record the exact command in `commands.txt`, even if no new `agents_cmd.rs`-local tests were added.

Required handoff artifacts:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Acceptance target for parent:

- Worker B stayed out of Worker A owned files
- clean bootstrap exit parks or attention-normalizes when continuity is valid
- broken startup still fails closed
- detached-world follow-up remains explicit and fail closed

### `task/m24-g2-parent-integration-gate`

Parent only.

Purpose:

- review both worker reports
- accept, reject, or quarantine each lane
- ensure the two-worker window remained honest

Gate rules:

1. both lanes must originate from the accepted post-`p1` launch-base tree
2. Worker B is rejected or quarantined if it widened into Worker A owned files
3. Worker A is rejected or quarantined if it widened into lifecycle files, integration tests, or docs
4. any lane that widened public contract scope is quarantined
5. any lane that assumes detached-world continuity is quarantined

Acceptance:

- accepted lane set recorded in `run-state.json`
- `06--task-m24-g2-parent-integration-gate.ok`

### `task/m24-p2-parent-integration-and-public-command-closeout`

Parent only.

Purpose:

- integrate accepted lanes in frozen order
- own the late public command closeout
- own the real bootstrap-path regression proof

Required parent actions:

1. integrate Worker A
2. integrate Worker B on top of the accepted Worker A tree
3. finish any narrow glue required for the merged tree
4. replace synthetic-only proof with the real bootstrap-path regression suite
5. extend [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) for:
   - `start -> parked_resumable`
   - `start -> awaiting_attention`
   - `start -> parked -> turn`
   - `start -> parked -> reattach`
   - broken bootstrap fail-closed
   - post-`Accepted` explicit failure delivery
6. keep detached-world follow-up regression coverage explicit

Acceptance:

- the integrated tree proves the real bootstrap path, not synthetic-only fixtures
- parent-owned late closeout surfaces remain parent-owned
- `07--task-m24-p2-parent-integration-and-public-command-closeout.ok`

### `task/m24-g3-parent-docs-launch-gate`

Parent only.

Docs stay late.

Gate condition:

1. merged code truth is already proven on the integrated tree
2. no runtime seam remains unsettled
3. docs can now describe shipped behavior instead of intent

Acceptance:

- `08--task-m24-g3-parent-docs-launch-gate.ok`

### `task/m24-p3-parent-docs-and-validation-wall`

Parent only.

Late doc targets:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), if it still reflects the old bootstrap model
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md), if it still implies attached-live bootstrap truth
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), only if runtime ownership wording actually drifted
- the SOW or ADR only if the final implementation materially changes their current wording needs

This task also owns the required validation wall from `PLAN.md`.

Acceptance:

- docs reflect proven behavior and not pre-fix assumptions
- required command wall, manual CLI evidence, and repo-level shell-runtime evidence are recorded under `validation/final/`
- `09--task-m24-p3-parent-docs-and-validation-wall.ok`

### `task/m24-p4-parent-final-closeout`

Parent only.

Required final actions:

1. run `gitnexus_detect_changes()` before any commit or landing prep
2. record final accepted symbol and flow drift
3. verify `validation/final/shim-doctor.json` and `validation/final/health.json` exist, plus either `validation/final/world-doctor.json` or `validation/final/world-doctor-rationale.md`
4. write `closeout.md`
5. mark the run complete in `run-state.json`

Acceptance:

- no unresolved quarantined output remains
- `blocked.json` is absent
- `10--task-m24-p4-parent-final-closeout.ok`

## Merge Order And Quarantine Rules

`merge-order.json` is frozen during `p1` and must record:

- `authoritative_branch: "feat/host-orchestrator-durable-session"`
- `launch_base: "accepted_p1_compile_seam_tree"`
- `integration_order: ["task/m24-a1-worker-continuity-readiness-implementation", "task/m24-b1-worker-bootstrap-teardown-lifecycle-implementation", "task/m24-p2-parent-integration-and-public-command-closeout"]`
- `public_command_closeout_owner: "parent"`
- `docs_owner: "parent"`
- `replay_required_before_acceptance: false`
- `replay_not_required_rationale: "slice remains shell-runtime bootstrap/lifecycle only and does not change replay-owned surfaces"`
- `quarantine_on_scope_drift: true`

Quarantine rules:

1. Quarantined output is copied under `.runs/plan-24-host-bootstrap-readiness/quarantine/<task-id>/`.
2. Quarantined output must include the worker patch, report, impact artifact, and evidence manifest.
3. Quarantined output is never partially treated as accepted without an explicit parent reconciliation note in `session-log.md`.
4. Any worker output that assumes a seam not present in the accepted `p1` compile-stable launch base is quarantined automatically.
5. If either lane is quarantined, the parent decides whether to repair sequentially or block the run.

## Parent-Owned Gates

There are no human approval gates in this orchestration plan. Every gate is parent-owned.

### Gate 0: Source lock

Required before any code work:

- required source inputs are read and locked
- run-state artifacts exist
- lane-ownership scaffolding exists
- parent-owned GitNexus preflight and `p1` impact capture are recorded
- the external QA test-plan artifact required by `PLAN.md` exists and its path is recorded

### Gate 1: Contract freeze

Required before worker launch:

- detached continuity contract frozen
- compile-stable seam scaffold written into the accepted launch base
- park-vs-fail contract frozen
- exact Worker A and Worker B boundaries frozen
- merge order frozen

### Gate 2: Window A acceptance

Required before integration:

- both lanes returned with validation artifacts
- both lanes respected ownership
- parent accepted or quarantined every lane

### Gate 3: Docs launch

Required before docs:

- merged tree proves the runtime behavior
- public control regressions are on the integrated tree
- no runtime seam remains unsettled

### Gate 4: Final closeout

Required before completion:

- required command wall and manual CLI evidence recorded
- repo-level shell-runtime evidence recorded under `validation/final/`
- docs reflect shipped truth
- `gitnexus_detect_changes()` recorded

## GitNexus Operating Procedure

GitNexus is mandatory for the future execution run described by this controller.

During `task/m24-p0-parent-run-init-and-source-lock`:

1. collect the symbol inventory that the run expects to edit
2. if the index is stale, run `npx gitnexus analyze`
3. run `gitnexus_impact` for every symbol the parent may edit during `p1`
4. record the preflight requirement and parent impact artifacts in `impact/preflight/`
5. do not permit `p1` code edits until those parent impact artifacts exist

Before Worker A edits owned symbols:

1. run `gitnexus_impact` for the readiness and continuity symbols in `state_store.rs` and `control.rs`
2. record direct callers, affected processes, and risk level in `impact-analysis.md`
3. stop and escalate to the parent if any required symbol is `HIGH` or `CRITICAL`

Before Worker B edits owned symbols:

1. run `gitnexus_impact` for the lifecycle symbols in `async_repl.rs` and `agents_cmd.rs`
2. record direct callers, affected processes, and risk level in `impact-analysis.md`
3. stop and escalate to the parent if any required symbol is `HIGH` or `CRITICAL`

During `task/m24-p4-parent-final-closeout`:

1. run `gitnexus_detect_changes()`
2. verify only expected symbols and execution flows changed
3. record the result in `validation/final/`
4. do not commit or declare the run complete without that record

## Context-Control Rules

Parent live context limit:

1. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
2. `run-state.json`
3. `contract-freeze.json`
4. the latest integrated diff summary

Worker A prompt contents only:

1. the task brief for `a1` and `a2`
2. the frozen detached continuity contract
3. the exact owned files
4. the explicit forbidden touch surfaces
5. the validation commands

Worker B prompt contents only:

1. the task brief for `b1` and `b2`
2. the frozen park-vs-fail contract
3. the exact owned files
4. the compile-stable readiness seam scaffold from `p1` that it may consume but not edit
5. the validation commands

Return contract for every worker:

1. changed files
2. symbols touched
3. commands run and exit codes
4. blockers, disputed assumptions, or seam drift requests

Additional context rules:

1. workers do not broad-search unrelated repo areas once ownership is assigned
2. workers do not edit docs, ADR text, or plan files
3. workers do not edit `.runs/**`
4. workers stop immediately if they need to widen the public contract or another lane's surface

## Validation And Acceptance

### Required command wall from `PLAN.md`

These commands are mandatory for the final run. They are frozen from [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md):

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

If any command above needs a narrower or syntactically adjusted cargo filter in the actual execution run, the parent may substitute the narrowest equivalent command only if:

1. it covers the same behavior
2. the substitution is written to `validation/final/command-mapping.md`
3. the original `PLAN.md` command and the exact replacement are both recorded

### Required manual CLI wall from `PLAN.md`

The parent must also rerun the real CLI flow and record the exact output artifacts:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
```

### Required repo-level shell-runtime evidence

Because this slice changes shell runtime behavior, the parent must also record the repo-guideline evidence surfaces in `.runs/plan-24-host-bootstrap-readiness/validation/final/`:

```bash
substrate shim doctor --json
substrate health --json
```

These outputs must be captured as:

- `.runs/plan-24-host-bootstrap-readiness/validation/final/shim-doctor.json`
- `.runs/plan-24-host-bootstrap-readiness/validation/final/health.json`

World-doctor evidence is conditional for this slice:

1. if the final merged change touches world backends, world transport, or world-facing runtime ownership behavior, the parent must run:

```bash
substrate world doctor --json
```

and record:

- `.runs/plan-24-host-bootstrap-readiness/validation/final/world-doctor.json`

2. if the final merged change does not alter world backends and the slice remains shell-runtime-only, the parent may skip `substrate world doctor --json` only by writing:

- `.runs/plan-24-host-bootstrap-readiness/validation/final/world-doctor-rationale.md`

That rationale must name the touched files and state that world-doctor evidence was not required because the merged diff stayed outside world backends.

### Required external QA artifact

The run must also preserve the QA-facing artifact required by `PLAN.md`:

- `~/.gstack/projects/<slug>/` artifact covering `start`, `turn`, `reattach`, parked-empty versus attention-needed outcomes, broken bootstrap versus valid parked continuity, post-`Accepted` explicit failure delivery, and detached-world fail-closed protection

The parent must record the resolved artifact path in:

- `.runs/plan-24-host-bootstrap-readiness/qa/test-plan-artifact-path.txt`

### Lane-level minimum validation

Worker A:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
```

Worker B:

```bash
cargo test -p shell async_repl -- --nocapture
```

If Worker B touches `agents_cmd.rs`, it must also run and record the narrowest equivalent caller-surface command that covers the changed `run_start(...)`, `run_turn(...)`, or `run_reattach(...)` behavior.

Parent integration-stage targeted validation, before the final wall if needed:

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

### Acceptance criteria for the orchestration run itself

The run is complete only if all of these are true on the same merged tree:

1. the required sentinels through `10--task-m24-p4-parent-final-closeout.ok` exist
2. `blocked.json` does not exist
3. no quarantined output remains unresolved
4. the final command wall is green and recorded
5. the manual CLI wall is green and recorded
6. `validation/final/shim-doctor.json` and `validation/final/health.json` exist, plus either `validation/final/world-doctor.json` or `validation/final/world-doctor-rationale.md`
7. the external QA artifact required by `PLAN.md` exists and its path is recorded
8. the merged tree proves:
   - readiness accepts valid detached continuity
   - clean bootstrap exit parks or attention-normalizes instead of invalidating
   - broken bootstrap still fails closed
   - parked-session `turn` succeeds on the exact session created by the bootstrap path
   - `reattach` succeeds on that same session without submitting a prompt
   - post-`Accepted` requests end with `Completed` or `Failed`
   - detached-world follow-up stays fail closed
9. docs reflect the shipped contract, not the stale slice-23 posture
10. `gitnexus_detect_changes()` is recorded and consistent with expected scope
11. `closeout.md` exists and `run-state.json` marks the run complete

## Blocked-Run Artifact Behavior

`blocked.json` is parent-written only, exactly once, at the moment the parent decides the run cannot advance.

Required fields in `.runs/plan-24-host-bootstrap-readiness/blocked.json`:

- `run_id`
- `authoritative_branch`
- `plan_source`
- `timestamp`
- `current_task_id`
- `gate_state`
- `stop_condition_id`
- `summary`
- `blocking_files`
- `worker_lane`
- `accepted_sentinels`
- `rejected_or_quarantined_outputs`
- `impact_artifacts`
- `next_required_parent_action`

Blocked-run rules:

1. parent writes `blocked.json` before any later-phase sentinel can be created
2. parent updates `run-state.json` to blocked in the same decision window
3. parent records the stop reason in `session-log.md`
4. `closeout.md` is not written on a blocked run

## Completion Conditions

This orchestration controller has succeeded only when the parent can say all of the following are true without qualification:

1. the current [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) slice, not the old slice-23 plan, drove the run from start to finish
2. the authoritative branch stayed `feat/host-orchestrator-durable-session`
3. the run used one honest two-worker window and no dishonest extra concurrency
4. the parent remained the only integrator and the only gate authority
5. the final merged tree fixed the original host bootstrap readiness and clean-detach parking defect end-to-end
6. the run left behind a complete `.runs/plan-24-host-bootstrap-readiness/` artifact trail that another operator can audit
