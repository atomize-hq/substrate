# ORCH_PLAN-14: Execute PLAN-14 Through A Parent-Frozen Auth-Bundle Contract And Honest Two-Lane Parallelism

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-14.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-14.md)  
Source SOW: [14-secret-handoff-into-the-world-gateway.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/14-secret-handoff-into-the-world-gateway.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Reference style source: [ORCH_PLAN-13.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-13.md)  
Execution type: fresh orchestration plan, Linux-first, backend-only, cross-crate gateway hardening with one real parallel window and parent-owned contract freeze

## Summary

This document is the execution control artifact for `PLAN-14`.

The orchestration truth is fixed:

1. `task/m14-p1-parent-contract-freeze-and-run-init` is parent-only and must complete before any worker starts.
2. The parent is the only integrator and the only writer of orchestration run-state artifacts.
3. The parent is the sole writer of accepted branch truth on `feat/session-centric-state-store`.
4. The shared auth-bundle contract is frozen before worker launch and is not reopened during lane execution.
5. The only honest parallel window is exactly two lanes after that freeze:
   - `L1` world-agent auth-bundle producer
   - `L2` gateway startup auth-bundle consumer
6. The regression/docs lane does not run in parallel with `L1` or `L2`. It starts only after both code lanes are accepted and integrated.
7. `GatewayLifecycleRequestV1.integrated_auth` remains unchanged for the whole run.
8. Host-side policy gating, host-side auth sourcing, and request-shape semantics remain unchanged for the whole run.
9. The only new stable env surface is `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
10. The parent freezes the exact Linux manual env-proof method during `p1` and `p3` must execute that frozen method verbatim.
11. The run is not complete until the parent finishes the full validation wall and the Linux manual proof for `sync`, `status`, and `restart`.

Canonical task IDs:

- `task/m14-p1-parent-contract-freeze-and-run-init`
- `task/m14-g1-worker-launch-gate`
- `task/m14-l1-world-agent-auth-bundle-producer`
- `task/m14-l2-gateway-startup-auth-bundle-consumer`
- `task/m14-g2-code-lane-integration-gate`
- `task/m14-p2-parent-code-lane-integration`
- `task/m14-g3-regression-launch-gate`
- `task/m14-l3-regression-docs-closeout`
- `task/m14-g4-validation-wall-gate`
- `task/m14-p3-parent-validation-wall-and-manual-proof`
- `task/m14-p4-closeout`

## Hard Guards

### Frozen Contract Truth

1. `GatewayLifecycleRequestV1.integrated_auth` stays frozen.
2. No host policy source or precedence change is allowed.
3. No change to [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) is allowed.
4. No change to [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs) is allowed.
5. The shared schema owner is [crates/common/src/gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs), not `agent-api-types`.
6. The only stable pointer env name is `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
7. No second stable env pointer, alias, or duplicate constant is allowed.
8. The integrated path must not inject `SUBSTRATE_LLM_BACKEND_AUTH_*`, `OPENAI_API_KEY`, or equivalent secret-bearing values into the managed gateway process environment by default.
9. `host_only` remains a bounded compatibility path and is not redefined.
10. The gateway integrated startup order is frozen as:
    - parse config without secret interpolation
    - if `in_world`, read `GatewayAuthBundleV1`
    - overlay integrated auth in memory
    - resolve remaining non-secret env placeholders
    - build `ProviderRegistry`
    - start serving traffic
11. Bundle failures remain transient integrated-startup failures, not policy-denial reclassification.
12. `sync` and `restart` must redeliver a fresh bundle every time.
13. The bundle is write-once, read-once, no-disk, and closed promptly.
14. The parent-owned Linux env-proof method is frozen during `p1`; `p3` must not improvise a replacement proof method.

### Parent-Owned And Lane-Owned Boundaries

Parent-only for the entire run:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
- `.runs/plan-14/*`
- `.runs/task-m14-*/**`

Parent-only during `p1`, then frozen:

- [crates/common/src/gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs)
- [crates/common/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/lib.rs)
- [crates/common/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/Cargo.toml)
- [crates/gateway/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/Cargo.toml)

Lane-owned after `p1`:

- `L1`: world-agent producer surfaces only
- `L2`: gateway consumer surfaces only
- `L3`: regression, shell tests, contract docs, and gap-matrix only

### Stop Conditions

Stop the run, write `.runs/plan-14/blocked.json`, and do not advance if any of these occur:

1. Preflight requires changing `agent-api-types`.
2. Preflight requires changing shell host policy or auth-sourcing logic.
3. The shared auth-bundle schema spreads into `agent-api-types`.
4. A worker introduces a second stable env surface beyond `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
5. A worker reintroduces secret-env fallback on the default integrated path.
6. A worker edits another lane’s owned files.
7. A worker updates `.runs/plan-14/*` or `.runs/task-m14-*/**`.
8. `L3` attempts to encode intermediate env-based behavior as final truth.
9. The gap-matrix row is marked closed before tests, docs, and final integrated code all agree on bundle-based default delivery.
10. Shell tests are changed to encode carrier mechanics instead of policy/failure-taxonomy truth.
11. The managed gateway process environment still contains secret-bearing auth values after the integrated path is green.
12. The parent-only integrator rule would be violated.
13. More than two simultaneous workers are required before the regression/docs phase.
14. The gateway startup order would need to move auth overlay after provider construction.
15. `p1` cannot freeze an exact Linux proof method for PID discovery, env inspection, and restart fresh-instance proof.

## Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/world-agent-auth-bundle-producer`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/gateway-startup-auth-bundle-consumer`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/regression-docs-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m14-world-agent-auth-bundle-producer`
- `codex/feat-session-centric-state-store-m14-gateway-startup-auth-bundle-consumer`
- `codex/feat-session-centric-state-store-m14-regression-docs-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/world-agent-auth-bundle-producer -b codex/feat-session-centric-state-store-m14-world-agent-auth-bundle-producer feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/gateway-startup-auth-bundle-consumer -b codex/feat-session-centric-state-store-m14-gateway-startup-auth-bundle-consumer feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/regression-docs-closeout -b codex/feat-session-centric-state-store-m14-regression-docs-closeout feat/session-centric-state-store
```

### Parent Integration Surface

Integration happens on the current checkout on `feat/session-centric-state-store`.

No dedicated parent integration worktree is used for this run because:

1. there is exactly one serialized integrator
2. accepted branch truth must land on the authoritative branch checkout the parent already owns
3. `.runs/plan-14/*` artifacts and accepted integration state are parent-only and are easiest to keep coherent on that authoritative checkout
4. there is no honest parallel parent-merge activity in this plan

The parent must not apply speculative multi-lane combinations on that checkout before `g2` classification is complete.

## Topology And Concurrency

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Worker cap is `2`.
2. `task/m14-p1-parent-contract-freeze-and-run-init` is parent-only and must finish first.
3. `task/m14-g1-worker-launch-gate` is parent-only and must accept before any worker starts.
4. The only real parallel window is:
   - `task/m14-l1-world-agent-auth-bundle-producer`
   - `task/m14-l2-gateway-startup-auth-bundle-consumer`
5. `task/m14-g2-code-lane-integration-gate` is parent-only and classifies `L1` and `L2` before integration.
6. `task/m14-p2-parent-code-lane-integration` is parent-only and integrates accepted code lanes in this order:
   - `L1` first
   - `L2` second
7. `task/m14-g3-regression-launch-gate` is parent-only and starts only after `p2` is accepted.
8. `task/m14-l3-regression-docs-closeout` runs alone.
9. `task/m14-g4-validation-wall-gate`, `task/m14-p3-parent-validation-wall-and-manual-proof`, and `task/m14-p4-closeout` are parent-only.
10. No other concurrency is honest because the regression floor and docs must encode the final merged contract, not intermediate lane reality.

### Why The Worker Cap Stays `2`

1. `PLAN-14` itself defines one safe parallel window only after the parent freezes the shared contract.
2. The producer and consumer code lanes are the only independently executable slices after that freeze.
3. The regression/docs lane depends on final integrated behavior and would create fake concurrency if started earlier.
4. Inflating to `3` would either duplicate shared-contract ownership or let tests/docs lock intermediate behavior, both of which are forbidden.

## PLAN-14 Step Mapping

| Orchestration task | PLAN-14 implementation step alignment |
| --- | --- |
| `task/m14-p1-parent-contract-freeze-and-run-init` | Step 1: freeze shared auth-bundle contract |
| `task/m14-l1-world-agent-auth-bundle-producer` | Step 2: replace world-agent env injection with bundle delivery |
| `task/m14-l2-gateway-startup-auth-bundle-consumer` | Step 3: refactor gateway startup to consume the bundle before provider construction |
| `task/m14-l3-regression-docs-closeout` | Step 4: rewrite regression floor and close the docs gap |
| `task/m14-p3-parent-validation-wall-and-manual-proof` and `task/m14-p4-closeout` | Step 5 and Definition of Done: validation wall, manual proof, final closeout |

## Task Registry And State Surfaces

Canonical parent-owned state:

- `.runs/plan-14/run-state.json`
- `.runs/plan-14/tasks.json`
- `.runs/plan-14/session-log.md`
- `.runs/plan-14/frozen-contract.json`
- `.runs/plan-14/frozen-proof-method.json`
- `.runs/plan-14/validation-wall.md`

`frozen-contract.json` is the single source of truth for:

- `plan_id: "PLAN-14"`
- `plan_source: "llm-last-mile/PLAN-14.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-14.md"`
- `branch: "feat/session-centric-state-store"`
- `schema_owner_file`
- `pointer_env_name`
- `canonical_fields_by_backend`
- `forbidden_child_env_keys`
- `startup_order`
- `failure_taxonomy`
- `host_policy_source_surface_frozen: true`
- `request_shape_surface_frozen: true`
- `host_only_compatibility_path_frozen: true`

`frozen-proof-method.json` is the single source of truth for the Linux manual proof method:

- `pid_discovery_command`
- `env_dump_command_template`
- `restart_fresh_instance_assertion_command_template`
- `expected_present_env_keys`
- `expected_absent_env_keys`
- `artifact_filenames`
- `proof_owner_task: "task/m14-p1-parent-contract-freeze-and-run-init"`
- `consuming_task: "task/m14-p3-parent-validation-wall-and-manual-proof"`

Rules for `frozen-proof-method.json`:

1. `pid_discovery_command` must be an exact parent-authored command string that returns the managed gateway PID for the integrated runtime from authoritative runtime-owned surfaces.
2. `env_dump_command_template` must be an exact parent-authored command string that inspects the managed process environment from the host side and writes it to artifacts without relying on gateway self-reporting.
3. `restart_fresh_instance_assertion_command_template` must be an exact parent-authored command string that proves pre-restart and post-restart process identity differ.
4. `p3` must execute these exact frozen commands or templates verbatim, with only the recorded PID substitution allowed where the template requires it.
5. If the proof method must change after `p1`, the run blocks and returns to planning.

`run-state.json` is the single source of truth for:

- `current_phase`
- `active_task_ids`
- `worker_cap: 2`
- `contract_freeze_status`
- `proof_method_freeze_status`
- `lane_status`
- `integration_order`
- `accepted_worker_outputs`
- `rejected_worker_outputs`
- `blocked_worker_outputs`
- `quarantined_worker_outputs`
- `attempt_counts`
- `retry_budget_by_lane`
- `validation_wall_status`
- `manual_proof_status`
- `terminal_state`

`tasks.json` is the ordered execution registry for:

- task ID
- owner
- worktree path
- branch
- allowed files
- forbidden files
- command gates
- expected artifacts
- sentinel name
- attempt number
- retry eligibility
- merge order
- current status

Required sentinels:

- `.runs/plan-14/sentinels/task-m14-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-14/sentinels/task-m14-g1-worker-launch-gate.ok`
- `.runs/plan-14/sentinels/task-m14-l1-world-agent-auth-bundle-producer.ok`
- `.runs/plan-14/sentinels/task-m14-l2-gateway-startup-auth-bundle-consumer.ok`
- `.runs/plan-14/sentinels/task-m14-g2-code-lane-integration-gate.ok`
- `.runs/plan-14/sentinels/task-m14-p2-parent-code-lane-integration.ok`
- `.runs/plan-14/sentinels/task-m14-g3-regression-launch-gate.ok`
- `.runs/plan-14/sentinels/task-m14-l3-regression-docs-closeout.ok`
- `.runs/plan-14/sentinels/task-m14-g4-validation-wall-gate.ok`
- `.runs/plan-14/sentinels/task-m14-p3-parent-validation-wall-and-manual-proof.ok`
- `.runs/plan-14/sentinels/task-m14-p4-closeout.ok`

Per-task artifact directories:

- `.runs/task-m14-p1-parent-contract-freeze-and-run-init/`
- `.runs/task-m14-g1-worker-launch-gate/`
- `.runs/task-m14-l1-world-agent-auth-bundle-producer/`
- `.runs/task-m14-l2-gateway-startup-auth-bundle-consumer/`
- `.runs/task-m14-g2-code-lane-integration-gate/`
- `.runs/task-m14-p2-parent-code-lane-integration/`
- `.runs/task-m14-g3-regression-launch-gate/`
- `.runs/task-m14-l3-regression-docs-closeout/`
- `.runs/task-m14-g4-validation-wall-gate/`
- `.runs/task-m14-p3-parent-validation-wall-and-manual-proof/`
- `.runs/task-m14-p4-closeout/`

Each task directory contains:

- `task.json`
- `summary.md`
- `commands.txt`
- `artifacts/`

Each worker task directory also contains:

- `worker-output.patch`
- `worker-report.md`

## Parent Workstreams

### `task/m14-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Initialize all `.runs/plan-14/*` and `.runs/task-m14-*/**` artifacts.
2. Record the frozen contract in `frozen-contract.json`.
3. Record the frozen Linux proof method in `frozen-proof-method.json`.
4. Author the shared schema location in `substrate-common` and export it through [crates/common/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/lib.rs).
5. Add the `substrate-common` dependency edge to [crates/gateway/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/Cargo.toml) if absent.
6. Freeze exactly one canonical field map:
   - `cli:codex` -> canonical `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`
   - `api:openai` -> `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`
7. Freeze exactly one pointer env name: `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
8. Freeze the gateway startup order and failure taxonomy.
9. Freeze the exact Linux manual proof method by recording:
   - the exact PID discovery command
   - the exact env dump command template
   - the exact fresh-instance assertion command template
   - the exact env keys expected present and absent
   - the exact artifact filenames `p3` must write
10. Seed all worker worktrees from the exact post-`p1` tree.

Command gate:

```bash
cargo test -p substrate-common --lib -- --nocapture
cargo test -p world-agent gateway_runtime --no-run
cargo test -p substrate-gateway --no-run
```

Acceptance:

1. No change to shell host policy/auth-sourcing surfaces occurred.
2. No change to `agent-api-types` occurred.
3. `substrate-common` is the only schema owner.
4. The shared contract is written once and frozen.
5. The Linux proof method is written once and frozen in `.runs/plan-14/frozen-proof-method.json`.
6. The parent writes `.runs/plan-14/sentinels/task-m14-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m14-g1-worker-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. All worker worktrees were seeded from the same post-`p1` tree.
3. Every worker prompt names only its owned file set, exact command gates, forbidden files, sentinel, retry budget, and narrow output contract.
4. Every worker prompt specifies `model: GPT-5.4` and `reasoning_effort: high`.
5. `run-state.json`, `tasks.json`, and `session-log.md` reflect launch state.

Acceptance:

1. No worker starts before this gate is accepted.
2. The parent writes `.runs/plan-14/sentinels/task-m14-g1-worker-launch-gate.ok`.

## Worker Workstreams

### `task/m14-l1-world-agent-auth-bundle-producer`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/world-agent-auth-bundle-producer`

Branch:

- `codex/feat-session-centric-state-store-m14-world-agent-auth-bundle-producer`

Owned files:

- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)

Forbidden files:

- all parent-only files
- all `L2` owned files
- all `L3` owned files
- `.runs/plan-14/*`
- `.runs/task-m14-*/**`

Scope:

1. Replace env-oriented integrated handoff modeling with `GatewayAuthBundleV1` consumption from `substrate-common`.
2. Create the inherited FD/pipe channel during runtime start.
3. Serialize the auth bundle once and pass only the read end to the managed gateway child.
4. Export only `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
5. Remove default integrated secret-env injection from the child environment.
6. Preserve `host_only` bounded compatibility behavior.
7. Preserve lifecycle semantics, restart/rotation behavior, and failure taxonomy.

Command gates:

```bash
cargo test -p world-agent gateway_runtime --no-run
cargo test -p world-agent gateway_runtime -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. The default integrated path no longer injects secret-bearing env values into the child.
3. `sync` and `restart` both flow through fresh bundle creation.
4. The lane does not alter request shape, shell policy, or schema ownership.
5. All command gates pass.
6. The worker returns changed files, commands run with exit codes, blockers or unresolved assumptions, `worker-output.patch`, and `worker-report.md`.

### `task/m14-l2-gateway-startup-auth-bundle-consumer`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/gateway-startup-auth-bundle-consumer`

Branch:

- `codex/feat-session-centric-state-store-m14-gateway-startup-auth-bundle-consumer`

Owned files:

- [crates/gateway/src/main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/main.rs)
- [crates/gateway/src/cli/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/cli/mod.rs)
- [crates/gateway/src/auth/codex_auth_context.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/auth/codex_auth_context.rs)
- [crates/gateway/src/launch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/launch.rs)
- [crates/gateway/src/server/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/server/mod.rs)
- [crates/gateway/src/providers/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/providers/registry.rs)

Forbidden files:

- all parent-only files
- all `L1` owned files
- all `L3` owned files
- `.runs/plan-14/*`
- `.runs/task-m14-*/**`

Scope:

1. Refactor integrated startup into the frozen two-phase order.
2. Read `GatewayAuthBundleV1` exactly once from `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
3. Close the FD promptly on success and failure.
4. Build a startup-owned in-memory integrated auth context before provider construction.
5. Overlay `cli:codex` and `api:openai` integrated auth in memory.
6. Preserve `host_only` behavior and non-secret env resolution behavior.
7. Fail closed for missing pointer env, unreadable FD, malformed JSON, or missing required fields.
8. Forbid silent fallback to env-based secret delivery in integrated mode.

Command gates:

```bash
cargo test -p substrate-gateway --no-run
cargo test -p substrate-gateway codex_auth_context -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Integrated `cli:codex` no longer depends on process env.
3. Integrated `api:openai` no longer depends on secret env interpolation during config load.
4. `host_only` remains unchanged.
5. No secret-env compatibility fallback is left on by default.
6. All command gates pass.
7. The worker returns changed files, commands run with exit codes, blockers or unresolved assumptions, `worker-output.patch`, and `worker-report.md`.

## Code-Lane Integration

### `task/m14-g2-code-lane-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned.
2. Each output is classified as `accepted`, `rejected`, or `blocked` before integration.
3. Any rejected output is quarantined before integration.
4. Any blocked output is preserved and may terminate the run.
5. No lane violated contract freeze, schema ownership, or file ownership.
6. `g2` goes green only if both code lanes are classified `accepted`.

Acceptance:

1. The parent writes `.runs/plan-14/sentinels/task-m14-g2-code-lane-integration-gate.ok` only if both code lanes are accepted.

### `task/m14-p2-parent-code-lane-integration`

Owner:

- parent only

Scope:

1. Integrate only accepted outputs.
2. Integrate `L1` first.
3. Re-run `L1` lane-local command gates on the parent tree.
4. Integrate `L2` second.
5. Re-run `L2` lane-local command gates on the parent tree.
6. If the later-integrated lane merges mechanically but reveals semantic drift against the frozen contract, the parent must not keep that combined state as tentative truth.
7. For semantic drift on schema ownership, startup order, or fallback posture, the parent must do exactly one of:
   - apply the frozen `PLAN-14` contract literally if one lane is already correct and the other lane is plainly non-conforming
   - quarantine and bounce the offending lane back to its worker slot with an explicit semantic-drift report
8. The parent must not invent hybrid truth during integration.
9. If `L1` and `L2` each look locally correct but together reveal contradictory semantics, the parent quarantines the later-integrated lane first, records the semantic conflict in `.runs/task-m14-p2-parent-code-lane-integration/artifacts/semantic-drift.md`, and bounces that lane for retry.
10. If the bounced lane returns and the semantic drift remains, the run blocks instead of synthesizing a compromise.
11. If either lane attempts to move schema ownership out of `substrate-common`, move auth overlay after provider construction, or leave default integrated secret-env fallback enabled, the parent bounces or blocks that lane instead of merging around it.
12. Record accepted code-lane truth in `run-state.json` and `session-log.md`.

Command gate:

```bash
cargo test -p world-agent gateway_runtime --no-run
cargo test -p substrate-gateway --no-run
cargo test -p substrate-gateway codex_auth_context -- --nocapture
```

Acceptance:

1. The parent remains the sole integrator.
2. Integrated code-lane truth matches the frozen contract.
3. No hybrid truth was invented during integration.
4. The parent writes `.runs/plan-14/sentinels/task-m14-p2-parent-code-lane-integration.ok`.

## Regression And Docs Lane

### `task/m14-g3-regression-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is accepted.
2. The regression/docs worktree is reseeded or rebased to the exact post-`p2` tree.
3. The worker prompt names only final regression/docs ownership, not intermediate behavior.
4. The worker prompt forbids encoding env-based secret delivery as final truth.

Acceptance:

1. The parent writes `.runs/plan-14/sentinels/task-m14-g3-regression-launch-gate.ok`.

### `task/m14-l3-regression-docs-closeout`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-14/regression-docs-closeout`

Branch:

- `codex/feat-session-centric-state-store-m14-regression-docs-closeout`

Owned files:

- [crates/world-agent/tests/gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)
- [crates/gateway/tests/openai_shared_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/tests/openai_shared_parity.rs)
- [crates/shell/tests/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)
- [docs/contracts/gateway/policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/policy-evaluation.md)
- [crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Forbidden files:

- all parent-only files
- all `L1` owned files
- all `L2` owned files
- `.runs/plan-14/*`
- `.runs/task-m14-*/**`

Scope:

1. Rewrite parity tests to assert pointer-env presence and secret-env absence.
2. Prove read-once behavior and fresh-bundle restart/rotation behavior.
3. Keep shell precedence and failure-taxonomy tests green without changing shell production semantics.
4. Shell tests may be updated only to preserve policy/failure-taxonomy truth or user-visible lifecycle classification truth. They must not encode carrier mechanics on the shell side.
5. Update docs so auth-bundle delivery is the landed default integrated path.
6. The gap-matrix row may be marked closed only if final integrated code, final tests, and final docs all agree on bundle-based default delivery.

Command gates:

```bash
cargo test -p world-agent --test gateway_runtime_parity --no-run
cargo test -p substrate-gateway openai_shared_parity --no-run
cargo test -p shell --test world_gateway --no-run
cargo test -p world-agent --test gateway_runtime_parity -- --nocapture
cargo test -p substrate-gateway openai_shared_parity -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Tests assert final bundle-based behavior only.
3. Shell tests preserve policy/failure-taxonomy truth and do not encode carrier mechanics.
4. Docs no longer describe env-based secret injection as the current default integrated path.
5. The [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) row is not marked closed unless tests, docs, and final integrated code all agree.
6. All command gates pass.
7. The worker returns changed files, commands run with exit codes, blockers or unresolved assumptions, `worker-output.patch`, and `worker-report.md`.

## Validation Wall And Closeout

### `task/m14-g4-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L3` returned and is classified before final validation.
2. `L3` is `accepted`.
3. No blocked or quarantined output remains unresolved.
4. The final command wall and manual proof plan are written to `validation-wall.md`.
5. `validation-wall.md` references the exact frozen proof method from `.runs/plan-14/frozen-proof-method.json`.

Acceptance:

1. The parent writes `.runs/plan-14/sentinels/task-m14-g4-validation-wall-gate.ok`.

### `task/m14-p3-parent-validation-wall-and-manual-proof`

Owner:

- parent only

Scope:

1. Integrate only accepted `L3` output.
2. Run the full validation wall in exact order.
3. Run the Linux manual proof for `sync`, `status`, and `restart`.
4. Execute the exact proof method frozen during `p1`; do not improvise or replace it.
5. Capture proof artifacts for env absence and fresh restart behavior using the artifact filenames frozen in `.runs/plan-14/frozen-proof-method.json`.
6. Record final results in `run-state.json`, `validation-wall.md`, and `session-log.md`.

Required validation commands, executed in this order:

```bash
cargo test -p world-agent gateway_runtime -- --nocapture
cargo test -p world-agent --test gateway_runtime_parity -- --nocapture
cargo test -p substrate-gateway codex_auth_context -- --nocapture
cargo test -p substrate-gateway openai_shared_parity -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
substrate world gateway sync
substrate world gateway status --json
substrate world gateway restart
substrate world gateway status --json
```

Required manual proof artifacts under `.runs/task-m14-p3-parent-validation-wall-and-manual-proof/artifacts/`:

- `status-after-sync.json`
- `status-after-restart.json`
- `pid-before-restart.txt`
- `pid-after-restart.txt`
- `env-before-restart.txt`
- `env-after-restart.txt`
- `manual-proof-summary.md`

Required manual proof execution rules:

1. Run `pid_discovery_command` from `.runs/plan-14/frozen-proof-method.json` after `sync` and write `pid-before-restart.txt`.
2. Run `env_dump_command_template` from `.runs/plan-14/frozen-proof-method.json` against that PID and write `env-before-restart.txt`.
3. Run `substrate world gateway restart`.
4. Run `pid_discovery_command` again and write `pid-after-restart.txt`.
5. Run `env_dump_command_template` against the post-restart PID and write `env-after-restart.txt`.
6. Run `restart_fresh_instance_assertion_command_template` exactly as frozen and record the result in `manual-proof-summary.md`.
7. If any exact frozen proof command cannot be executed as recorded, the run blocks instead of substituting a different proof.

Required manual assertions:

1. `status --json` reaches `available` after `sync`.
2. `status --json` reaches `available` after `restart`.
3. The managed gateway runtime exposes `SUBSTRATE_LLM_AUTH_BUNDLE_FD`.
4. The managed gateway runtime does not expose:
   - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
   - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
   - `SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY`
   - `OPENAI_API_KEY`
5. Restart produces a fresh process instance with fresh auth-bundle delivery.

Acceptance:

1. The full validation wall passes in order.
2. The Linux manual proof is recorded and green.
3. The manual proof used the exact frozen proof method from `p1`.
4. The parent writes `.runs/plan-14/sentinels/task-m14-p3-parent-validation-wall-and-manual-proof.ok`.

### `task/m14-p4-closeout`

Owner:

- parent only

Scope:

1. Verify all required sentinels exist.
2. Verify `run-state.json` includes final validation and manual proof status.
3. Verify `.runs/plan-14/blocked.json` is absent on the green path.
4. Verify the gap-matrix closeout state matches final code, tests, and docs.
5. Write `.runs/plan-14/closeout.md`.
6. Mark `run-state.json` as `completed`.

Command gate:

```bash
test -f .runs/plan-14/sentinels/task-m14-p3-parent-validation-wall-and-manual-proof.ok
```

Acceptance:

1. The run ends with either complete closeout or earlier blocked termination.
2. The parent writes `.runs/plan-14/sentinels/task-m14-p4-closeout.ok` only on green completion.

## Context-Control Rules

1. The parent keeps only the minimal live artifacts in working context:
   - frozen contract
   - frozen proof method
   - active task state
   - accepted lane summaries
   - narrow diffs
   - blockers
   - validation-wall status
2. The parent does not keep full worker transcripts in live context.
3. Worker prompts include only:
   - task ID
   - attempt number
   - worktree and branch
   - owned files
   - forbidden files
   - exact command gates
   - sentinel name
   - retry budget
   - `model: GPT-5.4`
   - `reasoning_effort: high`
4. Workers return only:
   - changed files
   - commands run with exit codes
   - blockers or unresolved assumptions
   - attempt classification: `clean`, `retryable`, or `blocked`
   - `worker-output.patch`
   - `worker-report.md`
5. The parent reviews summaries plus narrow diffs, not full transcripts.
6. After a lane is merged or quarantined, the parent closes that worker immediately.
7. Workers never update run-state artifacts.
8. If a lane needs authority outside its boundary, it stops and returns blocked evidence.

## Merge Refusal Rules

The parent refuses merge and blocks or bounces the run if integrating a lane would require:

1. changing `agent-api-types`
2. changing shell host policy or host auth sourcing
3. reopening parent-frozen shared schema files after `p1`
4. cross-lane ownership drift
5. a second stable env surface
6. secret-env fallback on the default integrated path
7. docs or tests encoding intermediate behavior as final truth
8. shell tests encoding carrier mechanics instead of policy/failure-taxonomy truth
9. creative conflict resolution that contradicts `PLAN-14`
10. hybrid truth on schema ownership, startup order, or fallback posture
11. integrating rejected or blocked output
12. bypassing the parent-only integrator rule

## Retry And Redrive Policy

1. Each worker lane has retry budget `1`.
2. Retries reuse the same lane slot and do not increase concurrency beyond `2`.
3. The parent must classify the first attempt before authorizing a retry.
4. A retry is allowed only for lane-local failure within owned files.
5. Contract-freeze violations, ownership drift, schema drift, or secret-env fallback are non-retryable.
6. If any lane exhausts retry budget without acceptance, the run blocks.

## Tests And Acceptance Matrix

### Task-Scoped Command Gates

`task/m14-p1-parent-contract-freeze-and-run-init`

```bash
cargo test -p substrate-common --lib -- --nocapture
cargo test -p world-agent gateway_runtime --no-run
cargo test -p substrate-gateway --no-run
```

`task/m14-l1-world-agent-auth-bundle-producer`

```bash
cargo test -p world-agent gateway_runtime --no-run
cargo test -p world-agent gateway_runtime -- --nocapture
```

`task/m14-l2-gateway-startup-auth-bundle-consumer`

```bash
cargo test -p substrate-gateway --no-run
cargo test -p substrate-gateway codex_auth_context -- --nocapture
```

`task/m14-l3-regression-docs-closeout`

```bash
cargo test -p world-agent --test gateway_runtime_parity --no-run
cargo test -p substrate-gateway openai_shared_parity --no-run
cargo test -p shell --test world_gateway --no-run
cargo test -p world-agent --test gateway_runtime_parity -- --nocapture
cargo test -p substrate-gateway openai_shared_parity -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
```

`task/m14-p3-parent-validation-wall-and-manual-proof`

```bash
cargo test -p world-agent gateway_runtime -- --nocapture
cargo test -p world-agent --test gateway_runtime_parity -- --nocapture
cargo test -p substrate-gateway codex_auth_context -- --nocapture
cargo test -p substrate-gateway openai_shared_parity -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
substrate world gateway sync
substrate world gateway status --json
substrate world gateway restart
substrate world gateway status --json
```

### Acceptance Matrix

| Gate | Required proof | Primary surfaces |
| --- | --- | --- |
| Contract freeze | one schema owner, one pointer env name, one startup order, frozen request/policy surfaces, frozen Linux proof method | `substrate-common`, `frozen-contract.json`, `frozen-proof-method.json`, `p1` sentinel |
| Worker launch | both code lanes seeded from same frozen tree | worktrees, `tasks.json`, `g1` sentinel |
| Producer lane | `world-agent` writes bundle, passes only pointer env, preserves restart semantics | `gateway_runtime.rs`, `L1` report |
| Consumer lane | gateway reads bundle once before provider construction and does not depend on secret env | gateway startup surfaces, `L2` report |
| Code integration | both code lanes accepted and parent-integrated in order without hybrid truth | parent checkout, `g2` and `p2` sentinels |
| Regression/docs launch | regression lane starts only after merged code truth exists | `g3` sentinel |
| Regression/docs lane | parity tests and docs encode final bundle-based truth only; shell tests stay policy/failure-taxonomy-only | tests, docs, gap matrix, `L3` report |
| Validation wall | required command wall passes in exact order | parent checkout, `g4` and `p3` sentinels |
| Manual proof | frozen proof method executed verbatim; `sync`, `status`, `restart`, env absence, fresh restart proof recorded | manual artifacts, `p3` sentinel |
| Closeout | all required sentinels exist and run-state is complete | `.runs/plan-14/closeout.md` |

## Run Exit Criteria

Successful completion requires all of the following:

1. Host policy and request-shape surfaces remained frozen.
2. `substrate-common` remained the only schema owner.
3. `world-agent` stopped using default secret-bearing child env injection on the integrated path.
4. `substrate-gateway` adopted the frozen startup order.
5. Integrated `cli:codex` and `api:openai` no longer depend on secret-bearing process env in the managed gateway.
6. The managed gateway process environment no longer contains secret-bearing auth values by default.
7. `sync` and `restart` redeliver fresh bundles.
8. Tests and docs describe the same final behavior.
9. The gap-matrix closeout state matches final code, tests, and docs.
10. The parent completed the validation wall and Linux manual proof using the exact frozen proof method.
11. `.runs/plan-14/closeout.md` exists and `run-state.json` is `completed`.

Blocked completion requires all of the following:

1. `.runs/plan-14/blocked.json` exists.
2. `run-state.json` is `blocked`.
3. `session-log.md` records the exact violated guard and stopping task.
4. Rejected or blocked worker output is preserved without integration.
5. No green closeout sentinel is written.

## Assumptions

1. [PLAN-14.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-14.md) is the authoritative execution contract.
2. The safe parallel window begins only after the parent freezes the shared contract.
3. The shared schema can be fully owned inside `substrate-common` without reopening public transport contracts.
4. The gateway consumer lane can implement the frozen startup order without needing new request surfaces.
5. The Linux proof method can be frozen concretely during `p1` from runtime-owned surfaces already present in the repo’s managed gateway lifecycle.
6. The regression/docs lane can stay fully serialized after code-lane integration without slowing overall completion materially.
