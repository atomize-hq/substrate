# ORCH_PLAN-13: Execute PLAN-13 Through Parent-Owned Placement Preflight And Three Frozen Lanes

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-13.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-13.md)  
Source SOW: [13-member-runtime-world-placement-gap-sow.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/13-member-runtime-world-placement-gap-sow.md)  
Execution type: fresh orchestration plan, Linux-first, backend-only, installer-sensitive, platform-posture closeout required

## Summary

This document is the execution control artifact for `PLAN-13`.

The orchestration truth is fixed:

1. `task/m13-p1-parent-preflight-and-run-init` is parent-only and must complete before any worker starts.
2. The payload contract stays frozen for the whole run.
3. The parent is explicitly authorized to thaw the crate-surface request bridge during preflight only if a rebase reveals that the shell can no longer legally consume the frozen request builder through `crate::execution::*`.
4. The preferred bridge repair remains a direct re-export of `MemberDispatchTransportRequest` through the allowed crate surface.
5. The only pre-authorized fallback remains one sanctioned adapter helper.
6. After preflight, exactly three implementation lanes may run in parallel:
   - `L1` world-agent placement core
   - `L2` shell fail-closed and status/trace truth
   - `L3` installer, provisioning, and docs alignment
7. The parent remains the sole integrator, the sole writer of accepted branch truth on `feat/session-centric-state-store`, and the sole writer of `.runs/plan-13/*`.
8. The run is not complete until the parent finishes the proof wall and platform-posture closeout.
9. There are no human approval gates in `PLAN-13`. The only serialized pauses are parent-owned gates and blocked termination conditions.

Canonical task IDs:

- `task/m13-p1-parent-preflight-and-run-init`
- `task/m13-g1-worker-launch-gate`
- `task/m13-l1-world-agent-placement-core`
- `task/m13-l2-shell-fail-closed-status-trace`
- `task/m13-l3-installer-provisioning-docs-alignment`
- `task/m13-g2-integration-gate`
- `task/m13-p2-parent-integration-and-proof-wall`
- `task/m13-g3-platform-posture-closeout-gate`
- `task/m13-p3-closeout`

## Hard Guards

### Frozen Payload And Bridge Truth

1. `MemberDispatchRequestV1` stays frozen.
2. No serialized `member_dispatch` payload change is allowed.
3. No second request-construction path may be added in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
4. Runtime selection stays in the shell. `world-agent` must not infer or take over runtime selection.
5. `resolved_runtime.backend_kind` remains explicit and shell-authored.
6. `resolved_runtime.binary_path` remains absolute and shell-authored.
7. `POST /v1/execute/stream` and `POST /v1/execute/cancel` remain the only transport seam.
8. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the only stream families.
9. The shell remains the only canonical writer of orchestration session and participant state.
10. `world-agent` remains the transport owner for startup, event streaming, cancel, and terminal observation.
11. If preflight finds bridge drift, the parent may thaw only these files and only in `task/m13-p1-parent-preflight-and-run-init`:
    - [routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
    - [prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
    - [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
12. Preflight must use this bridge repair order:
    - preferred: direct re-export of `MemberDispatchTransportRequest` through `prelude.rs` and `routing.rs`
    - only fallback: one sanctioned adapter helper on the existing routing surface, then re-exported through `prelude.rs` and `routing.rs`
13. After `task/m13-p1-parent-preflight-and-run-init` is accepted, the bridge files are frozen for the rest of the run.

### Placement Truth And Platform Posture

1. Linux-first placement correctness is required in this slice.
2. World-scoped member launch has only two allowed outcomes:
   - observably placed inside the authoritative session world
   - fail-closed before the member is advertised as live
3. Host-local fallback for a selected world-scoped member is forbidden.
4. Metadata-only world binding is not acceptable proof of placement.
5. The new placement path must preserve retained control, event streaming, and cancel-by-`span_id`.
6. Same-generation reuse and replacement semantics remain shell-owned.
7. macOS Lima verification is required if the guest service contract or staged runtime changes.
8. WSL verification is required only if this run claims WSL alignment.
9. If WSL is not aligned in-slice, the required proof is explicit fail-closed posture plus docs closeout. Silent incompatibility is forbidden.
10. Worker cap is `3`.
11. All worker lanes run with `model: GPT-5.4` and `reasoning_effort: high`.

### Run Artifacts And Serialized Gate Model

1. `.runs/plan-13/*` and `.runs/task-m13-*/**` are run artifacts for orchestration control and evidence capture.
2. These run artifacts are not assumed git-tracked deliverables.
3. Only the parent writes `.runs/plan-13/*` and `.runs/task-m13-*/**`.
4. There are no human approval gates in this run.
5. The only serialized pauses are:
   - parent-owned gates
   - blocked termination conditions
6. Workers do not wait on discretionary approval once launched. They either return accepted evidence, retryable evidence, or blocked evidence.

### Parent-Owned And Lane-Owned Boundaries

Parent-only for the entire run:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- `.runs/plan-13/*`
- `.runs/task-m13-*/**`

Parent-only during preflight, then frozen:

- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

Escalation-only, parent-owned if the proof wall reveals unplanned blast radius:

- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

### Stop Conditions

Stop the run, write `.runs/plan-13/blocked.json`, and do not advance if any of these occur:

1. Preflight requires changing [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs).
2. Preflight requires payload-shape or serialized-payload changes.
3. Any lane requires duplicate request construction in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
4. Any lane requires moving runtime selection into `world-agent`.
5. Any worker edits a frozen bridge file after preflight.
6. Any worker edits another lane's owned files.
7. `L1` cannot produce an observable Linux placement proof.
8. `L2` still leaves a warning-and-continue path for selected world-scoped members.
9. `L3` claims installer or platform alignment without updating every relevant contract author in its owned set.
10. WSL is neither aligned nor explicitly fail-closed in docs and closeout state.
11. A fourth workstream or new blast-radius authority is required before the parent proof wall.
12. The parent-only integrator rule would be violated.

## Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/world-agent-placement-core`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/shell-fail-closed-status-trace`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/installer-provisioning-docs-alignment`

Worker branches:

- `codex/feat-session-centric-state-store-m13-world-agent-placement-core`
- `codex/feat-session-centric-state-store-m13-shell-fail-closed-status-trace`
- `codex/feat-session-centric-state-store-m13-installer-provisioning-docs-alignment`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/world-agent-placement-core -b codex/feat-session-centric-state-store-m13-world-agent-placement-core feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/shell-fail-closed-status-trace -b codex/feat-session-centric-state-store-m13-shell-fail-closed-status-trace feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/installer-provisioning-docs-alignment -b codex/feat-session-centric-state-store-m13-installer-provisioning-docs-alignment feat/session-centric-state-store
```

## Topology And Concurrency

Parent checkout:

- current checkout on `feat/session-centric-state-store`

Concurrency rules:

1. Exact worker cap is `3`.
2. `task/m13-p1-parent-preflight-and-run-init` is parent-only and must finish first.
3. `task/m13-g1-worker-launch-gate` is parent-only and must accept before any worker starts.
4. The only honest parallel window is:
   - `task/m13-l1-world-agent-placement-core`
   - `task/m13-l2-shell-fail-closed-status-trace`
   - `task/m13-l3-installer-provisioning-docs-alignment`
5. `task/m13-g2-integration-gate` is parent-only and classifies all worker outputs before any integration starts.
6. `task/m13-p2-parent-integration-and-proof-wall` is parent-only and integrates accepted output in this order:
   - `L1` first
   - `L2` second
   - `L3` third
7. `L2` and `L3` must rebase on the accepted `L1` reality if integration reveals drift against the accepted placement-core implementation.
8. `task/m13-g3-platform-posture-closeout-gate` is parent-only.
9. `task/m13-p3-closeout` is parent-only.
10. No additional concurrency is honest because platform posture and final truth depend on integrated runtime behavior.

## Task Registry And State Surfaces

Canonical parent-owned state:

- `.runs/plan-13/run-state.json`
- `.runs/plan-13/tasks.json`
- `.runs/plan-13/session-log.md`

`run-state.json` is the single source of truth for:

- `plan_id: "PLAN-13"`
- `plan_source: "llm-last-mile/PLAN-13.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-13.md"`
- `sow_source: "llm-last-mile/13-member-runtime-world-placement-gap-sow.md"`
- `branch: "feat/session-centric-state-store"`
- `current_phase`
- `active_task_ids`
- `worker_cap: 3`
- `bridge_status`
- `bridge_choice`
- `bridge_files_frozen_after_task`
- `payload_contract_frozen: true`
- `placement_carrier_shape`
- `placement_helper_reuse_path`
- `wsl_posture`
- `platform_verification_required`
- `worktrees`
- `accepted_worker_outputs`
- `rejected_worker_outputs`
- `blocked_worker_outputs`
- `quarantined_worker_outputs`
- `attempt_counts`
- `retry_budget_by_lane`
- `gate_status`
- `escalation_usage`
- `final_validation`
- `platform_closeout`
- `terminal_state`

`tasks.json` is the ordered execution registry for:

- task ID
- owner
- worktree path
- branch
- allowed files
- forbidden files
- exact PLAN-13 excerpt pointer for the worker
- command gates
- expected artifacts
- sentinel name
- attempt number
- retry eligibility
- merge order
- current status

`session-log.md` is the append-only parent log for:

- preflight decisions
- bridge-thaw rationale if used
- worker launch records
- worker returns
- acceptance, rejection, quarantine, or blocked rationale
- integration order
- proof-wall results
- platform-posture closeout decision
- final completion or blocked termination

Required sentinels:

- `.runs/plan-13/sentinels/task-m13-p1-parent-preflight-and-run-init.ok`
- `.runs/plan-13/sentinels/task-m13-g1-worker-launch-gate.ok`
- `.runs/plan-13/sentinels/task-m13-l1-world-agent-placement-core.ok`
- `.runs/plan-13/sentinels/task-m13-l2-shell-fail-closed-status-trace.ok`
- `.runs/plan-13/sentinels/task-m13-l3-installer-provisioning-docs-alignment.ok`
- `.runs/plan-13/sentinels/task-m13-g2-integration-gate.ok`
- `.runs/plan-13/sentinels/task-m13-p2-parent-integration-and-proof-wall.ok`
- `.runs/plan-13/sentinels/task-m13-g3-platform-posture-closeout-gate.ok`
- `.runs/plan-13/sentinels/task-m13-p3-closeout.ok`

Required terminal artifacts:

- `.runs/plan-13/blocked.json`
- `.runs/plan-13/closeout.md`

Per-task artifact directories:

- `.runs/task-m13-p1-parent-preflight-and-run-init/`
- `.runs/task-m13-g1-worker-launch-gate/`
- `.runs/task-m13-l1-world-agent-placement-core/`
- `.runs/task-m13-l2-shell-fail-closed-status-trace/`
- `.runs/task-m13-l3-installer-provisioning-docs-alignment/`
- `.runs/task-m13-g2-integration-gate/`
- `.runs/task-m13-p2-parent-integration-and-proof-wall/`
- `.runs/task-m13-g3-platform-posture-closeout-gate/`
- `.runs/task-m13-p3-closeout/`

Each task directory contains:

- `task.json`
- `summary.md`
- `commands.txt`
- `artifacts/`

Each worker task directory also contains:

- `worker-output.patch`
- `worker-report.md`

Accepted worker outputs additionally contain:

- `accepted.json`

Rejected worker outputs additionally contain:

- `rejected.json`
- `quarantine/`

Blocked worker outputs additionally contain:

- `blocked.json`

Sentinel semantics:

1. A `.ok` sentinel means the parent validated and accepted the task.
2. Missing sentinel means the task is not accepted, even if code exists in a worker tree.
3. Only the parent writes `.runs/plan-13/*` and `.runs/task-m13-*/**`.
4. `blocked.json` exists only for blocked termination.
5. `closeout.md` exists only for successful completion.

## Parent Workstreams

### `task/m13-p1-parent-preflight-and-run-init`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-13/*` and all per-task directories.
2. Record the frozen payload contract, bridge authority, placement strategy, and platform-posture rules in `run-state.json`.
3. Confirm the shell can still legally consume the frozen request builder through `crate::execution::*`.
4. If a rebase broke that bridge, thaw and repair it using the sanctioned order:
   - preferred: direct re-export of `MemberDispatchTransportRequest`
   - only fallback: one sanctioned adapter helper
5. Freeze the bridge files after the repair or confirmation.
6. Record one agreed internal placement carrier shape and one agreed helper-reuse direction for `L1`.
7. Record the WSL decision as either `aligned-in-slice` or `fail-closed-docs-posture`.
8. Seed all worker worktrees from the exact post-preflight tree.

Command gate:

```bash
cargo test -p shell --lib -- --nocapture
```

Acceptance:

1. No payload or serialized transport change occurred.
2. `bridge_choice` is recorded as `direct-re-export`, `sanctioned-adapter-helper`, or `bridge-unchanged`.
3. The bridge files are frozen after this task.
4. `wsl_posture` is recorded explicitly.
5. The parent writes `.runs/plan-13/sentinels/task-m13-p1-parent-preflight-and-run-init.ok`.

### `task/m13-g1-worker-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. The bridge seam is frozen and recorded.
3. All three worker worktrees are seeded from the same post-`p1` tree.
4. Every worker prompt names only the worker’s owned file set, the exact `PLAN-13` excerpt needed, command gates, forbidden files, sentinel name, and retry budget.
5. Every worker prompt specifies `model: GPT-5.4` and `reasoning_effort: high`.
6. `run-state.json`, `tasks.json`, and `session-log.md` reflect launch state.

Acceptance:

1. No worker starts before this gate is accepted.
2. The parent writes `.runs/plan-13/sentinels/task-m13-g1-worker-launch-gate.ok`.

## Worker Workstreams

### `task/m13-l1-world-agent-placement-core`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/world-agent-placement-core`

Branch:

- `codex/feat-session-centric-state-store-m13-world-agent-placement-core`

Owned files:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/Cargo.toml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/Cargo.toml)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- one new Linux-only world-placement proof test in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/`

Forbidden files:

- all parent-only files
- all frozen bridge files
- all `L2` owned files
- all `L3` owned files
- `.runs/plan-13/*`
- `.runs/task-m13-*/**`

Scope:

1. Resolve authoritative placement facts after world validation.
2. Add a placement-aware member launcher for long-lived retained-control runtime.
3. Reuse or extract the existing world-entry and cgroup-binding logic from the gateway-runtime path instead of creating a second isolation truth.
4. Preserve `Start`, `Event`, `Exit`, `Error`, retained control, and cancel-by-`span_id`.
5. Fail closed on world mismatch, missing binary, unsupported backend, missing placement facts, or failed world entry.
6. Add one Linux-only observable placement proof test.

Command gates:

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test <linux-world-placement-proof-target> --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test <linux-world-placement-proof-target> -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Linux placement is proven by observable runtime facts, not metadata.
3. Cancel still reaches the live placed runtime.
4. All command gates pass.
5. The worker returns changed files, exact commands run, exit codes, `worker-output.patch`, and `worker-report.md`.
6. The lane does not request payload churn, bridge reopening, or shell-owned runtime selection.

### `task/m13-l2-shell-fail-closed-status-trace`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/shell-fail-closed-status-trace`

Branch:

- `codex/feat-session-centric-state-store-m13-shell-fail-closed-status-trace`

Owned files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Forbidden files:

- all parent-only files
- all frozen bridge files
- all `L1` owned files
- all `L3` owned files
- `.runs/plan-13/*`
- `.runs/task-m13-*/**`

Scope:

1. Replace the warning-and-continue path with explicit startup failure for selected world-scoped members.
2. Preserve `Allocating -> Ready` gating on session-handle evidence only.
3. Preserve same-generation reuse behavior.
4. Preserve replacement lineage and invalidation semantics.
5. Make status and trace assertions reflect actual placed-runtime truth.
6. Forbid fallback to host-local retained control after world-scoped selection.

Command gates:

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
cargo test -p shell --test agent_hub_trace_persistence --no-run
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

Acceptance:

1. The lane touches only its owned files.
2. Placement failure never yields authoritative-live status.
3. `Ready` and `Running` remain remote-evidence-backed.
4. Same-generation reuse stays a no-op.
5. Failed replacement never revives stale generation truth.
6. All command gates pass.
7. The worker returns changed files, exact commands run, exit codes, `worker-output.patch`, and `worker-report.md`.

### `task/m13-l3-installer-provisioning-docs-alignment`

Owner:

- worker only

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-13/installer-provisioning-docs-alignment`

Branch:

- `codex/feat-session-centric-state-store-m13-installer-provisioning-docs-alignment`

Owned files:

- [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh)
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh)
- [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
- [scripts/wsl/provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/wsl/provision.sh)
- [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md)
- [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md)
- [docs/cross-platform/wsl_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/wsl_world_setup.md)

Forbidden files:

- all parent-only files
- all frozen bridge files
- all `L1` owned files
- all `L2` owned files
- `.runs/plan-13/*`
- `.runs/task-m13-*/**`

Scope:

1. Align Linux dev-install, Linux release install, direct Linux provision, and macOS Lima guest provisioning with the final placement contract.
2. Update capability, env, RW path, socket/group, and binary-staging assumptions where required.
3. Either align WSL in-slice or make WSL explicitly fail-closed with docs and closeout posture.
4. Update operator-facing docs so the placement contract and verification steps match the shipped runtime contract.
5. Do not introduce a new installer templating system in this slice.

Command gates:

```bash
bash -n scripts/substrate/dev-install-substrate.sh
bash -n scripts/substrate/install-substrate.sh
bash -n scripts/linux/world-provision.sh
bash -n scripts/mac/lima-warm.sh
bash -n scripts/wsl/provision.sh
pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/windows/wsl-warm.ps1',[ref]$null,[ref]$null)"
```

Acceptance:

1. The lane touches only its owned files.
2. Every runtime-contract author in the owned set is reviewed and updated if required by the final placement contract.
3. WSL posture is explicit, not implicit.
4. All command gates pass.
5. The worker returns changed files, exact commands run, exit codes, `worker-output.patch`, and `worker-report.md`.

## Integration And Proof Wall

### `task/m13-g2-integration-gate`

Owner:

- parent only

Checks:

1. All three worker lanes returned.
2. Each worker output is classified as `accepted`, `rejected`, or `blocked` before integration.
3. Any rejected output is quarantined before the parent integrates anything.
4. Any blocked output is preserved and may terminate the run.
5. No cross-lane ownership violation, bridge reopen, or stop-condition drift is unresolved.
6. `g2` goes green only if `L1`, `L2`, and `L3` are all classified `accepted`.

Acceptance:

1. The parent writes `.runs/plan-13/sentinels/task-m13-g2-integration-gate.ok` only if all three lanes are accepted.

### `task/m13-p2-parent-integration-and-proof-wall`

Owner:

- parent only

Scope:

1. Integrate only accepted outputs.
2. Integrate accepted `L1` output first.
3. If accepted `L2` or `L3` output conflicts with the accepted `L1` placement-core reality, bounce that lane back for rebase or apply `PLAN-13` literally. Do not invent a hybrid.
4. Re-run `L1` lane-local command gates on the parent tree if context moved during integration.
5. Integrate accepted `L2` output second, rebased on accepted `L1` reality if needed.
6. Re-run `L2` lane-local command gates on the parent tree if context moved during integration.
7. Integrate accepted `L3` output third, rebased on accepted `L1` reality if needed.
8. If accepted lane outputs disagree on placement-carrier shape, WSL posture, or runtime-contract assumptions, do not resolve the disagreement creatively in parent integration. Either:
   - bounce the disagreement back to the owning lane
   - or apply `PLAN-13` literally where it already specifies the answer
9. Run the full Rust proof wall in order.
10. Run Linux platform verification.
11. Run macOS Lima verification if `L3` changed guest service contract or staged runtime behavior.
12. Run WSL verification only if the run claims WSL alignment; otherwise record explicit fail-closed docs posture and verify that docs landed.
13. Record final validation and platform posture in `run-state.json` and `session-log.md`.

Recommended verification command wall, executed in this order:

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test <linux-world-placement-proof-target> --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
cargo test -p shell --test agent_hub_trace_persistence --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test <linux-world-placement-proof-target> -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
scripts/linux/world-provision.sh --profile release
scripts/mac/lima-warm.sh
pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
```

Platform execution rule for that wall:

1. `scripts/linux/world-provision.sh --profile release` is always required.
2. `scripts/mac/lima-warm.sh` is required only if the guest service contract or staged runtime changed.
3. `pwsh -File scripts/windows/wsl-warm.ps1 ...` is required only if the run claims WSL alignment.
4. If WSL alignment is not claimed, the parent must not mark WSL green from non-WSL evidence; the proof is explicit fail-closed posture plus docs closeout.

Acceptance:

1. The parent remains the sole integrator.
2. The full Rust proof wall passes in order.
3. Linux verification is recorded as passed.
4. macOS Lima verification is recorded as passed or not-required with rationale.
5. WSL is recorded as passed or explicit fail-closed with rationale.
6. The parent writes `.runs/plan-13/sentinels/task-m13-p2-parent-integration-and-proof-wall.ok`.

## Context-Control Rules

1. The parent keeps only the minimal live artifacts in working context:
   - current task and gate state
   - frozen invariants
   - accepted lane summaries
   - narrow diffs under review
   - active blockers
   - final proof-wall status
2. The parent does not keep full worker transcripts in live working context.
3. Worker prompts include only:
   - owned file set
   - exact `PLAN-13` excerpt needed for that lane
   - command gates
   - forbidden files
   - sentinel name
   - retry budget
4. Worker prompts do not include unrelated plan sections, full repository dumps, or cross-lane speculative design.
5. Workers return only:
   - changed files
   - commands with exit codes
   - blockers or unresolved assumptions
   - `worker-output.patch`
   - `worker-report.md`
6. Workers do not return long transcripts, exploratory notes, or unused alternatives.
7. The parent reviews summaries plus narrow diffs, not full transcripts.
8. The parent reads only the exact diff and evidence needed to accept, reject, quarantine, or bounce a lane.
9. After a lane is merged or quarantined, the parent closes that worker immediately and removes it from active context.
10. Workers never update `.runs/plan-13/*` or `.runs/task-m13-*/**`.
11. If a lane needs new authority outside its owned boundary, the worker stops and returns blocked evidence instead of improvising.
12. The parent updates `run-state.json`, `tasks.json`, and `session-log.md` at every gate transition, launch, return, retry, classification, integration, and closeout step.

### Worker Prompt Contract

Every worker prompt must include:

1. exact task ID
2. attempt number
3. worktree and branch
4. owned files only
5. forbidden files
6. exact `PLAN-13` excerpt needed for that lane
7. exact command gates
8. sentinel name
9. retry budget
10. `model: GPT-5.4`
11. `reasoning_effort: high`

### Worker Return Contract

Every worker return must include:

1. changed files
2. commands run with exit codes
3. blockers or unresolved assumptions
4. whether the attempt is `clean`, `retryable`, or `blocked`
5. `worker-output.patch`
6. `worker-report.md`

### Merge Refusal Rules

The parent refuses merge and blocks or bounces the run if integrating a lane would require:

1. reopening frozen bridge files after preflight
2. cross-lane ownership drift
3. new blast-radius authority not already authorized by this plan
4. weakening placement truth, status truth, or trace truth to force green
5. creative conflict resolution that contradicts `PLAN-13`
6. inventing a hybrid when lane outputs disagree on placement-carrier shape, WSL posture, or runtime-contract assumptions
7. integrating rejected or blocked output
8. bypassing the parent-only integrator rule

## Tests And Acceptance

### Task-Scoped Command Gates

`task/m13-p1-parent-preflight-and-run-init`

```bash
cargo test -p shell --lib -- --nocapture
```

`task/m13-l1-world-agent-placement-core`

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p world-agent --test <linux-world-placement-proof-target> --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test <linux-world-placement-proof-target> -- --nocapture
```

`task/m13-l2-shell-fail-closed-status-trace`

```bash
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
cargo test -p shell --test agent_hub_trace_persistence --no-run
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

`task/m13-l3-installer-provisioning-docs-alignment`

```bash
bash -n scripts/substrate/dev-install-substrate.sh
bash -n scripts/substrate/install-substrate.sh
bash -n scripts/linux/world-provision.sh
bash -n scripts/mac/lima-warm.sh
bash -n scripts/wsl/provision.sh
pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/windows/wsl-warm.ps1',[ref]$null,[ref]$null)"
```

`task/m13-p2-parent-integration-and-proof-wall`

```bash
1. cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
2. cargo test -p world-agent --test <linux-world-placement-proof-target> --no-run
3. cargo test -p shell --test repl_world_first_routing_v1 --no-run
4. cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
5. cargo test -p shell --test agent_hub_trace_persistence --no-run
6. cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
7. cargo test -p world-agent --test <linux-world-placement-proof-target> -- --nocapture
8. cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
9. cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
10. cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
11. scripts/linux/world-provision.sh --profile release
12. scripts/mac/lima-warm.sh
13. pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
```

### Acceptance Matrix

| Gate | Required proof | Primary surfaces |
| --- | --- | --- |
| Preflight gate | payload frozen, bridge legal through `crate::execution::*`, WSL posture recorded | bridge files, `run-state.json`, `p1` sentinel |
| Worker launch gate | all three workers seeded from the same frozen post-`p1` tree | worktree records, `tasks.json`, `g1` sentinel |
| World-agent gate | live member runtime is observably world-placed and cancel still works | `world-agent` sources, `streamed_execute_cancel_v1.rs`, one new Linux-only world-placement proof test in `crates/world-agent/tests/` |
| Shell gate | placement failure is fail-closed and status/trace truth stays evidence-backed | `async_repl.rs`, shell tests |
| Installer gate | all contract authors align or WSL is explicitly fail-closed | scripts, docs, `L3` report |
| Integration gate | all worker outputs are accepted and safe to integrate | task directories, `g2` sentinel |
| Proof wall | Rust test wall and required platform verification pass in order | parent checkout, `p2` sentinel |
| Platform closeout gate | Linux required, macOS conditional, WSL conditional or explicit fail-closed | `run-state.json`, `g3` sentinel |
| Closeout | all required sentinels exist and terminal state is complete | `.runs/plan-13/closeout.md` |

### Run Exit Criteria

Successful completion requires all of the following:

1. The payload contract remained frozen.
2. Any bridge thaw happened only in preflight and used only the sanctioned repair path.
3. Linux placement truth is proven by runtime facts.
4. Selected world-scoped member launch fails closed if placement cannot be established.
5. Same-generation reuse remains intact.
6. Failed replacement never revives stale generation truth.
7. Status and trace surfaces remain truthful.
8. Linux verification passed.
9. macOS Lima verification passed if required.
10. WSL is either aligned and verified or explicitly fail-closed with docs.
11. `.runs/plan-13/closeout.md` exists and `run-state.json` is `completed`.

Blocked completion requires all of the following:

1. `.runs/plan-13/blocked.json` exists.
2. `run-state.json` is `blocked`.
3. `session-log.md` records the exact violated guard and stopping task.
4. Rejected or blocked worker output is preserved without integration.
5. No green closeout sentinel is written.

## Blocked And Closeout Artifacts

### Retry And Redrive Policy

1. Each worker lane has retry budget `1`.
2. Retries reuse the same lane slot and do not increase concurrency beyond `3`.
3. The parent must classify the first attempt as `accepted`, `rejected`, or `blocked` before authorizing any retry.
4. A retry is allowed only for lane-local failure inside the lane's owned files.
5. Hard-guard violation, cross-lane drift, bridge reopen, or new blast-radius authority makes the lane non-retryable.
6. The parent records every attempt, retry rationale, and final classification in `run-state.json`, `tasks.json`, and `session-log.md`.
7. If a lane exhausts retry budget without acceptance, the run blocks.

### Acceptance, Rejection, Quarantine, And Block Rules

Accepted worker output:

1. The parent records the output in `accepted_worker_outputs`.
2. The parent writes the task sentinel.
3. Only accepted output may be integrated.

Rejected worker output:

1. The parent records the output in `rejected_worker_outputs` and `quarantined_worker_outputs`.
2. The parent writes `rejected.json` and preserves evidence under `quarantine/`.
3. Rejected output is never integrated unless a later retry returns accepted evidence.

Blocked worker output:

1. The parent records the output in `blocked_worker_outputs`.
2. The parent writes task-local `blocked.json`.
3. If the block prevents the proof wall or violates a hard guard, the full run terminates with `.runs/plan-13/blocked.json`.

### `task/m13-g3-platform-posture-closeout-gate`

Owner:

- parent only

Checks:

1. `p2` is accepted.
2. Linux verification is recorded as passed.
3. macOS Lima status is recorded as passed or not-required with concrete rationale.
4. WSL status is recorded as passed or explicit fail-closed with concrete rationale.
5. No active blocked artifact remains.

Acceptance:

1. The parent writes `.runs/plan-13/sentinels/task-m13-g3-platform-posture-closeout-gate.ok`.

### `task/m13-p3-closeout`

Owner:

- parent only

Scope:

1. Verify all required sentinels exist.
2. Verify `run-state.json` includes final validation and platform posture.
3. Confirm `.runs/plan-13/blocked.json` is absent on the green path.
4. Write `.runs/plan-13/closeout.md`.
5. Mark `run-state.json` as `completed`.

Command gate:

```bash
test -f .runs/plan-13/sentinels/task-m13-g3-platform-posture-closeout-gate.ok
```

Acceptance:

1. The run ends with either complete closeout or earlier blocked termination.
2. The parent writes `.runs/plan-13/sentinels/task-m13-p3-closeout.ok` only on green completion.

## Assumptions

1. [PLAN-13.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-13.md) is the authoritative execution contract for this run.
2. [13-member-runtime-world-placement-gap-sow.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/13-member-runtime-world-placement-gap-sow.md) is the authoritative implementation context for the placement gap and installer-sensitive blast radius.
3. The implementation can stay inside the three planned worker lanes after parent preflight.
4. The sanctioned bridge thaw, if needed, is sufficient to restore legal consumption through `crate::execution::*` without payload churn.
5. One new Linux-only world-placement proof test in `crates/world-agent/tests/` is enough to establish real placement truth for this slice.
6. The parent can integrate `L1` first and reconcile `L2` and `L3` without reopening frozen bridge files.
7. Platform closeout can honestly distinguish `required`, `not-required`, and `explicit fail-closed` postures without inventing unsupported parity claims.
8. `.runs/plan-13/*` remain orchestration artifacts whether or not they are committed in git.
