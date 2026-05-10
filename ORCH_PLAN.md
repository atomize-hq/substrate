# ORCH_PLAN: Host Orchestrator Durable Session And Parked-Resumable Ownership

Live workspace branch: `feat/host-orchestrator-durable-session`  
Recorded branch in [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md): `feat/host-orchestrator-durable-session`  
Authoritative execution branch for this run: `feat/host-orchestrator-durable-session`  
Plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Source SOW: [llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
ADR anchor: [docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Style references: [llm-last-mile/ORCH_PLAN-20.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-20.md), [llm-last-mile/ORCH_PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-22.md)  
Execution type: fresh orchestration controller for shell runtime-state and lifecycle hardening, parent-frozen contract, parent-only gates and integration, docs closeout only after behavior is proven  
Live workspace root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

This document is the execution controller for [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), not a restatement of it.

The slice is complete only if the merged tree makes the Substrate-owned orchestration session, participant attachment truth, and session-local durable inbox the only authority for host ownership continuity, while preserving the public `start|turn|reattach|stop` contract exactly.

The exact code-worker cap is `2` because there are only two honest post-freeze implementation seams:

1. durable inbox persistence and pending-count authority in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. lifecycle parking/resume and terminal-envelope handling across [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), and [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

A third concurrent code lane would collide with one of the shared choke points that must stay serialized or parent-owned:

- [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) is a hotspot and must be settled during parent contract freeze
- [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) can only have one owning worker lane after freeze
- [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) and [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) are coupled and belong in one lane
- [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) is closeout-only and parent-owned
- docs are late and parent-owned by plan

Frozen run shape:

1. `task/m23-p0-parent-run-init-and-gitnexus-preflight`
2. `task/m23-p1-parent-contract-freeze-and-seam-prep`
3. `task/m23-g1-parent-lane-launch-gate`
4. parallel Window A
   - `task/m23-a1-worker-durable-inbox-implementation`
   - `task/m23-b1-worker-lifecycle-resume-and-terminal-envelope`
5. `task/m23-a2-worker-durable-inbox-validation-and-handoff`
6. `task/m23-b2-worker-lifecycle-validation-and-handoff`
7. `task/m23-g2-parent-integration-gate`
8. `task/m23-p2-parent-integration-and-cross-lane-test-finishing`
9. `task/m23-g3-parent-docs-launch-gate`
10. `task/m23-p3-parent-docs-closeout`
11. `task/m23-g4-parent-validation-wall-gate`
12. `task/m23-p4-parent-final-validation-and-closeout`

## Hard Guards

These are run-stopping invariants.

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/host-orchestrator-durable-session`.
2. The parent agent is the only integrator, the only approval authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/host-orch-durable-session/**`.
3. `substrate agent start|turn|reattach|stop` keep their current grammar and exact-selector rules.
4. `state` remains the orchestration lifecycle machine. `posture` is a separate persisted attachability and attention summary.
5. `attached_participant_id != null` if and only if `posture == active_attached`.
6. `attached_participant_id == null` for `parked_resumable`, `awaiting_attention`, and `terminal`.
7. A non-terminal detached session with `pending_inbox_count > 0` must normalize to `awaiting_attention`.
8. A non-terminal detached session with `pending_inbox_count == 0` and a resume-eligible host participant must normalize to `parked_resumable`.
9. `control_owner_retained`, `event_stream_active`, and `completion_observer_retained` remain diagnostics only.
10. Every unresolved orchestration event persists as one artifact under `sessions/<session>/inbox/<item_id>.json`.
11. Resolving an inbox item updates pending counts immediately and does not immediately delete the artifact.
12. Detached-world follow-up stays fail closed.
13. Once a public prompt request emits `Accepted`, it may terminate only with `Completed` or `Failed`.
14. No new daemon, no second source of truth, no public verb expansion, no fuzzy routing, no config/policy/schema work, and no platform-parity redesign are authorized.
15. Docs are late and parent-owned. No worker may edit docs or gap-matrix truth before merged code truth exists.
16. No worker may edit `.runs/**`.
17. If any frozen runtime contract point becomes disputed during implementation, the run stops and the parent writes `blocked.json`.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session/durable-inbox`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session/lifecycle-resume-terminal-envelope`

Worker branches:

- `codex/feat-macos-lima-shared-owner-member-runtime-parity-m23-durable-inbox`
- `codex/feat-macos-lima-shared-owner-member-runtime-parity-m23-lifecycle-resume-terminal-envelope`

Exact setup commands, run only after `task/m23-p1-parent-contract-freeze-and-seam-prep` is accepted and the main checkout `HEAD` equals the accepted post-freeze tree:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session/durable-inbox \
  -b codex/feat-macos-lima-shared-owner-member-runtime-parity-m23-durable-inbox \
  HEAD

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-host-orch-durable-session/lifecycle-resume-terminal-envelope \
  -b codex/feat-macos-lima-shared-owner-member-runtime-parity-m23-lifecycle-resume-terminal-envelope \
  HEAD
```

Parent integration surface:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/host-orchestrator-durable-session`
- no separate parent integration worktree is introduced
- the parent integrates in the main checkout only

## Parent-Owned Run-State Surface

Canonical parent-owned run path:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/host-orch-durable-session/`

Required top-level run artifacts:

- `run-state.json`
- `task-ledger.json`
- `session-log.md`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `gates/`
- `impact/`
- `validation/`
- `quarantine/`
- `sentinels/`
- `blocked.json` on blocked termination only
- `closeout.md` on successful completion only

Required per-task roots:

- `.runs/host-orch-durable-session/tasks/task-m23-p0-parent-run-init-and-gitnexus-preflight/`
- `.runs/host-orch-durable-session/tasks/task-m23-p1-parent-contract-freeze-and-seam-prep/`
- `.runs/host-orch-durable-session/tasks/task-m23-g1-parent-lane-launch-gate/`
- `.runs/host-orch-durable-session/tasks/task-m23-a1-worker-durable-inbox-implementation/`
- `.runs/host-orch-durable-session/tasks/task-m23-a2-worker-durable-inbox-validation-and-handoff/`
- `.runs/host-orch-durable-session/tasks/task-m23-b1-worker-lifecycle-resume-and-terminal-envelope/`
- `.runs/host-orch-durable-session/tasks/task-m23-b2-worker-lifecycle-validation-and-handoff/`
- `.runs/host-orch-durable-session/tasks/task-m23-g2-parent-integration-gate/`
- `.runs/host-orch-durable-session/tasks/task-m23-p2-parent-integration-and-cross-lane-test-finishing/`
- `.runs/host-orch-durable-session/tasks/task-m23-g3-parent-docs-launch-gate/`
- `.runs/host-orch-durable-session/tasks/task-m23-p3-parent-docs-closeout/`
- `.runs/host-orch-durable-session/tasks/task-m23-g4-parent-validation-wall-gate/`
- `.runs/host-orch-durable-session/tasks/task-m23-p4-parent-final-validation-and-closeout/`

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
- `impact-analysis.md`

Validation artifact roots:

- `.runs/host-orch-durable-session/validation/lane-a/`
- `.runs/host-orch-durable-session/validation/lane-b/`
- `.runs/host-orch-durable-session/validation/final/`
- `.runs/host-orch-durable-session/validation/validation-wall.md`

Quarantine root:

- `.runs/host-orch-durable-session/quarantine/<task-id>/`

Sentinel naming convention:

- `.runs/host-orch-durable-session/sentinels/<sequence>--<task-id>.ok`
- examples:
  - `01--task-m23-p0-parent-run-init-and-gitnexus-preflight.ok`
  - `02--task-m23-p1-parent-contract-freeze-and-seam-prep.ok`

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
- `quarantined_outputs`
- `merge_order`
- `frozen_runtime_contract_version`
- `frozen_symbol_list`
- `impact_artifact_paths`
- `validation_status`
- `blocked_state`
- `closeout_path`

## Product Runtime-State Surface Under Test

This run is proving and modifying these product runtime-state authorities:

- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/leases/<participant_id>.lease`
- `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/inbox/<item_id>.json`

The parent freezes the required additive session, participant, and inbox shapes in `contract-freeze.json` during `task/m23-p1-parent-contract-freeze-and-seam-prep`.

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m23-p0-parent-run-init-and-gitnexus-preflight` | parent | — | main checkout / `feat/host-orchestrator-durable-session` | run scaffold, symbol inventory, GitNexus preflight impact capture |
| `task/m23-p1-parent-contract-freeze-and-seam-prep` | parent | `p0` | main checkout / `feat/host-orchestrator-durable-session` | frozen posture/participant/store seam, lane ownership freeze, accepted post-freeze tree |
| `task/m23-g1-parent-lane-launch-gate` | parent | `p1` | main checkout / `feat/host-orchestrator-durable-session` | launch decision for Window A and worktree creation |
| `task/m23-a1-worker-durable-inbox-implementation` | worker A | `g1` | durable-inbox / `codex/feat-macos-lima-shared-owner-member-runtime-parity-m23-durable-inbox` | inbox persistence, pending-count maintenance, store-local tests |
| `task/m23-a2-worker-durable-inbox-validation-and-handoff` | worker A | `a1` | durable-inbox / same branch | validated worker report, patch, evidence manifest, impact artifact |
| `task/m23-b1-worker-lifecycle-resume-and-terminal-envelope` | worker B | `g1` | lifecycle-resume-terminal-envelope / `codex/feat-macos-lima-shared-owner-member-runtime-parity-m23-lifecycle-resume-terminal-envelope` | clean-detach parking, parked `turn`/`reattach`, explicit terminal envelope handling |
| `task/m23-b2-worker-lifecycle-validation-and-handoff` | worker B | `b1` | lifecycle-resume-terminal-envelope / same branch | validated worker report, patch, evidence manifest, impact artifact |
| `task/m23-g2-parent-integration-gate` | parent | `a2`, `b2` | main checkout / `feat/host-orchestrator-durable-session` | accept or quarantine decision for both lanes |
| `task/m23-p2-parent-integration-and-cross-lane-test-finishing` | parent | `g2` | main checkout / `feat/host-orchestrator-durable-session` | merged runtime truth, parent-owned integration tests, cross-lane fixes |
| `task/m23-g3-parent-docs-launch-gate` | parent | `p2` | main checkout / `feat/host-orchestrator-durable-session` | decision that behavior is proven enough for docs |
| `task/m23-p3-parent-docs-closeout` | parent | `g3` | main checkout / `feat/host-orchestrator-durable-session` | late docs truth updates only |
| `task/m23-g4-parent-validation-wall-gate` | parent | `p3` | main checkout / `feat/host-orchestrator-durable-session` | permission to run final wall |
| `task/m23-p4-parent-final-validation-and-closeout` | parent | `g4` | main checkout / `feat/host-orchestrator-durable-session` | final validation wall, detect_changes, closeout artifacts |

## Frozen Ownership Boundaries

Parent-only during `p1`:

- [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) for seam freeze and invariant helpers only

Worker A owned after `g1`:

- [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) for inbox persistence and pending-count maintenance only
- unit tests local to store-owned behavior

Worker B owned after `g1`:

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- read-only consumption of the frozen [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) seam unless parent reopens the gate

Parent-owned late:

- [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- docs and gap-matrix truth
- final merged-tree glue if required

## Task Graph And Control Points

### `task/m23-p0-parent-run-init-and-gitnexus-preflight`

Parent only.

Required actions:

1. create `.runs/host-orch-durable-session/` and every task root
2. write `task-ledger.json`, `run-state.json`, `lane-ownership.json`, and initial `validation/validation-wall.md`
3. inventory symbols that will be edited
4. run GitNexus impact analysis for the parent-frozen symbols before any edits
5. if GitNexus says the index is stale, run `npx gitnexus analyze`

Required preflight symbol list:

- `OrchestrationSessionRecord`
- session invariants and participant-state symbols in `session.rs`
- exact public turn resolution seam in `state_store.rs`
- public prompt bridge symbols in `control.rs`
- `run_turn(...)`
- `run_reattach(...)`
- the clean-exit lifecycle seam in `async_repl.rs`

Deliverables:

- `impact/preflight/` artifacts
- `session-log.md` entry recording blast radius and any high-risk warnings
- `01--task-m23-p0-parent-run-init-and-gitnexus-preflight.ok`

### `task/m23-p1-parent-contract-freeze-and-seam-prep`

Parent only.

Purpose:

- freeze the persisted posture contract
- freeze participant attachment/resume truth
- freeze the exact `state_store.rs` seam that later lanes must use
- settle the `session.rs` hotspot before concurrency starts

Required actions:

1. run GitNexus impact analysis for each symbol actually edited in `orchestration_session.rs`, `session.rs`, and `state_store.rs`
2. add or adjust additive posture and participant fields and invariant helpers
3. centralize impossible-combination rejection and posture normalization
4. write `contract-freeze.json` with:
   - frozen invariants
   - required session/participant/inbox shapes
   - lane ownership
   - stop conditions
   - allowed `state_store.rs` post-freeze touch points for worker A
5. write `merge-order.json`

Acceptance:

- `session.rs` no longer remains a live ownership ambiguity for both lanes
- `state_store.rs` seam is narrow enough that worker B can consume it without drift
- post-freeze tree is accepted in the main checkout
- `02--task-m23-p1-parent-contract-freeze-and-seam-prep.ok`

### `task/m23-g1-parent-lane-launch-gate`

Parent only.

Purpose:

- verify the post-freeze tree is the exact base for both lanes
- create worktrees from `HEAD`
- launch only if ownership boundaries are still honest

Gate must reject launch if:

- worker B already needs `state_store.rs` API drift beyond the frozen seam
- worker A cannot isolate inbox work inside `state_store.rs`
- parent-owned integration tests would need to start early

Acceptance:

- worktrees created
- `03--task-m23-g1-parent-lane-launch-gate.ok`

### `task/m23-a1-worker-durable-inbox-implementation`

Worker A only.

Scope:

- add the canonical durable inbox
- atomically persist inbox items
- maintain authoritative pending counts
- keep resolved inbox artifacts inspectable

Before first edit:

1. run GitNexus impact analysis for owned `state_store.rs` symbols
2. record the result in `impact-analysis.md`

Minimum lane-A behavior:

- create `sessions/<session>/inbox/<item_id>.json`
- support `approval_required`, `completion_notice`, `follow_up_message`, and `runtime_alert`
- support `pending`, `acknowledged`, and `dismissed`
- atomically persist item state and session count updates
- ensure detached-host posture reads become authoritative through inbox count truth
- keep steady-state reads O(1) via `pending_inbox_count`

### `task/m23-a2-worker-durable-inbox-validation-and-handoff`

Worker A only.

Minimum lane-A validation before return:

```bash
cargo test -p shell state_store -- --nocapture
```

If the filter above is too coarse, run the narrowest equivalent filter that executes all newly added inbox and pending-count tests, and record the exact command used.

Required handoff artifacts:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Acceptance target for parent:

- lane A stayed inside owned seams
- lane A did not reopen session contract or public CLI semantics

### `task/m23-b1-worker-lifecycle-resume-and-terminal-envelope`

Worker B only.

Scope:

- clean-detach parks instead of invalidates when the session remains valid
- `turn` resumes valid parked host sessions
- `reattach` resumes valid parked host sessions without prompt submission
- `Accepted` always terminates with `Completed` or `Failed`
- detached-world follow-up remains fail closed

Before first edit:

1. run GitNexus impact analysis for owned symbols in `async_repl.rs`, `agents_cmd.rs`, and `control.rs`
2. record the result in `impact-analysis.md`

If worker B discovers required `state_store.rs` API drift after launch:

- pause immediately
- write the gap in `worker-report.md`
- do not widen edits
- hand the issue back to the parent for gate re-evaluation

### `task/m23-b2-worker-lifecycle-validation-and-handoff`

Worker B only.

Minimum lane-B validation before return:

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell control -- --nocapture
```

If those filters are too coarse, run the narrowest equivalent commands that execute the newly added lifecycle, parking, and terminal-envelope tests, and record the exact commands used.

Required handoff artifacts:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Acceptance target for parent:

- lane B stayed out of heavy `state_store.rs` drift
- lane B did not touch integration tests or docs
- detached-world fail-closed behavior remains explicit

### `task/m23-g2-parent-integration-gate`

Parent only.

Purpose:

- review both worker reports
- accept, reject, or quarantine each lane
- decide whether parallelism remained honest

Gate rules:

1. both lanes must originate from the exact accepted post-freeze tree
2. if lane B required unplanned `state_store.rs` seam changes, pause lane B and resolve with the parent before continued parallelism
3. quarantine any worker output that:
   - widened public contract
   - touched non-owned files
   - edited docs
   - reopened `session.rs`
   - assumed fuzzy routing or detached-world continuity

Acceptance:

- `04--task-m23-g2-parent-integration-gate.ok`

### `task/m23-p2-parent-integration-and-cross-lane-test-finishing`

Parent only.

Purpose:

- merge accepted lane outputs into the main checkout
- finish parent-owned integration tests and any narrow glue
- keep [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) parent-owned and late

Required parent test additions and finishing work:

- replace the clean-exit invalidation regression with parked-resumable behavior
- extend [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) for:
  - parked-host `turn`
  - parked-host `reattach`
  - detached-world non-regression
  - explicit terminal-envelope behavior
- ensure unit tests exist in:
  - [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
  - [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
  - [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

Acceptance:

- integrated tree reflects merged runtime truth
- parent-owned tests compile and run on the integrated tree
- `05--task-m23-p2-parent-integration-and-cross-lane-test-finishing.ok`

### `task/m23-g3-parent-docs-launch-gate`

Parent only.

Docs stay late.

Gate condition:

- code behavior is proven on the integrated tree
- no production runtime seam remains unsettled
- only then may docs begin

Acceptance:

- `06--task-m23-g3-parent-docs-launch-gate.ok`

### `task/m23-p3-parent-docs-closeout`

Parent only.

Likely doc targets:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), only if runtime ownership wording actually needs it
- the SOW or ADR, only if implementation proved wording drift

Acceptance:

- docs describe the same authority model as the integrated code
- no early speculative wording remains
- `07--task-m23-p3-parent-docs-closeout.ok`

### `task/m23-g4-parent-validation-wall-gate`

Parent only.

Purpose:

- freeze the exact final command set
- ensure docs are already complete
- ensure no quarantined output remains unresolved

Acceptance:

- `08--task-m23-g4-parent-validation-wall-gate.ok`

### `task/m23-p4-parent-final-validation-and-closeout`

Parent only.

Required final actions:

1. run the full validation wall
2. run `gitnexus_detect_changes()` before any commit or final handoff
3. record final accepted symbol and flow drift
4. write `closeout.md`
5. mark the run complete in `run-state.json`

Acceptance:

- `09--task-m23-p4-parent-final-validation-and-closeout.ok`

## Gate Model

There are no human approval gates in this orchestration plan. Every gate is a parent validation checkpoint.

### Gate 0: Scope lock

Required before edits:

- preflight files read
- run-state initialized
- ownership matrix recorded
- GitNexus preflight completed

### Gate 1: Contract freeze

Required before any worker starts:

- posture and participant contracts frozen
- [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) hotspot settled
- [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) seam stable enough for both lanes
- parent stamps `task/m23-p1-parent-contract-freeze-and-seam-prep.ok`

### Gate 2: Lane acceptance

Required before merge:

- lane A meets inbox durability acceptance
- lane B meets lifecycle and terminal-envelope acceptance
- parent either accepts both or quarantines one and collapses to sequential repair

### Gate 3: Docs launch

Required before docs:

- merged tree passes the integration-stage validation
- no unresolved blocker remains in `session-log.md`
- parent stamps `task/m23-p2-parent-integration-and-cross-lane-test-finishing.ok`

### Gate 4: Validation wall

Required before final closeout:

- docs updated only after Gate 3
- wording matches shipped behavior
- no quarantined output remains unresolved

### Gate 5: Final closeout

Required before commit or handoff:

- `gitnexus_detect_changes()` matches expected scope
- `closeout.md` written
- run marked complete

## Merge Order

`merge-order.json` is frozen during `p1` and governs integration.

1. Both worker lanes branch from the exact accepted post-freeze tree in the main checkout.
2. Lane A does not automatically merge before lane B.
3. The parent may integrate in whichever order minimizes conflict unless the frozen seam requires lane-A-first because worker B cannot proceed without accepted `state_store.rs` seam changes.
4. If lane B needs `state_store.rs` API drift after launch, lane B is paused, the parent resolves the seam in the main checkout, and continued parallelism is re-evaluated before worker B resumes.
5. Docs begin only from the accepted integrated tree after `p2`, never from a worker branch.
6. The parent never hand-merges contradictory assumptions about posture, inbox count authority, or terminal-envelope semantics. Conflicting worker output is quarantined instead.

## Blocked-Run Record Contract

`blocked.json` is parent-written only, exactly once, at the moment the parent decides the run cannot advance.

Required fields in `.runs/host-orch-durable-session/blocked.json`:

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

Parent-only blocked-write rules:

1. parent writes `blocked.json` before any later-phase sentinel can be created
2. parent updates `run-state.json` to blocked in the same decision window
3. parent records the evidence in `session-log.md`
4. `closeout.md` is not written on a blocked run

Accepted versus quarantined output rules:

- accepted output is either merged or explicitly marked accepted in `run-state.json`
- quarantined output is copied under `.runs/host-orch-durable-session/quarantine/<task-id>/`
- quarantined output must include the worker patch, report, and evidence manifest
- quarantined output is never partially treated as accepted without an explicit parent reconciliation note in `session-log.md`

## GitNexus Operating Procedure

GitNexus is not optional in this run.

### Preflight

During `task/m23-p0-parent-run-init-and-gitnexus-preflight`:

1. run impact analysis for the initial symbol inventory
2. record blast radius in `impact/preflight/`
3. if the index is stale, run `npx gitnexus analyze`

### Lane startup

Before worker A or worker B edits owned symbols:

1. run impact analysis for the owned symbols
2. attach results to the lane task directory
3. if GitNexus returns `HIGH` or `CRITICAL` risk for a required edit, the worker stops and escalates to the parent instead of proceeding silently

### Final closeout

During `task/m23-p4-parent-final-validation-and-closeout`:

1. run `gitnexus_detect_changes()`
2. verify only expected symbols and flows changed
3. record the result in `validation/final/`
4. do not commit or declare completion without that record

## Lane-Level Validation Requirements

### Worker A minimum validation

Run at least one of these before return:

```bash
cargo test -p shell state_store -- --nocapture
```

And if [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) tests were touched indirectly by parent seam prep, the parent may additionally require:

```bash
cargo test -p shell orchestration_session -- --nocapture
```

### Worker B minimum validation

Run at least these before return:

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell control -- --nocapture
```

If the actual narrow test filters differ, record the exact commands in `commands.txt`.

### Parent integration-stage targeted validation

Helpful early commands before the final wall:

```bash
cargo test -p shell orchestration_session -- --nocapture
cargo test -p shell session -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell control -- --nocapture
```

The parent may replace these with narrower exact filters if the test names are known and cover the same new behaviors.

## Final Parent Validation Wall

Required commands from [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md):

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell -- --nocapture
cargo test --workspace -- --nocapture
```

Run these too if schema wording or runtime-state surfaces broaden coupling:

```bash
substrate agent status --json
substrate agent doctor --json
```

The validation wall is only accepted if the same merged tree proves all of these:

1. clean attached-host exit parks a valid session instead of invalidating it
2. canonical posture persists as `active_attached`, `parked_resumable`, `awaiting_attention`, or `terminal`
3. detached sessions with pending inbox items surface `awaiting_attention`
4. `turn` resumes valid parked host sessions under exact `(session, backend)` routing only
5. `reattach` resumes valid parked host sessions without prompt submission
6. detached-world follow-up still fails closed
7. every unresolved world-originated event persists in the session-local inbox
8. resolved inbox items update counts immediately and remain inspectable
9. every request that emitted `Accepted` ends with explicit `Completed` or `Failed`
10. docs and repo-truth artifacts describe the proven behavior and not the pre-fix model

## Context-Control Rules

1. Parent keeps only four live controller artifacts in working context at any one time:
   - [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
   - `run-state.json`
   - `session-log.md`
   - the latest integration diff summary
2. Each worker prompt contains only:
   - its task brief
   - its owned file set
   - the frozen invariants relevant to that lane
   - required validation commands
   - explicit forbidden touch surfaces
3. Workers return only:
   - changed files
   - symbols touched
   - commands run and exit codes
   - blockers or unresolved assumptions
4. Workers do not broad-search unrelated repo areas once ownership is assigned.
5. Workers do not touch `.runs/host-orch-durable-session/*`.
6. Workers do not edit [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs).
7. Workers stop immediately if they need to widen selector semantics, public contract, or parent-owned seams.
8. Parent resets context at every gate by re-reading `run-state.json`, `session-log.md`, and accepted sentinels before taking the next step.
9. Docs remain context-isolated until after the validation wall gate is passed.
10. If parallel execution becomes dishonest because of shared-file pressure, the parent collapses to sequential execution instead of forcing concurrency.
11. Close each worker once its output is either accepted or quarantined. Do not keep idle workers open through later gates.

## Assumptions / Risks

- [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) is the main concurrency risk. If worker B needs fresh store API drift after launch, parallelism should collapse to a parent-mediated sequence.
- The terminal-envelope hardening may expose more late-EOF paths than the current tests cover, which can expand parent-owned integration work without changing overall plan scope.
- [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) is the hotspot most likely to leak cross-lane assumptions. If it reopens after `p1`, the honest move is to stop and re-freeze, not to force concurrency.
- Doc closeout scope may grow slightly if runtime-state terminology is duplicated outside the currently named files.
