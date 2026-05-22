# SOW: Gateway-Mediated LLM Fulfillment Without Lifecycle Regression

Status: remaining-work draft. This SOW closes the next runtime seam after [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md) and [27-uaa-boundary-and-naming-cleanup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/27-uaa-boundary-and-naming-cleanup.md). It is anchored to [ADR-0040 — Substrate Gateway Boundary and Runtime Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md), [ADR-0041 — Substrate Gateway Backend Adapter Contract](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md), [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), and the current gateway operator/runtime contracts under [docs/contracts/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts).

This slice is not allowed to reopen the public prompt lifecycle. The requirement is to swap the LLM fulfillment seam under the existing public caller and durable-session contracts.

## Frozen Behavior Contract

This slice assumes and preserves all of the following:

1. `substrate agent start`, `turn`, `reattach`, and `stop` keep their currently landed meaning and user-visible behavior.
2. The user prompt remains the true initial or follow-up prompt. No hidden bootstrap prompts, fake warm-up turns, or synthetic pre-prompts may be reintroduced.
3. Root host orchestration remains host-scoped in v1 as fixed by [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:90).
4. Linux world follow-up remains on the typed `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream` public/runtime seam as fixed by [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:104).
5. Durable host-session semantics from [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) remain authoritative.

## Objective

Replace direct backend-wrapper LLM fulfillment with `substrate-gateway` mediated fulfillment wherever LLM requests are actually executed, while preserving the frozen lifecycle contract above.

This slice is done only when all of the following are true:

1. Host orchestrator prompt-taking turns no longer fulfill by directly instantiating `CodexBackend` or `ClaudeCodeBackend` in shell-owned runtime code.
2. World member prompt-taking turns no longer fulfill by directly instantiating `CodexBackend` or `ClaudeCodeBackend` inside `member_runtime`.
3. The effective LLM fulfillment path is mediated through the gateway/adapter seam defined by [ADR-0041](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md).
4. The current user-facing `start`/`turn`/`reattach`/`stop` behavior does not regress.
5. The current secure host-to-world auth bundle handoff remains intact and is reused rather than bypassed.

## Already Landed And Assumed

This SOW assumes the following are already true and must not be redesigned here:

- integrated gateway lifecycle commands already exist and are authoritative under [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:60),
- integrated gateway runtime selection and binding already exist for at least `cli:codex` and `api:openai` in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs:164),
- secure host-to-world auth bundle delivery via inherited FD already exists in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs:1122) and [gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs:30),
- host-side gateway auth payload synthesis already exists in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:640),
- the current direct host prompt-turn runtime still instantiates direct wrappers in [registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs:8),
- and the current world member runtime still instantiates direct wrappers in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:557).

## Current Repo Truth

### The gateway seam exists, but agent fulfillment still bypasses it

What is already real:

- `substrate-gateway` is the intended in-world front door and backend adapter runtime.
- the gateway lifecycle exposes typed status/sync/restart surfaces.
- auth handoff into the integrated gateway has already moved onto a secure FD bundle.

What is still wrong:

- host prompt turns still build a local `AgentWrapperGateway` and register direct concrete backends in [registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs:8),
- host prompt submission still drives that direct gateway in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1251),
- world member bootstraps and submitted turns still build direct wrappers in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:557),
- so UAA-style wrapper plumbing exists, but the true fulfillment seam still bypasses `substrate-gateway`.

### The required change is architectural, not user-facing

The user-facing behavior that must stay frozen:

- host orchestrator stays host-scoped,
- world workers remain world-scoped,
- prompt/turn semantics stay as landed,
- durable parked session behavior stays as landed,
- and typed world-member control/read surfaces stay as landed.

The thing that changes is only the LLM fulfillment path behind those surfaces.

## In Scope

- replace direct host-side backend-wrapper fulfillment with gateway-mediated fulfillment,
- replace direct world-member backend-wrapper fulfillment with gateway-mediated fulfillment,
- reuse the existing backend adapter contract and secure auth handoff path,
- preserve the existing public agent lifecycle and durable-session behavior exactly,
- and add regression coverage proving no hidden bootstrap or prompt semantics drift.

## Out Of Scope

This slice does not include:

- public `--scope world` root start,
- shared dispatch-envelope design for worker capability overrides,
- toolbox capability modeling,
- new public agent selector forms,
- host orchestrator moving in-world,
- redesign of durable parked-session semantics,
- or widening public world-member routing beyond the current typed surfaces.

## Concrete Work Breakdown

### 1. Freeze the prompt lifecycle and no-bootstrap invariants

Primary anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

Required outcome:

- the gateway rewrite must not reintroduce hidden bootstrap prompts,
- the user prompt must remain the first true prompt for `start`,
- and `turn` must remain one prompt-taking follow-up action on the same durable session.

### 2. Remove the direct host fulfillment seam

Primary anchors:

- [registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs:8)
- [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1251)
- [ADR-0041](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md)

Required outcome:

- host orchestrator prompt fulfillment no longer directly registers `CodexBackend` or `ClaudeCodeBackend`,
- the host path instead targets the gateway/adapter seam,
- and host orchestration remains host-scoped even though fulfillment is gateway-mediated.

### 3. Remove the direct world-member fulfillment seam

Primary anchors:

- [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:557)
- [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/service.rs:1485)
- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:104)

Required outcome:

- world member execution continues to use the typed submit surface,
- but the actual LLM fulfillment behind that surface is gateway-mediated rather than direct-wrapper mediated,
- and the public/runtime typed member-turn surface remains stable.

### 4. Reuse the landed auth and adapter seams instead of bypassing them

Primary anchors:

- [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:640)
- [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs:1122)
- [gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs:30)

Required outcome:

- the fulfillment rewrite must go through the existing adapter/auth contract,
- not introduce a second ad hoc auth delivery seam,
- and not regress to secret-bearing process env vars as the normal integrated path.

### 5. Keep durable-session behavior unchanged while fulfillment moves

Primary anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Required outcome:

- moving fulfillment through the gateway must not disturb `Accepted -> terminal` delivery guarantees,
- must not break `parked_resumable` or `awaiting_attention`,
- and must not change `reattach` versus `turn` semantics.

## Required Test Additions Or Tightening

### Host orchestration coverage

Required scenarios:

- `start` still uses the user prompt as the true initial prompt,
- `turn` still uses the user prompt as the true follow-up prompt,
- no hidden bootstrap prompt is observed in traces or runtime artifacts,
- and the host prompt path no longer depends on direct wrapper registration.

### World member coverage

Required scenarios:

- typed world member submission still works through `/v1/member_turn/stream`,
- world member follow-up still surfaces the same typed output behavior,
- and the underlying fulfillment path is gateway-mediated rather than direct wrapper driven.

### Gateway/auth coverage

Required scenarios:

- gateway-mediated host and world fulfillment continue to use the secure FD auth bundle path,
- no secret-bearing env vars become the normal fulfillment seam again,
- and adapter selection stays inventory-backed and allowlist-gated.

### Regression coverage

Required scenarios:

- `start`/`turn`/`reattach`/`stop` user-facing behavior remains unchanged,
- durable parked-session tests remain green,
- and no hidden bootstrap or synthetic warm-up session behavior reappears.

## Acceptance Criteria

- no LLM fulfillment path for host orchestrator or world member execution relies on directly registering `CodexBackend` or `ClaudeCodeBackend` in the current runtime seams.
- `substrate-gateway` or its adapter seam mediates LLM fulfillment wherever LLM requests are actually executed.
- the current secure auth-bundle contract remains the normal integrated path.
- host orchestrator scope remains host-only.
- world member typed surfaces remain stable.
- current public agent lifecycle and durable-session behavior remain unchanged.

## Validation Expectations

- run targeted shell and world-service tests covering host turns, world member turns, and gateway runtime parity,
- run the full touched package coverage and then full workspace tests:
  - `cargo test --workspace -- --nocapture`
- manual validation for this slice must explicitly confirm:
  - no hidden bootstrap prompts,
  - host prompt-taking still works,
  - world member prompt-taking still works,
  - and integrated auth still travels by FD bundle rather than process env.

## Docs And Truth Sync

When this slice is closed:

- update truth/docs so they stop implying that direct wrapper fulfillment is still an acceptable steady-state path,
- keep [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) behavior language intact,
- and align any gateway/runtime docs to the fact that agent fulfillment now actually uses the gateway/adapter seam in practice rather than only in adjacent lifecycle contracts.

