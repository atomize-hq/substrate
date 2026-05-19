# ORCH_PLAN-05: Restart Invalidation Semantics for Live State

Branch: `feat/restart-invalidation-semantics`  
Plan source: [PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md)  
Execution type: backend-only orchestration plan, no UI scope

## Summary

This run executes `PLAN-05` on the current branch `feat/restart-invalidation-semantics` with an exact concurrency cap of `2` worker lanes. The parent is the only integrator and the only final branch writer. The canonical run-state source of truth is `.runs/plan-05/run-state.json`.

Worktree set for the run:

- `../substrate-m05-status-suppression` on `codex/feat-restart-invalidation-semantics-m05-status-suppression`
- `../substrate-m05-contract-tests-docs` on `codex/feat-restart-invalidation-semantics-m05-contract-tests-docs`
- `../substrate-m05-restart-tests` on `codex/feat-restart-invalidation-semantics-m05-restart-tests`

The critical path stays parent-owned through these phases:

1. `task/m05-a1-preflight`
2. `task/m05-a2-foundation`
3. `task/m05-b2-restart-integration`
4. `task/m05-d1-integrate-and-validate`
5. `task/m05-d2-closeout`

This is deliberate. The real overlap surfaces are [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), and [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs). False concurrency across those files would create merge churn and contract drift faster than it would create throughput.

The parent-owned execution brief is:

1. freeze the authority and invalidation contract from `PLAN-05`,
2. land the bounded invalidation primitive and session-local sweep without pre-spending `PLAN-06`,
3. integrate restart ordering so stale generation invalidates before replacement publication becomes live,
4. accept a bounded worker change for status suppression only after foundation is frozen,
5. launch tests and docs only after production behavior stabilizes,
6. integrate all worker outputs in one parent-owned validation phase,
7. close the run only if all sentinels are present and the full validation stack is green.

## Orchestration State Surfaces

### Canonical run state

Single local source of truth for the run:

- `.runs/plan-05/run-state.json`

Parent-only writes to this file. It tracks:

- current phase,
- active task IDs,
- branch and worktree assignment,
- accepted worker outputs,
- gate status,
- blocked or completed terminal state,
- final closeout pointer.

If a worker report conflicts with `run-state.json`, the parent trusts `run-state.json` until it explicitly reconciles the discrepancy.

### Derived run artifacts

The parent may maintain these local artifacts:

- `.runs/plan-05/queue.json`
- `.runs/plan-05/session.log`
- `.runs/plan-05/sentinels/task-m05-a1-preflight.ok`
- `.runs/plan-05/sentinels/task-m05-a2-foundation.ok`
- `.runs/plan-05/sentinels/task-m05-b1-status-suppression.ok`
- `.runs/plan-05/sentinels/task-m05-b2-restart-integration.ok`
- `.runs/plan-05/sentinels/task-m05-c1-contract-tests-docs.ok`
- `.runs/plan-05/sentinels/task-m05-c2-restart-tests.ok`
- `.runs/plan-05/sentinels/task-m05-d1-integrate-and-validate.ok`
- `.runs/plan-05/sentinels/task-m05-d2-closeout.ok`
- `.runs/plan-05/blocked.json`
- `.runs/plan-05/closeout.md`

Sentinel rules:

1. `.ok` means the parent validated the task output and advanced the run.
2. Missing sentinel means the task is not accepted.
3. `blocked.json` is written only on blocked termination.
4. `closeout.md` is written only on successful completion.

## Concurrency Policy

1. The parent is the only integrator.
2. The parent is the only writer of final branch state on `feat/restart-invalidation-semantics`.
3. Exact worker cap: `2` active worker lanes.
4. The parent owns the critical overlap surfaces first:
   - [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   - [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
5. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) does not open as a worker lane until the parent has frozen the invalidation helper and state-store contract.
6. No two lanes may edit `async_repl.rs`, `agents_cmd.rs`, or `state_store.rs` concurrently.
7. Test and docs worktrees launch only after production integration stabilizes in the parent checkout.
8. Safe parallel windows:
   - Window B: parent restart integration plus one worker on `agents_cmd.rs`
   - Window C: two worker lanes on tests and docs after parent production validation passes

## Hard Guards

### Locked invariants

1. `participants/*.json` and `sessions/*.json` are the authoritative runtime store.
2. `handles/*.json` remains compatibility input only; this slice must not restore it as runtime authority.
3. The active generation source is the parent `OrchestrationSessionRecord.world_generation` from `PLAN-04`.
4. `PLAN-05` consumes that generation and must not assign it from any new store.
5. Only `role=member` plus `execution.scope=world` participants are invalidated by generation rollover.
6. Restart ordering must prefer fail-closed absence over stale presence.
7. `Invalidated` rows are tombstones for live surfaces and must suppress trace fallback.
8. Trace fallback suppression identity is `(orchestration_session_id, agent_id, execution.scope)`.
9. Persisted lineage naming stays `resumed_from_participant_id`.
10. `PLAN-05` must not pre-spend `PLAN-06` with a grouped registry rewrite or new active-generation index file.
11. `status` remains a live-state surface, not a historical reconstruction surface.
12. Docs named in `PLAN-05` are part of the slice.

### File-level boundaries

Parent-owned critical overlap surfaces:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Worker-safe implementation surface after foundation freeze:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Validation and docs surfaces:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)

Read-for-validation only:

- [llm-last-mile/PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md)
- [llm-last-mile/ORCH_PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-04.md)

### Non-negotiable stop conditions

Stop the run and write `blocked.json` if any of these occur:

1. A task requires a new session-grouped registry layout under `run/agent-hub/sessions/...`.
2. A task requires a new active-generation index file to make `PLAN-05` work.
3. A task makes `handles/*.json` or trace authoritative for live state.
4. A task preserves `(agent_id, role)` as the suppression boundary.
5. A task cannot invalidate stale generation before replacement publication without redesigning the slice.
6. A task keeps stale generation visible "for continuity" during replacement failure.
7. A task needs concurrent edits to `async_repl.rs` and `agents_cmd.rs` before the parent freezes the shared contract.
8. A task changes public status JSON shape beyond `PLAN-05` requirements.
9. A task renames persisted lineage fields away from `resumed_from_participant_id`.

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

- parent re-reads `PLAN-05` and `ORCH_PLAN-04`,
- parent records the invariant list,
- parent records the exact files that belong to the slice.

### Gate B: Foundation freeze

Required before any worker edits `agents_cmd.rs`:

- invalidation helper contract is fixed,
- session-local sweep contract is fixed,
- parent has proven no `PLAN-06` storage work is needed,
- worker prompts contain exact allowed files and stop conditions.

### Gate C: Production stabilization

Required before test/docs worktrees launch:

- parent production integration is complete,
- no structural churn remains in `session.rs`, `state_store.rs`, `async_repl.rs`, or `agents_cmd.rs`,
- parent targeted validation passes in the defined order.

### Gate D: Final acceptance

Required before closeout:

- targeted tests pass,
- shell crate regression passes,
- formatting and clippy gates pass,
- workspace regression passes,
- docs reflect the final contract.

## Workstream Plan

### Worktree topology

Common naming pattern:

- relative worktree path: `../substrate-m05-<lane>`
- worker branch name: `codex/feat-restart-invalidation-semantics-m05-<lane>`

Concrete worktrees for this run:

- `../substrate-m05-status-suppression` on `codex/feat-restart-invalidation-semantics-m05-status-suppression`
- `../substrate-m05-contract-tests-docs` on `codex/feat-restart-invalidation-semantics-m05-contract-tests-docs`
- `../substrate-m05-restart-tests` on `codex/feat-restart-invalidation-semantics-m05-restart-tests`

Parent checkout:

- current checkout on `feat/restart-invalidation-semantics`

Subagents do not merge each other’s work. They return patches, touched files, tests run, and blockers to the parent.

### Task index

Parent-only serialized tasks:

- `task/m05-a1-preflight`
- `task/m05-a2-foundation`
- `task/m05-b2-restart-integration`
- `task/m05-d1-integrate-and-validate`
- `task/m05-d2-closeout`

Worker-owned tasks:

- `task/m05-b1-status-suppression`
- `task/m05-c1-contract-tests-docs`
- `task/m05-c2-restart-tests`

### task/m05-a1-preflight

Ownership:

- parent only

Scope:

1. Re-read [PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md) and [ORCH_PLAN-04.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-04.md).
2. Freeze the authoritative-store rules, restart-ordering rule, suppression identity, and `PLAN-06` deferrals into the worker prompt packet.
3. Initialize:
   - `.runs/plan-05/run-state.json`
   - `.runs/plan-05/queue.json`
   - `.runs/plan-05/session.log`
4. Confirm the slice remains backend-only.
5. Confirm the primary seam list:
   - invalidation helper in `session.rs`
   - session-local sweep in `state_store.rs`
   - restart ordering in `async_repl.rs`
   - status suppression in `agents_cmd.rs`
   - tests and docs named in `PLAN-05`

Acceptance:

1. Parent can state the active-generation authority source.
2. Parent can state the exact suppression tuple.
3. Parent can state why grouped-registry rewrite is forbidden in this slice.
4. `run-state.json` records the initial phase and queue.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-a1-preflight.ok`

Blocked-path output:

- `.runs/plan-05/blocked.json`

### Parent validation gate A

Required before `task/m05-a2-foundation` starts:

1. No invariant contradiction remains unresolved.
2. The parent can explain why fail-closed absence is safer than stale presence.
3. The parent can explain why `PLAN-04` remains the only active-generation authority.

### task/m05-a2-foundation

Ownership:

- parent only

Why serialized:

- `session.rs`, `state_store.rs`, and `async_repl.rs` share the invalidation contract. Freezing the helper and sweep semantics in the parent avoids false concurrency and merge churn.

Allowed files:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Scope:

1. Add one explicit participant invalidation helper using existing `Invalidated`.
2. Add one bounded session-local invalidation sweep keyed by:
   - `orchestration_session_id`
   - `role=member`
   - `execution.scope=world`
   - `world_generation < active_generation`
   - currently live state only
3. Keep the sweep idempotent.
4. Keep host-scoped and already-invalidated rows untouched.
5. Freeze any state-store read helpers needed later by `agents_cmd.rs` and `async_repl.rs`.
6. Add or update unit coverage in the touched source files for:
   - invalidation metadata stamping,
   - session-local scope,
   - host-row exclusion,
   - idempotence.

Must not do:

1. No new state enum variant.
2. No new index file.
3. No directory-layout rewrite.
4. No changes to `list_live_manifests()` semantics beyond consuming existing authoritative-live behavior.

Acceptance:

1. A single invalidation helper exists for generation rollover semantics.
2. A single session-local sweep exists and returns invalidated participant IDs for logging and assertions.
3. Re-running the sweep is harmless.
4. The parent can now hand workers a frozen contract for status and restart integration.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-a2-foundation.ok`

Blocked-path output:

- `.runs/plan-05/blocked.json`

### Parent validation gate B

Required before `task/m05-b1-status-suppression` opens:

1. Foundation code is integrated in the parent checkout.
2. Helper signatures are stable enough for worker consumption.
3. No `PLAN-06` work was introduced.
4. `run-state.json` records the frozen foundation contract.

### Parallel window B

This is the first real concurrency window. It opens only after `task/m05-a2-foundation` is accepted.

### task/m05-b1-status-suppression

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m05-status-suppression`
- `codex/feat-restart-invalidation-semantics-m05-status-suppression`

Allowed files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Scope:

1. Replace the trace-fallback suppression key with `(orchestration_session_id, agent_id, execution.scope)`.
2. Build suppression identities from authoritative live rows first.
3. Build suppression identities from invalidated world-member tombstones on disk second.
4. Suppress trace fallback on either match.
5. Keep host-orchestrator selected-row behavior unchanged.
6. Keep public status JSON shape unchanged.

Must not do:

1. No edits to `state_store.rs` or `async_repl.rs`.
2. No public shape redesign for `agent status --json`.
3. No fallback to `(agent_id, role)`.
4. If the worker discovers it needs new helper signatures, broader state-store API changes, or additional authority-surface changes outside `agents_cmd.rs`, it must stop immediately, report the required delta, and bounce the work back to the parent instead of expanding scope.

Acceptance:

1. Worker output stays inside `agents_cmd.rs`.
2. Suppression semantics are session-aware and world-scope-aware.
3. The selected host row contract remains unchanged.
4. Any discovered dependency on new helper or API shape is escalated back to the parent rather than worked around in-lane.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-b1-status-suppression.ok` after parent review

Blocked-path output:

- no sentinel
- rejection entry in `.runs/plan-05/session.log` if the patch requires broader authority changes

### task/m05-b2-restart-integration

Ownership:

- parent only

Allowed files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) only for bounded call-site glue already implied by `task/m05-a2-foundation`

Scope:

1. Keep the `PLAN-04` parent-binding barrier.
2. After generation `G+1` is accepted and persisted on the parent, invalidate stale generation `G` members before replacement publication is considered complete.
3. Allow temporary absence.
4. Forbid temporary stale-live visibility.
5. Emit restart success only after:
   - parent binding persists,
   - stale member generation is invalidated,
   - replacement publication succeeds, or the path intentionally remains fail closed.
6. Keep trace non-authoritative for live state.

Must not do:

1. No replacement success publication before stale generation is dead.
2. No "continuity" path that leaves stale members live while replacement is missing.
3. No new authority store.

Acceptance:

1. Crash or injected failure after invalidation but before replacement leaves absence, not stale liveness.
2. Dual-live generations are impossible on the happy path.
3. Parent-owned restart sequencing is now stable enough for test worktrees.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-b2-restart-integration.ok`

Blocked-path output:

- `.runs/plan-05/blocked.json`

### Parent validation gate C

Required before any test/docs worktree launches:

1. `task/m05-b1-status-suppression` is accepted.
2. `task/m05-b2-restart-integration` is complete in the parent checkout.
3. No further structural API churn is expected in production files.
4. Parent targeted validation passes in the defined order.

### Parallel window C

This window opens only after gate C passes.

### task/m05-c1-contract-tests-docs

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m05-contract-tests-docs`
- `codex/feat-restart-invalidation-semantics-m05-contract-tests-docs`

Allowed files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md)

Read-for-validation only:

- [llm-last-mile/PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md)

Scope:

1. Add contract coverage for:
   - tombstone suppression beating stale trace fallback,
   - no-replacement-yet omission from live status,
   - same-agent concurrent sessions remaining independently visible,
   - `resumed_from_participant_id` persisted lineage assertions.
2. Update docs so they say:
   - `participants/*.json` is the authority path,
   - tombstones beat trace for live-state selection,
   - persisted lineage uses `resumed_from_participant_id`.
3. Keep compatibility-path references bounded and clearly secondary.

Must not do:

1. No silent rewrite of `PLAN-05`.
2. No new public status fields.
3. No shift back to `handles/*.json`-first fixtures.

Acceptance:

1. Contract tests cover the suppression tuple and tombstone rules.
2. Docs match the final runtime contract.
3. Worker stays within the allowed files.
4. New fixtures remain `participants/*.json`-first.
5. Any remaining `handles/*.json` references are compatibility-only and are not used as the primary authority path in new assertions.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-c1-contract-tests-docs.ok` after parent review

Blocked-path output:

- no sentinel
- escalation if docs and finished runtime behavior disagree

### task/m05-c2-restart-tests

Ownership:

- worker-owned
- parent-reviewed

Worktree:

- `../substrate-m05-restart-tests`
- `codex/feat-restart-invalidation-semantics-m05-restart-tests`

Allowed files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Scope:

1. Add integration coverage proving stale member generation invalidates before restart success publication.
2. Add fail-closed coverage proving injected failure between invalidation and replacement leaves the member absent.
3. Add replacement coverage proving the new generation becomes the only live generation after full restart.
4. Keep assertions tied to the real runtime ordering, not mocked abstractions.

Must not do:

1. No production-code edits unless the parent explicitly reopens the task as a repair lane.
2. No contract broadening beyond `PLAN-05`.

Acceptance:

1. Integration tests fail if publish-before-invalidate regresses.
2. Integration tests fail if a missing replacement still leaves stale liveness visible.
3. Worker provides exact tests added and exact commands run.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-c2-restart-tests.ok` after parent review

Blocked-path output:

- no sentinel
- escalation back to parent if production behavior, not only tests, must change

### task/m05-d1-integrate-and-validate

Ownership:

- parent only

Scope:

1. Merge accepted foundation, production, status, test, and docs outputs into the parent checkout in this order:
   - parent-owned `task/m05-a2-foundation`
   - accepted worker output from `task/m05-b1-status-suppression`
   - parent-owned `task/m05-b2-restart-integration`
   - accepted worker output from `task/m05-c1-contract-tests-docs`
   - accepted worker output from `task/m05-c2-restart-tests`
2. Reconcile any patch overlap in the parent checkout without delegating merge resolution back to workers.
3. Run the exact validation stack in the required order.
4. Record accepted worker branches, rejected worker branches, and any manual parent reconciliation in `.runs/plan-05/session.log`.
5. Refuse integration if competing outputs imply different interpretations of `PLAN-05`.

Merge refusal rules:

1. If a worker output implies `handles/*.json` is authoritative, do not merge it.
2. If a worker output requires new helper signatures or broader state-store changes not frozen at gate B, do not merge it; reopen parent planning instead.
3. If worker outputs disagree about the suppression tuple, do not merge either interpretation until the parent resolves the contract explicitly.
4. If test assertions only pass by weakening fail-closed behavior or tolerating stale liveness, do not merge them.
5. If docs describe a different authority path or lineage field than the code now enforces, stop and reconcile before merging.

Exact validation command ordering:

```bash
cargo test -p substrate-shell agent_runtime::state_store -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-shell -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Acceptance:

1. All accepted worker outputs are integrated or explicitly rejected by parent decision.
2. The validation stack passes in the exact order above.
3. The parent can state one coherent `PLAN-05` interpretation across runtime, status, tests, and docs.
4. No active `blocked.json` remains.

Green-path output:

- `.runs/plan-05/sentinels/task-m05-d1-integrate-and-validate.ok`

Blocked-path output:

- `.runs/plan-05/blocked.json`
- rejection details appended to `.runs/plan-05/session.log`

### task/m05-d2-closeout

Ownership:

- parent only

Scope:

1. Verify all accepted task sentinels are present.
2. Verify `task/m05-d1-integrate-and-validate` completed and its sentinel exists.
3. Confirm no blocked artifact remains active.
4. Write `.runs/plan-05/closeout.md`.
5. Mark `run-state.json` as `completed`.

Acceptance:

1. The slice is complete by the final acceptance checklist.
2. The orchestration artifacts show green termination rather than partial progress.
3. The closeout names any intentional deferrals to `PLAN-06`.

Green-path output:

- `.runs/plan-05/closeout.md`
- `.runs/plan-05/sentinels/task-m05-d2-closeout.ok`

Blocked-path output:

- `.runs/plan-05/blocked.json`

## Integration Procedure

### Integration order

1. Complete `task/m05-a1-preflight`.
2. Complete `task/m05-a2-foundation`.
3. Launch `task/m05-b1-status-suppression`.
4. In parallel, complete `task/m05-b2-restart-integration` in the parent checkout.
5. Review and accept or reject `task/m05-b1-status-suppression`.
6. Pass gate C.
7. Launch in parallel:
   - `task/m05-c1-contract-tests-docs`
   - `task/m05-c2-restart-tests`
8. Review and accept or reject both worker outputs.
9. Complete `task/m05-d1-integrate-and-validate`.
10. Complete `task/m05-d2-closeout`.

### Parent post-integration validation ordering

The parent does not treat procedural integration as complete until `task/m05-d1-integrate-and-validate` succeeds.

Validation order inside that task is fixed:

```bash
cargo test -p substrate-shell agent_runtime::state_store -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-shell -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Integration refusal rule:

- If two outputs require different interpretations of `PLAN-05`, neither merges until the parent resolves the contract mismatch and records the decision in `.runs/plan-05/session.log`.

## Context-Control Rules

1. The parent keeps only invariants, open tasks, gates, and blockers in working memory.
2. Every worker prompt must include:
   - exact task ID,
   - allowed files,
   - forbidden files,
   - acceptance criteria,
   - stop conditions,
   - expected sentinel name.
3. Workers return:
   - touched files,
   - concise rationale,
   - tests run,
   - blockers or uncertainty.
4. Workers do not paste large file dumps back to the parent.
5. The parent re-reads [PLAN-05.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md) before merging any lane that affects restart ordering or status suppression.
6. `async_repl.rs`, `state_store.rs`, and `session.rs` are single-writer parent-owned during production integration.
7. `agents_cmd.rs` is single-writer worker-owned only after gate B; the parent does not concurrently reshape its contract.
8. Test workers do not relax assertions to match drift.
9. Parent updates `run-state.json` and `session.log` at every gate transition, task acceptance, rejection, integration decision, or blocked termination.

## Tests And Acceptance

### Required targeted commands

These commands map directly to the files and behaviors named in `PLAN-05`.

1. Foundation sweep and runtime-state unit coverage in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs):

```bash
cargo test -p substrate-shell agent_runtime::state_store -- --nocapture
```

2. Status and successor contract validation in [agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs):

```bash
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
```

3. Restart ordering validation in [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs):

```bash
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
```

4. Full shell crate regression:

```bash
cargo test -p substrate-shell -- --nocapture
```

5. Repo gates:

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
| Authority gate | `participants/*.json` and `sessions/*.json` remain authoritative; no `PLAN-06` storage work leaks in | `session.rs`, `state_store.rs` |
| Invalidation gate | stale world members in the same orchestration session become `Invalidated` and stay tombstoned | `session.rs`, `state_store.rs` |
| Suppression gate | trace fallback suppresses by `(orchestration_session_id, agent_id, execution.scope)` using live rows and tombstones | `agents_cmd.rs`, `agent_successor_contract_ahcsitc0.rs` |
| Isolation gate | same `agent_id` across concurrent sessions does not cross-suppress | `agents_cmd.rs`, `agent_successor_contract_ahcsitc0.rs` |
| Restart ordering gate | parent binding persists, stale generation invalidates, then replacement publication becomes visible | `async_repl.rs`, `repl_world_first_routing_v1.rs` |
| Fail-closed gate | missing replacement yields absence, not stale liveness | `async_repl.rs`, `repl_world_first_routing_v1.rs` |
| Lineage gate | persisted lineage is asserted as `resumed_from_participant_id` | `agent_successor_contract_ahcsitc0.rs`, successor protocol doc |
| Docs gate | trace and protocol docs describe tombstone-beats-trace and authority-path terminology | `docs/TRACE.md`, successor protocol doc |

### Final acceptance checklist

1. `PLAN-04` parent session `world_generation` remains the only active-generation source.
2. Every older world-scoped member in the same orchestration session becomes `Invalidated` after rollover.
3. Live reads no longer surface stale-generation world members after invalidation commits.
4. `agent status --json` suppresses stale trace fallback when a matching tombstone exists.
5. Same-agent concurrent sessions remain independently visible.
6. Restart success cannot surface before stale generation is dead.
7. Replacement failure leaves absence, not stale presence.
8. `resumed_from_participant_id` remains the persisted lineage field name.
9. `PLAN-06` storage work remains deferred.
10. The docs named in `PLAN-05` match the final code contract.

## Run Exit Criteria

### Successful run

The run is complete only when all of these are true:

1. Updated production code exists only across the intended slice surfaces:
   - `session.rs`
   - `state_store.rs`
   - `agents_cmd.rs`
   - `async_repl.rs`
2. Updated tests exist in:
   - `agent_successor_contract_ahcsitc0.rs`
   - `repl_world_first_routing_v1.rs`
   - source-adjacent unit tests where foundation code was changed
3. Updated docs exist in:
   - `docs/TRACE.md`
   - successor protocol doc named in `PLAN-05`
4. All worker sentinels are present:
   - `task-m05-b1-status-suppression.ok`
   - `task-m05-c1-contract-tests-docs.ok`
   - `task-m05-c2-restart-tests.ok`
5. The parent integration sentinel is present:
   - `task-m05-d1-integrate-and-validate.ok`
6. The closeout sentinel is present:
   - `task-m05-d2-closeout.ok`
7. The full validation stack is green in the required order.
8. No active `.runs/plan-05/blocked.json` remains.
9. `.runs/plan-05/closeout.md` exists and summarizes:
   - accepted task IDs,
   - integrated worktrees and branches,
   - final validation commands and outcomes,
   - intentional deferrals to `PLAN-06`.
10. `.runs/plan-05/run-state.json` marks the run `completed`.

### Blocked termination

If the run cannot proceed without violating a hard guard:

1. parent writes `.runs/plan-05/blocked.json`,
2. parent records the blocking task ID, invariant, and unresolved conflict in `.runs/plan-05/session.log`,
3. parent marks `run-state.json` as `blocked`,
4. parent does not write `task-m05-d1-integrate-and-validate.ok`,
5. parent does not write `task-m05-d2-closeout.ok`.

## Assumptions

1. `PLAN-04` is already landed on this branch or otherwise available as the runtime authority model consumed by this slice.
2. The implementation branch for this work is `feat/restart-invalidation-semantics`, and worker branches layer on top of it rather than redefining scope.
3. This slice remains backend-only. No UI or transport redesign work is required.
4. `participants/*.json` and `sessions/*.json` are stable enough to carry the invalidation contract without a storage migration.
5. The docs named in `PLAN-05` are the only required documentation updates for this slice unless implementation reveals a direct contradiction.
6. If finished code conflicts with `PLAN-05` on any locked invariant, the correct action is to stop and reconcile the packet rather than stretch the slice into `PLAN-06`.
