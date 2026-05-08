# PLAN-20: Public Non-Interactive Agent Caller Surface With Exact Backend Targeting, Streamed Turns, And Session Posture Truth

Source SOW: [20-public-non-interactive-agent-caller-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Adjacent landed slices: [PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-18.md), [PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-19.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: public non-interactive prompt-taking caller surface for exact orchestration sessions and exact retained backends  
Review posture: unified execution plan, tightened to `/plan-eng-review` structure and explicit implementation sequencing  
Status: execution-ready planning pass on 2026-05-08  
Outside voice: not used for this document generation

## Objective

Expose one honest non-interactive public prompt-taking surface under `substrate agent` without changing `substrate -c`, without fuzzy routing, and without pretending retained ownership is something it is not.

This slice does five things and only five things:

1. makes `substrate agent start` mean `new session + first real prompt`,
2. adds `substrate agent turn` as the exact follow-up prompt surface for an existing session/backend pair,
3. makes `substrate agent reattach` the canonical lifecycle-recovery verb for owner-loop recovery,
4. streams real caller-visible output for `start` and `turn` in both human mode and `--json` mode,
5. reports authoritative completion-time `session_posture` so callers can tell whether the session stayed live, became detached-but-reattachable, or is terminal.

This slice does not redesign REPL grammar. It does not widen default-agent routing. It does not reinterpret plain stdin or `substrate -c` as agent prompting. It does not introduce a general daemon.

## Plan Summary

The runtime capability is mostly already here. The product contract is not.

Today the repo already has:

- public `substrate agent start|resume|fork|stop` wiring in [`AgentAction`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:493),
- exact session-only public control resolution in [`resolve_public_control_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:589),
- exact REPL targeted follow-up turns in [`dispatch_targeted_follow_up_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3180),
- host follow-up submission in [`submit_host_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4451),
- Linux world follow-up submission in [`submit_world_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4559), [`submit_member_turn_stream(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1472), and [`submit_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:249),
- hidden owner-helper launch machinery in [`launch_hidden_owner_helper(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:231).

What is still wrong:

1. public `start` does not take a real prompt source,
2. public `resume` is actually lifecycle reattachment, not conversational resume,
3. there is no public `turn` command,
4. the public resolver is session-centric only and cannot select the exact retained backend within a multi-backend orchestration session,
5. the retained-runtime bootstrap prompt still exists internally and must never leak into the public caller contract,
6. the public JSON contract is still a one-shot terminal object rather than a stream.

The minimum honest fix is:

1. add explicit prompt-source arguments,
2. add one exact `(orchestration_session_id, backend_id)` public turn resolver,
3. extract the existing targeted-turn execution seam so public CLI can reuse it,
4. bridge helper-owned output back to the invoking CLI in real time,
5. surface completion-time `session_posture` explicitly,
6. rename the canonical lifecycle verb to `reattach`.

That is the whole game.

## Step 0: Scope Challenge

### 0A. What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public agent namespace | [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs) | Reuse `substrate agent`. Do not create a second top-level caller family. |
| Public narrow control plane | [`run_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:291), [`run_resume(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:310), [`run_fork(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:328), [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:346) | Preserve the namespace, but change `start` semantics, add `turn`, and make `reattach` the canonical recovery term. |
| Hidden owner-helper launch seam | [`launch_hidden_owner_helper(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:231), [`HiddenOwnerHelperLaunchPlan`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:139) | Reuse the helper. Do not replace it with a new daemon. Extend it to support foreground streaming for prompt-taking calls. |
| Exact public session selector guardrails | [`resolve_public_control_target(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:589) | Reuse for `reattach`, `fork`, and `stop`. Add a separate backend-aware resolver for `turn`. |
| Exact backend selection for root start | [`validate_exact_backend_selection(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:246) | Reuse as the only root `start --backend` selector. |
| Host targeted follow-up execution | [`submit_host_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4451) | Reuse the exact run-control and event-stream behavior. Do not fork a second host follow-up implementation. |
| Linux world targeted follow-up execution | [`submit_world_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4559), [`submit_member_turn_stream(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1472), [`submit_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:249) | Reuse the same exact Linux-first retained-member follow-up seam. |
| No-turn retained startup capability detection | [`AGENT_API_NO_TURN_SESSION_START_V1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1683), [`runtime_supports_no_turn_session_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1685) | Reuse to classify whether `start` can leave a session `active` after the inaugural prompt. |
| Public stop owner plane | [`private_stop_transport_path(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), [`request_private_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) via [`run_stop(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:346) | Preserve. `stop` stays non-prompt-taking and exact. |
| Status degradation vs strict control split | [`PLAN-18.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-18.md) and current repo behavior | Preserve. Prompt-taking surfaces stay strict and fail closed. |

### 0B. Minimum honest diff

The minimum honest implementation is:

1. extend CLI args so `start` accepts an exact prompt source and add a new `turn` action,
2. introduce one shared prompt-source parser/loader with exact pre-launch failures,
3. introduce one authoritative backend-aware public turn resolver for exact `(orchestration_session_id, backend_id)` lookup,
4. extract the existing targeted-turn submit and stream-translation seam out of REPL-only call sites so public CLI can reuse it,
5. extend the hidden owner-helper contract to support a foreground stream bridge for prompt-taking modes,
6. rename the canonical recovery verb to `reattach`,
7. classify and render `session_posture`,
8. update tests and repo-truth docs.

Anything smaller leaves the prompt-taking surface fake.

### 0C. Complexity check

This slice trips the file-count smell, but it does not justify splitting the work. The surface crosses CLI parsing, exact resolution, retained ownership, streaming, and tests. If any one of those is deferred, the public contract lies.

Expected production files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
4. [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
5. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
6. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
7. possibly [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) for explicit posture helpers or metadata comments only

Expected tests and docs:

1. [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
3. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
4. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
5. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

This is engineered enough. Not overbuilt, not underbuilt.

### 0D. Search and completeness check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse exact backend validation from [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs),
- **[Layer 1]** reuse session-only strict control resolution from [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs),
- **[Layer 1]** reuse exact host and world follow-up submission seams from [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs),
- **[Layer 1]** reuse the existing hidden owner-helper instead of inventing a daemon,
- **[EUREKA]** the missing thin slice is not runtime capability. It is public contract honesty: prompt source parsing, exact `(session, backend)` targeting, and foreground output streaming while ownership may outlive the caller,
- **[EUREKA]** the smallest complete version is one explicit bridge between the current public owner plane and the already-landed REPL targeted-turn seam.

### 0E. Distribution check

No new artifact type is introduced.

The real distribution requirement is contract truth:

1. CLI help must show the new prompt-taking grammar,
2. docs must say `reattach`, not conversational `resume`,
3. JSON streaming must be stable enough for scripts from day one,
4. operator surfaces must preserve the `-c` shell contract by construction.

### 0F. NOT in scope

- redesigning `substrate -c` or plain pipe mode
- default-agent routing
- fuzzy backend or latest-session selection
- public world-root `start`
- member-level public selectors outside `orchestration_session_id`
- toolbox mutation features
- a general multi-session daemon or hub service
- macOS/Lima parity for world-sensitive follow-up turns
- Windows/WSL parity for world-sensitive follow-up turns
- redesigning `fork`
- broader session-history or list-sessions product work

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first.

### Non-negotiable invariants

1. `substrate -c`, `--command`, pipe mode, and plain stdin remain shell-wrap semantics. This slice must not reinterpret them as agent prompting.
2. Every prompt-taking call names an exact `backend_id`.
3. Every follow-up prompt-taking call names both exact `orchestration_session_id` and exact `backend_id`.
4. No public prompt-taking or lifecycle-recovery command accepts `participant_id`, `active_session_handle_id`, `session_handle_id`, or `internal.uaa_session_id` as input.
5. Prompt validation happens before any runtime launch, runtime recovery, or stream bridge setup.
6. Public `start` and `turn` stream caller-visible output while the turn is running. No full-output buffering is allowed.
7. Public `reattach`, `fork`, and `stop` remain non-prompt-taking operational surfaces.
8. Public prompt-taking must never inject `runtime_bootstrap_prompt` text as user-visible agent input.
9. Follow-up turn submission remains single-writer. If the exact session/backend slot is already live-owned, the turn routes through that owner or fails closed. No competing owner loops.
10. Host and world follow-up paths stay distinct. Linux world-sensitive prompt submission still goes through the exact retained-member seam. Non-Linux world-sensitive follow-up still fails closed.
11. Completion-time `session_posture` is explicit and authoritative only at command completion time, not a promise about the indefinite future.
12. `fork` remains the existing lifecycle surface. This slice does not redesign its meaning.

### Public command contract

```text
substrate agent start    --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]
substrate agent turn     --session <orchestration_session_id> --backend <backend_id> (--prompt <text> | --prompt-file <path> | --prompt-file -) [--json]
substrate agent reattach --session <orchestration_session_id> [--json]
substrate agent fork     --session <orchestration_session_id> [--json]
substrate agent stop     --session <orchestration_session_id> [--json]
```

Compatibility rule:

1. `reattach` is the canonical documented verb.
2. `resume` may remain as a hidden deprecated alias for one compatibility window only.
3. All docs, help text, human output, and JSON `action` values use `reattach`, never `resume`.
4. Future conversational resume work is still allowed because `resume` is not part of the canonical contract after this slice.

### Exact selector rules

| Command | Required selector | Resolution rule | Explicitly rejected |
| --- | --- | --- | --- |
| `start` | `--backend <backend_id>` | exact host-scoped backend match via [`validate_exact_backend_selection(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:246) | agent id, default backend, world-only root start, fuzzy or prefix matching |
| `turn` | `--session <orchestration_session_id>` + `--backend <backend_id>` | exact authoritative session plus exact authoritative backend slot within that session | latest session, latest backend, agent id, `participant_id`, `internal.uaa_session_id`, fuzzy matching |
| `reattach` | `--session <orchestration_session_id>` | exact existing session via strict control resolver | prompt args, backend-only resolution, non-canonical handles |
| `fork` | `--session <orchestration_session_id>` | exact existing session via strict control resolver | prompt args, backend-only resolution, non-canonical handles |
| `stop` | `--session <orchestration_session_id>` | exact existing session via strict control resolver plus exact owner transport | direct JSON mutation, PID-only heuristics, non-canonical handles |

### Prompt-source contract

Exactly one prompt source is required for `start` and `turn`:

1. `--prompt <text>`
2. `--prompt-file <path>`
3. `--prompt-file -` meaning read prompt text from stdin

Rules:

1. `--prompt` and `--prompt-file` are mutually exclusive.
2. `--prompt-file -` consumes stdin once and only for prompt input. It does not change `-c` semantics.
3. Effective prompt text is trimmed for emptiness checks only. The original prompt bytes are preserved for actual agent input.
4. Empty or all-whitespace effective prompt is a hard pre-launch failure.
5. Missing file, unreadable file, invalid UTF-8, or stdin read failure is a hard pre-launch failure.
6. No missing or malformed prompt source may fall back to bootstrap text.

### Session posture contract

The completion-time `session_posture` enum is:

- `active`
- `detached_reattachable`
- `terminal`

Meanings:

- `active`
  - at command completion time, a live retained owner is still attached to the exact target session/backend
  - later `turn`, `reattach`, `fork`, and `stop` remain subject to fresh reachability and posture checks
- `detached_reattachable`
  - at command completion time, no live retained owner is attached, but authoritative recovery metadata still exists
  - later `reattach`, `fork`, and possibly `turn` may succeed if exact metadata and posture remain valid
- `terminal`
  - at command completion time, the relevant session is already stopped, invalidated, failed, or completed beyond follow-up

Classification rule for `start`:

1. if the backend supports no-turn retained startup and the owner remains attached after the inaugural prompt, report `active`,
2. if the inaugural prompt completed but no live owner remains and authoritative recovery metadata still exists, report `detached_reattachable`,
3. otherwise report `terminal`.

### Streaming contract

Human mode:

1. stream user-visible agent output to stdout while the turn is running,
2. reserve stderr for pre-stream validation, setup, and bridge errors,
3. print one terminal summary line after completion that includes:
   - `action`
   - `orchestration_session_id`
   - `backend_id`
   - `participant_id` when available
   - turn outcome
   - `session_posture`

JSON mode:

1. stdout is line-delimited JSON only,
2. every record has `version: 1`,
3. every record has a stable string `kind`,
4. after the first stdout JSON record is emitted, all later runtime failures must be represented in-stream, not only on stderr.

Minimum NDJSON shape:

```json
{"version":1,"kind":"accepted","action":"start|turn","orchestration_session_id":"...","backend_id":"...","participant_id":"...","scope":"host|world"}
{"version":1,"kind":"event","event_kind":"message|status|stderr|tool","data":{...}}
{"version":1,"kind":"warning","message":"..."}
{"version":1,"kind":"completed","action":"start|turn","orchestration_session_id":"...","backend_id":"...","participant_id":"...","turn_outcome":"success|nonzero_exit|cancelled","session_posture":"active|detached_reattachable|terminal","state":"active|stopped|invalidated|failed","warnings":[]}
{"version":1,"kind":"failed","terminal":true,"stage":"setup|bridge|runtime","error_code":"...","message":"..."}
```

### Failure taxonomy to freeze now

Use these stable operator-facing error codes:

- `missing_backend`
- `unknown_backend`
- `ambiguous_backend`
- `missing_prompt_source`
- `malformed_prompt_source`
- `empty_prompt`
- `unknown_session`
- `noncanonical_session_selector`
- `backend_not_in_session`
- `ambiguous_backend_slot`
- `missing_active_parent`
- `stale_linkage`
- `missing_internal_session_id`
- `session_already_owned`
- `owner_unreachable`
- `unsupported_platform_or_posture`
- `policy_disallow`
- `stream_bridge_failed`
- `runtime_start_failed`

No prompt-taking or recovery failure may fall back to shell execution, implicit REPL grammar, synthetic prompt text, or best-effort session guessing.

## Architecture Review

### Locked architecture decisions

1. Keep all public caller and lifecycle verbs under `substrate agent`.
2. Make `reattach` the canonical recovery verb. `resume` is, at most, a hidden deprecated alias.
3. Add one shared prompt-source and stream-bridge implementation in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs). Do not invent a second lifecycle module.
4. Add one new authoritative backend-aware resolver for public `turn`. Keep session-only lifecycle resolution strict and separate.
5. Public `start` and `turn` are foreground one-shot surfaces. They may still rely on a helper-owned retained session internally, but the caller remains attached until the submitted turn reaches a terminal outcome.
6. Public `start` does not create an empty live session. A successful `start` always corresponds to one real user prompt submission.
7. Public `turn` never means root-start. Root prompt-taking belongs only to `start`.
8. `fork` stays the current lifecycle surface and is outside the prompt-taking redesign, except for keeping selector and status behavior coherent.
9. `stop` stays exact and non-prompt-taking.
10. `session_posture` is part of the public contract for `start` and `turn`.

### Architecture findings resolved in-plan

1. The current public `start` is still a detached lifecycle command, not a caller surface. This plan fixes that by turning `start` into a foreground prompt-taking surface with a stream bridge.
2. Session-only control resolution is not enough for public `turn`. A session can contain multiple retained backends. This plan fixes that with a backend-aware authoritative resolver.
3. Duplicating targeted-turn submission logic would be a mess. Host and world follow-up submission already exist and already know how to persist events and final state. This plan extracts and reuses them instead of rebuilding the same flow inside `agents_cmd.rs`.
4. Public streaming and retained ownership pull in opposite directions unless the helper contract is explicit. This plan freezes one bridge contract between them instead of pretending stdout inheritance solves it.
5. `reattach` and `turn` are not the same thing. `reattach` restores ownership. `turn` submits work. Keep them separate.

### Hidden owner-helper contract

The existing hidden helper is reused, not replaced.

Required modes after this slice:

- `start_prompted`
- `turn_prompted`
- `reattach`
- `fork`

Required helper inputs after public CLI resolution:

- exact action mode
- exact `orchestration_session_id`
- exact `backend_id`
- exact prompt source material for prompt-taking modes
- source `internal.uaa_session_id` for recovery or fork modes when required
- exact world posture metadata when the target is world-sensitive
- one one-shot bridge descriptor for prompt-taking modes

The helper does not guess. By the time it starts, selection is done.

### Stream bridge contract

The helper needs one prompt-taking bridge for `start` and `turn`.

Rules:

1. the bridge is per-invocation, not global,
2. the bridge is used only for prompt-taking modes,
3. the bridge carries ordered `accepted|event|warning|completed|failed` frames,
4. the bridge closes when the one submitted turn reaches a terminal result,
5. `reattach`, `fork`, and `stop` do not use the prompt-taking bridge,
6. bridge setup failure after command acceptance emits an in-stream `kind="failed"` terminal record in JSON mode,
7. helper-local retained ownership may outlive the bridge.

### Backend-aware public turn resolver

Add one new resolver in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) for:

```text
(orchestration_session_id, backend_id) -> exact authoritative target
```

Required behavior:

1. load the exact parent orchestration session,
2. reject non-canonical session selectors using the same existing guardrails,
3. find the exact backend slot within that session,
4. reject if zero authoritative candidates exist,
5. reject if more than one authoritative candidate exists,
6. distinguish host-orchestrator target vs world-member target explicitly,
7. report whether the target is currently live-owned, detached-but-reattachable, or terminal,
8. preserve Linux-first failure posture for world-sensitive follow-up.

### Session posture and execution split

```text
START
=====
exact backend_id
    |
    +--> validate prompt source before runtime launch
    +--> if backend supports no-turn retained startup:
    |       +--> establish retained owner first
    |       +--> submit inaugural real prompt through shared follow-up seam
    |       +--> classify final posture as active or detached_reattachable
    |
    +--> else:
            +--> run inaugural prompt directly through helper-owned foreground path
            +--> persist surfaced session metadata if any
            +--> classify final posture as detached_reattachable or terminal

TURN
====
exact orchestration_session_id + exact backend_id
    |
    +--> resolve exact backend slot
    +--> if live-owned:
    |       +--> submit through exact retained owner path
    |
    +--> else if detached but recoverable:
    |       +--> recover ownership first
    |       +--> submit through the same shared follow-up seam
    |
    +--> else:
            +--> fail closed

REATTACH
========
exact orchestration_session_id
    |
    +--> recover retained ownership only
    +--> no prompt submission
```

### Architecture ASCII diagrams

#### Public prompt-taking flow

```text
PUBLIC PROMPT CALLER
====================
substrate agent start|turn
    |
    +--> parse exact prompt source
    +--> validate prompt source before any runtime action
    +--> resolve exact target
    |      |
    |      +--> start: exact host backend_id
    |      +--> turn: exact orchestration_session_id + exact backend_id
    |
    +--> spawn/helper or reuse owner plane
    +--> open one-shot stream bridge
    +--> emit accepted
    +--> stream translated runtime events
    +--> persist authoritative state updates
    +--> classify completion-time session_posture
    +--> emit completed or failed
```

#### Exact turn resolution

```text
TURN TARGET RESOLUTION
======================
input: orchestration_session_id + backend_id
    |
    +--> load authoritative parent session
    +--> reject non-canonical selectors
    +--> search exact backend candidates within session
           |
           +--> 0 candidates      -> backend_not_in_session
           +--> >1 candidates     -> ambiguous_backend_slot
           +--> 1 host candidate  -> host follow-up target
           +--> 1 world candidate -> world follow-up target
    |
    +--> check live owner / recovery metadata / posture
    +--> return exact target enum or fail closed
```

#### Human and JSON bridge split

```text
HELPER-OWNED TURN
=================
helper runtime events
    |
    +--> bridge frame
           |
           +--> human renderer -> stdout text stream + terminal summary
           |
           +--> json renderer  -> NDJSON accepted/event/completed/failed

stderr stays reserved for pre-stream validation and bridge setup failures.
```

## Code Quality Review

### Findings resolved in-plan

1. Do not duplicate prompt-source parsing between `start` and `turn`. Add one shared loader.
2. Do not duplicate targeted-turn submission logic between REPL and public CLI. Extract shared helpers from [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs).
3. Do not overload `resolve_public_control_target(...)` with backend-aware turn semantics. Keep lifecycle and prompt-taking resolution separate.
4. Do not let `agents_cmd.rs` hand-assemble UAA extension payloads. Keep those helpers in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs).
5. Do not let `resume` and `reattach` both appear as first-class documented verbs. Pick one public term. This plan picks `reattach`.
6. Do not leak `internal.uaa_session_id` into public success output while adding `session_posture`.
7. Do not invent a second event translation format for public CLI streaming. Reuse translated runtime event shapes wherever possible and wrap them in the frozen top-level NDJSON envelope.

### Required code comments and diagrams

Add or update nearby ASCII comments in these places if code lands there:

1. [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), prompt-taking bridge and helper-owned turn flow
2. [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), exact `(session, backend)` public turn resolver
3. [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), if shared submit helpers move and nearby ownership diagrams become stale

Stale diagrams are worse than no diagrams. Update them in the same change.

## Test Review

### Test framework detection

This repo is Rust-first and the relevant review surface is `cargo test`.

Primary suites for this slice:

1. [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
3. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
4. targeted unit tests in [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
5. targeted unit tests in [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/cli.rs
    |
    ├── AgentStartArgs
    │   ├── [GAP] exact one-of prompt source parsing
    │   ├── [GAP] missing backend rejected at parse or validation boundary
    │   └── [GAP] --json remains orthogonal to prompt source
    |
    ├── AgentTurnArgs (new)
    │   ├── [GAP] requires exact --session and exact --backend
    │   ├── [GAP] requires exact one-of prompt source
    │   └── [GAP] root turn form is impossible
    |
    └── AgentReattachArgs / alias posture
        ├── [GAP] canonical `reattach` parses
        └── [GAP] deprecated `resume` alias behavior pinned if retained

[+] crates/shell/src/execution/agent_runtime/control.rs
    |
    ├── prompt source loader
    │   ├── [GAP] --prompt literal success
    │   ├── [GAP] --prompt-file path success
    │   ├── [GAP] --prompt-file - stdin success
    │   ├── [GAP] missing source rejection
    │   ├── [GAP] malformed source rejection
    │   └── [GAP] empty effective prompt rejection
    |
    ├── start_prompted path
    │   ├── [GAP] exact host backend root success with stream bridge
    │   ├── [GAP] world-only backend root rejection
    │   ├── [GAP] no synthetic bootstrap prompt used as public input
    │   ├── [GAP] completion emits session_posture
    │   └── [GAP] accepted/event/completed NDJSON contract
    |
    ├── turn_prompted path
    │   ├── [GAP] exact live host follow-up success
    │   ├── [GAP] exact live Linux world follow-up success
    │   ├── [GAP] detached recoverable target can recover then submit
    │   ├── [GAP] backend-not-in-session rejection
    │   └── [GAP] ambiguous backend slot rejection
    |
    └── stream bridge
        ├── [GAP] human-mode streaming surfaces output incrementally
        ├── [GAP] JSON mode emits line-delimited records only
        └── [GAP] bridge failure after stream start emits terminal kind="failed"

[+] crates/shell/src/execution/agent_runtime/state_store.rs
    |
    ├── session-only public control resolver
    │   └── [★★★ TESTED] strict non-canonical handle rejection already exists and must stay green
    |
    └── backend-aware public turn resolver (new)
        ├── [GAP] exact backend host slot resolution
        ├── [GAP] exact backend world slot resolution
        ├── [GAP] missing backend slot rejection
        ├── [GAP] ambiguous backend slot rejection
        ├── [GAP] Linux-first posture rejection for world-sensitive follow-up
        └── [GAP] detached-but-recoverable posture classification

[+] crates/shell/src/repl/async_repl.rs
    |
    ├── dispatch_targeted_follow_up_turn(...)
    │   ├── [★★★ TESTED] exact host/world targeted routing already exists
    │   └── [GAP] extracted shared helper leaves REPL behavior unchanged
    |
    ├── submit_host_targeted_turn(...)
    │   └── [★★★ TESTED] host follow-up streaming/persistence remains source truth
    |
    └── submit_world_targeted_turn(...)
        └── [★★★ TESTED] Linux world retained-member submit remains source truth

[+] crates/world-agent/src/service.rs + member_runtime.rs
    |
    └── world member submit seam
        └── [★★★ TESTED] exact retained member turn stream remains unchanged functionally

---------------------------------
COVERAGE TARGET
- every new public caller path has success + rejection tests
- every streaming mode has at least one assertion on incremental behavior
- every session_posture outcome is explicitly covered
- REPL targeted-turn behavior stays green
---------------------------------
```

### Operator flow coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] Operator runs `substrate agent start --backend cli:codex --prompt "hi" --json`
    |
    ├── [GAP] sees accepted early
    ├── [GAP] sees streamed events during execution
    └── [GAP] sees completed with session_posture

[+] Operator runs `substrate agent start --backend cli:codex --prompt-file -`
    |
    ├── [GAP] stdin prompt is read once
    └── [GAP] empty stdin rejects before runtime launch

[+] Operator runs `substrate agent turn --session <sess> --backend cli:codex --prompt "next"`
    |
    ├── [GAP] exact live host follow-up succeeds
    ├── [GAP] exact detached host follow-up can recover if authoritative metadata exists
    └── [GAP] wrong backend in right session rejects explicitly

[+] Operator runs `substrate agent turn --session <sess> --backend cli:claude_code --prompt "next"`
    |
    ├── [GAP] exact Linux world follow-up succeeds through retained member seam when posture is valid
    └── [GAP] non-Linux world-sensitive follow-up rejects explicitly

[+] Operator runs `substrate agent reattach --session <sess> --json`
    |
    ├── [GAP] canonical verb succeeds against recoverable session
    └── [GAP] already-owned session rejects with session_already_owned

[+] Operator targets non-canonical handles
    |
    ├── [GAP] participant_id rejected
    ├── [GAP] internal.uaa_session_id rejected
    └── [GAP] active_session_handle_id rejected

[+] Operator runs `substrate -c "echo hi"`
    |
    └── [GAP] remains shell-wrap mode. No prompt-taking regression.
```

### Required tests to add or extend

1. Add CLI parse coverage proving `start` requires exactly one prompt source.
2. Add CLI parse coverage proving `turn` requires `--session`, `--backend`, and exactly one prompt source.
3. Add prompt-source tests for `--prompt-file -`, unreadable files, invalid UTF-8, and empty effective prompt.
4. Extend the public control integration suite so `start` asserts accepted/event/completed streaming behavior rather than only one terminal JSON object.
5. Add a public `turn` integration test for an exact live host-owned backend.
6. Add a public `turn` integration test for an exact live Linux world-owned backend when authoritative posture is valid.
7. Add a public `turn` rejection test for `backend_not_in_session`.
8. Add a public `turn` rejection test for `ambiguous_backend_slot`.
9. Add a public `turn` rejection test for non-Linux world-sensitive follow-up.
10. Add a public `reattach` integration test for exact recoverable session rebind.
11. If the deprecated `resume` alias stays temporarily, add one compatibility test that the alias routes to `reattach` semantics while emitted `action` is still `reattach`.
12. Add `session_posture` classification tests covering `active`, `detached_reattachable`, and `terminal`.
13. Add a regression test proving no public prompt-taking path ever injects `runtime_bootstrap_prompt`.
14. Keep current `fork` and `stop` tests green.
15. Keep current REPL targeted-turn tests green.
16. Keep `substrate -c` shell-wrap behavior green.

### QA-facing test artifact

During implementation, write a QA-facing artifact to:

```text
~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. root `start` happy path with `--prompt`
2. root `start` stdin path with `--prompt-file -`
3. exact host `turn`
4. exact Linux world `turn`
5. detached recoverable `turn`
6. `reattach`
7. wrong-handle rejection flows
8. `-c` non-regression spot check

Keep it operator-journey oriented.

## Failure Modes Registry

| Failure mode | Test required | Error handling exists | Operator sees clear result | Critical gap before this slice lands |
| --- | --- | --- | --- | --- |
| public `start` launches ownership but never submits the real prompt | yes | no | no | yes |
| public `start` accepts missing or empty prompt input and falls back to bootstrap text | yes | no | no | yes |
| public `start` buffers output and only prints at the end | yes | no | no | yes |
| public `turn` routes by session alone and hits the wrong retained backend | yes | no | no | yes |
| public `turn` silently creates a competing retained owner for a live slot | yes | no | no | yes |
| public `turn` on non-Linux world-sensitive posture pretends to succeed | yes | partial today via fail-closed REPL posture | no | yes |
| `reattach` remains labeled as conversational resume in public output | yes | no | no | yes |
| bridge failure after stream start only appears on stderr in JSON mode | yes | no | no | yes |
| `session_posture` is guessed from helper spawn instead of command completion truth | yes | no | no | yes |
| `-c` begins accepting prompt-like input accidentally | yes | no | no | yes |
| public success output leaks `internal.uaa_session_id` | yes | partial today via omission in current control JSON | no | yes |
| prompt-taking path regresses current `fork` or `stop` behavior | yes | partial today via existing tests | no | yes |

Critical-gap rule for this plan:

No prompt-taking command is allowed to both mutate agent state and hide the actual turn outcome. If the caller cannot see the real submitted turn, the surface is incomplete.

## Performance Review

Performance is not the main risk, but streaming and helper bridging can still get sloppy.

### Findings resolved in-plan

1. Prompt-taking paths must stream incrementally. No full-output buffering into memory.
2. Bridge readers must use bounded frame processing. A single giant buffer of accumulated output is not acceptable.
3. The new backend-aware resolver must read exact authoritative session state, not scan trace history or broad inventory repeatedly.
4. Readiness and terminal waits must stay bounded with polling or backoff. No tight spin loops.
5. This slice does not justify a global bridge broker or daemon. One invocation-scoped bridge per prompt-taking call is enough.
6. World-sensitive rejection should happen before any expensive recovery attempt on unsupported platforms.

### Performance posture

- no new global listener
- no broad trace scans
- no full-response buffering
- bounded readiness waits only
- correctness and explicit failure beat a few milliseconds of latency

## DX Guardrails

This is a developer/operator surface even though the implementation is runtime-heavy.

Required experience:

1. the caller always knows whether they are starting, turning, reattaching, forking, or stopping,
2. the caller always sees the exact `backend_id` and exact `orchestration_session_id`,
3. JSON output is stable and line-delimited from day one,
4. errors distinguish `backend_not_in_session` from `unknown_session`,
5. errors distinguish `owner_unreachable` from `session_already_owned`,
6. help text and docs say `reattach`, not `resume`,
7. `-c` remains shell behavior by construction, not by best effort.

## Worktree Parallelization Strategy

This plan has real parallelization opportunities once the output contract is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze public caller grammar, prompt-source contract, `reattach` naming, NDJSON envelope, and failure taxonomy | `crates/shell/src/execution/`, repo docs | — |
| CLI and command-handler wiring for `start`, `turn`, and `reattach` | `crates/shell/src/execution/` | Freeze public caller grammar, prompt-source contract, `reattach` naming, NDJSON envelope, and failure taxonomy |
| Backend-aware public turn resolver and posture classification | `crates/shell/src/execution/agent_runtime/` | Freeze public caller grammar, prompt-source contract, `reattach` naming, NDJSON envelope, and failure taxonomy |
| Shared prompt-submit extraction and stream bridge | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | Freeze public caller grammar, prompt-source contract, `reattach` naming, NDJSON envelope, and failure taxonomy |
| Integration tests and repo-truth closeout | `crates/shell/tests/`, repo docs | CLI and command-handler wiring, Backend-aware public turn resolver and posture classification, Shared prompt-submit extraction and stream bridge |

### Parallel lanes

- Lane A: CLI and command-handler wiring
  - sequential inside the lane because these steps share `cli.rs` and `agents_cmd.rs`
- Lane B: backend-aware public turn resolver and posture classification
  - sequential inside the lane because these steps share `state_store.rs` and nearby runtime metadata
- Lane C: shared prompt-submit extraction and stream bridge
  - sequential inside the lane because these steps share `control.rs` and `async_repl.rs`
- Lane D: integration tests and repo-truth closeout
  - starts only after A, B, and C merge

### Execution order

1. Freeze the caller grammar, naming, NDJSON contract, and failure taxonomy.
2. Launch Lanes A, B, and C in parallel worktrees.
3. Merge A, B, and C.
4. Run Lane D for integration tests, REPL non-regressions, and doc closeout.

### Conflict flags

- Lane A and Lane C both depend on the exact NDJSON envelope. Freeze it first or they will drift.
- Lane B and Lane C both influence `session_posture`. Lane B owns authoritative classification inputs. Lane C must consume, not reinvent, them.
- Lane D owns the prompt-taking integration assertions. Keep that ownership there to avoid test churn across implementation lanes.
- Docs move last. Updating the gap matrix before tests and stream behavior land will drift repo truth again.

### Parallelization verdict

Four workstreams, three parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the public caller contract

Files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
4. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) for wording freeze if needed during implementation

Deliver:

1. add explicit prompt-source arg shapes for `start`,
2. add `turn`,
3. make `reattach` canonical,
4. freeze NDJSON envelope and human summary contract,
5. freeze `session_posture` enum and meanings,
6. freeze failure taxonomy.

Done means nobody is still guessing what the caller sees.

### Step 2. Add shared prompt-source loading

Files:

1. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Deliver:

1. one shared loader for `--prompt`, `--prompt-file <path>`, and `--prompt-file -`,
2. exact pre-launch error mapping for missing, malformed, and empty prompt input,
3. no bootstrap-text fallback.

Done means `start` and `turn` cannot disagree about prompt validation.

### Step 3. Add the exact backend-aware public turn resolver

Files:

1. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) only if posture helpers or metadata comments are needed
3. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) only if parent-state comments need clarification

Deliver:

1. exact `(orchestration_session_id, backend_id)` lookup,
2. host vs world target classification,
3. live vs detached vs terminal target posture classification,
4. exact failure codes for missing and ambiguous backend slots,
5. Linux-first fail-closed posture for world-sensitive follow-up.

Done means public `turn` can target the right runtime without guessing.

### Step 4. Extract shared prompt-submit helpers from REPL-only paths

Files:

1. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

Deliver:

1. extract host follow-up submit behavior from [`submit_host_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4451),
2. extract Linux world follow-up submit behavior from [`submit_world_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4559),
3. preserve REPL behavior unchanged by calling the shared helpers from existing REPL code,
4. expose a bridge-friendly event sink for public CLI prompt-taking modes.

Done means there is one prompt-submit truth, not a REPL truth and a public CLI truth.

### Step 5. Extend the helper with a prompt-taking stream bridge

Files:

1. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Deliver:

1. add prompt-taking helper modes,
2. add one invocation-scoped bridge descriptor to the helper launch plan,
3. translate runtime events into frozen bridge records,
4. close the bridge exactly once per submitted prompt,
5. keep helper ownership alive after caller exit when posture stays `active`.

Done means the public caller can see the real turn without owning the retained session forever.

### Step 6. Wire public `start`

Files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

Deliver:

1. validate exact host backend and exact prompt source,
2. allocate a new `orchestration_session_id`,
3. launch helper in prompted-start mode,
4. stream output to the caller,
5. classify and emit final `session_posture`,
6. preserve host-only root start rejection for world-only backends.

Done means `start` is a real prompt-taking caller surface.

### Step 7. Wire public `turn`

Files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
4. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Deliver:

1. validate exact prompt source,
2. resolve exact session/backend target,
3. route through live host follow-up when exact host owner exists,
4. route through live Linux world retained-member follow-up when exact world owner exists,
5. recover ownership first when the target is detached but reattachable,
6. fail closed otherwise,
7. emit streamed records and final `session_posture`.

Done means public `turn` is exact, streamed, and honest.

### Step 8. Wire canonical `reattach` and keep `fork` and `stop` coherent

Files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Deliver:

1. expose canonical `reattach`,
2. optionally keep hidden deprecated `resume` alias for one compatibility window,
3. keep `fork` unchanged functionally,
4. keep `stop` unchanged functionally,
5. keep emitted JSON `action` strings canonical.

Done means the public terminology matches the product truth.

### Step 9. Freeze the contract with tests and repo-truth docs

Files:

1. [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
2. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
3. [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
4. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
5. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

Deliver:

1. success and rejection coverage for `start`, `turn`, `reattach`, `fork`, and `stop`,
2. stream-contract coverage,
3. `session_posture` coverage,
4. `-c` non-regression coverage,
5. update repo truth to say the public non-interactive caller surface is landed,
6. update packet index to include `PLAN-20`.

Done means the repo says what the runtime actually does.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

If world-sensitive follow-up logic changes materially, also run:

```bash
cargo test -p world-agent --test member_runtime_world_placement_v1 -- --nocapture
```

Required operator-surface checks after tests are green:

```bash
substrate agent doctor --json
substrate shim doctor --json
substrate world doctor --json
substrate health --json
```

Manual spot checks:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
printf 'hello from stdin\n' | substrate agent start --backend <host_backend_id> --prompt-file - --json
substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
substrate agent fork --session <orchestration_session_id> --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json
substrate -c 'echo shell wrap still works'
```

## Definition of Done

1. `substrate agent start` accepts exact prompt input and submits one real inaugural prompt.
2. `substrate agent turn` exists and requires exact `--session` plus exact `--backend`.
3. `substrate agent reattach` is the canonical recovery surface.
4. public prompt-taking commands stream output in human mode and JSON mode.
5. JSON mode is NDJSON with stable `version` and `kind`.
6. `session_posture` is emitted on successful `start` and `turn`.
7. no prompt-taking path uses synthetic bootstrap text as public input.
8. no public prompt-taking or lifecycle command accepts non-canonical session handles.
9. exact Linux world follow-up still goes through the retained-member seam.
10. non-Linux world-sensitive follow-up still fails closed.
11. `substrate -c` remains shell wrap mode.
12. repo-truth docs reflect the landed caller surface accurately.

## Deferred Work

- removing the temporary hidden `resume` alias entirely if one compatibility window is kept
- default-agent routing
- `substrate -c` redesign
- public world-root `start`
- member-level public selectors
- macOS/Lima world-follow-up parity
- Windows/WSL world-follow-up parity
- broader session-history and session-list product work
- any general daemonized agent-hub service

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is after tightening the public contract around exact prompt-taking rather than lifecycle launch alone
- Architecture Review: 5 issues found, all resolved in-plan
- Code Quality Review: 7 issues found, all resolved in-plan
- Test Review: diagrams produced, 16 concrete regression gaps identified
- Performance Review: 6 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, no `TODOS.md` exists in this repo today
- Failure modes: 12 critical gaps flagged until streamed prompt-taking and exact backend-aware turn resolution land
- Outside voice: skipped for this document pass
- Parallelization: 4 workstreams, 3 parallel implementation lanes, 1 final integration lane
- Lake Score: 10/10 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Treat public prompt-taking honesty, not just lifecycle launch, as the core blocker | Mechanical | Pragmatic | The runtime already exists. The missing thing is a truthful caller contract | only adding clap flags and leaving helper behavior detached |
| 2 | Architecture | Keep the public surface under `substrate agent` | Mechanical | Minimal diff | The namespace already exists and is the correct home | inventing a second top-level caller family |
| 3 | Architecture | Make `reattach` the canonical recovery verb | Mechanical | Explicit over clever | The current behavior is owner recovery, not conversational resume | keeping `resume` as the public term indefinitely |
| 4 | Architecture | Add a separate backend-aware public `turn` resolver | Mechanical | Explicit over clever | Session-only resolution is insufficient once one session can retain multiple backends | teaching the existing session-only resolver to guess the target |
| 5 | Architecture | Reuse hidden owner-helper with a prompt-taking stream bridge | Mechanical | Boring by default | The helper already owns retained control correctly. It just cannot yet stream prompt results | inventing a new daemon or broker |
| 6 | Code Quality | Extract shared prompt-submit helpers out of REPL-only code | Mechanical | DRY | Host and world follow-up seams already exist and should stay single-source | duplicating host and world submit logic in `agents_cmd.rs` |
| 7 | Code Quality | Keep `fork` out of the prompt-taking redesign | Mechanical | Minimal diff | `fork` already has a stable lifecycle meaning and does not need product-surface churn here | folding `fork` into `turn` or `reattach` |
| 8 | Streaming | Freeze NDJSON accepted/event/completed/failed envelope up front | Mechanical | Systems over heroes | CLI scripts need stable framing from day one | ad hoc JSON per code path |
| 9 | Tests | Make `session_posture` coverage mandatory | Mechanical | Completeness | Without posture assertions, the product still lies about retained state after completion | inferring posture from helper PID or assuming it is always active |
| 10 | Tests | Keep `-c` shell-wrap regression coverage in this slice | Mechanical | Blast radius instinct | This is the easiest accidental product regression while adding stdin prompt support | assuming prompt-source parsing cannot affect shell mode |
