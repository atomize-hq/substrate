# SOW: Public World-Scoped Agent Start And Capability Flags

Status: remaining-work draft. This SOW is the public contract follow-on after [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md). It is anchored to [ADR-0025 — Agent Hub Core](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md), [ADR-0027 — LLM and Agent Config/Policy Surface](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md), and [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md).

This slice is a real public contract expansion. Today, public root start remains host-only. This slice is where that rule changes, if the repo chooses to change it.

## Current Frozen Contract

The current public contract still says:

- `substrate agent start --backend <backend_id> --prompt ...` is host-only in v1 in [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:115),
- world-only root start is intentionally rejected today in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:915),
- and [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:90) still fixes root `start` as host-only in v1.

This SOW is the slice that would intentionally broaden that contract.

## Objective

Broaden the public `substrate agent start` surface so a human can explicitly launch host-scoped or world-scoped agents through one shared dispatch contract, including explicit capability flags.

This slice is done only when all of the following are true:

1. `substrate agent start` accepts an explicit scope selector such as `--scope host|world`.
2. Human launches and orchestrator-only tool launches consume the same underlying dispatch envelope.
3. Capability flags exposed on the human command map onto the same effective launch contract used by orchestrator tool calls.
4. The new public world-scoped start semantics are explicit, documented, and fully tested.

## Required Product Decision

Before implementation is considered closed, this slice must choose one exact meaning for public `--scope world` root start.

### Option A: Host-rooted orchestration session plus world worker

Public `substrate agent start --scope world ...`:

- creates or binds a host-rooted orchestration session,
- launches a world-scoped worker/member under that orchestration session,
- and keeps the durable orchestration authority host-rooted.

This option is more compatible with the current [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) model.

### Option B: Standalone world-root session

Public `substrate agent start --scope world ...`:

- can create a standalone world-root session with no host-rooted orchestrator session above it.

This option is a broader contract change and requires a more substantial rethink of the current host-rooted durability model.

This SOW does not assume A or B. It requires the repo to choose one and align the public contract, runtime semantics, and docs to that choice.

## Already Landed And Assumed

This SOW assumes the following are already true and are not being redesigned here:

- the shared dispatch envelope from [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md) exists or is the prerequisite contract,
- agent inventory already provides baseline defaults and policy overlays in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:13),
- the orchestrator remains host-scoped in the current v1 model under [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md:141),
- and gateway-mediated LLM fulfillment is handled by the prior slice rather than being invented here.

## In Scope

- broaden public `substrate agent start` beyond host-only,
- add explicit scope selection for public root start,
- add explicit human-facing capability flags that map onto the shared dispatch envelope,
- align orchestrator tool launches and human launches behind the same launch contract,
- and document exactly what a public world-scoped root start means.

## Out Of Scope

This slice does not include:

- moving the orchestrator itself in-world,
- bypassing the shared dispatch envelope,
- inventing a second public worker-launch system separate from orchestrator tools,
- or reopening the gateway-mediated fulfillment seam as a separate architecture project.

## Concrete Work Breakdown

### 1. Freeze one public meaning for `--scope world`

Primary anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md)

Required outcome:

- the repo chooses whether world-scoped public root start is host-rooted or standalone world-root,
- that rule is explicit,
- and the rule is enforced consistently by runtime and docs.

### 2. Expose scope selection on public `agent start`

Primary anchors:

- [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)

Required outcome:

- the public root start command accepts an explicit scope selector,
- invalid scope/backend combinations fail closed,
- and current host-scoped starts continue to work under the same user-visible semantics.

### 3. Expose capability flags on the human command

Primary anchors:

- [ADR-0027](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md)
- [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)

Required outcome:

- human operators can express explicit launch-time capability choices as flags,
- those flags map onto the same internal dispatch envelope used by orchestrator tool calls,
- and the flag set is explicit rather than relying entirely on hidden inventory defaults.

### 4. Keep lifecycle semantics stable while broadening root start

Primary anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Required outcome:

- broadening public root start must not reintroduce hidden bootstrap prompts,
- must not regress `start`/`turn`/`reattach`/`stop` semantics for host-rooted orchestration,
- and must keep durable-session behavior truthful and explicit under the chosen contract.

### 5. Align human launch and orchestrator launch under one model

Primary anchors:

- [ADR-0025](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md)
- [ADR-0026](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md)

Required outcome:

- the human command is a first-class supported launch path,
- orchestrator tool calls remain the primary worker-dispatch path in normal orchestration use,
- and both consume the same effective launch contract rather than diverging over time.

## Required Test Additions Or Tightening

### Public CLI coverage

Required scenarios:

- `substrate agent start --scope host ...` preserves current behavior,
- `substrate agent start --scope world ...` follows the chosen contract exactly,
- invalid capability flags or invalid scope/backend combinations fail closed,
- and user-facing JSON/text output stays explicit about what was launched.

### Shared-envelope parity coverage

Required scenarios:

- human capability flags resolve to the same effective launch contract as orchestrator tool-call inputs,
- policy-denied flags fail closed in both paths,
- and inventory defaults remain visible in effective launch resolution.

### Lifecycle regression coverage

Required scenarios:

- no hidden bootstrap prompts reappear,
- host durable-session behavior remains correct,
- and any new world-scoped root start behavior is fully explicit rather than heuristic.

## Acceptance Criteria

- public `substrate agent start` supports explicit scope selection.
- human operators can express launch-time capability choices as explicit flags.
- the human command and orchestrator tool paths share one dispatch contract.
- the exact meaning of public world-scoped root start is frozen and documented.
- current host-scoped lifecycle semantics do not regress.

## Validation Expectations

- run targeted CLI, inventory, dispatch, and durable-session tests,
- run full touched package coverage and then full workspace tests:
  - `cargo test --workspace -- --nocapture`
- manual validation for this slice must explicitly exercise:
  - host-scoped public root start,
  - world-scoped public root start,
  - capability flag resolution,
  - and parity between human launches and orchestrator-driven launches.

## Docs And Truth Sync

When this slice is closed:

- update [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md),
- update [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) or its successor truth surfaces where they still say root start is host-only,
- and document the public flag surface so world-scoped launch and capability overrides are explicit operator-facing contract rather than tribal knowledge.
