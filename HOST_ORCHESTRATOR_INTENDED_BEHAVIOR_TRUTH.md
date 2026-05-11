# Host Orchestrator Intended Behavior Truth

## Purpose

This document records the intended behavior for the Substrate host orchestrator session model exactly as clarified during review of:

- [docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/home/spenser/__Active_code/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/home/spenser/__Active_code/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md)
- [llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/home/spenser/__Active_code/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/home/spenser/__Active_code/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)

This document is not a design brainstorm. It is a truth record of intended behavior and the currently known unfinished gaps.

## Core Model

The host orchestrator session is a Substrate-owned durable session.

The durable authority is:

- the Substrate orchestration session record
- the authoritative participant linkage
- the durable inbox / task state
- the Substrate-owned routing and lifecycle state

The durable authority is not:

- one currently attached backend process
- one currently running `codex exec` process
- one currently running helper PID

A Codex-backed host process is an attachable execution client. It may attach, run a prompt, exit cleanly, and later resume against the same durable orchestration session.

## Meaning Of `agent start`

`substrate agent start --backend <backend_id> --prompt ...` starts the host orchestrator session.

The command is intended to:

- create or bind the durable Substrate-owned orchestration session
- run the user's initial prompt as the true initial backend prompt
- establish the orchestration session as active and durable
- leave that orchestration session available for future orchestration work until explicit stop

The command is not intended to:

- send a hidden bootstrap prompt as the real backend user prompt
- wrap the user's prompt inside hidden control instructions and treat that wrapped text as the real initial backend prompt
- treat the backend process itself as the durable orchestration authority
- end the orchestration session merely because the initial prompt-bounded backend process exits

## Prompt Semantics

For a Codex-style backend:

- the initial user prompt for `agent start --prompt ...` must map to the real initial `codex exec` prompt semantics
- follow-up prompt-taking must map to resume semantics

That means:

- `agent start --prompt ...` must use the user prompt as the true initial backend prompt
- `agent turn --session ... --backend ... --prompt ...` is the follow-up prompt-taking resume path
- `agent reattach --session ...` is not a prompt-taking action

The helper launched by `agent start` exists for Substrate-owned session setup, state bookkeeping, routing, and lifecycle management. It is not supposed to replace or rewrite the user's initial backend prompt semantics.

## Session Lifetime

When `agent start` succeeds, the orchestration session should remain open until:

- `agent stop` is run
- or a future explicit stale/timeout lifecycle is added and deliberately transitions it

The session should not stop being an open orchestration session simply because:

- the initial `codex exec` process finished
- the foreground helper is no longer actively babysitting world agents
- the orchestrator is currently idle

## Meaning Of `parked_resumable`

`parked_resumable` does not mean the orchestration session ended.

`parked_resumable` means:

- the orchestration session is still active and durable
- no foreground attached host execution client is currently babysitting the session
- the session remains routable and resumable
- Substrate still owns the authoritative session state
- the session remains available to receive requests, messages, updates, approvals, completions, and other orchestration responsibilities

More simply:

- parked means idle or detached
- parked does not mean gone
- parked does not mean terminal

## Meaning Of `active_attached`

`active_attached` means:

- the orchestration session is active
- an attached host execution client is currently present
- that attached client can immediately receive prompt traffic and foreground orchestration work

## Meaning Of `awaiting_attention`

`awaiting_attention` means:

- the orchestration session is still active and durable
- no host client is currently attached
- pending durable inbox work exists
- host-side review or resumption is needed

## Meaning Of `terminal`

`terminal` means:

- the orchestration session is no longer routable
- the orchestration session is closed, failed, invalidated, or otherwise no longer available for orchestration work

## Parked Session Responsibilities

While parked, the orchestrator session is still supposed to represent a live durable authority that can continue owning world-agent responsibilities.

That means parked is intended to remain capable of:

- receiving world-originated messages
- receiving world-originated updates
- receiving approvals
- receiving completion notices
- receiving follow-up work
- retaining those items durably while no host client is attached
- allowing later `turn` or `reattach` against that same session

The orchestrator should not need to stay foreground-attached just to keep the session valid.

## Meaning Of `agent turn`

`substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` is the exact follow-up prompt-taking resume path.

It is intended to:

- target one exact durable orchestration session
- resume prompt-taking work against that same session
- preserve the durable session identity
- allow the session to return to parked afterward if no attached client remains

`turn` is not intended to:

- create a fuzzy new session
- bypass the durable session record
- require the original initial backend process to still be running

## Meaning Of `agent reattach`

`substrate agent reattach --session <orchestration_session_id>` is attached-owner recovery only.

It is intended to:

- restore attached host ownership for the already-existing durable orchestration session
- leave the same durable session active
- make the session truly attached again

It is not intended to:

- submit a prompt
- implicitly consume inbox work
- merely return success while the session immediately falls back to parked

If `reattach` reports success, the intended truth is that attached ownership was actually restored.

## Meaning Of `agent stop`

`substrate agent stop --session <orchestration_session_id>` is the explicit shutdown action for the durable orchestration session.

It is intended to:

- stop the durable orchestration session cleanly
- be the canonical closeout path for an active orchestration session
- work for the durable session model, not only for a currently attached live owner process

The session is intended to remain open until `stop` is run, subject only to future explicit stale/timeout lifecycle rules if those are later designed and implemented.

## Meaning Of `agent status`

`substrate agent status --json` is intended to surface the durable host orchestration session truth.

That includes parked sessions.

A parked session is still supposed to be visible as a real durable orchestration session, not disappear merely because no attached live owner process is currently present.

## Durable Inbox Expectations

The durable inbox is intended to be the Substrate-owned retained surface for orchestration-relevant events while no host client is attached.

That includes:

- approvals
- completion notices
- follow-up messages
- runtime alerts
- other orchestration-relevant retained work

If no host client is attached, those events are still supposed to land durably and remain actionable later.

## Current Unfinished Gaps

Based on the current validated behavior, the following are still unfinished:

### 1. `reattach` is not fully landed

Current observed behavior:

- `reattach` can return success
- but the session can still immediately end up parked again
- the resumed participant can quickly show detached / non-retained diagnostics
- attached ownership is not yet reliably restored in the persisted runtime truth

What still needs to be true:

- if `reattach` succeeds, attached ownership must actually be restored
- a successful `reattach` must not merely mean "a helper briefly ran and then reparks immediately"

### 2. `stop` is not fully landed for the durable parked-session model

Current observed behavior:

- `stop` is still tied too closely to the currently attached owner-control plane
- parked or stale-attached sessions can fail stop resolution instead of stopping cleanly

What still needs to be true:

- `stop` must work for the durable orchestration session model
- it must not require the old attached-live-only model in order to stop a still-valid active session

### 3. `status` projection is not fully landed

Current observed behavior:

- parked sessions can disappear from `agent status --json`

What still needs to be true:

- parked sessions must remain visible as durable active orchestration sessions
- status projection must be based on authoritative session truth, not only on attached-live participant truth

### 4. Parked durability for world-agent responsibility is not fully proven operationally

Current observed behavior proves some important parts:

- `start` now uses the user prompt as the true initial backend prompt
- a successful start can park the session instead of invalidating it
- `turn` can resume that parked session

What is not yet fully proven / landed:

- that the parked durable session is fully functioning as the live Substrate-owned orchestrator authority for ongoing world-agent responsibilities in the stronger sense described above
- especially while `reattach`, `stop`, and `status` still do not match the intended model

## Intended Acceptance Shape

The intended final shape is:

1. `agent start --prompt ...` starts a durable active orchestration session and runs the user prompt as the true initial backend prompt.
2. That session remains open until explicit `agent stop` or a future explicitly designed stale/timeout lifecycle.
3. The session may be `active_attached`, `parked_resumable`, or `awaiting_attention` while still remaining an active durable orchestration session.
4. `parked_resumable` means the session is idle/detached but still alive, routable, resumable, and still Substrate-owned for orchestration responsibilities.
5. `turn` resumes prompt-taking against that same durable session.
6. `reattach` restores actual attached host ownership for that same durable session without submitting a prompt.
7. `stop` cleanly stops that same durable session.
8. `status` shows that same durable session even while parked.
9. Durable inbox items continue to land and remain actionable while no host client is attached.

## Non-Negotiable Truths

The following are the key truths this repository still needs to honor:

- the durable authority is the Substrate orchestration session, not one backend process
- `agent start` is not supposed to end the orchestration session when the initial prompt-bounded backend run exits
- parked does not mean gone
- parked does not mean terminal
- parked means still alive as a durable orchestration session
- world agents must be able to keep working without requiring the host orchestrator to foreground-babysit them continuously
- the parked durable session must still be available to receive requests, messages, updates, approvals, completions, and follow-up orchestration work
- `reattach` must actually restore attachment if it reports success
- `stop` must be the real closeout path for the durable orchestration session
- `status` must represent the durable parked session truth
