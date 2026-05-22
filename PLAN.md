# PLAN: Gateway-Mediated LLM Fulfillment Without Lifecycle Regression

Source SOW: [28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md)  
Primary contract anchors: [ADR-0047](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [ADR-0040](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md), [ADR-0041](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [docs/contracts/substrate-gateway-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-runtime-parity.md), [docs/contracts/substrate-gateway-backend-adapter-protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/substrate-gateway-backend-adapter-protocol.md)  
Execution branch: `feat/gateway-mediated-llm-fulfillment`  
Base branch: `main`  
Plan type: runtime seam replacement with lifecycle freeze  
Status: implementation-ready execution plan, unified to `/autoplan` plus `/plan-eng-review` rigor on 2026-05-22

## Objective

Move all production prompt-bearing LLM fulfillment onto the existing `substrate-gateway` adapter seam without changing the already-landed public lifecycle contract.

After this slice lands:

1. host prompt-bearing execution no longer directly constructs `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend` in shell-owned runtime code,
2. world-member prompt-bearing execution no longer directly constructs those objects in `world-service`,
3. host first prompt, host follow-up prompt, world first targeted prompt, and world resumed follow-up all use one production fulfillment story:
   - stable backend id selection first,
   - gateway adapter dispatch second,
   - lifecycle semantics unchanged,
4. integrated auth still uses the existing FD auth-bundle handoff,
5. `start`, `turn`, `reattach`, `stop`, `Accepted -> terminal`, `parked_resumable`, `awaiting_attention`, and typed `/v1/member_turn/stream` behavior remain unchanged from the operator's point of view.

This is a seam replacement. It is not a public contract expansion, not a lifecycle redesign, and not a backend-matrix expansion project.

## Acceptance Criteria

This slice is complete only when all of the following are true:

1. No production code under `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/`, or `crates/world-service/src/member_runtime.rs` directly instantiates `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend` for prompt-bearing execution.
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

## Scope

### In scope

1. Replacing direct host prompt-bearing fulfillment with gateway-mediated fulfillment.
2. Replacing direct world-member prompt-bearing fulfillment with gateway-mediated fulfillment.
3. Removing hidden synthetic bootstrap-prompt behavior from production world-member startup.
4. Collapsing duplicated backend-registration logic into one production fulfillment seam.
5. Reusing the landed gateway runtime, adapter, backend-selection, and auth-bundle contracts.
6. Tightening regression coverage so lifecycle invariants stay frozen while the seam moves.
7. Updating truth docs that would otherwise keep describing the bypass as acceptable steady-state behavior.

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

## Step 0: Scope Challenge

This section answers the `/plan-eng-review` questions before implementation starts.

### What already exists

| Sub-problem | Existing code or contract | Reuse decision |
| --- | --- | --- |
| Lifecycle truth | `ADR-0047`, `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`, `crates/shell/tests/agent_public_control_surface_v1.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs` | Reuse exactly. This slice must not reinterpret lifecycle meaning. |
| Gateway runtime lifecycle and status surface | `docs/contracts/substrate-gateway-runtime-parity.md`, `crates/shell/src/builtins/world_gateway.rs`, `crates/world-service/src/gateway_runtime.rs` | Reuse the typed runtime ownership split. Do not reintroduce raw wrapper construction as runtime truth. |
| Adapter protocol and backend selection | `docs/contracts/substrate-gateway-backend-adapter-selection.md`, `docs/contracts/substrate-gateway-backend-adapter-protocol.md` | Reuse exactly. Backend selection remains explicit and stable before execution. |
| Auth carrier | `crates/common/src/gateway_auth_bundle.rs`, `crates/world-service/src/gateway_runtime.rs`, `crates/gateway/src/server/mod.rs` | Reuse exactly. No second auth seam. |
| Launch-time world-member transport | `crates/shell/src/execution/routing/dispatch/world_ops.rs`, `crates/world-service/src/service.rs` | Reuse the typed transport. Replace only the fulfillment implementation behind it. |
| Resumed world-member transport | `MemberTurnSubmitRequestV1`, `/v1/member_turn/stream` | Reuse exactly. No version bump or selector redesign. |

### Minimum honest change

The minimum honest change is still cross-cutting:

1. remove all production direct wrapper/backend registration from shell host prompt execution,
2. remove all production direct wrapper/backend registration from world-member prompt execution,
3. convert world bootstrap from a fake agent prompt into a control-plane concern rather than a user-visible turn,
4. route both first-turn and resumed-turn execution through one gateway-mediated production seam,
5. prove the seam change with tests that pin lifecycle behavior and auth carrier behavior.

Anything smaller leaves the repo in the current contradictory state: the contracts say gateway-mediated runtime ownership, but the main prompt-bearing runtime still does something else.

### Complexity check

This slice touches more than 8 files and crosses shell plus world runtime boundaries. That is acceptable because the duplication already spans those modules.

The constraint is not "touch fewer files at any cost." The constraint is:

1. no new public surface,
2. no new runtime framework,
3. no new schema version,
4. no new policy/config layer,
5. no second fulfillment seam hidden behind nicer naming.

The correct posture is narrow in contract, broad in implementation:

1. narrow in contract because the lifecycle and selectors stay frozen,
2. broad in implementation because all production bypasses must move together.

### Search check

This plan intentionally prefers existing repo seams over new architecture. It reuses:

1. the existing gateway runtime,
2. the existing adapter contract,
3. the existing backend-id selection contract,
4. the existing auth-bundle carrier,
5. the existing typed member-turn transport.

That is a Layer 1 choice, not a custom-framework choice.

### Expected blast radius

Treat this as the planned change budget. If implementation spills outside these surfaces, stop and justify the expansion before continuing.

| Surface class | Expected modules or files | Why this is the right boundary |
| --- | --- | --- |
| Production host runtime | `crates/shell/src/execution/agent_runtime/`, narrowly-targeted paths in `crates/shell/src/repl/async_repl.rs` | This is where host prompt submission and shell-owned bootstrap prep still treat local gateway construction as runtime truth. |
| Production world runtime | `crates/world-service/src/member_runtime.rs`, only minimum plumbing in `crates/world-service/src/service.rs` if needed | This is where launch-time and resumed world-member turns still directly build wrappers and backends. |
| Transport and lifecycle tests | `crates/shell/tests/agent_public_control_surface_v1.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`, `crates/world-service/tests/*`, `crates/shell/tests/world_gateway.rs`, `crates/gateway/tests/openai_shared_parity.rs` | These are the proof surfaces for lifecycle freeze, seam convergence, and auth continuity. |
| Truth docs | `PLAN.md`, `llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md`, gateway contracts, usage docs, gap matrix | The runtime story changes here after code changes, not before. |

### Explicit stop conditions

If implementation requires any of the following, stop and revise the plan first:

1. a new CLI flag or public verb,
2. a new JSON schema or route for world follow-up,
3. a new policy or config surface,
4. a new crate just to move the seam,
5. a synthetic prompt-bearing bootstrap step,
6. a fallback auth path through child secret env vars.

## Current Repo Truth

These facts are concrete in the repo today. This plan is anchored to them.

1. The shell host runtime still builds a local `AgentWrapperGateway` and directly registers concrete backends in `crates/shell/src/execution/agent_runtime/registry.rs`.
2. Host follow-up prompt submission still rebuilds that direct gateway in `submit_host_prompt_turn()` inside `crates/shell/src/execution/agent_runtime/control.rs`.
3. `crates/shell/src/repl/async_repl.rs` still carries gateway-shaped local runtime state and bootstrap helpers that treat local wrapper construction as an execution concern.
4. The world member runtime still builds another direct `AgentWrapperGateway` in `crates/world-service/src/member_runtime.rs`.
5. The world member runtime still contains a synthetic runtime bootstrap prompt via `runtime_bootstrap_prompt()`. That is incompatible with the frozen "the real user prompt is the first prompt" rule.
6. The typed world-member follow-up seam is already real and should be preserved:
   - request type: `MemberTurnSubmitRequestV1`,
   - route: `/v1/member_turn/stream`.
7. The gateway runtime and auth-bundle seams are already real and already owned:
   - auth bundle contract: `crates/common/src/gateway_auth_bundle.rs`,
   - world-service gateway runtime handoff: `crates/world-service/src/gateway_runtime.rs`,
   - gateway server consumption of the FD bundle: `crates/gateway/src/server/mod.rs`.
8. Current shell world dispatch already packages typed launch-time world-member requests with `member_dispatch.initial_prompt`. The transport seam is not what needs redesign.
9. The current contradiction is operational, not contractual:
   - docs and ADRs already say gateway-mediated runtime ownership,
   - production prompt-bearing execution still bypasses that seam.

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

## Target Architecture

### Canonical runtime story after cutover

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

### Runtime ownership after cutover

| Surface | Owns | Must stop owning |
| --- | --- | --- |
| `crates/shell/src/execution/agent_runtime/` | lifecycle state, prompt submission orchestration, posture persistence, trace publication | direct backend registration and local wrapper-gateway construction |
| `crates/shell/src/repl/async_repl.rs` | startup planning, world routing, typed member-dispatch request construction | preparing a shell-local authoritative world-member gateway for real execution |
| `crates/world-service/src/member_runtime.rs` | retained-member validation, typed member-turn handling, stream translation, placement checks | direct `AgentWrapperGateway` and concrete backend registration |
| `crates/world-service/src/gateway_runtime.rs` and gateway runtime contracts | runtime launch, auth-bundle handoff, managed runtime wiring | nothing new; these are reuse surfaces |
| `substrate-gateway` adapter protocol | backend resolution, capability validation, request normalization, normalized adapter execution | lifecycle and operator ownership |

### Canonical seam contract

| Prompt-bearing path | Current state | Required post-slice state |
| --- | --- | --- |
| Host first prompt | shell-local runtime owns wrapper and backend construction | shell owns lifecycle and routing only; fulfillment is gateway-mediated |
| Host follow-up prompt | shell-local runtime rebuilds gateway and backend per turn | same gateway-mediated fulfillment seam as host first prompt |
| World first targeted turn | shell launches world flow, then world-member runtime owns wrapper and backend construction and synthetic bootstrap prompt | first targeted world prompt reaches gateway-mediated fulfillment with no synthetic user prompt |
| World resumed follow-up turn | typed `/v1/member_turn/stream` transport, but member runtime still owns backend construction | same gateway-mediated fulfillment seam as world first targeted turn |
| Gateway auth | existing FD bundle already available | same FD bundle, same fail-closed auth precedence |
| Backend selection | stable backend id already exists | same stable backend id; adapter dispatch happens after selection, not instead of selection |

The required convergence is:

1. one stable backend selection story,
2. one adapter dispatch story,
3. one auth carrier story,
4. two placement-specific transport stories at most,
5. zero production direct backend-registration tables above the gateway seam.

### Helper constraints

Implementation is free to choose names, but it is not free to reopen the seam.

If a new helper is needed, it may only do one of these jobs:

1. translate already-selected runtime metadata into a gateway-mediated request,
2. normalize host and world resume metadata into one explicit input shape,
3. hide placement-specific transport details below the shared fulfillment boundary.

A new helper is not allowed to:

1. construct `AgentWrapperGateway`,
2. register concrete backends,
3. synthesize or rewrite the user prompt,
4. invent a second backend-selection path,
5. silently turn bootstrap work into a fake prompt-bearing run.

### Bootstrap rule

The hidden `runtime_bootstrap_prompt()` path is not allowed to survive as production behavior.

The replacement rule is:

1. member runtime startup may still need a control-plane attach or readiness step,
2. that step must not consume or overwrite the user prompt,
3. that step must not appear as a synthetic agent turn in traces, persisted state, or captured stdin,
4. the first prompt-bearing agent execution must be the real user prompt.

## Implementation Plan

### Execution order at a glance

This slice should execute in exactly this order:

1. freeze the contract and the grep wall,
2. cut over host prompt-bearing fulfillment,
3. cut over world-member prompt-bearing fulfillment,
4. reconverge in the shared conflict zone to remove bootstrap semantics and discarded shell-local gateway prep,
5. sync truth docs only after the runtime story is stable,
6. run the validation wall and publish one reviewer-facing proof artifact.

The sequencing rule is strict: host and world cutovers may proceed in parallel for a short window, but `async_repl.rs`, bootstrap semantics, and shared lifecycle assertions are a serialized reconvergence step, not cleanup.

### Workstream summary

| Workstream | Goal | Primary surfaces | Depends on | Exit gate |
| --- | --- | --- | --- | --- |
| 1 | Freeze lifecycle, seam contract, grep wall, and change budget | `PLAN.md`, truth docs, target tests | - | Later code work has one fixed contract and one fixed blast radius |
| 2 | Cut over host prompt-bearing fulfillment | `crates/shell/src/execution/agent_runtime/`, host bootstrap prep in `crates/shell/src/repl/async_repl.rs`, host tests | 1 | Host first prompt and follow-up no longer treat local gateway/backend construction as execution truth |
| 3 | Cut over world-member fulfillment | `crates/world-service/src/member_runtime.rs`, minimum required `crates/world-service/src/service.rs`, world routing tests | 1 | Launch-time first turn and resumed follow-up use one world-side fulfillment seam without changing typed transport |
| 4 | Reconverge shared bootstrap and startup semantics | `crates/shell/src/repl/async_repl.rs`, `crates/world-service/src/member_runtime.rs`, shared routing tests | 2, 3 | No production bootstrap prompt remains, and no discarded shell-local authoritative gateway prep remains |
| 5 | Sync truth docs and usage text | SOW, gap matrix, gateway contracts, usage docs | 4 | Live docs match the actual runtime story |
| 6 | Validation wall and closeout | static grep gates, focused tests, auth/runtime regression checks, full workspace gates | 2, 3, 4, 5 | Seam movement and lifecycle stability are proven together |

### Workstream 1: Freeze the lifecycle and fulfillment contract

Primary surfaces:

1. `PLAN.md`
2. `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`
3. `ADR-0047`
4. `docs/contracts/substrate-gateway-runtime-parity.md`
5. `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
6. `crates/shell/tests/agent_public_control_surface_v1.rs`
7. `crates/shell/tests/repl_world_first_routing_v1.rs`

Required changes:

1. Freeze the exact production runtime files allowed to change in this slice.
2. Freeze the behavioral invariants that later code work is not allowed to reopen.
3. Freeze the grep wall for forbidden production symbols:
   - `AgentWrapperGateway`,
   - `CodexBackend`,
   - `ClaudeCodeBackend`,
   - `runtime_bootstrap_prompt`.
4. Freeze the exact test files that carry the regression burden.

Must remain true:

1. no public lifecycle or selector change,
2. no new route or schema version,
3. no new auth carrier,
4. no widening of "what this slice is about."

Exit gate:

1. every later workstream can point back to one explicit contract,
2. implementation does not have to guess whether a behavior change is allowed,
3. reviewers can distinguish acceptable helper additions from forbidden seam drift.

### Workstream 2: Cut over host prompt-bearing fulfillment

Primary surfaces:

1. `crates/shell/src/execution/agent_runtime/registry.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/agent_runtime/mapping.rs`
4. `crates/shell/src/execution/agent_runtime/validator.rs`
5. `crates/shell/src/repl/async_repl.rs`
6. `crates/shell/tests/agent_public_control_surface_v1.rs`
7. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Required changes:

1. Replace `submit_host_prompt_turn()` as a direct wrapper-construction path. Today it builds a local gateway via `build_gateway_for_descriptor()`. After this slice it should invoke the gateway-mediated seam using the already-selected backend id plus existing resume metadata.
2. Remove shell-local `gateway` and `agent_kind` from any host bootstrap state that still treats them as authoritative execution truth. `ResolvedHostOrchestratorBootstrap` and `PreparedAgentRuntime` may keep metadata needed for routing, but they must stop carrying a production local backend registry just to submit prompt-bearing work.
3. Preserve host public meaning exactly:
   - `start` remains the real first user prompt,
   - `turn` remains the real follow-up prompt,
   - session resume state still threads through the execution path,
   - completion, posture, and trace publication semantics stay unchanged.
4. Keep failure buckets coherent:
   - invalid selection stays invalid selection,
   - dependency unavailable stays dependency unavailable,
   - policy denial stays policy denial.

Must remain true:

1. no hidden warm-up prompt appears in stdin, traces, or runtime artifacts,
2. no host-only direct backend-registration table survives above the gateway seam,
3. host execution is still host-scoped in public meaning even if fulfillment is now gateway-mediated.

Exit gate:

1. no host prompt-bearing production path directly registers concrete backends,
2. host `start` and host `turn` still behave the same from the CLI surface,
3. session resume metadata still threads correctly through the new seam.

### Workstream 3: Cut over world-member fulfillment for first-turn and resumed-turn execution

Primary surfaces:

1. `crates/world-service/src/member_runtime.rs`
2. `crates/world-service/src/service.rs`
3. `crates/world-service/src/lib.rs`
4. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
5. `crates/shell/src/repl/async_repl.rs`
6. `crates/shell/tests/repl_world_first_routing_v1.rs`
7. `crates/world-service/tests/streamed_execute_cancel_v1.rs`
8. `crates/world-service/tests/member_runtime_world_placement_v1.rs`

Required changes:

1. Replace both direct world execution call sites in `MemberRuntimeManager`:
   - `launch()` currently builds a gateway with `build_gateway_for_backend()` and falls back to `runtime_bootstrap_prompt()` when `dispatch.initial_prompt` is absent,
   - `submit_turn()` currently builds another direct gateway with `build_gateway_for_backend()` for resumed follow-up.
2. Make both call sites traverse one world-side gateway-mediated execution seam. The important outcome is not "same function name," it is "one execution boundary, one auth/runtime story, no duplicate backend registry."
3. Preserve the typed transport boundary exactly:
   - launch-time world first prompt still enters through `member_dispatch.initial_prompt`,
   - resumed world follow-up still enters through `MemberTurnSubmitRequestV1`,
   - `POST /v1/member_turn/stream` stays unchanged,
   - `service.rs` changes only if needed to preserve the existing typed request and response translation.
4. Preserve retained-member identity validation, world binding checks, participant/backend/world tuple validation, and detached-world fail-closed behavior.
5. Keep member-stream event translation and completion framing stable from the shell's perspective.

Must remain true:

1. first-turn and resumed-turn world execution are visibly on one fulfillment seam,
2. no synthetic bootstrap prompt survives,
3. typed transport contracts remain unchanged,
4. retained-member invariants still hold.

Exit gate:

1. world-member production execution no longer locally constructs wrappers or backends,
2. first-turn and resumed-turn world execution are visibly on one fulfillment seam,
3. retained-member validation and fail-closed behavior still hold.

### Workstream 4: Reconverge bootstrap semantics and shell/world startup state

This is the shared conflict zone. Treat it as a required merge step, not follow-up cleanup.

Primary surfaces:

1. `crates/world-service/src/member_runtime.rs`
2. `crates/shell/src/repl/async_repl.rs`
3. `crates/shell/tests/repl_world_first_routing_v1.rs`
4. `crates/shell/tests/support/repl_world_service.rs`

Required changes:

1. Delete or demote `runtime_bootstrap_prompt()` in both host and world startup logic so it is no longer part of production prompt semantics.
2. Ensure the first targeted world turn carries the real user prompt all the way to fulfillment. If any attach or readiness step remains, it must live as control-plane state, not as a fake prompt-bearing run.
3. Remove shell-local gateway prep in `async_repl.rs` that is created only to be discarded before real remote member execution. If a prepared runtime object still exists, it should carry routing metadata only, not a local authoritative gateway registry.
4. Update shared lifecycle assertions and prompt-capture tests in the same workstream so the bootstrap removal and seam cutover settle together.

Must remain true:

1. traces and captured transport payloads show only the real user prompt as the first prompt,
2. there is no production bootstrap prompt constant left on the execution path,
3. there is no production shell-local authoritative member gateway prep pattern left behind.

Exit gate:

1. the first prompt-bearing execution is always the real user prompt,
2. `async_repl.rs` no longer owns a discarded execution-time gateway for world members,
3. shared routing assertions prove the post-merge story end to end.

### Workstream 5: Sync truth docs to the new steady state

Primary surfaces:

1. `llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md`
2. `AGENT_ORCHESTRATION_GAP_MATRIX.md`
3. `docs/contracts/substrate-gateway-runtime-parity.md`
4. `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
5. `docs/USAGE.md`

Required changes:

1. Update live docs so they describe direct wrapper and backend registration as historical bypass behavior, not steady-state architecture.
2. Update the gap matrix so this seam is no longer described as outstanding once code lands.
3. Keep ADR-0040, ADR-0041, and ADR-0047 stable in ownership and lifecycle meaning. Only sync descriptive wording where implementation evidence now exists.

Must remain true:

1. docs lag implementation by at most one PR,
2. ADR meaning stays stable,
3. no doc implies shell-local or member-local backend registration is acceptable steady-state production behavior.

Exit gate:

1. live docs tell the same runtime story as the code,
2. no truth doc implies the bypass is still the intended architecture.

### Workstream 6: Validation and closeout

Required changes:

1. Run the static grep wall on production runtime code.
2. Run focused runtime tests for shell host control and world-member routing.
3. Run gateway and auth regression tests that prove the FD auth-bundle path still holds.
4. Run full workspace checks after focused coverage is green.
5. Produce one compact validation artifact for reviewers naming:
   - lifecycle invariants checked,
   - world first-turn versus follow-up checks,
   - auth-bundle checks,
   - seam-removal grep checks.

Exit gate:

1. seam movement and lifecycle stability are both proven,
2. auth, selection, and retained-member invariants still hold,
3. the repo no longer contains the production bypass in the targeted runtime surfaces.

## Code Quality Review

The implementation should be judged against these rules:

1. Prefer seam removal over seam layering. Delete duplicate backend-registration tables instead of hiding them behind more indirection.
2. Keep backend-kind mapping explicit and fail-closed, but move actual backend execution behind the gateway boundary.
3. Preserve typed request and response contracts wherever already landed. Do not widen stable schemas unless there is no internal-only alternative.
4. Any new helper must be named by responsibility, not by generic architecture vocabulary.
5. Update comments near changed control flow so future readers can see why direct backend registration is forbidden there.
6. Tests must prove the user-visible invariants, not just internal call graphs.
7. If a compatibility harness still needs direct wrapper instantiation, keep it in tests only and label it as non-production.
8. `async_repl.rs` is the biggest merge-conflict trap in the slice. Keep host-only and world-only prep out of that file unless the change is part of Workstream 4.

## Test Review

Runtime tests, seam-removal invariants, and auth-carrier tests are the authoritative test layers for this slice.

This is runtime and transport work, not prompt-quality work. No LLM output-quality eval suite is required. The burden is lifecycle, routing, auth, and seam-removal proof.

### CODE PATH COVERAGE

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

### USER FLOW COVERAGE

```text
USER FLOW COVERAGE
==================
[+] Host start
    substrate agent start --prompt ...
        - real user prompt is first prompt-bearing execution
        - lifecycle events still close with explicit terminal envelope

[+] Host follow-up
    substrate agent turn --session ... --backend ... --prompt ...
        - exact backend selector still required
        - resumed metadata still threads through the run
        - no local shell backend registry is reintroduced

[+] World first targeted turn
    shell routes member_dispatch.initial_prompt into world-service
        - typed request survives unchanged
        - first world prompt is the user prompt
        - any attach/readiness step stays off the prompt-bearing path

[+] World resumed follow-up
    shell routes MemberTurnSubmitRequestV1 into /v1/member_turn/stream
        - tuple validation still blocks detached or mismatched follow-up
        - same fulfillment seam as first targeted turn

[+] Failure-facing flows
    invalid backend id / dependency unavailable / policy denial / detached world
        - failures stay in the correct bucket
        - no bypass path silently "helps" by routing around the gateway seam
```

### Concrete test additions or tightening

| Target | Test requirement | Type |
| --- | --- | --- |
| `crates/shell/tests/agent_public_control_surface_v1.rs` | Add or tighten assertions that `start`, `turn`, `reattach`, `stop`, posture transitions, and `Accepted -> terminal` remain unchanged after the seam move. | focused integration |
| `crates/shell/tests/agent_successor_contract_ahcsitc0.rs` | Prove no hidden bootstrap or synthetic prompt appears in host prompt-bearing execution artifacts. | focused integration |
| `crates/shell/tests/repl_world_first_routing_v1.rs` | Tighten first-targeted-world-turn assertions so the real user prompt is the only first prompt-bearing input, and launch-time first turn plus resumed follow-up prove the same seam. | focused integration |
| `crates/world-service/tests/streamed_execute_cancel_v1.rs` | Preserve cancel and completion behavior while removing member-local backend registration. | focused integration |
| `crates/world-service/tests/member_runtime_world_placement_v1.rs` | Preserve retained-member tuple and world placement validation while the fulfillment seam moves. | focused integration |
| `crates/shell/tests/world_gateway.rs` and `crates/gateway/tests/openai_shared_parity.rs` | Prove the FD auth-bundle path, one-time consumption, and fail-closed auth precedence remain intact. | focused integration |
| Static repo invariant | Add one grep-backed invariant or test-time assertion that the targeted production runtime surfaces no longer instantiate `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend`. | static validation |
| Static repo invariant | Add one grep-backed invariant or test-time assertion that `runtime_bootstrap_prompt` or equivalent synthetic prompt-bearing bootstrap behavior is gone from production runtime code. | static validation |

### QA handoff artifact

Implementation is not done when the tests compile. The PR or implementation notes must also leave one compact handoff artifact that names:

1. the exact lifecycle commands exercised,
2. the exact world first-turn and resumed-follow-up scenarios exercised,
3. the auth-bundle checks performed,
4. the static seam-removal checks performed,
5. any remaining test-only direct wrapper uses that are intentionally non-production.

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
5. The implementation should spend zero innovation tokens on new architecture. This is a convergence slice. Boring is correct.

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
| A0. Freeze lifecycle and seam contract | truth docs, target tests, grep wall definitions | - |
| A1. Host fulfillment cutover | `crates/shell/src/execution/agent_runtime/`, host-facing paths in `crates/shell/src/repl/async_repl.rs`, host control tests | A0 |
| A2. World-member fulfillment cutover | `crates/world-service/src/member_runtime.rs`, minimum `crates/world-service/src/service.rs`, world routing tests | A0 |
| A3. Shared reconvergence and bootstrap removal | `crates/shell/src/repl/async_repl.rs`, `crates/world-service/src/member_runtime.rs`, shared routing tests | A1, A2 |
| B. Truth-doc sync | SOW, gap matrix, gateway contracts, usage docs | A3 |
| C. Validation wall and closeout | repo root, targeted tests, auth tests | A1, A2, A3, B |

### Parallel lanes

Lane 0: A0  
Reason: there is no useful parallelism until the contract, grep wall, and change budget are frozen.

Lane H: A1  
Reason: host cutover work is mostly concentrated in `crates/shell/src/execution/agent_runtime/` plus host lifecycle tests.

Lane W: A2  
Reason: world cutover work is mostly concentrated in `crates/world-service/src/member_runtime.rs` plus world routing tests.

Lane R: A3  
Reason: reconvergence touches the exact shared conflict zone, `async_repl.rs` plus bootstrap semantics plus shared routing assertions, so it must wait for H and W to land.

Lane D: B  
Reason: docs should not move until the runtime story has stopped changing.

Lane V: C  
Reason: validation is the merge gate and must run after code and docs converge.

### Safe parallel split

The only honest parallel split in this slice is:

1. one owner or worktree handles host fulfillment cutover,
2. one owner or worktree handles world-member fulfillment cutover,
3. one reconvergence owner handles bootstrap removal, `async_repl.rs`, and the shared lifecycle test updates after both cutovers merge.

This works because:

1. host seam work is concentrated in `crates/shell/src/execution/agent_runtime/`,
2. world seam work is concentrated in `crates/world-service/src/member_runtime.rs` and world routing tests,
3. the conflict zone is `crates/shell/src/repl/async_repl.rs`, bootstrap semantics, and shared lifecycle assertions, which is why A3 is serialized after A1 and A2.

### Suggested worktree ownership

| Lane | Owns | Should avoid |
| --- | --- | --- |
| H | `crates/shell/src/execution/agent_runtime/`, host lifecycle tests, host resume metadata plumbing | world-member execution semantics, typed member-turn transport, doc sync |
| W | `crates/world-service/src/member_runtime.rs`, world execution tests, minimal service plumbing | host bootstrap state, shell lifecycle semantics, doc sync |
| R | `crates/shell/src/repl/async_repl.rs`, shared routing tests, bootstrap removal, final seam wording validation | inventing new runtime abstractions after H and W already converged |

### Conflict flags

1. Do not split A3 across worktrees. It touches the exact boundary where host and world behavior becomes one user-visible prompt story.
2. `crates/shell/src/repl/async_repl.rs` is not fair game during both A1 and A2 except for the narrowest unavoidable plumbing. Major edits there belong to A3.
3. Do not start doc sync before A3. The plan depends on the final bootstrap decision and the exact post-cutover seam wording.
4. If A1 introduces helper shapes that A2 needs, rebase A2 onto A1 before final integration rather than cloning the helper differently in both lanes.

### Execution order

1. Land A0 first.
2. Launch H and W in parallel worktrees.
3. Merge H and W back to the main branch.
4. Run R as the serialized reconvergence pass.
5. Run D after the runtime story is stable.
6. Run V last, and use that output as the ship gate.

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
3. Step 0: the minimum honest change, blast radius, stop conditions, and reuse surfaces are explicit.
4. Current repo truth: the exact bypass points and bootstrap regression points are named.
5. Architecture review: ownership is explicit and the target seam is singular.
6. Implementation plan: each workstream has inputs, boundaries, constraints, and exit gates.
7. Test review: the exact runtime paths, proof obligations, and regression blockers are defined.
8. Failure modes: the dangerous regressions are named and treated as blockers.
9. Parallelization: one freeze step, one honest host/world parallel window, one serialized reconvergence step, then docs and validation.
10. Validation: grep, focused tests, auth checks, manual proof points, and full workspace gates are all specified.

After this slice lands, the runtime story should read as if it were intentional from the start:

1. shell owns lifecycle and routing,
2. typed runtime surfaces own transport and world attachment,
3. stable backend ids are selected before execution,
4. `substrate-gateway` owns adapter dispatch and backend internals,
5. no production prompt-bearing path bypasses that seam anymore.
