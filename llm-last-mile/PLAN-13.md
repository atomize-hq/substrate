# PLAN-13: Make World-Scoped Member Runtime Placement Real And Keep Runtime Contracts Aligned

Source plan: [PLAN-12.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-12.md)  
Source SOW: [13-member-runtime-world-placement-gap-sow.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/13-member-runtime-world-placement-gap-sow.md)  
Supersedes: `PLAN-12` as the next execution document after the transport-owned remote member cutover landed  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: Linux-first runtime-correctness plan, backend-only, installer-sensitive, no UI scope  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` depth, rewritten as one cohesive execution plan  
Status: execution-ready planning pass on 2026-05-03  
Outside voice: not used for this document generation

## Objective

`PLAN-12` closed the request-surface and remote-ownership gap. It did not close the placement gap.

Today the shell selects a world-scoped member, persists the right world binding, and launches that
member through `world-agent` over the existing `/v1/execute/stream` seam. But the launched runtime
still comes from daemon-local `gateway.run_control(...)` inside
[crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:40),
so the process is correlated to the session world without actually entering the authoritative
session world boundary.

This plan finishes the missing truth:

1. keep shell authority, request shape, and transport seam exactly as they are now,
2. make `world-agent` launch the member runtime inside the active session world for real,
3. make the shell fail closed if that placement cannot be established,
4. align Linux, macOS Lima, and WSL provisioning posture with the final runtime contract,
5. prove placement via observable runtime facts instead of metadata-only claims.

The user-visible outcome is simple:

- when a world-scoped member says it is live in world `W` generation `N`,
- it is actually executing inside world `W` generation `N`,
- under the same filesystem, cgroup, and network posture contract as other world execution,
- and `substrate agent status`, cancellation, and trace surfaces do not lie.

## Locked Starting State

### What is already done

The following work is already landed and must be treated as starting truth, not something to
reopen in this slice:

- the shell still owns canonical orchestration session and participant persistence
- the shell still resolves backend kind and binary path before dispatch
- the shell already sends world-scoped members through typed `member_dispatch` over
  `/v1/execute/stream`
- `world-agent` already validates the authoritative `world_id` and `world_generation`
- `execute_cancel` already has a remote member span path
- same-generation reuse and replacement semantics already exist at the shell lifecycle layer
- the crate-surface request bridge from `PLAN-12` is already open in
  [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs:35)

### Carry-forward bridge authority from `PLAN-12`

This placement plan does not reopen the transport problem, but it does carry forward the
anti-blocking authority from `PLAN-12` so the implementation cannot get stuck on the same seam
again.

The payload contract stays frozen, but the parent is explicitly authorized to thaw the
crate-surface request bridge before reopening lanes if a rebase or merge reveals that the shell can
no longer legally consume the already-frozen request builder through `crate::execution::*`.

The preferred fix remains:

- direct re-export of `MemberDispatchTransportRequest` through the allowed crate surface

The only pre-authorized fallback remains:

- one sanctioned adapter helper on the existing shell routing surface

Hard limits:

- no `MemberDispatchRequestV1` shape change
- no serialized payload change
- no second request-construction path in `async_repl.rs`
- no runtime-selection move into `world-agent`

### Exact gap

The remaining correctness gap is narrow and concrete:

1. `execute_stream(...)` ensures the authoritative world and validates the dispatch binding in
   [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1223).
2. The member lane then calls
   `MemberRuntimeManager::launch(agent_id, cwd, env, span_id, dispatch, binding)` in
   [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1238).
3. `MemberRuntimeManager::launch(...)` validates the runtime binary, builds an `AgentWrapperGateway`,
   then calls `gateway.run_control(...)` directly in
   [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:50).

That means the member runtime is remote-owned, but not world-placed.

### Why this is still unsafe

The current implementation can truthfully say:

- the member belongs to the current orchestration session
- the member belongs to the current `world_id`
- the member belongs to the current `world_generation`
- `world-agent` owns the retained control handle
- `/v1/execute/cancel` can reach it

It still cannot truthfully say:

- the member process is inside the authoritative session overlay/rootfs view
- the member process is attached to the session cgroup
- the member process sees the session network posture rather than the daemon host posture
- operator-visible world metadata matches the actual process boundary

That is the gap this plan closes.

## Frozen Execution Contract

This section is not optional interpretation space.

### Non-negotiable invariants

1. The shell remains the only canonical writer of orchestration session and participant state.
2. `world-agent` remains the transport owner for member startup, streaming, cancel, and terminal observation.
3. `MemberDispatchRequestV1` stays frozen. No request-shape churn in this slice.
4. `/v1/execute/stream` and `/v1/execute/cancel` remain the only transport seam.
5. `ExecuteStreamFrame::{Start,Event,Exit,Error}` remain the stream families.
6. World-scoped member launch fails closed if true placement cannot be established.
7. Same-generation reuse and replacement semantics remain shell-owned.
8. Backend selection stays in the shell. `world-agent` must not infer runtime from `backend_id`.
9. Linux-first remains explicit. macOS and WSL must either align their service/runtime contract or fail closed with clear posture.
10. Installer and provisioning drift is part of the bug, not separate cleanup.

### Chosen placement strategy

This plan chooses one concrete strategy:

- reuse the existing world-session discovery and cgroup-binding rules already used by the
  gateway-runtime path in
  [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1560)
  and
  [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:696)
- add a Linux-only placement-aware member launcher inside `world-agent`
- keep `agent_api` backend construction in `world-agent`
- move the actual child spawn under world-entry logic, rather than daemon-local direct spawn

The plan does not require the exact API to match `backend.exec(&world, ...)`, but it does require
equivalent guarantees:

- world already resolved and generation-validated
- child attached to the authoritative session cgroup
- child launched against the correct session filesystem view
- child subject to the active world network posture
- retained-control ownership, event forwarding, and cancel retention preserved

### Execution policy for this slice

This is a one-direction plan. The implementation is not allowed to drift into a hybrid state where
some members are merely metadata-bound and others are truly world-placed.

Allowed outcomes:

1. world-scoped member launch is observably world-placed and shell state reflects that truth
2. startup fails closed before the member is advertised as live

Disallowed halfway states:

- daemon-local launch with world metadata attached
- warning-and-continue startup for a selected world-scoped member
- installer parity asserted by docs only, without script or service proof
- "temporary" request-bridge churn that changes payload shape or duplicates request construction

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| typed member-dispatch contract | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) | Reuse exactly. No payload changes. |
| shell-side request construction and remote startup seam | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2829) | Reuse. Tighten fail-closed behavior only. |
| authoritative world validation before member dispatch | [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1223) | Reuse. Extend the post-validation carrier with placement context. |
| long-lived runtime cgroup attach pattern | [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:696) | Reuse as the placement reference, not as a new ownership model. |
| cancel delivery by `span_id` | [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1446), [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:102) | Reuse. Preserve the current transport truth. |
| installer and provisioning authors | [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh:1482), [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh:1932), [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh:535), [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:659), [scripts/wsl/provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/wsl/provision.sh:31) | Reuse the existing surfaces, but align them in the same slice. |

### 0B. Minimum diff decision

The smallest honest diff is:

1. add one internal placement-context carrier between `service.rs` and `member_runtime.rs`
2. change the member runtime launcher from daemon-local spawn to world-aware spawn
3. preserve request shape, stream framing, readiness contract, and shell authority
4. make the shell fail closed when placement cannot be established
5. update all service/provisioning writers that author the runtime contract
6. add proof tests for real placement, not just world metadata

Anything smaller leaves the lie in place. Anything larger risks reopening already-landed transport work.

### 0C. Complexity check

This slice touches more than 8 files. That is justified and still minimal.

The likely production path is:

1. `crates/world-agent/src/service.rs`
2. `crates/world-agent/src/member_runtime.rs`
3. `crates/world-agent/src/gateway_runtime.rs` or a new internal helper extracted from it
4. `crates/world-agent/src/lib.rs`
5. `crates/world-agent/tests/streamed_execute_cancel_v1.rs`
6. one new Linux-only world-placement proof test in `crates/world-agent/tests/`
7. `crates/shell/src/repl/async_repl.rs`
8. `crates/shell/tests/repl_world_first_routing_v1.rs`
9. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
10. `crates/shell/tests/agent_hub_trace_persistence.rs`
11. `scripts/substrate/dev-install-substrate.sh`
12. `scripts/substrate/install-substrate.sh`
13. `scripts/linux/world-provision.sh`
14. `scripts/mac/lima-warm.sh`
15. `scripts/wsl/provision.sh`
16. `scripts/windows/wsl-warm.ps1`
17. the relevant docs in `docs/WORLD.md`, `docs/INSTALLATION.md`, and cross-platform verification docs

That is a smell only if the slice invents new abstraction layers. It does not need to.

### 0D. Search and completeness check

Search-before-building result:

- **[Layer 1]** reuse the existing world-session and cgroup-attach logic instead of inventing a new isolation model
- **[Layer 1]** reuse the existing stream transport and cancel path
- **[Layer 1]** reuse the existing shell readiness gate keyed on the session-handle event
- **[Layer 3]** treat installer alignment as part of correctness because multiple scripts author the effective runtime contract

Shortcut options that are explicitly rejected:

- keep launching the member daemon-locally and document the mismatch
- infer placement from `world_id` metadata only
- update only Linux provisioning and leave dev-install or Lima stale
- keep the shell warning path that says "world-scoped member runtime unavailable" and then continues

### 0E. Distribution and runtime contract check

No new distributable artifact type is introduced, but the runtime contract changes anyway.

That means the plan must cover:

- service-unit capabilities
- service-unit `ReadWritePaths`
- `SUBSTRATE_HOME` or other required env propagation
- any cached Linux guest bundle used by macOS dev-install
- explicit WSL posture if it cannot be aligned in the same slice

### 0F. NOT in scope

- moving runtime selection into `world-agent`
- creating a second member-dispatch transport family
- redesigning shell-owned participant or orchestration-session persistence
- new public `substrate agent start|resume|fork|stop` productization work
- cross-platform feature parity beyond explicit runtime-contract posture
- unrelated status or doctor UX redesign
- extracting a shared installer template generator unless the duplication itself blocks correctness

## Architecture Review

### Findings

`[P1] (confidence: 10/10) crates/world-agent/src/member_runtime.rs:50-61 - member launch still uses daemon-local gateway.run_control(...), so a runtime can be tagged with the active world binding without inheriting the world boundary itself.`

Recommendation:

- replace direct local spawn with a world-aware retained-control launcher
- keep stream framing and cancel registry intact
- do not treat metadata validation as a substitute for world entry

`[P1] (confidence: 9/10) crates/world-agent/src/service.rs:1233-1247 - the member-dispatch lane passes only cwd, env, dispatch, and binding into the runtime manager, which is not enough to attach the child to the session cgroup or guarantee the session filesystem/network view.`

Recommendation:

- introduce one internal placement carrier resolved at the service layer
- include the world handle or equivalent resolved facts needed for cgroup, filesystem, and network placement
- keep that carrier internal to `world-agent`

`[P1] (confidence: 9/10) crates/shell/src/repl/async_repl.rs:3406-3414 - the shell still prints a warning and continues when the world-scoped member runtime is unavailable, but after this slice that state becomes a fail-closed correctness failure, not a degradable warning.`

Recommendation:

- convert placement failure into a startup failure for selected world-scoped members
- keep the REPL alive, but do not advertise the member as available or silently fall back

`[P1] (confidence: 9/10) scripts/substrate/install-substrate.sh:1932, scripts/linux/world-provision.sh:535, scripts/mac/lima-warm.sh:659, scripts/wsl/provision.sh:31 - multiple independent scripts author the world-agent runtime contract today, and they already drift on group/capability/home-path details, so a placement-sensitive change can easily ship half-applied.`

Recommendation:

- update all contract authors in the same slice
- add verification steps that compare the resulting service posture, not just whether the binary exists
- choose and document explicit WSL posture if parity is deferred

### Ownership split

| Concern | Owner | Why |
| --- | --- | --- |
| runtime selection, backend kind, binary path | shell | Already resolved there. Re-resolving creates drift. |
| authoritative world selection and generation validation | `world-agent` service layer | That is where the active world is proven. |
| actual child placement into the world | `world-agent` runtime layer | That is the missing truth this slice lands. |
| retained control, event streaming, cancel registry | `world-agent` runtime layer | Already remote-owned. Must remain there. |
| participant persistence, readiness transitions, replacement decisions | shell | Canonical state stays shell-owned. |
| service-unit/runtime-contract authoring | installers and provisioning scripts | Those scripts create the effective runtime environment. |

### Architecture ASCII diagrams

```text
CURRENT STATE
=============
shell
  ├── selects member runtime
  ├── persists world_id/world_generation
  └── dispatches typed member request over /v1/execute/stream
         │
         ▼
world-agent service
  ├── ensures authoritative session world
  ├── validates dispatch world binding
  └── calls MemberRuntimeManager::launch(...)
         │
         ▼
member_runtime.rs
  ├── build AgentWrapperGateway
  └── gateway.run_control(...)
         │
         ▼
daemon-local child process

Result:
  remote-owned, but not guaranteed world-placed
```

```text
TARGET STATE
============
shell
  ├── selects runtime and persists allocating state
  ├── dispatches typed member request
  └── waits for remote session-handle evidence
         │
         ▼
world-agent service
  ├── ensures authoritative session world
  ├── validates world_id/world_generation
  ├── resolves placement context
  │     ├── world identity
  │     ├── cgroup path
  │     ├── filesystem / overlay facts
  │     └── network posture
  └── calls placement-aware member launcher
         │
         ▼
member runtime launcher
  ├── builds backend from resolved_runtime
  ├── spawns inside session world
  ├── attaches child to session cgroup
  ├── registers span_id for cancel
  ├── forwards Start / Event / Exit / Error
  └── fails closed if placement cannot be established
         │
         ▼
shell
  ├── marks Ready only after session-handle event
  └── never advertises live state on placement failure
```

```text
RUNTIME CONTRACT AUTHORS
========================
release install      dev install            direct provision          macOS Lima           WSL
install-substrate -> dev-install-substrate -> world-provision.sh -> lima-warm.sh -> wsl/provision.sh
         \___________________________ all must agree on ___________________________/
             capabilities, RW paths, socket ownership, env, binary staging, posture
```

## Code Quality Review

### Findings

`[P2] (confidence: 9/10) crates/world-agent/src/service.rs + crates/world-agent/src/gateway_runtime.rs - the cgroup attach and world-binding preparation logic already exists in one long-lived runtime path, so duplicating that behavior independently for member runtime placement would create a second isolation truth.`

Recommendation:

- extract or mirror one internal helper from the gateway-runtime path
- keep the member-specific behavior in `member_runtime.rs`
- do not fork a second isolation-preparation implementation

`[P2] (confidence: 8/10) scripts/substrate/install-substrate.sh + scripts/linux/world-provision.sh + scripts/mac/lima-warm.sh + scripts/wsl/provision.sh - service-unit duplication is already real, but introducing a brand-new shared generator in this slice would be over-engineering unless the current duplication blocks correctness.`

Recommendation:

- update all existing authors in lockstep now
- add parity assertions or smoke verification
- defer template extraction unless the same fields are still drifting after this slice

`[P2] (confidence: 8/10) crates/shell/src/repl/async_repl.rs:3406-3414 - the shell currently conflates "member runtime unavailable" with a soft warning path, which is the wrong abstraction once placement becomes mandatory.`

Recommendation:

- split "transport setup failed" from "placement contract failed"
- make the latter a concrete persisted failure state with actionable error messaging

### Allowed code shape

1. One new internal placement-context carrier is allowed.
2. One internal world-aware launcher or helper extraction is allowed.
3. No new public request family.
4. No duplicate runtime selector.
5. No duplicate cgroup/world-setup implementation if existing helpers can be reused.
6. No silent installer drift.

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Primary crates: `world-agent`, `shell`
- No prompt or LLM eval suite is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] world-agent service -> member runtime handoff
    │
    ├── [GAP]         binding validation passes and placement context resolves
    ├── [GAP]         missing cgroup/world facts fail before child spawn
    └── [GAP]         world mismatch still fails before startup

[+] placement-aware member launch
    │
    ├── [GAP] [->E2E] child observes session filesystem view, not daemon host view
    ├── [GAP]         child attaches to authoritative session cgroup
    ├── [GAP]         isolated-network session constrains member runtime
    ├── [GAP]         missing binary fails closed
    ├── [GAP]         unsupported backend kind fails closed
    └── [GAP]         cancel reaches the live placed member span

[+] shell lifecycle truth
    │
    ├── [GAP]         selected world-scoped member fails closed on placement failure
    ├── [GAP]         Ready requires session-handle evidence from the placed runtime
    ├── [GAP]         same-generation reuse still does not relaunch
    ├── [GAP]         replacement launch still preserves lineage after placement work
    └── [GAP]         failed replacement never revives stale generation liveness

[+] installer / provisioning contract
    │
    ├── [GAP]         Linux dev-install provisions a compatible world-agent contract
    ├── [GAP]         Linux release install provisions the same relevant contract
    ├── [GAP]         macOS Lima guest service contract matches Linux where required
    └── [GAP]         WSL is either aligned or explicitly fail-closed with docs

[+] operator surfaces
    │
    ├── [GAP]         status reflects the real placed producer
    └── [GAP]         trace rows do not overclaim placement on failure paths

─────────────────────────────────
COVERAGE: 0/16 placement-truth paths proven
GAPS: 16 paths require coverage before closeout
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First world-backed member launch
    ├── [GAP] [->E2E] shell selects member -> remote launch -> session-handle -> Ready
    ├── [GAP]         placement failure produces failed startup, not warning-and-continue
    └── [GAP]         same-generation reuse remains no-op

[+] Live member cancel
    ├── [GAP] [->E2E] execute_cancel stops the placed runtime by span_id
    ├── [GAP]         cancel before span registration is reported clearly
    └── [GAP]         terminal state is honest after cancel

[+] Shared-world generation rollover
    ├── [GAP] [->E2E] replacement runtime launches in the replacement world
    ├── [GAP]         replacement placement failure leaves no authoritative-live successor
    └── [GAP]         stale generation never regains liveness

[+] Operator verification
    ├── [GAP]         status shows remote producer truth
    ├── [GAP]         trace remains participant-correct after cancel
    └── [GAP]         trace does not claim world placement if launch failed before entry
```

### Required tests to add or extend

1. `crates/world-agent/tests/streamed_execute_cancel_v1.rs`
   - keep existing remote cancel proof
   - extend with placed-member success, binding mismatch, missing binary, unsupported backend, and cancel cases

2. New Linux-only proof test in `crates/world-agent/tests/`
   - prove real placement using observable facts
   - acceptable proof styles:
     - session cgroup membership
     - overlay/rootfs view rather than daemon-host view
     - network posture restrictions when isolation is enabled
   - do not infer placement from metadata alone

3. `crates/shell/tests/repl_world_first_routing_v1.rs`
   - assert fail-closed startup when placement cannot be established
   - keep same-generation reuse and replacement semantics green
   - assert Ready is still gated on the session-handle event

4. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
   - assert status truth for a real placed member runtime
   - ensure stale generation does not win over live runtime truth

5. `crates/shell/tests/agent_hub_trace_persistence.rs`
   - assert launch, cancel, failure, and replacement trace truth after placement changes

### Failure-mode test rule

Any path that can simultaneously satisfy all three conditions below is a critical gap and must get a test:

1. no placement proof
2. no clear error handling
3. user-facing or operator-facing state still suggests the member is live

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| placement context resolution | service validates world metadata but cannot resolve cgroup/world facts | no | partial | partial | yes |
| placed child spawn | runtime still starts on daemon host even though dispatch binding is valid | no | no | no | yes |
| cgroup attach | child spawns but never enters session cgroup | no | partial | no | yes |
| filesystem view | child sees daemon-host files instead of session overlay/rootfs | no | no | no | yes |
| isolated network posture | child escapes session network restrictions | no | no | no | yes |
| shell fail-closed path | placement failure is downgraded to warning and loop continues | no | partial | partial | yes |
| cancel path | cancel misses the placed runtime span and state remains live | no | partial | partial | yes |
| replacement path | replacement placement fails and stale generation remains authoritative-live | no | partial | no | yes |
| installer drift | one install path has the required capability/env/path contract and another does not | no | no | no | yes |
| WSL posture | WSL silently keeps an incompatible root/root socket contract | no | no | no | yes |

## Performance Review

This is a correctness-first slice, but a few performance rules still matter:

1. keep using the existing request transport and streaming path
2. reuse the current session world rather than creating a fresh world per member launch
3. preserve same-generation reuse so steady-state commands do not relaunch the member
4. avoid re-deriving expensive world facts more than once per launch if the service layer can carry them down

There is no new throughput feature here. The performance risk is accidental relaunch or accidental repeated world preparation.

## DX Guardrails

This is still a developer tool, so failures must explain themselves.

Required error-message posture:

1. placement failures must say whether the failure happened before spawn, during world entry, during cgroup attach, or after stream establishment
2. shell-visible errors must include `participant_id`, `world_id`, `world_generation`, and backend kind when available
3. installer failures must name the exact contract element that is missing, such as capability, env var, RW path, or socket/group posture
4. WSL or macOS non-parity must be explicit in docs and doctor output, not implicit behavior

## Worktree Parallelization Strategy

There is a real parallel window here, but only after the parent-owned preflight is frozen.
That preflight includes the placement contract and the carry-forward bridge safeguard above.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| P0. Freeze placement contract, carrier shape, and bridge safeguard | `crates/world-agent/`, `crates/shell/src/repl/`, `crates/shell/src/execution/`, planning docs | - |
| L1. World-agent placement core | `crates/world-agent/` | P0 |
| L2. Shell fail-closed and status/trace lane | `crates/shell/src/repl/`, `crates/shell/tests/` | P0 |
| L3. Installer and provisioning alignment | `scripts/`, `docs/` | P0 |
| P1. Parent integration and proof wall | `crates/world-agent/tests/`, `crates/shell/tests/`, `scripts/`, `docs/` | L1, L2, L3 |

### Parallel lanes

- Lane A: `P0`, sequential, parent-owned
- Lane B: `L1`, world-agent placement core
- Lane C: `L2`, shell lifecycle and regression wall preparation
- Lane D: `L3`, installer/provisioning and docs alignment
- Lane E: `P1`, sequential integration and final proof

### Execution order

1. Parent freezes the placement strategy, the internal carrier shape, and confirms the shell-side
   request bridge is still legally reachable through the sanctioned crate surface.
2. Launch `L1`, `L2`, and `L3` in parallel worktrees.
3. Merge the world-agent lane first, because it defines the final runtime behavior.
4. Merge shell and installer lanes after they rebase on that reality.
5. Run the parent-owned proof wall and closeout.

### Conflict flags

- `L1` and `L2` must not change `MemberDispatchRequestV1` or request construction
- only the parent may reopen the sanctioned shell bridge safeguard if rebases reveal drift there
- `L2` must not invent a second readiness contract
- `L3` must not silently redefine capability or env requirements without feeding those back into `L1`
- if `L1` needs changes in `crates/agent-api-types/`, stop and escalate
- if `L3` discovers a new helper binary or artifact requirement, update the plan and rerun the dependency check before merging

### Parallelization verdict

Three independent workstreams exist after one parent-owned contract freeze. Worker cap can be `3`.

## Implementation Sequence

### Step 1. Freeze the placement contract and re-confirm the bridge safeguard

Files:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/routing.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing.rs)
- [crates/shell/src/execution/routing/dispatch/prelude.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/prelude.rs)

Work:

1. define one internal placement carrier resolved at the service layer
2. explicitly choose the placement helper reuse path from gateway-runtime logic
3. re-confirm that the shell can still reach the already-frozen request builder through the
   sanctioned crate surface
4. if that bridge has drifted, the parent applies the pre-authorized fix before reopening worker
   lanes:
   - preferred: direct re-export of `MemberDispatchTransportRequest`
   - fallback: one sanctioned adapter helper
5. lock the shell fail-closed posture for selected world-scoped members
6. record the WSL posture decision: aligned in-slice or explicitly unsupported/fail-closed

Validation gate:

- no payload changes
- no transport changes
- sanctioned request bridge still reachable, or parent-fixed without payload churn
- one agreed internal carrier shape
- one agreed fail-closed behavior

### Step 2. World-agent placement core

Files:

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- `crates/world-agent/tests/streamed_execute_cancel_v1.rs`
- one new Linux-only world-placement proof test

Work:

1. resolve authoritative placement facts after world validation
2. route member launch through a placement-aware launcher
3. reuse cgroup attach and runtime-preparation rules from the gateway-runtime path where possible
4. keep `span_id`, `Start`, `Event`, `Exit`, and `Error` behavior intact
5. preserve the existing remote ownership model, not a new shell-owned retained-control path
6. fail closed on missing binary, unsupported backend, world mismatch, or placement setup failure

Validation gate:

- live member runtime is observably placed in the session world
- cancel still reaches the placed runtime
- metadata and runtime truth cannot diverge silently

### Step 3. Shell fail-closed and lifecycle truth

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
- `crates/shell/tests/agent_hub_trace_persistence.rs`

Work:

1. replace the warning-and-continue path with explicit startup failure for selected world-scoped members
2. preserve Allocating -> Ready gating on session-handle evidence only
3. keep same-generation reuse behavior unchanged
4. keep replacement lineage and invalidation semantics unchanged in shape
5. update status and trace assertions to reflect real placed-runtime truth
6. forbid fallback to host-local retained control once a world-scoped member has been selected

Validation gate:

- placement failure never yields an authoritative-live member
- Ready and Running still require remote evidence
- replacement failure never revives stale generation

### Step 4. Installer and provisioning alignment

Files:

- [scripts/substrate/dev-install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-install-substrate.sh)
- [scripts/substrate/install-substrate.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/install-substrate.sh)
- [scripts/linux/world-provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/linux/world-provision.sh)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
- [scripts/wsl/provision.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/wsl/provision.sh)
- [scripts/windows/wsl-warm.ps1](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/windows/wsl-warm.ps1)
- the matching docs in `docs/`

Work:

1. align service-unit capabilities, env, RW paths, group/socket posture, and binary staging with the final placement contract
2. ensure Linux dev-install and Linux release install land the same relevant runtime assumptions
3. ensure macOS Lima guest provisioning mirrors Linux where required
4. either align WSL or document and enforce fail-closed unsupported posture
5. update docs so operators know how to verify the contract

Validation gate:

- there is no silent drift between Linux dev-install, Linux release install, direct Linux provision, and macOS guest provisioning
- WSL posture is explicit

### Step 5. Parent proof wall and closeout

Work:

1. run the targeted Rust tests in order
2. run one Linux dev-install verification pass
3. run one Linux release-install or `world-provision.sh` verification pass
4. run one macOS Lima verification pass if the guest service contract changed
5. record the WSL result or explicit deferral posture
6. update docs and closeout evidence

Closeout is not done until the plan has runtime proof, installer proof, and operator-facing documentation proof.

## Recommended Verification Commands

Run in this order. Do not skip forward. The parent proof wall is not complete until the Rust test
wall is green and the affected install/provision paths have been exercised for the platforms they
claim to support.

```bash
cargo test -p world-agent --test streamed_execute_cancel_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
cargo test -p shell --test agent_hub_trace_persistence --no-run
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test agent_hub_trace_persistence -- --nocapture
scripts/linux/world-provision.sh --profile release
scripts/mac/lima-warm.sh
pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
```

If a dedicated Linux-only placement proof test is added, insert it between the world-agent stream
test and the shell test wall.

Platform posture for this pass:

- Linux verification is required
- macOS Lima verification is required if the guest service contract or staged runtime changes
- WSL verification is required if the slice claims WSL alignment; otherwise the proof is the
  explicit fail-closed posture plus docs update

## Definition of Done

1. world-scoped member launch still uses `/v1/execute/stream`
2. the launched runtime is observably inside the authoritative session world
3. `/v1/execute/cancel` still cancels the live member by `span_id`
4. shell-owned persistence, readiness, invalidation, and replacement semantics remain authoritative
5. same-generation reuse still works
6. placement failure is fail-closed, not warning-and-continue
7. Linux dev-install, Linux release install, and Linux direct provision are aligned for the final runtime contract
8. macOS Lima is aligned where required, or explicitly documented if not
9. WSL is either aligned or explicitly fail-closed with documentation
10. status and trace surfaces do not overclaim placement

The run is not done if the code is correct but the sanctioned request bridge regressed and was left
to "follow-up later." That is the same class of stall this plan is meant to prevent.

## Deferred Work

- broader public agent lifecycle productization
- cross-platform feature parity beyond explicit runtime-contract posture
- installer templating cleanup if duplication remains painful after this slice
- status and doctor UX polish unrelated to placement truth

No new `TODOS.md` entry is required yet. These are explicit deferrals, not forgotten work.

## Completion Summary

- Step 0: scope accepted as a narrow post-`PLAN-12` correctness follow-on
- Architecture Review: 4 core issues found and resolved in-plan
- Code Quality Review: 3 structural cautions found and resolved in-plan
- Test Review: diagram produced, 16 placement-truth gaps identified
- Performance Review: 4 correctness-preserving cautions, 0 new throughput features
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 durable TODOs proposed
- Failure modes: 10 critical gaps identified for implementation proof
- Outside voice: not used for this document generation
- Parallelization: 5 execution phases, 3 parallel lanes after one parent-owned freeze
- Lake Score: complete option chosen over metadata-only placement, partial installer updates, or warning-path fallback

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Treat this as a post-`PLAN-12` placement-truth slice, not a transport redesign | Mechanical | Minimal diff | The request bridge and remote ownership are already landed | Reopening payload or transport work |
| 2 | Placement strategy | Reuse gateway-runtime world/cgroup preparation patterns for member launch | Taste | Explicit over clever | One existing runtime path already solves the hard placement problems | Inventing a separate world-entry system |
| 3 | Shell posture | Convert selected world-scoped member placement failure into fail-closed startup failure | Mechanical | Completeness | Warning-and-continue would preserve a correctness lie | Silent warning fallback |
| 4 | Installer scope | Update all runtime-contract authors in the same slice | Mechanical | Boil the lake | Partial installer updates would ship inconsistent truth | Linux-only script edits |
| 5 | WSL | Force an explicit aligned-or-fail-closed posture | Mechanical | Explicit over clever | Silent drift is worse than unsupported | Leaving WSL ambiguous |
| 6 | Tests | Require observable placement proof, not metadata-only assertions | Mechanical | Systems over heroes | Operator trust depends on runtime facts | Trusting `world_id` fields alone |
| 7 | Parallelization | Freeze the contract first, then run world-agent, shell, and installer lanes in parallel | Taste | Pragmatic | That is the widest parallel window with contained merge risk | Fully serial execution or uncontrolled overlap |
