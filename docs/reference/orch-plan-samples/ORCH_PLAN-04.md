# ORCH_PLAN-04: Thread World Binding Into Runtime State

Branch: `feat/thread-world-binding`  
Plan source: [PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-04.md)  
Execution type: backend-only orchestration plan, no UI scope

## Summary

This document is the execution control artifact for `PLAN-04`, not a summary of it.

The parent agent owns the contract, the single-writer runtime integration path, and the final branch state. Worker lanes exist only where the repo seams permit safe parallelism without corrupting the startup and restart ordering centered in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).

The critical path is:

1. persist a pending parent [OrchestrationSessionRecord](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) before the first shared-world attach/create request,
2. ensure that first request is owner-bound,
3. make the persisted parent-session record the live binding authority for `world_id/world_generation`,
4. reorder alerts and host runtime events so they publish only after parent binding persistence succeeds,
5. expose an optional proof in `substrate agent toolbox status --json` while keeping the selected host row in `substrate agent status --json` unchanged.

This slice must stop if implementation drifts into lease-sidecar expansion, host participant world binding, UI changes, or `PLAN-05` invalidation semantics.

## Orchestration State Surfaces

These are run artifacts for the orchestration control plane. They are derived process surfaces, not product source.

### Canonical run state

Single local source of truth for the run:

- `.runs/plan-04/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch/worktree assignment,
- worker ownership,
- gate status,
- blocked or green terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local derived surfaces under the repo root:

- `.runs/plan-04/queue.json`
  - ordered execution queue and dispatch metadata
- `.runs/plan-04/session.log`
  - append-only parent log for decisions, gate results, merge outcomes, and blockers
- `.runs/plan-04/sentinels/task-m04-a1-preflight.ok`
- `.runs/plan-04/sentinels/task-m04-a2-foundation.ok`
- `.runs/plan-04/sentinels/task-m04-b1-record-authority.ok`
- `.runs/plan-04/sentinels/task-m04-b2-toolbox-proof.ok`
- `.runs/plan-04/sentinels/task-m04-c1-runtime-integration.ok`
- `.runs/plan-04/sentinels/task-m04-d1-world-tests.ok`
- `.runs/plan-04/sentinels/task-m04-d2-status-tests-docs.ok`
- `.runs/plan-04/sentinels/task-m04-e1-closeout.ok`
- `.runs/plan-04/blocked.json`
  - present only on blocked termination
- `.runs/plan-04/closeout.md`
  - final successful-run closeout

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Absence of a sentinel means the task is not accepted.
3. Blocked termination writes `blocked.json` and does not write downstream `.ok` sentinels.

## Concurrency Policy

1. The parent remains the only integrator.
2. The parent remains the only writer of final branch state on `feat/thread-world-binding`.
3. Exact concurrency cap: `2` active worker lanes.
4. Reason for the cap:
   - [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) is the dominant overlap surface,
   - startup/restart ordering is single-writer work,
   - more than two worker lanes here would create merge churn faster than throughput.
5. The parent may run `0`, `1`, or `2` workers depending on phase stability.
6. `async_repl.rs` is single-writer by default. No worker lane edits it while foundational runtime integration is still moving.
7. Test worktrees are launched only after parent runtime integration stabilizes.

## Hard Guards

### Locked invariants

1. The parent-session record is the live binding authority. The host participant manifest is not.
2. The first shared-world `AttachOrCreate` request must be owner-bound and must occur only after a persisted pending parent session record exists.
3. `world_restarted`, `world_restart_required`, and host/orchestrator runtime events that claim a world binding must publish only after parent binding persistence succeeds.
4. `toolbox status --json` may gain an optional proof object; the selected host row in `agent status --json` must remain unchanged.
5. `--no-world` is an explicit bypass path. It must preserve host-only startup ordering and must not attempt world-binding persistence.
6. Lease-sidecar expansion is deferred. No worker may broaden slice scope to make lease files authoritative.
7. Integration stops immediately if `PLAN-04.md`, [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md), or [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md) drift in a way that contradicts these rules.

### File-level boundaries

Parent-owned critical overlap surface:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Parent-reviewed shared contract surfaces:

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs)

Validation and documentation surfaces:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
- [llm-last-mile/05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)

### Non-negotiable stop conditions

Stop the orchestration and return to planning if any of these occur:

1. A worker proposes writing `world_id/world_generation` into host-scoped participant manifests.
2. The first attach/create request cannot be owner-bound without changing `PLAN-03` semantics.
3. `toolbox status --json` proof requires exposing `orchestration_session_id` publicly.
4. The selected host status row changes shape or gains world fields.
5. Runtime ordering changes cannot be proven with test coverage in the real shell test files.
6. Two workstreams need concurrent edits to `async_repl.rs`.

## Approval And Gate Model

There are no true human approval gates defined for this run.

Replacement control mechanism:

1. Parent validation gates at every phase transition.
2. Blocked-path recording in `.runs/plan-04/blocked.json`.
3. Green-path recording in `.runs/plan-04/closeout.md`.
4. No phase transition without a parent-written sentinel for the prior task.

Green-path rule:

- The parent writes the task sentinel, updates `run-state.json`, appends `session.log`, and opens the next task set.

Blocked-path rule:

- The parent writes `blocked.json`, updates `run-state.json` to `blocked`, appends `session.log`, and stops dispatch.

## Workstream Plan

### Orchestration topology

Parent checkout:

- current checkout on `feat/thread-world-binding`

Child worktrees and branches:

1. `../substrate-m04-record-authority`
   - branch: `codex/feat-thread-world-binding-m04-record-authority`
2. `../substrate-m04-toolbox-proof`
   - branch: `codex/feat-thread-world-binding-m04-toolbox-proof`
3. `../substrate-m04-world-tests`
   - branch: `codex/feat-thread-world-binding-m04-world-tests`
4. `../substrate-m04-status-tests-docs`
   - branch: `codex/feat-thread-world-binding-m04-status-tests-docs`

The parent remains the only integrator. Subagents do not merge each other’s work. They return patches, exact file lists, test evidence, and blockers to the parent.

### Task index

Parent-only serialized tasks:

- `task/m04-a1-preflight`
- `task/m04-a2-foundation`
- `task/m04-c1-runtime-integration`
- `task/m04-e1-closeout`

Worker-owned tasks:

- `task/m04-b1-record-authority`
- `task/m04-b2-toolbox-proof`
- `task/m04-d1-world-tests`
- `task/m04-d2-status-tests-docs`

### task/m04-a1-preflight

Ownership:

- parent only

Primary objective:

- establish the run, freeze scope, and create the control-plane surfaces

Scope:

1. Re-read [PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-04.md), [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md), and [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md).
2. Freeze the invariant list into the worker prompt packet.
3. Confirm the slice remains backend-only.
4. Confirm the active seams still center on:
   - `start_world_session(...)`
   - `start_host_orchestrator_runtime(...)`
   - `handle_detected_world_drift(...)`
   - `restart_world_session(...)`
   - `resolve_live_orchestrator_session(...)`
   - `build_toolbox_status_report(...)`
5. Initialize:
   - `.runs/plan-04/run-state.json`
   - `.runs/plan-04/queue.json`
   - `.runs/plan-04/session.log`

Acceptance:

1. Parent can name the persisted-authority barrier.
2. Parent can name the proof surface and the selected-row non-change contract.
3. `run-state.json` records the initial phase and queue.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-a1-preflight.ok`

Blocked-path output:

- `.runs/plan-04/blocked.json`

### Parent validation gate A

Required before `task/m04-a2-foundation` starts:

1. No packet contradiction is unresolved.
2. The parent can state why host participant world binding is forbidden in this slice.
3. The parent can state why `toolbox status --json` is the proof surface and `agent status --json` is not.

### task/m04-a2-foundation

Ownership:

- parent only

Why serialized:

- [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) owns startup, runtime bootstrap, restart ordering, fail-closed behavior, and alert emission. Splitting first-start context plumbing across workers would be false parallelism.

Primary files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs) only if startup request plumbing requires aligned validation work

Scope:

1. Introduce durable startup context before first world start.
2. Persist a pending parent session record in `Allocating` before the first shared-world request.
3. Thread that context through:
   - first startup,
   - startup drift,
   - auto-restart,
   - fail-closed restart-required handling.
4. Make `--no-world` the explicit bypass path.
5. Lock the first `AttachOrCreate` request contract so workers can safely code against it.

Acceptance:

1. A persisted parent session record exists before any first-start shared-world attach/create request leaves the shell.
2. The first shared-world request uses the persisted `orchestration_session_id`.
3. Pre-live startup drift handling no longer depends on `resolve_active_orchestration_session_id()`.
4. `--no-world` still preserves host-only startup.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-a2-foundation.ok`

Blocked-path output:

- `.runs/plan-04/blocked.json` if a second authority store, `PLAN-03` semantic drift, or `--no-world` regression is required

### Parallel window B

This is the first real worker window. It opens only after `task/m04-a2-foundation` is accepted.

### task/m04-b1-record-authority

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m04-record-authority`
- `codex/feat-thread-world-binding-m04-record-authority`

Allowed files:

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Scope:

1. Add explicit parent-session mutators:
   - `set_world_binding(...)`
   - `clear_world_binding()`
2. Keep parent-session persistence as the binding write and read authority.
3. Keep participant and lease persistence outside the binding barrier.

Must not do:

1. No host participant world fields.
2. No lease-sidecar expansion.
3. No slice `06` registry-shape work.

Acceptance:

1. The parent can call a single explicit parent-binding mutator path from `async_repl.rs`.
2. The worker output does not broaden state ownership.
3. The worker returns the legal clear points for slice `04`.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-b1-record-authority.ok` after parent review

Blocked-path output:

- no sentinel
- session-log rejection entry if the patch couples binding truth to participant or lease writes

### task/m04-b2-toolbox-proof

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m04-toolbox-proof`
- `codex/feat-thread-world-binding-m04-toolbox-proof`

Allowed files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Scope:

1. Extend `substrate agent toolbox status --json` with optional:

```json
"active_world_binding": {
  "world_id": "wld_active_0002",
  "world_generation": 7
}
```

2. Read the proof from `resolve_live_orchestrator_session(...)`.
3. Keep omission non-fatal when live authoritative resolution is unavailable.
4. Preserve the selected host row contract in `agent status --json`.

Must not do:

1. No selected-row schema broadening.
2. No new transport semantics.
3. No exposure of `orchestration_session_id` in the proof object.

Acceptance:

1. JSON shape is locked for tests before merge.
2. The new proof field is cleanly optional.
3. No selected-row behavior changes are introduced.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-b2-toolbox-proof.ok` after parent review

Blocked-path output:

- no sentinel
- session-log rejection entry if proof requires public correlation-key exposure or hidden `agent status` changes

### Parent validation gate B

Required before `task/m04-c1-runtime-integration` starts:

1. `task/m04-b1-record-authority` and `task/m04-b2-toolbox-proof` are both parent-reviewed.
2. Neither worker output changes the authority surface.
3. Neither worker output changes the selected-row contract.
4. `run-state.json` records which worker outputs were accepted.

### task/m04-c1-runtime-integration

Ownership:

- parent only

Primary files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs) if needed for request-path alignment
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) only for final proof-field integration

Scope:

1. Integrate `task/m04-b1-record-authority`.
2. Add the parent-binding persistence barrier in `async_repl.rs`.
3. Reorder startup runtime events so `registered` and `task_start` emit only after binding persistence succeeds when a world session exists.
4. Reorder drift alerts:
   - `world_restarted` only after replacement binding persists
   - `world_restart_required` only after current authoritative binding re-persists
5. Make bootstrap failure after world attach close the world, mark the parent terminal, and clear binding only after close succeeds.
6. Stamp host and orchestrator runtime events from the persisted parent snapshot, not transient pre-persist state.
7. Integrate `task/m04-b2-toolbox-proof` only after runtime truth sourcing is stable.

Acceptance:

1. No alert or host/orchestrator runtime event can claim a world binding not already persisted in the parent record.
2. Bootstrap failure after world attach cannot leave a live orphaned world tied to a never-live runtime.
3. `resolve_active_orchestration_session_id()` is no longer the deciding source for pre-live startup or restart paths.
4. Selected host status rows still omit world fields.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-c1-runtime-integration.ok`

Blocked-path output:

- `.runs/plan-04/blocked.json` if runtime truth sourcing remains ambiguous after integration

### Parent validation gate C

This gate must pass before any test worktree is launched.

Checks:

1. Foundational runtime integration is complete in the parent checkout.
2. No further structural API churn is expected for worker test lanes.
3. `async_repl.rs` is no longer under active structural edit.

Green-path output:

- session log entry: `gate-c accepted; launch test worktrees`

Blocked-path output:

- no test worktrees launched

### task/m04-d1-world-tests

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m04-world-tests`
- `codex/feat-thread-world-binding-m04-world-tests`

Allowed files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Scope:

1. Prove first-start shared-world attach/create is owner-bound.
2. Prove startup drift before first command retains the persisted startup context.
3. Prove parent binding persistence happens before `world_restarted`.
4. Prove fail-closed drift re-persists the current authoritative binding before `world_restart_required`.
5. Prove bootstrap failure after attach cleans up the world and parent-session state.

Must not do:

1. No production-code edits unless the parent explicitly reopens the task as a red/green repair.
2. No contract broadening beyond slice `04`.

Acceptance:

1. Integration tests in the real REPL shell test file fail if persist-before-publish regresses.
2. Worker provides exact tests added and commands run.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-d1-world-tests.ok` after parent review

Blocked-path output:

- no sentinel
- escalation back to parent if runtime behavior, not just tests, must change

### task/m04-d2-status-tests-docs

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m04-status-tests-docs`
- `codex/feat-thread-world-binding-m04-status-tests-docs`

Allowed files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

Read-for-validation only:

- [llm-last-mile/05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)

Scope:

1. Add contract coverage for:
   - `toolbox status --json` `active_world_binding`
   - proof omission remaining non-fatal
   - selected host status rows remaining unchanged
2. Update `llm-last-mile/README.md` so slice `04` is documented as the runtime-state bridge into slice `05` and slice `06`.
3. Flag downstream contract drift against `05-restart-invalidation-semantics.md`.

Must not do:

1. No silent rewrite of `05-restart-invalidation-semantics.md`.
2. No selected-status contract rewrite.

Acceptance:

1. Contract tests prove the new proof field while preserving selected-row behavior.
2. Docs reflect the bridge role of slice `04` without reordering the packet.

Green-path output:

- `.runs/plan-04/sentinels/task-m04-d2-status-tests-docs.ok` after parent review

Blocked-path output:

- no sentinel
- escalation if code and downstream packet assumptions no longer agree

## Integration Procedure

This is a parent-agent procedure, not general merge advice.

### Integration order

1. Complete `task/m04-a1-preflight`.
2. Complete `task/m04-a2-foundation`.
3. Launch at most two worker lanes:
   - `task/m04-b1-record-authority`
   - `task/m04-b2-toolbox-proof`
4. Review both worker outputs and pass Parent validation gate B.
5. Perform `task/m04-c1-runtime-integration` in the parent checkout.
6. Run parent post-integration validation before launching any test worktrees.
7. Launch test worktrees only after gate C passes:
   - `task/m04-d1-world-tests`
   - `task/m04-d2-status-tests-docs`
8. Integrate test and docs outputs.
9. Run final validation stack.
10. Complete `task/m04-e1-closeout`.

### Parent post-merge validation ordering

After `task/m04-c1-runtime-integration`, run in this order from the parent checkout:

```bash
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Only if these pass should the parent launch the Phase D test worktrees. This prevents dispatching workers against unstable runtime integration.

After integrating `task/m04-d1-world-tests` and `task/m04-d2-status-tests-docs`, run in this order:

```bash
cargo test -p substrate-shell -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Integration refusal rule:

- If two workstreams produce outputs that require different interpretations of `PLAN-04.md`, neither merges until the parent resolves the contract mismatch explicitly and records it in `.runs/plan-04/session.log`.

## Context-Control Rules

1. The parent keeps only the invariant set, merge state, gates, and open blockers in working memory.
2. Worker prompts must include the exact task ID, allowed files, forbidden files, acceptance criteria, and expected sentinel name.
3. Workers return:
   - touched files,
   - concise rationale,
   - tests run,
   - blockers or uncertainty.
4. Workers do not paste large file dumps back to the parent.
5. The parent re-reads [PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-04.md) before merging any lane that affects runtime ordering or status surfaces.
6. The parent re-reads [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md) before closing the slice to confirm that `PLAN-05` can still consume the parent-session binding model unchanged.
7. `async_repl.rs` is single-writer. Workers may read it for context, but only the parent changes it unless the parent explicitly delegates a bounded repair after main integration.
8. Test workers do not “fix” production behavior by changing assertions to match drift.
9. Parent updates `run-state.json` and `session.log` at every gate transition or blocked termination.

## Tests And Acceptance

### Required targeted commands

These commands map to the real files and contracts in this repo.

1. Runtime and bootstrap sequencing in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs):

```bash
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
```

2. World-start, startup-drift, and fail-closed validation in [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs):

```bash
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
```

3. Status and toolbox successor contract validation in [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs):

```bash
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
```

4. Full shell crate regression:

```bash
cargo test -p substrate-shell -- --nocapture
```

5. Workspace formatting and lint gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

6. Final workspace regression:

```bash
cargo test --workspace -- --nocapture
```

### Acceptance matrix

| Gate | Required proof | Primary files |
| --- | --- | --- |
| Startup authority gate | first attach/create request is owner-bound and preceded by a persisted pending parent session record | `async_repl.rs`, `repl_persistent_session.rs`, `repl_world_first_routing_v1.rs` |
| Binding authority gate | parent session record persists active `world_id/world_generation`; host participant stays world-empty | `orchestration_session.rs`, `state_store.rs`, `async_repl.rs` |
| Publish ordering gate | alerts and host runtime events publish only after parent binding persistence succeeds | `async_repl.rs`, `repl_world_first_routing_v1.rs` |
| Bootstrap cleanup gate | post-attach bootstrap failure closes world, marks parent terminal, clears binding at the right point | `async_repl.rs`, `repl_world_first_routing_v1.rs` |
| Proof surface gate | `toolbox status --json` shows optional `active_world_binding` without changing selected host status rows | `agents_cmd.rs`, `agent_successor_contract_ahcsitc0.rs` |
| Explicit bypass gate | `--no-world` path does not require world-binding persistence and keeps host-only ordering | `async_repl.rs`, targeted runtime tests |
| Contract preservation gate | `agent status --json` selected host row remains unchanged | `agents_cmd.rs`, `agent_successor_contract_ahcsitc0.rs` |

### Runtime validation rules

1. The parent does not accept “code looks right” as evidence for ordering-sensitive paths.
2. Any regression involving `world_restarted`, `world_restart_required`, or startup host-runtime events must be backed by test assertions in the real shell test files.
3. If targeted tests pass but full `substrate-shell` fails on related runtime-state surfaces, the slice stays open.
4. If `clippy` or workspace tests fail for reasons caused by this slice, the slice stays open.

### Final acceptance checklist

1. Pending parent session is persisted before first shared-world attach/create.
2. First shared-world attach/create is owner-bound.
3. Parent session record is the live world-binding authority.
4. Alerts and host runtime events publish only after binding persistence succeeds.
5. Host participant manifests remain world-empty.
6. `toolbox status --json` gets the optional proof field.
7. `agent status --json` selected host row remains unchanged.
8. Lease-sidecar expansion remains deferred.
9. `--no-world` remains an explicit bypass case.
10. `PLAN-05` prerequisites in [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md) still hold after merge.

## Run Exit Criteria

This defines DONE for the orchestration run itself, not just for the code slice.

### Successful run

The run is complete only when all of these exist or are true:

1. Updated production code across the intended backend surfaces:
   - `async_repl.rs`
   - `orchestration_session.rs`
   - `state_store.rs`
   - `agents_cmd.rs`
   - `repl_persistent_session.rs` only if needed by the final implementation
2. Updated tests in:
   - `repl_world_first_routing_v1.rs`
   - `agent_successor_contract_ahcsitc0.rs`
3. Updated packet documentation in:
   - `llm-last-mile/README.md`
4. Parent validation commands have passed in the defined order.
5. `.runs/plan-04/closeout.md` exists and summarizes:
   - accepted task IDs,
   - integrated worktrees and branches,
   - final validation commands and outcomes,
   - intentional deferrals to `PLAN-05` or slice `06`.
6. `.runs/plan-04/sentinels/task-m04-e1-closeout.ok` exists.
7. `.runs/plan-04/run-state.json` marks the run `completed`.

### Blocked termination

If the run cannot proceed without violating a hard guard:

1. parent writes `.runs/plan-04/blocked.json`,
2. parent records the blocking task ID, invariant, and unresolved conflict in `.runs/plan-04/session.log`,
3. parent marks `run-state.json` as `blocked`,
4. parent does not write `task-m04-e1-closeout.ok`.

### task/m04-e1-closeout

Ownership:

- parent only

Scope:

1. Verify all accepted task sentinels are present.
2. Verify final validation commands passed.
3. Confirm no blocked artifact remains active for the final run state.
4. Write `.runs/plan-04/closeout.md`.
5. Mark `run-state.json` as `completed`.

Acceptance:

1. The code slice is complete by the checklist above.
2. The orchestration artifacts clearly show green termination rather than a partial stop.
3. The closeout explains what was deliberately deferred.

Green-path output:

- `.runs/plan-04/closeout.md`
- `.runs/plan-04/sentinels/task-m04-e1-closeout.ok`

Blocked-path output:

- `.runs/plan-04/blocked.json`

## Assumptions

1. `PLAN-03` owner-binding and shared-world generation semantics are already landed and are not being redesigned in this slice.
2. The current implementation branch for this work is `feat/thread-world-binding`, and subagent branches layer on top of it rather than forking a new product direction.
3. This slice remains backend-only. No UI or UX redesign work is required.
4. The live proof surface for this slice is `substrate agent toolbox status --json`; TCP transport remains unsupported and unchanged.
5. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) is updated only to reflect the runtime-state bridge role of slice `04`, not to reorder the packet.
6. [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md) is a downstream contract consumer for validation. If it conflicts with the finished slice, the correct action is to stop and reconcile the packet, not to silently stretch slice `04`.
