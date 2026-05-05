# SOW: Public Agent Control Surfaces

Status: implementation-oriented follow-on draft. This SOW is the narrow public control-plane follow-on after [18-status-surface-and-session-handle-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/18-status-surface-and-session-handle-hardening.md:3). It defines the minimal public `substrate agent start|resume|fork|stop` family, the selector contract those commands must use, and the fail-closed posture for Linux-first world-bound work. It does not absorb the preceding status/session-handle hardening slice, it does not redesign `substrate -c`, and it does not redesign the REPL `::<backend_id> <prompt>` grammar.

## Objective

Expose one narrow public control surface under `substrate agent` that:

- makes session creation and shutdown first-class instead of REPL-only side effects,
- reserves explicit public entry points for `resume` and `fork` instead of leaving them implied,
- uses `orchestration_session_id` as the only public session handle,
- keeps backend-native session ids internal,
- and stays fail-closed anywhere the current runtime cannot prove ownership, linkage, platform posture, or policy allowability.

This slice is about public control-surface shape and bounded productization of already-landed runtime pieces. It is not a daemonization project and not a general hub redesign.

## Current Repo Truth

The current repo already proves most of the internal control-plane substrate:

- the public CLI still stops at inspection surfaces. `AgentAction` exposes `List`, `Status`, `Doctor`, and `Toolbox` only in [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:471)
- host-runtime startup already exists internally in `start_host_orchestrator_runtime(...)` and its prepared variant in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1802) and [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1930)
- internal shutdown already exists in `shutdown_host_orchestrator_runtime(...)` in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4090)
- session-resume extension shaping already exists in `AGENT_API_SESSION_RESUME_V1` and `build_session_resume_extension(...)` in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1572) and [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3776)
- inventory capability flags already model `session_start`, `session_resume`, `session_fork`, and `session_stop` in [crates/shell/src/execution/agent_inventory.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:89) and [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:99)
- authoritative session persistence already uses `orchestration_session_id` at the parent-session layer in [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:25)
- surfaced backend-native session ids are still stored separately as internal `uaa_session_id` in [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81)
- the gap matrix already says the surface is not yet public and should be treated as follow-on work after the current status/session-handle hardening concerns in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:104), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:110), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:112), and [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:179)

The honest summary is:

- `start` and `stop` are partially landed internally because startup, attached control retention, persistence, and shutdown plumbing already exist inside the REPL-owned runtime,
- `resume` and `fork` have partial ingredients such as capability flags and session-resume extension wiring, but they are not yet public Substrate control-plane actions,
- and the current control owner is still the long-lived shell process rather than a reusable daemon, as the gap matrix explicitly notes in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:108).

## Scope

This SOW defines only the minimal public command family:

```text
substrate agent start --backend <backend_id> [--json]
substrate agent resume --session <orchestration_session_id> [--json]
substrate agent fork --session <orchestration_session_id> [--json]
substrate agent stop --session <orchestration_session_id> [--json]
```

The command family is intentionally narrow:

- `start` creates one orchestration session for one exact backend
- `resume` attempts to re-enter one existing orchestration session
- `fork` creates a successor orchestration session from one existing orchestration session
- `stop` requests authoritative shutdown of one existing orchestration session

No command in this slice takes a prompt payload. Prompt submission remains the existing REPL targeted-turn surface from [15-targeted-repl-agent-turns-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/15-targeted-repl-agent-turns-linux-first.md:3) and its follow-on hardening work, not this public control-plane slice.

## Public Selector Contract

The public selector contract must stay boring and exact.

### 1. `orchestration_session_id` is the only public session handle

For any command that targets an existing session, the only accepted public selector is:

- `--session <orchestration_session_id>`

That id already exists as the parent-session authority in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:25) and is the cleanest durable Substrate-owned handle.

This slice must not accept any of these as the primary public selector:

- `participant_id`
- `active_session_handle_id`
- `session_handle_id`
- `uaa_session_id`
- raw backend-native thread or run ids

### 2. `uaa_session_id` stays internal

The surfaced backend-native session id remains an internal runtime detail:

- it is persisted as `internal.uaa_session_id` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81)
- it is already used internally when the REPL submits a resumed host follow-up turn through `build_session_resume_extension(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3776)

Public control commands must resolve from `orchestration_session_id` to any needed participant record or internal `uaa_session_id` through the state store. They must not ask operators to provide backend-native handles.

### 3. `start` uses exact `backend_id`

`start` must require:

- `--backend <backend_id>`

and that selector must be an exact derived `backend_id`, not `agent_id`, not a display label, and not "the default agent."

This follows the already-landed exact backend targeting posture from [16-explicit-backend-selection-by-backend-id.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/16-explicit-backend-selection-by-backend-id.md:14) and avoids reopening the unresolved broader routing/default-selection question called out in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:90).

### 4. No fuzzy or "latest" resolution

This slice must not introduce:

- prefix matching on session ids,
- newest-session wins behavior,
- backend inference from agent id,
- or automatic fallthrough to the "only active session I found."

Every selector path must be exact or fail closed.

## Landing Posture

Implementation order should follow the repo truth rather than pretend all four verbs are equally ready.

### Phase A: publicize the already-landed internals for `start` and `stop`

This phase should:

- add public CLI entries for `start` and `stop` under `AgentAction`
- reuse the existing startup and shutdown plumbing in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1802) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4090)
- emit `orchestration_session_id` as the public handle on success
- keep stop tied to authoritative-live ownership only

This is public productization of internal start/stop, not net-new runtime invention.

### Phase B: expose `resume` and `fork` on the same selector contract

This phase should:

- add public CLI entries for `resume` and `fork`
- resolve the selected session through the authoritative state store, not through trace history
- reuse internal session-resume wiring only behind the `orchestration_session_id` lookup boundary
- fail closed anywhere the runtime cannot prove that the selected session still has a valid active parent, a valid retained control owner, and any required world linkage

This phase must stay honest that `resume` and `fork` are not public today and must not be papered over with "best effort" heuristics.

## Linux-First World Posture

Where the selected session is purely host-scoped orchestrator work, the command family can follow the existing host runtime path.

Where the selected session depends on world-member posture, this slice stays Linux-first:

- if a command requires authoritative shared-world ownership, current `world_generation`, or retained world-member reuse, it must only advertise success on the Linux path that already persists and validates those bindings
- macOS/Lima and Windows/WSL must fail closed for any control action that would require the not-yet-equivalent shared-world/member-runtime contract described as still missing in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:115) and [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:117)
- public help text and error messages must say `unsupported_platform_or_posture` instead of implying parity that does not exist

This slice must not weaken the existing Linux-first truth just to make the public CLI look symmetrical across platforms.

## Fail-Closed Rules

The public control surface must fail closed in these cases:

1. Ambiguity
   - more than one candidate session matches
   - more than one candidate backend matches
   - or a selected session contains conflicting live ownership state
2. Stale linkage
   - the selected parent points at a missing or inactive selected participant
   - the selected participant points at an inactive or missing parent
   - or the selected world member is tied to stale world binding or stale world generation
3. Missing active parent
   - the chosen `orchestration_session_id` exists historically but is no longer active enough to authorize control-plane actions
4. Unsupported platform or posture
   - the selected action would require world-member control semantics that are not implemented on the current platform
5. World-boundary unavailable
   - the action requires current world ownership, socket reachability, or generation continuity and the shell cannot prove it
6. Policy disallow
   - the broker or runtime validator rejects the backend, scope, or action even though inventory advertises the capability bit

The preceding status/session-handle slice already points at the right read-side authority helpers for this: `list_live_sessions()` and `resolve_single_live_session_for_agent(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:421) and [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:433). This slice should consume that hardened authority model, not redefine it.

## Non-Goals

This SOW explicitly does not include:

- absorbing or partially redoing [18-status-surface-and-session-handle-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/18-status-surface-and-session-handle-hardening.md:3)
- redesigning `substrate -c` or `--command`; that remains shell wrap mode per the gap matrix in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:106)
- redesigning the REPL targeted-turn grammar or broadening it beyond the current `::<backend_id> <prompt>` path
- adding a default-agent surface or any heuristic backend/session picker
- making `uaa_session_id` public API
- promoting the REPL-owned attached-control runtime into a general daemon or toolbox mutation service

## Acceptance Criteria

This slice is done when all of the following are true:

1. `substrate agent` publicly exposes `start`, `resume`, `fork`, and `stop` in the CLI surface.
2. `start` requires exact `--backend <backend_id>` and returns `orchestration_session_id` on success.
3. `resume`, `fork`, and `stop` require exact `--session <orchestration_session_id>`.
4. No public command accepts or documents `uaa_session_id`.
5. Host-only success paths reuse existing internal runtime start/stop plumbing rather than reimplementing ownership logic.
6. World-sensitive paths remain Linux-first and fail closed elsewhere.
7. All ambiguity, stale-linkage, missing-parent, unsupported-platform/posture, world-boundary-unavailable, and policy-disallow cases return explicit operator-facing errors rather than fallback selection.
8. This slice lands without changing `substrate -c` semantics and without changing REPL targeted-turn grammar.

## Testing Expectations

At minimum, this slice should add targeted coverage for:

- CLI parse coverage proving the four new subcommands exist and require the exact selectors above
- exact `backend_id` requirement for `start`
- exact `orchestration_session_id` requirement for `resume`, `fork`, and `stop`
- rejection of `uaa_session_id` as a public selector
- fail-closed behavior on ambiguity, stale linkage, missing active parent, unsupported platform/posture, unavailable world boundary, and policy disallow
- Linux-first success and non-Linux fail-closed cases for world-sensitive actions

The right goal is not broad surface area. The right goal is one narrow public namespace that tells the truth about what the runtime can already prove.
