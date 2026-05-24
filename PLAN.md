# PLAN: Shared Dispatch Contract Closeout And Parity Hardening

Source SOW: [29.5-shared-dispatch-contract-closeout-and-parity-hardening.md](llm-last-mile/29.5-shared-dispatch-contract-closeout-and-parity-hardening.md)  
Primary code anchors: [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs), [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agent_runtime/validator.rs](crates/shell/src/execution/agent_runtime/validator.rs), [crates/shell/src/execution/agent_inventory.rs](crates/shell/src/execution/agent_inventory.rs), [crates/shell/src/execution/policy_model.rs](crates/shell/src/execution/policy_model.rs), [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs), [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs), [crates/shell/src/execution/routing/dispatch/world_ops.rs](crates/shell/src/execution/routing/dispatch/world_ops.rs)  
Primary test anchors: [crates/shell/tests/agent_public_control_surface_v1.rs](crates/shell/tests/agent_public_control_surface_v1.rs), [crates/shell/tests/repl_world_first_routing_v1.rs](crates/shell/tests/repl_world_first_routing_v1.rs), [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](crates/shell/tests/agent_successor_contract_ahcsitc0.rs), tests in [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)  
Adjacent slices: [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md), [31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Execution branch: `feat/gateway-mediated-llm-fulfillment`  
Base branch: `main`  
Plan type: contract-closeout slice, no UI scope, strong DX scope  
Review posture: unified execution plan tightened to `/autoplan` and `/plan-eng-review` rigor  
Status: implementation-ready planning pass on 2026-05-24

## Objective

Make slice 29 true enough that slices 30 and 31 can build on one shared dispatch contract and one durable host-attach contract without reopening semantics.

This slice is complete only when the repo has one contract floor that does all of the following:

1. inventory-backed launches resolve through the shared contract and produce merged effective policy, accepted bounded overrides, and truthful provenance;
2. persisted host attach launches reuse durable attach truth instead of reconstructing it from ambient runtime state;
3. `HostAttachContract` is authoritative for attach-relevant capabilities and attach knobs, not a partial shadow of launch truth;
4. retained orchestrator member turns no longer bypass the shared contract vocabulary through hidden manifest-only semantics;
5. docs, tests, and downstream slice assumptions all describe the same supported override families and the same parity story.

This is a closeout slice, not a new product slice. It does not ship public `agent start --scope world`. It does not ship lazy attach. It does not create a second durable object.

## Acceptance Criteria

This plan is only done when all of the following are true in code, tests, and docs:

1. a host-scoped resolved launch contract can be persisted into `HostAttachContract` without losing attach-relevant capability truth or attach-launch knobs;
2. `resolve_persisted_host_attach_contract(...)` reuses persisted capabilities and attach knobs from `HostAttachContract` instead of hardcoding permissive defaults;
3. inventory-backed resolution merges validated `policy_overlay` data into `ResolvedLaunchContract.effective_policy` as narrowing-only truth;
4. bounded dispatch-time capability narrowing is genuinely supported for the explicitly approved family in this plan, and all other capability override families fail closed with truthful diagnostics;
5. retained orchestrator member turns consume a resolved subset derived from the shared contract instead of reconstructing launch semantics from live manifest state alone;
6. equivalent human and orchestrator-controlled launches with equivalent baseline truth produce equivalent resolved contract truth for backend, protocol, scope, capabilities, attach knobs, and effective policy;
7. slice 30 can expose public scope and capability flags without inventing a second override model;
8. slice 31 can trust the durable attach contract for continuity-vs-fresh attach inputs without re-deriving launch truth.

## Locked Decisions

These decisions are now frozen. Implementation does not get to keep both branches open.

| Topic | Locked decision | Why |
| --- | --- | --- |
| Override closeout direction | Implement bounded dispatch-time capability narrowing now, not docs-only narrowing | Slice 30 needs a truthful override model to expose publicly; scaffolding-only fields are not enough |
| Supported capability override family | In 29.5, inventory-backed launches may narrow `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream` from `true` to `false` | These are the attach-relevant and control-surface-relevant capabilities already modeled by `HostAttachContract` and the state store |
| Unsupported capability override family | `session_start`, `llm`, and `mcp_client` remain dispatch-time unsupported in 29.5 and must fail closed with field-scoped diagnostics | `session_start` is not a meaningful per-dispatch narrowing knob for a start path, and `llm` / `mcp_client` would broaden the slice into product behavior and runtime capability semantics |
| Scope override posture | `requested_execution_scope_override` remains narrowing-only and may not change baseline scope in 29.5 | The current resolver already rejects scope drift; 30 owns the public scope surface |
| Policy overlay merge posture | Inventory `policy_overlay` must be applied into `effective_policy` using shared patch semantics and remain restriction-only | Validation without merge is a lie; merge without restriction-only semantics is unsafe |
| Durable attach object | `HostAttachContract` remains the only durable attach object | A second durable attach structure would reopen the exact semantic split this slice exists to close |
| Retained member-turn parity shape | Retained orchestrator member turns will consume a persisted or already-resolved subset derived from the shared contract, not a fresh full-envelope resolution on every follow-up turn | Retained turns are transport reuse, not new baseline selection; the subset approach preserves parity without re-running inventory selection at the wrong layer |
| Sync semantics | `sync_host_attach_contract(...)` is allowed to refresh continuity-only runtime state, not baseline launch truth | Birth-time launch truth should not drift opportunistically as the session runs |
| Successor semantics | `fork_successor_attach_contract(...)` copies generalized truth forward and clears only continuity-specific state | This is the smallest correct rule for later slice 31 fresh-attach behavior |

## Scope

### In scope

1. generalize `HostAttachContract` so it is derived from `ResolvedLaunchContract` and remains authoritative for attach-relevant truth;
2. apply validated inventory `policy_overlay` patches into `ResolvedLaunchContract.effective_policy`;
3. implement bounded capability override narrowing for the approved capability family in inventory-backed resolution;
4. keep unsupported capability override families explicit and fail closed;
5. route retained orchestrator member-turn semantics through a shared-contract-derived subset;
6. extend tests and truth docs until 29, 29.5, 30, and 31 all describe the same contract floor.

### Out of scope

1. public `substrate agent start --scope world`;
2. lazy host attach trigger policy;
3. born-unattached posture UX beyond the fields 31 will later consume;
4. adding a new crate or a second orchestration state model;
5. redesigning transport APIs when the existing typed member-dispatch request can carry the shared-contract-derived subset;
6. widening policy semantics to allow dispatch-time broadening.

## Step 0: Scope Challenge

### 0A. What already exists

The repo already has the important foundation. This slice is not architecture discovery.

| Sub-problem | Existing code | Reuse decision |
| --- | --- | --- |
| Shared launch vocabulary | `DispatchRequestEnvelope`, `ResolvedLaunchContract`, `DispatchCallerKind`, `DispatchBaselineKind` in [dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs) | Reuse and finish. Do not create a second resolver surface. |
| Inventory-backed baseline resolution | `resolve_inventory_contract_for_exact_backend(...)`, `resolve_inventory_contract_for_unique_scope(...)` in [dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs) | Reuse, then add overlay merge and bounded capability narrowing. |
| Durable attach seam | `HostAttachContract`, `sync_host_attach_contract(...)`, `fork_successor_attach_contract(...)` in [orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse, but derive from resolved truth instead of manifest-era defaults. |
| Attach-time selection gate | `resolve_public_control_target(...)` in [state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse. It already enforces `supports_resume`, `supports_fork`, `supports_stop`, and continuity gating. |
| Attach launch planning | `build_attach_launch_plan(...)` in [agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs) | Reuse, but make persisted attach truth actually authoritative. |
| Retained member transport | `build_member_dispatch_transport_request(...)` in [async_repl.rs](crates/shell/src/repl/async_repl.rs) plus typed request handling in [world_ops.rs](crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse transport. Replace hidden launch semantics only. |
| Policy patch model | `PolicyPatch` and internal patch application logic in [policy_model.rs](crates/shell/src/execution/policy_model.rs) | Reuse semantics. Extract or expose a crate-private helper rather than inventing parallel merge logic. |
| Overlay validation | `validate_policy_overlay(...)` in [agent_inventory.rs](crates/shell/src/execution/agent_inventory.rs) | Reuse. Keep this as the narrowing-only gate. |
| Runtime realizability | `materialize_runtime_descriptor(...)` in [validator.rs](crates/shell/src/execution/agent_runtime/validator.rs) | Reuse exactly. This slice is not a runtime materialization redesign. |

### 0B. Exact gaps being closed

The code proves the SOW right:

1. persisted attach resolution still returns `Policy::default()` and hardcoded permissive capabilities in [dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs);
2. inventory `policy_overlay` is validated in [agent_inventory.rs](crates/shell/src/execution/agent_inventory.rs) but is not applied to `ResolvedLaunchContract.effective_policy`;
3. `validate_dispatch_overrides(...)` currently rejects every capability override with "frozen but not supported";
4. `HostAttachContract::from_manifest(...)` still derives from manifest/runtime defaults rather than a resolved host contract;
5. `sync_host_attach_contract(...)` only refreshes continuity selector state, which is correct, but that means birth-time contract truth must be complete;
6. retained member-turn transport in [async_repl.rs](crates/shell/src/repl/async_repl.rs) still builds its request from `PreparedAgentRuntime` and live manifest data without one explicit shared-contract-derived parity object.

### 0C. Minimum honest change

The minimum honest implementation is:

1. keep one resolver module;
2. add one crate-private policy-overlay merge helper reusable by the resolver;
3. add one resolved-to-persisted conversion path for `HostAttachContract`;
4. add one crate-private retained-turn parity builder that reconstructs the required subset from participant/session fields already persisted at cold start from resolved contract truth;
5. add tests proving overlay merge, bounded capability narrowing, persisted attach authority, and retained-turn parity.

Anything smaller leaves 29.5 half-claimed and forces 30 or 31 to rediscover semantics.

### 0D. Complexity check

This slice is a real complexity smell on paper:

1. it touches dispatch resolution, persistent orchestration state, state-store gates, CLI attach planning, REPL member dispatch, tests, and docs;
2. it will touch more than 8 files;
3. it can easily sprawl into public CLI design if discipline slips.

We keep it engineered enough by freezing these constraints:

1. no new crate;
2. no new public CLI verb;
3. no public transport redesign;
4. no new persisted parity object; reconstruct retained-turn parity from the existing persisted participant/session fields already written at cold start;
5. docs only update the truth surface directly affected by these contract semantics.

### 0E. Completeness and distribution check

Completeness wins here.

The shortcut version is to fix the docs and keep the code mostly as-is. That saves almost no AI-assisted implementation time and guarantees slice 30 or 31 reopens the same seams.

The complete version is:

1. make durable attach truth real;
2. make overlay merge real;
3. make bounded capability narrowing real;
4. make retained-turn parity real;
5. prove it with targeted tests.

No new binary, package, container, or artifact type is introduced, so distribution work is not applicable.

### 0F. NOT in scope

1. public flag UX design for every future capability family;
2. a generic "policy patch engine" public API;
3. re-resolving inventory on every retained member follow-up turn;
4. retrofitting `HostAttachContract` into a generic policy persistence object;
5. world-worker lazy attach launch behavior;
6. standalone world-root continuity.

## Architecture Contract

### Thesis

The repo already has the right seams. The missing piece is truthful contract ownership.

After 29.5:

1. inventory-backed launches resolve once through the shared contract;
2. host attach launches resolve from durable attach truth, not ambient runtime snapshots;
3. retained member turns use a contract-derived subset rather than a second dialect;
4. policy and capability narrowing are visible in the resolved contract and survive into later consumers where required.

### Baseline domains

There are exactly two baseline domains. This slice does not allow a third.

| Domain | Used by | Baseline source | 29.5 behavior |
| --- | --- | --- | --- |
| Inventory launch | human `start`, orchestrator member cold start, future public world start | effective inventory + effective config + validated inventory `policy_overlay` | may accept bounded capability narrowing; may not broaden scope or policy |
| Persisted host attach | `reattach`, `fork`, detached turn attach planning, future lazy attach | durable `HostAttachContract` under the orchestration session | may change continuity-vs-fresh attach selection only; may not reconstruct launch truth from ambient state |

### Target architecture

```text
INVENTORY-BACKED LAUNCH
=======================
caller
  |
  v
DispatchRequestEnvelope
  |
  v
dispatch_contract.rs
  1. select inventory baseline
  2. validate bounded override family
  3. apply accepted capability narrowing
  4. merge validated policy_overlay into effective_policy
  5. emit ResolvedLaunchContract + provenance
  |
  +--> materialize_runtime_descriptor(...)
  +--> host attach persistence for host-scoped owner sessions
  `--> member dispatch parity subset for world members


PERSISTED ATTACH LAUNCH
=======================
caller
  |
  v
resolve_public_control_target(...)
  |
  v
HostAttachContract
  |
  v
resolve_persisted_host_attach_contract(...)
  1. trust persisted backend/protocol/scope/runtime descriptor
  2. trust persisted attach-relevant capabilities
  3. trust persisted attach_launch_knobs baseline
  4. apply attach-mode request checks only
  |
  `--> materialize_runtime_descriptor(...)


RETAINED MEMBER TURN
====================
member cold start
  |
  v
ResolvedLaunchContract
  |
  v
MemberDispatchParitySubset   <-- chosen 29.5 parity shape
  |
  v
build_member_dispatch_transport_request(...)
  |
  `--> typed transport stays, hidden launch dialect goes away
```

### Authoritative fields after 29.5

#### `ResolvedLaunchContract`

Remains authoritative for:

1. `backend_id`
2. `backend_kind`
3. `protocol`
4. `execution_scope`
5. runtime descriptor inputs
6. bounded accepted capabilities
7. attach launch knobs
8. merged effective policy
9. field provenance

#### `HostAttachContract`

Must become authoritative for:

1. `backend_id`
2. `execution_scope`
3. `protocol`
4. `launch_descriptor`
5. attach-relevant capabilities:
   - `session_resume`
   - `session_fork`
   - `session_stop`
   - `status_snapshot`
   - `event_stream`
6. attach launch knobs
7. continuity selector state when present

It is intentionally not required to persist full `Policy`, `session_start`, `llm`, or `mcp_client` in 29.5.

#### `MemberDispatchParitySubset`

The retained-turn parity subset is reconstructed from existing persisted participant/session fields written at cold start. It may use a crate-private helper type in memory, but it does not introduce a new persisted parity object in 29.5.

It must contain exactly the fields retained member dispatch needs from already-persisted resolved truth:

1. `backend_id`
2. `protocol`
3. `backend_kind`
4. `binary_path`
5. `execution_scope`
6. capability truth needed for follow-up authorization where relevant
7. orchestration/world linkage fields already required by typed transport

It must not perform inventory selection, config fallback, or policy re-resolution on retained follow-up turns.

## Implementation Plan

## Workstream 1: Generalize `HostAttachContract` from resolved truth

### Goal

Make the durable attach contract truthful at session birth and successor creation.

### Primary files

1. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
2. [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
3. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
4. [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)

### Required changes

1. add a crate-private conversion from host-scoped `ResolvedLaunchContract` to `HostAttachContract`;
2. stop deriving the durable contract from manifest defaults in the steady-state birth path;
3. persist the attach-relevant capability subset from the resolved contract instead of defaulting all of them to `true`;
4. persist attach launch knobs from the resolved contract instead of recomputing them later;
5. keep `sync_host_attach_contract(...)` continuity-only:
   - allowed to refresh `continuity_uaa_session_id`;
   - not allowed to mutate backend, scope, protocol, runtime descriptor, capabilities, or attach knobs;
6. keep `fork_successor_attach_contract(...)` as clone-plus-clear-continuity only.

### Non-goals

1. do not persist full effective policy;
2. do not add a second durable attach structure;
3. do not let post-birth runtime drift mutate baseline truth silently.

## Workstream 2: Merge inventory `policy_overlay` into resolved effective policy

### Goal

Make `effective_policy` in inventory-backed resolution real, not placeholder.

### Primary files

1. [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
2. [crates/shell/src/execution/agent_inventory.rs](crates/shell/src/execution/agent_inventory.rs)
3. [crates/shell/src/execution/policy_model.rs](crates/shell/src/execution/policy_model.rs)

### Required changes

1. expose or extract a crate-private helper in `policy_model.rs` that applies a `PolicyPatch` over an existing `Policy` using the same semantics already encoded there;
2. keep `validate_policy_overlay(...)` as the pre-merge narrowing-only gate;
3. in inventory-backed resolution:
   - start from the effective base policy;
   - if `policy_overlay` exists, apply it into a working copy;
   - validate the merged result before returning it;
4. if merge application or merged validation fails, return a `DispatchResolutionError` at the policy layer with an exact field and reason;
5. record provenance so later diagnostics can say whether the final policy stayed baseline-only or was narrowed by overlay.

### Non-goals

1. do not change global/workspace policy precedence;
2. do not broaden what `policy_overlay` keys are permitted to express;
3. do not create a generic public policy-patch API just to support this resolver path.

## Workstream 3: Close the capability override contract honestly

### Goal

Support the bounded override family this stack actually needs and deny the rest explicitly.

### Primary files

1. [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
2. [crates/shell/src/execution/agent_runtime/validator.rs](crates/shell/src/execution/agent_runtime/validator.rs)
3. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
4. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)

### Required changes

1. replace blanket capability-override rejection in `validate_dispatch_overrides(...)` with field-by-field handling;
2. accepted 29.5 behavior:
   - only inventory-backed resolution may accept capability overrides;
   - accepted values may only narrow from `true` to `false`;
   - if the baseline capability is already `false`, keep the current fail-closed behavior;
3. rejected 29.5 behavior:
   - `session_start`, `llm`, and `mcp_client` remain unsupported override fields;
   - persisted attach launches still reject dispatch-time capability overrides entirely;
4. write accepted narrowed capabilities into `ResolvedLaunchContract.capabilities`;
5. when a host-scoped session persists `HostAttachContract`, carry the attach-relevant narrowed values forward so later `resume`, `fork`, and `stop` gates remain truthful;
6. ensure state-store gates continue to read those persisted capability values and therefore reflect the narrowed contract in operator behavior.

### Exact support matrix

| Capability field | 29.5 support | Reason |
| --- | --- | --- |
| `session_resume` | supported as narrowing-only | consumed by attach and state-store control gates |
| `session_fork` | supported as narrowing-only | consumed by successor attach semantics |
| `session_stop` | supported as narrowing-only | consumed by control gates |
| `status_snapshot` | supported as narrowing-only | part of control-surface truth and future public semantics |
| `event_stream` | supported as narrowing-only | part of retained ownership and operator surface behavior |
| `session_start` | rejected in 29.5 | not a meaningful per-dispatch narrowing knob for the active start path |
| `llm` | rejected in 29.5 | would broaden into runtime product semantics |
| `mcp_client` | rejected in 29.5 | would broaden into runtime product semantics |

## Workstream 4: Finish retained member-turn parity

### Goal

Remove the hidden second contract without rewriting working transport.

### Primary files

1. [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/routing/dispatch/world_ops.rs](crates/shell/src/execution/routing/dispatch/world_ops.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
4. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
5. [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

### Required changes

1. define a crate-private retained-turn parity builder over the existing persisted participant/session fields written at member startup;
2. ensure member startup persists every field that builder needs through the existing participant/session records;
3. feed retained member-turn transport from the reconstructed subset plus live orchestration linkage fields;
4. keep `MemberDispatchTransportRequest` as the wire shape in 29.5 and satisfy its inputs from the reconstructed shared-contract-derived subset;
5. ensure retained member turns do not consult inventory, effective config defaults, or ad hoc launch reconstruction on follow-up turns.

### Why this shape is frozen

Direct full-envelope re-resolution on every retained turn sounds pure but is the wrong abstraction:

1. the retained member already exists;
2. inventory or workspace defaults may have drifted since startup;
3. follow-up turn transport should reuse the already-authorized launch basis, not pretend it is a new selection event.

The parity contract therefore is:

1. cold start uses full shared resolution;
2. retained turn uses a derived subset from that same resolved truth;
3. both surfaces speak one vocabulary.

## Workstream 5: Sync docs and downstream assumptions

### Goal

Stop future slices from building on outdated claims.

### Primary files

1. [llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
2. [llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)
3. [llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)
4. any active operator-facing docs that currently describe attach or dispatch semantics

### Required changes

1. 29 must no longer imply unbounded capability override support;
2. 29 must explicitly say which capability family 29.5 makes real and which fields remain denied;
3. 30 must explicitly depend on 29.5 for truthful public capability and scope mapping;
4. 31 must explicitly depend on 29.5 for durable attach truth and successor-copy semantics;
5. if active code comments or inline ASCII diagrams near touched runtime seams become stale, update them in the same diff.

## Architecture Review

### Locked architecture decisions

1. Keep one resolver module.
   - `dispatch_contract.rs` remains the top-level contract owner.

2. Keep policy narrowing explicit.
   - Inventory overlay merge happens inside the resolver, not in callers.

3. Keep runtime materialization below the contract.
   - `materialize_runtime_descriptor(...)` stays a lower layer.

4. Keep durable attach truth small but real.
   - `HostAttachContract` persists only what later attach/control flows actually need.

5. Keep retained member transport typed.
   - Fix hidden semantics, not the wire format.

6. Keep later slices boring.
   - 30 and 31 should consume this contract, not reinterpret it.

### Architecture issues being solved

1. Persisted attach truth is currently under-specified.
   - Fix: derive `HostAttachContract` from resolved truth and make the state store gates consume that truth.

2. Effective policy is currently a lie in inventory-backed contract output when overlay exists.
   - Fix: apply the overlay at resolution time.

3. Retained member turns still have a hidden contract dialect.
   - Fix: use a resolved subset derived from the shared contract.

## Code Quality Review

### Implementation guardrails

1. one source of launch truth: `dispatch_contract.rs`;
2. one durable attach object: `HostAttachContract`;
3. one state-store authorization gate for persisted attach capabilities;
4. one policy-overlay merge semantics source, reused from `policy_model.rs`;
5. no duplicated ad hoc capability narrowing logic in CLI or REPL callers;
6. no "just set everything true" fallback paths.

### Minimal-diff rules

1. prefer exposing a crate-private merge helper over reimplementing patch semantics in the resolver;
2. prefer one parity subset struct over threading ad hoc booleans through multiple transport builders;
3. keep public type changes additive or avoid them entirely;
4. do not widen this slice into public flag parsing or lazy-attach product behavior.

### Code quality issues being solved

1. `HostAttachContract::from_manifest(...)` encodes defaults that can drift from resolved truth.
   - Fix: move birth-time durable contract derivation to the resolved contract path.

2. Blanket capability rejection hides which fields are intentionally unsupported.
   - Fix: make support and denial field-scoped and truthful.

3. Overlay validation without overlay merge is duplicated mental load for maintainers.
   - Fix: keep validation where it is and finish the actual merge in the resolver.

## Test Review

100 percent coverage for the changed contract edges is the goal. This slice changes control truth, so the plan must name every codepath and user-visible failure that needs proof.

### Code path coverage

```text
CODE PATHS
==========
[+] dispatch_contract.rs
  ├── resolve_inventory_contract_for_exact_backend(...)
  │   ├── [GAP] policy_overlay absent -> base effective_policy passes through unchanged
  │   ├── [GAP] policy_overlay present -> merged effective_policy is narrower
  │   ├── [GAP] accepted capability narrowing persists into resolved contract
  │   ├── [GAP] unsupported capability field fails closed with field-scoped reason
  │   └── [GAP] scope change still fails closed
  ├── resolve_inventory_contract_for_unique_scope(...)
  │   └── [GAP] world-member parity path sees the same narrowed capability and merged policy truth
  └── resolve_persisted_host_attach_contract(...)
      ├── [GAP] persisted capabilities are reused, not hardcoded permissive defaults
      ├── [GAP] persisted attach knobs are reused as baseline truth
      ├── [GAP] continuity-required without selector still fails closed
      └── [GAP] dispatch-time capability override still rejected for persisted attach

[+] orchestration_session.rs
  ├── HostAttachContract::from_manifest(...) or its replacement birth-time conversion
  │   └── [GAP] host-scoped resolved contract round-trips into durable attach truth
  ├── sync_host_attach_contract(...)
  │   └── [GAP] only continuity selector refreshes; baseline truth does not drift
  └── fork_successor_attach_contract(...)
      └── [GAP] successor copy preserves generalized truth and clears continuity only

[+] state_store.rs
  ├── resolve_public_control_target(...)
  │   ├── [GAP] narrowed resume capability denies resume
  │   ├── [GAP] narrowed fork capability denies fork
  │   └── [GAP] narrowed stop capability denies stop
  └── detached host continuity helpers
      └── [GAP] continuity gating still keys off durable contract truth only

[+] async_repl.rs / world_ops.rs
  ├── member cold start
  │   └── [GAP] retained member parity subset derived from resolved contract
  └── retained member turn transport
      └── [GAP] follow-up turn uses resolved subset, not manifest-only hidden semantics
```

### User flow coverage

```text
USER FLOWS
==========
[+] human host start with no overlay and no overrides
  └── [GAP] baseline behavior remains unchanged

[+] human host start with inventory policy_overlay
  └── [GAP] resolved contract and diagnostics reflect narrowed effective_policy

[+] human host start with supported capability narrowing
  ├── [GAP] launch succeeds
  ├── [GAP] durable attach contract persists narrowed capabilities
  └── [GAP] later resume/fork/stop honor those narrowed capabilities

[+] human host start with unsupported capability override field
  └── [GAP] fail closed with exact field + caller-contract reason

[+] human reattach / turn / fork from durable host contract
  ├── [GAP] resolved attach path reuses persisted capabilities and knobs
  ├── [GAP] missing continuity still fails closed for continuity-required attach
  └── [GAP] no ambient participant-state reconstruction occurs

[+] orchestrator world-member cold start
  └── [GAP] same shared resolver semantics as human inventory-backed start

[+] retained orchestrator member follow-up turn
  ├── [GAP] typed transport remains valid
  └── [GAP] no hidden second launch-resolution dialect remains
```

### Required test additions by file

#### `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`

Add direct unit coverage for:

1. overlay-free inventory resolution returns unchanged base effective policy;
2. overlay-backed inventory resolution returns narrower effective policy;
3. supported capability narrowing writes narrowed booleans into `ResolvedLaunchContract.capabilities`;
4. unsupported capability fields fail closed with exact field names;
5. persisted attach resolution reuses persisted capabilities and attach knobs;
6. persisted attach launches still reject dispatch-time capability overrides.

#### `crates/shell/src/execution/agent_runtime/orchestration_session.rs`

Add unit coverage for:

1. host attach contract birth-time derivation from resolved truth;
2. continuity-only sync semantics;
3. successor-copy semantics preserving generalized truth while clearing continuity only.

#### `crates/shell/tests/agent_public_control_surface_v1.rs`

Add CLI-surface coverage for:

1. persisted narrowed `session_resume=false` denies `reattach`;
2. persisted narrowed `session_fork=false` denies `fork`;
3. persisted narrowed `session_stop=false` denies `stop`;
4. continuity-required attach still fails closed when durable continuity selector is absent.

#### `crates/shell/tests/repl_world_first_routing_v1.rs`

Add parity coverage for:

1. equivalent human inventory-backed world launch and orchestrator member cold start produce equivalent resolved contract truth for backend, scope, capabilities, and policy;
2. retained member follow-up turn uses the shared-contract-derived subset and not a reconstructed hidden contract.

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Add contract-proof coverage for:

1. docs and runtime continue to use one truthful fail-closed wording for parity and capability-denial semantics;
2. successor durable attach truth preserves generalized capability and attach-knob state.

### Test plan artifact

Write a QA handoff artifact for this slice under `~/.gstack/projects/<slug>/` during implementation. It should cover:

1. host start with and without overlay;
2. supported capability narrowing flow;
3. persisted attach resume/fork/stop gates;
4. retained world-member follow-up turn parity;
5. continuity-required attach denial.

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| overlay merge in resolver | overlay validates but effective policy stays broader than docs claim | not yet | not fully | no | yes until merge tests land |
| bounded capability narrowing | resolved contract narrows capability but durable attach contract reverts it to permissive defaults | not yet | no | no | yes |
| persisted attach resolution | attach/fork/turn silently regain capabilities by hardcoded defaults | not yet | no | no | yes |
| retained member-turn parity | follow-up turns drift from cold-start contract semantics after inventory/config changes | not yet | partial | no | yes |
| continuity-only sync | runtime refresh accidentally mutates durable baseline truth | not yet | partial | no | yes |
| unsupported capability override | caller sees vague blanket rejection instead of field-scoped denial | not yet | partial | partial | no once field-scoped denial ships |

Critical-gap rule for this slice:

If any path can still silently broaden capability truth, silently ignore overlay narrowing, or let retained member turns bypass the shared vocabulary, 29.5 is not done.

## Performance Review

This is a correctness slice first, but there are still performance rules:

1. do not re-run inventory resolution on every retained follow-up turn;
2. do not add a new policy cache for a problem that only needs a one-shot merge during resolution;
3. keep parity data small and serializable;
4. keep all new checks O(1) or bounded by already-loaded inventory/session state.

Performance issues found:

- 0 material throughput issues if the slice follows the chosen subset-parity design.

The main performance footgun would be re-resolving the full inventory+policy baseline on every retained member turn. This plan explicitly forbids that.

## DX Review

This slice has no UI scope. It has strong DX scope because the product is a developer runtime surface.

### Developer journey map

| Stage | What the developer is doing | Current friction | Target after this slice |
| --- | --- | --- | --- |
| 1 | Read 29 and assume capability overrides are real | medium-high | docs say exactly which fields work |
| 2 | Launch a host runtime with overlay data | medium | resolved contract truth matches docs |
| 3 | Resume or fork later | high if durable contract drifted | durable attach truth is trustworthy |
| 4 | Debug retained world-member follow-up behavior | high | one parity model, no hidden second dialect |
| 5 | Plan slice 30 | high | public flags map to a truthful internal support matrix |
| 6 | Plan slice 31 | high | attach worker can trust durable knobs and capabilities |

### Developer empathy narrative

I am about to build 30 or 31. I need to know whether the shared dispatch contract is real or whether it still contains "frozen but not supported" scaffolding that I am expected to paper over later.

After 29.5, I should not need repo archaeology for that answer. I should be able to read one plan, inspect one resolver, and trust that persisted attach behavior and retained member-turn behavior are using the same contract floor.

### DX scorecard

| Dimension | Score | Notes |
| --- | --- | --- |
| Getting started on this stack | 6/10 | too many partial promises today |
| Naming and surface truth | 7/10 | core names are good, semantics still incomplete |
| Error messages | 6/10 | blanket override rejection is not precise enough |
| Docs findability | 6/10 | 29, 30, and 31 still need a tighter truth chain |
| Upgrade path safety | 8/10 | this slice makes later slices safer if completed fully |
| Debuggability | 6/10 | retained member parity is still too implicit today |

Target after this slice: 8/10 overall DX for the dispatch/attach contract floor.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Durable attach truth closeout | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/execution/agents_cmd.rs` | - |
| B. Policy overlay merge closeout | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/execution/policy_model.rs`, `crates/shell/src/execution/agent_inventory.rs` | - |
| C. Capability override support closeout | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agents_cmd.rs` | A, B |
| D. Retained member-turn parity closeout | `crates/shell/src/repl/`, `crates/shell/src/execution/routing/dispatch/`, `crates/shell/src/execution/agent_runtime/` | B |
| E. Docs and downstream truth sync | `llm-last-mile/`, active docs only if drifted | A, B, C, D |
| F. Final validation and fixups | `crates/shell/`, `llm-last-mile/`, active docs | C, D, E |

### Parallel lanes

- Lane A: Step A
- Lane B: Step B
- Lane C: Step C after A + B
- Lane D: Step D after B
- Lane E: Step E after A + B + C + D
- Lane F: Step F after C + D + E

### Execution order

1. Launch Lane A and Lane B in parallel first.
2. Merge A and B before starting capability override support in Lane C.
3. Lane D can start once Lane B lands because it depends on stable contract vocabulary and merged-policy semantics, not on persisted attach finish.
4. Launch Lane E only after A/B/C/D have settled the truth surface.
5. Run Lane F last for the full validation pass and any mechanical doc/test fixups.

### Conflict flags

1. `dispatch_contract.rs` is shared by Lanes A, B, C, and D.
   - Treat it as the primary merge-conflict hotspot. Sequence landings carefully or use one owner branch as the integration lane.

2. `orchestration_session.rs` and `state_store.rs` are Lane A and Lane C territory.
   - Keep Lane D out of those files unless absolutely necessary.

3. `async_repl.rs` is Lane D only.
   - Do not let capability work drift into retained-turn transport code. Lane D reuses persisted participant/session fields and fixes parity only.

4. `policy_model.rs` is Lane B only.
   - Lane C should consume the exported crate-private helper, not modify policy patch semantics independently.

### Parallelization verdict

This slice has real parallelization opportunity, but only after the contract seams are partitioned deliberately.

- 6 lanes total
- 2 lanes can launch immediately in parallel
- 2 more lanes can proceed in partial parallel after those land
- final validation remains sequential and non-negotiable

## Implementation Tasks

Synthesized from this plan's findings. Each task derives from a concrete contract gap above.

- [ ] **T1 (P1, human: ~2h / CC: ~20min)** - durable attach contract - derive `HostAttachContract` from resolved host launch truth
  - Surfaced by: Workstream 1 - current durable attach birth path still derives from manifest-era defaults
  - Files: `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/agents_cmd.rs`
  - Verify: targeted orchestration-session tests plus attach control-surface tests

- [ ] **T2 (P1, human: ~2h / CC: ~20min)** - policy merge - apply validated inventory `policy_overlay` into `ResolvedLaunchContract.effective_policy`
  - Surfaced by: Workstream 2 - overlay validation exists but merge truth does not
  - Files: `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/policy_model.rs`, `crates/shell/src/execution/agent_inventory.rs`
  - Verify: resolver unit tests proving narrower effective policy

- [ ] **T3 (P1, human: ~3h / CC: ~30min)** - bounded capability overrides - support narrowing-only overrides for the approved capability family
  - Surfaced by: Workstream 3 - blanket override rejection is no longer acceptable
  - Files: `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agents_cmd.rs`
  - Verify: resolver tests plus `agent_public_control_surface_v1.rs` denial coverage

- [ ] **T4 (P1, human: ~3h / CC: ~30min)** - retained member parity - feed retained member-turn transport from a shared-contract-derived subset
  - Surfaced by: Workstream 4 - retained follow-up still has a hidden launch dialect
  - Files: `crates/shell/src/repl/async_repl.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - Verify: `repl_world_first_routing_v1.rs` parity regressions

- [ ] **T5 (P2, human: ~90min / CC: ~15min)** - successor truth - preserve generalized durable attach truth on fork successor copy while clearing only continuity
  - Surfaced by: Workstream 1 - successor behavior must be explicit for slice 31
  - Files: `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agents_cmd.rs`
  - Verify: successor contract tests

- [ ] **T6 (P2, human: ~60min / CC: ~10min)** - truth docs - update 29, 30, and 31 to cite the same support matrix and dependency floor
  - Surfaced by: Workstream 5 - downstream slices should not guess what 29.5 made real
  - Files: `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`, `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`, `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`
  - Verify: manual doc cross-check against code and tests

- [ ] **T7 (P2, human: ~90min / CC: ~15min)** - coverage closeout - add direct regression coverage for overlay merge, capability narrowing, persisted attach authority, and retained-turn parity
  - Surfaced by: Test Review - multiple critical gaps remain unproven
  - Files: `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/tests/agent_public_control_surface_v1.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - Verify: targeted shell test commands and full package pass

## Validation Commands

Run at minimum:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_public_control_surface_v1 -- --nocapture
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Then run:

```bash
cargo test -p shell -- --nocapture
cargo clippy -p shell --all-targets -- -D warnings
```

Manual validation must prove:

1. a host-scoped resolved launch contract persists attach-relevant truth into `HostAttachContract`;
2. `reattach`, `turn`, and `fork` consume that durable truth instead of reconstructing permissive defaults;
3. overlay-backed inventory entries produce materially narrower `effective_policy`;
4. supported capability narrowing changes later attach/control behavior in the state store;
5. retained member follow-up turns use the resolved subset path and do not re-run hidden baseline selection.

## Definition of Done

This slice is done only when all of the following are true:

1. `HostAttachContract` is birth-time-derived from resolved host launch truth;
2. persisted attach resolution no longer hardcodes permissive capability or policy defaults;
3. validated `policy_overlay` data is merged into resolver output as actual narrowing truth;
4. the approved capability override family works as narrowing-only and persists where later attach/control flows depend on it;
5. unsupported override fields fail closed with exact, bounded reasons;
6. retained member turns consume a shared-contract-derived subset and no longer represent a second hidden dialect;
7. slices 30 and 31 can cite 29.5 as the truthful contract floor without reopening semantics;
8. targeted shell tests and the full shell package pass are green;
9. docs, code comments, and any nearby ASCII diagrams touched by the change match the shipped semantics.

## Completion Summary

- Objective: freeze the shared dispatch contract floor so 30 and 31 can build on truth instead of scaffolding
- UI scope: none
- DX scope: strong
- Locked decisions: bounded capability narrowing implemented now; retained member-turn parity uses a shared-contract-derived subset
- Architecture focus: one resolver, one durable attach object, one retained-turn vocabulary
- Main technical gaps closed: durable attach truth, overlay merge truth, capability override truth, retained-turn parity truth
- Parallelization: 6 lanes, with A+B parallel first, C+D partially parallel after that, and final validation last
- Outcome target: one truthful shared dispatch contract and one truthful durable host-attach contract, with no hidden second dialect left behind
