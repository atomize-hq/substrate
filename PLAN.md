# PLAN: Gateway-Mediated LLM Fulfillment Without Lifecycle Regression

Source SOW: [28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md)  
Primary contract anchors: [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [ADR-0040](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md), [ADR-0041](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md), [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md)  
Current workspace branch: `testing`  
Base branch: `main`  
Plan type: runtime seam replacement with lifecycle freeze  
Status: fresh execution plan, unified to `/autoplan` plus `/plan-eng-review` rigor on 2026-05-22

## Objective

Move all production prompt-bearing LLM fulfillment onto the existing `substrate-gateway` adapter seam without changing the already-landed public lifecycle contract.

After this slice:

1. host prompt-taking execution no longer directly constructs `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend` in shell-owned runtime code,
2. world-member prompt-taking execution no longer directly constructs `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend` in `world-service`,
3. host first prompt, host follow-up prompt, world first targeted prompt, and world resumed follow-up all use one production fulfillment story: stable backend id selection first, gateway adapter dispatch second, lifecycle semantics unchanged,
4. the current secure FD auth-bundle handoff remains the normal integrated auth carrier,
5. `start`, `turn`, `reattach`, `stop`, `Accepted -> terminal`, `parked_resumable`, `awaiting_attention`, and typed `/v1/member_turn/stream` behavior remain unchanged from the user and operator point of view.

This is a seam replacement. It is not a public contract expansion, not a lifecycle redesign, and not a backend-matrix expansion project.

## Locked Decisions

These decisions are already made. Implementation does not reopen them.

| Topic | Locked decision | Why |
| --- | --- | --- |
| Public lifecycle | Keep `start`, `turn`, `reattach`, and `stop` exactly as they are | ADR-0047 is already the lifecycle floor |
| Public routing selector | Keep exact `--backend <kind:name>` routing on prompt-bearing follow-up | Selector widening would change operator meaning |
| World follow-up seam | Keep `MemberTurnSubmitRequestV1` plus `POST /v1/member_turn/stream` | That seam is already landed and frozen |
| Initial prompt semantics | The user prompt remains the true first prompt | Hidden bootstrap prompts are a regression |
| Detached world posture | Detached world follow-up remains fail-closed | No ownership-bypass recovery path is allowed |
| Auth carrier | Keep `SUBSTRATE_LLM_AUTH_BUNDLE_FD` and the existing auth bundle contract | Secure carrier already exists and is landed |
| Backend selection grammar | Keep stable backend ids as `<kind>:<name>` | Selection and adapter contracts already own this |
| Gateway ownership | Substrate owns lifecycle and placement; `substrate-gateway` owns adapter dispatch and backend internals | ADR-0040 already fixed the boundary |
| Scope boundary | Do not absorb SOW 29 or SOW 30 | Shared dispatch envelope and public world-scoped start are separate slices |

## Acceptance Criteria

This slice is complete only when all of the following are true:

1. No production code under [`crates/shell/src/execution/agent_runtime/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/), [`crates/shell/src/repl/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/), or [`crates/world-service/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs) directly instantiates `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend` for prompt-bearing execution.
2. Host initial prompt execution and host follow-up prompt execution both fulfill through the gateway adapter seam rather than shell-local backend registration.
3. World launch-time first targeted turn and world resumed follow-up turn both fulfill through the gateway adapter seam rather than world-local backend registration.
4. The effective adapter dispatch target remains the already-landed stable backend id chosen before execution begins.
5. The visible lifecycle contract does not regress:
   - `start` uses the user prompt as the real first prompt,
   - `turn` uses the user prompt as the real follow-up prompt,
   - `reattach` remains recovery-only,
   - `stop` remains canonical closeout,
   - `Accepted` still terminates with an explicit terminal envelope.
6. World follow-up still routes through typed `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream`.
7. Detached world follow-up still fails closed.
8. Integrated auth still travels through the FD auth-bundle handoff and does not regress to child secret env vars.
9. The shell no longer prepares and discards a shell-local authoritative world-member gateway object before actual world dispatch.
10. Tests and invariants prove that host and world prompt-bearing execution no longer maintain separate direct backend-registration tables.

## Scope

In scope:

1. replacing direct host prompt-bearing fulfillment with gateway-mediated fulfillment,
2. replacing direct world-member prompt-bearing fulfillment with gateway-mediated fulfillment,
3. removing hidden synthetic bootstrap-prompt behavior from production world-member startup,
4. collapsing duplicated backend-registration logic into one production fulfillment seam,
5. reusing the landed gateway runtime, adapter, backend-selection, and auth-bundle contracts,
6. tightening regression coverage so lifecycle invariants stay frozen while the seam moves,
7. updating truth docs that would otherwise keep describing the bypass as acceptable steady-state behavior.

## NOT in scope

1. Public `--scope world` root `start`.
2. Shared dispatch-envelope and capability-override work from SOW 29.
3. Human capability flags and world-root public start work from SOW 30.
4. New backend-id grammar, new backend-selection semantics, or allowlist redesign.
5. Generic multi-backend integrated expansion beyond the already-wired bindings.
6. Moving the orchestrator into the world.
7. Durable-session redesign or new posture semantics.
8. New public config or policy surfaces already owned by ADR-0027.
9. Reintroducing secret-bearing process env vars as the normal auth path.

## Current Repo Truth

These facts are already true. This plan builds on them and does not reopen them.

1. The direct production bypass still exists in the repo:
   - shell host runtime builds a local `AgentWrapperGateway` in [`crates/shell/src/execution/agent_runtime/registry.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs),
   - host follow-up prompt submission still consumes that direct builder in [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1251),
   - REPL-owned startup still carries gateway-shaped local runtime state in [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs),
   - world member runtime still builds another direct `AgentWrapperGateway` in [`crates/world-service/src/member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:560).
2. The world-member runtime still contains a synthetic runtime bootstrap prompt in [`runtime_bootstrap_prompt()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:536). That is incompatible with the frozen "real user prompt is the first prompt" rule for production.
3. The typed world-member follow-up seam is already real and should be preserved:
   - request type: [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:29),
   - route: [`/v1/member_turn/stream`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/lib.rs:271).
4. The gateway runtime and auth bundle seams are already real and already owned:
   - auth bundle constant and contract: [`crates/common/src/gateway_auth_bundle.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs),
   - world-service gateway runtime handoff: [`crates/world-service/src/gateway_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs),
   - gateway server consumption of the FD bundle: [`crates/gateway/src/server/mod.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/server/mod.rs).
5. Current shell world dispatch already packages typed launch-time world-member requests with `member_dispatch.initial_prompt`; the public/runtime transport seam is not the thing that needs redesign:
   - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs),
   - [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
6. The current contract stack already says the right thing: shell/operator surfaces must consume a typed runtime surface, and adapter dispatch starts only after stable backend-id selection. The runtime bypass is implementation drift, not contract ambiguity.

## Step 0: Scope Challenge

### What already exists

| Sub-problem | Existing code or surface | Reuse decision |
| --- | --- | --- |
| Lifecycle truth | [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs), [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Reuse exactly. This slice must not reinterpret lifecycle meaning. |
| Gateway runtime lifecycle/status surface | [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md), [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs), [crates/world-service/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs) | Reuse the typed runtime ownership split. Do not reintroduce raw wrapper construction as runtime truth. |
| Adapter protocol | [docs/contracts/substrate-gateway-backend-adapter-selection.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-selection.md), [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md) | Reuse exactly. Backend selection remains explicit and stable before execution. |
| Auth carrier | [crates/common/src/gateway_auth_bundle.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/gateway_auth_bundle.rs), [crates/world-service/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/gateway_runtime.rs), [crates/gateway/src/server/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/src/server/mod.rs) | Reuse exactly. No second auth seam. |
| Launch-time world-member transport | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [crates/world-service/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/service.rs) | Reuse the typed transport. Replace only the fulfillment implementation behind it. |
| Resumed world-member transport | [`MemberTurnSubmitRequestV1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs:29), [`/v1/member_turn/stream`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/lib.rs:271) | Reuse exactly. No version bump or selector redesign. |

### Minimum honest change

The minimum honest change is still cross-cutting:

1. remove all production direct wrapper/backend registration from shell host prompt execution,
2. remove all production direct wrapper/backend registration from world-member prompt execution,
3. convert world bootstrap from a fake agent prompt into a control-plane concern rather than a user-visible turn,
4. route both first-turn and resumed-turn execution through one gateway-mediated production seam,
5. prove the seam change with tests that pin lifecycle behavior and auth carrier behavior.

Anything smaller leaves the repo in the current contradictory state: the contracts say gateway-mediated runtime ownership, but the main prompt-bearing runtime still does something else.

### Complexity and distribution check

1. This slice touches multiple runtime modules across `crates/shell`, `crates/world-service`, and gateway-adjacent test surfaces. That breadth is justified because the duplicate fulfillment seam already spans those modules.
2. This is not just a local refactor. Execution semantics, prompt semantics, retained-member semantics, and auth-carrying behavior all have to remain coherent together.
3. The right posture is "narrow in contract, broad in implementation":
   - narrow in contract because no public lifecycle surface widens,
   - broad in implementation because all production bypasses must move together.
4. No large new abstraction stack should appear. The plan allows a small shared fulfillment helper or shared request translator only if it removes the duplicate production seam without inventing a second framework.

### Target fulfillment architecture after cutover

```text
Public shell lifecycle
    substrate agent start|turn|reattach|stop
                    |
                    v
        lifecycle + routing owner (shell)
                    |
       stable backend-id selection + allowlist
                    |
                    v
      gateway-mediated fulfillment boundary
      (typed runtime surface + adapter protocol)
             /                       \
            /                         \
           v                           v
  host prompt-bearing path     world-member prompt-bearing path
  no local wrapper registry    no member-local wrapper registry
           |                           |
           +-----------+   +-----------+
                       |   |
                       v   v
               substrate-gateway adapter dispatch
                       |
                       v
             concrete backend/provider internals
```

The important rule is not "host and world must share identical transport." The important rule is "host and world must share the same production fulfillment boundary." Placement-specific transport is allowed below that seam. Duplicate backend-registration tables above that seam are not.

### Exact remaining gap

The architecture already exists on paper. The remaining gap is operational:

1. shell host prompt submission still treats backend construction as shell-owned runtime truth,
2. world-member prompt submission still treats backend construction as member-runtime truth,
3. launch-time world first prompt and resumed world follow-up do not obviously share one backend-fulfillment seam,
4. world startup still uses a hidden bootstrap prompt in production,
5. docs and contracts already describe the desired boundary, but the main prompt-bearing runtime has not yet been cut over to it.

## Frozen Execution Contract

If implementation wants to violate any rule below, stop and revise the plan first.

1. `substrate agent start` remains host-only in v1.
2. `start` uses the user prompt as the true initial prompt.
3. `turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` remains the exact public follow-up contract.
4. `reattach --session <orchestration_session_id>` remains recovery-only and must not submit a prompt.
5. `stop --session <orchestration_session_id>` remains canonical closeout for attached, parked, and attention-needed durable host sessions.
6. Once a public prompt request emits `Accepted`, the bridge still terminates with an explicit terminal envelope.
7. Durable postures remain authoritative:
   - `parked_resumable` means detached, resumable, no pending inbox work,
   - `awaiting_attention` means detached, resumable, pending inbox work,
   - `terminal` remains the only non-routable posture family.
8. Detached world follow-up remains fail-closed until valid host ownership is restored.
9. Linux world follow-up remains on typed `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream`.
10. Stable backend ids remain selectors only. They do not become auth carriers or provider-quirk carriers.
11. Integrated auth remains on the FD auth-bundle contract. Partial env auth remains invalid; blocked envs do not fall back through a bypass path.
12. Production bootstrap mechanics must not surface as hidden user prompts, synthetic warm-up turns, or replay-visible fake agent prompts.

## Architecture Review

This slice should stay boring but explicit.

1. The single architectural move is: "prompt-bearing fulfillment becomes gateway-mediated everywhere."
2. The shell continues to own:
   - lifecycle verbs,
   - posture transitions,
   - backend selection and allowlist checks,
   - world placement and retained-member ownership,
   - operator UX and trace publication.
3. `substrate-gateway` continues to own:
   - adapter lookup after backend-id selection,
   - backend capability and extension validation,
   - request normalization before backend execution,
   - backend/provider internals and normalized adapter events.
4. `world-service` continues to own typed world transport, retained-member validation, and in-world execution hosting, but not backend registration truth.
5. The allowed hidden divergence is only transport and bootstrap mechanics below the typed runtime surface. The forbidden divergence is separate host and world production backend registries.

### Canonical runtime ownership after cutover

| Surface | Owns | Must stop owning |
| --- | --- | --- |
| `crates/shell/src/execution/agent_runtime/` | lifecycle state, prompt submission orchestration, posture persistence, trace publication | direct backend registration and local wrapper-gateway construction |
| `crates/shell/src/repl/async_repl.rs` | startup planning, world routing, typed member-dispatch request construction | preparing a shell-local authoritative world-member gateway for real execution |
| `crates/world-service/src/member_runtime.rs` | retained-member validation, typed member-turn handling, stream translation, placement checks | direct `AgentWrapperGateway` and concrete backend registration |
| `crates/world-service/src/gateway_runtime.rs` and gateway runtime contracts | runtime launch, auth-bundle handoff, managed runtime wiring | nothing new; these are reuse surfaces |
| `substrate-gateway` adapter protocol | backend resolution, capability validation, request normalization, normalized adapter execution | lifecycle and operator ownership |

### Required implementation shape

Implementation is free to choose names, but it must converge to this functional shape:

1. one production fulfillment entry surface for host prompt-bearing execution,
2. one production fulfillment entry surface for world-member prompt-bearing execution,
3. both entry surfaces hand off into the same gateway-mediated execution seam,
4. any helper added to make that possible must be explicit, placement-aware, and narrow,
5. no placement-specific helper may secretly recreate direct wrapper registration.

A small shared "gateway fulfillment request" translator is acceptable. A new generic execution framework is not required.

### Bootstrap decision

The hidden `runtime_bootstrap_prompt()` path is not allowed to survive as production behavior.

The replacement rule is:

1. member runtime startup may still need a control-plane attach or readiness step,
2. that step must not consume or overwrite the user prompt,
3. that step must not appear as a synthetic agent turn in traces, persisted state, or captured stdin,
4. the first prompt-bearing agent execution must be the real user prompt.

## Code Quality Review

The implementation should be judged against these quality rules:

1. Prefer seam removal over seam layering. Delete duplicate backend-registration tables instead of hiding them behind more indirection.
2. Keep backend-kind mapping explicit and fail-closed, but move the actual backend execution behind the gateway boundary.
3. Preserve typed request and response contracts wherever already landed. Do not widen stable schemas unless there is no internal-only alternative.
4. Any new helper must be named by responsibility, not by generic architecture vocabulary.
5. Update comments near changed control flow so future readers can see why direct backend registration is forbidden there.
6. Tests must prove the user-visible invariants, not just internal call graphs.
7. If a compatibility harness still needs direct wrapper instantiation, keep it in tests only and label it as non-production.

## Canonical Seam Contract

This slice needs one exact mental model for "done."

| Prompt-bearing path | Current state | Required post-slice state |
| --- | --- | --- |
| Host first prompt | shell-local runtime owns wrapper/backend construction | shell owns lifecycle and routing only; fulfillment is gateway-mediated |
| Host follow-up prompt | shell-local runtime rebuilds gateway and backend per turn | same gateway-mediated fulfillment seam as host first prompt |
| World first targeted turn | shell launches world flow, then world-member runtime owns wrapper/backend construction and synthetic bootstrap prompt | first targeted world prompt reaches gateway-mediated fulfillment with no synthetic user prompt |
| World resumed follow-up turn | typed `/v1/member_turn/stream` transport, but member runtime still owns backend construction | same gateway-mediated fulfillment seam as world first targeted turn |
| Gateway auth | existing FD bundle already available | same FD bundle, same fail-closed auth precedence |
| Backend selection | stable backend id already exists | same stable backend id; adapter dispatch happens after selection, not instead of selection |

The required convergence is:

1. one stable backend selection story,
2. one adapter dispatch story,
3. one auth carrier story,
4. two placement-specific transport stories at most,
5. zero production direct backend-registration tables above the gateway seam.

## Implementation Plan

### Phase summary

| Phase | Purpose | Modules or surfaces touched | Hard dependency | Exit gate |
| --- | --- | --- | --- | --- |
| 1 | Freeze lifecycle, seam contract, and grep boundary | `PLAN.md`, truth docs, target tests, runtime comments | — | Frozen contract is written and implementation targets are exact |
| 2 | Cut over host prompt-bearing fulfillment | `crates/shell/src/execution/agent_runtime/`, related tests | 1 | Host first-prompt and follow-up no longer use direct wrapper/backend registration |
| 3 | Cut over world-member prompt-bearing fulfillment | `crates/world-service/src/member_runtime.rs`, `crates/world-service/src/service.rs`, shell world dispatch tests | 1 | World first-turn and resumed follow-up no longer use direct wrapper/backend registration |
| 4 | Remove synthetic bootstrap and discarded shell-local world gateway prep | `crates/world-service/src/member_runtime.rs`, `crates/shell/src/repl/async_repl.rs`, related tests | 2, 3 | No production synthetic bootstrap prompt; no discarded authoritative local member gateway prep |
| 5 | Truth-doc and contract sync | SOW, gap matrix, gateway contracts, usage docs | 2, 3, 4 | Live docs describe gateway-mediated fulfillment as the steady-state runtime |
| 6 | Validation wall and closeout | tests, grep gates, auth/runtime regression checks | 2, 3, 4, 5 | Full runtime behavior and seam movement are proven together |

### Phase 1: Freeze the lifecycle and fulfillment contract

Primary surfaces:

1. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
2. [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
3. [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
4. [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md)
5. [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md)
6. [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
7. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

Required actions:

1. Freeze the exact list of runtime files allowed to change for this slice.
2. Freeze the behavioral invariants that later code work is not allowed to reopen.
3. Freeze the grep wall for forbidden production symbols:
   - `AgentWrapperGateway`,
   - `CodexBackend`,
   - `ClaudeCodeBackend`,
   - `runtime_bootstrap_prompt`.
4. Freeze the exact test files that will carry the regression burden.

Done when:

1. every later phase can point back to one explicit contract,
2. implementation does not have to guess whether a behavior change is allowed,
3. the team can distinguish acceptable helper additions from forbidden seam drift.

### Phase 2: Replace the host prompt-bearing fulfillment seam

Primary surfaces:

1. [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
2. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
3. [crates/shell/src/execution/agent_runtime/mapping.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs)
4. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
5. [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
6. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

Required actions:

1. Remove production use of `build_gateway_for_descriptor()` as the host fulfillment path.
2. Route host first prompt and host follow-up prompt through a gateway-mediated executor that consumes:
   - already-selected backend id,
   - current prompt,
   - existing resume/session-handle metadata,
   - existing lifecycle persistence and event translation.
3. Keep host execution host-scoped in public meaning even if backend execution is now gateway-mediated under the hood.
4. Preserve current completion, posture, and trace publication semantics.
5. Keep failure buckets coherent:
   - invalid selection stays invalid selection,
   - dependency unavailable stays dependency unavailable,
   - policy denial stays policy denial.

Done when:

1. no host prompt-bearing production path directly registers concrete backends,
2. host `start` and host `turn` still behave the same from the CLI surface,
3. session resume metadata still threads correctly through the new seam,
4. no hidden warm-up prompt appears in stdin, traces, or runtime artifacts.

### Phase 3: Replace the world-member fulfillment seam for both first-turn and resumed-turn execution

Primary surfaces:

1. [crates/world-service/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs)
2. [crates/world-service/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/service.rs)
3. [crates/world-service/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/lib.rs)
4. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
5. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
6. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
7. [crates/world-service/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/tests/streamed_execute_cancel_v1.rs)
8. [crates/world-service/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/tests/member_runtime_world_placement_v1.rs)

Required actions:

1. Remove `build_gateway_for_backend()` as the production world-member fulfillment mechanism.
2. Make launch-time first targeted world execution and resumed follow-up both traverse the same gateway-mediated execution path inside `world-service`.
3. Preserve the typed transport boundary:
   - launch-time world first prompt still enters through `member_dispatch.initial_prompt`,
   - resumed world follow-up still enters through `MemberTurnSubmitRequestV1`,
   - fulfillment behind both inputs becomes the same gateway-mediated seam.
4. Preserve retained-member identity validation, world binding checks, participant/backend/world tuple validation, and detached-world fail-closed behavior.
5. Keep member-stream event translation and completion framing stable from the shell’s perspective.

Done when:

1. world-member production execution no longer locally constructs wrappers/backends,
2. first-turn and resumed-turn world execution are visibly on one fulfillment seam,
3. typed transport contracts remain unchanged,
4. retained-member invariants still hold.

### Phase 4: Remove synthetic bootstrap behavior and misleading shell-local member gateway prep

Primary surfaces:

1. [crates/world-service/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs)
2. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
3. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
4. [crates/shell/tests/support/repl_world_service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_service.rs)

Required actions:

1. Delete or demote `runtime_bootstrap_prompt()` so it is not part of production prompt semantics.
2. Ensure the first targeted world turn carries the real user prompt all the way to fulfillment.
3. Remove any shell-local gateway object or equivalent authoritative member-runtime prep that is created only to be discarded before real world execution.
4. Keep any necessary readiness or attach step as control-plane state, not as a fake prompt-bearing run.

Done when:

1. traces and captured transport payloads show only the real user prompt as the first prompt,
2. there is no production bootstrap prompt constant left on the execution path,
3. there is no production shell-local authoritative member gateway prep pattern left behind.

### Phase 5: Sync truth docs to the new steady state

Primary surfaces:

1. [llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md)
2. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
3. [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md)
4. [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md)
5. [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)

Required actions:

1. Update live docs so they describe direct wrapper/backend registration as historical bypass behavior, not steady-state architecture.
2. Update the gap matrix so this seam is no longer described as outstanding once code lands.
3. Keep ADR-0040, ADR-0041, and ADR-0047 stable in ownership and lifecycle meaning; only sync descriptive wording if implementation evidence now exists.

Done when:

1. live docs tell the same runtime story as the code,
2. no truth doc implies shell-local or member-local backend registration is acceptable steady-state production behavior.

### Phase 6: Validation and closeout

Required actions:

1. Run the static grep wall on production runtime code.
2. Run focused runtime tests for shell host control and world-member routing.
3. Run gateway/auth regression tests that prove the FD auth-bundle path still holds.
4. Run full workspace tests after focused coverage is green.
5. Produce one compact validation artifact for reviewers naming:
   - lifecycle invariants checked,
   - world first-turn versus follow-up checks,
   - auth-bundle checks,
   - seam-removal grep checks.

Done when:

1. seam movement and lifecycle stability are both proven,
2. auth, selection, and retained-member invariants still hold,
3. the repo no longer contains the production bypass in the targeted runtime surfaces.

## Test Review

Runtime tests, seam-removal invariants, and auth-carrier tests are the authoritative test layers for this slice.

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] Host prompt-bearing fulfillment
    crates/shell/src/execution/agent_runtime/control.rs
    crates/shell/src/execution/agent_runtime/registry.rs
    crates/shell/tests/agent_public_control_surface_v1.rs
    crates/shell/tests/agent_successor_contract_ahcsitc0.rs
    Required proof:
        - start uses real user prompt
        - turn uses real user prompt
        - no direct backend registration remains in production path
        - Accepted -> terminal still holds

[+] World first-turn fulfillment
    crates/shell/src/repl/async_repl.rs
    crates/shell/src/execution/routing/dispatch/world_ops.rs
    crates/world-service/src/member_runtime.rs
    crates/shell/tests/repl_world_first_routing_v1.rs
    Required proof:
        - launch-time initial_prompt carries the real user prompt
        - no synthetic bootstrap prompt remains
        - no local member-runtime backend registry remains

[+] World resumed follow-up fulfillment
    crates/world-service/src/member_runtime.rs
    crates/world-service/src/lib.rs
    crates/shell/tests/repl_world_first_routing_v1.rs
    crates/world-service/tests/streamed_execute_cancel_v1.rs
    Required proof:
        - resumed follow-up still uses /v1/member_turn/stream
        - retained-member tuple validation still holds
        - same fulfillment seam as launch-time first turn

[+] Gateway/auth/adapter continuity
    crates/world-service/src/gateway_runtime.rs
    crates/gateway/src/server/mod.rs
    crates/shell/tests/world_gateway.rs
    crates/gateway/tests/openai_shared_parity.rs
    Required proof:
        - FD auth bundle still used
        - bundle is consumed once
        - no child secret env fallback path appears
        - backend selection remains stable and allowlist-driven
```

### Concrete test additions or tightening

| Target | Test requirement | Type |
| --- | --- | --- |
| [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | Add or tighten assertions that `start`, `turn`, `reattach`, `stop`, posture transitions, and `Accepted -> terminal` remain unchanged after the seam move. | focused integration |
| [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) | Prove no hidden bootstrap or synthetic prompt appears in host prompt-bearing execution artifacts. | focused integration |
| [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Tighten first-targeted-world-turn assertions so the real user prompt is the only first prompt-bearing input, and launch-time first turn plus resumed follow-up prove the same seam. | focused integration |
| [`crates/world-service/tests/streamed_execute_cancel_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/tests/streamed_execute_cancel_v1.rs) | Preserve cancel and completion behavior while removing member-local backend registration. | focused integration |
| [`crates/world-service/tests/member_runtime_world_placement_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/tests/member_runtime_world_placement_v1.rs) | Preserve retained-member tuple and world placement validation while the fulfillment seam moves. | focused integration |
| [`crates/shell/tests/world_gateway.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs) and [`crates/gateway/tests/openai_shared_parity.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/tests/openai_shared_parity.rs) | Prove the FD auth-bundle path, one-time consumption, and fail-closed auth precedence remain intact. | focused integration |
| static repo invariant | Add one grep-backed invariant or test-time assertion that the targeted production runtime surfaces no longer instantiate `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend`. | static validation |
| static repo invariant | Add one grep-backed invariant or test-time assertion that `runtime_bootstrap_prompt` or equivalent synthetic prompt-bearing bootstrap behavior is gone from production runtime code. | static validation |

### QA handoff artifact

Implementation is not done when the tests compile. The PR or implementation notes must also leave one compact handoff artifact that names:

1. the exact lifecycle commands exercised,
2. the exact world first-turn and resumed-follow-up scenarios exercised,
3. the auth-bundle checks performed,
4. the static seam-removal checks performed,
5. and any remaining test-only direct wrapper uses that are intentionally non-production.

### Regression rule for this slice

These are mandatory blockers:

1. any path that causes `start` or `turn` to send a hidden synthetic prompt before the real user prompt,
2. any path that breaks `Accepted -> terminal`,
3. any path that widens or changes the typed world follow-up contract,
4. any path that restores direct backend registration in the targeted production runtime files,
5. any path that reopens auth delivery through secret-bearing env vars.

## Failure Modes Registry

| Failure mode | Where it happens | Required handling | Test requirement | Critical gap if missing |
| --- | --- | --- | --- | --- |
| Host fulfillment still routes through shell-local wrapper construction after refactor | `crates/shell/src/execution/agent_runtime/` | fail review; remove remaining bypass | static seam check plus host integration tests | Yes |
| World first-turn and resumed-turn drift onto different fulfillment seams | `crates/world-service/src/member_runtime.rs`, world dispatch tests | refactor until both paths converge | world first-turn plus resumed-turn regression tests | Yes |
| Synthetic bootstrap prompt survives as hidden runtime behavior | `member_runtime.rs`, REPL startup flow | delete or demote to non-prompt control-plane step | prompt-capture regression tests | Yes |
| Detached world follow-up reopens through a bypass path | member-turn validation | fail closed with current error posture | retained-member and detached-world tests | Yes |
| Stable backend-id selection is bypassed by local backend mapping logic | shell or world helper code | fail closed and restore selection-before-dispatch | focused host/world integration plus static inspection | Yes |
| FD auth-bundle handoff regresses to child env or duplicate secret path | gateway runtime or server startup | fail closed | gateway parity and auth tests | Yes |
| Docs still describe direct wrapper construction as acceptable steady state | SOW, gap matrix, gateway contracts, usage docs | update in same slice | doc review | No, but unacceptable at ship |

Any failure mode with silent prompt-semantic drift or silent auth drift is a release blocker for this slice.

## Performance And Complexity Review

1. This slice should not add new hot-path layers beyond the minimum gateway-mediated handoff needed to remove duplicate backend registration.
2. Do not add per-turn discovery work that can be derived from already-selected backend ids and existing runtime wiring.
3. The dominant cost center here is behavioral complexity, not CPU. Optimize for one obvious runtime story.
4. If a helper can only exist by smuggling placement-specific behavior into generic terminology, do not add it.

## Deferred Follow-Ups / TODO Candidates

These are real follow-ups, but they are not part of this slice:

1. Shared dispatch-envelope and capability-override work from SOW 29.
2. Public world-scoped root start and capability flags from SOW 30.
3. Any generic integrated backend matrix expansion beyond the already-supported bindings.
4. Any future cleanup that further normalizes gateway runtime wiring once this seam replacement is complete.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A0. Freeze lifecycle and seam contract | `PLAN.md`, truth docs, target tests | — |
| A1. Host fulfillment cutover | `crates/shell/src/execution/agent_runtime/`, host control tests | A0 |
| A2. World-member fulfillment cutover | `crates/world-service/src/member_runtime.rs`, `crates/world-service/src/service.rs`, world routing tests | A0 |
| A3. Remove bootstrap and discarded shell-local member gateway prep | `crates/world-service/src/member_runtime.rs`, `crates/shell/src/repl/async_repl.rs`, world routing tests | A1, A2 |
| B. Truth-doc sync | SOW, gap matrix, gateway contracts, usage docs | A3 |
| C. Validation wall and closeout | repo root, targeted tests, auth tests | A1, A2, A3, B |

### Parallel lanes

Lane A: A0 -> A1 and A2 -> A3  
Reason: after the contract is frozen, host and world seam cutovers are separate enough to proceed in parallel for a short window, but they must reconverge before bootstrap removal and final cleanup because they share lifecycle invariants and some REPL-facing tests.

Lane B: B  
Reason: doc sync should start only after A3 settles the final runtime story. Starting docs earlier creates churn because the seam details are still moving.

Lane C: C  
Reason: validation is the merge gate and must run after code and docs converge.

### Safe parallel split

The only honest parallel split in this slice is:

1. one owner or worktree handles host fulfillment cutover,
2. one owner or worktree handles world-member fulfillment cutover,
3. both owners reconverge before bootstrap cleanup and final test stabilization.

This works because:

1. host seam work is concentrated in `crates/shell/src/execution/agent_runtime/`,
2. world seam work is concentrated in `crates/world-service/src/member_runtime.rs` and world routing tests,
3. the conflict zone is `crates/shell/src/repl/async_repl.rs` plus shared lifecycle assertions, which is why A3 is serialized after A1 and A2.

### Conflict flags

1. Do not split A3 across worktrees. It touches the exact boundary where host/world behavior becomes one user-visible prompt story.
2. Do not start doc sync before A3. The plan specifically depends on the final bootstrap decision and the exact post-cutover seam wording.
3. If A1 introduces new helper shapes used by A2, rebase A2 onto A1 before final integration rather than cloning the helper differently in both lanes.

### Parallelization verdict

This slice has:

1. one required contract-freeze step,
2. two short parallel execution lanes for host and world seam cutover,
3. one serialized reconvergence step for bootstrap removal and REPL cleanup,
4. one doc lane,
5. one final validation lane.

Peak honest parallelism is `A1 + A2`. Everything after that should be sequenced.

## Validation Commands

### Static seam-removal gates

These commands are expected to be green in the targeted production runtime surfaces after implementation:

```bash
rg -n "AgentWrapperGateway|CodexBackend|ClaudeCodeBackend" \
  crates/shell/src/execution/agent_runtime \
  crates/shell/src/repl \
  crates/world-service/src/member_runtime.rs
```

Expected result after the slice:

1. no production hits in the targeted runtime files,
2. any remaining hits must be in tests or explicitly documented non-production harnesses.

Synthetic bootstrap prompt gate:

```bash
rg -n "runtime_bootstrap_prompt|Enter persistent Substrate world-scoped member mode" \
  crates/world-service/src/member_runtime.rs \
  crates/shell/src/repl \
  crates/shell/tests \
  crates/world-service/tests
```

Expected result after the slice:

1. no production hits,
2. test fixtures may refer to the old behavior only if the test is explicitly historical or transitional.

### Focused cargo gates

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
cargo test -p world-service --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-service --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p substrate-gateway --test openai_shared_parity -- --nocapture
```

### Full workspace gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

### Manual validation proof points

Manual validation for this slice must explicitly confirm:

1. host `start` uses the real user prompt as the first prompt,
2. host `turn` uses the real user prompt as the follow-up prompt,
3. world first targeted turn uses the real user prompt and not a bootstrap prompt,
4. resumed world follow-up still travels through `/v1/member_turn/stream`,
5. detached world follow-up still fails closed,
6. gateway startup still consumes the FD auth bundle,
7. runtime artifacts or traces provide evidence that fulfillment is gateway-mediated rather than shell-local or member-local wrapper construction.

## Completion Summary

This plan is implementation-ready because it now freezes the entire execution story:

1. Objective: gateway-mediated fulfillment replaces the production bypass without changing lifecycle meaning.
2. Locked decisions: public lifecycle, backend-id selection, typed world-member transport, and FD auth carrier remain fixed.
3. Current repo truth: the exact bypass points and bootstrap regression points are named.
4. Architecture review: ownership is explicit and the target seam is singular.
5. Code quality review: no duplicate backend-registration seam, no hidden bootstrap behavior, no broad new framework.
6. Test review: the exact runtime paths and regression obligations are defined.
7. Failure modes: the dangerous regressions are named and treated as blockers.
8. Parallelization: one freeze step, one honest host/world parallel window, one serialized reconvergence step, then docs and validation.
9. Validation: grep, focused tests, auth checks, and full workspace gates are all specified.

After this slice lands, the runtime story should read as if it were intentional from the start:

1. shell owns lifecycle and routing,
2. typed runtime surfaces own transport and world attachment,
3. stable backend ids are selected before execution,
4. `substrate-gateway` owns adapter dispatch and backend internals,
5. no production prompt-bearing path bypasses that seam anymore.
