# PLAN-15: Land Explicit Targeted REPL Agent Turns With Backend-Routed Session Resume

Source SOW: [15-targeted-repl-agent-turns-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/15-targeted-repl-agent-turns-linux-first.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: Linux-first targeted REPL agent-turn plan, developer-facing shell UX, cross-crate (`shell` + `agent-api-types` + `agent-api-client` + `world-agent`), no UI scope  
Review posture: `/autoplan` scope discipline with `/plan-eng-review` depth, rewritten as one cohesive execution plan  
Status: execution-ready planning pass on 2026-05-04  
Outside voice: not used for this document generation

## Objective

Land one narrow, production-honest caller surface where the interactive REPL can submit explicit follow-up turns to a named backend that is already part of the live orchestration session.

The shipped behavior is:

1. the REPL accepts exactly one targeted-turn grammar, `::<backend_id> <prompt>`
2. the shell resolves that token to one exact configured backend such as `cli:codex`
3. routing is by explicit `backend_id`, never by "the one eligible member"
4. host-scoped turns resume the shell-owned UAA session locally
5. world-scoped turns resume the world-owned UAA session through a new typed `world-agent` route
6. streamed output comes back through the REPL with backend identity attached
7. plain REPL input still means shell execution
8. `substrate -c` stays wrap mode

This is the whole point of the slice. Not new agent product surface. Not a generalized daemon. Not a broader CLI redesign. Just explicit targeted REPL turns that are true to the ownership model already in the repo.

## Plan Summary

This plan does not try to inject text into the original bootstrap process. That would be fake progress.

It standardizes targeted turns as fresh submitted control runs against the exact surfaced UAA session id that bootstrap already established. Host turns stay shell-local. World turns go through one new typed `world-agent` stream route and reuse the existing host-to-world execution seam for placement, cancel, and authoritative world binding.

The first landing is intentionally serialized for world members: one retained world member per REPL session, explicit stop-and-start when switching backends, no hidden multiplexing.

## Locked Starting State

### What already exists

The repo already has the hard parts that this slice should reuse:

- shell-owned orchestrator bootstrap through UAA in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- authoritative runtime state persistence in [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- canonical backend identity derivation in [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)
- runtime realizability and protocol gating in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- Linux world-member bootstrap over the existing host-to-world stream seam in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- retained world-member control ownership inside [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- surfaced UAA session ids already persisted into runtime manifests during bootstrap
- focused test seams in [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs), [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs), and [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)

### Exact remaining gap

The remaining gap is smaller than the SOW had to assume:

1. the REPL only treats `:host` and `:pty` as directives on single-line input, then falls through to shell execution
2. world-member routing still uses `validate_member_selection(...)`, which fails closed on ambiguity instead of selecting the exact backend the operator named
3. both host and world bootstrap paths are fixed bootstrap prompts, not arbitrary follow-up prompt submission
4. `world-agent` only exposes launch, stream, and cancel today, not "submit a turn to the already-live member session"
5. `MemberRuntimeManager` is keyed by launch `span_id`, which is fine for launch cancel and wrong for later participant-targeted resume
6. the shell still retains only one `member_runtime: Option<AsyncReplAgentRuntime>`, so the first targeted-turn landing must embrace one-at-a-time world-member ownership
7. `substrate -c` and a broader public `substrate agent start|resume|fork|stop` surface remain intentionally out of scope

## Frozen Execution Contract

This section removes the wiggle room. If implementation wants to do something else, this plan is wrong and should be revised first.

### Non-negotiable invariants

1. Plain REPL input remains shell execution.
2. Only explicit `::<backend_id> <prompt>` input enters the targeted-agent lane.
3. `:host` and `:pty` keep their current meaning.
4. `substrate -c` remains `ShellMode::Wrap`.
5. Backend selection for targeted turns is by exact `backend_id`, never by eligible-member heuristics.
6. Targeted turns reuse surfaced UAA session identity through resume semantics. They do not replay the bootstrap prompt.
7. Host-scoped targeted turns stay shell-local.
8. World-scoped targeted turns stay Linux-first and go through `world-agent`.
9. The first landing supports at most one retained world-member runtime per REPL session at a time.
10. Cancellation remains span-based. Bootstrap lifetime spans and submitted-turn spans are different things and must stay different.

### Caller grammar

Grammar is frozen to one spelling:

- accepted: `::<backend_id> <prompt>`
- accepted example: `::cli:codex summarize the last failure`
- rejected: `@cli:codex ...`
- rejected: `:: cli:codex ...`
- rejected: `::cli:codex` with no prompt
- rejected: multi-line targeted turns in the first landing

Parser rules:

1. directive parsing only runs when `has_embedded_newlines(...) == false`
2. targeted-turn parsing runs before shell fallback execution
3. malformed targeted-turn syntax returns a REPL error, never shell execution
4. non-targeted input never partially matches and never rewrites into agent execution

### Backend-resolution contract

Targeted turns use one new additive selector in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs):

`validate_targeted_backend_selection(requested_backend_id, effective_config, inventory, base_policy) -> RuntimeSelectionDescriptor`

That helper must:

1. validate `requested_backend_id` syntax
2. resolve one exact effective inventory entry
3. require `derived_backend_id == requested_backend_id`
4. require `protocol == PURE_AGENT_PROTOCOL`
5. reuse current runtime-realizability checks
6. reuse current backend allowlist checks
7. preserve the entry's configured `execution.scope`

It must not call `validate_member_selection(...)` internally.

### Session-resume contract

Targeted turns are session-level follow-up runs, not stdin injection into the original process.

That means:

1. bootstrap still exists only to establish runtime ownership and surfaced UAA session identity
2. each targeted turn starts a new short-lived submitted control run against that same UAA session id
3. the submitted turn carries a fresh `run_id` and `span_id`
4. `participant_id`, `backend_id`, `orchestration_session_id`, and world binding stay stable
5. both host and world paths use the exact surfaced `internal.uaa_session_id`, not "resume last"

### Host contract

Host behavior is intentionally narrow:

1. the only active host-side agent runtime in this slice is the orchestrator runtime already attached to the current REPL session
2. a host-targeted turn is valid only when `requested_backend_id == retained_orchestrator_manifest.backend_id`
3. if the backend is host-scoped but does not match the active orchestrator backend for this REPL session, fail closed with a REPL-visible error
4. the shell rebuilds a gateway via [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
5. it submits `AgentWrapperRunRequest { prompt, working_dir, extensions["agent_api.session.resume.v1"] = { selector: "id", id: <uaa_session_id> } }`
6. shell-side output translation must tag the targeted `backend_id` and must end with a targeted-turn completion, not a shell-command completion

### World contract

World behavior is also narrow:

1. the shell never recreates world placement locally
2. `world-agent` remains the retained owner of the live world-member runtime after bootstrap
3. world submission goes through one new typed stream route
4. the route resolves the active member by stable participant identity, not launch span
5. the route validates `backend_id`, `orchestration_session_id`, `world_id`, and `world_generation` against the retained runtime
6. the route rejects submission when no surfaced `uaa_session_id` is retained
7. the route rejects concurrent submitted turns for the same participant
8. switching between world backends is explicit stop-and-start because the shell still owns only one retained `member_runtime` pointer

### Wire contract

The world path gets one additive typed route. It does not overload `ExecuteRequest.member_dispatch`.

New request type in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs):

`MemberTurnSubmitRequestV1`

Frozen fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | must be `1` |
| `orchestration_session_id` | active REPL orchestration session |
| `participant_id` | stable world-member participant to resume |
| `orchestrator_participant_id` | owning host orchestrator participant |
| `backend_id` | exact targeted backend |
| `run_id` | new submitted-turn run id |
| `world_id` | authoritative world id |
| `world_generation` | authoritative generation that must still match |
| `prompt` | non-empty targeted user prompt |

New route:

- `POST /v1/member_turn/stream`

New client method in [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs):

- `submit_member_turn_stream(MemberTurnSubmitRequestV1) -> Response<Incoming>`

Response contract:

1. reuse `ExecuteStreamFrame` as the NDJSON envelope so the shell does not need a second stream-decoding model
2. emit a fresh `Start { span_id }`
3. emit translated events/output for the submitted turn
4. emit `Exit { span_id }` for the submitted turn
5. keep cancel span identity aligned with that submitted-turn `span_id`

### Member-runtime registry contract

`MemberRuntimeManager` must stop pretending launch span is stable identity.

The refactor is frozen to two registries:

1. `active_members_by_participant_id`
   - key: `participant_id`
   - value: retained bootstrap ownership and resume context
2. `active_turns_by_span_id`
   - key: submitted-turn `span_id`
   - value: cancel handle plus participant association

`ActiveMemberRuntime` must retain at least:

- `participant_id`
- `orchestration_session_id`
- `orchestrator_participant_id`
- `backend_id`
- `world_id`
- `world_generation`
- `backend_kind`
- `binary_path`
- `working_dir`
- effective env overrides needed for resumed runs
- surfaced `uaa_session_id`
- bootstrap cancel handle
- launcher cleanup state

### Cancel contract

Cancel semantics are frozen because this is an easy place to lie accidentally:

1. the bootstrap lifetime span still cancels the retained member runtime itself
2. a submitted-turn span cancels only that submitted turn
3. `world-agent` must be able to resolve both span classes correctly
4. canceling a submitted turn must not silently tear down the retained bootstrap runtime
5. if a future cleanup step intentionally tears down the retained member, that is a separate action

### Failure taxonomy

Request and routing failures are not interchangeable:

- malformed targeted syntax, unknown backend id, wrong protocol, missing prompt: REPL user error
- backend not allowlisted: policy-style deny with current exit posture
- host runtime missing surfaced `uaa_session_id`: fail-closed runtime error
- host backend mismatch against active orchestrator backend: fail-closed runtime error
- world targeted turn on non-Linux: explicit Linux-first error
- world binding mismatch or stale generation: fail-closed runtime error
- participant not present in `world-agent` active registry: runtime unavailable
- concurrent submitted turn for one participant: conflict-style runtime error
- submitted-turn resume failure after validation: runtime failure, not policy failure

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| REPL one-line directive interception | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse the existing one-line directive gate. Insert targeted-turn parsing before shell fallback. |
| canonical `backend_id` derivation and inventory resolution | [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs) | Reuse. No second backend naming layer. |
| runtime realizability and allowlist checks | [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse realizability checks and add one explicit targeted selector. |
| shell-local backend construction | [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs) | Reuse for host-targeted resume runs. |
| authoritative live runtime identity | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse. No shadow session registry. |
| shell-to-world member launch transport | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse for launch only. Add one second typed route for submitted turns. |
| retained world-member control ownership | [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | Extend. Do not move control ownership back into the shell. |
| UAA resume semantics | external `unified-agent-api` already consumed by `shell` and `world-agent` | Reuse `agent_api.session.resume.v1`. Do not invent a bespoke prompt-submission protocol. |
| integration harnesses | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) and [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs) | Extend. Do not create a brand-new harness unless the current stub cannot express the route. |

### 0B. Minimum honest diff

The minimum honest implementation is:

1. add a targeted-turn parser to the REPL loop
2. add exact named-backend selection in `validator.rs`
3. add a shell-local host submitted-turn path that uses UAA resume against the stored session id
4. add one typed world submit-turn route plus client
5. refactor `MemberRuntimeManager` to resolve active members by participant identity and active turns by span
6. extend integration tests and the gap matrix so repo truth matches code truth

Anything smaller is fake progress.

Rejected shortcuts:

- overloading `substrate -c`
- injecting prompt text into the bootstrap process stdin
- overloading `ExecuteRequest.member_dispatch` for both launch and submit
- adding a generic new agent hub daemon
- pretending simultaneous retained world members already exist in the shell

### 0C. Complexity check

This slice touches more than 8 files. That is still the minimal blast radius because the seam crosses:

- REPL grammar and routing
- backend validation and policy gating
- typed host-to-world transport
- retained runtime identity inside `world-agent`
- integration tests proving shell semantics did not drift

Primary files expected to move:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
3. [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
4. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
5. [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
6. [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs)
7. [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
8. [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)
9. [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
10. [crates/world-agent/src/handlers.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs)
11. [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
12. [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
13. [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
14. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

### 0D. Search and completeness check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse UAA `agent_api.session.resume.v1`
- **[Layer 1]** reuse `build_gateway_for_descriptor(...)` for host resume runs
- **[Layer 1]** reuse the existing span-based cancel surface
- **[Layer 1]** keep `ExecuteRequest.member_dispatch` launch-only and add a separate typed request for submitted turns
- **[EUREKA]** the real world-path blocker is not parser work. It is the registry keying model in `MemberRuntimeManager`
- **[EUREKA]** the shell's single `member_runtime: Option<AsyncReplAgentRuntime>` is a product boundary, not an annoyance to paper over

Completeness rule for this plan:

- support the full explicit host path
- support the full explicit Linux world path
- include the regression floor for shell input, `:host`, `:pty`, and `-c`
- do not defer error handling for malformed syntax, backend deny, stale world generation, or submitted-turn cancel

### 0E. Distribution and runtime contract check

No new artifact type is introduced. This is not a packaging plan.

The ship surface is behavioral:

- REPL grammar and error posture
- typed `world-agent` transport contract
- retained runtime ownership correctness
- regression proof that shell semantics did not drift

### 0F. NOT in scope

- redesigning `substrate -c`
- adding non-interactive targeted-agent syntax
- public `substrate agent start|resume|fork|stop`
- implicit default-backend routing for ordinary REPL input
- simultaneous retained world members in one REPL session
- multi-line targeted prompts
- cross-platform world-target parity beyond explicit Linux-first behavior
- transcript persistence redesign
- toolbox or gateway work as a substitute for pure-agent targeted turns

## Architecture Review

### Locked architecture decisions

1. Targeted turns are explicit-only and single-line-only in v1.
2. Backend routing is exact `backend_id` routing, not ambiguity resolution.
3. Host submitted turns reuse UAA resume extensions locally.
4. World submitted turns go through a new typed `world-agent` stream route.
5. `ExecuteRequest.member_dispatch` remains launch-only.
6. The first landing supports one retained world-member runtime at a time per REPL session.

### Architecture findings resolved in-plan

**Issue 1. Host follow-up turns cannot be "inject into the existing process."**

The UAA boundary exposes new control runs, cancellation, surfaced session handles, and resume extensions. It does not expose "write another prompt into the already-running control handle." This plan standardizes follow-up turns as fresh resume runs keyed to the stored session id.

**Issue 2. Host routing has to acknowledge the real product shape.**

There is only one active host-side agent runtime in this slice: the orchestrator already booted by the REPL. This plan makes that explicit instead of pretending there is a general pool of host participants. A host-targeted turn either matches the active orchestrator backend or fails closed.

**Issue 3. Overloading `ExecuteRequest.member_dispatch` would collapse two different meanings into one struct.**

Launch needs runtime descriptor and empty `cmd`. Submit needs prompt plus active participant lookup. That is not one request with optional knobs. That is two different contracts. The plan keeps them separate.

**Issue 4. World runtime identity is keyed incorrectly for later targeted turns.**

Launch span is good enough for bootstrap cancellation and bad for later follow-up turns. The plan fixes the registry model before layering the new route on top.

**Issue 5. Multi-backend world targeting is larger than the current shell ownership model.**

The shell already keeps one retained world-member pointer. This plan freezes one-at-a-time world backend ownership and makes switching explicit instead of hiding that product boundary.

### Architecture ASCII diagrams

### REPL targeted-turn flow

```text
operator input
    |
    v
"::cli:codex summarize failures"
    |
    +--> parse_targeted_agent_turn(...)
           |
           +--> malformed? ----------> REPL error, stop
           |
           v
      validate_targeted_backend_selection(...)
           |
           +--> denied / unknown / unrealizable? --> REPL error, stop
           |
           v
      execution.scope?
      /              \
     /host            \world
    v                  v
submit_host_       ensure_targeted_member_runtime_ready(...)
targeted_turn(...)     |
    |                  +--> same backend + same generation? reuse
    |                  +--> different backend? stop old member, launch requested one
    |                  +--> non-Linux / stale binding? fail closed
    |                  v
    |             POST /v1/member_turn/stream
    |                  |
    |                  v
    |            world-agent MemberRuntimeManager
    |                  |
    |                  v
    |            UAA resume run_control(...)
    |                  |
    +-------- stream translated AgentEvents / text / completion --------+
                                                                      |
                                                                      v
                                                             REPL printer + trace
```

### World-member ownership model

```text
shell REPL session
    |
    +-- authoritative runtime state store
    |      orchestration_session_id
    |      participant_id
    |      backend_id
    |      world_id/world_generation
    |
    +-- retained local pointer
           member_runtime: Option<AsyncReplAgentRuntime>
                |
                +-- at most one active world member in this slice
                |
                +-- points to world-agent-owned runtime
                          |
                          +-- active_members_by_participant_id
                          |      participant_id -> bootstrap ownership + uaa_session_id
                          |
                          +-- active_turns_by_span_id
                                 submit span -> cancel handle
```

### Cancel model

```text
bootstrap launch
    |
    +-- bootstrap span_id ----------------------------> cancel retained member lifetime
    |
submitted targeted turn
    |
    +-- submitted-turn span_id -----------------------> cancel this turn only
```

## Code Quality Review

### Findings resolved in-plan

**Issue 1. Parser sprawl in `async_repl.rs`.**

Keep the parser small and local:

- one small `TargetedAgentTurn` value type
- one parser helper
- one routed handler per scope

No generic directive framework. No new parser crate. Minimal diff wins here.

**Issue 2. Selection logic drift risk.**

Backend validation already lives in `validator.rs`. This plan keeps targeted-turn routing logic there too instead of re-implementing backend-id checks inside the REPL loop.

**Issue 3. Event translation duplication risk.**

Host and world paths must both stamp `backend_id`, `participant_id`, `run_id`, `span_id`, and scope consistently. Add one submitted-turn translation helper or equivalent shared path. Do not let host and world invent incompatible output shapes.

**Issue 4. Registry explicitness is more important than abstraction purity.**

The world registry changes are structural, not cosmetic. This plan prefers a blunt, obvious `ActiveMemberRuntime` over a clever generic state machine.

### Allowed code shape

1. Keep targeted-turn parsing in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
2. Keep backend selection in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs).
3. Add only one new typed host-to-world request for submitted turns.
4. Keep `MemberRuntimeManager` explicit. No trait hierarchy. No generalized runtime state machine.
5. Do not add new global shell state beyond what the REPL already owns.

## Test Review

### Test framework detection

This repo is Rust-first. The test surface for this plan is the existing Rust unit and integration suite driven by `cargo test`.

Relevant suites already exist in:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs)
- unit tests in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- wrap-mode tests in [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs)
- config validation tests in [crates/shell/tests/agents_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs)

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/repl/async_repl.rs
    |
    ├── parse_targeted_agent_turn()
    │   ├── [GAP] Accepts exact "::<backend_id> <prompt>" syntax
    │   ├── [GAP] Rejects missing backend id
    │   ├── [GAP] Rejects missing prompt
    │   ├── [GAP] Rejects multi-line targeted input
    │   └── [REGRESSION] Plain shell input still falls through untouched
    |
    ├── validate_targeted_backend_selection(...)
    │   ├── [GAP] Unknown backend id
    │   ├── [GAP] Wrong protocol
    │   ├── [GAP] Policy deny
    │   └── [GAP] Explicit world backend chosen among multiple configured members
    |
    ├── submit_host_targeted_turn(...)
    │   ├── [GAP] Exact surfaced session id mapped into resume extension
    │   ├── [GAP] Missing stored session id fails closed
    │   ├── [GAP] Host backend mismatch fails closed
    │   └── [GAP] Streamed completion is surfaced as targeted-turn completion
    |
    └── ensure_targeted_member_runtime_ready(...)
        ├── [GAP] Launches requested backend when no world member is retained
        ├── [GAP] Reuses current retained backend when same backend + generation
        ├── [GAP] Switches retained world backend when target differs
        └── [GAP] Non-Linux path fails closed with explicit error

[+] crates/world-agent/src/member_runtime.rs
    |
    ├── register_active_member(...)
    │   ├── [GAP] Stores stable participant identity + surfaced session id
    │   └── [GAP] Cleanup removes participant registry entry and launcher dir
    |
    ├── submit_turn(...)
    │   ├── [GAP] Resolves by participant/backend/world identity
    │   ├── [GAP] Rejects concurrent active turn for the same participant
    │   ├── [GAP] Uses exact session id resume, not "last"
    │   └── [GAP] Submitted-turn cancel works by returned submit span id
    |
    └── cancel(...)
        ├── [GAP] Cancels bootstrap span
        └── [GAP] Cancels submitted-turn span

[+] crates/agent-api-types/src/lib.rs
    |
    └── MemberTurnSubmitRequestV1::validate()
        ├── [GAP] Empty prompt rejected
        ├── [GAP] Empty participant/backend rejected
        └── [GAP] Round-trip serde boundary

─────────────────────────────────
COVERAGE: 0/19 targeted paths covered today
QUALITY TARGET: every new path reaches at least ★★, and all regressions reach ★★★
GAPS: 19 new/changed paths need tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Operator targeted host turn
    |
    ├── [GAP] "::cli:codex hello" routes to host backend, not shell execution
    ├── [GAP] streamed output is visible in REPL
    ├── [GAP] completion/failure is visibly distinct from shell command completion
    └── [GAP] host backend mismatch returns explicit guidance

[+] Operator targeted world turn
    |
    ├── [GAP] "::cli:codex hello" on Linux launches or reuses world member then submits turn
    ├── [GAP] world backend switch tears down previous retained backend explicitly
    ├── [GAP] stale world generation fails closed
    └── [GAP] cancel stops the submitted turn, not the bootstrap runtime

[+] Operator error states
    |
    ├── [GAP] malformed syntax explains exact fix
    ├── [GAP] unknown backend suggests `substrate agent list`
    ├── [GAP] denied backend names the blocked backend id
    └── [GAP] non-Linux world target returns explicit Linux-first error

[+] Regression floor
    |
    ├── [REGRESSION] Plain "echo hi" still executes as shell input
    ├── [REGRESSION] ":host ..." still works when enabled
    ├── [REGRESSION] ":pty ..." still works
    └── [REGRESSION] `substrate -c` still maps to wrap mode
```

### Required tests to add or extend

1. Add parser unit tests in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for exact syntax, missing prompt, backend-id extraction, and multi-line rejection.
2. Add unit tests in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) for exact backend selection, deny, wrong protocol, unknown backend, and multi-member explicit routing.
3. Extend [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) to prove:
   - targeted syntax is not shell execution
   - plain shell input is unchanged
   - targeted host turn emits a resume request against the stored session id
   - host backend mismatch fails closed
   - targeted world turn uses the new world submit route
   - targeted world turn chooses the named backend when multiple world members exist
   - switching world backends is explicit and one-at-a-time
4. Extend [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) to capture and script `/v1/member_turn/stream`.
5. Add request boundary tests in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) for `MemberTurnSubmitRequestV1`.
6. Extend [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs) to cover submitted-turn streaming and cancel.
7. Keep [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs) green to prove the registry refactor did not break placement.
8. Keep [crates/shell/tests/agents_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs) green to prove config/inventory validation posture did not drift.
9. Add or extend a regression in [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs) proving `-c` remains wrap mode after targeted-turn support lands.

### Regression rule for this slice

This slice changes existing REPL parsing behavior. That makes these regressions mandatory:

1. plain shell input still executes as shell input
2. `:host` and `:pty` still behave exactly as before
3. `substrate -c` still remains wrap mode

These are the highest-priority tests in the plan. If they fail, the feature is not shippable.

## Failure Modes Registry

| Failure mode | Surface | Test coverage required | Handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| malformed `::` syntax | shell parser | yes | reject before shell fallback | clear syntax error |
| backend id not in inventory | selector | yes | fail closed | "unknown backend `<id>`" |
| backend denied by policy | selector | yes | fail closed | deny naming exact backend |
| host runtime missing `uaa_session_id` | host submit | yes | fail closed | runtime unavailable error |
| host backend mismatch | host submit | yes | fail closed | explicit mismatch guidance |
| world target on non-Linux | shell route | yes | fail closed | explicit Linux-first error |
| targeted world backend on stale generation | shell + world-agent | yes | fail closed | stale-generation error |
| participant not in `world-agent` registry | world submit | yes | fail closed | runtime unavailable error |
| concurrent submitted turn for one participant | world submit | yes | fail closed | "turn already in flight" |
| cancel delivered to wrong span class | world cancel | yes | handle both bootstrap and submitted-turn spans | correct target stops |
| plain shell input accidentally intercepted | shell parser | yes, regression | reject targeted parser path | shell behavior preserved |

Critical gap rule for this plan:

no failure mode is allowed to be both untested and silent. Every fail-closed path must produce either a typed transport error or a REPL-visible error message.

## Performance Review

This slice is latency-sensitive and human-paced, not throughput-sensitive.

### Findings resolved in-plan

1. Each targeted turn spawns a new short-lived resume process. That is acceptable because operator pacing and agent latency dominate process-spawn cost.
2. The plan avoids a new long-lived daemon or generic multiplexor. That keeps operational complexity down and spends zero extra innovation tokens on infrastructure.
3. The one-at-a-time world-member rule is also a performance guardrail. It prevents hidden concurrency and shared-state contention in the first landing.

### Performance posture

- no new N+1 style concern exists
- no caching layer is needed
- no new background polling loop is introduced
- the only persistent new memory is one stable active-member record per retained participant plus one active-turn record per submitted turn

## DX Guardrails

This is developer-facing product work even though it is backend code.

Required operator experience:

1. malformed syntax errors must show the exact accepted format: `::<backend_id> <prompt>`
2. unknown backend errors should suggest `substrate agent list`
3. policy denies must name the exact blocked backend id
4. host backend mismatch must explain that the active REPL session is attached to a different host backend
5. world-target errors on non-Linux must say Linux-first explicitly
6. submitted-turn status and completion lines should include the targeted `backend_id`

Small details. Real product impact.

## Worktree Parallelization Strategy

This plan has real parallelization opportunities after the wire contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze targeted-turn contract | `crates/agent-api-types/`, `crates/agent-api-client/`, `crates/world-agent/src/` | — |
| Shell parser + selector + host submit path | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | Freeze targeted-turn contract |
| World submit route + member registry refactor | `crates/world-agent/src/`, `crates/agent-api-types/`, `crates/agent-api-client/` | Freeze targeted-turn contract |
| Integration tests + stubs + docs | `crates/shell/tests/`, `crates/world-agent/tests/`, repo docs | Shell parser + selector + host submit path, World submit route + member registry refactor |

### Parallel lanes

- Lane A: Shell parser + selector + host submit path
  - sequential inside the lane because these steps share `crates/shell/src/repl/` and `crates/shell/src/execution/agent_runtime/`
- Lane B: World submit route + member registry refactor
  - sequential inside the lane because these steps share `crates/world-agent/src/` and the new typed request/client contract
- Lane C: Integration tests + stubs + docs
  - starts after A and B because the test stub must know the final route and request shape

### Execution order

1. Freeze the transport and resume contract.
2. Launch Lane A and Lane B in parallel worktrees.
3. Merge A and B.
4. Run Lane C on top for integration proof, cancel-path proof, and gap-matrix closeout.

### Conflict flags

- Lane A and Lane C both touch `crates/shell/tests/repl_world_first_routing_v1.rs`. Keep that file owned by Lane C after A lands.
- Lane B and Lane C both touch `crates/world-agent/tests/streamed_execute_cancel_v1.rs`. Same rule: B lands runtime changes, C lands final proof coverage.
- `crates/agent-api-types/src/lib.rs` is the contract hotspot. Only one lane edits it at a time.

### Parallelization verdict

Three workstreams, two parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the submit-turn wire contract

Files:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/src/handlers.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)

Deliver:

1. add `MemberTurnSubmitRequestV1`
2. validate non-empty prompt plus required identity fields
3. add `AgentClient::submit_member_turn_stream(...)`
4. add `POST /v1/member_turn/stream`
5. reuse `ExecuteStreamFrame` for stream output

Done means the new transport contract compiles, validates at the boundary, and has request round-trip tests before any runtime logic is layered on top.

### Step 2. Add exact targeted-turn parsing and selection in the shell

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)

Deliver:

1. add `TargetedAgentTurn { backend_id, prompt }`
2. add `parse_targeted_agent_turn(...)`
3. add `validate_targeted_backend_selection(...)`
4. route targeted turns before shell fallback and after the current single-line directive gate
5. keep plain shell input, `:host`, and `:pty` behavior unchanged

Done means the shell can parse and route targeted turns to a scope-specific handler without depending on the world submit path yet.

### Step 3. Implement the shell-local host submitted-turn path

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)

Deliver:

1. resolve the active host runtime from the retained orchestrator manifest
2. require `requested_backend_id == retained_orchestrator.backend_id`
3. fail closed if the retained manifest does not contain a surfaced `uaa_session_id`
4. build a new `AgentWrapperRunRequest` carrying:
   - the operator prompt
   - current working directory
   - `agent_api.session.resume.v1 = { selector: "id", id: <uaa_session_id> }`
5. translate wrapper events into targeted-turn REPL and trace output
6. surface completion and failures distinctly from shell-command completion

Done means `::cli:codex hello` works for a host-scoped `cli:codex` REPL session and errors clearly when the requested host backend is not the active orchestrator backend.

### Step 4. Refactor `world-agent` member ownership for submitted turns

Files:

- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)

Deliver:

1. store stable active member state by `participant_id`
2. retain surfaced `uaa_session_id` from bootstrap registration events
3. keep submitted-turn cancel handles in a separate `span_id` map
4. add `submit_turn(...)` that:
   - validates participant/backend/world identity
   - rejects concurrent active turns
   - uses UAA resume by exact session id
   - streams `ExecuteStreamFrame` NDJSON back to the caller
5. extend cancel handling so both bootstrap spans and submitted-turn spans resolve correctly

Done means `world-agent` can truthfully resume an already-live member session instead of only launching it.

### Step 5. Wire the targeted Linux world path in the REPL

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. add `ensure_targeted_member_runtime_ready(...)`
2. when no world member is retained, launch the requested backend through the existing member-dispatch seam
3. when the requested backend matches the retained backend and generation, reuse it
4. when the requested backend differs, stop the current retained member cleanly and launch the requested backend
5. submit the targeted prompt through `submit_member_turn_stream(...)`
6. reject non-Linux world targets explicitly
7. preserve current world drift and generation reconciliation behavior

Done means explicit backend targeting works end to end for Linux world members, including backend switching, without pretending simultaneous retained members already exist.

### Step 6. Close the regression floor and repo truth

Files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/shell/tests/agents_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs)
- [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Deliver:

1. targeted-turn integration tests for host and world paths
2. submit-route capture and scripted responses in the REPL world-agent stub
3. cancel-path proof for submitted turns
4. explicit wrap-mode regression for `substrate -c`
5. gap-matrix update that marks explicit targeted turns and real user-turn submission as landed, while keeping broader CLI productization open

Done means the docs tell the truth and the regression floor is real.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p world-agent --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

## Definition of Done

1. `::<backend_id> <prompt>` is the only accepted targeted-turn grammar.
2. Plain REPL input still runs as shell input.
3. `:host` and `:pty` still behave as before.
4. Host-targeted turns resume the exact surfaced UAA session id.
5. Host-targeted turns fail closed when the requested host backend is not the active orchestrator backend for the current REPL session.
6. Linux world-targeted turns go through the new typed `world-agent` submit route.
7. Multiple configured world members no longer block targeted routing when `backend_id` is explicit.
8. The first landing enforces one retained world-member runtime at a time and makes backend switching explicit.
9. Submitted turns stream output visibly in the REPL and return a distinct completion signal.
10. Submitted-turn cancel works by returned span id without tearing down the retained runtime accidentally.
11. `substrate -c` still remains wrap mode.
12. Tests and gap-matrix docs are updated together.

## Deferred Work

- public `substrate agent start|resume|fork|stop`
- non-interactive targeted prompt surface
- multi-line targeted prompts
- simultaneous retained world members in one REPL session
- cross-platform world-target parity beyond Linux
- transcript persistence and richer session-history UX

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is with one explicit constraint added: one retained world member at a time
- Architecture Review: 5 issues found, all resolved in-plan
- Code Quality Review: 4 issues found, all resolved in-plan
- Test Review: diagram produced, 19 targeted gaps identified
- Performance Review: 3 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, deferred work stays inside this plan
- Failure modes: 0 acceptable silent gaps, 0 unresolved critical gaps after planned coverage lands
- Outside voice: skipped for this document generation
- Parallelization: 3 lanes, 2 parallel / 1 sequential integration lane
- Lake Score: 9/9 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Freeze grammar to `::<backend_id> <prompt>` only | Mechanical | Explicit over clever | One spelling keeps parser, docs, and tests honest | `@backend`, implicit default-agent routing |
| 2 | Architecture | Use UAA resume semantics for submitted turns | Mechanical | DRY | The dependency already supports resume; do not invent a second backend protocol | stdin injection into bootstrap process |
| 3 | Architecture | Make host-targeted turns valid only for the active orchestrator backend of the current REPL session | Mechanical | Systems over heroes | There is one real host runtime in this slice, not a synthetic pool of host participants | fake host multi-runtime routing |
| 4 | Architecture | Add `MemberTurnSubmitRequestV1` instead of overloading `member_dispatch` | Mechanical | Explicit over clever | Launch and submit are different contracts and should stay different | dual-purpose `ExecuteRequest.member_dispatch` |
| 5 | Architecture | Refactor `MemberRuntimeManager` to key active members by participant id and active turns by span id | Mechanical | Systems over heroes | Stable participant identity is required for follow-up turns and correct cancel behavior | launch-span-only registry |
| 6 | Scope | Support one retained world member at a time in this landing | Taste, resolved | Pragmatic | Matches the current shell ownership model and keeps the diff bounded | pretending simultaneous retained members already exist |
| 7 | Test Review | Make plain-shell and `-c` behavior regressions mandatory | Mechanical | Completeness | Parser work is dangerous if the old shell contract is not explicitly proven | relying on manual validation |
| 8 | DX | Require explicit backend-specific errors for deny, mismatch, and Linux-only failures | Mechanical | Completeness | This feature is pure operator UX, so vague errors are product bugs | generic runtime failures |
| 9 | Parallelization | Freeze the wire contract first, then run shell and world lanes in parallel | Mechanical | Pragmatic | The request shape is the shared seam; parallel work is safe only after that stabilizes | parallel edits to the request contract |
