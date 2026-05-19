# ORCH_PLAN-09: Live-State Authority and Compatibility Cutover

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md)  
Execution type: shell/runtime authority-cutover orchestration plan, no UI scope, strong DX/doc contract scope

## Summary

This run executes `PLAN-09` on the current branch `feat/session-centric-state-store` with an exact active worker cap of `2`. The parent remains the only integrator, the only final branch writer, and the only agent allowed to mutate the parent-owned production seam in [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) and [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs). The canonical run-state source of truth is `.runs/plan-09/run-state.json`.

This slice is not an event-emission cleanup, a host-runtime lifecycle redesign, or a bridge-removal run. It is a contract-freeze slice around live-state authority and compatibility cutover. The only honest concurrency in this repo is:

1. parent-only preflight and store authority freeze first,
2. one worker lane for operator-surface tightening and contract regressions second,
3. one worker lane for active-doc authority wording second, in parallel with the operator lane,
4. parent-only integration and final validation last.

The active worker cap is exactly `2` because only two child worktrees are needed for the only real parallel window in this slice. The parent checkout itself carries all parent-only tasks. That means the practical execution surface is exactly three checkouts:

- parent checkout on `feat/session-centric-state-store` for `task/m09-a1`, `task/m09-a2`, `task/m09-e1`, and `task/m09-e2`
- `../substrate-m09-operator-contracts` on `codex/feat-session-centric-state-store-m09-operator-contracts`
- `../substrate-m09-authority-docs` on `codex/feat-session-centric-state-store-m09-authority-docs`

The critical path stays parent-owned through these phases:

1. `task/m09-a1-preflight`
2. `task/m09-a2-store-authority-freeze`
3. `task/m09-e1-integrate-and-validate`
4. `task/m09-e2-closeout`

This is deliberate. `PLAN-09` is already truthful about the repo topology: the authority seam must freeze first, then operator tightening and docs can proceed in parallel, then final validation closes the run. `state_store.rs` owns the live-state authority ladder, torn-root posture, selected-participant linkage, and fail-closed session resolution. Splitting workers before that file is frozen would create merge churn and contract drift rather than throughput.

The parent-owned execution brief is:

1. freeze the live-state authority ladder and retained compatibility bridge in the store first,
2. prove the missing precedence and inactive-selected-participant regressions at the store seam second,
3. seed both worker worktrees from that exact frozen parent tree,
4. let one worker tighten operator-surface behavior and contract coverage without touching parent-owned production files,
5. let one worker tighten active docs and only audit secondary docs if contradiction remains,
6. integrate accepted worker outputs in the parent checkout,
7. perform one exact validation stack centered on the `PLAN-09` seam,
8. close the run only if the authority contract, contract tests, shell suite, and repo gates are all green.

## Hard Guards

### Locked invariants

1. Canonical session-root parent and participant records are the live-state authority boundary for operator surfaces.
2. Flat compatibility parent, participant, and lease files are bridge input/output only during cutover.
3. Legacy `handles/*.json` is last-resort compatibility input only and never outranks canonical or flat records.
4. Trace is historical fallback only for `status` gaps and never current-session toolbox authorization.
5. `substrate agent status` and `substrate agent toolbox ...` fail closed on ambiguity, corruption, broken parent/child linkage, missing selected participant, or inactive selected participant.
6. `persist_runtime_snapshots(...)` remains the caller choke point, but `PLAN-09` does not authorize worker-lane production edits in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
7. No caller outside [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) may directly write flat compatibility parent, participant, lease, or handle artifacts.
8. Dual-write ownership stays localized to store helpers in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).
9. Bridge removal is gated and explicitly not part of this slice.
10. `PLAN-06` session-centric authority semantics and `PLAN-08` trace/event authority semantics are upstream constraints, not reopenable scope.
11. No new selector UX, cache file, registry, or transactional persistence layer is allowed.
12. Active docs must describe the same authority ladder as runtime behavior.
13. Validation commands must use the real package name in this repo: `cargo ... -p shell`.

### File-level boundaries

Parent-owned serialized production surfaces:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Worker-safe post-freeze operator surface:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

Worker-safe post-freeze docs surface:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Conditional docs-only audit surface:

- [compatibility-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md)
- [manual_testing_playbook.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md)

Read-for-truth only:

- [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md)
- [ORCH_PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-08.md)
- [ORCH_PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-06.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/plan-09/blocked.json` if any of these occur:

1. A task requires concurrent edits to `state_store.rs` and worker-lane files before the authority freeze lands.
2. A task restores flat compatibility files, legacy handles, or trace rows as live-state authority.
3. A task weakens fail-closed behavior into heuristic newest-session or trace-first recovery.
4. A task removes flat compatibility reads or writes instead of merely freezing their bridge posture.
5. A task spreads flat compatibility write ownership outside the store.
6. A worker lane needs to touch [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) or [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) to complete its assignment.
7. A docs lane needs to touch test or production files to make its wording true.
8. A task introduces new selector UX, a new registry, or a new persistence layer.
9. A task leaves inactive selected-participant behavior ambiguous instead of fail-closed.

## Orchestration State Surfaces

### Canonical run state

Single local source of truth for the run:

- `.runs/plan-09/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- frozen live-state authority ladder,
- frozen fail-closed operator-surface contract,
- frozen retained-bridge and bridge-removal-gate posture,
- accepted and rejected worker outputs,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/plan-09/queue.json`
- `.runs/plan-09/session.log`
- `.runs/plan-09/sentinels/task-m09-a1-preflight.ok`
- `.runs/plan-09/sentinels/task-m09-a2-store-authority-freeze.ok`
- `.runs/plan-09/sentinels/task-m09-b1-operator-contracts.ok`
- `.runs/plan-09/sentinels/task-m09-c1-authority-docs.ok`
- `.runs/plan-09/sentinels/task-m09-e1-integrate-and-validate.ok`
- `.runs/plan-09/sentinels/task-m09-e2-closeout.ok`
- `.runs/plan-09/blocked.json`
- `.runs/plan-09/closeout.md`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.
5. Worker-generated notes never replace parent-written sentinels or run-state artifacts.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact active worker cap: `2`.
4. There are zero worker lanes during `task/m09-a1-preflight` and `task/m09-a2-store-authority-freeze`.
5. Only two child worktrees exist in this run because only two worker-safe late-phase lanes exist in this slice.
6. No worker may edit [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) or [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
7. Worker lanes open only after the parent has stabilized the store authority diff and seeded each worktree from that exact state.
8. `task/m09-b1-operator-contracts` and `task/m09-c1-authority-docs` are the only parallel window in this run.
9. If `task/m09-b1` proves that the write-ownership drift guard requires production changes in `async_repl.rs`, the worker stops and hands the issue back to the parent instead of widening scope.
10. If `task/m09-c1` proves that a doc contradiction requires code or contract-test changes, the worker stops and hands the issue back to the parent instead of widening scope.

## Approval And Gate Model

There are no human approval gates defined for this run.

Replacement control mechanism:

1. parent validation gates,
2. parent-written sentinels,
3. `session.log` for acceptance and rejection rationale,
4. `blocked.json` for hard-stop termination,
5. `closeout.md` for successful completion.

### Gate A: Scope And Topology Lock

Required before implementation starts:

- parent re-reads `PLAN-09`, `ORCH_PLAN-08`, and `ORCH_PLAN-06`,
- parent records the honest run shape: authority freeze first, operator/docs parallel second, final validation last,
- parent records the exact parent-owned files and the exact worker-safe files,
- parent records the package-name normalization: use `shell`, never `substrate-shell`.

### Gate B: Authority Freeze

Required before the worker window opens:

- the store-owned authority ladder is frozen in the parent checkout,
- canonical-over-flat-over-legacy precedence is frozen,
- inactive selected-participant fail-closed behavior is frozen,
- retained-bridge and bridge-removal-gate posture is frozen,
- the parent has completed the pre-worker proof order defined below,
- both worker worktrees are seeded from the exact post-`task/m09-a2` tree.

### Gate C: Final Acceptance

Required before closeout:

- both worker outputs are accepted or deliberately rejected and replaced by parent work,
- the final validation order passes,
- active docs reflect the final authority ladder and toolbox fail-closed posture,
- no caller-owned flat compatibility writes exist outside the store.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-m09-operator-contracts`
  - `codex/feat-session-centric-state-store-m09-operator-contracts`
- `../substrate-m09-authority-docs`
  - `codex/feat-session-centric-state-store-m09-authority-docs`

Execution topology:

1. The parent checkout serves every parent-only lane in this run.
2. Exactly two child worktrees exist because exactly two worker-safe tasks exist after Gate B.
3. No third child worktree is authorized because there is no third honest parallel seam in `PLAN-09`.

Subagents do not merge each other’s work. They return patches, touched files, tests run, and blockers to the parent.

### Task graph

Execution graph for the run:

1. `task/m09-a1-preflight`
2. `task/m09-a2-store-authority-freeze`
3. `task/m09-b1-operator-contracts` and `task/m09-c1-authority-docs` in parallel
4. `task/m09-e1-integrate-and-validate`
5. `task/m09-e2-closeout`

Parent-only serialized tasks:

- `task/m09-a1-preflight`
- `task/m09-a2-store-authority-freeze`
- `task/m09-e1-integrate-and-validate`
- `task/m09-e2-closeout`

Worker-owned tasks:

- `task/m09-b1-operator-contracts`
- `task/m09-c1-authority-docs`

## Parallel Window BC

This is the only worker window in the run.

It opens only after Gate B passes and after the parent has completed the authority freeze in `task/m09-a2-store-authority-freeze`. There are zero honest production-code workers before that because [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) owns the authority ladder that both operator surfaces and docs must consume.

The parallel window is optimal rather than merely acceptable because the two late tasks are materially independent once the store contract is frozen:

- the operator lane owns [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) plus the contract suite,
- the docs lane owns active authority wording plus contradiction-only secondary-doc audit,
- neither worker needs the other’s files to make progress,
- both workers can validate against the same frozen parent tree without touching parent-owned production seams.

### task/m09-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md), [ORCH_PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-08.md), and [ORCH_PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-06.md).
2. Confirm the run executes from `feat/session-centric-state-store`.
3. Freeze the no-UI scope, exact `2`-worker cap, parent-only production boundaries, and retained-bridge posture into the run packet.
4. Initialize:
   - `.runs/plan-09/run-state.json`
   - `.runs/plan-09/queue.json`
   - `.runs/plan-09/session.log`
5. Record the repo-truth validation rule that all package-targeted commands use `-p shell`.

Acceptance:

1. The parent can explain why `state_store.rs` must freeze before any worker lane opens.
2. The parent can restate the live-state authority ladder without ambiguity.
3. The parent can name the required missing regression work for this slice:
   - legacy-handle precedence never outranking canonical or flat records,
   - inactive selected participant fail-closed coverage,
   - write-ownership drift guard proving no caller outside the store writes flat compatibility artifacts directly.
4. `run-state.json` records the initial phase and queue.

Green-path output:

- `.runs/plan-09/sentinels/task-m09-a1-preflight.ok`

Blocked-path output:

- `.runs/plan-09/blocked.json`

### Parent validation gate A

Required before `task/m09-a2-store-authority-freeze` starts:

1. No invariant contradiction remains unresolved.
2. The parent can explain why bridge removal is explicitly deferred.
3. The parent can explain why only two child worktrees are needed for this run.

### task/m09-a2-store-authority-freeze

Ownership:

- parent only

Why serialized:

- [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) owns the precedence contract, torn-root posture, live-session selection, selected-participant linkage, and last-resort legacy-handle fallback. There is no honest throughput gain from splitting downstream work before those rules are frozen.

Allowed files:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Scope:

1. Freeze the read order in `load_authoritative_session(...)` and adjacent participant/session readers:
   - canonical parent,
   - flat parent only if canonical parent is absent,
   - canonical participant,
   - flat participant only if canonical participant is absent,
   - legacy handle alias last.
2. Keep `list_live_sessions()` and `resolve_single_live_session_for_agent(...)` strict about:
   - active parent state,
   - owner PID liveness,
   - selected-participant presence,
   - selected-participant activity,
   - parent/child linkage.
3. Add or retain direct store-level regressions for:
   - canonical participant beating conflicting legacy-handle fallback,
   - flat participant beating conflicting legacy-handle fallback when the canonical child is absent,
   - selected participant present but inactive failing closed.
4. Keep dual-write ownership localized to the store helpers. Do not move any write ownership into callers.
5. Do not touch `agents_cmd.rs`, `async_repl.rs`, docs, or contract tests yet.

Must not do:

1. No operator-surface tightening yet.
2. No docs edits yet.
3. No bridge removal or dual-write retirement.
4. No new helper layer, cache, or registry to “clarify” the authority ladder.

Acceptance:

1. The parent can point to one frozen store-owned authority ladder for live discovery.
2. Store-level tests prove canonical and flat participant records outrank legacy handles.
3. Store-level tests prove an inactive selected participant fails closed.
4. No authority decision depends on trace or caller-owned precedence logic.

Green-path output:

- `.runs/plan-09/sentinels/task-m09-a2-store-authority-freeze.ok`

Blocked-path output:

- `.runs/plan-09/blocked.json`

### Parent validation gate B

Required before the worker window opens:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture` passes.
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run` passes.
3. `cargo test -p shell --tests --no-run` passes from the stabilized parent checkout.
4. The parent records the frozen authority ladder, fail-closed operator contract, retained bridge posture, and bridge-removal gates in `run-state.json`.
5. The parent seeds both worker worktrees from the exact post-`task/m09-a2` tree.

### task/m09-b1-operator-contracts

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m09-operator-contracts`
- `codex/feat-session-centric-state-store-m09-operator-contracts`

Allowed files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

Scope:

1. Keep `build_status_report(...)` live-first and trace-second:
   - live session projection first,
   - tombstone suppression next,
   - trace fallback only for still-uncovered status tuples.
2. Keep `build_toolbox_status_report(...)` and `build_toolbox_env_report(...)` resolving through `resolve_single_live_session_for_agent(...)` with no local precedence or newest-session heuristics.
3. Add CLI-surface regression coverage for an active parent whose selected participant exists but is inactive.
4. Add a bounded write-ownership drift guard in the contract suite proving that no production caller outside the store writes flat compatibility parent, participant, lease, or handle artifacts directly.
5. Preserve current toolbox JSON fields and exit-code posture, especially exit `3` for `dependency_unavailable`.

Must not do:

1. No edits to [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).
2. No edits to [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
3. No docs edits.
4. No bridge removal or flat dual-write retirement.
5. No heuristic “latest session” or trace-only live-session recovery.
6. No new persistence owner outside the store.

Commands:

1. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`

Execution packet for the worker:

1. Read [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md).
2. Read [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs).
3. Read [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs).
4. Make only allowed-file changes.
5. Run the exact command listed above.
6. Stop immediately if the drift guard requires production edits in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).

Acceptance:

1. Contract coverage proves inactive selected-participant resolution fails closed across operator surfaces.
2. Contract coverage proves no caller outside the store owns direct flat compatibility writes.
3. No patch reintroduces trace-authorized toolbox state or direct caller-owned compatibility writes.
4. The worker report includes touched files, exact command run, and whether `async_repl.rs` follow-up is needed from the parent.
5. The lane stays inside its allowed files.

Green-path output:

- `.runs/plan-09/sentinels/task-m09-b1-operator-contracts.ok`

Blocked-path output:

- `.runs/plan-09/blocked.json` if the worker needs parent-owned production files

### task/m09-c1-authority-docs

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m09-authority-docs`
- `codex/feat-session-centric-state-store-m09-authority-docs`

Allowed files:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Conditional allowed files only if active docs would otherwise remain contradictory:

- [compatibility-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md)
- [manual_testing_playbook.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md)

Scope:

1. Keep one clear authority statement: canonical session-root parent plus participant records are the live-state authority boundary.
2. Keep one clear bridge statement: flat compatibility files are bridge input/output only during cutover.
3. Keep one clear legacy statement: `handles/*.json` remains last-resort compatibility input only.
4. Keep one clear trace statement: trace is historical fallback for `status` gaps only and never toolbox authorization.
5. Keep one clear toolbox statement: `substrate agent toolbox env` emits variables only for a current live host-scoped orchestrator session and otherwise fails closed with exit `3`.
6. Touch secondary docs only if they contradict the active runtime contract, and then only change the contradictory sentence or bullet.

Must not do:

1. No edits to code or tests.
2. No bridge-removal edits disguised as doc cleanup.
3. No assertion changes in contract suites from this lane.

Commands:

1. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`

Execution packet for the worker:

1. Read [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md).
2. Read [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md).
3. Read [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md).
4. Audit the two secondary docs only if active-doc contradictions remain after the primary edits.
5. Make only allowed-file changes.
6. Run the exact command listed above.
7. Stop immediately if any required fix would need code or contract-test edits.

Acceptance:

1. `TRACE.md` and `USAGE.md` describe the same authority ladder as the runtime.
2. No active doc implies that flat compatibility files, legacy handles, or trace history are current truth.
3. Any secondary-doc edits are minimal and contradiction-driven.
4. The worker report includes touched files, exact command run, and whether any unresolved contradiction remains outside docs.
5. The lane stays inside its allowed files.

Green-path output:

- `.runs/plan-09/sentinels/task-m09-c1-authority-docs.ok`

Blocked-path output:

- `.runs/plan-09/blocked.json` if the worker needs code or test edits

### task/m09-e1-integrate-and-validate

Ownership:

- parent only

Scope:

1. Review both worker outputs against the frozen authority and fail-closed contracts.
2. Reject any worker patch that touches unowned files, assumes stale authority, or omits test evidence.
3. Integrate accepted worker outputs into the parent checkout.
4. If the operator worker proves that the write-ownership drift guard requires bounded parent-owned production changes in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), make those changes in the parent checkout and rerun the affected validation steps.
5. Run the final validation stack in the exact order defined below.

Acceptance:

1. Every accepted worker output has a matching parent-written sentinel.
2. No flat compatibility or legacy-handle source outranks canonical session-root authority.
3. No operator surface authorizes current liveness from trace.
4. No direct caller-owned flat compatibility writes exist outside the store.
5. `run-state.json` records accepted and rejected outputs explicitly.

Green-path output:

- `.runs/plan-09/sentinels/task-m09-e1-integrate-and-validate.ok`

Blocked-path output:

- `.runs/plan-09/blocked.json`

### task/m09-e2-closeout

Ownership:

- parent only

Scope:

1. Mark the run complete in `.runs/plan-09/run-state.json`.
2. Write `.runs/plan-09/closeout.md` with:
   - accepted tasks,
   - integrated worktrees and branches,
   - validation commands and outcomes,
   - retained bridge behavior and explicit removal gates,
   - confirmation that live-state authority remains canonical session-root parent plus participant records,
   - confirmation that no caller outside the store writes flat compatibility artifacts directly.
3. Append the final acceptance rationale to `.runs/plan-09/session.log`.

Acceptance:

1. `closeout.md` is present and matches the final branch state.
2. All sentinels through `task/m09-e2-closeout` exist.
3. No blocked-path artifact exists for a successful run.

Green-path output:

- `.runs/plan-09/sentinels/task-m09-e2-closeout.ok`

## Context-Control Rules

1. The parent owns `.runs/plan-09/*`. Workers do not edit run-state, sentinels, queue, session log, blocked state, or closeout artifacts.
2. Worker packets include only:
   - the task ID,
   - allowed files,
   - frozen invariants,
   - stop conditions,
   - exact commands to run.
3. Workers must stop immediately if they need to touch any file outside their allowed list.
4. Workers report:
   - touched files,
   - commands run,
   - unresolved assumptions,
   - blockers,
   - whether they observed any stale authority wording, trace-authority assumptions, or write-ownership drift that requires parent-owned follow-up.
5. Workers do not rebase, merge, or integrate each other’s work.
6. The parent rejects any worker patch that silently broadens scope or makes authority decisions not already frozen in `run-state.json`.
7. If worker outputs conflict with current parent truth, the parent re-derives the correct result from production code and rewrites the patch locally instead of negotiating a blended state across worktrees.

## Tests And Acceptance

### Pre-worker proof order

Run these parent-owned checks before dispatching the worker window:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture`
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
3. `cargo test -p shell --tests --no-run`

### Final validation order

Run these commands in this exact order during `task/m09-e1-integrate-and-validate`:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture`
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
3. `cargo test -p shell -- --nocapture`
4. `cargo fmt --all -- --check`
5. `cargo clippy -p shell --all-targets -- -D warnings`
6. `cargo clippy --workspace --all-targets -- -D warnings`
7. `cargo test --workspace -- --nocapture`

### Acceptance checklist

The slice is accepted only if all of these are true:

1. Canonical session-root parent and participant records are the implemented and documented live-state authority.
2. Flat compatibility parent, participant, and lease files remain bridge input/output only during cutover.
3. Legacy `handles/*.json` remains documented and tested as last-resort compatibility input only.
4. Canonical or flat participant records never lose precedence to legacy-handle fallback.
5. `status` uses live state first and trace only as bounded historical fill.
6. `toolbox status|env` never authorize current liveness from trace and fail closed on ambiguity or broken linkage.
7. An inactive selected participant fails closed at both store and operator-surface levels.
8. No caller-owned flat compatibility write path exists outside the store.
9. Active docs reflect the current authority ladder and retained bridge posture.
10. Bridge removal remains deferred behind explicit later gates.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch assumes flat compatibility files, legacy handles, or trace rows are authoritative for live discovery.
3. The patch weakens fail-closed behavior for ambiguity, broken linkage, or inactive selected participants.
4. The patch removes bridge behavior or dual-write without a separate gated slice.
5. The patch requires concurrent parent edits to `state_store.rs` or `async_repl.rs` to become intelligible.
6. The patch omits test evidence for the behavior it claims to cover.
7. The patch leaves docs or fixtures contradicting the final runtime contract.
8. The patch introduces or preserves direct flat compatibility writes outside the store.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/plan-09/sentinels/task-m09-a1-preflight.ok` exists.
2. `.runs/plan-09/sentinels/task-m09-a2-store-authority-freeze.ok` exists.
3. `.runs/plan-09/sentinels/task-m09-b1-operator-contracts.ok` exists.
4. `.runs/plan-09/sentinels/task-m09-c1-authority-docs.ok` exists.
5. `.runs/plan-09/sentinels/task-m09-e1-integrate-and-validate.ok` exists.
6. `.runs/plan-09/sentinels/task-m09-e2-closeout.ok` exists.
7. `.runs/plan-09/run-state.json` exists and records a completed terminal state.
8. `.runs/plan-09/queue.json` and `.runs/plan-09/session.log` exist.
9. `.runs/plan-09/closeout.md` exists and matches the final accepted branch state.
10. `.runs/plan-09/blocked.json` does not exist.

### Blocked termination

The run terminates blocked only if all of these are true:

1. `.runs/plan-09/blocked.json` exists and records the blocking reason.
2. `.runs/plan-09/run-state.json` exists and records `blocked` as the terminal state.
3. `session.log` contains the parent rationale for the stop.
4. No downstream task sentinel may be written after the blocking point.
5. `.runs/plan-09/sentinels/task-m09-e2-closeout.ok` must not exist.
6. `.runs/plan-09/closeout.md` must not exist.

## Closeout

Successful closeout records:

1. the final frozen live-state authority ladder that shipped,
2. the final fail-closed operator-surface contract,
3. the exact validation order and outcomes,
4. confirmation that flat compatibility files remain bridge input/output only,
5. confirmation that legacy handles remain last-resort compatibility input only,
6. confirmation that no caller outside the store writes flat compatibility artifacts directly,
7. confirmation that active docs reflect the final authority boundary.

## Assumptions

1. The parent run starts from the current branch baseline `feat/session-centric-state-store`.
2. `PLAN-06` and `PLAN-08` already define upstream runtime and trace constraints that this slice must preserve.
3. The Rust package name `shell` is authoritative for validation commands in this repository.
4. The bridge around flat compatibility files remains intentionally present during this slice.
5. Any secondary doc drift can be corrected with minimal docs-only edits and does not require reopening runtime authority design.
