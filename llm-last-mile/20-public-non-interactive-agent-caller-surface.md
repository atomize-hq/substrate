# SOW: Public Non-Interactive Agent Caller Surface

Status: implementation-oriented follow-on draft. This SOW defines the next caller-surface
slice after the already-landed public control namespace and the already-landed REPL
targeted-turn path.

It changes the public command model to:

- start = new session + first real prompt
- turn = existing session + next turn
- reattach = restore retained ownership to an existing session

This slice is intentionally narrow: make prompt-taking explicit on public start, keep
substrate -c / --command as shell wrap mode, require exact target selectors, and return
real streamed output to the caller. It does not redesign REPL grammar, does not widen
default-agent routing, and does not implicitly repurpose shell command mode.

## Objective

Expose one explicit non-interactive public prompt-taking model:

substrate agent start --backend <backend_id> (--prompt <text> | --prompt-file <path> |
--prompt-file -) [--json]
substrate agent turn --session <orchestration_session_id> --backend <backend_id> (--prompt
<text> | --prompt-file <path> | --prompt-file -) [--json]

This slice must:

- keep substrate -c / --command unchanged as shell wrap mode,
- require exact backend_id targeting for every prompt-taking call,
- require exact orchestration_session_id plus exact backend_id for follow-up turns,
- make start create a new host-scoped orchestration session and submit the provided prompt
  as the inaugural real turn,
- make turn submit one foreground follow-up turn into the exact retained backend inside the
  exact orchestration session,
- stream caller-visible output during execution,
- fail malformed or empty prompt input before any runtime launch,
- forbid synthetic bootstrap text from being injected as public agent input on either start
  or turn,
- preserve explicit lifecycle commands as non-prompt-taking operational surfaces,
- and reserve the word resume for a possible future true interactive/conversational resume
  UX rather than owner-loop recovery.

## Current Repo Truth

The repo has already landed more than the older control-surface drafts assumed, but the
current public start shape is no longer acceptable as the product contract.

What is already landed:

- substrate agent start|resume|fork|stop are already public in ../crates/shell/src/
  execution/cli.rs:493 and handled in ../crates/shell/src/execution/agents_cmd.rs:291.
- REPL targeted follow-up turns are already public as exact ::<backend_id> <prompt> syntax
  in ../crates/shell/src/repl/async_repl.rs:710.
- Exact REPL turn dispatch already distinguishes host vs world follow-up routing in
  dispatch_targeted_follow_up_turn(...) at ../crates/shell/src/repl/async_repl.rs:3180.
- Host follow-up turn submission is already real in submit_host_targeted_turn(...) at ../
  crates/shell/src/repl/async_repl.rs:4451.
- Linux world-member follow-up turn submission is already real in
  submit_world_targeted_turn(...) at ../crates/shell/src/repl/async_repl.rs:4559,
  submit_member_turn_stream(...) at ../crates/world-agent/src/service.rs:1472, and
  submit_turn(...) at ../crates/world-agent/src/member_runtime.rs:249.
- Public control selection already fails closed on non-canonical selectors, stale linkage,
  missing internal session ids, missing owners, and non-Linux world-sensitive posture in
  resolve_public_control_target(...) at ../crates/shell/src/execution/agent_runtime/
  state_store.rs:589, with regression coverage at ../crates/shell/src/execution/
  agent_runtime/state_store.rs:2194.
- Public root agent start --backend ... is intentionally host-only in v1, with failure
  coverage for world-only backends in ../crates/shell/tests/
  agent_public_control_surface_v1.rs:962.
- substrate -c is still shell wrap mode by contract in ../crates/shell/src/execution/
  cli.rs:27, ../crates/shell/src/execution/invocation/plan.rs:29, ../crates/shell/src/
  execution/invocation/plan.rs:619, and ../crates/shell/src/execution/invocation/
  tests.rs:26.

What is wrong with the current public start shape:

- AgentStartArgs currently takes only --backend plus --json, with no public prompt source,
  in ../crates/shell/src/execution/cli.rs:456.
- The current public start path launches a detached hidden owner-helper with stdin, stdout,
  and stderr all set to null in ../crates/shell/src/execution/agents_cmd.rs:231.
- The runtime already distinguishes backends that support no-turn retained startup through
  agent_api.session.start.no_turn.v1 in ../crates/shell/src/repl/async_repl.rs:1683.
- The current retained-owner runtime also has an internal synthetic bootstrap prompt used
  to establish long-lived control ownership in runtime_bootstrap_prompt(...) at ../crates/
  shell/src/repl/async_repl.rs:1758.

That current shape means public start can create a live background session without a real
user prompt and rely on internal retained-control bootstrap semantics. Given the current
Codex integration posture, that is not acceptable as the product model: it pays context
cost up front for an artificial turn and preserves hidden-message behavior we now want to
eliminate from the public contract.

What is true about the current public resume shape:

- today’s resume is not conversational resume,
- it does not submit a turn,
- it does not reopen an interactive REPL surface,
- it reattaches a retained owner loop to an already-existing orchestration session.

This SOW therefore uses reattach as the target public term for that lifecycle behavior and
treats resume as reserved for a possible future true interactive/conversational resume
surface.

What remains open:

- there is still no first-class non-interactive public prompt-taking caller under substrate
  agent,
- public start still means detached host-only retained ownership rather than “first real
  prompt,”
- turn still needs a public exact-target follow-up contract for sessions that may contain
  multiple retained backends,
- the current public control resolver is session-centric and is not by itself sufficient
  for exact follow-up targeting when a session may contain multiple retained backends,
- and no public non-interactive surface yet defines a caller-visible streaming return path.

The gap matrix already calls the non-interactive caller surface out directly in ../
AGENT_ORCHESTRATION_GAP_MATRIX.md:107 and recommends deliberate new caller contracts rather
than implicit -c changes in ../AGENT_ORCHESTRATION_GAP_MATRIX.md:179.

## Relationship To Existing Slices

This SOW consumes already-landed work and should not reopen it:

- ./15-targeted-repl-agent-turns-linux-first.md established the REPL-first exact targeted-
  turn contract, and that contract is now live.
- ./18-status-surface-and-session-handle-hardening.md hardened the status/addressability
  substrate this slice should reuse.
- ./19-public-agent-control-surfaces.md scoped the public control plane; most of that
  namespace is now landed, but this slice changes the public meaning of start, defines turn
  as follow-up only, and renames the current public resume concept to reattach.

## Scope

This slice defines one public prompt-taking command family:

substrate agent start --backend <backend_id> (--prompt <text> | --prompt-file <path> |
--prompt-file -) [--json]
substrate agent turn --session <orchestration_session_id> --backend <backend_id> (--prompt
<text> | --prompt-file <path> | --prompt-file -) [--json]

It also renames one lifecycle concept in the target public contract:

substrate agent reattach --session <orchestration_session_id> [--json]

The command family is intentionally narrow:

- start --backend ... means create one new host-scoped orchestration session for that exact
  backend and submit exactly one inaugural real turn in the foreground.
- turn --session ... --backend ... means submit exactly one foreground follow-up turn into
  that exact backend inside that exact orchestration session.
- reattach --session ... means restore retained ownership to an existing orchestration
  session and does not submit a prompt.
- backend_id is always required for prompt-taking calls.
- orchestration_session_id is required for follow-up turns and lifecycle recovery calls.
- exactly one prompt source is required for both start and turn.
- prompt text is always agent input, never shell input.

## Out Of Scope

This slice does not include:

- preserving no-prompt public start as a valid product surface,
- changing substrate -c / --command,
- changing REPL ::<backend_id> <prompt> grammar,
- adding default-agent or implicit routing,
- fuzzy selector resolution,
- public world-root start,
- member-level public selectors independent of orchestration_session_id,
- daemonizing the control plane,
- promising macOS/Lima parity for world-sensitive follow-up turns,
- preserving the current detached hidden owner-helper bootstrap semantics as the public
  start contract,
- redefining reattach as a prompt-taking or conversational command,
- introducing a future true interactive resume surface,
- or redesigning agent status semantics beyond preserving coherence after start, turn, and
  reattach.

If future operational lifecycle workflows are needed, they should be separate explicitly
operational commands, not preserved by overloading public start.

## Public Selector Contract

### 1. backend_id is always required for prompt-taking calls

Every prompt-taking call must name an exact backend_id.

Required rules:

- no agent_id routing,
- no default backend,
- no “latest backend in this session” behavior,
- no prefix matching.

### 2. Public start requires exact backend_id plus exact prompt source

The root selector model for v1 is:

substrate agent start --backend <backend_id> (--prompt <text> | --prompt-file <path> |
--prompt-file -)

Required rules:

- start without --backend is invalid,
- exactly one prompt source is required,
- malformed or empty prompt input must fail before any runtime launch,
- start must not create an empty live session with no real prompt,
- start must not inject synthetic bootstrap text as agent input,
- root start remains host-only in v1.

### 3. Follow-up turns require both orchestration_session_id and backend_id

The follow-up selector model for v1 is:

substrate agent turn --session <orchestration_session_id> --backend <backend_id> ...

Required rules:

- --session alone is invalid in follow-up mode,
- --session selects the parent orchestration session,
- --backend selects the exact retained backend inside that session,
- follow-up targeting must use a new or extended authoritative state-store resolver for
  exact (orchestration_session_id, backend_id) selection,
- the current session-centric resolve_public_control_target(...) helper is not sufficient
  by itself for this selector contract.

That new or extended resolver must fail closed if:

- the selected session contains no matching backend slot,
- the selected session contains more than one authoritative candidate for that backend,
- or only stale, inactive, or non-authoritative candidates exist.

### 4. Canonical public session handle remains orchestration_session_id

For follow-up targeting and lifecycle recovery, the only accepted public session handle
remains orchestration_session_id.

The surface must reject these as public session selectors:

- participant_id
- active_session_handle_id
- session_handle_id
- internal.uaa_session_id

That posture already exists in the public control/state-store contract and must not
regress.

### 5. reattach remains session-only and non-prompt-taking

The lifecycle recovery model for v1 is:

substrate agent reattach --session <orchestration_session_id> [--json]

Required rules:

- reattach does not accept a prompt,
- reattach does not create a new conversation turn,
- reattach restores retained ownership to an existing orchestration session when
  authoritative recovery metadata is intact,
- reattach must fail closed if the session is already live-owned, lacks required internal
  session metadata, or is otherwise not recoverable.

## Execution Model

Public start and turn are foreground one-shot caller surfaces.

That means:

- the invoking CLI process remains attached until the submitted turn completes or fails,
- the caller receives streamed output while the turn is running,
- the command exits after exactly one submitted turn reaches a terminal result,
- and any retained owner or session state that survives past command exit is an explicit
  postcondition, not an implicit side effect.

This slice must not blur together detached control-plane ownership and foreground caller
experience.

### Product distinction between start, turn, and reattach

The product distinction must stay obvious:

- substrate agent start --backend ... --prompt ... creates a new session and submits the
  first real user prompt.
- substrate agent turn --session ... --backend ... --prompt ... submits the next real user
  prompt into an exact existing session/backend target.
- substrate agent reattach --session ... restores retained ownership to an existing session
  and does not submit a turn.
- none of these commands may inject synthetic bootstrap text as public agent input.

Root start --backend may legitimately complete in any of the three session_posture states
described below, depending on backend capability and runtime outcome. active is not the
only expected result.

## Session Lifetime Postconditions

A successful start or turn must report the resulting session posture explicitly.

The minimum postcondition enum is:

- active
- detached_reattachable
- terminal

These meanings are mandatory, but they are authoritative only at command completion time.
Later calls remain subject to fresh reachability, policy, posture, and state-store checks.

- active
  - at command completion time, a live retained owner is still attached
  - substrate agent status must surface the session coherently as live at completion time
  - later turn --session ... --backend ..., stop, reattach, and fork remain subject to
    fresh reachability, policy, and posture checks
- detached_reattachable
  - at command completion time, the session is not live-owned, but authoritative recovery
    metadata remains intact
  - substrate agent status must surface the detached session coherently at completion
    time
  - later reattach, fork, or follow-up turn --session ... --backend ... remain subject to
    fresh reachability, policy, and posture checks
- terminal
  - at command completion time, the session is already completed, failed, invalidated, or
    stopped
  - substrate agent status must surface the final state coherently at completion time
  - later turn --session ... --backend ..., reattach, fork, and stop remain subject to
    fresh reachability, policy, and posture checks and may fail closed

### Capability constraint from current repo truth

The repo already distinguishes runtimes that support no-turn retained startup via
agent_api.session.start.no_turn.v1 in ../crates/shell/src/repl/async_repl.rs:1683.

This slice must respect that distinction:

- when the selected backend supports no-turn retained startup, start --backend may prefer
  leaving the new session active after the inaugural turn completes,
- when the backend does not support that capability, this slice may still succeed, but it
  must not claim that the resulting session will remain live-owned beyond completion,
- the actual resulting session_posture must be surfaced explicitly in the terminal output
  contract.

## Streaming Return Path

The public prompt-taking surface must have a real streamed caller-visible return path.

This requirement is mandatory regardless of internal mechanism.

Allowed internal patterns include:

- parent CLI process streaming frames directly from a local runtime or control handle,
- parent CLI process bridging a hidden owner-helper stream over inherited pipes,
- parent CLI process bridging a private owner transport that supports streamed turn output,
- or another equivalent streamed return path.

What is not allowed:

- buffering the whole turn and printing only at the end,
- writing output only to persistent trace or status surfaces and asking the caller to poll,
- adding a private request enum with no caller-visible streaming channel,
- or swapping real prompt input for synthetic bootstrap text before streaming begins.

### Human-mode streaming rules

Default human mode must:

- stream user-visible agent output to stdout while the turn is running,
- reserve stderr for CLI-local usage, configuration errors, and pre-stream setup failures,
- and print a terminal summary that includes at least orchestration_session_id, backend_id,
  participant_id when available, final turn outcome, and resulting session_posture.

### JSON streaming rules

--json should be a line-delimited streaming contract on stdout.

Recommended minimum record shape:

{"version":1,"kind":"accepted",...}
{"version":1,"kind":"event",...}
{"version":1,"kind":"warning",...}
{"version":1,"kind":"completed",...}

Required rules:

- one JSON object per line,
- stable integer version,
- stable string kind,
- accepted must surface target identity early,
- completed must surface final turn outcome plus resulting session_posture,
- after the first stdout JSON record is emitted, subsequent runtime or turn errors must be
  represented in-stream rather than only on stderr.

For setup or bridge failures that occur after stream start, emit one explicit terminal
failure record on stdout such as:

{
"version": 1,
"kind": "failed",
"terminal": true,
"stage": "setup|bridge|runtime",
"error_code": "owner_unreachable|stream_bridge_failed|runtime_start_failed|...",
"message": "..."
}

The exact event payload may reuse translated runtime and agent event shapes where
practical.

## Required Semantics And Invariants

### 1. substrate -c semantics must not change

This slice must not reinterpret -c, pipe mode, or plain stdin as agent prompting.

### 2. Exact selectors only

Every start or turn path must resolve by exact backend_id, and follow-up turn paths must
additionally resolve by exact orchestration_session_id.

No fallback to:

- latest session,
- only live session,
- only live backend,
- agent id,
- or trace-derived history.

### 3. Prompt validation happens before runtime launch

Both start and turn must validate prompt presence and prompt source shape before launching
or resuming any runtime.

Required rules:

- malformed prompt source is a pre-launch failure,
- empty effective prompt is a pre-launch failure,
- and neither command may replace invalid or missing prompt input with synthetic bootstrap
  text.

### 4. Follow-up ownership must remain single-writer

A follow-up turn must not create competing retained owners for the same exact backend slot
inside the same exact orchestration session.

Required rule:

- if a live owner already exists for the exact session/backend target, submit through that
  owner or fail closed,
- do not silently create a second retained owner for the same live target.

### 5. Host and world follow-up paths must stay distinct

The public non-interactive surface must preserve current runtime truth:

- host follow-up may use local retained ownership paths,
- Linux world follow-up must continue to rely on the exact retained member-turn seam or an
  equivalent owner path that ultimately submits through the same typed world-member
  contract,
- non-Linux world-sensitive follow-up must fail closed.

### 6. Live runtime state remains authoritative

Follow-up authorization must be derived from authoritative persisted runtime state, not
trace fallback.

### 7. Policy and posture must remain fail-closed

The start, turn, and reattach surfaces must preserve current policy and posture rules:

- root world-only backend rejection remains explicit,
- policy drift remains a hard error,
- non-Linux world-sensitive follow-up remains an explicit posture error,
- detached sessions remain reattachable only when authoritative recovery metadata is
  intact,
- and strict selector surfaces stay fail-closed.

## Recommended Implementation Shape

1. Extend AgentStartArgs so public start requires exact backend_id plus one exact prompt
   source.
2. Add AgentAction::Turn under the existing public agent namespace.
3. Rename the target public meaning of the current resume lifecycle command to reattach.
4. Make backend_id mandatory for all turn calls.
5. Make orchestration_session_id mandatory for follow-up turn calls and reattach.
6. Reuse the existing authoritative session registry and exact public control selectors
   where they still fit.
7. Add a new or extended authoritative state-store resolver for exact
   (orchestration_session_id, backend_id) follow-up targeting.
8. Reuse the landed host and world turn-submission behavior, but do not hard-code this SOW
   to any one startup helper or helper-specific call graph.
9. Add whatever private streamed bridge is necessary so the foreground caller sees live
   output while the exact retained target handles the turn.
10. Surface resulting session_posture explicitly so session-lifetime truth at completion
    time is visible and testable.

## Concrete Work Breakdown

### 1. Change the public start CLI shape

Update public start in ../crates/shell/src/execution/cli.rs:456 so it requires:

- --backend <backend_id>
- exactly one of:
  - --prompt <text>
  - --prompt-file <path>
  - --prompt-file -
- optional --json

### 2. Implement prompt-required root start as a foreground call

For start --backend ...:

- validate exact host-scoped backend selection,
- validate exact prompt source before runtime launch,
- reject world-only backends with the same posture classification already used by public
  root start,
- create a new orchestration session,
- submit exactly one inaugural real prompt,
- stream output to the caller,
- return the new orchestration_session_id,
- and surface the resulting session_posture explicitly.

This slice requires the behavior, not one particular helper chain.

### 3. Remove the root turn --backend form

Public turn becomes follow-up only.

There is no supported public root form:

substrate agent turn --backend <backend_id> ...

That root prompt-taking role now belongs to start.

### 4. Implement exact follow-up targeting by session plus backend

For turn --session ... --backend ...:

- resolve the exact orchestration session through authoritative state,
- resolve the exact backend target within that session through the new or extended backend-
  aware authoritative resolver,
- validate exact prompt source before runtime launch or resume,
- submit one follow-up turn through the current retained target if live,
- otherwise use the existing exact public recovery posture only if the target remains
  reattachable,
- otherwise fail closed.

### 5. Rename and preserve the operational recovery command as reattach

For reattach --session ...:

- preserve the current operational recovery behavior of the existing public resume path,
- keep the command non-prompt-taking,
- keep it session-scoped,
- keep it fail-closed when the selected session is already live-owned, lacks required
  internal session metadata, or is otherwise not recoverable,
- and update the public terminology and docs so this behavior is no longer described as
  conversational “resume.”

### 6. Add a real streamed bridge

If the exact start or turn target is not directly owned by the invoking CLI process, add a
streamed return path that bridges runtime output back to the caller.

This is mandatory for both human mode and JSON mode.

### 7. Preserve status and read-side coherence

After any successful or failed start, turn, or reattach, the resulting session must remain
coherent in substrate agent status:

- new active sessions appear as live,
- detached reattachable sessions appear as detached and coherent rather than disappearing,
- terminal sessions appear with truthful final state,
- strict selector surfaces remain fail-closed.

## Failure Posture

The surface must fail closed in these cases:

1. Unknown backend
2. Ambiguous backend
3. World-only backend passed to root start --backend
4. Missing --backend
5. Missing prompt source
6. Malformed prompt source
7. Empty effective prompt
8. Follow-up mode invoked without --session
9. Unknown session in follow-up mode
10. Non-canonical follow-up session selector
11. Exact backend not present in the selected follow-up session
12. Exact backend slot stale or ambiguous inside the selected follow-up session
13. Missing active parent
14. Missing internal session id for reattachable follow-up recovery
15. Live owner unreachable
16. Unsupported platform or posture for world-sensitive follow-up
17. Policy disallow or policy drift
18. Stream bridge establishment failure

No failure case may fall back to shell execution, implicit REPL grammar, best-effort
session discovery, or synthetic bootstrap text as agent input.

## Acceptance Criteria

This slice is done when all of the following are true:

1. substrate agent start is a public prompt-taking surface.
2. start requires exact --backend <backend_id> plus exactly one prompt source.
3. start creates a new host-scoped orchestration session and submits the provided prompt as
   the inaugural real turn.
4. start streams caller-visible output in the foreground.
5. start returns the new orchestration_session_id.
6. start does not create an empty live session with no real prompt.
7. start does not inject synthetic bootstrap text as public agent input.
8. start remains host-only in v1 and rejects world-only backends explicitly.
9. substrate agent turn is follow-up only.
10. turn requires both --session <orchestration_session_id> and --backend <backend_id> plus
    exactly one prompt source.
11. turn uses a new or extended authoritative backend-aware resolver rather than session-
    only control resolution.
12. turn can target exact retained backends within an existing orchestration session
    without relying on session id alone.
13. substrate agent reattach is the public lifecycle recovery surface for restoring
    retained ownership to an existing session.
14. reattach does not take a prompt and does not submit a conversation turn.
15. The caller receives real streamed output during start and turn execution.
16. --json is a line-delimited streaming contract with stable version and kind.
17. Setup or bridge failures after JSON stream start emit one terminal kind="failed" record
    in-stream.
18. Every successful start or turn reports resulting session_posture as active,
    detached_reattachable, or terminal, authoritative at command completion time.
19. substrate agent status surfaces the resulting session coherently after start, turn, and
    reattach operations.
20. Strict control and read-side selector contracts do not regress: non-canonical selectors
    still fail closed.
21. Linux world-sensitive follow-up can succeed only through the exact landed world follow-
    up posture.
22. Non-Linux world-sensitive follow-up still fails closed.
23. Human-mode streaming and JSON-mode streaming both preserve the -c non-agent contract by
    construction because prompt-taking remains available only through substrate agent start
    and substrate agent turn.

## Testing Expectations

At minimum, this slice should add targeted regression coverage for:

- CLI parse coverage for prompt-required agent start
- CLI parse coverage for follow-up-only agent turn
- CLI parse or alias coverage for agent reattach
- exact required backend_id
- exact required prompt source
- rejection of missing, malformed, or empty prompt input before runtime launch
- rejection of world-only root backends
- host root start success with streamed output
- exact follow-up success into a live host-owned backend
- exact follow-up success into a Linux world-retained backend when authoritative posture is
  valid
- fail-closed non-Linux world-sensitive follow-up
- explicit session_posture postconditions at command completion time
- agent status coherence after successful and failed start, turn, and reattach operations
- no regression to strict fail-closed selector behavior on reattach|fork|stop|toolbox
- no regression to -c wrap-mode semantics, with both human-mode and JSON-mode start and
  turn paths remaining structurally separate from shell command mode
- no synthetic bootstrap text being used as public agent input on start or turn

Validation expectations should also include the repo’s required operator surfaces on
touched platforms and postures:

substrate agent doctor --json
substrate shim doctor --json
substrate world doctor --json
substrate health --json

Where the slice touches world-sensitive follow-up behavior, capture Linux-first validation
evidence and ensure these surfaces remain coherent with the resulting session and state
truth.

## Summary

The missing thin slice is not “teach -c to talk to agents.” The missing thin slice is one
explicit public prompt-taking model where:

- start means new session plus first real prompt,
- turn means existing session plus next turn,
- reattach means restore retained ownership to an existing session,
- exact backend_id targeting is mandatory for prompt-taking calls,
- exact orchestration_session_id is mandatory for follow-up turns and recovery calls,
- no default backend, fuzzy matching, or implicit latest-session behavior exists,
- malformed or empty prompt input fails before runtime launch,
- synthetic bootstrap text is not accepted as public agent input,
- the current owner-loop recovery behavior is no longer mislabeled as conversational
  resume,
- and the repo’s current fail-closed control and status posture remains intact.
