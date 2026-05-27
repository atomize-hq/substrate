# SOW: Targeted REPL Agent Turns, Linux-First

Status: implementation-oriented draft. This SOW defines the next landing after the Linux-first member-runtime placement and gateway-auth slices. It is intentionally narrow: add one explicit REPL caller grammar for targeted agent turns, route those turns by named `backend_id`, submit a real user prompt by lazily launching the selected runtime on first targeted use or reusing it when already live, and stream the result back through the REPL. It does not redesign `substrate -c`, and it does not productize a broader `substrate agent start|resume|fork|stop` command family.

## Objective

Land one Linux-first production path where the interactive REPL can:

- parse an explicit targeted agent-turn syntax,
- resolve that syntax to a named backend such as `cli:codex`,
- require routing by `backend_id` instead of "the one eligible member,"
- submit the user’s prompt by lazily launching the selected runtime on first targeted use or reusing it when already live,
- and stream the resulting agent output back through the REPL without collapsing the input into normal shell execution.

The preferred syntax decision for this slice is:

- `::cli:codex <prompt>`
- generalized as `::<backend_id> <prompt>`

This slice stays REPL-only. Non-interactive `substrate -c` remains shell wrap mode and is intentionally out of scope.

## Why This Is Needed

The repo now has most of the runtime control plane needed for a first targeted-agent caller path, but the operator-facing turn-submission seam is still missing.

What already exists:

- the REPL starts a shell-owned orchestrator runtime through UAA in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1729) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1848)
- that startup path persists authoritative orchestration and participant state in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:100)
- backend inventory already derives canonical `backend_id` values such as `cli:codex` in [agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:105)
- member-runtime selection, realizability checks, and policy gating already exist in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:30)
- Linux member runtimes already launch through `world-agent` over the host<->world seam in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3364), [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2840), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:54)
- for active world-scoped members, the retained control handle is owned inside `world-agent`, not the shell, because `MemberRuntimeManager` stores the active cancel handle in its own registry in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:38) through [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:157)
- the gap matrix now explicitly records that invocation grammar, user-turn submission, and explicit backend targeting are still open in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:86), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:106), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:112), and [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:113)

What is still missing:

- the REPL only recognizes shell directives like `:host` and `:pty`; normal input still falls through to shell execution in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:669) through [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:930)
- both orchestrator and member runtimes are currently started with a fixed bootstrap prompt rather than a real user turn in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1622), [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1932), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:68)
- there is no repo-visible path after startup that submits an arbitrary user prompt into the active retained UAA session; current `agent_api` usage in the shell and `world-agent` is bootstrap-only `run_control(...)`
- the current shell<->world seam for active members is limited to remote launch, streamed events/output, and cancel:
  - the shell starts a member through `execute_stream(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2921)
  - `world-agent` branches that request into `member_runtime.launch(...)` in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1233) through [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1266)
  - later cancellation goes back through `execute_cancel(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2827) and [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1465)
  - there is no follow-on request seam today for "submit this user turn into the already-live member session"
- world-member selection still fails closed when more than one eligible member exists instead of routing by a named backend in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:161) through [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:235)
- `substrate -c` is still explicitly planned and tested as shell wrap mode in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:27), [plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:29), [plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:616), and [tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs:56)
- the public CLI still exposes `agent list|status|doctor|toolbox` only; there is no first-class `start|resume|fork|stop` namespace in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:399) through [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:480)

That makes the current runtime honest about startup ownership, but not yet useful as a user-facing targeted turn surface.

## Relationship To Existing Slices

This SOW consumes the already-landed runtime slices and does not reopen them:

- [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md) established the need for a real member runtime lifecycle seam
- [13-member-runtime-world-placement-gap-sow.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/13-member-runtime-world-placement-gap-sow.md) bounded the Linux-first in-world placement hardening
- [14-secret-handoff-into-the-world-gateway.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/14-secret-handoff-into-the-world-gateway.md) bounded the gateway auth-carrier fix and is not a substitute for pure-agent turn submission
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:170) already recommends the next runtime slice as explicit caller grammar plus real user-turn submission

This slice should be treated as the first operator-facing caller seam on top of the landed REPL-owned orchestrator and Linux-first world-member runtime.

## Current Relevant Code Surfaces

### REPL parsing and current shell-first behavior

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:669)
  - rejects bare `:host` and `:pty`
  - handles `:host <command>`
  - handles `:pty <command>`
  - otherwise executes the input as a shell command in host or world scope
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4095)
  - already centralizes the "embedded newlines disable directive parsing" rule

This is the insertion point for explicit agent-turn grammar.

### Runtime startup and current bootstrap-only UAA ownership

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1729)
  - prepares orchestrator selection, inventory, policy gating, and state-store setup
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1932)
  - starts the orchestrator with `AgentWrapperGateway.run_control(...)` and the fixed bootstrap prompt
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2663)
  - prepares a member runtime manifest with authoritative world binding
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2840)
  - starts the Linux member runtime over the remote member-dispatch seam
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:68)
  - also uses `run_control(...)` with the same fixed member bootstrap prompt

These paths prove retained-control runtime ownership exists already. They do not provide a user-turn submission call after startup, and for world-scoped members the retained control owner is `world-agent`, not the shell.

### Current member control seam is launch, stream, and cancel only

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1233)
  - receives `member_dispatch` on `execute_stream(...)`
  - launches the active member runtime and returns an NDJSON event stream
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1465)
  - receives `execute_cancel(...)` and routes it to `member_runtime.cancel(...)`
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:91)
  - stores the active runtime in the `world-agent` process, including the cancel handle

That means Linux world-member targeted turns cannot be solved by a shell-only submission helper. The shell does not own the live member control handle after launch, and there is no current request seam for follow-on turn submission into that already-live member session.

### Backend identity, routing, and current ambiguity posture

- [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:105)
  - derives canonical backend ids from inventory entries
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:12)
  - already carries `backend_id` on `RuntimeSelectionDescriptor`
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:161)
  - selects world members by "collect every eligible world-scoped member"
- [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:229)
  - fails closed when more than one eligible member exists
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2518)
  - still calls the ambiguity-based member selector rather than a named-backend selector

This is the exact seam that must change from eligibility-only routing to explicit `backend_id` routing.

### Active-session state and public status surface

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433)
  - already has `resolve_single_live_session_for_agent(...)`
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421)
  - status preflights `resolve_single_live_session_for_agent(...)`
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:971)
  - toolbox also depends on a single live orchestrator session

Those surfaces matter for context, but this SOW does not require a full status redesign. It only needs enough runtime identity to send a targeted REPL turn into the selected active session.

### `substrate -c` and `substrate agent` posture that stays out of scope

- [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:27)
  - `-c/--command` is documented as "Execute a single command"
- [crates/shell/src/execution/invocation/plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:29)
  - `ShellMode::Wrap(String)` is still the `-c` mode
- [crates/shell/src/execution/invocation/plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:619)
  - `cli.command` maps directly to `ShellMode::Wrap(cmd)`
- [crates/shell/src/execution/invocation/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/tests.rs:56)
  - existing tests enforce wrap-mode semantics
- [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:471)
  - `agent` subcommands still stop at `list|status|doctor|toolbox`

This slice should reference those facts, not fight them.

## In Scope

- add explicit REPL grammar for targeted agent turns
- prefer `::cli:codex <prompt>` and support generalized `::<backend_id> <prompt>`
- route targeted turns by named `backend_id`
- add one real prompt-submission path into the active retained UAA session for the selected backend
- stream the submitted turn’s result back through the REPL
- keep the first landing Linux-first for world-scoped member turns
- allow host-scoped targeted turns only if they reuse the already-live shell-owned orchestrator runtime without widening product scope
- add one additive shell<->world-agent turn-submission seam for already-live Linux world-member runtimes
- add the minimum runtime/session bookkeeping needed to correlate a submitted turn with the selected active session
- add focused tests for grammar parsing, named-backend routing, and fail-closed behavior

## Out Of Scope

- redesigning `substrate -c` or changing `ShellMode::Wrap`
- adding non-interactive agent invocation syntax
- productizing `substrate agent start|resume|fork|stop`
- inventing default-agent implicit routing for plain REPL input
- replacing the shell/world transport seam for member runtimes
- toolbox work
- gateway auth-carrier work
- cross-platform parity promises beyond an explicit Linux-first implementation
- broad session-resume or session-fork UX beyond the minimum context this document needs to mention
- toolbox, gateway, or nested-LLM paths as substitutes for pure-agent targeted turns

## Required Semantics And Invariants

### 1. Agent turns must be explicit in the REPL

Normal REPL input must remain shell execution. Agent turns must require explicit syntax.

Required rule:

- only inputs matching `::<backend_id> <prompt>` enter the targeted-agent path
- plain shell input must keep its current shell/world execution behavior
- `:host` and `:pty` keep their existing meanings

This avoids silently changing the semantics of an existing shell-first REPL.

### 2. Backend routing must be by named `backend_id`, not by eligible-member heuristics

The targeted-agent caller surface must resolve exactly the backend the user named.

Required rule:

- the parser extracts a concrete `backend_id`
- runtime selection uses inventory lookup plus policy validation for that backend
- the implementation must not reuse `validate_member_selection(...)` as the final routing rule for explicit agent turns

When multiple world-scoped members are configured, the targeted-turn path must not ask the runtime to "pick one." It must route to the named backend or fail closed.

### 3. Host vs world placement still follows inventory scope

Named backend routing does not erase the existing placement model.

Required rule:

- if the selected backend resolves to `execution.scope=host`, route to the active host-scoped orchestrator-side runtime only if that runtime matches the targeted backend contract for this slice
- if the selected backend resolves to `execution.scope=world`, require the Linux world-member runtime path and authoritative world binding already established by the REPL session
- no host fallback is allowed for a targeted world-scoped backend

This keeps the new caller grammar aligned with the repo’s existing execution-scope model.

### 4. A real user turn must be submitted after bootstrap

This slice must stop at least one path from being bootstrap-prompt-only.

Required rule:

- the targeted REPL path must send the user’s actual prompt into the active selected UAA session after startup
- the submitted prompt must not be represented by re-running the runtime bootstrap prompt
- the shell must preserve the existing bootstrap prompt only for startup ownership, not for operator turns

This is the central functional gap the slice must close.

### 5. REPL streaming must stay attached and operator-visible

The targeted turn must stream output back through the interactive REPL rather than waiting for a detached snapshot.

Required rule:

- the operator sees streamed response events in the REPL
- streamed output must be correlated to the targeted backend and active session
- cancellation and terminal completion must surface through the same REPL control lane

The first pass does not need a polished transcript model, but it must be a real streamed interaction.

### 6. Host-scoped and world-scoped submission ownership must stay distinct

The submission path is not symmetric across scopes in the current repo.

Required rule:

- host-scoped targeted turns may use a shell-local submission lane because the shell owns the live host orchestrator runtime after startup
- world-scoped targeted turns must use a remote submission lane owned by `world-agent` because `world-agent` owns the active retained member runtime after launch
- the Linux world-member solution must add an additive shell<->world-agent request seam for turn submission into an already-live member session
- this seam must stay bounded to active member-turn submission and must not broaden into a generic new hub service

### 7. Runtime state must stay authoritative

The new targeted-turn seam must consume the current runtime authority model rather than inventing a parallel one.

Required rule:

- live session/participant identity stays in the existing runtime store under `~/.substrate/run/agent-hub/sessions/...`
- the targeted-turn path resolves the selected active session from authoritative live state
- no new ad hoc in-memory-only session registry becomes the source of truth

### 8. `substrate -c` remains out of scope

This slice must not quietly broaden into non-interactive agent execution.

Required rule:

- `substrate -c` remains shell wrap mode exactly as it is today
- no reuse of `-c` for agent prompts in this landing
- any later non-interactive agent surface must be handled by a separate slice

### 9. Public agent-command productization remains deferred

This slice can mention future `start|resume|fork|stop` surfaces for context, but it must not block on them.

Required rule:

- do not require a new public `substrate agent start|resume|fork|stop` family to land targeted REPL turns
- do not redesign the `agent` CLI namespace as part of this work

## Recommended Implementation Shape

### 1. Add a dedicated REPL parse step for targeted agent turns

Add a small explicit parser in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) near the existing directive handling.

Recommended behavior:

- run before shell execution fallback
- ignore multi-line inputs for the first pass, matching the existing directive posture around [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4095)
- parse `::<backend_id> <prompt>`
- reject missing backend ids
- reject missing prompts
- produce clear REPL-facing errors for malformed syntax

The parser does not need to support alternate spellings in this slice. The preferred contract should be explicit and narrow.

### 2. Add explicit backend-id selection helpers

Introduce a selector next to the current validator helpers that resolves one named backend from effective inventory and policy.

The helper should:

- validate the requested `backend_id` format
- resolve the matching inventory entry
- confirm the derived backend id exactly matches the requested backend id
- enforce `PURE_AGENT_PROTOCOL`
- reuse realizability checks from [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:112)
- enforce policy allowlisting through `backend_allowed(...)`
- return a `RuntimeSelectionDescriptor`

This should be additive, not a rewrite of the current ambiguity-based helper. Existing startup flows can keep using `validate_member_selection(...)` until a later slice says otherwise.

### 3. Separate "runtime startup" from "turn submission"

The current code path conflates runtime usefulness with bootstrap ownership. This slice should introduce a second operation after startup: submit a turn into an already-live selected runtime.

Recommended shape:

- keep current orchestrator/member startup flows for retained ownership
- add one shell-local submission function for host-scoped targeted turns that accepts:
  - selected `RuntimeSelectionDescriptor`
  - authoritative live participant/session identity
  - prompt text
  - REPL printer / telemetry handles
- add one additive shell<->world-agent submission request for Linux world-scoped targeted turns that addresses an already-live member runtime rather than launching a new one
- route both paths through already-live runtime ownership rather than by spawning a fresh runtime per prompt

The repo does not currently show any post-bootstrap active-session submission usage of `agent_api`; it shows bootstrap `run_control(...)` usage only. This slice must therefore add the narrowest wrappers and request seams needed to surface real post-bootstrap turn submission rather than implying that the capability is already exercised here.

### 4. Add runtime resolution by backend and scope

The targeted-turn lane needs to map a named backend to the active runtime that owns that backend in the current REPL session.

Recommended behavior:

- for host scope:
  - ensure the active runtime exists
  - ensure it matches the requested backend id
  - submit the turn through a shell-local active-runtime lane
  - fail closed if the selected backend is not the active host runtime for this REPL session
- for Linux world scope:
  - ensure authoritative world binding exists
  - ensure the member runtime for that backend is ready for the active `world_generation`
  - start it through the existing Linux member path if needed
  - submit the turn through a narrow shell<->world-agent request addressed to the already-live member runtime that `world-agent` owns
  - fail closed on non-Linux platforms

This keeps routing honest without requiring a full multi-backend scheduler.

### 5. Add a narrow remote turn-submission seam for active Linux member runtimes

The Linux world-member path needs one additive control request beyond today’s launch/event/cancel contract.

Recommended shape:

- add one internal shell<->world-agent request that means "submit this user turn into the already-live member runtime identified by the current orchestration session, participant/runtime identity, and backend"
- keep the request bounded to active member-turn submission; do not broaden it into a generic new agent hub or reusable multi-purpose runtime service
- reuse the existing remote ownership model in `world-agent` rather than copying member session control back into the shell
- return streamed output over the same narrow request lane so the REPL can render the response live

This is the missing Linux-first world-member seam. Without it, explicit backend parsing still cannot drive a real targeted user turn into an already-live in-world member session.

### 6. Stream targeted-turn output through the REPL

The submission lane should reuse the existing event/printing discipline already used for runtime startup and world-command passthrough where possible.

Minimum expected behavior:

- print backend-targeted progress/errors to the REPL
- stream agent output incrementally
- surface completion or failure distinctly from shell command completion
- keep runtime-owned cancellation semantics intact

This slice does not need a final transcript persistence design, but it should not reduce the interaction to a one-shot buffered response if the active UAA session can stream.

### 7. Keep world-member routing Linux-first and fail-closed elsewhere

The repo already encodes Linux-only member runtime readiness in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3430).

Required posture:

- world-scoped targeted turns are supported only on Linux in this slice
- non-Linux targeted world-member turns fail closed with a clear error
- the SOW must not promise macOS or Windows parity

## Concrete Work Breakdown

1. Add a targeted-turn parser and handler in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) before shell fallback execution.

2. Add named-backend resolution helpers in [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) so explicit REPL turns can select one backend deterministically.

3. Add a shell-side active-runtime resolution layer that maps the named backend plus scope to:
   - the active host runtime, or
   - the active Linux member runtime for the current `world_generation`

4. Add a shell-local user-turn submission primitive for the active host-scoped runtime.

5. Add one additive shell<->world-agent turn-submission seam for already-live Linux member runtimes.

6. Add REPL streaming for submitted turn output and terminal completion across both the shell-local host path and the remote world-member path.

7. Add focused tests for:
   - grammar parsing
   - malformed syntax
   - unknown backend ids
   - denied backends
   - multiple eligible members with explicit backend routing
   - proof that the world-member path uses a remote submission seam rather than a shell-only helper
   - Linux-only world-member targeting
   - proof that plain REPL input still executes as shell input
   - proof that `substrate -c` still stays in wrap mode

## Validation Expectations

At minimum, this slice should add or update tests that prove:

- `:host` and `:pty` still behave as they do today
- `::cli:codex hello` is not treated as a shell command
- targeted routing uses the named backend id rather than the ambiguity-based member selector
- the selected active host runtime receives a real user prompt after bootstrap through the shell-local submission lane
- the selected active Linux world-member runtime receives a real user prompt after bootstrap through the additive shell<->world-agent submission seam
- Linux world-member targeted turns fail closed when authoritative world binding or active member runtime cannot be established
- non-Linux world-member targeted turns fail closed with an explicit Linux-only error
- `substrate -c` still maps to `ShellMode::Wrap`

If the implementation needs a fake CLI backend fixture to prove post-bootstrap submission, it should add the narrowest test harness necessary rather than widening the product surface.

## Open Questions To Resolve During Implementation

1. Which exact post-bootstrap turn-submission primitive should the repo standardize on for the host path, given that current repo usage only shows bootstrap `run_control(...)` and does not yet exercise active-session submission?
   - This must be resolved explicitly in code rather than implied by the SOW.

2. Should host-scoped targeted turns be limited to the selected orchestrator backend only in the first pass?
   - Current repo reality only guarantees one live host-scoped orchestrator runtime per REPL session, so widening to arbitrary additional host backends should not be assumed.

3. What is the smallest request/response shape the repo should add between the shell and `world-agent` for active member-turn submission without turning it into a generic new runtime service?
   - The existing Linux member seam is launch, stream, and cancel only, so this needs one explicit additive contract.

4. Should the first pass reject multi-line `::<backend_id>` prompts entirely?
   - The current REPL directive model already treats embedded newlines specially, so explicit rejection is safer than silent partial parsing.

5. How much new trace or `AgentEvent` structure is actually required for submitted user turns?
   - This SOW expects real streamed interaction, but it does not require a final message-history schema if existing event machinery is sufficient for the first landing.

## Non-Goals Repeated Explicitly

- no `substrate -c` redesign
- no broader public `substrate agent start|resume|fork|stop` productization
- no toolbox substitution for targeted agent turns
- no cross-platform parity commitment beyond Linux-first world-member support
- no implicit "default backend" shell-to-agent conversion for ordinary REPL input
