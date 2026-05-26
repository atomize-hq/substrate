# SOW: Finish True In-World Placement For World-Scoped Member Dispatch

Status: implementation-oriented draft. This SOW covers the remaining correctness gap after the transport cutover work from [11-in-world-member-dispatch-over-existing-host-world-transport.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/11-in-world-member-dispatch-over-existing-host-world-transport.md). The shell now launches world-scoped members through `world-agent` over the existing host<->world transport, but the launched member runtime is still not actually placed into the authoritative per-session world boundary.

## Objective

Land one Linux-first production path where a world-scoped member runtime is:

- selected and persisted by the shell as it is today,
- launched by `world-agent` over the existing `/v1/execute/stream` seam as it is today,
- and actually executed inside the active session world rather than merely correlated to that world in metadata.

The required outcome is not "transport-owned member runtime." The required outcome is "transport-owned and world-placed member runtime."

## Problem Statement

The current implementation moved ownership of member startup, event forwarding, and cancellation into `world-agent`, but it stopped short of real world placement.

Today the flow is:

1. The shell chooses a world-scoped member, prepares lineage and authoritative world binding, and builds a typed member-dispatch request in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3394) and [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:203).
2. The shell sends that request over the existing world socket in [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1176).
3. `world-agent` ensures the authoritative shared world and validates that the request matches the current `world_id` and `world_generation` in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1223) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1233).
4. `world-agent` then launches the member by calling `gateway.run_control(...)` directly from `MemberRuntimeManager::launch(...)` in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:50).

The missing step is that `MemberRuntimeManager::launch(...)` never re-enters the session world:

- it does not execute through `backend.exec(&world, ...)`,
- it does not use the session overlay root,
- it does not attach the child to the session cgroup,
- it does not establish the same world placement guarantees as normal world exec or the isolated gateway-runtime path.

That means the member runtime is:

- authoritative in shell state,
- remote-owned for startup and cancel,
- but still not honestly placed inside the world boundary it claims to be bound to.

## Why This Matters

This is a correctness issue, not just a documentation issue.

The current code can truthfully say:

- the member belongs to `world_id=X`,
- the member belongs to `world_generation=Y`,
- `world-agent` owns the retained control handle,
- `/v1/execute/cancel` can reach it.

But it cannot yet truthfully say:

- the member process is actually executing under the same per-session world isolation contract as world commands.

This is also the remaining unclosed gap called out in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:110), and it sits directly on top of the ordered SOW packet in [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:23).

That mismatch undermines:

- filesystem isolation expectations,
- cgroup accounting expectations,
- network posture expectations,
- replay/trace/operator trust in `world_id` and `world_generation`,
- and the intent captured in [11-in-world-member-dispatch-over-existing-host-world-transport.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/11-in-world-member-dispatch-over-existing-host-world-transport.md:182).

## Current Relevant Code Surfaces

### Shell request construction and shell-owned authority

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3353)
  - `ensure_member_runtime_ready(...)`
  - `prepare_member_runtime_startup_for_descriptor(...)`
  - `start_remote_member_runtime_with_prepared(...)`
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:203)
  - `MemberDispatchTransportRequest`
  - `build_member_dispatch_payload(...)`
  - `build_agent_client_and_member_dispatch_request_impl(...)`

These surfaces are already doing the right transport and state-authority work. This SOW does not redesign member selection, lineage, or canonical persistence.

### World-agent request routing and current gap

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1233)
  - `execute_stream(...)` branches to member dispatch after ensuring the authoritative world.
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:50)
  - `MemberRuntimeManager::launch(...)` directly calls `gateway.run_control(...)`.
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1446)
  - `execute_cancel(...)` already reaches member runtime spans.

### World placement reference implementations

- Normal non-PTY world execution goes through the backend in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1290) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1343).
- Gateway runtime uses an explicit isolated runtime binding and session cgroup attach in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1580), [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1601), and [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:696).

These are the reference patterns the member-runtime placement fix must reuse or mirror.

## In Scope

- make world-scoped member runtime launch actually enter the authoritative session world
- preserve the existing `/v1/execute/stream` and `/v1/execute/cancel` transport seam
- preserve shell ownership of canonical participant/session state
- preserve current replacement, invalidation, and lineage semantics
- preserve current typed member-dispatch request shape unless one additive field is required for placement
- update installer/provisioning paths so shipped/dev-installed `world-agent` behavior matches the new runtime contract
- update docs and targeted tests to prove true placement rather than metadata-only correlation

## Out Of Scope

- redesigning orchestrator ownership
- moving runtime selection into `world-agent`
- public `substrate agent start|resume|fork|stop` productization
- cross-platform feature parity beyond explicit Linux-first fail-closed posture
- secret-carrier changes for gateway integrated auth
- toolbox work

## Required Semantics

1. Shell authority stays authoritative.
The shell must remain the canonical writer of orchestration-session and participant state.

2. No host-side fallback for world-scoped members.
If world placement for a selected world-scoped member cannot be established, startup must fail closed. The shell must not silently fall back to a host-local retained-control launch.

3. World placement must be real, not inferred.
The launched member runtime must receive the same effective world placement guarantees that make `world_id` and `world_generation` meaningful operator-visible facts.

4. Transport ownership stays in `world-agent`.
The fix must not regress back to a shell-owned member runtime.

5. Replacement semantics remain shell-owned.
Generation rollover still invalidates stale members and launches replacements via shell authority, but the actual replacement runtime must now be placed into the replacement world.

6. "Unavailable member runtime" must not silently become "member placement skipped."
The current Linux shell path still prints `world-scoped member runtime unavailable` and returns control to the REPL loop in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3405). The final behavior for a selected world-scoped member must be fail-closed once real placement is required.

## Core Implementation Work

### 1. Introduce a real world-placement launch path for member runtime

`MemberRuntimeManager::launch(...)` in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:50) must stop treating `gateway.run_control(...)` as sufficient by itself.

The fix must add one honest placement mechanism for the spawned member runtime that covers at least:

- authoritative session-world resolution
- session cgroup attachment
- correct world-root / overlay / filesystem view
- correct network posture
- correct retained-control ownership and cancellation retention

This does not require reusing the exact normal `backend.exec(&world, ...)` API if that API is a poor fit for a long-lived retained-control runtime. It does require equivalent world-placement guarantees.

### 2. Decide and implement the member-runtime world-entry strategy

The implementation must choose one sanctioned strategy and document it in code and docs. Acceptable direction:

- extend `world-agent` with a placement-aware runtime launcher for long-lived retained-control member processes
- reuse the same world-session discovery and cgroup/overlay preparation rules that back the normal exec path
- keep `agent_api` backend construction in `world-agent`, but wrap the actual child launch in world-entry logic rather than direct daemon-local spawn

The implementation must not leave placement implicit.

### 3. Thread the required placement context into member runtime launch

Today `service.rs` validates the world and passes only `cwd`, `env`, `dispatch`, and `binding` into the member-runtime lane in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1233).

That lane will likely need additive context such as:

- the resolved `WorldHandle`
- the session cgroup path
- overlay root or equivalent resolved world-root placement facts
- any explicit placement mode needed by the launcher

The exact carrier can be a new internal context struct. The SOW does not require a particular struct name.

### 4. Keep cancel, event, and completion semantics intact

The new placement-aware lane must preserve:

- `ExecuteStreamFrame::Start`
- streamed `Event`
- surfaced `agent_api.session.handle.v1`
- terminal `Exit` and `Error`
- `/v1/execute/cancel` delivery through the same `span_id`

Nothing in the placement fix should regress the already-landed remote cancel path in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1446).

### 5. Keep shell lifecycle logic unchanged except where placement failures become stricter

The shell side in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3353) should continue to:

- prepare the member participant
- persist `allocating`
- wait for surfaced session handle
- mark runtime ownership retained
- persist ready/failed/invalidation transitions

The only intended shell behavior change is that world-scoped member launch should now fail when true world placement cannot be established, instead of merely succeeding with metadata correlation.

## Installer And Provisioning Work

This slice is not just a code-path change. It changes the runtime contract of `world-agent`, so install and provisioning flows must be aligned.

### Why installer work is required

There are multiple independent authors of the effective `world-agent` runtime environment:

- dev install on Linux/macOS in [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:1482)
- production install on Linux in [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1932)
- Linux provisioning helper in [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:535)
- macOS Lima guest provisioning in [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:659)

Any new world-agent placement dependency that only lands in one of those paths will produce drift between:

- source checkout dev installs,
- release installs,
- direct Linux provisioning,
- and macOS guest provisioning.

There is already observable drift today:

- the release installer unit in [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1977) includes `SUBSTRATE_HOME` and a home-path `ReadWritePaths` entry,
- while the Linux provisioning helper in [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:564) includes `CAP_CHOWN`,
- and the WSL provisioning chain still writes `Group=root` and `SocketGroup=root` in [scripts/wsl/provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/wsl/provision.sh:37).

This SOW therefore treats installer alignment as part of the runtime correctness work, not as optional cleanup.

### Minimum installer/provisioning update set

The SOW requires review and update of both user-named install paths:

- [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:1482)
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1932)

And it requires lockstep review of the underlying provisioning writers they depend on:

- [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:535)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:659)

And it requires an explicit decision about the WSL path:

- [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1:248)
- [scripts/wsl/provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/wsl/provision.sh:37)

The WSL lane does not need Linux-first feature parity for this slice, but it does need an explicit posture. If the placement fix changes unit/runtime requirements, either:

- WSL is updated in the same change,
- or the implementation documents and enforces a fail-closed unsupported posture there until the WSL provisioning chain is aligned.

At minimum, those paths must be checked and updated for any new:

- installed binary or helper artifact needed by member placement
- systemd unit env vars
- `ReadWritePaths`
- `CapabilityBoundingSet`
- `AmbientCapabilities`
- runtime directories
- staging/caching of Linux guest binaries under dev-install

Even if the final implementation does not need a new binary, the service-unit duplication still requires explicit review so the runtime assumptions remain aligned.

### Specific installer concerns to cover in the implementation

1. `dev-install-substrate.sh`
- must keep the staged Linux guest bundle honest if the member-placement fix changes the Linux `world-agent` runtime requirements or artifacts
- must keep Linux provisioning and macOS Lima provisioning in sync
- must continue to disable world mode fail-closed when provisioning cannot establish the required runtime

2. `install-substrate.sh`
- must ship any new runtime artifact required for member placement
- must keep the generated `substrate-world-agent.service` and `.socket` consistent with the placement fix
- must not preserve an older capability set if the new path needs additional sanctioned world-placement privileges

3. `world-provision.sh` and `lima-warm.sh`
- must mirror the same unit-level runtime contract as the two higher-level installers
- must remain the authoritative service-unit templates for Linux host and macOS guest deployment

4. WSL provisioning
- must not be left silently stale if the final member-placement fix depends on service-unit contract changes
- must either adopt the same relevant runtime contract or be explicitly gated out by product posture and documentation

## Testing And Validation

### Existing tests that should remain green

- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs:186)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1576)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1682)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:2124)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:2268)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:3907)
- [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs:941)

These already cover lazy launch, same-generation reuse, replacement launch, replacement failure, remote cancellation semantics, and the higher-level status/trace surfaces that consume member-runtime truth.

### New tests required for this SOW

Add focused proof that the member runtime is actually placed into the world, not just tagged with world metadata.

At least one Linux-first test should prove one or more of:

- the spawned member process is attached to the authoritative session cgroup
- the spawned member process observes the session overlay/rootfs view rather than the daemon host view
- the spawned member process is constrained by the session network posture when isolation is enabled
- the runtime fails closed when placement cannot be established

The test should verify placement through observable runtime facts, not by inferring from participant metadata alone.

Support harnesses likely involved:

- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs:156)
- [crates/shell/tests/support/socket.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/socket.rs:1124)

### Suggested validation commands

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
```

If the placement fix adds a dedicated Linux-only proof test, that command must be added to this section and to the final implementation PR.

### Installer/provisioning verification

The implementation closeout should include at least:

- one Linux dev-install verification pass
- one Linux production-install or `world-provision.sh` verification pass
- one macOS Lima provisioning verification pass if the guest service-unit contract changes

Relevant reference docs:

- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:105)
- [docs/INSTALLATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/INSTALLATION.md:79)
- [docs/reference/world/verification/linux_world_socket.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/verification/linux_world_socket.md:1)
- [docs/reference/world/platforms/windows-wsl-setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/windows-wsl-setup.md:1)
- [docs/reference/world/platforms/macos-lima-setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:1)

## Acceptance Criteria

1. A selected world-scoped member still launches through `world-agent` over `/v1/execute/stream`.
2. The launched member runtime is actually placed inside the authoritative session world rather than merely labeled with `world_id` and `world_generation`.
3. `/v1/execute/cancel` still cancels the live member runtime by transport `span_id`.
4. Shell-owned participant persistence, readiness gating, invalidation, and replacement semantics remain authoritative and unchanged in shape.
5. Same-generation reuse and world-generation replacement continue to work.
6. World-scoped member startup fails closed when true world placement cannot be established.
7. Both [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:1482) and [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1932) are updated as needed, and their downstream provisioning paths are kept in sync with the final runtime contract.
8. Any WSL/macOS service-unit or provisioning drift introduced by the placement fix is either resolved in the same slice or explicitly documented and fail-closed.

## Non-Negotiable Guardrails

- Do not move runtime selection into `world-agent`.
- Do not reintroduce shell-local host fallback for a world-scoped member.
- Do not let participant metadata claim a world placement guarantee that the process launch path does not actually establish.
- Do not update only one installer/provisioning surface when the runtime contract changes.
