# PLAN: Shared Agent Dispatch Envelope And Capability Override Contract

Source SOW: [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)  
Primary truth anchors: [ADR-0027](docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md), [ADR-0025](docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md), [ADR-0026](docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md)  
Primary code anchors: [crates/shell/src/execution/agent_inventory.rs](crates/shell/src/execution/agent_inventory.rs), [crates/shell/src/execution/agent_runtime/validator.rs](crates/shell/src/execution/agent_runtime/validator.rs), [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs), [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs), [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs), [crates/shell/src/execution/prompt_fulfillment.rs](crates/shell/src/execution/prompt_fulfillment.rs)  
Adjacent slices: [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](llm-last-mile/28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md), [30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md), [31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Execution branch: `feat/gateway-mediated-llm-fulfillment`  
Base branch: `main`  
Plan type: shared contract-definition and runtime convergence slice  
Review posture: unified execution plan tightened to `/autoplan` and `/plan-eng-review` rigor  
Status: implementation-ready planning pass on 2026-05-24

## Objective

Land one internal dispatch contract that every launch surface can trust.

This slice is complete only when the repo has one deterministic, explanation-ready resolution path that:

1. starts from effective inventory truth for new launches,
2. starts from persisted attach truth for attach/fork flows,
3. accepts only explicitly supported dispatch-time overrides,
4. applies policy as narrowing-only,
5. produces one resolved launch contract for runtime execution,
6. derives one generalized persisted host attach contract from that resolved contract,
7. is consumed by human launch surfaces and orchestrator-controlled dispatch surfaces without caller-specific semantics drift.

This is a contract slice. It is not a public CLI expansion slice, not a world-root architecture debate, and not a generic "add capability flags everywhere" cleanup.

## Acceptance Criteria

This plan is only done when all of the following are true in code, tests, and docs:

1. one internal `DispatchRequestEnvelope`-style shape exists and becomes the only entrypoint for launch resolution;
2. one internal `ResolvedLaunchContract`-style shape exists and becomes the only source of truth for:
   - host prompt-bearing launch,
   - detached host attach planning,
   - successor attach-contract persistence,
   - orchestrator-controlled world-member dispatch planning;
3. inventory remains the baseline truth for new launches:
   - workspace inventory still overrides global inventory,
   - effective config still backfills omitted inventory fields,
   - embedded inventory `policy_overlay` still validates as restriction-only;
4. persisted `HostAttachContract` truth remains the baseline for reattach/fork and detached-turn attach planning;
5. supported override families are explicit and fail closed when unknown or disallowed;
6. policy narrowing is explicit and auditable:
   - no silent coercion,
   - no "close enough" fallback,
   - no broadening through dispatch overrides;
7. `HostAttachContract` is derived from the resolved host launch contract, not reconstructed from ambient participant state;
8. equivalent human and orchestrator-controlled requests produce equivalent resolved contracts when they start from equivalent baseline truth;
9. docs explain:
   - baseline sources,
   - merge precedence,
   - supported override families,
   - denial taxonomy,
   - persisted attach truth,
   - downstream dependencies for slices 30 and 31;
10. targeted tests prove:
    - baseline inventory resolution,
    - persisted attach resolution,
    - override acceptance and denial,
    - attach-contract generalization,
    - caller parity,
    - truthful diagnostics and fail-closed behavior.

## Locked Decisions

These decisions are already made. Implementation does not reopen them.

| Topic | Locked decision | Why |
| --- | --- | --- |
| Baseline truth for new launch | Effective inventory plus effective config remains the source of default backend identity, scope, CLI mode, capabilities, and embedded restriction-only policy overlays | The repo already models backend truth there; a second baseline would rot immediately |
| Baseline truth for attach/fork | Persisted `HostAttachContract` remains the source of exact host attach truth for reattach, detached-turn attach planning, and successor-copy behavior | Reconstructing attach truth from live participant state is exactly the drift 28.5 was meant to stop |
| Runtime descriptor role | `RuntimeSelectionDescriptor` and `ResolvedRuntimeDescriptor` stay low-level runtime materialization artifacts | They are not rich enough to be the top-level policy and override contract |
| Policy posture | Policy remains narrowing-only and fail-closed | ADR-0027 and existing inventory overlay validation already commit to that contract |
| Caller parity | Human CLI and orchestrator-controlled dispatch must resolve through the same contract module | Two launch dialects guarantees downstream drift in 30 and 31 |
| Persisted attach truth | `HostAttachContract` remains the durable attach object, but its payload is generalized from the resolved contract | Minimal diff, stable naming, no need for a second durable concept |
| Public prompt verbs | `PublicPromptCommandRequest` stays a prompt-bearing transport shape, not the shared launch envelope | Mixing operator input transport with internal contract semantics is how hidden bootstrap behavior comes back |
| World-root direction | `--scope world` remains future host-rooted orchestration plus world worker, not standalone world-root continuity | 30 and 31 already depend on that decision |
| Wire contracts | Existing typed world-member follow-up transport stays intact unless this slice proves a missing field cannot be represented without additive change | Minimal diff and proven Linux-first behavior matter more than elegance |

## Scope

### In scope

1. define one shared internal dispatch request envelope;
2. define one resolved launch contract with exact provenance and denial semantics;
3. define the supported override taxonomy for this stage;
4. implement one merge order across inventory defaults, persisted attach truth, dispatch overrides, and policy restrictions;
5. generalize the persisted host attach contract from that resolved host contract;
6. route both human launch surfaces and orchestrator-controlled dispatch surfaces through the same resolver or the same persisted resolved-truth subset;
7. publish exact denial and narrowing behavior in docs and tests.

### Out of scope

1. public `substrate agent start --scope world` shipping;
2. lazy host attach behavior;
3. automatic attach-worker launch from pending work;
4. public capability-flag UX design beyond reserving the internal contract;
5. inventing a new crate or a second runtime subsystem;
6. replacing the existing world-member dispatch wire contract unless additive fields are proven necessary;
7. moving the orchestrator in-world;
8. broadening policy semantics to allow dispatch-time privilege escalation.

## Scope Challenge

### What already exists and must be reused

| Area | Existing code or contract | Reuse decision |
| --- | --- | --- |
| Effective inventory resolution | [`load_effective_agent_inventory(...)`](crates/shell/src/execution/agent_inventory.rs), [`AgentFileV1::effective_scope(...)`](crates/shell/src/execution/agent_inventory.rs), [`AgentFileV1::effective_cli_mode(...)`](crates/shell/src/execution/agent_inventory.rs) | Reuse exactly. This is the baseline truth layer for new launches. |
| Gateway/backend identity from inventory | [`resolve_gateway_backend_inventory_entry(...)`](crates/shell/src/execution/agent_inventory.rs) | Reuse the inventory semantics, but move launch ownership above it into the shared resolver. |
| Runtime realizability checks | [`validate_runtime_realizability(...)`](crates/shell/src/execution/agent_runtime/validator.rs), [`validate_exact_backend_selection(...)`](crates/shell/src/execution/agent_runtime/validator.rs), [`validate_member_selection(...)`](crates/shell/src/execution/agent_runtime/validator.rs) | Reuse, but demote them to materialization and selection helpers under the shared contract. |
| Human prompt-bearing start/turn | [`run_start(...)`](crates/shell/src/execution/agents_cmd.rs), [`run_turn(...)`](crates/shell/src/execution/agents_cmd.rs), [`run_public_prompt_command(...)`](crates/shell/src/execution/agent_runtime/control.rs) | Reuse transport and prompt streaming. Replace ad hoc launch planning only. |
| Durable attach seam | [`HostAttachContract`](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`sync_host_attach_contract(...)`](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`fork_successor_attach_contract(...)`](crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse the durable seam and generalize its payload. Do not create a second persisted object. |
| Detached control targeting | [`resolve_public_control_target(...)`](crates/shell/src/execution/agent_runtime/state_store.rs), [`resolve_public_turn_target(...)`](crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse exact-session and posture checks. Do not let the new contract weaken them. |
| Attach execution path | [`build_attach_launch_plan(...)`](crates/shell/src/execution/agents_cmd.rs), [`PromptFulfillmentBridge::run_attach_control(...)`](crates/shell/src/execution/prompt_fulfillment.rs) | Reuse, but source the launch descriptor from persisted resolved truth instead of ambient state reconstruction. |
| Orchestrator-controlled member dispatch | [`build_member_dispatch_transport_request(...)`](crates/shell/src/repl/async_repl.rs), [`MemberDispatchTransportRequest`](crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse the typed transport. Feed it from the shared resolver instead of bespoke prepared-runtime assumptions. |
| Existing proof floor | [`agent_public_control_surface_v1.rs`](crates/shell/tests/agent_public_control_surface_v1.rs), [`repl_world_first_routing_v1.rs`](crates/shell/tests/repl_world_first_routing_v1.rs), state-store tests in [`state_store.rs`](crates/shell/src/execution/agent_runtime/state_store.rs), orchestration-session tests in [`orchestration_session.rs`](crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse and extend. Do not create a second test harness for a contract slice. |

### Exact gap being closed

The repo already has:

1. effective inventory truth,
2. runtime selection/materialization helpers,
3. durable host attach persistence,
4. exact-session control semantics,
5. typed world-member dispatch transport.

What it does not have is one caller-neutral contract above those seams.

Right now the shape is fragmented:

1. `PublicPromptCommandRequest` models prompt-bearing human commands, not shared launch semantics;
2. `HiddenOwnerHelperLaunchPlan` models owner-helper startup, not caller-neutral resolution;
3. `MemberDispatchTransportRequest` models typed world-member transport, not shared dispatch intent;
4. `RuntimeSelectionDescriptor` models runtime-realizable selection, not inventory plus override plus policy provenance;
5. `HostAttachContract` persists exact host launch truth, but not a first-class resolved contract vocabulary that later slices can extend safely.

Slice 30 needs public world-scoped start to use the same scope/capability/attach semantics.  
Slice 31 needs continuity attach and fresh attach to use the same persisted truth.  
If 29 does not freeze the contract now, 30 and 31 will each invent partial semantics and "align later." That is exactly the failure this slice exists to prevent.

### Minimum honest change

The minimum honest implementation is:

1. add one internal dispatch contract module;
2. define one explicit baseline model for inventory-backed launches and one explicit baseline mode for persisted-attach-backed launches;
3. define supported override families and denial rules explicitly;
4. produce one resolved contract with provenance and one denial taxonomy;
5. derive the generalized host attach contract from that resolved contract;
6. route current human and orchestrator-controlled call sites through it or through the persisted subset it already wrote;
7. prove parity and fail-closed behavior with targeted tests.

Anything smaller keeps the branch in its current state where multiple launch shapes all look "obviously right" but disagree on who owns defaults, narrowing, and attach truth.

### Complexity and fit

This slice is not small:

1. it will touch persistent state, CLI entrypoints, runtime realization, and REPL-controlled world dispatch;
2. it will add one new internal module and update multiple high-consequence seams;
3. it will require additive session-state changes plus caller-parity tests.

It is still the correct scope because the alternative is worse:

1. letting 30 invent a CLI-only capability model;
2. letting 31 invent a lazy-attach-only host contract;
3. keeping `agents_cmd.rs`, `async_repl.rs`, and `state_store.rs` on parallel semantics.

The minimal-diff rule here is:

1. one new module, not a new crate;
2. zero new public verbs;
3. zero public wire changes unless proven necessary;
4. no second persisted attach object;
5. keep existing transport code and exact-session state-store rules intact.

## Architecture Review

### Thesis

Create one shared resolution layer above runtime realization and below caller transport.

The new layer must answer this once:

> Given baseline truth, caller kind, requested target, requested scope/capability/attach overrides, effective policy, and workspace context, what launch contract is allowed right now, and why?

Everything after that is materialization:

1. host owner-helper launch,
2. host attach launch,
3. world-member typed dispatch request,
4. durable attach persistence,
5. diagnostics and tests.

### Resolution domains

This slice has two legitimate baseline domains. They must be explicit in the contract.

| Resolution domain | Used by | Baseline source | What may change in 29 |
| --- | --- | --- | --- |
| Inventory-backed resolution | human `start`, orchestrator-controlled member dispatch, future public world-scoped start | effective inventory + effective config + validated embedded `policy_overlay` | supported scope/capability/attach-knob overrides plus narrowing-only dispatch policy overlay |
| Persisted-attach-backed resolution | `reattach`, `fork`, detached-turn attach planning, future lazy attach | persisted `HostAttachContract` under the orchestration session | continuity selector state and attach-mode selection only; no backend/scope/capability broadening |

That split removes the current ambiguity where some flows act like inventory is always authoritative and others act like the last live participant is authoritative.

### Target architecture

```text
TARGET
======

caller surface
    |
    +--> human start / turn / reattach / fork
    +--> orchestrator-controlled world-member dispatch
    `--> future world-scoped start / lazy attach
            |
            v
    DispatchRequestEnvelope
            |
            v
    DispatchContractResolver
      1. load baseline truth
         - inventory-backed for new launches
         - persisted-attach-backed for attach/fork
      2. validate caller kind and target identity
      3. validate supported override families
      4. merge baseline + overrides
      5. apply effective policy as narrowing-only
      6. emit provenance or exact denial reasons
            |
            v
    ResolvedLaunchContract
            |
            +--> runtime materializer
            |      `--> RuntimeSelectionDescriptor / ResolvedRuntimeDescriptor
            |
            +--> host attach persistence
            |      `--> HostAttachContract
            |
            +--> world-member transport adapter
            |      `--> MemberDispatchTransportRequest
            |
            `--> diagnostics / tests / docs
```

### Canonical contract vocabulary

Exact Rust identifiers may differ slightly, but the concept split below is frozen.

#### 1. `DispatchRequestEnvelope`

Required fields:

1. `caller_kind`
   - `human_start`
   - `human_turn`
   - `human_reattach`
   - `human_fork`
   - `orchestrator_member_start`
   - `orchestrator_member_turn`
   - reserved now for future `human_start_world` and `lazy_attach`
2. `baseline_kind`
   - `inventory_launch`
   - `persisted_host_attach`
3. target identity
   - exact `backend_id` when caller already knows it,
   - exact `agent_id` only when the caller surface truly owns that abstraction,
   - exact `orchestration_session_id` when the caller is resolving from persisted attach truth;
4. requested execution-scope override;
5. requested capability override set;
6. requested attach-relevant launch knobs;
7. requested narrowing-only policy overlay, if supported for that caller;
8. workspace root and policy-resolution context;
9. prompt-bearing payload marker when a prompt exists.

Rules:

1. inventory-backed requests may select a backend or a unique eligible world member, but they must still resolve through effective inventory;
2. persisted-attach-backed requests may not replace backend identity, scope, or capabilities with fresh inventory-driven values;
3. `human_turn`, `human_reattach`, and `human_fork` must still pass through exact-session state-store checks before the resolver is allowed to materialize launch truth.

#### 2. `DispatchCapabilityOverrideSet`

Use the current inventory capability families. Keep it explicit and boring.

This should be an `Option<bool>`-style override shape for the existing booleans:

1. `session_start`
2. `session_resume`
3. `session_fork`
4. `session_stop`
5. `status_snapshot`
6. `event_stream`
7. `llm`
8. `mcp_client`

Rules:

1. `true` may only be requested if the baseline truth already allows it;
2. `false` narrows or disables capability;
3. unknown capability names fail closed;
4. capability selection is part of the resolved contract and part of the explanation surface;
5. persisted-attach-backed requests may not broaden a capability that the persisted contract already narrowed or omitted.

#### 3. `AttachLaunchKnobs`

Slice 29 must freeze the internal knob vocabulary that 30 and 31 depend on, even though 29 does not expose full public CLI syntax for it yet.

Minimum required internal knobs:

1. `requested_execution_scope`
   - `host`
   - `world`
2. `host_execution_client_start`
   - `start_now`
   - `defer`
3. `attach_mode_preference`
   - `continuity_required`
   - `continuity_preferred`
   - `fresh_allowed`

Rules:

1. current public host start uses `host + start_now + continuity_required`;
2. future world-scoped start in 30 uses `world + defer`;
3. future lazy attach in 31 depends on `fresh_allowed`;
4. unsupported combinations fail closed now;
5. persisted `HostAttachContract` must retain the resolved attach knobs required to perform later continuity attach or fresh attach without guessing.

#### 4. `ResolvedLaunchContract`

Required contents:

1. resolved `agent_id`;
2. resolved `backend_id`;
3. resolved backend kind;
4. resolved protocol;
5. resolved execution scope;
6. resolved runtime-realizable descriptor payload;
7. resolved capability set;
8. resolved attach-launch knobs;
9. effective policy restrictions actually applied;
10. baseline source metadata;
11. field-level provenance.

Rules:

1. this is the last place where merge semantics live;
2. everything downstream consumes it or a deliberately derived subset of it;
3. no caller-specific adapter may rewrite resolved scope/capability/attach semantics after the contract is emitted.

#### 5. `FieldProvenance`

The current plan needs more precision than a single flat enum. Provenance must distinguish:

1. baseline origin
   - `global_inventory`
   - `workspace_inventory`
   - `persisted_host_attach_contract`
2. value origin
   - `inventory_explicit`
   - `effective_config_default`
   - `dispatch_override_accepted`
   - `dispatch_override_narrowed_by_policy`

Rules:

1. if a resolved field came from config backfill rather than an explicit inventory value, the contract must be able to say so;
2. if a resolved field came from persisted attach truth, the contract must say that directly rather than pretending inventory was re-read;
3. denial is not a provenance state; it is a resolution error category.

#### 6. `DispatchResolutionError`

Implementation may choose exact type names, but the denial taxonomy is frozen.

Minimum categories:

1. `unknown_override_family`
2. `override_not_supported_for_caller`
3. `override_exceeds_baseline`
4. `invalid_policy_overlay`
5. `override_denied_by_policy`
6. `runtime_unrealizable_after_resolution`
7. `missing_required_attach_continuity`

Rules:

1. every denial must name the field or override family that failed;
2. every denial must name the rejecting layer:
   - caller contract,
   - baseline truth,
   - policy,
   - runtime materialization;
3. no denial may degrade into a silent fallback.

### Merge precedence and failure behavior

The merge order is frozen:

1. baseline truth
   - inventory-backed requests:
     - workspace inventory row wins over global row by existing repo rules,
     - effective config backfills omitted inventory fields,
     - embedded inventory `policy_overlay` validates as restriction-only;
   - persisted-attach-backed requests:
     - `HostAttachContract` is authoritative for backend, protocol, scope, launch descriptor, attach-relevant capabilities, and attach knobs;
2. dispatch envelope overrides
   - only supported families are accepted,
   - unsupported or unknown families fail closed,
   - persisted-attach-backed requests may not broaden contract fields;
3. effective policy
   - base policy plus any validated restriction-only overlay,
   - policy narrows, denies, or leaves as-is,
   - policy never broadens;
4. runtime materialization
   - only after the contract is resolved.

Failure behavior is also frozen:

1. unknown override family -> hard error;
2. supported but disallowed override -> hard error;
3. attempt to broaden beyond baseline truth -> hard error;
4. policy-denied override -> hard error with field + rejecting layer + reason;
5. policy-narrowed override -> resolved contract carries the narrowed value plus provenance;
6. impossible runtime realization after merge -> hard error before launch;
7. continuity-required attach with no valid continuity selector -> hard error before attach launch.

No silent fallback.  
No "host is close enough to world."  
No "event_stream=true was ignored because backend probably does not need it."

## Code Quality Review

### Module ownership rules

1. [`agent_inventory.rs`](crates/shell/src/execution/agent_inventory.rs) owns raw inventory parsing, validation, and baseline projection helpers. It does not own caller semantics.
2. New [`dispatch_contract.rs`](crates/shell/src/execution/agent_runtime/dispatch_contract.rs) owns:
   - baseline normalization,
   - override validation,
   - policy narrowing merge,
   - provenance,
   - denial taxonomy,
   - resolved contract output.
3. [`validator.rs`](crates/shell/src/execution/agent_runtime/validator.rs) owns runtime realizability checks and conversion to runtime descriptors.
4. [`orchestration_session.rs`](crates/shell/src/execution/agent_runtime/orchestration_session.rs) owns persisted attach truth shape and invariants.
5. [`state_store.rs`](crates/shell/src/execution/agent_runtime/state_store.rs) owns authoritative session/posture checks, not launch-merge logic.
6. [`agents_cmd.rs`](crates/shell/src/execution/agents_cmd.rs) and [`async_repl.rs`](crates/shell/src/repl/async_repl.rs) become thin adapters.

### Compatibility rules

1. Do not rename `HostAttachContract` unless there is a hard serialization blocker.
2. Persisted JSON changes must be additive or migration-safe.
3. [`OrchestrationSessionRecord::validate_persisted_invariants(...)`](crates/shell/src/execution/agent_runtime/orchestration_session.rs) must remain the invariant gate for session reload.
4. Do not persist the full provenance tree inside `HostAttachContract`.

Persist only:

1. exact resolved launch truth needed to attach later;
2. resolved capability selections relevant to attach;
3. resolved attach-launch knobs;
4. continuity selector state when present.

Why:

1. session JSON is long-lived durable state;
2. full provenance there would bloat files and future migrations;
3. explanation is a command-time concern, not a session-serialization concern.

### File-level implementation map

| Area | Files | Planned change |
| --- | --- | --- |
| New contract module | `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, [`mod.rs`](crates/shell/src/execution/agent_runtime/mod.rs) | Add the envelope, override set, attach knobs, resolved contract, provenance, denial taxonomy, and merge logic in one place |
| Baseline projection | [`crates/shell/src/execution/agent_inventory.rs`](crates/shell/src/execution/agent_inventory.rs) | Add helpers that project effective inventory entries into baseline contract data without changing inventory semantics |
| Runtime materialization | [`crates/shell/src/execution/agent_runtime/validator.rs`](crates/shell/src/execution/agent_runtime/validator.rs) | Keep realizability checks, but materialize them from `ResolvedLaunchContract` instead of letting selection helpers act as top-level contract |
| Human caller adoption | [`crates/shell/src/execution/agents_cmd.rs`](crates/shell/src/execution/agents_cmd.rs), [`crates/shell/src/execution/agent_runtime/control.rs`](crates/shell/src/execution/agent_runtime/control.rs), [`crates/shell/src/execution/prompt_fulfillment.rs`](crates/shell/src/execution/prompt_fulfillment.rs) | Replace start/attach/resume/fork launch-plan assembly with shared resolver output or persisted attach truth derived from it |
| Durable attach generalization | [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`crates/shell/src/execution/agent_runtime/state_store.rs`](crates/shell/src/execution/agent_runtime/state_store.rs) | Expand `HostAttachContract` to carry the resolved host launch truth needed for continuity attach and fresh attach later |
| Orchestrator-controlled caller adoption | [`crates/shell/src/repl/async_repl.rs`](crates/shell/src/repl/async_repl.rs), [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](crates/shell/src/execution/routing/dispatch/world_ops.rs) | Build world-member dispatch from the resolved contract, not from bespoke prepared-runtime assumptions |
| Tests | [`crates/shell/tests/agent_public_control_surface_v1.rs`](crates/shell/tests/agent_public_control_surface_v1.rs), [`crates/shell/tests/repl_world_first_routing_v1.rs`](crates/shell/tests/repl_world_first_routing_v1.rs), unit tests in `dispatch_contract.rs`, `orchestration_session.rs`, `state_store.rs`, `control.rs`, and `validator.rs` | Prove parity, denial taxonomy, attach-contract persistence, and future-compat attach knobs |
| Truth-doc sync | [ADR-0027](docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md), [llm-last-mile/29*.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [30*.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md), [31*.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md), [llm-last-mile/README.md](llm-last-mile/README.md) | Publish the contract once and point downstream slices at it |

### Implementation phases

#### A0. Freeze the contract vocabulary

Files:

1. `PLAN.md`
2. [`llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
3. truth docs that mention the downstream dependency

Required outcome:

1. one canonical vocabulary for baseline domains, override families, attach knobs, provenance, and denial taxonomy;
2. slices 30 and 31 reference the frozen knob vocabulary instead of inventing synonyms.

Exit criteria:

1. this plan and the SOW say the same thing about baseline truth, attach truth, and policy narrowing;
2. no downstream doc reopens world-root continuity or hidden bootstrap prompts.

#### A1. Land the shared resolver module

Files:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. [`crates/shell/src/execution/agent_runtime/mod.rs`](crates/shell/src/execution/agent_runtime/mod.rs)
3. [`crates/shell/src/execution/agent_inventory.rs`](crates/shell/src/execution/agent_inventory.rs)

Required outcome:

1. the repo has one place that owns baseline normalization, override validation, narrowing-only merge, provenance, and denial taxonomy;
2. inventory-backed and persisted-attach-backed baselines are both modeled explicitly;
3. no caller surface is still defining its own merge precedence.

Exit criteria:

1. new unit tests cover both baseline domains;
2. the resolver can emit a resolved contract or an exact denial without touching prompt transport code.

#### A2. Materialize runtime descriptors from the resolved contract

Files:

1. [`crates/shell/src/execution/agent_runtime/validator.rs`](crates/shell/src/execution/agent_runtime/validator.rs)
2. [`crates/shell/src/execution/agent_runtime/control.rs`](crates/shell/src/execution/agent_runtime/control.rs)

Required outcome:

1. `RuntimeSelectionDescriptor` and `ResolvedRuntimeDescriptor` are downstream materialization artifacts;
2. runtime-realizability failures occur after the contract is resolved, not during ad hoc caller-specific planning.

Exit criteria:

1. runtime materialization can be invoked from `ResolvedLaunchContract`;
2. error paths still preserve existing exit-code expectations.

#### A3. Generalize the persisted host attach contract

Files:

1. [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
2. [`crates/shell/src/execution/agent_runtime/state_store.rs`](crates/shell/src/execution/agent_runtime/state_store.rs)

Required outcome:

1. session birth persists the generalized host attach contract derived from the resolved host launch contract;
2. successor allocation copies that contract forward while clearing inherited continuity;
3. detached attach planning consumes persisted resolved truth, not ambient participant state.

Exit criteria:

1. session reload validation remains additive and fail closed;
2. `fork_successor_attach_contract(...)` preserves exact launch truth and clears only continuity-specific state.

#### A4. Adopt the contract in human caller surfaces

Files:

1. [`crates/shell/src/execution/agents_cmd.rs`](crates/shell/src/execution/agents_cmd.rs)
2. [`crates/shell/src/execution/agent_runtime/control.rs`](crates/shell/src/execution/agent_runtime/control.rs)
3. [`crates/shell/src/execution/prompt_fulfillment.rs`](crates/shell/src/execution/prompt_fulfillment.rs)

Required outcome:

1. host `start` uses the inventory-backed resolver;
2. `reattach`, detached-turn attach planning, and `fork` use persisted attach truth derived from the resolver;
3. denial messages name field + layer + reason.

Exit criteria:

1. human caller code no longer rebuilds launch semantics outside the shared contract or the persisted contract it already wrote;
2. prompt-bearing validation remains exact.

#### A5. Adopt the contract in orchestrator-controlled dispatch

Files:

1. [`crates/shell/src/repl/async_repl.rs`](crates/shell/src/repl/async_repl.rs)
2. [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](crates/shell/src/execution/routing/dispatch/world_ops.rs) only if additive transport fields are required

Required outcome:

1. world-member launch planning comes from the inventory-backed shared resolver;
2. orchestrator-controlled dispatch uses the same capability and policy semantics as human launch;
3. no second hidden launch dialect survives in REPL-only code.

Exit criteria:

1. equivalent inventory-backed requests resolve to equivalent backend/scope/capability truth across CLI and REPL surfaces;
2. transport changes, if any, are additive and justified by a concrete missing field.

#### A6. Update docs and downstream slice references

Files:

1. [ADR-0027](docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md)
2. [llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
3. [llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)
4. [llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)
5. [llm-last-mile/README.md](llm-last-mile/README.md)

Required outcome:

1. the repo explains the contract once;
2. downstream slices depend on that explanation instead of restating it differently;
3. no doc still implies attach truth can be reconstructed from the last live participant.

#### A7. Validate and close out

Required outcome:

1. targeted unit and integration coverage is green;
2. full workspace gates are green;
3. doc wording and runtime semantics match.

## Test Review

### Coverage floor

The honest test floor for this slice is:

1. new unit tests in `dispatch_contract.rs`;
2. unit updates in `orchestration_session.rs`;
3. state-store resolution tests;
4. command-surface integration tests in [`agent_public_control_surface_v1.rs`](crates/shell/tests/agent_public_control_surface_v1.rs);
5. Linux-first world-member parity tests in [`repl_world_first_routing_v1.rs`](crates/shell/tests/repl_world_first_routing_v1.rs);
6. full `cargo test --workspace -- --nocapture` before closeout.

### Required test additions

1. Add resolver unit tests in `crates/shell/src/execution/agent_runtime/dispatch_contract.rs` covering:
   - inventory-backed baseline projection,
   - persisted-attach-backed baseline projection,
   - supported override acceptance,
   - unsupported override denial,
   - baseline-broadening denial,
   - policy narrowing provenance,
   - impossible runtime-realization rejection.
2. Extend [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](crates/shell/src/execution/agent_runtime/orchestration_session.rs) tests to prove:
   - generalized `HostAttachContract` invariants,
   - successor copy clears continuity but preserves resolved attach truth,
   - host-scoped protocol/backend drift still fails invariant validation.
3. Extend [`crates/shell/src/execution/agent_runtime/state_store.rs`](crates/shell/src/execution/agent_runtime/state_store.rs) tests to prove:
   - `resolve_public_control_target(...)` requires generalized attach truth,
   - continuity-required attach still fails closed when the selector is missing,
   - fork can consume generalized attach truth without live continuity.
4. Extend [`crates/shell/src/execution/agent_runtime/control.rs`](crates/shell/src/execution/agent_runtime/control.rs) tests to prove:
   - shared resolver output feeds host launch plans,
   - denial messages are explicit,
   - prompt-bearing request validation remains exact.
5. Extend [`crates/shell/tests/agent_public_control_surface_v1.rs`](crates/shell/tests/agent_public_control_surface_v1.rs) to prove:
   - human `start` now uses the shared inventory-backed resolver,
   - detached turn attach planning uses persisted attach truth,
   - persisted attach truth after `start` and `fork` matches the resolved host contract,
   - no silent capability broadening appears.
6. Extend [`crates/shell/tests/repl_world_first_routing_v1.rs`](crates/shell/tests/repl_world_first_routing_v1.rs) to prove:
   - orchestrator-controlled member dispatch consumes the same resolved backend/scope/capability rules,
   - future `scope=world` knobs are reserved but still fail closed on unsupported public paths.

### Failure modes registry

| Codepath | Failure mode | Required proof |
| --- | --- | --- |
| Inventory baseline projection | Shared resolver ignores workspace override and launches the wrong backend | Resolver unit tests must prove workspace precedence survives the refactor |
| Persisted attach baseline | Detached attach planning re-derives launch truth from inventory or live participant state | State-store and orchestration-session tests must prove persisted attach truth is authoritative |
| Capability override | Caller asks for disallowed capability and resolver silently leaves baseline enabled | Resolver unit tests must fail closed and name the capability |
| Policy narrowing | Policy removes `llm` or forbids backend and resolver still materializes runtime descriptor | Resolver + validator tests must stop before launch and name the rejecting policy field |
| Session birth persistence | Session birth writes descriptor but omits attach knobs or attach-relevant capability truth | Orchestration-session tests must compare persisted attach truth to the resolved host contract |
| Successor copy | Fork preserves parent continuity session id on successor | Successor tests must keep clearing continuity while preserving exact launch truth |
| Caller parity | `agents_cmd.rs` and `async_repl.rs` resolve equivalent inventory-backed inputs differently | Integration parity tests must compare resolved contract fields across surfaces |

Critical gap rule:

If any resolved field can differ across callers for equivalent baseline input, or if persisted attach truth is still missing any field that 30 or 31 needs, this slice is not done.

## Performance Review

This is a correctness slice, but two performance constraints are worth freezing now:

1. keep full provenance out of durable session JSON;
2. do not turn resolution into repeated inventory scans inside one command path.

Hard rules:

1. resolve effective inventory once per command path;
2. build one resolved contract per launch action;
3. materialize runtime descriptors from that result;
4. persist only the attach-relevant subset.

Potential footguns:

1. serializing giant provenance trees into session files;
2. repeatedly reloading inventory in `agents_cmd.rs`, `state_store.rs`, and `async_repl.rs` separately;
3. over-abstracting the resolver into a trait maze because "future callers might need it."

The boring solution is correct:

1. plain structs;
2. one resolver function or small resolver object;
3. one additive durable-state update;
4. one pass per launch.

## Docs And Truth Sync

When this slice lands, update all of these surfaces together:

1. [ADR-0027](docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md)
   - document baseline domains,
   - document supported override families,
   - document merge precedence,
   - document narrowing-only behavior for dispatch-time policy overlays;
2. [llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
   - keep it aligned to the landed type split and denial taxonomy;
3. [llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)
   - point at the frozen internal scope and host-start knobs from this slice;
4. [llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)
   - point at the generalized persisted attach truth and attach-mode vocabulary from this slice;
5. [llm-last-mile/README.md](llm-last-mile/README.md)
   - explain the contract stack ordering cleanly.

Doc rule:

The repo should explain the contract once, then reference it.  
Do not write five slightly different descriptions of merge precedence.

## Worktree Parallelization Strategy

This slice has one real foundation lane, one durable-state freeze lane, and one safe parallel window after those land. Do not split the contract definition itself across multiple workers.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A0. Contract vocabulary freeze | `PLAN.md`, `llm-last-mile/29*.md`, truth docs | - |
| A1. Shared resolver module | `agent_runtime/dispatch_contract.rs`, `agent_runtime/mod.rs`, `agent_inventory.rs` | A0 |
| A2. Runtime materialization convergence | `agent_runtime/validator.rs`, `agent_runtime/control.rs` | A1 |
| A3. Generalized host attach contract | `agent_runtime/orchestration_session.rs`, `agent_runtime/state_store.rs` | A1, A2 |
| A4. Human caller adoption | `agents_cmd.rs`, `agent_runtime/control.rs`, `prompt_fulfillment.rs` | A2, A3 |
| A5. Orchestrator-controlled dispatch adoption | `async_repl.rs`, maybe `routing/dispatch/world_ops.rs` | A1, A2 |
| A6. Truth-doc and downstream-slice sync | `docs/`, `llm-last-mile/` | A1; final wording waits for A3-A5 |
| A7. Final validation and closeout | targeted tests, full workspace tests, final doc pass | A4, A5, A6 |

### Parallel lanes

Lane A: A0 -> A1 -> A2 -> A3  
Reason: the contract shape, materialization boundary, and persisted attach schema must stabilize first.

Lane B: A4  
Reason: once the contract and persisted attach truth are frozen, human CLI surfaces can adopt them without guessing.

Lane C: A5  
Reason: orchestrator-controlled world-member dispatch can adopt the shared resolver after A2, as long as it does not reopen attach persistence semantics.

Lane D: A6  
Reason: docs can start after A1 freezes names, but wording must be finalized only after runtime semantics are confirmed.

Lane V: A7  
Reason: validation is the merge gate and must run after runtime and docs converge.

### Execution order

1. Land A0, A1, and A2 first.
2. Land A3 next. This is the durable-state freeze.
3. After A3, launch Lane B and Lane D in parallel.
4. Launch Lane C in parallel with Lane B only if it stays inside `async_repl.rs` and optional dispatch transport adapters and does not reopen `orchestration_session.rs` or `state_store.rs`.
5. Merge runtime lanes.
6. Finalize docs.
7. Run Lane V last.

### Conflict boundaries

1. `dispatch_contract.rs` belongs to Lane A only. Parallel edits there guarantee semantic drift.
2. `orchestration_session.rs` and `state_store.rs` belong to Lane A through A3. Nobody else edits those files before A3 lands.
3. `agents_cmd.rs`, `agent_runtime/control.rs`, and `prompt_fulfillment.rs` belong to Lane B after A3. Lane C does not touch them.
4. `async_repl.rs` and optional additive `world_ops.rs` changes belong to Lane C. Lane B does not touch them.
5. docs may draft early, but they do not merge before runtime semantics are confirmed.

### Parallelization verdict

Peak low-risk parallelism is `Lane B + Lane D`.  
`Lane C` is parallel-safe only if it respects the module boundary and does not reopen the shared contract or durable attach schema.

## Validation Commands

### Targeted contract and persistence tests

```bash
cargo test -p shell resolve_public_control_target -- --nocapture
cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture
cargo test -p shell new_session_starts_active_attached -- --nocapture
cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

### World-member and parity validation

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

### Workspace gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

### Manual validation expectations

Manual validation must prove:

1. inventory-backed defaults resolve deterministically;
2. persisted attach truth is used for reattach/fork and detached-turn attach planning;
3. supported overrides narrow or select the effective launch contract correctly;
4. policy-denied overrides fail closed with field-specific reasons;
5. persisted host attach truth matches the resolved host launch contract;
6. equivalent human and orchestrator-controlled inventory-backed inputs resolve to equivalent launch truth.

## Deferred Work

There is no root `TODOS.md` today, so the real deferrals stay here and in the downstream slice docs.

1. public capability-flag CLI syntax
   - deferred to slice 30 after the internal contract is frozen;
2. born-unattached status semantics
   - deferred to slice 31;
3. automatic attach-worker launch from pending work
   - deferred to slice 31 after the operator contract is chosen;
4. richer explain surfaces such as a dedicated `dispatch explain` command
   - optional future DX slice, not needed to land the internal contract;
5. any wire-level expansion of `MemberDispatchTransportRequest`
   - only if orchestrator-controlled parity proves existing fields are insufficient.

## Completion Checklist

- [ ] one shared internal dispatch request envelope exists
- [ ] inventory-backed and persisted-attach-backed baseline domains are explicit
- [ ] one resolved launch contract exists with exact provenance
- [ ] denial taxonomy is explicit and fail closed
- [ ] supported override families are explicit and bounded
- [ ] policy narrowing is deterministic and explanation-ready
- [ ] `HostAttachContract` persists generalized host launch truth
- [ ] successor attach-contract copy clears continuity and preserves exact launch truth
- [ ] human `start` consumes the shared inventory-backed resolver
- [ ] detached attach planning consumes persisted attach truth from that resolver
- [ ] orchestrator-controlled world-member dispatch consumes the shared resolver
- [ ] no caller-specific launch dialect remains
- [ ] docs publish merge precedence and baseline domains exactly once
- [ ] slices 30 and 31 reference the frozen contract instead of inventing their own
- [ ] targeted tests are green
- [ ] full workspace gates are green
