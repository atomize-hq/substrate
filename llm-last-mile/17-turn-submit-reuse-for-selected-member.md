# SOW B: Finish Turn Submit + Reuse For Selected Member

Status: implementation-oriented split draft. This document narrows the second half of [15-targeted-repl-agent-turns-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/15-targeted-repl-agent-turns-linux-first.md:1). It does not re-open the already-landed REPL grammar or claim user-turn submit is absent. Instead, it defines the remaining work needed to finish, harden, and deliberately surface selected-backend follow-up turn submission and reuse for the current REPL-first caller path, while explicitly keeping new CLI submit surfaces out of scope.

## Objective

Keep the current Linux-first REPL targeted-turn path intact, then finish the remaining product work so that selected-backend follow-up turns:

- submit real user prompts into the retained active session for the selected backend,
- reuse the retained world member when that member is still valid for the selected backend and world generation,
- relaunch only when the retained runtime is unavailable or stale,
- keep one coherent submit/reuse contract across the current host-targeted and world-targeted REPL paths,
- and fail closed with clear operator-facing errors and docs.

This SOW is intentionally narrow. It is about finishing submit/reuse semantics for the selected member, not redesigning orchestration wholesale.

## Why This Is Needed

The repo is already past the "can we submit turns at all?" stage.

What is already landed:

- the REPL recognizes exact single-line targeted syntax and routes it before shell fallback in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:683) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4550)
- host follow-up turns already resume the retained active orchestrator session in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3781)
- Linux world follow-up turns already go through a typed submit route in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3889), [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1472), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:232)
- the world submit request shape and identity contract already exist in [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:836)
- current Linux tests already prove grammar rejection, typed world submit, host fail-closed routing, and same-generation member reuse in [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1776), [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1841), [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1964), and [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:2029)

What is still incomplete:

- the product surface is still REPL-first; the gap matrix still records non-interactive caller work as open in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:106)
- exact backend targeting is only partially productized; the matrix still describes it as REPL-first in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:114)
- `substrate -c` is still shell wrap mode in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:27) and [plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:620), so this slice keeps CLI submit parity out of scope rather than reopening command-mode semantics
- world-targeted follow-up turns still fail closed off Linux in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4014), and the gap matrix still tracks macOS/Lima parity as open in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:118)
- active-session resolution outside the current REPL path still assumes a single live orchestrator session in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433), and operator surfaces such as `agent status` / toolbox still preflight that single-session model in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421) and [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:972)
- the operator-facing contract is spread across code paths and tests, but not yet fully tightened as a deliberate user-facing submit/reuse surface

This SOW exists to finish that remaining layer without pretending the current Linux-first REPL path does not already work.

## Relationship To Existing Docs

- [15-targeted-repl-agent-turns-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/15-targeted-repl-agent-turns-linux-first.md:1) is the broader combined slice
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:105) now records the repo truth that targeted REPL grammar and Linux-first turn submission are already landed, while broader caller-surface and parity work remains open

This document should be read as "SOW B after the initial Linux-first REPL submit path landed."

## Current Relevant Code Surfaces

### Targeted REPL parse and route entry points

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:683)
  - recognizes `::...` input before shell fallback
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4550)
  - enforces exact `::<backend_id> <prompt>` parsing on a single line
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2704)
  - resolves targeted turns by exact backend id and fails closed when the requested host backend is not the active orchestrator backend

### Current submit paths

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3781)
  - host follow-up submit path using session-resume extensions against the retained host runtime
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3889)
  - Linux world follow-up submit path shaping `MemberTurnSubmitRequestV1`
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1472)
  - typed `/v1/member_turn/stream` handoff into `MemberRuntimeManager`
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:232)
  - retained-member submit path that resumes the surfaced UAA session id

### Reuse and relaunch decision points

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3527)
  - generation reconciliation for stale retained members
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3577)
  - ensure-ready path that reuses an existing member runtime when the descriptor and world binding are still valid
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:444)
  - rejects concurrent submitted turns against one retained member slot

### Request shape and identity validation

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:836)
  - `MemberTurnSubmitRequestV1`
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:803)
  - retained identity drift checks for orchestration session id, participant ids, backend id, world id, and world generation

### Caller surfaces that still constrain the remaining work

- [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:27)
  - `-c/--command` still means shell command execution
- [crates/shell/src/execution/invocation/plan.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/plan.rs:620)
  - non-interactive command mode still maps to `ShellMode::Wrap`
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433)
  - current single-live-session resolution helper
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:421)
  - status still preflights a single live session
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:972)
  - toolbox status also assumes one selected live orchestrator session

## In Scope

- finish and harden selected-backend follow-up turn submission on top of the already-landed REPL Linux-first path
- define exact reuse-vs-relaunch behavior for follow-up turns to the selected world member
- make submit/reuse semantics explicit and consistent across the current REPL host-targeted and world-targeted paths
- tighten operator-facing errors for malformed syntax, bad backend selection, stale/missing retained state, unsupported platform paths, and concurrent submit collisions
- update the operator docs that describe the contract, current Linux-first limitations, and any new submit surface
  - for this slice, "new submit surface" means the already-landed REPL targeted-turn path plus its operator docs and help text, not a new CLI caller

## Out Of Scope

- redesigning plain REPL input into implicit default-agent routing
- changing `substrate -c` semantics by implication
- broader `substrate agent start|resume|fork|stop` productization
- replacing the retained-runtime ownership split between shell-owned host control and world-owned member control
- full toolbox product work
- broad status-surface redesign unrelated to submit/reuse
- cross-platform parity beyond what this slice needs to document and fail closed around

## Required Semantics And Invariants

### 1. Do not regress the landed REPL contract

The current exact grammar and shell-first fallback posture must remain true.

- `::<backend_id> <prompt>` stays the only targeted follow-up grammar for the REPL
- malformed targeted syntax must fail before shell fallback
- plain REPL input remains shell execution

### 2. Follow-up submit must reuse retained ownership when valid

For a selected backend that already has a valid retained runtime:

- host follow-up turns must resume the retained host orchestrator session rather than relaunching the orchestrator
- Linux world follow-up turns must submit into the retained selected member rather than relaunching a sibling member
- successful follow-up submit must not swap participant identity or create duplicate live members

### 3. Relaunch is allowed only for real invalidation or drift

Relaunch is correct only when the retained runtime is no longer valid for the requested turn.

- if the retained member is missing, invalidated, or stale for the current `world_generation`, the shell may re-ready the member before submit
- same-generation follow-up turns must prefer reuse
- submit itself must not be the trigger that silently creates unnecessary replacement members

### 4. Exact backend selection remains mandatory

- backend routing must remain by exact `backend_id`
- host follow-up turns may only target the active orchestrator backend for the current REPL session
- world follow-up turns must not revert to "pick the one eligible member" heuristics once a backend was explicitly named

### 5. State authority stays in the existing runtime store and retained handles

- active orchestration and participant identity stays in the existing state store under `~/.substrate/run/agent-hub/...`
- host follow-up submit continues to use the retained surfaced UAA session handle
- world follow-up submit continues to use the persisted participant/world identity plus the retained surfaced UAA session id inside `world-agent`

### 6. No new CLI caller surface is introduced in this slice

- the submit/reuse hardening work in this SOW applies to the current REPL targeted-turn paths only
- `substrate -c` remains shell wrap mode
- any later non-REPL caller must be a separate slice with its own explicit session-selection contract when more than one live orchestration session exists

### 7. Operator-facing failures must be deliberate and explainable

Errors should stay precise and fail closed. Relevant existing examples already exist in code:

- malformed targeted syntax in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:685)
- wrong active host backend in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2724)
- non-Linux world-targeted follow-up rejection in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4021)
- missing retained member / missing surfaced session id / active submitted turn collision in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:241), [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:249), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:444)

This slice should standardize how those errors surface to operators across the current REPL targeted-turn paths rather than leaving them as implementation accidents.

## Recommended Implementation Shape

1. Treat the current REPL Linux-first path as the reference contract, not as throwaway scaffolding.

2. Factor the shared selected-backend submit/reuse logic behind one internal helper seam so the host-targeted and world-targeted REPL paths do not fork their own route-selection, readiness, and submit behavior.

3. Preserve the current ownership split:
   - host follow-up submits stay shell-owned and session-resume based
   - world follow-up submits stay `world-agent`-owned and use the typed member-turn route

4. Keep CLI parity out of scope for this slice instead of changing `-c` by stealth.
   - `substrate -c` stays wrap mode.
   - If a later slice introduces a CLI submit surface, it should be explicit and should require a deterministic active-session selector when ambiguity exists.

5. Tighten reuse-vs-relaunch bookkeeping around the already-landed ready/reconcile paths.
   - reuse the retained member when descriptor, backend id, world id, and world generation still match
   - relaunch only after invalidation, drift, or missing runtime state
   - keep submitted-turn collision behavior explicit instead of letting overlapping calls race

6. Update docs and help text so operators can discover:
   - the exact targeted-turn grammar
   - the Linux-first scope of world-member follow-up submit/reuse
   - the explicit error cases and what action the operator should take next

## Concrete Work Breakdown

1. Codify the current selected-turn contract.
   - Document the already-landed REPL behavior and Linux-first retained-member submit path.
   - Remove any remaining stale doc language that still implies submit is wholly missing.

2. Consolidate shared submit/reuse orchestration.
   - Centralize exact backend routing, retained-runtime readiness checks, and host-vs-world submit branching.
   - Keep REPL wiring thin around that shared core.

3. Tighten the REPL-only contract instead of widening the caller set.
   - Keep `-c` unchanged.
   - Remove stale doc language that implies a new CLI submit surface is part of this landing.

4. Harden reuse and collision behavior.
   - Make reuse the default for same-generation follow-up turns.
   - Preserve fail-closed behavior on identity drift, missing retained session ids, and concurrent submitted turns.

5. Finish operator-facing docs and errors.
   - Update [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:105) and any help text or operator docs touched by the new surface.
   - Ensure surfaced errors describe whether the operator should retry, target a different backend, restart the REPL/session, or switch to Linux for world follow-up turns.

## Validation Expectations

Existing regression coverage that must keep passing:

- [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1776)
  - malformed targeted syntax fails before shell fallback
- [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1841)
  - world targeted follow-up turns use typed submit without relaunching the member
- [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:1964)
  - host targeted follow-up turns reject non-active host backends
- [repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs:2029)
  - same-generation world commands reuse the retained member runtime

Additional validation expected for this slice:

- tests that operator-visible errors remain fail-closed and do not fall back to shell execution
- regression coverage for concurrent submitted-turn rejection and identity-drift rejection
- coverage that non-Linux paths keep the explicit fail-closed posture for world-targeted follow-up turns until parity is intentionally landed

Recommended commands:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent -- --nocapture
cargo test -p agent-api-types -- --nocapture
```

## Explicit Non-Goals

- Do not claim the repo still lacks targeted REPL follow-up submit or Linux retained-member reuse.
- Do not silently broaden this slice into a general orchestration control-plane redesign.
- Do not convert shell command surfaces into agent caller surfaces without an explicit contract change.
- Do not promise non-Linux world-member follow-up turn parity before the underlying backend path is actually wired.
