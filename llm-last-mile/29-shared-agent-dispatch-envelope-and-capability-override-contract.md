# SOW: Shared Agent Dispatch Envelope And Capability Override Contract

Status: remaining-work draft. This SOW closes the next contract-definition slice after [28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md). It is anchored to [ADR-0025 — Agent Hub Core](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md), [ADR-0026 — Orchestration Toolbox](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md), [ADR-0027 — LLM and Agent Config/Policy Surface](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md), and the current inventory/runtime code in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:13).

This slice is not “invent agent profiles.” The inventory/profile layer already exists. The missing contract is one shared dispatch-time override and resolution model that both human commands and orchestrator-only tool calls can use.

## Objective

Define and implement one shared dispatch envelope that:

- uses agent inventory as the baseline profile/default layer,
- accepts dispatch-time overrides in a strict, fail-closed way,
- resolves inventory defaults plus dispatch overrides plus policy restrictions into one effective launch contract,
- and is reusable by both human CLI dispatch and orchestrator-only tool-call dispatch.

This slice is done only when all of the following are true:

1. One explicit dispatch envelope exists as a shared internal contract.
2. Both human-facing dispatch surfaces and orchestrator-only tool-call dispatch can target that same envelope.
3. Capability overrides are validated, merged, and policy-gated in one place.
4. Inventory defaults remain the source of baseline truth rather than being bypassed by ad hoc flags.

## Already Landed And Assumed

This SOW assumes the following are already true and must not be redesigned here:

- agent inventory items already live under `~/.substrate/agents/*.yaml` and workspace overrides under `.substrate/agents/*.yaml` as documented in [ADR-0027](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:204),
- inventory already carries `config.kind`, `config.execution.scope`, `config.cli.binary`, `config.cli.mode`, and basic capability booleans in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:25),
- inventory already supports restriction-only `policy_overlay` in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:19),
- derived backend ids already follow the `<kind>:<name>` contract in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:105),
- the orchestrator remains host-scoped in v1 per [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md:141) and [ADR-0026](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md:56),
- and worker/world launches are expected to be primarily orchestrator-dispatched rather than orchestrator-in-world.

## Current Repo Truth

### Inventory/profile already exists

The repo already has the right baseline concepts:

- one-file-per-agent inventory,
- backend id derivation,
- default scope,
- default CLI mode,
- basic capability booleans,
- and restriction-only policy overlays.

That means this slice must not pretend the profile layer is missing.

### The missing contract is dispatch-time capability selection

What still looks under-modeled:

- explicit dispatch-time scope selection and override precedence,
- richer capability choices such as selected MCP servers, selected skills, narrower policy views, and other custom capability bundles,
- one effective resolved launch envelope shared by human and orchestrator-controlled dispatch,
- and one validation story for unsupported or policy-blocked overrides.

## Required Shared Contract

This slice must create one shared internal dispatch contract with at least these conceptual layers:

1. Inventory/profile defaults
   - backend id
   - default scope
   - default mode
   - declared capabilities
   - restriction-only policy overlay
2. Dispatch request
   - prompt or task payload
   - explicit scope override
   - explicit capability overrides
   - explicit optional selections for allowed tooling or skills
3. Resolved effective launch contract
   - the final merged backend, scope, capability set, and policy-restricted launch envelope that runtime code actually consumes

The merge rules must be explicit and fail closed.

## In Scope

- define the shared dispatch envelope,
- define merge precedence between inventory defaults, dispatch-time overrides, and policy restrictions,
- model dispatch-time capability overrides for both human and orchestrator-driven launches,
- leave room for selected MCP/tool affordances, skills, narrower policy overlays, and other explicit capability bundles,
- and land the shared runtime resolution seam without forcing full public CLI contract expansion in the same slice.

## Out Of Scope

This slice does not include:

- broadening public `substrate agent start` on its own,
- choosing final user-facing flag names for every capability category if those belong to a later CLI-focused slice,
- moving the orchestrator in-world,
- bypassing inventory defaults with fully ad hoc one-off launch requests,
- or baking worker capability semantics directly into the gateway fulfillment rewrite.

## Concrete Work Breakdown

### 1. Freeze inventory as the source of baseline truth

Primary anchors:

- [ADR-0027](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md:204)
- [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:13)

Required outcome:

- the new dispatch model explicitly starts from inventory-defined defaults,
- inventory remains the durable profile layer,
- and dispatch overrides do not replace or bypass the inventory system.

### 2. Define one shared dispatch request envelope

Primary anchors:

- [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md)
- [ADR-0026](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md)

Required outcome:

- one internal request envelope exists for worker launches,
- that envelope can be consumed by a human CLI surface or an orchestrator-only tool surface,
- and the envelope has stable semantics independent of which caller produced it.

### 3. Define explicit capability override categories

Primary anchors:

- [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md)
- [ADR-0026](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md)

Required outcome:

- the repo explicitly defines which capability families are first-class dispatch overrides,
- at minimum leaving room for:
  - scope,
  - backend selection where permitted,
  - narrower policy overlays,
  - selected MCP/tool affordances,
  - selected skills,
  - and other explicit custom capability bundles,
- and unsupported capability categories fail closed instead of being silently ignored.

### 4. Define merge precedence and fail-closed validation

Primary anchors:

- [ADR-0027](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md)
- [PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/PHASE_8_CROSS_CUTTING_DECISION_REGISTRY.md)

Required outcome:

- one merge order is chosen and implemented,
- policy restrictions remain narrowing-only,
- invalid or disallowed overrides fail closed with actionable errors,
- and effective launch capability truth is explainable and deterministic.

### 5. Keep orchestrator-owned worker dispatch as the primary model

Primary anchors:

- [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md:141)
- [ADR-0026](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md:131)

Required outcome:

- the shared envelope must support human launches,
- but it must also clearly support the primary v1 model where world workers are usually dispatched by the host orchestrator through orchestrator-only tools,
- and the same effective launch semantics must apply in both cases.

## Required Test Additions Or Tightening

### Inventory-resolution coverage

Required scenarios:

- inventory defaults resolve deterministically,
- workspace inventory overrides global inventory correctly,
- and unsupported inventory/data combinations still fail closed.

### Dispatch-merge coverage

Required scenarios:

- dispatch overrides narrow or refine defaults correctly,
- policy overlays remain restriction-only,
- invalid capability requests fail closed,
- and the effective resolved launch contract is stable and explainable.

### Caller-parity coverage

Required scenarios:

- the same envelope semantics hold when dispatch originates from a human CLI path,
- the same envelope semantics hold when dispatch originates from an orchestrator tool path,
- and no second ad hoc worker-launch contract appears.

## Acceptance Criteria

- one shared dispatch envelope exists and is consumed by both human-facing and orchestrator-driven dispatch paths.
- inventory/profile remains the baseline source of defaults.
- dispatch-time scope and capability overrides have explicit semantics.
- policy restrictions stay fail-closed and narrowing-only.
- richer worker capability selection is modeled as explicit launch input rather than implicit hardcoded runtime behavior.

## Validation Expectations

- run targeted inventory/config/policy tests and any new dispatch-resolution tests,
- run the full touched package coverage and then full workspace tests:
  - `cargo test --workspace -- --nocapture`
- manual validation for this slice must explicitly demonstrate:
  - inventory defaults,
  - dispatch override application,
  - policy-denied override failure,
  - and equivalent behavior between human and orchestrator-driven dispatch inputs.

## Docs And Truth Sync

When this slice is closed:

- update docs so inventory/profile is clearly described as the baseline layer,
- document the shared dispatch envelope and merge rules explicitly,
- and stop implying that worker capability selection is only hardcoded profile behavior or only future hand-waving.

