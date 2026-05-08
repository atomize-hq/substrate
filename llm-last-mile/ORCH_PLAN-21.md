# ORCH_PLAN-21: Execute PLAN-21 Through A Parent-Frozen Lima Parity Contract And One Honest Two-Lane Opening

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-21.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-21.md)  
Style reference: [ORCH_PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-18.md)  
Style reference: [ORCH_PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-19.md)  
Structure reference: user M26 orchestration-plan example  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:171)  
Execution type: fresh orchestration plan, Linux-contract parity rollout for macOS/Lima, parent-frozen contract, parent-only gates and integrations, docs/smoke closeout only after merged code truth  
Worker model: GPT-5.4 workers with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

- Authoritative branch: `feat/session-centric-state-store` on `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.
- Critical path stays local to the parent for contract freeze, launch gates, both integration windows, the validation wall, and final closeout.
- Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/`.
- Worker branches:
  - `codex/feat-session-centric-state-store-m21-backend-contract-parity`
  - `codex/feat-session-centric-state-store-m21-shared-owner-bootstrap-parity`
  - `codex/feat-session-centric-state-store-m21-shell-member-runtime-parity`
  - `codex/feat-session-centric-state-store-m21-macos-validation-docs-closeout`
- Worker policy: every child worker uses GPT-5.4 with `reasoning_effort=high`.
- Parent-owned run-state surfaces live under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/`.
- `.runs/plan-21/*` and `.runs/task-m21-*/**` are run artifacts for orchestration state and evidence, not assumed git-tracked deliverables.
- The only honest opening parallelism is `L1` plus `L2`; member-runtime parity waits for merged Window A truth; smoke/docs/gap-matrix closeout waits for merged Window B truth.

## Assumptions

1. [PLAN-21.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-21.md) remains the authoritative dependency graph for this run.
2. The authoritative integration checkout remains the current workspace checkout on `feat/session-centric-state-store`.
3. Linux already defines the correct shared-owner and retained member-runtime contract. macOS/Lima must converge to that contract rather than define a sibling contract.
4. The supported macOS posture is specifically the Lima-forwarded guest path. Unsupported non-Lima postures may continue to reject explicitly.
5. The exact identity tuple is frozen and must not change: `orchestration_session_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, `world_generation`.
6. The guest remains authoritative for world-sensitive member binding validation and cancel ownership.
7. Shared-owner bootstrap parity and backend contract widening can begin in parallel because they touch disjoint primary seams when ownership is enforced strictly.
8. Shell member-runtime parity depends on both merged bootstrap truth and merged backend field-preservation truth.
9. Scripts, docs, and gap-matrix truth must wait for merged runtime behavior. Early documentation edits are not honest.
10. The parent agent is the only integrator, the only approval authority, and the only writer of run-state artifacts.

## Execution Shape

This document is the execution controller for `PLAN-21`, not a restatement of it.

The run shape is frozen up front:

1. `task/m21-p1-parent-contract-freeze-and-run-init` is parent-only and freezes the Linux-source-of-truth contract, explicit shared-owner contract, exact identity checks, lane ownership, merge order, retry budget, blocked-run posture, and validation wall.
2. `task/m21-g1-implementation-window-a-launch-gate` is parent-only and is the only gate that may launch the first implementation window.
3. The only honest initial parallel window is exactly two lanes:
   - `task/m21-l1-backend-contract-parity`
   - `task/m21-l2-shared-owner-bootstrap-parity`
4. `task/m21-g2-window-a-integration-gate` and `task/m21-p2-parent-window-a-integration` are parent-only.
5. `task/m21-g3-member-runtime-launch-gate` starts only after merged Window A truth exists.
6. `task/m21-l3-shell-member-runtime-parity` runs alone on the accepted post-`p2` tree.
7. `task/m21-g4-member-runtime-integration-gate` and `task/m21-p3-parent-member-runtime-integration` are parent-only.
8. `task/m21-g5-closeout-launch-gate` starts only after merged member-runtime truth exists.
9. `task/m21-l4-macos-validation-docs-closeout` runs alone on the accepted post-`p3` tree.
10. `task/m21-g6-validation-wall-gate` is parent-only and permits one final validation wall.
11. `task/m21-p4-parent-validation-wall` runs the exact command wall.
12. `task/m21-p5-parent-closeout-phase` writes terminal artifacts and closes the run.

Canonical task IDs:

- `task/m21-p1-parent-contract-freeze-and-run-init`
- `task/m21-g1-implementation-window-a-launch-gate`
- `task/m21-l1-backend-contract-parity`
- `task/m21-l2-shared-owner-bootstrap-parity`
- `task/m21-g2-window-a-integration-gate`
- `task/m21-p2-parent-window-a-integration`
- `task/m21-g3-member-runtime-launch-gate`
- `task/m21-l3-shell-member-runtime-parity`
- `task/m21-g4-member-runtime-integration-gate`
- `task/m21-p3-parent-member-runtime-integration`
- `task/m21-g5-closeout-launch-gate`
- `task/m21-l4-macos-validation-docs-closeout`
- `task/m21-g6-validation-wall-gate`
- `task/m21-p4-parent-validation-wall`
- `task/m21-p5-parent-closeout-phase`

## Hard Guards

These are run-stopping invariants, not preferences:

1. Linux remains the source-of-truth orchestration contract.
2. Shared-world ownership stays explicit through `SharedWorldOwnerSpec` and `SharedWorldBindingSnapshot`.
3. macOS must stop rejecting the supported shared-owner path before guest bootstrap.
4. World-scoped member launch, targeted follow-up, and cancel must fail closed if the forwarded guest contract cannot be established.
5. No host-local fallback is allowed for world-scoped member runtime ownership.
6. Exact identity checks stay unchanged for `orchestration_session_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, and `world_generation`.
7. Targeted follow-up keeps using `/v1/member_turn/stream`.
8. Cancel stays guest-owned through `/v1/execute/cancel`.
9. Replacement stays shell-owned and fail-closed through `ReplaceExpectedGeneration`.
10. Backend-level execution and handles must stop silently dropping Linux orchestration fields.
11. No new orchestration model, daemon, transport family, or mac-native authority service is authorized.
12. No worker may weaken validation by duplicating proof validators or relaxing guest-side binding rules.
13. Docs and gap-matrix work are late-only. They may not start before merged code truth exists.
14. The parent is the only integrator, the only approval authority, and the only writer of `.runs/plan-21/**`.
15. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store`.

Stop the run, write `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/blocked.json`, and do not advance if any of these occur:

1. A lane requires a second orchestration model instead of Linux-parity widening.
2. A lane requires host-local fallback for world-scoped member runtime.
3. A lane requires changing endpoint ownership away from `/v1/member_turn/stream` or `/v1/execute/cancel`.
4. A lane requires relaxing any frozen identity check.
5. A lane requires parent invention of hybrid semantics across conflicting worker assumptions.
6. A worker touches files outside its frozen ownership surface.
7. `L3` starts before `p2` is accepted.
8. `L4` starts before `p3` is accepted.
9. The final validation wall cannot prove attach/create, replacement, lazy launch, targeted follow-up, cancel, backend field preservation, and doc truth on the same merged tree.
10. The gap matrix or `docs/WORLD.md` would need to overclaim beyond what the final validation wall proves.

### Blocked-Run Record Contract

`blocked.json` is parent-written only, exactly once, at the moment the parent decides the run cannot advance.

Required fields in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/blocked.json`:

- `run_id`
- `branch`
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
- `next_required_parent_action`

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/backend-contract-parity`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/shared-owner-bootstrap-parity`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/shell-member-runtime-parity`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/macos-validation-docs-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m21-backend-contract-parity`
- `codex/feat-session-centric-state-store-m21-shared-owner-bootstrap-parity`
- `codex/feat-session-centric-state-store-m21-shell-member-runtime-parity`
- `codex/feat-session-centric-state-store-m21-macos-validation-docs-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/backend-contract-parity -b codex/feat-session-centric-state-store-m21-backend-contract-parity feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/shared-owner-bootstrap-parity -b codex/feat-session-centric-state-store-m21-shared-owner-bootstrap-parity feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/shell-member-runtime-parity -b codex/feat-session-centric-state-store-m21-shell-member-runtime-parity feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-21/macos-validation-docs-closeout -b codex/feat-session-centric-state-store-m21-macos-validation-docs-closeout feat/session-centric-state-store
```

No separate parent integration worktree is introduced. The parent integrates only on `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `blocked.json` on failure only
- `quarantine/`
- `sentinels/`

These are orchestration artifacts and evidence surfaces. They are not assumed repository deliverables and are not part of the feature’s product contract unless a later human explicitly chooses to commit selected outputs.

Required per-task artifact roots:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p1-parent-contract-freeze-and-run-init/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-g1-implementation-window-a-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-l1-backend-contract-parity/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-l2-shared-owner-bootstrap-parity/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-g2-window-a-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p2-parent-window-a-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-g3-member-runtime-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-l3-shell-member-runtime-parity/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-g4-member-runtime-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p3-parent-member-runtime-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-g5-closeout-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-l4-macos-validation-docs-closeout/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-g6-validation-wall-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p4-parent-validation-wall/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p5-parent-closeout-phase/`

Each task directory must contain:

- `task.json`
- `commands.txt`
- `summary.md`

Each gate task directory must also contain:

- `gate-checklist.md`
- `gate-result.json`

Each worker task directory must also contain:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`

Required sentinels:

- `.runs/plan-21/sentinels/task-m21-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-21/sentinels/task-m21-g1-implementation-window-a-launch-gate.ok`
- `.runs/plan-21/sentinels/task-m21-l1-backend-contract-parity.ok`
- `.runs/plan-21/sentinels/task-m21-l2-shared-owner-bootstrap-parity.ok`
- `.runs/plan-21/sentinels/task-m21-g2-window-a-integration-gate.ok`
- `.runs/plan-21/sentinels/task-m21-p2-parent-window-a-integration.ok`
- `.runs/plan-21/sentinels/task-m21-g3-member-runtime-launch-gate.ok`
- `.runs/plan-21/sentinels/task-m21-l3-shell-member-runtime-parity.ok`
- `.runs/plan-21/sentinels/task-m21-g4-member-runtime-integration-gate.ok`
- `.runs/plan-21/sentinels/task-m21-p3-parent-member-runtime-integration.ok`
- `.runs/plan-21/sentinels/task-m21-g5-closeout-launch-gate.ok`
- `.runs/plan-21/sentinels/task-m21-l4-macos-validation-docs-closeout.ok`
- `.runs/plan-21/sentinels/task-m21-g6-validation-wall-gate.ok`
- `.runs/plan-21/sentinels/task-m21-p4-parent-validation-wall.ok`
- `.runs/plan-21/sentinels/task-m21-p5-parent-closeout-phase.ok`

`merge-order.json` is frozen during `p1` and must record:

- `integration_order: ["task/m21-l1-backend-contract-parity", "task/m21-l2-shared-owner-bootstrap-parity", "task/m21-l3-shell-member-runtime-parity", "task/m21-l4-macos-validation-docs-closeout"]`
- `l2_acceptance_basis: "accepted_l1_tree_only"`
- `l3_acceptance_basis: "accepted_p2_tree_only"`
- `l4_acceptance_basis: "accepted_p3_tree_only"`
- `replay_required_before_acceptance: true`
- `quarantine_on_branch_local_assumption: true`

## PLAN-21 Workstream Mapping

| PLAN-21 workstream | Orchestration tasks | Why this mapping is exact |
| --- | --- | --- |
| Persistent-session shared-owner parity | `task/m21-l2-shared-owner-bootstrap-parity`, `task/m21-g2-window-a-integration-gate`, `task/m21-p2-parent-window-a-integration` | This lane owns host-side pre-bootstrap rejection removal, forwarded attach/create, replacement proof, and shell-side fail-closed shared-world echo validation. |
| Backend contract parity | `task/m21-l1-backend-contract-parity`, `task/m21-g2-window-a-integration-gate`, `task/m21-p2-parent-window-a-integration` | This lane owns additive `ExecRequest` widening, backend preservation, Lima field propagation, and authoritative `shared_binding` surfacing. |
| Shell member-runtime parity on macOS | `task/m21-g3-member-runtime-launch-gate`, `task/m21-l3-shell-member-runtime-parity`, `task/m21-g4-member-runtime-integration-gate`, `task/m21-p3-parent-member-runtime-integration` | This window owns lazy launch, targeted follow-up reuse, and cancel on supported macOS/Lima, on top of merged backend and bootstrap truth. |
| Validation and documentation | `task/m21-g5-closeout-launch-gate`, `task/m21-l4-macos-validation-docs-closeout`, `task/m21-g6-validation-wall-gate`, `task/m21-p4-parent-validation-wall`, `task/m21-p5-parent-closeout-phase` | This closeout window owns smoke harness, macOS docs truth, gap-matrix truth, full validation wall, and final run closure. |

## Concurrency And Merge Order

Concurrency rules:

1. Worker cap is exactly `2` until `g2` completes.
2. `p1` must finish before any worker starts.
3. `g1` must be green before any worker starts.
4. The only honest initial parallel window is `L1` plus `L2`.
5. `L3` waits for accepted and integrated `L1` plus `L2`.
6. `L4` waits for accepted and integrated `L3`.
7. The validation wall runs exactly once on the final merged tree.
8. No third concurrent code lane is authorized because docs/smoke must describe integrated truth, not branch-local assumptions.

Why `L1` integrates before `L2`:

1. `L1` widens the upstream backend contract at [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs), [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs), and [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs).
2. `L3` depends on authoritative backend field preservation and authoritative Lima `shared_binding` surfacing. That truth must be frozen before shell follow-up parity starts.
3. `L2` is a narrower shell consumer of the forwarded bootstrap path and can be replayed cleanly on the accepted backend seam.
4. Integrating `L2` first risks proving shell behavior against branch-local Lima semantics that still erase orchestration fields underneath it.

## Kickoff Initialization Order

The parent initializes the run in this exact order:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/`, `.runs/plan-21/sentinels/`, `.runs/plan-21/quarantine/`, and every `.runs/task-m21-*/` directory.
2. Create `task.json`, `commands.txt`, `summary.md`, and `artifacts/` in every task directory.
3. Create `gate-checklist.md` and `gate-result.json` in every gate task directory.
4. Create placeholder `worker-report.md`, `worker-output.patch`, and `evidence-manifest.json` in every worker task directory.
5. Write `tasks.json` as the canonical launch queue and execution ledger.
6. Write `run-state.json` with `current_phase: "kickoff"`, `worker_cap: 2`, every task in `pending`, and empty accepted, rejected, quarantined, and blocked arrays.
7. Write `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
8. Freeze the exact `ExecRequest` constructor callsite list with parent search and record it in `lane-ownership.json`. `L1` may touch only that frozen constructor list in addition to its primary files.
9. Review the frozen hotspots:
   - [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
   - [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
   - [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
   - [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs)
   - [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)
   - [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
   - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   - [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
10. No pre-launch production scaffold is authorized. If either initial lane needs parent edits before it can start, block the run rather than invent a hybrid seed state.
11. Seed `L1` and `L2` worktrees from the exact same post-`p1` tree.
12. Write `session-log.md` with kickoff timestamp, branch, worktree roots, worker cap, and the explicit statement that the only honest initial parallel window is `L1` plus `L2`.

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m21-p1-parent-contract-freeze-and-run-init` | parent | — | authoritative checkout | frozen contract and seeded worker basis |
| `task/m21-g1-implementation-window-a-launch-gate` | parent | `p1` | authoritative checkout | launch approval for `L1` and `L2` |
| `task/m21-l1-backend-contract-parity` | worker | `g1` | `backend-contract-parity` / `codex/feat-session-centric-state-store-m21-backend-contract-parity` | additive backend contract widening and Lima field preservation |
| `task/m21-l2-shared-owner-bootstrap-parity` | worker | `g1` | `shared-owner-bootstrap-parity` / `codex/feat-session-centric-state-store-m21-shared-owner-bootstrap-parity` | macOS shared-owner bootstrap and replacement parity |
| `task/m21-g2-window-a-integration-gate` | parent | `l1`, `l2` | authoritative checkout | acceptance, rejection, or quarantine for Window A |
| `task/m21-p2-parent-window-a-integration` | parent | `g2` | authoritative checkout | merged backend parity then bootstrap parity |
| `task/m21-g3-member-runtime-launch-gate` | parent | `p2` | authoritative checkout | launch approval for `L3` |
| `task/m21-l3-shell-member-runtime-parity` | worker | `g3` | `shell-member-runtime-parity` / `codex/feat-session-centric-state-store-m21-shell-member-runtime-parity` | macOS lazy launch, targeted follow-up, and cancel parity |
| `task/m21-g4-member-runtime-integration-gate` | parent | `l3` | authoritative checkout | acceptance, rejection, or quarantine for `L3` |
| `task/m21-p3-parent-member-runtime-integration` | parent | `g4` | authoritative checkout | merged member-runtime parity truth |
| `task/m21-g5-closeout-launch-gate` | parent | `p3` | authoritative checkout | launch approval for `L4` |
| `task/m21-l4-macos-validation-docs-closeout` | worker | `g5` | `macos-validation-docs-closeout` / `codex/feat-session-centric-state-store-m21-macos-validation-docs-closeout` | smoke harness, docs, and gap-matrix truth |
| `task/m21-g6-validation-wall-gate` | parent | `l4` | authoritative checkout | permission to run the final validation wall |
| `task/m21-p4-parent-validation-wall` | parent | `g6` | authoritative checkout | exact PLAN-21 validation wall results |
| `task/m21-p5-parent-closeout-phase` | parent | `p4` | authoritative checkout | terminal run-state, closeout, and artifact audit |

## Lane Ownership By File Set

| Lane | Allowed files | Forbidden touch surfaces |
| --- | --- | --- |
| `L1` / backend contract parity | [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs), [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs), [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs), explicit `ExecRequest` constructor callsites frozen by parent into `lane-ownership.json`, adjacent `world-agent`, `world-api`, and `world-mac-lima` tests only if required by this contract widening | `async_repl.rs`, `world_persistent_session.rs`, `repl_persistent_session.rs`, `world_ops.rs`, `scripts/mac/**`, `docs/**`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `.runs/**` |
| `L2` / shared-owner bootstrap parity | [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs), [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs), [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs), shell tests directly proving these seams only | `world-api`, `world-agent`, `world-mac-lima`, `async_repl.rs`, `world_ops.rs`, `scripts/mac/**`, `docs/**`, `.runs/**` |
| `L3` / shell member-runtime parity | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs), [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) | `platform_world/mod.rs`, `world_persistent_session.rs`, `repl_persistent_session.rs`, `world-api`, `world-agent`, `world-mac-lima`, `scripts/mac/**`, `docs/**`, `.runs/**` |
| `L4` / macOS validation docs closeout | [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh), [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh), `scripts/mac/orchestration-smoke.sh`, [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) | every production Rust file, `.runs/**`, any doc claim beyond validated parity scope |

## Worker Interfaces

Every worker prompt must include:

1. task ID
2. attempt number
3. worktree path
4. branch
5. allowed files
6. forbidden files
7. frozen contract clauses relevant to that lane
8. required commands
9. retry budget
10. required return artifacts
11. sentinel path

Every worker return must include:

1. changed files list
2. commands run with exit codes
3. explicit attempt classification: `clean`, `retryable`, or `blocked`
4. unresolved assumptions or blockers
5. `worker-output.patch`
6. `worker-report.md`
7. `evidence-manifest.json`

## Parent Phases And Worker Packets

### `task/m21-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Freeze the Linux-source-of-truth parity contract in `contract-freeze.json`.
2. Freeze explicit ownership invariants for `SharedWorldOwnerSpec`, `SharedWorldBindingSnapshot`, `ReplaceExpectedGeneration`, `/v1/member_turn/stream`, and `/v1/execute/cancel`.
3. Freeze exact file ownership, merge order, retry budget, stop conditions, and validation wall.
4. Freeze the exact `ExecRequest` constructor list that `L1` may edit.
5. Seed `L1` and `L2` from the same post-`p1` tree.

Command gates:

```bash
cargo test -p world-api --no-run
cargo test -p world-mac-lima --no-run
cargo test -p shell --no-run
```

Acceptance:

1. `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, `tasks.json`, and `run-state.json` exist.
2. The freeze artifact explicitly records that this is parity work, not a new orchestration model.
3. The frozen contract records no host-local fallback, unchanged exact identity checks, unchanged endpoints, and parent-only replacement ownership.
4. The parent writes `.runs/plan-21/sentinels/task-m21-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m21-g1-implementation-window-a-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. `L1` and `L2` were seeded from the exact same post-`p1` tree.
3. `L1` prompt explicitly forbids shell follow-up or docs work.
4. `L2` prompt explicitly forbids backend contract edits and member-runtime work.
5. Both prompts repeat the frozen prohibitions on host-local fallback, identity relaxation, endpoint changes, and orchestration-model widening.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `.runs/plan-21/sentinels/task-m21-g1-implementation-window-a-launch-gate.ok`.

### `task/m21-l1-backend-contract-parity`

Owner:

- single worker on `codex/feat-session-centric-state-store-m21-backend-contract-parity`

Packet fields:

- Owned files:
  - [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs)
  - [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
  - [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
  - Frozen `ExecRequest` constructor callsites recorded by parent in `lane-ownership.json`
  - Adjacent tests only when required by this exact widening
- Forbidden touch surfaces:
  - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs)
  - `scripts/mac/**`
  - `docs/**`
  - `.runs/**`
- Exact scope:
  1. Add optional `shared_world` and `member_dispatch` fields to `ExecRequest`.
  2. Update every frozen `ExecRequest` constructor to be explicit about absence or presence.
  3. Pass those fields through [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) when backend execution is orchestration-sensitive.
  4. Make [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs) preserve those fields instead of zeroing them.
  5. Make Lima `WorldHandle.shared_binding` authoritative in owner mode.
  6. Keep Linux and Windows compile-clean without changing their semantics.
- Exact required commands:

```bash
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-api -- --nocapture
cargo test -p world-mac-lima -- --nocapture
```

- Exact acceptance:
  1. The lane touches only its owned files and frozen constructor callsites.
  2. `ExecRequest` widening is additive and optional.
  3. `world-agent` forwards orchestration fields into backend execution when relevant.
  4. Lima no longer zeros `shared_world` or `member_dispatch`.
  5. Owner-mode Lima handles surface authoritative `shared_binding`.
  6. Linux and Windows remain compile-clean without semantic widening.
  7. The worker writes `.runs/plan-21/sentinels/task-m21-l1-backend-contract-parity.ok`.

### `task/m21-l2-shared-owner-bootstrap-parity`

Owner:

- single worker on `codex/feat-session-centric-state-store-m21-shared-owner-bootstrap-parity`

Packet fields:

- Owned files:
  - [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs)
  - [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)
  - [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
  - Shell tests directly proving these seams only
- Forbidden touch surfaces:
  - `world-api`
  - `world-agent`
  - `world-mac-lima`
  - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - `scripts/mac/**`
  - `docs/**`
  - `.runs/**`
- Exact scope:
  1. Replace blanket non-Linux shared-owner rejection with capability-aware forwarding that allows the Lima-backed path and still rejects unsupported platforms.
  2. Keep one forwarded persistent-session request path.
  3. Keep [validate_shared_world_echo(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308) as the only shell-side proof validator.
  4. Preserve replacement semantics through `ReplaceExpectedGeneration`.
  5. Add or update shell tests for attach/create, replacement advancement, and invalid proof rejection on the macOS forwarded path.
- Exact required commands:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. macOS shared-owner attach/create reaches the forwarded guest path.
  3. macOS replacement still requires generation advancement.
  4. Missing, malformed, stale, or inactive proof still fails closed.
  5. Unsupported non-Lima paths still reject explicitly.
  6. The worker writes `.runs/plan-21/sentinels/task-m21-l2-shared-owner-bootstrap-parity.ok`.

### `task/m21-g2-window-a-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned patch, report, command transcript, and evidence manifest.
2. Every touched file is inside the lane’s ownership boundary.
3. `L1` landed additive backend contract widening without semantic drift.
4. `L2` landed forwarded bootstrap parity without reopening member-runtime or backend field preservation work.
5. `L2` is replayed only against the accepted `L1` tree.

Quarantine and retry behavior:

1. If `L1` broadened beyond additive backend seam work, quarantine `L1` immediately.
2. If `L2` depended on branch-local backend semantics instead of accepted `L1` truth, quarantine `L2`.
3. Retry budget remains `1` per lane and is available only for lane-local defects inside owned files.
4. If either lane violated ownership or frozen endpoint/identity constraints, mark non-retryable and quarantine rather than redrive.

Acceptance:

1. Accepted, rejected, or quarantined status for both lanes is recorded in `run-state.json`.
2. The parent writes `.runs/plan-21/sentinels/task-m21-g2-window-a-integration-gate.ok`.

### `task/m21-p2-parent-window-a-integration`

Owner:

- parent only

Scope:

1. Integrate accepted `L1` output first.
2. Re-run `L1` command gates on the authoritative checkout.
3. Replay `L2` on top of accepted `L1`. If `L2` assumed different backend field semantics, quarantine `L2` instead of hand-editing around the mismatch.
4. Integrate accepted `L2` output second.
5. Freeze the merged backend-plus-bootstrap truth before any member-runtime work starts.

Command gates:

```bash
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p world-api -- --nocapture
cargo test -p world-mac-lima -- --nocapture
cargo test -p shell --no-run
```

Acceptance:

1. The parent remains the sole integrator.
2. The authoritative tree now preserves backend orchestration fields and allows supported macOS shared-owner bootstrap.
3. No hybrid contract was invented during integration.
4. The parent writes `.runs/plan-21/sentinels/task-m21-p2-parent-window-a-integration.ok`.

### `task/m21-g3-member-runtime-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is green.
2. `L3` worktree is seeded from the exact accepted post-`p2` tree.
3. The prompt names only `async_repl.rs`, `world_ops.rs`, and the frozen shell test files.
4. The prompt explicitly forbids host-local fallback, backend contract edits, and docs work.

Acceptance:

1. No `L3` worker starts before this gate is green.
2. The parent writes `.runs/plan-21/sentinels/task-m21-g3-member-runtime-launch-gate.ok`.

### `task/m21-l3-shell-member-runtime-parity`

Owner:

- single worker on `codex/feat-session-centric-state-store-m21-shell-member-runtime-parity`

Packet fields:

- Owned files:
  - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  - [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
  - [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- Forbidden touch surfaces:
  - [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs)
  - [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)
  - [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)
  - `world-api`
  - `world-agent`
  - `world-mac-lima`
  - `scripts/mac/**`
  - `docs/**`
  - `.runs/**`
- Exact scope:
  1. Widen shell member-runtime orchestration so supported macOS/Lima no longer hard-errors on non-Linux cfg stubs.
  2. Reuse the existing `member_dispatch` request builder in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs).
  3. Route lazy launch through guest `member_dispatch`.
  4. Route targeted follow-up turns through `/v1/member_turn/stream`.
  5. Route bootstrap and submitted-turn cancel through `/v1/execute/cancel`.
  6. Preserve exact fail-closed behavior for wrong backend, stale generation, wrong world, and missing authoritative binding.
  7. Add or update shell tests proving reuse, mismatch rejection, and cancel routing on the macOS path.
- Exact required commands:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. Supported macOS/Lima no longer hits Linux-only hard-error stubs for world-scoped orchestration.
  3. No host-local fallback is introduced.
  4. Follow-up uses `/v1/member_turn/stream` and cancel uses `/v1/execute/cancel`.
  5. Exact identity mismatch paths still fail closed.
  6. The worker writes `.runs/plan-21/sentinels/task-m21-l3-shell-member-runtime-parity.ok`.

### `task/m21-g4-member-runtime-integration-gate`

Owner:

- parent only

Checks:

1. `L3` returned patch, report, command transcript, and evidence manifest.
2. Every touched file is inside `L3` ownership.
3. `L3` consumed accepted `p2` truth and did not invent endpoint drift or local fallback.
4. `L3` proof commands are green on the authoritative checkout.

Quarantine and retry behavior:

1. If `L3` changed endpoint ownership, introduced host-local fallback, or touched forbidden surfaces, quarantine `L3` immediately as non-retryable.
2. If `L3` failed only for lane-local defects within owned files, retry budget `1` remains available.
3. The parent must not hand-edit around mismatched assumptions. It either redrives the lane or blocks the run.

Acceptance:

1. Accepted, rejected, or quarantined status for `L3` is recorded in `run-state.json`.
2. The parent writes `.runs/plan-21/sentinels/task-m21-g4-member-runtime-integration-gate.ok`.

### `task/m21-p3-parent-member-runtime-integration`

Owner:

- parent only

Scope:

1. Integrate accepted `L3` output.
2. Re-run `L3` command gates on the authoritative checkout.
3. Freeze the merged shell parity truth before smoke/docs start.

Command gates:

```bash
cargo test -p shell -- --nocapture
```

Acceptance:

1. The authoritative tree now supports shared-owner bootstrap, member lazy launch, targeted follow-up, and cancel on the supported macOS/Lima contract.
2. The merged tree still preserves frozen Linux-owned semantics and fail-closed identity checks.
3. The parent writes `.runs/plan-21/sentinels/task-m21-p3-parent-member-runtime-integration.ok`.

### `task/m21-g5-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p3` is green.
2. `L4` worktree is seeded from the exact accepted post-`p3` tree.
3. The prompt names only `scripts/mac/**`, `docs/WORLD.md`, and `AGENT_ORCHESTRATION_GAP_MATRIX.md`.
4. The prompt explicitly forbids reopening production Rust files or overclaiming parity beyond validated support.

Acceptance:

1. No `L4` worker starts before this gate is green.
2. The parent writes `.runs/plan-21/sentinels/task-m21-g5-closeout-launch-gate.ok`.

### `task/m21-l4-macos-validation-docs-closeout`

Owner:

- single worker on `codex/feat-session-centric-state-store-m21-macos-validation-docs-closeout`

Packet fields:

- Owned files:
  - [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
  - [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
  - `scripts/mac/orchestration-smoke.sh`
  - [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
  - [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- Forbidden touch surfaces:
  - every production Rust file
  - `.runs/**`
  - any doc claim beyond validated parity scope
- Exact scope:
  1. Add `scripts/mac/orchestration-smoke.sh`.
  2. Keep [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh) and [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh) aligned with the new orchestration smoke path.
  3. Make the smoke harness prove attach/create, replacement, lazy launch, targeted follow-up, cancel, and mismatch rejection.
  4. Update [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md) to remove stale pre-bootstrap rejection claims for the supported macOS path.
  5. Update [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) to reflect validated parity and any remaining out-of-scope gaps only.
- Exact required commands:

```bash
scripts/mac/lima-warm.sh
scripts/mac/orchestration-smoke.sh
rg -n "shared-owner|member_turn/stream|execute/cancel|macOS|Lima" docs/WORLD.md AGENT_ORCHESTRATION_GAP_MATRIX.md scripts/mac/orchestration-smoke.sh scripts/mac/smoke.sh
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. There is one reproducible macOS/Lima orchestration smoke path.
  3. `docs/WORLD.md` matches the integrated runtime truth.
  4. The gap matrix does not overclaim Windows parity, host-local fallback, or a new orchestration model.
  5. The worker writes `.runs/plan-21/sentinels/task-m21-l4-macos-validation-docs-closeout.ok`.

### `task/m21-g6-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L4` returned and is accepted.
2. No quarantined or blocked output remains unresolved.
3. `validation-wall.md` names the exact final command order.
4. The parent can map every frozen completion promise to a command or artifact in the merged tree.

Quarantine and retry behavior:

1. If `L4` overclaimed beyond validated behavior, quarantine `L4` instead of editing docs by hand.
2. If `L4` failed only within owned files and stayed within validated scope, retry budget `1` remains available.
3. No validation wall starts until `L4` is either accepted or the run is blocked.

Acceptance:

1. The parent writes `.runs/plan-21/sentinels/task-m21-g6-validation-wall-gate.ok`.
2. The validation wall is permitted to run exactly once.

### `task/m21-p4-parent-validation-wall`

Owner:

- parent only

Scope:

1. Integrate only accepted `L4` output.
2. Run the exact PLAN-21 validation wall on the authoritative checkout in this order.
3. Record command results and artifact paths.
4. Confirm docs and gap matrix match the validated runtime truth.

Validation wall commands:

```bash
cargo test -p world-agent member_runtime
cargo test -p shell
cargo test -p world-api
cargo test -p world-mac-lima
cargo test --workspace -- --nocapture
scripts/mac/lima-warm.sh
scripts/mac/orchestration-smoke.sh
```

Required artifacts under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p4-parent-validation-wall/artifacts/`:

- `world-agent-member-runtime.txt`
- `shell.txt`
- `world-api.txt`
- `world-mac-lima.txt`
- `workspace.txt`
- `lima-warm.txt`
- `orchestration-smoke.txt`
- `contract-audit.md`

Acceptance:

1. All validation wall commands succeed in order.
2. The validation wall proves attach/create, replacement, lazy launch, targeted follow-up, cancel, backend field preservation, and doc truth on the same merged tree.
3. The parent writes `.runs/plan-21/sentinels/task-m21-p4-parent-validation-wall.ok`.

### `task/m21-p5-parent-closeout-phase`

Owner:

- parent only

Scope:

1. Confirm all required sentinels exist and `blocked.json` does not.
2. Confirm `tasks.json` and `run-state.json` match actual accepted, rejected, and quarantined outcomes.
3. Confirm no quarantined or blocked output was partially integrated.
4. Write terminal `closeout.md`.
5. Mark the run complete only if the final validated state matches the frozen parity contract.

Required artifacts under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m21-p5-parent-closeout-phase/artifacts/`:

- `closeout.md`
- `final-run-state.json`
- `final-task-ledger.json`
- `final-sentinel-audit.md`

Acceptance:

1. `run-state.json` records a successful terminal state.
2. `closeout.md` states exactly what parity landed and what remains out of scope.
3. The parent writes `.runs/plan-21/sentinels/task-m21-p5-parent-closeout-phase.ok`.

## Quarantine, Retry, And Blocked-Run Posture

1. Each worker lane has retry budget `1`.
2. Retry is allowed only for lane-local defects inside owned files.
3. Non-retryable violations include host-local fallback, endpoint drift, identity-check relaxation, orchestration-model widening, early docs work, and any cross-lane file touch.
4. If `L1` cannot stay additive at the backend seam, quarantine it immediately.
5. If `L2` can only work by bypassing the existing shared-world echo validator or by editing backend files, quarantine it immediately.
6. If `L3` can only pass by adding host-local fallback or changing `/v1/member_turn/stream` or `/v1/execute/cancel`, quarantine it immediately.
7. If `L4` overclaims parity beyond the smoke harness and final validation wall, quarantine it immediately.
8. The parent never hand-merges a hybrid truth from conflicting worker guesses.

When a lane is quarantined, the parent must preserve the returned materials in both places:

1. The original task artifact directory.
2. `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/quarantine/<task-id>/`.

`quarantine/<task-id>/quarantine-reason.json` must record:

- `task_id`
- `classification`
- `summary`
- `files_touched`
- `frozen_contract_clause_violated`
- `retry_available`
- `next_parent_action`

## Validation Wall

The parent may run the final validation wall exactly once, only after `g6` is green.

The wall is green only when all of these are true:

1. Backend contract widening is present and additive.
2. Supported macOS shared-owner bootstrap no longer pre-rejects.
3. Supported macOS replacement remains generation-advancing and fail-closed.
4. Supported macOS member lazy launch uses forwarded guest `member_dispatch`.
5. Targeted follow-up uses `/v1/member_turn/stream`.
6. Cancel uses `/v1/execute/cancel`.
7. No supported macOS path silently falls back to host-local member ownership.
8. `docs/WORLD.md` and `AGENT_ORCHESTRATION_GAP_MATRIX.md` match the validated runtime truth.
9. No quarantined or blocked output remains unresolved.

## Tests And Acceptance

### Backend Contract

Acceptance requires all of these to be true:

- `ExecRequest` carries optional `shared_world` and `member_dispatch` fields without changing absence semantics for existing callers.
- Every frozen `ExecRequest` constructor compiles and is explicit about presence or absence.
- `world-agent` passes orchestration-sensitive fields into backend execution.
- `world-mac-lima` preserves those fields instead of zeroing them.
- Owner-mode Lima returns authoritative `WorldHandle.shared_binding`.
- `cargo test -p world-agent member_runtime`, `cargo test -p world-api`, and `cargo test -p world-mac-lima` are green.

### Bootstrap Parity

Acceptance requires all of these to be true:

- Supported macOS/Lima shared-owner attach/create reaches the forwarded guest path.
- Supported macOS/Lima replacement remains shell-owned and requires generation advancement.
- Missing, malformed, stale, or inactive shared-world proof fails closed.
- Unsupported non-Lima paths still reject explicitly.
- `cargo test -p shell --lib -- --nocapture` and the relevant bootstrap coverage in shell tests are green.

### Member-Runtime Parity

Acceptance requires all of these to be true:

- Supported macOS/Lima no longer hard-errors on the Linux-only member-runtime stub path.
- Lazy launch uses guest `member_dispatch`.
- Targeted follow-up uses `/v1/member_turn/stream`.
- Cancel uses `/v1/execute/cancel`.
- Wrong backend, wrong world, stale generation, and missing authoritative binding all fail closed.
- `cargo test -p shell` proves reuse, mismatch rejection, and cancel routing on the merged tree.

### Smoke, Docs, And Gap-Matrix Truth

Acceptance requires all of these to be true:

- `scripts/mac/orchestration-smoke.sh` exists and is the single reproducible macOS/Lima orchestration smoke path.
- `scripts/mac/lima-warm.sh` and `scripts/mac/smoke.sh` align with the orchestration smoke path.
- `scripts/mac/orchestration-smoke.sh` proves attach/create, replacement, lazy launch, targeted follow-up, cancel, and mismatch rejection.
- `docs/WORLD.md` no longer claims pre-bootstrap rejection for the supported macOS/Lima path.
- `AGENT_ORCHESTRATION_GAP_MATRIX.md` reflects validated parity and only remaining out-of-scope gaps.

### Operator Flow And Run-State Artifacts

Acceptance requires all of these to be true:

- The parent is the sole writer of `.runs/plan-21/**` and `.runs/task-m21-*/**`.
- `tasks.json` and `run-state.json` accurately reflect accepted, rejected, quarantined, and blocked outcomes.
- Every required sentinel exists on the green path.
- `blocked.json` is absent on the green path.
- Final validation artifacts exist under the `p4` and `p5` task directories.
- `closeout.md` states exactly what landed and what remains out of scope without overclaim.

## Closeout Phase

The run is complete only when:

1. Every required sentinel exists.
2. `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-21/blocked.json` does not exist.
3. The exact validation wall is green.
4. `closeout.md` names the landed parity contract and any remaining out-of-scope gaps without overclaim.
5. The final merged tree reflects parity work, not a new orchestration model.
