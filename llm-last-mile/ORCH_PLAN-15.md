# ORCH_PLAN-15: Execute PLAN-15 Through A Parent-Frozen Submit-Turn Contract And One Honest Two-Lane Parallel Window

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Execution type: fresh orchestration plan, Linux-first, REPL-targeted agent turns, shell + world-agent + typed local transport, parent-owned contract freeze with one real parallel window

## Summary

This document is the execution controller for `PLAN-15`, not a restatement of it.

The run shape is fixed before any worker starts:

1. `task/m15-p1-parent-contract-freeze-and-run-init` is parent-only and freezes the submit-turn contract, launch-only boundaries, run-state surface, and worker ownership.
2. `task/m15-g1-worker-launch-gate` is parent-only and is the only gate that may launch parallel workers.
3. The only honest parallel window is exactly two code lanes after `p1`:
   - `task/m15-l1-shell-parser-selector-host-submit`
   - `task/m15-l2-world-submit-route-and-member-registry`
4. `task/m15-g2-code-lane-integration-gate` and `task/m15-p2-parent-code-lane-integration` are parent-only.
5. Tests, stubs, docs, and gap-matrix closeout do not run in parallel with the code lanes. They start only after `p2` lands final merged code truth.
6. `task/m15-l3-tests-stubs-docs-closeout` is a single sequential worker lane on top of the integrated code.
7. `task/m15-g4-validation-wall-gate` and `task/m15-p3-parent-validation-wall-and-closeout` are parent-only and finish the run.

Canonical task IDs:

- `task/m15-p1-parent-contract-freeze-and-run-init`
- `task/m15-g1-worker-launch-gate`
- `task/m15-l1-shell-parser-selector-host-submit`
- `task/m15-l2-world-submit-route-and-member-registry`
- `task/m15-g2-code-lane-integration-gate`
- `task/m15-p2-parent-code-lane-integration`
- `task/m15-g3-closeout-launch-gate`
- `task/m15-l3-tests-stubs-docs-closeout`
- `task/m15-g4-validation-wall-gate`
- `task/m15-p3-parent-validation-wall-and-closeout`

## Assumptions

1. `PLAN-15.md` remains the authoritative dependency graph for this run.
2. The parent already owns the authoritative checkout on `feat/session-centric-state-store`.
3. The new submit-turn request and client method can be frozen in `p1` without requiring shell or world-agent file edits in the same phase.
4. Linux is the only world-target platform that must go green in this run; non-Linux world targeting may fail closed.
5. Existing Rust test harnesses named in `PLAN-15` are still the correct validation seams for parser, world submit routing, member registry behavior, and wrap regression.
6. The shell continues to own orchestration session truth; `world-agent` continues to own retained world runtime control after bootstrap.

## Immutable Run Shape

### Frozen Contract Truth

These are run-stopping invariants, not preferences:

1. The only targeted-turn grammar is `::<backend_id> <prompt>`.
2. `:host` and `:pty` retain their existing meanings.
3. Plain REPL input remains shell execution.
4. `substrate -c` remains `ShellMode::Wrap`.
5. `ExecuteRequest.member_dispatch` remains launch-only for world-member bootstrap.
6. World follow-up turns use a new typed submit route and do not overload `member_dispatch`.
7. Host follow-up turns may target only the active orchestrator backend for the current REPL session.
8. World follow-up turns are Linux-first and must use the new typed submit route.
9. One retained world member at a time is the product boundary for this run.
10. Bootstrap spans and submitted-turn spans remain distinct and cancel independently.
11. Routing is by exact `backend_id`, never by "the one eligible member".
12. Submitted turns reuse surfaced `internal.uaa_session_id`; they do not replay bootstrap prompts.

### Parent-Only Versus Worker-Owned Authority

Parent-only for the full run:

- `.runs/plan-15/*`
- `.runs/task-m15-*/**`
- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)

Parent-only during `p1`, then frozen by artifact and branch seed:

- request shape for `MemberTurnSubmitRequestV1`
- client surface for `submit_member_turn_stream(...)`
- launch-only posture of `ExecuteRequest.member_dispatch`
- lane ownership map
- validation-wall command list

Worker-owned after `p1`:

- `L1`: shell parser, exact selector, host submit path, Linux-first REPL world submit wiring
- `L2`: world-agent submit route, member registry refactor, submit-span cancel semantics
- `L3`: tests, stubs, wrap regression, gap-matrix closeout

No worker may write parent-owned run-state. No worker may change another lane's file set. No worker may reopen the frozen contract.

### Stop Conditions

Stop the run, write `.runs/plan-15/blocked.json`, and do not advance if any of these occur:

1. `p1` cannot freeze the new submit-turn request without changing the grammar or launch-only `member_dispatch` boundary.
2. Any implementation requires a second targeted-turn grammar spelling.
3. Any implementation changes `substrate -c` away from wrap mode.
4. Any worker edits [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) or [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs) after `p1`.
5. Any worker reuses `ExecuteRequest.member_dispatch` for submitted turns.
6. Any worker broadens host targeting beyond the active orchestrator backend for the current REPL session.
7. Any worker broadens world targeting beyond Linux-first behavior.
8. Any worker attempts to retain more than one world member at once.
9. Any worker collapses bootstrap spans and submitted-turn spans into one cancel class.
10. A lane needs an unowned file to proceed and the parent cannot re-plan without breaking the dependency graph.
11. `L3` starts before `p2` accepts final code truth.
12. The gap-matrix row is marked closed before code, tests, and final docs agree on the same landed behavior.
13. The parent would need to invent hybrid semantics during integration to make two lanes appear compatible.
14. The validation wall cannot prove exact grammar, wrap regression, launch-only `member_dispatch`, distinct span classes, and one-retained-world-member behavior.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/shell-parser-selector-host-submit`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/world-submit-route-and-member-registry`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/tests-stubs-docs-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m15-shell-parser-selector-host-submit`
- `codex/feat-session-centric-state-store-m15-world-submit-route-and-member-registry`
- `codex/feat-session-centric-state-store-m15-tests-stubs-docs-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/shell-parser-selector-host-submit -b codex/feat-session-centric-state-store-m15-shell-parser-selector-host-submit feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/world-submit-route-and-member-registry -b codex/feat-session-centric-state-store-m15-world-submit-route-and-member-registry feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/tests-stubs-docs-closeout -b codex/feat-session-centric-state-store-m15-tests-stubs-docs-closeout feat/session-centric-state-store
```

### Parent Integration Surface

The parent integrates on the authoritative checkout already on `feat/session-centric-state-store`.

No parent integration worktree is introduced for this run because:

1. `PLAN-15` has one serialized integrator.
2. accepted branch truth must land on the authoritative branch context named in the plan.
3. `.runs/plan-15/*` is parent-owned state and is easiest to keep coherent on that checkout.
4. there is no honest parallel parent merge activity in this plan.

## Phase Graph And Concurrency

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Worker cap is `2` until `g2` completes.
2. `p1` must finish before any worker starts.
3. `g1` must accept before any worker starts.
4. The only real parallel window is:
   - `task/m15-l1-shell-parser-selector-host-submit`
   - `task/m15-l2-world-submit-route-and-member-registry`
5. `g2` classifies both code-lane outputs before integration.
6. `p2` integrates accepted code lanes in this order:
   - `L2` first
   - `L1` second
7. `g3` starts only after `p2` is green.
8. `L3` runs alone on the exact post-`p2` tree.
9. `g4` and `p3` are parent-only.
10. No third concurrent worker is honest because test stubs and docs must encode final merged route semantics, not intermediate lane guesses.

### Why `L2` Integrates Before `L1`

1. `L2` lands the world-agent server truth that the new REPL world submit path depends on.
2. `L1` is the user-facing shell surface. Integrating it second avoids landing outward REPL affordances before the backend submit route exists on the parent tree.
3. If the combined state reveals semantic drift, the later-integrated shell lane is the first quarantine target instead of mutating world-agent ownership rules under active submit semantics.

## PLAN-15 Step Mapping

| Orchestration task | PLAN-15 step alignment |
| --- | --- |
| `task/m15-p1-parent-contract-freeze-and-run-init` | Step 1: freeze the submit-turn wire contract |
| `task/m15-l1-shell-parser-selector-host-submit` | Step 2, Step 3, and Step 5 shell-side work |
| `task/m15-l2-world-submit-route-and-member-registry` | Step 4 world-agent ownership and typed submit route |
| `task/m15-l3-tests-stubs-docs-closeout` | Step 6 regression floor and repo truth |
| `task/m15-p3-parent-validation-wall-and-closeout` | Recommended verification commands plus Definition of Done enforcement |

## Parent-Owned Run-State Surface

Canonical parent-owned state under `.runs/plan-15/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `frozen-contract.json`
- `lane-ownership.json`
- `validation-wall.md`
- `quarantine/`
- `blocked.json` on failure only
- `sentinels/`

`frozen-contract.json` is the single source of truth for:

- `plan_id: "PLAN-15"`
- `plan_source: "llm-last-mile/PLAN-15.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-15.md"`
- `branch: "feat/session-centric-state-store"`
- `accepted_targeted_grammar: "::<backend_id> <prompt>"`
- `launch_only_member_dispatch: true`
- `host_target_rule: "active_orchestrator_backend_only"`
- `world_target_rule: "linux_first_typed_submit_route_only"`
- `world_retention_rule: "single_retained_member"`
- `span_classes: ["bootstrap", "submitted_turn"]`
- `required_stable_fields`
- `failure_taxonomy`
- `validation_commands`

`lane-ownership.json` is the single source of truth for:

- task ID
- owner
- worktree path
- branch
- allowed files
- forbidden files
- command gates
- retry budget
- merge order
- sentinel filename

`run-state.json` is the single source of truth for:

- `current_phase`
- `active_task_ids`
- `worker_cap`
- `contract_freeze_status`
- `lane_status`
- `accepted_outputs`
- `rejected_outputs`
- `quarantined_outputs`
- `blocked_outputs`
- `retry_budget_by_lane`
- `attempt_counts`
- `integration_order`
- `gate_status`
- `validation_wall_status`
- `termination_reason`
- `terminal_state`

Required sentinels:

- `.runs/plan-15/sentinels/task-m15-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-15/sentinels/task-m15-g1-worker-launch-gate.ok`
- `.runs/plan-15/sentinels/task-m15-l1-shell-parser-selector-host-submit.ok`
- `.runs/plan-15/sentinels/task-m15-l2-world-submit-route-and-member-registry.ok`
- `.runs/plan-15/sentinels/task-m15-g2-code-lane-integration-gate.ok`
- `.runs/plan-15/sentinels/task-m15-p2-parent-code-lane-integration.ok`
- `.runs/plan-15/sentinels/task-m15-g3-closeout-launch-gate.ok`
- `.runs/plan-15/sentinels/task-m15-l3-tests-stubs-docs-closeout.ok`
- `.runs/plan-15/sentinels/task-m15-g4-validation-wall-gate.ok`
- `.runs/plan-15/sentinels/task-m15-p3-parent-validation-wall-and-closeout.ok`

Per-task artifact directories:

- `.runs/task-m15-p1-parent-contract-freeze-and-run-init/`
- `.runs/task-m15-g1-worker-launch-gate/`
- `.runs/task-m15-l1-shell-parser-selector-host-submit/`
- `.runs/task-m15-l2-world-submit-route-and-member-registry/`
- `.runs/task-m15-g2-code-lane-integration-gate/`
- `.runs/task-m15-p2-parent-code-lane-integration/`
- `.runs/task-m15-g3-closeout-launch-gate/`
- `.runs/task-m15-l3-tests-stubs-docs-closeout/`
- `.runs/task-m15-g4-validation-wall-gate/`
- `.runs/task-m15-p3-parent-validation-wall-and-closeout/`

Each task directory contains:

- `task.json`
- `summary.md`
- `commands.txt`
- `artifacts/`
- `evidence-manifest.json`

Each worker task directory also contains:

- `worker-output.patch`
- `worker-report.md`
- `rejected.json` on rejection only
- `blocked.json` on blocked lane return only
- `quarantine/` when the parent quarantines the output

Blocked termination surfaces, parent-owned:

- `.runs/plan-15/blocked.json`
- `.runs/plan-15/quarantine/<task-id>/`
- `.runs/task-m15-*/blocked.json` when a worker explicitly returns blocked evidence

Blocked termination minimum contents:

1. task or gate where execution stopped
2. classification: `rejected`, `blocked`, or `merge_refused`
3. exact contract clause or ownership rule that stopped the run
4. whether retry remained available
5. artifact paths for patch, report, command output, and semantic-drift evidence
6. explicit statement that no blocked output was integrated

## Parent Phase Details

### `task/m15-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-15/*` and `.runs/task-m15-*/**`.
2. Freeze the exact targeted-turn grammar and stop conditions into `frozen-contract.json`.
3. Add `MemberTurnSubmitRequestV1` only on the parent branch.
4. Add `submit_member_turn_stream(...)` only on the parent branch.
5. Preserve launch-only `ExecuteRequest.member_dispatch` behavior and boundary tests.
6. Freeze the validation-wall commands the parent will later execute.
7. Seed all worker worktrees from the exact post-`p1` tree.

Owned files:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)
- `.runs/plan-15/*`
- `.runs/task-m15-p1-parent-contract-freeze-and-run-init/**`

Command gates:

```bash
cargo test -p agent-api-types --lib -- --nocapture
cargo test -p agent-api-client --lib --no-run
```

Acceptance:

1. The request and client contract compile on the parent tree.
2. `ExecuteRequest.member_dispatch` is still launch-only after `p1`.
3. No shell or world-agent implementation file is needed to define the shared contract.
4. `frozen-contract.json`, `lane-ownership.json`, `run-state.json`, and `tasks.json` exist.
5. The parent writes `.runs/plan-15/sentinels/task-m15-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m15-g1-worker-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. All worker worktrees were seeded from the exact same post-`p1` tree.
3. Worker prompts name only owned files, forbidden files, command gates, sentinel, and retry budget.
4. Worker prompts explicitly repeat the frozen contract clauses that apply to each lane.
5. `run-state.json`, `tasks.json`, and `session-log.md` reflect launch state.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `.runs/plan-15/sentinels/task-m15-g1-worker-launch-gate.ok`.

## Worker Lanes

### `task/m15-l1-shell-parser-selector-host-submit`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/shell-parser-selector-host-submit`

Branch:

- `codex/feat-session-centric-state-store-m15-shell-parser-selector-host-submit`

Owned files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)

Forbidden files:

- all parent-owned files
- all `L2` owned files
- all `L3` owned files
- `.runs/plan-15/*`
- `.runs/task-m15-*/**`

Scope:

1. Add exact targeted-turn parsing for `::<backend_id> <prompt>`.
2. Route targeted turns before shell fallback and only on single-line input.
3. Add exact `backend_id` selection without reusing ambiguity-driven member selection.
4. Implement the shell-local host resume path against the stored surfaced `uaa_session_id`.
5. Enforce that host-targeted turns only target the active orchestrator backend for the current REPL session.
6. Wire Linux-first REPL world submit usage through the frozen typed client method.
7. Preserve one-retained-world-member behavior in REPL control flow.
8. Preserve `:host`, `:pty`, and plain shell behavior.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. Malformed `::` syntax fails before shell fallback.
3. Host-targeted turns fail closed when the requested backend is not the active orchestrator backend.
4. The shell lane does not broaden world retention beyond one member at a time.
5. `substrate -c` semantics are not changed by this lane.
6. All command gates pass.
7. Acceptance evidence exists before the parent marks the lane green:
   - `worker-output.patch`
   - `worker-report.md`
   - `commands.txt` with exit codes
   - `evidence-manifest.json`
8. The worker returns changed files, commands run with exit codes, unresolved assumptions, `worker-output.patch`, and `worker-report.md`.

### `task/m15-l2-world-submit-route-and-member-registry`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/world-submit-route-and-member-registry`

Branch:

- `codex/feat-session-centric-state-store-m15-world-submit-route-and-member-registry`

Owned files:

- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/src/handlers.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)

Forbidden files:

- all parent-owned files
- all `L1` owned files
- all `L3` owned files
- `.runs/plan-15/*`
- `.runs/task-m15-*/**`

Scope:

1. Add the new typed `/v1/member_turn/stream` server route on world-agent surfaces frozen in `p1`.
2. Refactor retained-member ownership to stable `participant_id` identity.
3. Keep submitted-turn cancel handles in a separate `span_id` registry.
4. Validate `backend_id`, `orchestration_session_id`, `world_id`, and `world_generation` against the retained runtime.
5. Reject submit when no surfaced `uaa_session_id` is retained.
6. Reject concurrent submitted turns for one participant.
7. Preserve bootstrap cancel semantics while adding submitted-turn cancel semantics.
8. Keep `ExecuteRequest.member_dispatch` launch-only and separate from follow-up submit routing.

Command gates:

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. The retained registry is keyed by participant identity for active members and by submit span for active turns.
3. Submitted-turn cancel does not tear down the retained bootstrap runtime.
4. The lane does not overload `member_dispatch`.
5. All command gates pass.
6. Acceptance evidence exists before the parent marks the lane green:
   - `worker-output.patch`
   - `worker-report.md`
   - `commands.txt` with exit codes
   - `evidence-manifest.json`
7. The worker returns changed files, commands run with exit codes, unresolved assumptions, `worker-output.patch`, and `worker-report.md`.

## Parent Integration Phases

### `task/m15-g2-code-lane-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned.
2. Each output is classified as `accepted`, `rejected`, or `blocked` before integration.
3. No lane violated file ownership, contract freeze, or stop conditions.
4. `L2` proves typed submit route separation from `member_dispatch`.
5. `L1` proves grammar freeze and active-orchestrator-only host targeting.
6. `g2` goes green only if both code lanes are `accepted`.

Acceptance:

1. The parent writes `.runs/plan-15/sentinels/task-m15-g2-code-lane-integration-gate.ok` only if both code lanes are accepted.

### `task/m15-p2-parent-code-lane-integration`

Owner:

- parent only

Scope:

1. Integrate only accepted outputs.
2. Integrate `L2` first and rerun its lane-local gates on the parent tree.
3. Integrate `L1` second and rerun its lane-local gates on the parent tree.
4. If `L1` depends on reopening contract files from `p1`, quarantine it and bounce it back instead of editing around the contract freeze.
5. If the combined state blurs bootstrap spans and submitted-turn spans, quarantine the later-integrated lane and record semantic drift.
6. If the combined state broadens host targeting beyond the active orchestrator backend, quarantine `L1`.
7. If the combined state broadens world ownership beyond one retained member or reuses `member_dispatch` for submit, quarantine `L2`.
8. Record accepted integration truth in `run-state.json` and `session-log.md`.
9. If an accepted worker patch merges mechanically but contradicts `frozen-contract.json`, refuse merge, write `merge-refusal.md`, quarantine the output, and either redrive or block. The parent does not hand-edit around the contradiction.

Command gates:

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

Acceptance:

1. The parent remains the sole integrator.
2. The integrated parent tree still matches the frozen contract.
3. No hybrid truth was invented during integration.
4. The parent writes `.runs/plan-15/sentinels/task-m15-p2-parent-code-lane-integration.ok`.

## Sequential Closeout Lane

### `task/m15-g3-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is accepted.
2. The closeout worktree is reseeded or rebased to the exact post-`p2` tree.
3. The worker prompt names only final tests, stubs, wrap regression, and docs ownership.
4. The worker prompt explicitly forbids changing code-lane production files.

Acceptance:

1. The parent writes `.runs/plan-15/sentinels/task-m15-g3-closeout-launch-gate.ok`.

### `task/m15-l3-tests-stubs-docs-closeout`

Owner:

- worker only

Why this stays worker-owned instead of parent-owned:

1. The lane is sequential, but still bounded and mechanically distinct.
2. The file set is broad enough to benefit from a dedicated closeout worker once final code truth exists.
3. Keeping it worker-owned prevents the parent from mixing final validation with test-authoring edits in the same phase.

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-15/tests-stubs-docs-closeout`

Branch:

- `codex/feat-session-centric-state-store-m15-tests-stubs-docs-closeout`

Owned files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/shell/tests/agents_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs)
- [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Forbidden files:

- all parent-owned files
- all `L1` owned files
- all `L2` owned files
- `.runs/plan-15/*`
- `.runs/task-m15-*/**`

Scope:

1. Prove the exact grammar acceptance and rejection rules in shell tests.
2. Extend the REPL world-agent stub to script `/v1/member_turn/stream`.
3. Prove host-targeted turns resume only the active orchestrator backend.
4. Prove world-targeted turns go through the new typed submit route.
5. Prove cancel semantics distinguish bootstrap spans from submitted-turn spans.
6. Prove one retained world member at a time and explicit backend switching behavior.
7. Add the mandatory `substrate -c` wrap-mode regression.
8. Update the gap matrix so repo truth matches landed behavior and still leaves broader productization open.

Command gates:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Tests assert the final typed submit route and final grammar only.
3. Wrap-mode regression is explicit and green.
4. The gap-matrix row is not marked closed unless code, tests, and docs agree on the same shipped truth.
5. All command gates pass.
6. Acceptance evidence exists before the parent marks the lane green:
   - `worker-output.patch`
   - `worker-report.md`
   - `commands.txt` with exit codes
   - `evidence-manifest.json`
7. The worker returns changed files, commands run with exit codes, unresolved assumptions, `worker-output.patch`, and `worker-report.md`.

## Validation Wall And Final Closeout

### `task/m15-g4-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L3` returned and is classified before final validation.
2. `L3` is `accepted`.
3. No quarantined or blocked output remains unresolved.
4. `validation-wall.md` names the exact final command order.
5. `validation-wall.md` includes contract assertions for grammar, wrap mode, typed world submit route, launch-only `member_dispatch`, single retained world member, and distinct span classes.

Acceptance:

1. The parent writes `.runs/plan-15/sentinels/task-m15-g4-validation-wall-gate.ok`.

### `task/m15-p3-parent-validation-wall-and-closeout`

Owner:

- parent only

Scope:

1. Integrate only accepted `L3` output.
2. Run the full validation wall in exact order.
3. Capture doctor outputs because this run touches shell and world-agent logic.
4. Record final command results and acceptance evidence in `.runs/task-m15-p3-parent-validation-wall-and-closeout/artifacts/`.
5. Mark the run complete only if the validation wall proves the frozen contract rather than merely compiling.

Required validation commands, executed in this order:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-api-types --lib -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
substrate world doctor --json
substrate shim doctor --json
substrate health --json
```

Required artifacts under `.runs/task-m15-p3-parent-validation-wall-and-closeout/artifacts/`:

- `fmt.txt`
- `clippy.txt`
- `agent-api-types-tests.txt`
- `shell-lib-tests.txt`
- `repl-world-first-routing.txt`
- `agents-validate.txt`
- `world-agent-streamed-execute-cancel.txt`
- `world-agent-member-runtime-placement.txt`
- `world-doctor.json`
- `shim-doctor.json`
- `health.json`
- `contract-audit.md`
- `closeout.md`

Required contract assertions in `contract-audit.md`:

1. The only accepted targeted syntax is `::<backend_id> <prompt>`.
2. `substrate -c` still resolves to wrap mode.
3. `ExecuteRequest.member_dispatch` is still launch-only.
4. Host-targeted turns are valid only for the active orchestrator backend in the current REPL session.
5. World-targeted turns are Linux-first and use the typed submit route.
6. One retained world member at a time remains the shipped behavior.
7. Bootstrap spans and submitted-turn spans are distinct in tests and cancel semantics.

Acceptance:

1. The full validation wall passes in order.
2. The doctor outputs are captured.
3. The artifact audit proves the frozen contract rather than approximate behavior.
4. The parent writes `.runs/plan-15/sentinels/task-m15-p3-parent-validation-wall-and-closeout.ok`.

## Gates

Gate sequencing is serialized and parent-owned:

1. `task/m15-p1-parent-contract-freeze-and-run-init`
2. `task/m15-g1-worker-launch-gate`
3. parallel launch of `task/m15-l1-shell-parser-selector-host-submit` and `task/m15-l2-world-submit-route-and-member-registry`
4. `task/m15-g2-code-lane-integration-gate`
5. `task/m15-p2-parent-code-lane-integration`
6. `task/m15-g3-closeout-launch-gate`
7. `task/m15-l3-tests-stubs-docs-closeout`
8. `task/m15-g4-validation-wall-gate`
9. `task/m15-p3-parent-validation-wall-and-closeout`

Gate rules:

1. The parent updates `run-state.json`, `tasks.json`, and `session-log.md` at every gate transition, worker launch, worker return, retry authorization, quarantine decision, merge refusal, and terminal closeout.
2. A gate is green only when all prerequisite sentinels and evidence for the prior phase exist.
3. A gate cannot go green on narrative progress alone; it requires command evidence, artifact presence, and ownership compliance.
4. `g2` and `g4` are classification gates, not integration gates. They may accept, reject, quarantine, or block; they do not mutate production files.

### Retry And Redrive Policy

1. Each worker lane has retry budget `1`.
2. The parent must classify the first attempt before authorizing any retry.
3. A retry is allowed only for lane-local failure inside the lane's owned files.
4. A retry is allowed only if the first attempt did not violate a hard guard, frozen contract clause, or cross-lane ownership rule.
5. Contract-freeze drift, `member_dispatch` submit reuse, host-target broadening, world-retention broadening, or wrap-mode regression are non-retryable.
6. The lane identity does not change on retry; only attempt metadata changes.
7. If a lane exhausts retry budget without acceptance, the run blocks.

### Merge Refusal Rules

1. The parent refuses merge for any lane whose patch requires parent-only file edits after `p1`.
2. The parent refuses merge for any lane that is locally green but semantically contradicts `frozen-contract.json`.
3. The parent refuses merge for any lane that broadens scope beyond its owned files, even if the code compiles.
4. The parent refuses merge for any lane that encodes intermediate behavior as final repo truth.
5. Merge refusal writes:
   - `.runs/task-<task-id>/artifacts/merge-refusal.md`
   - `.runs/task-<task-id>/rejected.json`
   - `.runs/plan-15/quarantine/<task-id>/`

### Quarantine, Rejection, And Blocked Termination

1. Rejected output is never integrated.
2. The parent records rejected output in `rejected_outputs` and `quarantined_outputs`.
3. Quarantine preserves `worker-output.patch`, `worker-report.md`, `commands.txt`, and any semantic-drift notes under `.runs/plan-15/quarantine/<task-id>/`.
4. A rejection is `retryable` only if the lane stayed inside owned files and did not violate frozen contract truth.
5. A blocked worker return, non-retryable rejection, exhausted retry budget, or failed parent gate writes `.runs/plan-15/blocked.json` and stops the run.
6. Blocked termination is explicit. The run ends either `completed` or `blocked`, never in a silent partial state.

## Worker Interfaces

### Worker Prompt Contract

Every worker prompt sent by the parent must include exactly:

1. task ID
2. attempt number
3. worktree path
4. branch name
5. owned files
6. forbidden files
7. exact frozen contract clauses relevant to that lane
8. command gates
9. retry budget
10. required return artifacts
11. sentinel name

The parent must not send:

1. discretionary broad goals without file boundaries
2. permission to edit parent-owned or cross-lane files
3. instructions that reopen `p1` contract decisions
4. instructions that imply hidden concurrency beyond the one honest two-lane window

### Worker Return Contract

Every worker must return exactly:

1. changed files list
2. commands run, each with exit code
3. explicit attempt classification: `clean`, `retryable`, or `blocked`
4. unresolved assumptions or blockers
5. `worker-output.patch`
6. `worker-report.md`
7. `evidence-manifest.json` naming every artifact the parent should inspect

For a return to be eligible for acceptance, the parent must be able to verify:

1. the patch stays within owned files
2. command gates actually ran
3. exit codes are present
4. the attempt classification matches the evidence
5. no frozen contract clause was reopened

## Tests And Acceptance

### Task-Scoped Command Gates

`task/m15-p1-parent-contract-freeze-and-run-init`

```bash
cargo test -p agent-api-types --lib -- --nocapture
cargo test -p agent-api-client --lib --no-run
```

`task/m15-l1-shell-parser-selector-host-submit`

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

`task/m15-l2-world-submit-route-and-member-registry`

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
```

`task/m15-p2-parent-code-lane-integration`

```bash
cargo test -p world-agent --lib -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

`task/m15-l3-tests-stubs-docs-closeout`

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

`task/m15-p3-parent-validation-wall-and-closeout`

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-api-types --lib -- --nocapture
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
substrate world doctor --json
substrate shim doctor --json
substrate health --json
```

### Acceptance Matrix

| Phase | Required proof | Refuse / block trigger |
| --- | --- | --- |
| Parent freeze `p1` | `MemberTurnSubmitRequestV1` and `submit_member_turn_stream(...)` frozen on parent branch; `member_dispatch` still launch-only; run-state artifacts exist | contract requires shell/world edits in `p1`, or `member_dispatch` loses launch-only posture |
| `L1` shell lane | exact grammar parse path, active-orchestrator-only host targeting, no wrap regression, no cross-lane edits | shell broadens host targeting, changes wrap behavior, or reopens contract files |
| `L2` world lane | typed submit route exists, retained registry split is explicit, submit cancel does not kill bootstrap runtime | submit path overloads `member_dispatch`, broadens retention beyond one member, or blurs span classes |
| Parent integration `p2` | accepted `L2` then accepted `L1`, no hybrid truth, semantic drift resolved by acceptance or quarantine | merge requires parent hand-edit around contradiction, or combined state violates frozen contract |
| `L3` closeout lane | tests and stubs prove final route and grammar, wrap regression explicit, gap matrix matches landed truth | tests encode intermediate behavior or docs close a gap earlier than code truth |
| Final validation wall `p3` | full command wall green, doctor evidence captured, contract audit proves exact frozen clauses | any final assertion fails, any blocked output remains unresolved, or audit cannot prove frozen truth |

### Run Exit Criteria

Success requires all of:

1. every required sentinel exists
2. no blocked artifact exists under `.runs/plan-15/`
3. accepted outputs are integrated in the prescribed order only
4. final validation commands pass
5. `contract-audit.md` proves grammar freeze, launch-only `member_dispatch`, active-orchestrator-only host targeting, Linux-first typed world submit, one retained world member, and distinct span classes

Blocked termination requires any of:

1. hard-guard violation
2. non-retryable rejection
3. exhausted retry budget
4. merge refusal with no legal redrive path
5. failed validation wall or doctor evidence

On blocked termination the parent must write:

1. `.runs/plan-15/blocked.json`
2. terminal state and rationale in `run-state.json`
3. gate and failure summary in `session-log.md`
4. preserved evidence under `.runs/plan-15/quarantine/` and the relevant task artifact directory

## Acceptance Criteria

The run is complete only if all of these are true:

1. The parent froze the contract before worker launch and never reopened it.
2. The only parallel window was `L1` plus `L2`.
3. `L3` started only after code-lane integration was complete.
4. No file ownership overlap occurred outside parent integration and final closeout takeover.
5. The final code path uses exact `backend_id` targeting for targeted turns.
6. Host turns are constrained to the active orchestrator backend.
7. World turns are Linux-first and use the typed submit route.
8. `member_dispatch` remains launch-only.
9. One retained world member at a time remains true.
10. Bootstrap and submitted-turn span classes remain distinct.
11. `substrate -c` stays wrap mode.
12. Gap-matrix truth matches code and tests.

## Context-Control Rules

1. The parent keeps only frozen contract artifacts, active task state, worker reports, narrow diffs, blockers, and validation status in live context.
2. The parent does not keep full worker transcripts in live context.
3. Worker prompts include only task ID, attempt number, worktree, branch, owned files, forbidden files, contract clauses, command gates, and output contract.
4. If a worker discovers it needs an unowned file, it must stop and report that need instead of silently expanding scope.
5. If a lane returns code that is locally green but semantically drifts from `frozen-contract.json`, the parent quarantines it instead of integrating around it.
