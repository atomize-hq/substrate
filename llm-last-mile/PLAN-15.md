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

The runtime substrate is already real. The missing piece is the operator-facing turn surface.

This plan lands one narrow, production-honest path where the interactive REPL can:

1. recognize only one explicit targeted-turn grammar, `::<backend_id> <prompt>`,
2. resolve that token to one exact configured backend such as `cli:codex`,
3. route by explicit `backend_id` instead of the current "single eligible world member" heuristic,
4. submit the prompt as a real follow-up turn into the already-established UAA session for that backend,
5. stream the resulting agent output back through the REPL with backend identity attached,
6. preserve current shell-first REPL semantics for all non-targeted input,
7. keep `substrate -c` untouched as shell wrap mode.

The critical implementation decision is this:

- targeted turns do not try to inject text into the original bootstrap process,
- they reuse the surfaced UAA session handle through UAA resume semantics,
- and they do it through two explicit lanes:
  - shell-local host submission for host-scoped backends,
  - a new typed `world-agent` submit-turn route for Linux world-scoped backends.

That is the smallest honest answer. Anything smaller either lies about the existing ownership model or pretends a post-bootstrap submission primitive already exists when it does not.

## Locked Starting State

### What is already done

The following work is landed and is not reopened here:

- shell-owned orchestrator bootstrap through UAA in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- authoritative runtime state persistence in [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- canonical backend identity derivation and inventory loading in [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)
- runtime realizability and protocol gating in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- Linux world-member launch over the existing host-to-world transport seam in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- retained control ownership inside `world-agent` for active member runtimes in [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
- UAA session-handle surfacing and resume capability in the external `unified-agent-api` dependency already consumed by `shell` and `world-agent`

### Exact remaining gap

The remaining gap is concrete and narrower than the SOW originally needed to assume:

1. the REPL still only treats `:host` and `:pty` as directives, and every other one-line input falls through to shell execution in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:669)
2. world-member routing still depends on `validate_member_selection(...)`, which fails closed on multiple eligible members instead of resolving the backend the operator named in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:176)
3. host bootstrap and world-member bootstrap both use fixed bootstrap prompts, but neither surface a repo-owned helper for arbitrary post-bootstrap turn submission
4. `world-agent` only exposes launch, stream, and cancel for member runtimes today, not "submit a prompt to the already-live session"
5. `MemberRuntimeManager` keys active runtimes by launch `span_id`, which is enough for cancel, but wrong for later targeted turns that naturally address a stable participant/backend identity
6. the REPL only retains one local `member_runtime` handle at a time, so the first targeted-turn landing must codify one-at-a-time world-member ownership instead of pretending simultaneous retained world members already exist
7. `substrate -c` and the public `substrate agent` namespace still intentionally stay outside this slice

## Frozen Execution Contract

This section removes the implementation wiggle room.

### Non-negotiable invariants

1. Plain REPL input remains shell execution.
2. Only explicit `::<backend_id> <prompt>` input enters the targeted-agent lane.
3. `:host` and `:pty` keep their current meaning.
4. `substrate -c` remains `ShellMode::Wrap`.
5. Backend selection for targeted turns is by exact `backend_id`, never by "pick the one eligible member."
6. Targeted turns reuse surfaced UAA session identity through resume semantics. They do not replay the bootstrap prompt.
7. Host-scoped targeted turns stay shell-local.
8. World-scoped targeted turns stay Linux-first and go through `world-agent`.
9. The first landing supports at most one retained world-member runtime per REPL session at a time. Switching world backends is explicit stop-and-start, not hidden multiplexing.
10. Cancellation remains span-based and best-effort. Submitted turns get their own span ids and use the existing cancel surface.

### Chosen caller grammar

The grammar is frozen to one spelling:

- accepted: `::<backend_id> <prompt>`
- accepted example: `::cli:codex summarize the last failure`
- rejected: `@cli:codex ...`
- rejected: `:: cli:codex ...`
- rejected: `::cli:codex` with no prompt
- rejected: multi-line targeted turns in the first landing

Parser rules:

1. directive parsing only runs when `has_embedded_newlines(...) == false`
2. the parser checks targeted-turn syntax before shell fallback execution
3. malformed targeted-turn syntax returns REPL-facing user errors, not shell execution
4. non-targeted input never partially matches and never rewrites into agent execution

### Chosen backend-resolution contract

Targeted turns use a new additive selector in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs):

`validate_targeted_backend_selection(requested_backend_id, effective_config, inventory, base_policy) -> RuntimeSelectionDescriptor`

The helper must:

1. validate `requested_backend_id` syntax
2. resolve one exact effective inventory entry
3. require `derived_backend_id == requested_backend_id`
4. require `protocol == PURE_AGENT_PROTOCOL`
5. reuse existing runtime-realizability checks
6. require `backend_allowed(...) == true`
7. preserve the entry's configured `execution.scope`

It must not call `validate_member_selection(...)` internally.

### Chosen session-resume contract

This plan resolves the biggest ambiguity explicitly:

targeted turns are session-level follow-up runs, not stdin injection into the original bootstrap process.

That means:

1. bootstrap still exists only to establish authoritative runtime ownership and surfaced UAA session identity
2. each targeted turn starts a new short-lived submitted control turn against the same UAA session handle
3. the submitted turn carries a new `run_id` and `span_id`
4. the participant identity, `backend_id`, `orchestration_session_id`, and world binding remain the same
5. host and world paths both use exact surfaced `internal.uaa_session_id`; they do not rely on `resume last`

For host-scoped backends:

- the shell rebuilds a gateway via [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
- it sends `AgentWrapperRunRequest { prompt, extensions["agent_api.session.resume.v1"] = { selector: "id", id: <uaa_session_id> } }`
- it streams events back into the REPL and trace layer as a submitted targeted turn

For world-scoped backends:

- `world-agent` stores the surfaced `uaa_session_id` in the stable active member registry
- `world-agent` starts a new short-lived submitted control turn against that stored session id
- the shell never tries to recreate world placement locally

### Chosen world submit-turn transport contract

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

New route in `world-agent`:

- `POST /v1/member_turn/stream`

New client method in [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs):

- `submit_member_turn_stream(MemberTurnSubmitRequestV1) -> Response<Incoming>`

The route contract is:

1. validate the request shape at the transport boundary
2. resolve one active member runtime by stable participant identity, not launch span
3. confirm `backend_id`, `orchestration_session_id`, `world_id`, and `world_generation` still match the retained runtime
4. reject submission if no surfaced `uaa_session_id` is retained
5. start one submitted control turn with UAA resume semantics
6. stream NDJSON frames back to the shell
7. register the submitted-turn cancel handle by submitted-turn `span_id`
8. reject concurrent submitted turns for the same participant with a clear error

### Chosen member-runtime registry contract

`MemberRuntimeManager` stops pretending launch span is stable identity.

The refactor is frozen to two registries:

1. `active_members_by_participant_id`
   - key: `participant_id`
   - value: retained bootstrap ownership and resume context
2. `active_turns_by_span_id`
   - key: submitted-turn `span_id`
   - value: cancel handle and participant association for the active targeted turn

`ActiveMemberRuntime` must retain:

- `participant_id`
- `orchestration_session_id`
- `orchestrator_participant_id`
- `backend_id`
- `world_id`
- `world_generation`
- `backend_kind`
- `binary_path`
- `working_dir`
- effective env overrides for resumed turns
- surfaced `uaa_session_id`
- bootstrap cancel handle and launcher cleanup state

This is required so the world submit-turn route can resume exactly the session that launch established, inside the same world placement, without copying control back into the shell.

### Failure taxonomy freeze

Request and routing failures are not all the same:

- malformed targeted syntax, unknown backend id, wrong protocol, missing prompt: REPL user error
- backend not allowlisted: policy-style deny with the current exit posture
- host runtime missing surfaced `uaa_session_id`: fail-closed runtime error
- world targeted turn on non-Linux: explicit Linux-only error
- world binding mismatch or stale generation: fail-closed runtime error
- participant not present in `world-agent` active registry: runtime unavailable
- concurrent submitted turn for one participant: conflict-style runtime error
- submitted-turn resume failure after validation: runtime failure, not policy failure

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| REPL one-line directive interception | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse the existing one-line directive gate and insert targeted-turn parsing before shell fallback. |
| canonical `backend_id` derivation and inventory resolution | [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs) | Reuse. Do not invent a second backend naming layer. |
| runtime realizability and allowlist checks | [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse the realizability checks, add one explicit targeted-backend selector. |
| shell-local backend construction | [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs) | Reuse for host-targeted submitted turns. |
| authoritative live runtime identity | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse. No new shell-only session registry. |
| shell-to-world member launch transport | [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) | Reuse for launch only. Add a second typed route for submitted turns. |
| member runtime bootstrap ownership in world-agent | [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | Extend. Do not replace with a shell-owned world helper. |
| UAA resume semantics | external `unified-agent-api` already consumed by `shell` and `world-agent` | Reuse `agent_api.session.resume.v1`; do not invent a bespoke post-bootstrap backend protocol. |
| existing shell/world integration tests | [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs), [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs) | Extend. Do not create a brand-new test harness unless the existing stub cannot express the new stream route. |

### 0B. Minimum honest diff

The minimum honest implementation is:

1. add a targeted-turn parser to the REPL loop
2. add exact named-backend selection in `validator.rs`
3. add a shell-local host submitted-turn path that uses UAA resume against stored session id
4. add one typed world submit-turn route plus client
5. refactor `MemberRuntimeManager` to resolve active members by participant identity and submitted turns by span
6. extend integration tests and the gap matrix so the repo truth matches the code truth

Anything smaller is fake progress.

Specifically rejected:

- overloading `substrate -c`
- trying to inject follow-up prompt text into the bootstrap process stdin
- overloading `ExecuteRequest.member_dispatch` for both launch and submit
- adding a generic new "agent hub daemon" or broader runtime service
- pretending simultaneous retained world members already exist in the shell

### 0C. Complexity check

This slice touches more than 8 files. That is justified and still minimal because the seam is cross-boundary by definition.

Expected primary files:

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

That looks large. It is still the minimal blast radius because the change spans:

- user-facing REPL grammar
- backend policy/routing
- typed host/world transport
- runtime ownership inside `world-agent`
- end-to-end regression tests

### 0D. Search and completeness check

Search-before-building result:

- **[Layer 1]** reuse UAA `agent_api.session.resume.v1` instead of inventing a new backend protocol
- **[Layer 1]** reuse `build_gateway_for_descriptor(...)` for host resumed turns instead of a second backend registry
- **[Layer 1]** reuse the existing cancel surface by span id instead of adding a second cancel API
- **[Layer 1]** keep `ExecuteRequest.member_dispatch` launch-only and add a separate typed request for submitted turns
- **[EUREKA]** the real world-path blocker is not parsing or transport. It is that `MemberRuntimeManager` stores active runtime ownership by launch span, which is the wrong key for any later targeted resume operation
- **[EUREKA]** the shell's single `member_runtime: Option<AsyncReplAgentRuntime>` is a hard product boundary for this slice. The clean first landing is explicit one-at-a-time world backend switching, not hidden multi-member concurrency

Shortcut options rejected because they save human work, not AI work:

- collapse host and world submit semantics into one implicit helper
- silently use `resume last` instead of the exact surfaced session id
- support multi-line targeted prompts in the same landing
- broaden the public `substrate agent` namespace while this REPL seam is still landing

### 0E. Distribution and runtime contract check

No new artifact type is introduced. This is not a packaging plan.

The real ship surface is:

- REPL syntax and error behavior
- `world-agent` typed transport contract
- runtime ownership correctness for active world members
- regression tests proving `substrate -c` and shell-first REPL semantics did not drift

That means the required proof is test and behavior proof, not release-pipeline work.

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

The current UAA boundary exposes `run_control(...)`, cancellation, session handles, and resume extensions. It does not expose "write another prompt into this already-running control handle."

The plan resolves this by standardizing submitted turns as UAA resume runs keyed to the surfaced session id. That is explicit, supported by the dependency, and testable.

**Issue 2. Overloading `ExecuteRequest.member_dispatch` would mix two incompatible meanings.**

Launch needs `resolved_runtime.binary_path` and empty `cmd`. A submitted turn needs a prompt and a stable active participant lookup. Combining them would produce a muddy request model and more conditionals than the extra route saves.

The plan resolves this by adding `MemberTurnSubmitRequestV1` as a second typed transport contract.

**Issue 3. World runtime identity is keyed incorrectly for later targeted turns.**

Today the registry is keyed by launch span. That is enough for cancellation of the bootstrap stream and not enough for exact resume against a stable participant. The plan fixes the keying model before adding the submit route.

**Issue 4. Multi-backend world targeting is larger than the current shell ownership model.**

The shell already keeps only one retained world-member runtime handle. The plan does not pretend otherwise. It freezes one-at-a-time world backend ownership for this landing and makes switching explicit.

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
    |                  +--> switch backend if current retained world member differs
    |                  +--> fail closed on non-Linux / stale world binding
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
    +-- authoritative state store
    |      orchestration_session_id
    |      participant_id
    |      backend_id
    |      world_id/world_generation
    |
    +-- retained local pointer
           member_runtime: Option<AsyncReplAgentRuntime>
                |
                +-- one active world member at a time in this slice
                |
                +-- points to world-agent-owned runtime
                          |
                          +-- active_members_by_participant_id
                          |      participant_id -> bootstrap ownership + uaa_session_id
                          |
                          +-- active_turns_by_span_id
                                 submit span -> cancel handle
```

### Transport field mapping

| Surface | Identity source | Prompt source | Stream identity |
| --- | --- | --- | --- |
| host targeted turn | shell manifest `internal.uaa_session_id` + `backend_id` | REPL input text | submitted-turn `run_id` + `span_id`, same participant/backend |
| world targeted turn | `world-agent` active member registry `uaa_session_id` + request participant/backend fields | `MemberTurnSubmitRequestV1.prompt` | submitted-turn `run_id` + `span_id`, same participant/backend |

## Code Quality Review

### Findings resolved in-plan

**Issue 1. Parser sprawl in `async_repl.rs`.**

The file is already large. The plan keeps the parser tiny and local:

- one small `TargetedAgentTurn` value type
- one parser helper
- one routed handler per scope

No generic directive framework. No new parser module.

**Issue 2. Selection logic drift risk.**

Backend validation already lives in `validator.rs`. The plan keeps targeted-turn routing there too, instead of re-implementing backend-id checks inside the REPL loop.

**Issue 3. Event translation duplication risk.**

The host path and world path should both stamp `backend_id`, `participant_id`, `run_id`, `span_id`, and scope consistently. The plan reuses the existing event-building style and adds one submitted-turn translation helper instead of letting host and world invent separate event shapes.

**Issue 4. Registry correctness is more important than micro-abstraction purity.**

The world registry changes are structural, not cosmetic. This plan spends the extra explicit fields to make submit-turn routing obvious. That is the right trade for a control-plane boundary.

### Allowed code shape

1. Keep targeted-turn parsing inside [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
2. Keep backend selection inside [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs).
3. Add only one new typed host-to-world request for submitted turns.
4. Keep `MemberRuntimeManager` explicit. No trait hierarchy. No generic state machine abstraction for two maps.
5. Do not add new global shell state beyond what the REPL already owns.

## Test Review

### Test framework detection

This repo is Rust-first. The test framework is the existing Rust unit/integration suite driven by `cargo test`.

Relevant current suites already exist in:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs)
- unit tests in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- CLI wrap-mode tests in [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs)

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
COVERAGE: 0/18 targeted paths covered today
QUALITY TARGET: every new path reaches at least ★★, and all regressions reach ★★★
GAPS: 18 new/changed paths need tests
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
    └── [GAP] completion/failure is visibly distinct from shell command completion

[+] Operator targeted world turn
    |
    ├── [GAP] "::cli:codex hello" on Linux launches or reuses world member then submits turn
    ├── [GAP] world backend switch tears down previous retained backend explicitly
    ├── [GAP] stale world generation fails closed
    └── [GAP] cancel stops the submitted turn, not the whole bootstrap registry

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

1. Add parser unit tests in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) for exact syntax, missing prompt, and multi-line rejection.
2. Add unit tests in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) for exact backend selection, deny, wrong protocol, and multi-member explicit routing.
3. Extend [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) to prove:
   - targeted syntax is not shell execution
   - plain shell input is unchanged
   - targeted host turn emits a resume request against the stored session id
   - targeted world turn uses the new world submit route
   - targeted world turn chooses the named backend when multiple world members exist
   - switching world backends is explicit and one-at-a-time
4. Extend [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs) to capture and script the new `/v1/member_turn/stream` route.
5. Add request boundary tests in [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs) for `MemberTurnSubmitRequestV1`.
6. Extend [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs) to cover submitted-turn streaming and cancel.
7. Keep [crates/world-agent/tests/member_runtime_world_placement_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/member_runtime_world_placement_v1.rs) green to prove the registry refactor did not break world placement.
8. Add or extend a regression in [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs) proving `-c` still remains wrap mode after targeted-turn support lands.

### Regression rule for this slice

This slice changes existing REPL parsing behavior. That makes these regression tests mandatory:

1. plain shell input still executes as shell input
2. `:host` and `:pty` still behave exactly as before
3. `substrate -c` still remains wrap mode

These are not optional. They are the highest-priority regression floor for this feature.

## Failure Modes Registry

| Failure mode | Surface | Test coverage required | Handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| malformed `::` syntax | shell parser | yes | reject before shell fallback | clear syntax error |
| backend id not in inventory | selector | yes | fail closed | "unknown backend `<id>`" |
| backend denied by policy | selector | yes | fail closed | deny naming exact backend |
| host runtime missing `uaa_session_id` | host submit | yes | fail closed | runtime unavailable error |
| world target on non-Linux | shell route | yes | fail closed | explicit Linux-only error |
| targeted world backend on stale generation | shell + world-agent | yes | fail closed | stale-generation error |
| participant not in `world-agent` registry | world submit | yes | fail closed | runtime unavailable error |
| concurrent submitted turn for one participant | world submit | yes | fail closed | "turn already in flight" |
| cancel delivered to wrong span class | world cancel | yes | handle both bootstrap and submitted turn spans | correct turn stops |
| plain shell input accidentally intercepted | shell parser | yes, regression | reject targeted parser path | shell behavior preserved |

Critical gap rule for this plan:

no failure mode is allowed to be both untested and silent. Every fail-closed path must either produce a typed transport error or a REPL-visible error message.

## Performance Review

This slice is latency-sensitive but not throughput-sensitive.

### Findings resolved in-plan

1. Each targeted turn spawns a new short-lived resume process. That is acceptable here because targeted turns are human-paced and agent latency dominates process-spawn cost.
2. The plan explicitly avoids a new long-lived daemon or generic multiplexor. That keeps operational complexity down and spends zero extra innovation tokens on infrastructure.
3. The one-at-a-time world-member rule is also a performance guardrail. It prevents hidden concurrency and cross-turn contention in the first landing.

### Performance posture

- no new N+1 style concern exists
- no caching layer is needed
- no new background polling loop is introduced
- the only persistent new memory is the stable `world-agent` member registry entry per active participant plus one submitted-turn entry per active targeted turn

## DX Guardrails

This is developer-facing product work even though it is backend code.

Required operator experience:

1. malformed syntax errors must show the exact accepted format: `::<backend_id> <prompt>`
2. unknown backend errors should suggest `substrate agent list`
3. policy denies must name the exact blocked backend id
4. world-target errors on non-Linux must say Linux-first explicitly
5. submitted-turn status/error lines should include the targeted `backend_id`

This is small stuff. It matters because this feature is pure operator UX.

## Worktree Parallelization Strategy

This plan has real parallelization opportunities after the transport contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze targeted-turn contract | `crates/agent-api-types/`, `crates/world-agent/src/`, `crates/shell/src/repl/` | — |
| Shell parser + selector + host submit path | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | Freeze targeted-turn contract |
| World submit route + member registry refactor | `crates/agent-api-types/`, `crates/agent-api-client/`, `crates/world-agent/src/` | Freeze targeted-turn contract |
| Integration tests + stubs + docs | `crates/shell/tests/`, `crates/world-agent/tests/`, repo docs | Shell parser + selector + host submit path, World submit route + member registry refactor |

### Parallel lanes

- Lane A: Shell parser + selector + host submit path
  - sequential inside the lane because these steps share `crates/shell/src/repl/` and `crates/shell/src/execution/agent_runtime/`
- Lane B: World submit route + member registry refactor
  - sequential inside the lane because these steps share `crates/world-agent/src/` and the new typed request/client contract
- Lane C: Integration tests + stubs + docs
  - starts after A and B because the test stub must know the final request shape

### Execution order

1. Freeze the transport and resume contract.
2. Launch Lane A and Lane B in parallel worktrees.
3. Merge A and B.
4. Run Lane C on top for integration tests, cancel-path proof, and gap-matrix closeout.

### Conflict flags

- Lane A and Lane C both touch `crates/shell/tests/repl_world_first_routing_v1.rs`. Keep that file owned by Lane C after A lands.
- Lane B and Lane C both touch `crates/world-agent/tests/streamed_execute_cancel_v1.rs`. Same rule: B lands runtime changes, C lands final proof coverage.
- `crates/agent-api-types/src/lib.rs` is a contract hotspot. Only one lane should edit it at a time.

### Parallelization verdict

Three workstreams, two parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the targeted-turn contract

Files:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/agent-api-client/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs)
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
- [crates/world-agent/src/handlers.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs)

Deliver:

1. add `MemberTurnSubmitRequestV1`
2. add validation for non-empty prompt and identity fields
3. add `AgentClient::submit_member_turn_stream(...)`
4. add the new `world-agent` route and handler skeleton

Done means the contract compiles and the request round-trip boundary tests exist before any runtime logic is layered on top.

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

Done means the shell can parse and route targeted turns to a scope-specific handler without yet requiring the world path to be implemented.

### Step 3. Implement the shell-local host submitted-turn path

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)

Deliver:

1. resolve the active host runtime for the targeted backend
2. require exact backend match with the retained orchestrator participant
3. build a new `AgentWrapperRunRequest` carrying:
   - the operator prompt
   - current working directory
   - `agent_api.session.resume.v1 = { selector: "id", id: <uaa_session_id> }`
4. translate wrapper events into targeted-turn REPL/trace output
5. surface completion and failures distinctly from shell commands

Done means `::cli:codex hello` works for host-scoped `cli:codex` with no world path involved.

### Step 4. Refactor world-agent member ownership for submitted turns

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
   - streams frames back as NDJSON
5. extend cancel handling so `execute_cancel` or its successor can stop both bootstrap and submitted-turn spans correctly

Done means `world-agent` can resume an already-live member session truthfully instead of only launching it.

### Step 5. Wire the targeted Linux world path in the REPL

Files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. add `ensure_targeted_member_runtime_ready(...)`
2. when the targeted backend differs from the currently retained world member:
   - stop the current retained world member cleanly
   - launch the requested backend through the existing member-dispatch seam
3. submit the targeted prompt through `submit_member_turn_stream(...)`
4. reject non-Linux world targets explicitly
5. preserve current world drift and generation reconciliation behavior

Done means explicit backend targeting actually works end to end for Linux world members, even when multiple world backends are configured.

### Step 6. Close the regression floor and repo truth

Files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/support/repl_world_agent.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/repl_world_agent.rs)
- [crates/world-agent/tests/streamed_execute_cancel_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/streamed_execute_cancel_v1.rs)
- [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

Deliver:

1. targeted-turn integration tests for host and world paths
2. submit-route capture and scripted responses in the test stub
3. cancel-path proof for submitted turns
4. explicit wrap-mode regression for `substrate -c`
5. gap-matrix update that marks explicit targeted turns and real user-turn submission as landed, while keeping broader CLI productization open

Done means the repo documentation tells the truth and the regression floor is real.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-agent member_runtime_world_placement_v1 -- --nocapture
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell invocation -- --nocapture
```

## Definition of Done

1. `::<backend_id> <prompt>` is the only accepted targeted-turn grammar.
2. Plain REPL input still runs as shell input.
3. `:host` and `:pty` still behave as before.
4. Host-targeted turns resume the exact surfaced UAA session id.
5. Linux world-targeted turns go through the new typed `world-agent` submit route.
6. Multiple configured world members no longer block targeted routing when the backend id is explicit.
7. The first landing enforces one retained world-member runtime at a time and makes backend switching explicit.
8. Submitted turns stream output visibly in the REPL and return a distinct completion signal.
9. Submitted-turn cancel works by returned span id.
10. `substrate -c` still remains wrap mode.
11. Tests and gap-matrix docs are updated together.

## Deferred Work

- public `substrate agent start|resume|fork|stop`
- non-interactive targeted prompt surface
- multi-line targeted prompts
- simultaneous retained world members in one REPL session
- cross-platform world-target parity beyond Linux
- transcript persistence and richer session-history UX

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is with one explicit constraint added: one retained world member at a time
- Architecture Review: 4 issues found, all resolved in-plan
- Code Quality Review: 4 issues found, all resolved in-plan
- Test Review: diagram produced, 18 targeted gaps identified
- Performance Review: 3 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, deferred work stays inside this plan
- Failure modes: 0 acceptable silent gaps, 0 unresolved critical gaps after planned coverage lands
- Outside voice: skipped for this document generation
- Parallelization: 3 lanes, 2 parallel / 1 sequential integration lane
- Lake Score: 8/8 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Freeze grammar to `::<backend_id> <prompt>` only | Mechanical | Explicit over clever | One spelling keeps parsing and docs honest | `@backend`, implicit default-agent routing |
| 2 | Architecture | Use UAA resume semantics for submitted turns | Mechanical | DRY | The dependency already supports resume; do not invent a second backend protocol | stdin injection into bootstrap process |
| 3 | Architecture | Add `MemberTurnSubmitRequestV1` instead of overloading `member_dispatch` | Mechanical | Explicit over clever | Launch and submit are different contracts and should stay different | dual-purpose `ExecuteRequest.member_dispatch` |
| 4 | Architecture | Refactor `MemberRuntimeManager` to key active members by participant id | Mechanical | Systems over heroes | Stable participant identity is required for later targeted turns and cancel correctness | launch-span-only registry |
| 5 | Scope | Support one retained world member at a time in this landing | Taste, resolved | Pragmatic | Matches the current shell ownership model and keeps the diff bounded | pretending simultaneous retained members already exist |
| 6 | Test Review | Make plain-shell and `-c` behavior regressions mandatory | Mechanical | Completeness | Parser work is dangerous if the old shell contract is not explicitly proven | relying on manual validation |
