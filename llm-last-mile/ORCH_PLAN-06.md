# ORCH_PLAN-06: Session-Centric Runtime State Store

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-06.md)  
Execution type: backend and operator-CLI orchestration plan, no UI scope

## Summary

This run executes `PLAN-06` on the current branch `feat/session-centric-state-store` with an exact worker cap of `2`. The parent is the only integrator, the only final branch writer, and the only agent allowed to mutate the overlapping production seams in [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), and [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs). The canonical run-state source of truth is `.runs/plan-06/run-state.json`.
The worker cap is exactly `2` because the only honest parallelism in this slice is the late tests/docs window after Gate C, and there are intentionally zero production-code workers before that because the parent owns every overlapping production seam.

Worktree set for the run:

- `../substrate-m06-contract-tests-docs` on `codex/feat-session-centric-state-store-m06-contract-tests-docs`
- `../substrate-m06-persistence-routing-tests` on `codex/feat-session-centric-state-store-m06-persistence-routing-tests`

The critical path stays parent-owned through these phases:

1. `task/m06-a1-preflight`
2. `task/m06-a2-projection-foundation`
3. `task/m06-b1-consumer-invalidation-cutover`
4. `task/m06-c1-canonical-writer-cutover`
5. `task/m06-e1-integrate-and-validate`
6. `task/m06-e2-closeout`

This is deliberate. `PLAN-06`'s sequencing truth is correct, but the repo seams mean Phase A, Phase B, and Phase C are not real parallel lanes here. The session-record API, tombstone walker, toolbox/session resolution, and canonical writer helpers all converge in `state_store.rs`, and the writer cutover shares a choke point with `persist_runtime_snapshots()` in `async_repl.rs`. Opening workers on those files before the parent freezes each contract would create merge churn and contract drift faster than it would create throughput.

The parent-owned execution brief is:

1. freeze `PLAN-05` semantics as a non-negotiable acceptance gate,
2. land the store-owned session projection and hardened source walkers first,
3. cut `agent status`, `toolbox status`, `toolbox env`, tombstone reads, and invalidation over to the new store contract second,
4. move canonical session-root writes under the existing lifecycle ordering third,
5. launch worker lanes only after production code stabilizes in the parent checkout,
6. integrate worker outputs in one parent-owned validation pass,
7. close the run only if the shell-targeted validation stack and final repo gates are green.

## Hard Guards

### Locked invariants

1. `PLAN-05` semantics are a hard guard for `PLAN-06`, not a best-effort carry-forward.
2. The suppression identity remains `(orchestration_session_id, agent_id, execution.scope)`.
3. `OrchestrationSessionRecord.world_generation` remains the only active-generation authority.
4. `handles/*.json` remains compatibility input only until the parent explicitly removes those reads last.
5. Session grouping becomes store-owned; caller-owned regrouping in `agents_cmd.rs` is forbidden once the cutover lands.
6. `substrate agent status` must preserve one row per live participant, not collapse back to one row per `agent_id`.
7. `substrate agent toolbox status|env` must resolve exactly one live session record and fail closed on ambiguity or incomplete live state.
8. `persist_runtime_snapshots()` remains the write choke point. This slice does not authorize lifecycle redesign or a new transactional layer.
9. Any temporary dual-write is allowed only inside the store-owned writer path, not in callers or tests.
10. Canonical session-root traversal must harden the local trust boundary: no symlink-following, no non-regular file promotion, no implicit nested traversal.
11. `docs/USAGE.md` currently contains stale wording that still claims live discovery is backed by `handles/`; correcting that wording is required for slice completion.
12. No new selector UX such as `--orchestration-session-id` is allowed in this slice.
13. No new cache file, index file, or second authority store is allowed.
14. Crash-window tolerance is part of the contract. Parent-only roots, participant-only roots, and stale active-handle references must degrade safely instead of failing whole commands.

### File-level boundaries

Parent-owned critical overlap surfaces:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Worker-safe late-phase validation and docs surfaces:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Read-for-truth only:

- [llm-last-mile/PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-06.md)
- [llm-last-mile/ORCH_PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-05.md)
- [llm-last-mile/ORCH_PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-04.md)
- [.runs/plan-05/closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-05/closeout.md)

### Non-negotiable stop conditions

Stop the run and write `.runs/plan-06/blocked.json` if any of these occur:

1. A task requires new selector UX or any public CLI expansion beyond `PLAN-06`.
2. A task requires a new index file, cache file, or second authority store to make session-centric reads work.
3. A task restores `handles/*.json` or trace rows as live-state authority.
4. A task cannot preserve the `PLAN-05` suppression tuple or invalidation semantics after regrouping.
5. A task requires concurrent edits to `state_store.rs`, `agents_cmd.rs`, or `async_repl.rs`.
6. A task requires moving write ownership outside `persist_runtime_snapshots()` or inventing a transactional layer.
7. A task leaves `docs/USAGE.md` claiming `handles/` is the authority for live discovery.
8. A task turns normal torn-root states into hard command failures on `status` or `toolbox`.
9. A worker lane needs to touch any parent-owned production file to complete its assignment.

## Orchestration State Surfaces

### Canonical run state

Single local source of truth for the run:

- `.runs/plan-06/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- gate status,
- accepted and rejected worker outputs,
- blocked or completed terminal state,
- final closeout pointer,
- frozen `PLAN-05` guard contract,
- frozen projection/read contract,
- frozen writer-cutover contract.

If a worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/plan-06/queue.json`
- `.runs/plan-06/session.log`
- `.runs/plan-06/sentinels/task-m06-a1-preflight.ok`
- `.runs/plan-06/sentinels/task-m06-a2-projection-foundation.ok`
- `.runs/plan-06/sentinels/task-m06-b1-consumer-invalidation-cutover.ok`
- `.runs/plan-06/sentinels/task-m06-c1-canonical-writer-cutover.ok`
- `.runs/plan-06/sentinels/task-m06-d1-contract-tests-docs.ok`
- `.runs/plan-06/sentinels/task-m06-d2-persistence-routing-tests.ok`
- `.runs/plan-06/sentinels/task-m06-e1-integrate-and-validate.ok`
- `.runs/plan-06/sentinels/task-m06-e2-closeout.ok`
- `.runs/plan-06/blocked.json`
- `.runs/plan-06/closeout.md`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.
5. Worker-generated notes never replace parent-written sentinels.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/session-centric-state-store`.
3. Exact worker cap: `2` active worker lanes.
4. There are no worker lanes during the Phase A, Phase B, or Phase C production cutovers.
5. No two lanes may edit `state_store.rs`, `agents_cmd.rs`, or `async_repl.rs` concurrently.
6. Worker lanes open only after the parent has stabilized the production diff and seeded each worktree from that exact state.
7. `task/m06-d1-contract-tests-docs` and `task/m06-d2-persistence-routing-tests` are the only parallel window in this run.
8. If worker tasks reveal missing support changes in `state_store.rs`, `agents_cmd.rs`, `async_repl.rs`, or any shared helper outside their allowed files, the worker stops and hands the change back to the parent instead of widening scope.

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

- parent re-reads `PLAN-06`, `ORCH_PLAN-05`, `ORCH_PLAN-04`, and `PLAN-05` closeout,
- parent records the exact sequencing truth `A -> B -> C -> D`,
- parent records the authoritative overlap surfaces and worker-safe late-phase files,
- parent records the package-name normalization: use `shell`, never `substrate-shell`.

### Gate B: Projection freeze

Required before consumer cutover starts:

- the `state_store.rs` session-record API is frozen in the parent checkout,
- object-level precedence and torn-root warning behavior are frozen,
- path-hardening behavior is frozen,
- the parent has validated the projection unit stack before touching consumers.

### Gate C: Production stabilization

Required before worker lanes launch:

- Phase B and Phase C production integration are complete in the parent checkout,
- no structural churn remains in `state_store.rs`, `agents_cmd.rs`, or `async_repl.rs`,
- the parent has seeded worker worktrees from the stabilized production state,
- the parent has completed the pre-worker compile/proof checks defined below.

### Gate D: Final acceptance

Required before closeout:

- both worker outputs are accepted or deliberately rejected and replaced by parent work,
- the exact validation command order passes,
- docs reflect the final authority boundary,
- `PLAN-05` invalidation and tombstone semantics still hold under the canonical session-root layout.

## Workstream Plan

### Worktree topology

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Child worktrees and branches:

- `../substrate-m06-contract-tests-docs`
  - `codex/feat-session-centric-state-store-m06-contract-tests-docs`
- `../substrate-m06-persistence-routing-tests`
  - `codex/feat-session-centric-state-store-m06-persistence-routing-tests`

Subagents do not merge each other’s work. They return patches, touched files, tests run, and blockers to the parent.

### Task graph

Execution graph for the run:

1. `task/m06-a1-preflight`
2. `task/m06-a2-projection-foundation`
3. `task/m06-b1-consumer-invalidation-cutover`
4. `task/m06-c1-canonical-writer-cutover`
5. `task/m06-d1-contract-tests-docs` and `task/m06-d2-persistence-routing-tests` in parallel
6. `task/m06-e1-integrate-and-validate`
7. `task/m06-e2-closeout`

Parent-only serialized tasks:

- `task/m06-a1-preflight`
- `task/m06-a2-projection-foundation`
- `task/m06-b1-consumer-invalidation-cutover`
- `task/m06-c1-canonical-writer-cutover`
- `task/m06-e1-integrate-and-validate`
- `task/m06-e2-closeout`

Worker-owned tasks:

- `task/m06-d1-contract-tests-docs`
- `task/m06-d2-persistence-routing-tests`

## Parallel Window D

This is the only worker window in the run.

It opens only after Gate C passes and after the parent has finished the full production-code path through `task/m06-c1-canonical-writer-cutover`. There are zero production-code workers before Gate C because the parent owns the overlapping seams in `state_store.rs`, `agents_cmd.rs`, and `async_repl.rs`, and splitting those files earlier would create merge churn rather than throughput.

### task/m06-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read [PLAN-06.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-06.md), [ORCH_PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-05.md), [ORCH_PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-04.md), and [.runs/plan-05/closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-05/closeout.md).
2. Confirm the run executes from `feat/session-centric-state-store`.
3. Freeze the `PLAN-05` invariants, the `PLAN-06` phase order, the parent-only seam list, and the late worker-safe seam list into the run packet.
4. Initialize:
   - `.runs/plan-06/run-state.json`
   - `.runs/plan-06/queue.json`
   - `.runs/plan-06/session.log`
5. Record the repo-truth validation rule that all package-targeted commands use `-p shell`.

Acceptance:

1. The parent can explain why Phase A, Phase B, and Phase C stay serialized in this repo.
2. The parent can state the `PLAN-05` suppression tuple and active-generation authority without ambiguity.
3. The parent can name the stale `docs/USAGE.md` authority wording that must be removed.
4. `run-state.json` records the initial phase and queue.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-a1-preflight.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json`

### Parent validation gate A

Required before `task/m06-a2-projection-foundation` starts:

1. No invariant contradiction remains unresolved.
2. The parent can explain why caller-owned regrouping is forbidden after the session-record API lands.
3. The parent can explain why `PLAN-05` remains an acceptance gate for `PLAN-06`, not a separate slice.

### task/m06-a2-projection-foundation

Ownership:

- parent only

Why serialized:

- [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) owns the projection contract, source precedence, hardening, torn-root warnings, and discovery rules. There is no safe throughput gain from splitting that file before its API is frozen.

Allowed files:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Scope:

1. Add canonical session-root helpers for:
   - `sessions/<id>/session.json`
   - `sessions/<id>/participants/<participant_id>.json`
   - `sessions/<id>/leases/<participant_id>.lease`
2. Add one hardened participant walker that merges canonical, flat-current, and legacy participant objects by object precedence.
3. Add parent-session loading that prefers canonical `session.json` over flat `sessions/<id>.json`.
4. Add:
   - `load_session()`
   - `list_sessions()`
   - `list_live_sessions()`
   - `resolve_single_live_session_for_agent()`
   - compatibility wrappers only if they reduce diff size
5. Add torn-root warnings and incomplete-root handling.
6. Add session discovery from the union of canonical session roots, flat session files, and participant-derived `orchestration_session_id` values.
7. Add path hardening with `symlink_metadata` and explicit rejection of symlinked or non-regular entries.
8. Add unit coverage for precedence, torn roots, path hardening, and participant-only discovery.

Must not do:

1. No `agents_cmd.rs` consumer cutover yet.
2. No canonical writer cutover yet.
3. No new cache or index file.
4. No caller-owned merge helpers outside the store.

Acceptance:

1. The parent can name one store-owned session-record API surface for live discovery.
2. `list_sessions()` can discover participant-only torn roots.
3. `list_live_sessions()` excludes incomplete roots.
4. Canonical, flat, and legacy precedence is proven in source-local tests.
5. Path-hardening tests prove symlinked or non-regular entries are ignored.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-a2-projection-foundation.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json`

### Parent validation gate B

Required before `task/m06-b1-consumer-invalidation-cutover` starts:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture` passes.
2. The projection/read contract is frozen into `run-state.json`.
3. The parent can state the object-level precedence contract without referring back to callers.

### task/m06-b1-consumer-invalidation-cutover

Ownership:

- parent only

Why serialized:

- [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) currently depends on `list_live_manifests()`, `resolve_live_orchestrator_session()`, and flat tombstone scans that are all being replaced by the same evolving store contract in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs). Splitting this phase across workers would create API churn without real parallelism.

Allowed files:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Scope:

1. Move `build_status_report()` onto `list_live_sessions()`.
2. Preserve one selected status row per live participant.
3. Preserve same-agent concurrent-session visibility instead of collapsing rows by `agent_id`.
4. Move tombstone suppression onto `list_invalidated_participants_across_sources()`.
5. Move invalidation reads onto the same shared participant walker by updating `invalidate_stale_world_members_for_session()`.
6. Restrict trace fallback to gaps that are not already covered by a live record or tombstone.
7. Move `build_toolbox_status_report()` and `build_toolbox_env_report()` onto `resolve_single_live_session_for_agent()`.
8. Preserve the current toolbox JSON fields:
   - `active_orchestration_session_id`
   - `active_world_binding`
   - `dependency_unavailable`
9. Preserve fail-closed ambiguity with operator-readable errors.

Must not do:

1. No canonical writer cutover yet.
2. No `async_repl.rs` lifecycle edits.
3. No public JSON reshaping beyond the plan contract.
4. No regression to `PLAN-05` suppression semantics.

Acceptance:

1. No authoritative live status selection depends on `list_live_manifests()`.
2. No tombstone or invalidation path assumes only flat `participants/*.json`.
3. Toolbox resolution fails closed via session-record selection.
4. Same-agent concurrent sessions stay distinct in status output.
5. The parent can prove the suppression tuple still matches `PLAN-05`.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-b1-consumer-invalidation-cutover.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json`

### task/m06-c1-canonical-writer-cutover

Ownership:

- parent only

Why serialized:

- The writer cutover spans [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) across the same persistence choke point. That is single-writer work.

Allowed files:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Scope:

1. Keep current bootstrap and lifecycle ordering semantics intact.
2. Move parent, participant, and lease writes onto store-owned canonical helpers.
3. Keep `persist_runtime_snapshots()` as the write choke point.
4. Keep any temporary dual-write inside the store only.
5. Preserve mixed-layout read correctness while canonical and flat files coexist.
6. Add crash-window coverage for:
   - parent written, participant missing,
   - participant written, parent missing,
   - active handle points at stale participant.
7. Update any helper readers in `async_repl.rs` that still assume only flat participant layout.

Must not do:

1. No transactional redesign.
2. No scope expansion into new operator UX.
3. No worker dispatch before the parent validates the stabilized production diff.

Acceptance:

1. Canonical writes land under `sessions/<id>/...` through store-owned helpers.
2. Crash-window reads still degrade safely.
3. Mixed canonical and compatibility layouts remain readable by the store.
4. No lifecycle ordering change escapes the write path.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-c1-canonical-writer-cutover.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json`

### Parent validation gate C

Required before the worker window opens:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture` passes again after the writer cutover.
2. `cargo test -p shell --tests --no-run` passes so the late worker packet starts from a compiling production state.
3. The parent seeds both worker worktrees from the exact post-`task/m06-c1` tree.
4. The parent records any still-allowed compatibility bridge in `run-state.json`.

### task/m06-d1-contract-tests-docs

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m06-contract-tests-docs`
- `codex/feat-session-centric-state-store-m06-contract-tests-docs`

Allowed files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Scope:

1. Update contract fixtures and assertions to default to canonical session-root trees.
2. Add contract coverage for:
   - toolbox status resolution from session records,
   - toolbox env resolution from the same selected session,
   - same-agent concurrent session visibility,
   - fail-closed ambiguity,
   - canonical participant precedence over flat compatibility data,
   - flat compatibility fallback when canonical roots are incomplete.
3. Correct the stale `docs/USAGE.md` wording that still says live discovery is backed by `handles/`.
4. Update `docs/TRACE.md` so the live-state authority description matches the final session-root truth plus any bounded compatibility bridge.

Must not do:

1. No edits to `state_store.rs`, `agents_cmd.rs`, or `async_repl.rs`.
2. No edits to any other tests.
3. No new scope beyond the session-centric store slice.

Commands:

1. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`

Acceptance:

1. `docs/USAGE.md` no longer names `handles/` as the live authority.
2. Contract tests assert session-record-driven status and toolbox behavior.
3. The worker report includes exact tests run and any remaining doc ambiguities.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-d1-contract-tests-docs.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json` if the worker needs parent-owned files

### task/m06-d2-persistence-routing-tests

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m06-persistence-routing-tests`
- `codex/feat-session-centric-state-store-m06-persistence-routing-tests`

Allowed files:

- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Scope:

1. Add integration coverage for canonical session-root persistence.
2. Add coverage that `PLAN-05` invalidation still works after participant location changes.
3. Add coverage that tombstone suppression still wins after the consumer regrouping cutover.
4. Add coverage that same-agent concurrent world rows stay isolated across session ids.
5. Add externally visible crash-window coverage where the named integration tests can prove safe degradation.

Must not do:

1. No edits to helper readers in `async_repl.rs`.
2. No edits to `state_store.rs` or `agents_cmd.rs`.
3. No expansion into unrelated integration suites.

Commands:

1. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`
2. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

Acceptance:

1. The named integration suites cover persistence and routing behavior under the canonical layout.
2. The worker report makes any unmet helper dependency explicit instead of silently widening scope.
3. `PLAN-05` invalidation expectations are preserved in the updated tests.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-d2-persistence-routing-tests.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json` if the worker needs parent-owned files

### task/m06-e1-integrate-and-validate

Ownership:

- parent only

Scope:

1. Review both worker outputs against the frozen contracts.
2. Reject any worker patch that touches unowned files, assumes stale authority, or omits test evidence.
3. Integrate accepted worker outputs into the parent checkout.
4. If worker outputs expose missing parent-owned support changes, make those changes in the parent checkout and rerun the affected targeted validation steps.
5. Run the final validation stack in the exact order defined below.

Acceptance:

1. Every accepted worker output has a matching parent-written sentinel.
2. No stale `handles/` authority wording remains in docs.
3. The full validation order passes.
4. `run-state.json` records accepted and rejected outputs explicitly.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-e1-integrate-and-validate.ok`

Blocked-path output:

- `.runs/plan-06/blocked.json`

### task/m06-e2-closeout

Ownership:

- parent only

Scope:

1. Mark the run complete in `.runs/plan-06/run-state.json`.
2. Write `.runs/plan-06/closeout.md` with:
   - accepted tasks,
   - integrated worktrees and branches,
   - validation commands and outcomes,
   - any compatibility behavior intentionally retained,
   - any explicit removals of legacy `handles/*.json` reads.
3. Append the final acceptance rationale to `.runs/plan-06/session.log`.

Acceptance:

1. `closeout.md` is present and matches the final branch state.
2. All sentinels through `task/m06-e2-closeout` exist.
3. No blocked-path artifact exists for a successful run.

Green-path output:

- `.runs/plan-06/sentinels/task-m06-e2-closeout.ok`

## Context-Control Rules

1. The parent owns `.runs/plan-06/*`. Workers do not edit run-state, sentinels, queue, session log, blocked state, or closeout artifacts.
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
   - whether they observed any stale authority wording or compatibility assumptions.
5. Workers do not rebase, merge, or integrate each other’s work.
6. The parent rejects any worker patch that silently broadens scope or makes compatibility policy decisions not already frozen in `run-state.json`.
7. If worker outputs conflict with current parent truth, the parent re-derives the correct result from production code and rewrites the patch locally instead of negotiating a blended state across worktrees.

## Tests And Acceptance

### Pre-worker proof order

Run these parent-owned checks before dispatching the worker window:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture`
2. `cargo test -p shell --tests --no-run`

### Final validation order

Run these commands in this exact order during `task/m06-e1-integrate-and-validate`:

1. `cargo test -p shell agent_runtime::state_store -- --nocapture`
2. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
3. `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`
4. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
5. `cargo test -p shell -- --nocapture`
6. `cargo fmt --all -- --check`
7. `cargo clippy -p shell --all-targets -- -D warnings`
8. `cargo clippy --workspace --all-targets -- -D warnings`
9. `cargo test --workspace -- --nocapture`

### Acceptance checklist

The slice is accepted only if all of these are true:

1. The store can answer `load_session(<id>)` and `list_live_sessions()` directly.
2. `agent status` reads live rows from store-owned session records.
3. `toolbox status|env` resolves exactly one live session record and fails closed on ambiguity.
4. `PLAN-05` invalidation and tombstone suppression remain correct under the canonical session-root layout.
5. Crash-window reads remain safe.
6. Docs reflect the current authority boundary.
7. No caller-owned regrouping or stale `handles/` authority survives the cutover.

## Merge Refusal Rules

The parent refuses to merge a worker output if any of these are true:

1. The patch edits a file outside the task’s allowed file list.
2. The patch assumes `handles/*.json` is authoritative for live discovery.
3. The patch weakens `PLAN-05` suppression, invalidation, or same-agent concurrent-session visibility.
4. The patch requires concurrent parent edits to `state_store.rs`, `agents_cmd.rs`, or `async_repl.rs` to become intelligible.
5. The patch omits test evidence for the behavior it claims to cover.
6. The patch leaves doc wording or fixture assumptions in a state that contradicts the final production code.

## Run Exit Criteria

### Successful run

The run is successful only if all of these are true:

1. `.runs/plan-06/sentinels/task-m06-a1-preflight.ok` exists.
2. `.runs/plan-06/sentinels/task-m06-a2-projection-foundation.ok` exists.
3. `.runs/plan-06/sentinels/task-m06-b1-consumer-invalidation-cutover.ok` exists.
4. `.runs/plan-06/sentinels/task-m06-c1-canonical-writer-cutover.ok` exists.
5. `.runs/plan-06/sentinels/task-m06-d1-contract-tests-docs.ok` exists.
6. `.runs/plan-06/sentinels/task-m06-d2-persistence-routing-tests.ok` exists.
7. `.runs/plan-06/sentinels/task-m06-e1-integrate-and-validate.ok` exists.
8. `.runs/plan-06/sentinels/task-m06-e2-closeout.ok` exists.
9. `.runs/plan-06/run-state.json` exists and records a completed terminal state.
10. `.runs/plan-06/queue.json` and `.runs/plan-06/session.log` exist.
11. `.runs/plan-06/closeout.md` exists and matches the final accepted branch state.
12. `.runs/plan-06/blocked.json` does not exist.

### Blocked termination

The run terminates blocked only if all of these are true:

1. `.runs/plan-06/blocked.json` exists and records the blocking reason.
2. `.runs/plan-06/run-state.json` exists and records `blocked` as the terminal state.
3. `session.log` contains the parent rationale for the stop.
4. No downstream task sentinel may be written after the blocking point.
5. `.runs/plan-06/sentinels/task-m06-e2-closeout.ok` must not exist.
6. `.runs/plan-06/closeout.md` must not exist.

## Closeout

Successful closeout records:

1. the final frozen contract actually shipped,
2. the exact validation order and outcomes,
3. whether compatibility reads from `handles/*.json` remain and why,
4. confirmation that the stale `docs/USAGE.md` authority wording is gone,
5. confirmation that `PLAN-05` still governs invalidation and tombstone semantics after the session-centric cutover.

## Assumptions

1. The parent run starts from the current branch baseline `feat/session-centric-state-store`.
2. `PLAN-05` is already the upstream invalidation and tombstone contract and must be preserved rather than redesigned here.
3. The Rust package name `shell` is authoritative for validation commands in this repository.
4. A bounded read-compatibility bridge around `handles/*.json` may remain temporarily during migration, but only as compatibility input inside the store and never as live-state authority.
