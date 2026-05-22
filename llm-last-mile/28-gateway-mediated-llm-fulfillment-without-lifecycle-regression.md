# SOW: Gateway-Mediated LLM Fulfillment Without Lifecycle Regression

Status: remaining-work draft. This SOW closes the runtime seam after [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md) and [27-uaa-boundary-and-naming-cleanup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/27-uaa-boundary-and-naming-cleanup.md). It is anchored to the lifecycle truth in [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), the gateway ownership split in [ADR-0040 — Substrate Gateway Boundary and Runtime Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md), the adapter intent in [ADR-0041 — Substrate Gateway Backend Adapter Contract](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md), and the already-landed gateway contracts under [docs/contracts/](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts).

This slice is a seam replacement, not a public contract expansion. It must not absorb the shared dispatch-envelope work in [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md) or the public `--scope world` / capability-flag work in [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md).

## Objective

Remove direct production construction of `AgentWrapperGateway`, `CodexBackend`, and `ClaudeCodeBackend` from prompt-bearing host execution and world-member execution, and route those requests through the `substrate-gateway` adapter seam instead, without changing the already-landed `start` / `turn` / `reattach` / `stop` lifecycle contract.

This slice is done only when all of the following are true:

1. Production host prompt-taking paths no longer fulfill through direct backend registration in shell-owned runtime code.
2. Production world-member prompt-taking paths no longer fulfill through direct backend registration inside `world-service` member runtime code.
3. The effective fulfillment seam for those paths is the existing gateway adapter/runtime contract rather than duplicated shell-local or world-local wrapper construction.
4. The current user-visible lifecycle and prompt semantics remain unchanged.
5. The current secure host-to-world auth bundle handoff remains the normal integrated auth carrier.

## Sequencing And Landed Prerequisites

This slice assumes all of the following are already landed and must be treated as fixed floor, not reopened here:

1. [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md) is the lifecycle floor for parked/resumable durability, `reattach`, `stop`, and `Accepted -> terminal`.
2. [27-uaa-boundary-and-naming-cleanup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/27-uaa-boundary-and-naming-cleanup.md) is the cleanup floor for the current boundary terminology and runtime naming.
3. The current typed world-member follow-up seam on `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream` is already landed and remains frozen as fixed by [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md:103).

This slice does not absorb:

1. dispatch-time capability override modeling from `29`,
2. public world-root `start` or human capability flags from `30`,
3. new config or policy surfaces already owned by [ADR-0027](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md),
4. new backend-selection semantics beyond the already-landed stable `<kind>:<name>` backend-id contract,
5. generic multi-backend gateway expansion beyond the currently wired integrated bindings.

## Frozen Behavior Contract

This slice must preserve all of the following exactly:

1. `substrate agent start` remains host-only in v1.
2. `substrate agent start` uses the user prompt as the true initial prompt. No hidden bootstrap prompt, synthetic warm-up turn, or composed pre-prompt may reappear.
3. `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` remains the exact public follow-up contract. No fuzzy routing, default routing, or selector widening is allowed here.
4. `substrate agent reattach --session <orchestration_session_id>` remains recovery-only. It does not submit a prompt and does not become a shortcut for `turn`.
5. `substrate agent stop --session <orchestration_session_id>` remains the canonical closeout surface for attached, parked, and attention-needed durable host sessions.
6. Once a public prompt request emits `Accepted`, the bridge must still terminate with an explicit terminal envelope. EOF without `Completed` or `Failed` remains a bug.
7. Durable host postures remain authoritative:
   - `parked_resumable` means detached, resumable, and no pending inbox work.
   - `awaiting_attention` means detached, resumable, and pending inbox work.
   - `terminal` remains the only non-routable posture family.
8. Detached world follow-up remains fail-closed until the valid host ownership path is restored.
9. Linux world follow-up remains on the typed `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream` seam.

Primary truth anchors:

- [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

## Already Landed And Assumed

This SOW assumes the following are already true and must be reused rather than redesigned:

1. The gateway ownership split is already fixed in repo truth:
   - Substrate owns policy evaluation, world placement, lifecycle control, host-to-world secret delivery, operator UX, and canonical tracing.
   - `substrate-gateway` owns the in-world front door, adapter dispatch, and backend/provider internals.
   - Primary anchors: [docs/contracts/substrate-gateway-operator-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-operator-contract.md), [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md), and [ADR-0040](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md).
2. Stable backend selection is already a landed contract:
   - backend ids remain stable `<kind>:<name>`,
   - allowlisting happens before adapter dispatch,
   - unsupported-but-well-formed backend ids remain a dependency/runtime-unavailable problem rather than an excuse to reuse another adapter.
   - Primary anchors: [docs/contracts/substrate-gateway-backend-adapter-selection.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-selection.md) and [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md).
3. Integrated auth already has a real secure carrier:
   - host-side auth payload synthesis already exists,
   - world-service already converts that payload into an inherited FD auth bundle,
   - gateway startup already consumes `SUBSTRATE_LLM_AUTH_BUNDLE_FD` once and strips the pointer env.
   - Primary anchors: [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs), [crates/world-service/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs), [crates/common/src/gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs), and [crates/gateway/src/server/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/server/mod.rs).
4. Current integrated binding scope is intentionally narrow:
   - the binding table is currently real for `cli:codex` and `api:openai`,
   - this slice does not broaden the integrated backend matrix by itself.
5. Auth precedence is already fixed and must stay fail-closed:
   - complete allowlisted env auth wins,
   - file reads are fallback-only when env auth is absent,
   - partial env auth is invalid integration,
   - blocked envs do not fall back through a bypass path.

## Current Repo Truth

### The direct fulfillment bypass is still real

The production bypass points are concrete:

1. Host shell runtime still builds a local `AgentWrapperGateway` and directly registers concrete backends in [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs:8).
2. Host follow-up prompt submission still rebuilds that direct gateway in [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1251).
3. REPL-owned host startup and preparation paths still depend on the same direct shell-local builder in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
4. World member runtime still builds another direct `AgentWrapperGateway` and directly registers concrete backends in [crates/world-service/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:557).
5. The shell still prepares a local member gateway for world startup and then throws it away before dispatching `member_dispatch` into world-service. That is a real seam mismatch, not just stylistic duplication.

### The public lifecycle is already narrower than the runtime seam

What is already true:

1. The public verbs and selector rules are narrow and regression-proven.
2. The Linux world follow-up seam is already typed and retained-member aware.
3. Durable parked and attention-needed host semantics are already landed.
4. The gateway lifecycle is already real for `status | sync | restart`.

What is still wrong:

1. production prompt-bearing execution still bypasses the gateway adapter seam,
2. host and world duplicate backend-registration logic,
3. first-turn world launch and resumed world follow-up do not clearly share one backend-fulfillment seam,
4. auth and adapter lifecycle truth exist, but the main agent fulfillment path still does not consume them as its steady-state runtime boundary.

## In Scope

1. Replace direct production host fulfillment for prompt-bearing execution with gateway-mediated fulfillment.
2. Replace direct production world-member fulfillment for prompt-bearing execution with gateway-mediated fulfillment.
3. Remove dead or misleading shell-local world-member gateway preparation that is no longer authoritative once fulfillment is gateway-mediated.
4. Reuse the existing adapter-selection, auth-bundle, and gateway runtime contracts rather than inventing new ones.
5. Preserve current public lifecycle, prompt, and retained-member semantics exactly.
6. Add regression coverage that proves the seam moved while lifecycle behavior stayed frozen.

## Out Of Scope

1. Public `--scope world` root start.
2. Shared dispatch-envelope or capability-override design.
3. New capability/toolbox modeling.
4. New backend-id grammar, new allowlist semantics, or new config/policy files.
5. Generic adapter-matrix expansion beyond the currently supported integrated bindings.
6. Moving the orchestrator in-world.
7. Durable-session redesign, selector widening, or new prompt lifecycle semantics.
8. Reintroducing secret-bearing process env vars as the normal integrated auth path.

## Concrete Work Breakdown

### 1. Freeze the lifecycle and prompt invariants before moving fulfillment

Required outcome:

1. SOW implementation and tests treat [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) and [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md) as immutable behavior truth for this slice.
2. No gateway handshake, adapter bootstrap, or runtime warm-up can surface as the first user-visible prompt.
3. `start`, `turn`, `reattach`, `stop`, `parked_resumable`, `awaiting_attention`, and `Accepted -> terminal` semantics remain as-is.

### 2. Replace the host prompt-bearing fulfillment seam

Primary anchors:

- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required outcome:

1. The first visible host prompt-taking path and later host follow-up prompt-taking path both stop directly registering `CodexBackend` or `ClaudeCodeBackend` in shell-owned runtime code.
2. Host execution remains host-scoped in public meaning even if the actual LLM fulfillment is now gateway-mediated.
3. Host `turn` keeps exact session/backend routing and still surfaces the same completion/posture contract.

### 3. Replace the world-member fulfillment seam for both first-turn and resumed-turn execution

Primary anchors:

- [crates/world-service/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs)
- [crates/world-service/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/service.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required outcome:

1. The first targeted world turn that currently rides `member_dispatch.initial_prompt` and the resumed world follow-up path that currently rides `MemberTurnSubmitRequestV1` both use the same gateway-mediated fulfillment seam.
2. The typed `/v1/member_turn/stream` public/runtime seam remains unchanged.
3. Retained-member identity validation, world binding checks, and fail-closed detached-world behavior remain intact.

### 4. Collapse duplicated backend-registration logic into one explicit fulfillment seam

Required outcome:

1. Adding or changing a fulfillment backend no longer requires duplicating direct wrapper registration logic across shell host runtime and world-service member runtime.
2. Any remaining backend-kind mapping stays explicit and fail-closed, but the production fulfillment seam itself is singular rather than copy-pasted.
3. The shell no longer prepares a local world-member gateway that is discarded before real execution begins.

### 5. Reuse the landed gateway auth and adapter contracts exactly

Primary anchors:

- [docs/contracts/substrate-gateway-backend-adapter-selection.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-selection.md)
- [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md)
- [docs/contracts/substrate-gateway-policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-policy-evaluation.md)
- [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md)

Required outcome:

1. Backend ids remain stable selectors, not auth carriers or provider-quirk carriers.
2. Adapter lookup remains binding-driven and backend-aware.
3. Integrated auth still travels by the existing FD auth-bundle contract.
4. The rewrite does not reintroduce child secret env vars or a second ad hoc auth handoff seam.
5. Runtime reuse, if any, remains keyed by the existing effective binding inputs, including backend id.

### 6. Keep the done condition concrete and grepable

This slice is not done until all of the following are true:

1. No production code under `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/`, or `crates/world-service/src/member_runtime.rs` directly instantiates `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend` for prompt-bearing execution.
2. Any surviving direct references are limited to tests or clearly bounded compatibility harnesses that are explicitly documented as non-production.
3. Host initial prompt execution, host follow-up prompt execution, world first-turn execution, and world resumed-turn execution all flow through the same gateway-mediated fulfillment boundary in practice.

## Required Test Additions Or Tightening

### Host lifecycle and prompt-semantic coverage

Required scenarios:

1. `start` still uses the user prompt as the true first prompt.
2. `turn` still uses the user prompt as the true follow-up prompt.
3. No hidden bootstrap or synthetic warm-up prompt appears in traces, runtime artifacts, or captured stdin.
4. `Accepted -> terminal` remains guaranteed after the fulfillment seam moves.
5. `parked_resumable`, `awaiting_attention`, `reattach`, and `stop` semantics remain unchanged.

Primary regression anchors:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

### World-member seam coverage

Required scenarios:

1. First targeted world turn still uses the real user prompt and does not fall back to a hidden bootstrap path.
2. Resumed world follow-up still goes through `/v1/member_turn/stream`.
3. Retained-member identity validation still rejects mismatched participant/backend/world tuples.
4. Launch-time first turn and resumed follow-up prove the same fulfillment seam rather than drifting apart.

Primary regression anchors:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
- [crates/world-service/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs)

### Gateway/auth/adapter coverage

Required scenarios:

1. The fulfillment rewrite continues to use the FD auth-bundle path.
2. Gateway startup still consumes the auth bundle once and does not depend on child secret env vars.
3. Backend selection remains allowlist-gated and binding-driven.
4. Unsupported-but-well-formed backend ids still fail in the correct dependency/runtime bucket rather than silently reusing another adapter.

Primary regression anchors:

- [crates/shell/tests/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)
- [crates/world-service/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs)
- [crates/world-service/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/service.rs)
- [crates/gateway/tests/openai_shared_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/tests/openai_shared_parity.rs)

### Structural regression coverage

Required scenarios:

1. There is at least one test or invariant check that host and world prompt-bearing execution no longer maintain separate direct backend-registration tables.
2. There is at least one regression check that catches accidental reintroduction of the discarded shell-local world-member gateway prep pattern.

## Acceptance Criteria

1. Production host prompt-bearing fulfillment no longer directly registers `CodexBackend` or `ClaudeCodeBackend` in shell runtime code.
2. Production world-member prompt-bearing fulfillment no longer directly registers `CodexBackend` or `ClaudeCodeBackend` in `world-service` member runtime code.
3. The effective fulfillment seam for those paths is the existing gateway adapter/runtime boundary.
4. Current public `start` / `turn` / `reattach` / `stop` behavior does not regress.
5. Typed `/v1/member_turn/stream` world follow-up behavior does not regress.
6. Integrated auth still travels through the current FD auth-bundle path.
7. The shell no longer constructs and discards a local authoritative world-member fulfillment gateway before dispatching the real world launch.

## Validation Expectations

Run targeted coverage for all touched runtime surfaces and then full workspace tests:

```bash
cargo test --workspace -- --nocapture
```

Manual validation for this slice must explicitly confirm all of the following:

1. no hidden bootstrap prompt reappears,
2. host prompt-taking still works with the same visible lifecycle behavior,
3. world prompt-taking still works with the same visible lifecycle behavior,
4. world follow-up still routes through `/v1/member_turn/stream`,
5. integrated auth still arrives through the FD auth-bundle path rather than child secret env,
6. runtime artifacts or traces provide evidence that fulfillment is actually gateway-mediated.

## Docs And Truth Sync

When this slice closes, update the truth/docs that would otherwise keep describing direct-wrapper fulfillment as acceptable steady-state behavior:

1. [llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md)
2. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md)
4. [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md)
5. [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Review and update if needed, but do not reopen their core ownership/lifecycle rules:

1. [docs/contracts/substrate-gateway-backend-adapter-selection.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-selection.md)
2. [docs/contracts/substrate-gateway-policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-policy-evaluation.md)
3. [ADR-0040](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md)
4. [ADR-0041](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md)

Keep these behavior truths stable rather than rewriting them for this slice:

1. [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
2. [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
