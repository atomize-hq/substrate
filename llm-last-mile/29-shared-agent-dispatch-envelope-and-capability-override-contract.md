# SOW: Shared Agent Dispatch Envelope And Capability Override Contract

Status: implementation-ready follow-on slice. This SOW follows [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md) and is the next execution slice after 28.5. It is anchored to [ADR-0025](../docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md), [ADR-0026](../docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md), [ADR-0027](../docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md), and the current inventory/runtime code in [crates/shell/src/execution/agent_inventory.rs](../crates/shell/src/execution/agent_inventory.rs).

This slice is not a generic capability-injection exercise. It is the contract-definition slice that turns inventory defaults, dispatch-time overrides, and policy narrowing into one resolved launch contract and one generalized persisted host attach contract that later slices can reuse without guessing.

## Objective

Define and implement one shared internal dispatch contract that:

1. starts from inventory/profile defaults,
2. accepts strict dispatch-time overrides,
3. resolves inventory plus overrides plus policy into one effective launch contract,
4. generalizes the already-landed minimal persisted host-orchestrator attach contract under the durable orchestration session,
5. is consumed by both human launch surfaces and orchestrator-controlled dispatch surfaces.

This slice is done only when the repo has one deterministic contract that explains:

1. how a worker launch is resolved,
2. how a future host attach is derived from persisted `HostAttachContract` truth,
3. which capability overrides are allowed,
4. why a requested override was accepted or rejected.

## Landed Floor From 28.5

The current repo already has the minimum durable attach seam that 28.5 said it would land:

1. exact backend/runtime selection already resolves through `RuntimeSelectionDescriptor`,
2. durable launch-descriptor persistence already exists through `ResolvedRuntimeDescriptor` plus `HostAttachContract`,
3. durable orchestration-session state already stores that contract under `OrchestrationSessionRecord.host_attach_contract`,
4. session-birth persistence, continuity sync, and successor-copy behavior already exist through `HostAttachContract::from_manifest(...)`, `sync_host_attach_contract(...)`, and `fork_successor_attach_contract(...)`,
5. public `reattach` already plans from the persisted attach contract and fails closed when required continuity is missing,
6. public `fork` already allocates a successor durable session, copies the attach-contract shape forward, clears inherited continuity, and returns truthful `parked_resumable` posture.

What is still missing is the broader shared dispatch-envelope layer above that seam:

1. one first-class dispatch request envelope shared by human and orchestrator-controlled callers,
2. explicit override taxonomy for dispatch-time capability/scope shaping,
3. explanation-ready provenance for why requested fields were accepted, narrowed, or denied,
4. one unified contract surface that all future host/world launch planning reuses.

## Landed Implementation Truth

The merged runtime truth for this slice is now:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs` is the single internal owner of shared dispatch resolution semantics.
2. `DispatchRequestEnvelope` is the shared caller contract for both human control surfaces and orchestrator-controlled dispatch.
3. The baseline domains are explicit:
   - inventory-backed resolution for new dispatch,
   - persisted-attach-backed resolution for later host attach, detached follow-up turn recovery, and successor allocation.
4. `DispatchCapabilityOverrideSet` and `AttachLaunchKnobs` are the named override families consumed by runtime code.
5. Runtime materialization flows from the resolved shared contract rather than from caller-specific launch planning.
6. `HostAttachContract` persists the generalized resolved host launch truth, including continuity selector state when present, and no second durable attach object exists.
7. Human `start` / `reattach` / `turn` / `fork` and orchestrator-controlled dispatch now consume the same contract semantics.

## Validated Architecture Assumptions

This SOW assumes the following from 28.5 and treats them as fixed floor:

1. the durable authority is the Substrate orchestration session, not the attached host execution client;
2. public `reattach` is a Substrate control action;
3. public `fork` is a Substrate successor allocator;
4. public `--scope world` will mean host-rooted orchestration plus world worker, not standalone world-root continuity;
5. the previously missing durable seam is now present as a minimal persisted host attach contract, and the remaining gap is the shared dispatch-envelope and override contract that generalizes it.

## Frozen Decisions

### 1. Inventory remains the baseline truth

Inventory already owns:

1. agent identity,
2. backend kind and derived backend id,
3. default execution scope,
4. default CLI mode,
5. baseline capability declarations,
6. restriction-only policy overlays.

This slice must not bypass that layer.

### 2. One resolved launch contract feeds both launch and later attach

The same resolution engine must produce:

1. the contract used to launch a host or world runtime now,
2. the generalized host attach contract persisted for later host attach or lazy attach.

Later attach must not reconstruct launch truth from whatever participant happened to be active most recently.

### 3. Policy remains narrowing-only and fail-closed

Policy may:

1. deny a backend,
2. deny a scope,
3. deny a capability family,
4. narrow capability selections.

Policy may not broaden inventory defaults or silently coerce invalid requests into something "close enough."

### 4. Caller parity is mandatory

Human CLI dispatch and orchestrator-controlled dispatch must resolve through the same internal contract. No second worker-launch dialect is allowed.

### 5. This slice owns the persisted host attach contract generalization

28.5 already introduced the minimum durable host attach contract needed to break the hidden owner-helper coupling. This slice expands the existing `RuntimeSelectionDescriptor` / `ResolvedRuntimeDescriptor` plus `HostAttachContract` path into the stable shared resolved-contract shape that 30 and 31 will reuse.

## Ownership Boundaries

### Substrate resolution layer owns

1. dispatch request parsing,
2. inventory-plus-override-plus-policy merge logic,
3. the resolved launch contract,
4. generalization of the already-persisted host attach contract under the durable orchestration session,
5. caller parity between human and orchestrator-controlled dispatch.

### Inventory layer owns

1. baseline backend identity and profile defaults,
2. default scope and CLI mode,
3. baseline capability declarations,
4. restriction-only embedded policy overlays.

### Policy layer owns

1. allow or deny decisions for backends, scopes, and capability categories,
2. narrowing of the resolved launch contract,
3. fail-closed rejection when a request exceeds allowed posture.

### UAA and backend adapters own

1. execution of the prompt-bearing launch contract after resolution is complete,
2. backend-native continuity selectors once Substrate chooses to use them.

### UAA and backend adapters do not own

1. dispatch-time override resolution semantics,
2. persisted host attach contract shape,
3. durable-session attachability semantics,
4. caller-specific launch dialects.

## In Scope

1. define the shared dispatch request envelope,
2. define merge precedence between inventory defaults, dispatch overrides, and policy restrictions,
3. define the resolved launch contract consumed by runtime code,
4. define the generalized persisted host attach contract derived from that resolved launch contract,
5. validate caller parity across human and orchestrator dispatch,
6. make failures explicit and explainable.

## Out Of Scope

1. shipping public `substrate agent start --scope world`,
2. implementing lazy host attach,
3. moving the orchestrator in-world,
4. inventing a second ad hoc launch system separate from inventory,
5. broad public CLI design for every possible future capability family.

## Contract Shape To Land

The implementation may choose exact type names, but it must land the following conceptual layers.

### 1. Dispatch request

Required fields:

1. selected agent or backend target,
2. caller kind,
3. requested execution scope override when allowed,
4. requested capability overrides,
5. requested prompt or task payload when the launch is prompt-bearing,
6. workspace and policy-resolution context.

### 2. Resolved launch contract

Required contents:

1. resolved backend identity,
2. resolved execution scope,
3. resolved protocol and launch descriptor,
4. resolved capability set,
5. effective policy restrictions applied,
6. explanation-ready provenance of which layer supplied or denied each field.

### 3. Persisted host attach contract

Required contents:

1. the resolved host-orchestrator launch contract needed to attach a host execution client later,
2. attach-relevant capability selections and restrictions,
3. continuity selector state when a backend-native session id exists,
4. enough exact launch truth to support both:
   - continuity attach,
   - fresh attach.

## Concrete Work Breakdown

### 1. Freeze inventory as baseline input

Primary anchors:

- [ADR-0027](../docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md)
- [crates/shell/src/execution/agent_inventory.rs](../crates/shell/src/execution/agent_inventory.rs)

Required outcome:

1. the resolution engine starts from inventory-defined defaults,
2. workspace inventory still overrides global inventory,
3. invalid inventory/data combinations fail closed before runtime launch.

### 2. Define one dispatch request envelope

Primary anchors:

- [ADR-0025](../docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md)
- [ADR-0026](../docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md)

Required outcome:

1. one internal request shape exists for worker launch and host attach planning,
2. the envelope is stable regardless of whether the caller was human or orchestrator-controlled,
3. prompt-bearing and non-prompt-bearing callers share the same resolution substrate.

### 3. Define first-class override categories

Required outcome:

1. the repo explicitly names which override families are supported now,
2. unsupported families fail closed,
3. the minimum supported families cover:
   - execution scope,
   - explicit capability narrowing or selection,
   - attach-relevant launch knobs needed later by 30 and 31,
   - narrower policy overlays where permitted.

### 4. Define merge precedence and failure behavior

Primary anchors:

- [ADR-0027](../docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md)
- [docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md](../docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md)

Required outcome:

1. one merge order is chosen and implemented,
2. policy stays narrowing-only,
3. invalid or disallowed overrides produce actionable errors,
4. the effective contract is deterministic and explainable.

### 5. Persist the host attach contract under the orchestration session

Primary anchors:

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md)

Required outcome:

1. the existing host-rooted session-birth persistence path now writes the generalized exact host attach contract,
2. the existing successor-allocation path now copies that generalized contract forward,
3. later attach does not infer launch truth from ambient inventory or a stale participant record.

### 6. Enforce caller parity

Required outcome:

1. human launches and orchestrator-controlled launches both resolve through the same code path,
2. the same effective launch contract is visible to tests and diagnostics in both paths,
3. no second hidden contract appears in REPL-only or toolbox-only code.

## Acceptance Criteria

1. One shared dispatch envelope exists and is the only launch-resolution contract used by runtime code.
2. Inventory remains the baseline source of defaults.
3. Dispatch-time scope and capability overrides have explicit, fail-closed semantics.
4. The resolved launch contract is deterministic and explainable.
5. The already-landed persisted host attach contract seam is generalized so the contract written under the orchestration session and copied across successors comes from the shared dispatch contract.
6. Human and orchestrator-controlled callers consume the same resolution logic.

## Validation Plan

Run at minimum:

1. targeted inventory/config tests,
2. targeted dispatch-resolution tests,
3. targeted state-store tests covering persisted host attach contract behavior,
4. parity tests proving equivalent resolution for human and orchestrator-controlled launch requests,
5. full workspace tests:
   - `cargo test --workspace -- --nocapture`

Manual validation must prove:

1. inventory defaults resolve deterministically,
2. allowed overrides narrow the effective launch contract correctly,
3. denied overrides fail closed with actionable errors,
4. the persisted host attach contract matches the resolved host launch contract,
5. the same effective contract is produced for equivalent human and orchestrator-controlled inputs.

## Docs And Truth Sync

When this slice closes:

1. update [ADR-0027](../docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md) or successor truth docs if they still underspecify dispatch-time override resolution,
2. document the shared dispatch envelope and merge rules explicitly,
3. document the generalized persisted host attach contract as a Substrate durable-state concept without regressing the already-landed 28.5 floor,
4. stop implying that future host attach can be reconstructed by guessing from the last attached participant.

## Sequencing

Ready to implement after 28.5:

1. this SOW.

Downstream slices that depend on this landing and remain draft:

1. [30-public-world-scoped-agent-start-and-capability-flags.md](30-public-world-scoped-agent-start-and-capability-flags.md)
2. [31-lazy-host-attach-for-host-rooted-world-start.md](31-lazy-host-attach-for-host-rooted-world-start.md)
