# ORCH_PLAN-10: Production World-Scoped Member Runtime Launch

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md)  
Reference style source: [ORCH_PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-09.md)  
Execution type: shell/runtime launch orchestration plan, no UI scope, strong DX/runtime/status/trace scope

## Summary

This run executes `PLAN-10` on `feat/session-centric-state-store` with an exact active worker cap of
`2`. The parent remains the only integrator, the only final branch writer, and the only agent
allowed to mutate the production ownership seam in
[crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs),
[crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs),
and
[crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
The canonical run-state source of truth is `.runs/plan-10/run-state.json`.

This slice is not a scheduler, not a multi-member selector product, not a new persistence layer,
and not a docs-first cleanup run. It is one bounded production seam:

1. freeze the shared world-member selection contract in `validator.rs`,
2. align `agent doctor` to that same contract in `agents_cmd.rs`,
3. land lazy world-backed member launch in the REPL command branch,
4. land restart replacement on that same retained-control seam,
5. close the runtime/status/trace test wall,
6. touch docs only if the landed runtime behavior actually changed wording.

The honest concurrency shape is narrower than the abstract lane count in `PLAN-10`.
`PLAN-10` is explicit about implementation order: validator first, doctor alignment as required
production work, lazy launch next, restart replacement on that same seam, then the test wall, then
optional docs. That means the parent owns all production-code steps serially through restart
replacement. The only real worker window opens after that seam is frozen:

1. parent-only preflight,
2. parent-only validator contract,
3. parent-only doctor alignment,
4. parent-only lazy launch seam,
5. parent-only restart replacement,
6. one worker lane for operator/trace contract tests,
7. one worker lane for world-routing/restart integration tests,
8. parent-only integration, final validation, optional docs review, and closeout.

This is deliberate. `validator.rs`, `agents_cmd.rs`, and `async_repl.rs` together define one
selection-and-launch contract. Parallelizing those production edits would create selection drift,
error-posture drift, and merge churn around the same retained-control seam. After the runtime seam
lands, the test wall becomes the only honest place to split work without forking production truth.

Worker-model execution policy for this run:

1. The parent remains the only integrator and the only final branch writer.
2. Every child worker uses `GPT-5.4` with `reasoning_effort=high`.
3. The active worker cap remains exactly `2`.
4. Worker coordination uses sentinels and long waits. Tight polling loops are not allowed.

## Hard Guards

### Locked invariants

1. V1 member selection means: zero eligible world members preserves host-only behavior, exactly one
   eligible world member is required for launch, and more than one eligible world member fails
   closed.
2. Doctor and runtime launch must share the same selection truth. No boolean "any world member
   exists" shortcut is allowed once `PLAN-10` lands.
3. The first production caller stays in the world-backed command branch in
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   after `ensure_no_policy_drift(...)` and before `exec_world_pty(...)` or `exec_world_line(...)`.
4. A new member participant is persisted in `allocating` first and does not become
   authoritative-live until retained UAA control, active event stream, and completion observer are
   all present.
5. Restart replacement must reuse the same retained-control lifecycle seam and
   `new_replacement_participant(...)`. No second lifecycle definition is allowed.
6. `substrate agent status --json` must surface the live member from runtime state; stale trace
   rows must never resurrect current liveness.
7. `substrate agent toolbox status|env` remains orchestrator-scoped for this slice even when a
   world member is live.
8. The store remains the only persistence owner. No new direct caller-owned canonical,
   compatibility, lease, or handle writes are allowed.
9. `registry.rs`, `state_store.rs`, and `world_gateway.rs` remain default no-change surfaces. If a
   real launch gap forces edits there, the parent owns them and must record the reason in
   `run-state.json` before touching the file.
10. No selector UX, member scheduler, gateway cache, runtime manager, or public `/v1/agents`
    surface is allowed in this run.
11. The test wall must be green before any docs edits are considered.
12. Package-targeted cargo commands use the real package name in this repo: `-p shell`.
13. The parent is the only writer of `.runs/plan-10/*`.

### File-level boundaries

Parent-owned serialized production surfaces:

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Parent-owned escalation-only production surfaces:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)

Worker-safe test surface after the runtime seam is frozen:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Optional parent-owned docs surface only after the test wall is green:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Read-for-truth only:

- [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md)
- [ORCH_PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-09.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/plan-10/blocked.json` if any of these occur:

1. A task introduces a second member-selection rule or preserves a doctor/runtime mismatch.
2. A task guesses among multiple world-member candidates instead of failing closed.
3. A task falls back to host execution after a hard member-launch preflight failure other than the
   explicit zero-eligible-member no-op case.
4. A task advertises a member live before retained UAA control and the full liveness boundary are
   proven.
5. A task reopens store ownership by writing canonical or compatibility state directly from the
   REPL or doctor path.
6. A worker lane needs to touch `validator.rs`, `agents_cmd.rs`, `async_repl.rs`, `state_store.rs`,
   `registry.rs`, or `world_gateway.rs` to complete its assignment.
7. A task lets stale generation state regain liveness through trace fallback after restart.
8. A task requires docs edits to explain behavior that the runtime and tests still do not prove.
9. A task broadens scope into scheduler, selector UX, or multi-member control-plane work.

## Orchestration State Surfaces

### Canonical run state

The only canonical source of truth for run orchestration state:

- `.runs/plan-10/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- frozen selection contract,
- frozen doctor/runtime alignment contract,
- frozen lazy-launch and restart-replacement contract,
- accepted and rejected worker outputs,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent treats `run-state.json` as
authoritative and ignores the worker report until the parent explicitly reconciles the discrepancy.

### Derived run artifacts

The parent maintains these local artifacts:

- `.runs/plan-10/queue.json`
- `.runs/plan-10/session.log`
- `.runs/plan-10/sentinels/task-m10-a1-preflight.ok`
- `.runs/plan-10/sentinels/task-m10-a2-validator-contract.ok`
- `.runs/plan-10/sentinels/task-m10-a3-doctor-alignment.ok`
- `.runs/plan-10/sentinels/task-m10-a4-member-launch-seam.ok`
- `.runs/plan-10/sentinels/task-m10-a5-restart-replacement.ok`
- `.runs/plan-10/sentinels/task-m10-b1-runtime-contract-tests.ok`
- `.runs/plan-10/sentinels/task-m10-b2-world-routing-restart-tests.ok`
- `.runs/plan-10/sentinels/task-m10-e1-integrate-and-validate.ok`
- `.runs/plan-10/sentinels/task-m10-e2-docs-review.ok`
- `.runs/plan-10/sentinels/task-m10-e3-closeout.ok`
- `.runs/plan-10/blocked.json`
- `.runs/plan-10/closeout.md`

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
4. There are zero worker lanes during `task/m10-a1` through `task/m10-a5`.
5. The only honest parallel window is the test wall after the shared launch seam is frozen.
6. No worker may edit `validator.rs`, `agents_cmd.rs`, `async_repl.rs`, `state_store.rs`,
   `registry.rs`, `world_gateway.rs`, or docs.
7. The parent seeds both child worktrees from the exact post-`task/m10-a5` tree.
8. `task/m10-b1` and `task/m10-b2` are the only parallel tasks in this run.
9. Optional docs stay parent-owned and sequential because they depend on the final accepted
   runtime behavior rather than generating independent throughput.
10. If a worker proves a missing production change is required, the worker stops and hands the
    issue back to the parent instead of widening scope.
11. Worker coordination uses sentinel files and long waits. Tight polling against run-state or
    branch state is forbidden.

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

- parent re-reads `PLAN-10.md` and `ORCH_PLAN-09.md`,
- parent records the exact serialized production order: validator, doctor, lazy launch, restart
  replacement, then the test wall,
- parent records the exact `2`-worker cap and why the worker window opens only after
  `task/m10-a5`,
- parent records parent-owned and worker-safe files,
- parent records the package-name normalization: use `shell`, never `substrate-shell`.

### Gate B: Shared Selection Freeze

Required before doctor alignment proceeds:

- `validate_member_selection(...)` is frozen in `validator.rs`,
- zero vs one vs many eligible world-member outcomes are explicit and tested,
- helper wording is neutral enough for both doctor and runtime call sites,
- no REPL or doctor behavior change ships through a private selection rule.

### Gate C: Runtime Seam Freeze

Required before the worker window opens:

- doctor and runtime share one frozen selection contract,
- the REPL world-backed command branch owns lazy launch before `exec_world_pty(...)` /
  `exec_world_line(...)`,
- restart replacement is frozen on that same retained-control seam,
- any escalation into `state_store.rs` or `registry.rs` is already integrated and justified,
- the parent has completed the pre-worker proof order defined below,
- both worker worktrees are seeded from the exact post-`task/m10-a5` tree.

### Gate D: Final Acceptance

Required before closeout:

- both worker outputs are accepted or explicitly rejected and replaced by parent work,
- the final validation order passes,
- docs review is complete and either produced minimal aligned edits or an explicit no-change
  decision,
- `run-state.json` records the final selection, doctor, launch, restart, and test contracts.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-m10-runtime-contract-tests`
  - `codex/feat-session-centric-state-store-m10-runtime-contract-tests`
- `../substrate-m10-world-routing-restart`
  - `codex/feat-session-centric-state-store-m10-world-routing-restart`

Suggested creation commands:

```bash
git worktree add ../substrate-m10-runtime-contract-tests \
  -b codex/feat-session-centric-state-store-m10-runtime-contract-tests \
  feat/session-centric-state-store
git worktree add ../substrate-m10-world-routing-restart \
  -b codex/feat-session-centric-state-store-m10-world-routing-restart \
  feat/session-centric-state-store
```

Execution topology:

1. The parent checkout serves every production task and every final integration task.
2. Exactly two child worktrees exist because exactly two worker-safe late test tasks exist in this
   slice.
3. No third child worktree is authorized because no earlier production seam can be parallelized
   honestly without forking selection or retained-control truth.

Subagents do not merge each other’s work. They return only changed files, exact commands run, and
blockers to the parent.

### Task graph

Execution graph for the run:

1. `task/m10-a1-preflight`
2. `task/m10-a2-validator-contract`
3. `task/m10-a3-doctor-alignment`
4. `task/m10-a4-member-launch-seam`
5. `task/m10-a5-restart-replacement`
6. `task/m10-b1-runtime-contract-tests` and `task/m10-b2-world-routing-restart-tests` in parallel
7. `task/m10-e1-integrate-and-validate`
8. `task/m10-e2-docs-review`
9. `task/m10-e3-closeout`

Parent-only serialized tasks:

- `task/m10-a1-preflight`
- `task/m10-a2-validator-contract`
- `task/m10-a3-doctor-alignment`
- `task/m10-a4-member-launch-seam`
- `task/m10-a5-restart-replacement`
- `task/m10-e1-integrate-and-validate`
- `task/m10-e2-docs-review`
- `task/m10-e3-closeout`

Worker-owned tasks:

- `task/m10-b1-runtime-contract-tests`
- `task/m10-b2-world-routing-restart-tests`

## Parallel Window B

This is the only worker window in the run.

It opens only after Gate C passes and after the parent has completed the production launch seam in
`task/m10-a5-restart-replacement`. There is no earlier honest worker window because:

1. `validator.rs` freezes the selection contract that both doctor and runtime must consume,
2. `PLAN-10` requires doctor alignment as required production work before the slice is considered
   coherent,
3. `async_repl.rs` owns both lazy launch and restart replacement on the same retained-control seam,
4. splitting those production edits across worktrees would create contract drift faster than it
   creates throughput.

The test wall does parallelize cleanly after the runtime seam lands:

- `task/m10-b1` covers operator/trace contract suites in files disjoint from the restart-routing
  integration file,
- `task/m10-b2` covers the world-first routing and restart integration suite,
- both workers validate against the same frozen production tree,
- both workers must stop if their assertions require parent-owned production edits.

### task/m10-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md)
   and [ORCH_PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-09.md).
2. Confirm the run executes from `feat/session-centric-state-store`.
3. Freeze the no-UI scope, exact `2`-worker cap, serialized production order, and parent-only
   production ownership into the run packet.
4. Initialize:
   - `.runs/plan-10/run-state.json`
   - `.runs/plan-10/queue.json`
   - `.runs/plan-10/session.log`
5. Record the repo-truth validation rule that package-targeted commands use `-p shell`.

Commands:

1. `git branch --show-current`
2. `mkdir -p .runs/plan-10/sentinels`

Acceptance:

1. The parent can explain why production work stays serial through restart replacement.
2. The parent can restate the v1 selection rule without ambiguity.
3. The parent can name the only honest worker window in the slice: the test wall after
   `task/m10-a5`.
4. `run-state.json` records the initial phase and queue.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-a1-preflight.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### Parent validation gate A

Required before `task/m10-a2-validator-contract` starts:

1. No invariant contradiction remains unresolved.
2. The parent can explain why zero eligible world members is the only case that preserves host-only
   behavior.
3. The parent can explain why docs are explicitly deferred until after the test wall.

### task/m10-a2-validator-contract

Ownership:

- parent only

Allowed files:

- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)

Scope:

1. Land the shared member-selection contract in `validator.rs`.
2. Add `validate_member_selection(...)` returning a generic `RuntimeSelectionDescriptor`.
3. Make zero, one, and many eligible world-member outcomes explicit.
4. Reuse `validate_runtime_realizability(...)` through neutral wording rather than forking a second
   realizability contract.
5. Add unit coverage proving the selection contract without changing doctor or REPL behavior yet.

Must not do:

1. No edits to `agents_cmd.rs`.
2. No edits to `async_repl.rs`.
3. No launch ownership in the store.
4. No selector UX, config surface, or heuristic fallback.

Commands:

1. `cargo test -p shell --lib -- --nocapture`
2. `cargo test -p shell --tests --no-run`

Acceptance:

1. The repo has one shared member-selection truth.
2. Zero, one, and many eligible world-member cases are tested.
3. Runtime descriptor shape stays generic.
4. Doctor and REPL behavior remain unchanged until later tasks.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-a2-validator-contract.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### Parent validation gate B

Required before `task/m10-a3-doctor-alignment` starts:

1. `cargo test -p shell --lib -- --nocapture` passes.
2. The parent records the frozen zero/one/many selection contract in `run-state.json`.
3. No downstream task is allowed to fork its own selection logic.

### task/m10-a3-doctor-alignment

Ownership:

- parent only

Allowed files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Scope:

1. Align `build_doctor_report(...)` to the same selection contract as runtime launch preflight.
2. Preserve host-only success when zero eligible world members exist.
3. Fail closed on ambiguous world-member selection before any world-member boundary claim is made.
4. Keep allowlist and world-boundary checks scoped to the unique selected member path.
5. Do not redesign `status` or `toolbox` here; this task is doctor alignment only.

Must not do:

1. No boolean "any world member exists" launch truth.
2. No new selection logic outside the helper contract already landed in `task/m10-a2`.
3. No edits to `async_repl.rs`.
4. No docs or test-wall edits yet.

Commands:

1. `cargo test -p shell --lib -- --nocapture`
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`

Acceptance:

1. Doctor and runtime share one selection truth.
2. `agent doctor` can distinguish zero vs one vs many eligible world-member candidates.
3. Ambiguous selection fails closed.
4. No runtime-launch logic has landed yet.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-a3-doctor-alignment.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### task/m10-a4-member-launch-seam

Ownership:

- parent only

Allowed files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Conditional parent-only escalation files only if a concrete launch gap forces it:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)

Scope:

1. Add lazy `ensure_member_runtime_ready(...)` in the world-backed command branch after
   `ensure_no_policy_drift(...)` and before `exec_world_pty(...)` / `exec_world_line(...)`.
2. Reuse the shared selection contract and authoritative parent world binding from
   `RuntimeOrchestrationContext`.
3. Reuse `AsyncReplAgentRuntime` and retained-control lifecycle machinery rather than copying a
   second member lifecycle.
4. Persist the member in `allocating` first and advertise it live only after retained UAA control,
   active event stream, and completion observer all exist.
5. Add bounded inline or local runtime tests in `async_repl.rs` proving launch-state progression.

Must not do:

1. No startup-time auto-launch.
2. No second member-only runtime holder unless reuse proves impossible and the run is blocked.
3. No direct writes to canonical or compatibility state outside store helpers.
4. No restart replacement yet beyond bounded scaffolding required by the same seam.

Commands:

1. `cargo test -p shell async_repl -- --nocapture`
2. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`

Acceptance:

1. The first world-backed command can lazily launch the selected member.
2. The second command on the same generation can reuse the live member.
3. Failed preflight exits without a half-live participant.
4. Any escalation into `state_store.rs` or `registry.rs` is justified and recorded.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-a4-member-launch-seam.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### task/m10-a5-restart-replacement

Ownership:

- parent only

Allowed files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Conditional parent-only escalation files only if a concrete seam gap forces it:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)

Scope:

1. Land restart replacement on the same retained-control seam as lazy launch.
2. After authoritative world binding persists and stale members are invalidated, either launch a
   replacement participant for the new generation or leave honest absence.
3. Use `new_replacement_participant(...)` with fresh `participant_id` and
   `resumed_from_participant_id=<old participant>`.
4. Keep stale generation state invalidated and non-live even when terminal trace rows persist.
5. Finalize bounded inline/runtime tests around replacement lifecycle in `async_repl.rs`.

Must not do:

1. No second restart-specific lifecycle definition.
2. No trace-authorized resurrection of stale members.
3. No worker dispatch before restart replacement is frozen.
4. No docs edits.

Commands:

1. `cargo test -p shell async_repl -- --nocapture`
2. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
3. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

Acceptance:

1. Restart with a live member yields either a live replacement on the new generation or a clear
   failed replacement outcome.
2. Stale generation never appears live again.
3. The retained-control seam still defines liveness for both first launch and replacement.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-a5-restart-replacement.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### Parent validation gate C

Required before the worker window opens:

1. `cargo test -p shell async_repl -- --nocapture` passes.
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run` passes.
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run` passes.
4. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run` passes.
5. `run-state.json` records the frozen selection, doctor, lazy-launch, and restart-replacement
   contracts.
6. The parent seeds both worker worktrees from the exact post-`task/m10-a5` tree.

### task/m10-b1-runtime-contract-tests

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m10-runtime-contract-tests`
- `codex/feat-session-centric-state-store-m10-runtime-contract-tests`

Allowed files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Scope:

1. Add doctor contract cases for zero vs one vs many world-member candidates, especially
   ambiguous-selection fail-closed behavior.
2. Add `agent status --json` contract coverage proving the live launched member comes from runtime
   state with top-level `world_id` and `world_generation`.
3. Add contract coverage proving `toolbox status|env` stays orchestrator-anchored even when the
   member is live.
4. Add trace persistence coverage for member `Registered`, `Status`, and terminal events plus
   replacement lineage fields.
5. Keep stale terminal rows auditable without making them live again.

Must not do:

1. No edits to `validator.rs`, `agents_cmd.rs`, or `async_repl.rs`.
2. No docs edits.
3. No fresh selection logic in tests that the production seam does not own.
4. No weakening of orchestrator-only toolbox anchoring.

Commands:

1. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
2. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`

Execution packet for the worker:

1. Read [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md).
2. Read [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs).
3. Read [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs).
4. Make only allowed-file changes.
5. Run the exact commands listed above.
6. Stop immediately if the assertions require production edits in parent-owned files.

Acceptance:

1. Contract coverage proves doctor, status, toolbox, and trace all consume the frozen launch truth.
2. Trace lineage stays auditable without becoming current liveness.
3. The worker report includes only changed files, exact commands run, and blockers.
4. The lane stays inside its allowed files.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-b1-runtime-contract-tests.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json` if the worker needs parent-owned production files

### task/m10-b2-world-routing-restart-tests

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m10-world-routing-restart`
- `codex/feat-session-centric-state-store-m10-world-routing-restart`

Allowed files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Scope:

1. Add integration coverage for first world-backed command causing lazy member launch.
2. Add coverage for same-generation reuse on a second command.
3. Add restart integration coverage proving a live replacement member can come up on the new
   generation.
4. Add replacement-failure coverage proving honest absence instead of stale-liveness resurrection.
5. Keep the integration suite anchored to the frozen parent-owned production seam rather than
   inventing new hooks.

Must not do:

1. No edits to `async_repl.rs`.
2. No edits to any other test file.
3. No docs edits.
4. No workarounds that bypass the real world-backed command branch.

Commands:

1. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

Execution packet for the worker:

1. Read [PLAN-10.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-10.md).
2. Read [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs).
3. Make only allowed-file changes.
4. Run the exact command listed above.
5. Stop immediately if the assertions require production edits in parent-owned files.

Acceptance:

1. The integration suite proves lazy launch, same-generation reuse, restart replacement, and
   honest absence on replacement failure.
2. The worker report includes only changed files, exact commands run, and blockers.
3. The lane stays inside its allowed file.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-b2-world-routing-restart-tests.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json` if the worker needs parent-owned production files

### task/m10-e1-integrate-and-validate

Ownership:

- parent only

Scope:

1. Review both worker outputs against the frozen selection, doctor, launch, and restart
   contracts.
2. Reject any worker patch that touches unowned files, assumes stale trace authority, or omits test
   evidence.
3. Integrate accepted worker outputs into the parent checkout.
4. If a worker proves bounded production fallout is required, make those changes in the parent
   checkout and rerun the affected validation steps before acceptance.
5. Run the final validation stack in the exact order defined below.

Acceptance:

1. Every accepted worker output has a matching parent-written sentinel.
2. No accepted patch forks selection truth or weakens fail-closed launch behavior.
3. No accepted patch lets stale trace rows authorize current liveness.
4. `run-state.json` records accepted and rejected outputs explicitly.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-e1-integrate-and-validate.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### task/m10-e2-docs-review

Ownership:

- parent only

Allowed files:

- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Scope:

1. Review whether the landed runtime behavior changed user-visible wording or trace examples.
2. If no wording drift exists, record an explicit no-change decision in `run-state.json` and
   `session.log`.
3. If wording drift exists, make the smallest truthful docs edits needed after the test wall is
   already green.
4. Keep docs aligned with the shipped behavior:
   - explicit unique-member selection rule,
   - lazy world-backed launch on first need,
   - orchestrator-anchored toolbox posture,
   - restart replacement or honest absence,
   - no stale-trace resurrection.

Must not do:

1. No docs-first speculation.
2. No docs edits to justify unproven runtime behavior.
3. No reopening of runtime design through docs.

Acceptance:

1. Docs are either unchanged with an explicit recorded reason or minimally updated to match shipped
   behavior.
2. No doc implies scheduler, selector UX, or trace-authorized liveness.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-e2-docs-review.ok`

Blocked-path output:

- `.runs/plan-10/blocked.json`

### task/m10-e3-closeout

Ownership:

- parent only

Scope:

1. Mark the run complete in `.runs/plan-10/run-state.json`.
2. Write `.runs/plan-10/closeout.md` with:
   - accepted tasks,
   - integrated worktrees and branches,
   - validation commands and outcomes,
   - final shipped selection rule,
   - final doctor/runtime alignment statement,
   - final lazy-launch and restart-replacement behavior,
   - explicit docs-reviewed outcome,
   - confirmation that the parent remained the only final branch writer.
3. Append the final acceptance rationale to `.runs/plan-10/session.log`.

Acceptance:

1. `closeout.md` is present and matches the final branch state.
2. All sentinels through `task/m10-e3-closeout` exist.
3. No blocked-path artifact exists for a successful run.

Green-path output:

- `.runs/plan-10/sentinels/task-m10-e3-closeout.ok`

## Context-Control Rules

1. The parent owns `.runs/plan-10/*`. Workers do not edit run-state, sentinels, queue, session
   log, blocked state, or closeout artifacts.
2. Worker packets include only:
   - the task ID,
   - allowed files,
   - frozen invariants,
   - stop conditions,
   - exact commands to run.
3. Workers must stop immediately if they need to touch any file outside their allowed list.
4. Workers report:
   - changed files,
   - exact commands run,
   - blockers.
5. Workers do not rebase, merge, or integrate each other’s work.
6. The parent rejects any worker patch that silently broadens scope or re-derives selection or
   liveness rules outside the frozen production contract.
7. If worker outputs conflict with current parent truth, the parent re-derives the correct result
   from production code and rewrites the patch locally instead of negotiating blended semantics
   across worktrees.

## Tests And Acceptance

### Pre-worker proof order

Run these parent-owned checks before dispatching the worker window:

1. `cargo test -p shell --lib -- --nocapture`
2. `cargo test -p shell async_repl -- --nocapture`
3. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --no-run`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --no-run`
5. `cargo test -p shell --test agent_hub_trace_persistence -- --no-run`

### Final validation order

Run these commands in this exact order during `task/m10-e1-integrate-and-validate`:

1. `cargo test -p shell async_repl -- --nocapture`
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
3. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
4. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`
5. `cargo test -p shell -- --nocapture`
6. `cargo fmt --all -- --check`
7. `cargo clippy -p shell --all-targets -- -D warnings`
8. `cargo clippy --workspace --all-targets -- -D warnings`
9. `cargo test --workspace -- --nocapture`

### Acceptance checklist

The slice is accepted only if all of these are true:

1. One unique eligible world-scoped member can be selected deterministically from inventory.
2. Zero eligible world members preserves host-only behavior without inventing fake member state.
3. Ambiguous world-member selection fails closed in both doctor and runtime launch.
4. The first world-backed REPL command can lazily launch the selected member under a live
   orchestrator session.
5. The launched member is persisted in `allocating` first and only becomes authoritative-live after
   retained UAA ownership is proven.
6. `substrate agent status --json` can surface the live launched member from runtime state with
   top-level `world_id` and `world_generation`.
7. `substrate agent toolbox status|env` remains orchestrator-scoped.
8. World restart invalidates the stale member and can launch a replacement participant on the new
   generation.
9. Failed replacement leaves honest absence, not stale liveness.
10. Trace persistence remains auditable without becoming current liveness authority.
11. The targeted shell commands and workspace gates above pass.
12. Docs were touched only if the runtime wording actually changed, and otherwise the no-change
    decision was recorded explicitly.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch invents or preserves a selection rule not frozen in `validator.rs`.
3. The patch authorizes current liveness from trace rows.
4. The patch weakens fail-closed behavior for ambiguity, missing parent binding, or replacement
   failure.
5. The patch requires concurrent parent edits to production files to become intelligible.
6. The patch omits test evidence for the behavior it claims to cover.
7. The patch implies docs changes are required to explain behavior the runtime still does not prove.
8. The patch broadens scope into scheduler, selector UX, or public control-plane work.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/plan-10/sentinels/task-m10-a1-preflight.ok` exists.
2. `.runs/plan-10/sentinels/task-m10-a2-validator-contract.ok` exists.
3. `.runs/plan-10/sentinels/task-m10-a3-doctor-alignment.ok` exists.
4. `.runs/plan-10/sentinels/task-m10-a4-member-launch-seam.ok` exists.
5. `.runs/plan-10/sentinels/task-m10-a5-restart-replacement.ok` exists.
6. `.runs/plan-10/sentinels/task-m10-b1-runtime-contract-tests.ok` exists.
7. `.runs/plan-10/sentinels/task-m10-b2-world-routing-restart-tests.ok` exists.
8. `.runs/plan-10/sentinels/task-m10-e1-integrate-and-validate.ok` exists.
9. `.runs/plan-10/sentinels/task-m10-e2-docs-review.ok` exists.
10. `.runs/plan-10/sentinels/task-m10-e3-closeout.ok` exists.
11. `.runs/plan-10/run-state.json` exists and records a completed terminal state.
12. `.runs/plan-10/queue.json` and `.runs/plan-10/session.log` exist.
13. `.runs/plan-10/closeout.md` exists and matches the final accepted branch state.
14. `.runs/plan-10/blocked.json` does not exist.

### Blocked termination

The run terminates blocked only if all of these are true:

1. `.runs/plan-10/blocked.json` exists and records the blocking reason.
2. `.runs/plan-10/run-state.json` exists and records `blocked` as the terminal state.
3. `session.log` contains the parent rationale for the stop.
4. No downstream task sentinel may be written after the blocking point.
5. `.runs/plan-10/sentinels/task-m10-e3-closeout.ok` must not exist.
6. `.runs/plan-10/closeout.md` must not exist.

## Closeout

Successful closeout records:

1. the final shipped member-selection rule,
2. the final doctor/runtime alignment contract,
3. the final lazy-launch and restart-replacement behavior,
4. the exact validation order and outcomes,
5. whether docs were changed or explicitly left untouched,
6. confirmation that the parent remained the only final branch writer,
7. confirmation that worker lanes stayed inside their file boundaries.

## Assumptions

1. The parent run starts from the current branch baseline `feat/session-centric-state-store`.
2. `PLAN-09` already froze the store-owned session authority ladder that `PLAN-10` must consume
   rather than redesign.
3. The Rust package name `shell` is authoritative for validation commands in this repository.
4. `state_store.rs`, `registry.rs`, and `world_gateway.rs` can remain untouched unless a concrete
   runtime seam gap proves otherwise.
5. Any user-visible wording drift can be corrected with minimal post-test docs edits and does not
   require reopening runtime architecture.
