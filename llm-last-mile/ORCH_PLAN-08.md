# ORCH_PLAN-08: Explicit Orchestration Authority For Event Emission

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-08.md)  
Execution type: shell/runtime authority cleanup orchestration plan, no UI scope

## Summary

This run executes `PLAN-08` on the current branch `feat/session-centric-state-store` with an exact worker cap of `2`. The parent remains the only integrator, the only final branch writer, and the only agent allowed to mutate the coupled production seam across [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs), [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs), and [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs). The canonical run-state source of truth is `.runs/plan-08/run-state.json`.

The worker cap is exactly `2` because the only honest parallelism in this slice is the late regression/doc window after the shell event context contract and launch-boundary authority/correlation mapping are frozen. There are intentionally zero safe production-code worker lanes before that point. `agent_events.rs`, `async_repl.rs`, `exec.rs`, and `world_ops.rs` are one authority seam, and `execute_command(...)` signature fallout into [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs) is part of that same parent-owned path, not an independent lane.

Worktree set for the run:

- `../substrate-m08-trace-contract-docs` on `codex/feat-session-centric-state-store-m08-trace-contract-docs`
- `../substrate-m08-repl-world-regressions` on `codex/feat-session-centric-state-store-m08-repl-world-regressions`

The critical path stays parent-owned through these phases:

1. `task/m08-a1-preflight`
2. `task/m08-a2-context-and-launch-freeze`
3. `task/m08-b1-host-plumbing-and-signature-fallout`
4. `task/m08-c1-world-stream-plumbing-and-helper-retirement`
5. `task/m08-e1-integrate-and-validate`
6. `task/m08-e2-closeout`

This is deliberate. `PLAN-08` is truthful that most production work is sequential until the explicit shell context contract and launch-boundary authority/correlation mapping are frozen. Fake parallelism across `agent_events.rs`, `async_repl.rs`, `exec.rs`, and `world_ops.rs` would create merge churn and contract drift faster than it would create throughput. The parent-owned execution brief is:

1. freeze the explicit shell event context shape and the no-PID-lookup rule first,
2. freeze the REPL launch-boundary authority and correlation mapping second,
3. thread that frozen contract through host execution and `execute_command(...)` fallout third,
4. thread the same frozen contract through world non-PTY streaming and retire production helper lookups fourth,
5. open workers only after the production seam is stable in the parent checkout,
6. integrate worker outputs in one parent-owned validation pass,
7. close the run only if the shell-targeted validation stack and final repo gates are green.

## Hard Guards

### Locked invariants

1. `PLAN-08` is a shell/runtime authority cleanup slice only. No UI work is authorized.
2. The parent is the only integrator and the only final branch writer.
3. Shell-owned orchestration-scoped rows must use explicit caller-provided authority; ambient PID recovery on production emission paths is forbidden.
4. Shell-owned stream rows must not emit with synthetic correlation such as `run_id="unknown"`.
5. Missing orchestration context suppresses only the orchestration-scoped `agent_event` row; stdout, stderr, and trace spans must keep working.
6. `RuntimeOrchestrationContext` in `async_repl.rs` remains the authority source for REPL-owned orchestration context.
7. [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs) remains a non-orchestrator caller in this slice and must pass `None` explicitly after `execute_command(...)` changes.
8. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) is a helper-retirement seam only; this slice does not authorize live-state redesign.
9. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md) is a contract lock seam; it must reflect the shipped suppression and correlation rules.
10. No new cache, registry, service, selector UX, or infrastructure is allowed.
11. No human approval gate is required or invented for this run.
12. Validation commands must use actual package names in this repo, especially `cargo test -p shell`.

### File-level boundaries

Parent-owned serialized production surfaces:

- [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Worker-safe late-phase validation and doc surfaces:

- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)

Read-for-truth only:

- [llm-last-mile/PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-08.md)
- [llm-last-mile/ORCH_PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-06.md)
- [llm-last-mile/ORCH_PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-05.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/plan-08/blocked.json` if any of these occur:

1. A task requires parallel edits across `agent_events.rs`, `async_repl.rs`, `exec.rs`, or `world_ops.rs` before the context and launch contract is frozen.
2. A task requires `invocation/runtime.rs` to derive orchestration context instead of explicitly passing `None`.
3. A task restores `find_active_orchestration_session_for_pid(...)` or any renamed equivalent to production event-emission control flow.
4. A task emits orchestration-scoped shell stream rows with synthetic fallback correlation.
5. A task blocks stdout, stderr, or trace span emission when orchestration-scoped rows are suppressed.
6. A worker lane needs to touch any parent-owned production file or any source-local unit test inside those files.
7. A task expands into `PLAN-09` live-state authority work, restart semantics redesign, or schema redesign.
8. A task requires new infrastructure or operator approval not already present in the plan.

## Orchestration State Surfaces

### Canonical run state

Single local source of truth for the run:

- `.runs/plan-08/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- frozen shell event context contract,
- frozen launch-boundary authority/correlation mapping,
- accepted and rejected worker outputs,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/plan-08/queue.json`
- `.runs/plan-08/session.log`
- `.runs/plan-08/sentinels/task-m08-a1-preflight.ok`
- `.runs/plan-08/sentinels/task-m08-a2-context-and-launch-freeze.ok`
- `.runs/plan-08/sentinels/task-m08-b1-host-plumbing-and-signature-fallout.ok`
- `.runs/plan-08/sentinels/task-m08-c1-world-stream-plumbing-and-helper-retirement.ok`
- `.runs/plan-08/sentinels/task-m08-d1-trace-contract-docs.ok`
- `.runs/plan-08/sentinels/task-m08-d2-repl-world-regressions.ok`
- `.runs/plan-08/sentinels/task-m08-e1-integrate-and-validate.ok`
- `.runs/plan-08/sentinels/task-m08-e2-closeout.ok`
- `.runs/plan-08/blocked.json`
- `.runs/plan-08/closeout.md`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.
5. Worker-generated notes never replace parent-written sentinels or run-state artifacts.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact worker cap: `2` active worker lanes.
4. There are zero worker lanes during `task/m08-a2`, `task/m08-b1`, or `task/m08-c1`.
5. No two lanes may edit `agent_events.rs`, `async_repl.rs`, `exec.rs`, `world_ops.rs`, `invocation/runtime.rs`, or `state_store.rs` concurrently.
6. Worker lanes open only after the parent has stabilized the production seam and seeded both worktrees from that exact state.
7. `task/m08-d1-trace-contract-docs` and `task/m08-d2-repl-world-regressions` are the only parallel window in this run.
8. If worker tasks expose missing parent-owned support work, the worker stops and hands the change back to the parent instead of widening scope.

## Approval And Gate Model

There are no human approval gates defined for this run.

Replacement control mechanism:

1. parent validation gates,
2. parent-written sentinels,
3. `session.log` for acceptance and rejection rationale,
4. `blocked.json` for hard-stop termination,
5. `closeout.md` for successful completion.

### Gate A: Packet freeze

Required before implementation starts:

- parent re-reads `PLAN-08`, `ORCH_PLAN-06`, and `ORCH_PLAN-05`,
- parent records that early production work stays parent-owned because the authority seam is coupled across REPL launch, host execution, and world streaming,
- parent records the package-name normalization: use `shell`, never `substrate-shell`.

### Gate B: Context and launch freeze

Required before host plumbing starts:

- the shell event context type is frozen,
- the REPL launch-boundary mapping for `cmd_id`, `run_id`, and `span_id` is frozen,
- the parent can explain why non-REPL callers in `invocation/runtime.rs` must explicitly pass `None`.

### Gate C: Host-path freeze

Required before world plumbing starts:

- `execute_command(...)` signature fallout is complete,
- host stream threads capture immutable explicit context before background I/O starts,
- no host production path still emits `run_id="unknown"` for orchestration-scoped rows.

### Gate D: Production stabilization

Required before the worker window opens:

- production integration is complete in the parent checkout,
- no structural churn remains in `agent_events.rs`, `async_repl.rs`, `exec.rs`, `world_ops.rs`, `invocation/runtime.rs`, or `state_store.rs`,
- the parent has completed the pre-worker compile/proof checks defined below,
- both worker worktrees are seeded from the exact post-`task/m08-c1` tree.

### Gate E: Final acceptance

Required before closeout:

- both worker outputs are accepted or deliberately rejected and replaced by parent work,
- the final validation order passes,
- docs reflect the final authority boundary,
- production emitters no longer depend on PID-based orchestration recovery.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-m08-trace-contract-docs`
  - `codex/feat-session-centric-state-store-m08-trace-contract-docs`
- `../substrate-m08-repl-world-regressions`
  - `codex/feat-session-centric-state-store-m08-repl-world-regressions`

Subagents do not merge each other’s work. They return patches, touched files, tests run, and blockers to the parent.

### Task graph

Execution graph for the run:

1. `task/m08-a1-preflight`
2. `task/m08-a2-context-and-launch-freeze`
3. `task/m08-b1-host-plumbing-and-signature-fallout`
4. `task/m08-c1-world-stream-plumbing-and-helper-retirement`
5. `task/m08-d1-trace-contract-docs` and `task/m08-d2-repl-world-regressions` in parallel
6. `task/m08-e1-integrate-and-validate`
7. `task/m08-e2-closeout`

Parent-only serialized tasks:

- `task/m08-a1-preflight`
- `task/m08-a2-context-and-launch-freeze`
- `task/m08-b1-host-plumbing-and-signature-fallout`
- `task/m08-c1-world-stream-plumbing-and-helper-retirement`
- `task/m08-e1-integrate-and-validate`
- `task/m08-e2-closeout`

Worker-owned tasks:

- `task/m08-d1-trace-contract-docs`
- `task/m08-d2-repl-world-regressions`

## Parallel Window D

This is the only worker window in the run.

It opens only after Gate D passes and after the parent has finished the full production path through `task/m08-c1-world-stream-plumbing-and-helper-retirement`. There are zero safe production-code workers before that because the launch-boundary authority contract and the host/world streaming contract are one coupled seam. `agent_events.rs`, `async_repl.rs`, `exec.rs`, and `world_ops.rs` cannot be split honestly until the parent has frozen the exact context shape and correlation rules.

### task/m08-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read [PLAN-08.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-08.md), [ORCH_PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-06.md), and [ORCH_PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-05.md).
2. Confirm the run executes from `feat/session-centric-state-store`.
3. Freeze the no-UI scope, parent-only critical seam list, and honest concurrency limits into the run packet.
4. Initialize:
   - `.runs/plan-08/run-state.json`
   - `.runs/plan-08/queue.json`
   - `.runs/plan-08/session.log`
5. Record the repo-truth validation rule that all package-targeted commands use `-p shell`.

Acceptance:

1. The parent can explain why early production work stays parent-owned.
2. The parent can name the execute-command fallout into `invocation/runtime.rs`.
3. `run-state.json` records the initial phase and queue.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-a1-preflight.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json`

### Parent validation gate A

Required before `task/m08-a2-context-and-launch-freeze` starts:

1. No invariant contradiction remains unresolved.
2. The parent can explain why fake parallelism across `agent_events.rs`, `async_repl.rs`, `exec.rs`, and `world_ops.rs` is disallowed.
3. The parent can restate the suppression rule as "real id or suppress."

### task/m08-a2-context-and-launch-freeze

Ownership:

- parent only

Why serialized:

- [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs) and [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) define the contract that every downstream host/world emitter must consume. There is no honest throughput gain from splitting downstream work before this mapping is frozen.

Allowed files:

- [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Scope:

1. Add the explicit shell event context type and any adjacent shell-local helpers.
2. Move command-completion emission onto the explicit context contract.
3. Freeze how REPL launch points derive authority from `RuntimeOrchestrationContext` plus the live manifest/world snapshot.
4. Freeze how REPL launch points choose `cmd_id`, `run_id`, and `span_id` for shell-owned rows.
5. Remove REPL production dependence on `resolve_active_orchestration_session_id()`.
6. Keep source-local regression additions in these files parent-owned.

Must not do:

1. No host or world stream plumbing yet.
2. No `invocation/runtime.rs` fallback logic.
3. No new shared schema or cross-crate contract redesign.

Acceptance:

1. `publish_command_completion(...)` consumes the frozen shell context contract.
2. REPL command-completion paths no longer recover orchestration identity from PID-owned runtime state.
3. The parent can state exactly when `run_id` is authoritative and when rows must be suppressed.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-a2-context-and-launch-freeze.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json`

### Parent validation gate B

Required before `task/m08-b1-host-plumbing-and-signature-fallout` starts:

1. `cargo test -p shell publish_command_completion -- --nocapture` passes.
2. `cargo test -p shell build_world_restart_required_alert_only_builds_with_orchestration_context -- --nocapture` passes.
3. The frozen shell context contract and launch-boundary mapping are recorded in `run-state.json`.

### task/m08-b1-host-plumbing-and-signature-fallout

Ownership:

- parent only

Why serialized:

- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs) owns `execute_command(...)`, `execute_external(...)`, and host stream thread setup. The signature fallout into [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs) is mechanical but coupled to the same change.

Allowed files:

- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
- [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for direct call-site fallout only

Scope:

1. Thread optional explicit shell event context into `execute_command(...)` and `execute_external(...)`.
2. Capture immutable host-stream authority and stable run correlation before background stream threads start.
3. Remove host-path PID lookup and stop emitting orchestration-scoped host stream rows with `run_id="unknown"`.
4. Update `invocation/runtime.rs` callers to pass `None` explicitly.
5. Preserve host-only execution behavior when orchestration context is absent.

Must not do:

1. No world non-PTY frame-loop changes yet.
2. No attempt to infer orchestration context inside `invocation/runtime.rs`.
3. No worker dispatch before the parent proves the host path compiles and suppresses correctly.

Acceptance:

1. Non-REPL host invocation callers compile with explicit `None`.
2. Host stream emission uses launch-owned correlation only.
3. No host production path still depends on `resolve_active_orchestration_session_id()`.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-b1-host-plumbing-and-signature-fallout.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json`

### Parent validation gate C

Required before `task/m08-c1-world-stream-plumbing-and-helper-retirement` starts:

1. `cargo test -p shell emit_stream_chunk -- --nocapture` passes.
2. `cargo test -p shell --tests --no-run` passes so the parent is carrying a compiling host-path diff forward.
3. The parent can explain why `invocation/runtime.rs` passing `None` is a contract requirement, not an omission.

### task/m08-c1-world-stream-plumbing-and-helper-retirement

Ownership:

- parent only

Why serialized:

- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) consumes the same frozen authority and correlation contract as the host path, and [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) is only being narrowed as a helper-retirement seam.

Allowed files:

- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for parent-owned fallout only

Scope:

1. Thread explicit event context into `stream_non_pty_via_agent(...)`, `process_agent_stream_body(...)`, and `emit_stream_chunk(...)`.
2. Remove world-path PID lookup and suppress orchestration-scoped stream rows until both context and real run correlation exist.
3. Preserve stdout/stderr passthrough and deny-path terminal behavior when rows are suppressed.
4. Leave `find_active_orchestration_session_for_pid(...)` available only for tests or diagnostics if still needed.
5. Keep any source-local regression additions in these parent-owned files.

Must not do:

1. No new state-store authority behavior.
2. No synthetic run-correlation fallback from `active_span_id` alone.
3. No widening of worker-safe scope into source-local tests.

Acceptance:

1. No production event-emission path in `exec.rs`, `world_ops.rs`, or `async_repl.rs` resolves orchestration identity from PID-owned runtime state.
2. World non-PTY rows no longer emit with synthetic correlation.
3. Suppression preserves terminal output and trace span behavior.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-c1-world-stream-plumbing-and-helper-retirement.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json`

### Parent validation gate D

Required before the worker window opens:

1. `cargo test -p shell emit_world_restarted_alert_only_emits_with_orchestration_context -- --nocapture` passes.
2. `cargo test -p shell start_host_orchestrator_runtime_persists_participant_snapshots_across_lifecycle_states -- --nocapture` passes.
3. `cargo test -p shell --tests --no-run` passes from the stabilized parent checkout.
4. The parent seeds both worker worktrees from the exact post-`task/m08-c1` tree.

### task/m08-d1-trace-contract-docs

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m08-trace-contract-docs`
- `codex/feat-session-centric-state-store-m08-trace-contract-docs`

Allowed files:

- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)

Scope:

1. Update `docs/TRACE.md` so shell-owned command completion and shell-owned stream emitters are explicit examples of the "real id or suppress" rule.
2. Add or tighten trace assertions for:
   - no-context shell completion suppressing orchestration-scoped `agent_event` rows,
   - runtime-owned rows retaining real orchestration session identity,
   - shell-owned omission behavior staying additive rather than heuristic.
3. Add a bounded guard in the allowed integration surface if needed, but only if it does not require touching production files.

Must not do:

1. No edits to `agent_events.rs`, `async_repl.rs`, `exec.rs`, `world_ops.rs`, `invocation/runtime.rs`, or `state_store.rs`.
2. No edits to `repl_world_first_routing_v1.rs`.
3. No source-local unit-test edits inside production files.

Commands:

1. `cargo test -p shell no_context_shell_command_completion_does_not_synthesize_agent_event_trace_row -- --nocapture`
2. `cargo test -p shell runtime_owned_agent_event_rows_retain_shell_session_and_real_orchestration_session -- --nocapture`
3. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`

Acceptance:

1. `docs/TRACE.md` reflects the final shell-owned suppression and correlation contract.
2. The worker report includes exact tests run and any remaining contract ambiguities.
3. No worker patch widens into parent-owned source files.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-d1-trace-contract-docs.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json` if the worker needs parent-owned files

### task/m08-d2-repl-world-regressions

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m08-repl-world-regressions`
- `codex/feat-session-centric-state-store-m08-repl-world-regressions`

Allowed files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Scope:

1. Lock restart-alert suppression behavior so the refactor does not weaken the existing explicit-context-only posture.
2. Add or tighten world-routing regression coverage where the existing integration harness can prove suppression-only behavior without touching production code.
3. Make unmet helper dependencies explicit instead of silently requesting production edits.

Must not do:

1. No edits to any production file.
2. No edits to `agent_hub_trace_persistence.rs` or `docs/TRACE.md`.
3. No new integration harness outside the allowed file.

Commands:

1. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

Acceptance:

1. Restart-alert suppression tests still pass after the production seam changes.
2. The worker report clearly identifies any remaining world-stream coverage gaps that require parent-owned work.
3. The lane stays inside the single allowed file.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-d2-repl-world-regressions.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json` if the worker needs parent-owned files

### task/m08-e1-integrate-and-validate

Ownership:

- parent only

Scope:

1. Review both worker outputs against the frozen authority and correlation contracts.
2. Reject any worker patch that touches unowned files, assumes ambient lookup, or omits test evidence.
3. Integrate accepted worker outputs into the parent checkout.
4. If worker outputs expose missing parent-owned support changes, make those changes in the parent checkout and rerun the affected targeted validation steps.
5. Run the final validation stack in the exact order defined below.

Acceptance:

1. Every accepted worker output has a matching parent-written sentinel.
2. No production emitter depends on PID-based orchestration recovery.
3. The full validation order passes.
4. `run-state.json` records accepted and rejected outputs explicitly.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-e1-integrate-and-validate.ok`

Blocked-path output:

- `.runs/plan-08/blocked.json`

### task/m08-e2-closeout

Ownership:

- parent only

Scope:

1. Mark the run complete in `.runs/plan-08/run-state.json`.
2. Write `.runs/plan-08/closeout.md` with:
   - accepted tasks,
   - integrated worktrees and branches,
   - validation commands and outcomes,
   - any retained diagnostic-only use of helper APIs,
   - confirmation that production emitters no longer consult PID-based session lookup.
3. Append the final acceptance rationale to `.runs/plan-08/session.log`.

Acceptance:

1. `closeout.md` is present and matches the final branch state.
2. All sentinels through `task/m08-e2-closeout` exist.
3. No blocked-path artifact exists for a successful run.

Green-path output:

- `.runs/plan-08/sentinels/task-m08-e2-closeout.ok`

## Context-Control Rules

1. The parent owns `.runs/plan-08/*`. Workers do not edit run-state, sentinels, queue, session log, blocked state, or closeout artifacts.
2. Worker packets include only:
   - the task ID,
   - allowed files,
   - frozen invariants,
   - stop conditions,
   - exact tests to run.
3. Workers must stop immediately if they need to touch any file outside their allowed list.
4. Workers report:
   - touched files,
   - tests run,
   - unresolved assumptions,
   - blockers,
   - whether they observed any remaining ambient-lookup or synthetic-correlation assumptions.
5. Workers do not rebase, merge, or integrate each other’s work.
6. The parent rejects any worker patch that silently broadens scope or makes authority decisions not already frozen in `run-state.json`.
7. If worker outputs conflict with current parent truth, the parent re-derives the correct result from production code and rewrites the patch locally instead of negotiating a blended state across worktrees.

## Tests And Acceptance

### Pre-worker proof order

Run these parent-owned checks before dispatching the worker window:

1. `cargo test -p shell publish_command_completion -- --nocapture`
2. `cargo test -p shell emit_stream_chunk -- --nocapture`
3. `cargo test -p shell build_world_restart_required_alert_only_builds_with_orchestration_context -- --nocapture`
4. `cargo test -p shell emit_world_restarted_alert_only_emits_with_orchestration_context -- --nocapture`
5. `cargo test -p shell start_host_orchestrator_runtime_persists_participant_snapshots_across_lifecycle_states -- --nocapture`
6. `cargo test -p shell --tests --no-run`

### Final validation order

Run these commands in this exact order during `task/m08-e1-integrate-and-validate`:

1. `cargo test -p shell publish_command_completion -- --nocapture`
2. `cargo test -p shell emit_stream_chunk -- --nocapture`
3. `cargo test -p shell build_world_restart_required_alert_only_builds_with_orchestration_context -- --nocapture`
4. `cargo test -p shell emit_world_restarted_alert_only_emits_with_orchestration_context -- --nocapture`
5. `cargo test -p shell start_host_orchestrator_runtime_persists_participant_snapshots_across_lifecycle_states -- --nocapture`
6. `cargo test -p shell no_context_shell_command_completion_does_not_synthesize_agent_event_trace_row -- --nocapture`
7. `cargo test -p shell runtime_owned_agent_event_rows_retain_shell_session_and_real_orchestration_session -- --nocapture`
8. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`
9. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
10. `cargo test -p shell -- --nocapture`
11. `cargo fmt --all -- --check`
12. `cargo clippy -p shell --all-targets -- -D warnings`
13. `cargo clippy --workspace --all-targets -- -D warnings`
14. `cargo test --workspace -- --nocapture`

Conditional blast-radius checks:

1. Run `cargo test -p world-agent -- --nocapture`
2. Run `cargo test -p world-api -- --nocapture`
3. Run `cargo test -p agent-api-types -- --nocapture`

Only require that conditional block if the implementation escapes shell-local authority plumbing and changes shared stream-frame, request, or cross-crate event-shape assumptions.

### Acceptance checklist

The slice is accepted only if all of these are true:

1. No production event-emission path in `async_repl.rs`, `exec.rs`, or `world_ops.rs` resolves orchestration identity from `shell_owner_pid`.
2. Shell-owned command-completion rows consume explicit caller-provided authority.
3. Shell-owned stream rows consume explicit caller-provided authority and real run correlation.
4. Non-REPL callers in `invocation/runtime.rs` pass `None` explicitly instead of re-deriving runtime context.
5. Missing context suppresses only orchestration-scoped `agent_event` rows and does not suppress stdout, stderr, or trace spans.
6. `docs/TRACE.md` reflects the final "real id or suppress" rule.
7. `find_active_orchestration_session_for_pid(...)`, if retained, is no longer part of production emission control flow.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch assumes PID-based recovery or synthetic run correlation is acceptable for production emitters.
3. The patch requires concurrent parent edits to `agent_events.rs`, `async_repl.rs`, `exec.rs`, `world_ops.rs`, `invocation/runtime.rs`, or `state_store.rs` to become intelligible.
4. The patch omits test evidence for the behavior it claims to cover.
5. The patch leaves `docs/TRACE.md` or the targeted regression surfaces contradicting the final production code.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/plan-08/sentinels/task-m08-a1-preflight.ok` exists.
2. `.runs/plan-08/sentinels/task-m08-a2-context-and-launch-freeze.ok` exists.
3. `.runs/plan-08/sentinels/task-m08-b1-host-plumbing-and-signature-fallout.ok` exists.
4. `.runs/plan-08/sentinels/task-m08-c1-world-stream-plumbing-and-helper-retirement.ok` exists.
5. `.runs/plan-08/sentinels/task-m08-d1-trace-contract-docs.ok` exists.
6. `.runs/plan-08/sentinels/task-m08-d2-repl-world-regressions.ok` exists.
7. `.runs/plan-08/sentinels/task-m08-e1-integrate-and-validate.ok` exists.
8. `.runs/plan-08/sentinels/task-m08-e2-closeout.ok` exists.
9. `.runs/plan-08/run-state.json` exists and records a completed terminal state.
10. `.runs/plan-08/queue.json` and `.runs/plan-08/session.log` exist.
11. `.runs/plan-08/closeout.md` exists and matches the final accepted branch state.
12. `.runs/plan-08/blocked.json` does not exist.

### Blocked termination

The run terminates blocked only if all of these are true:

1. `.runs/plan-08/blocked.json` exists and records the blocking reason.
2. `.runs/plan-08/run-state.json` exists and records `blocked` as the terminal state.
3. `session.log` contains the parent rationale for the stop.
4. No downstream task sentinel may be written after the blocking point.
5. `.runs/plan-08/sentinels/task-m08-e2-closeout.ok` must not exist.
6. `.runs/plan-08/closeout.md` must not exist.

## Closeout

Successful closeout records:

1. the final frozen shell event context contract that shipped,
2. the final launch-boundary authority/correlation mapping,
3. the exact validation order and outcomes,
4. confirmation that `invocation/runtime.rs` remained an explicit `None` caller path,
5. confirmation that production emitters no longer consult PID-based session lookup,
6. confirmation that `docs/TRACE.md` reflects the final suppression contract.

## Assumptions

1. The parent run starts from the current branch baseline `feat/session-centric-state-store`.
2. `PLAN-08` remains a shell-local authority-plumbing slice and does not expand into live-state redesign.
3. The Rust package name `shell` is authoritative for validation commands in this repository.
4. Source-local unit tests that live inside parent-owned production files remain parent-owned even after the production seam stabilizes.
