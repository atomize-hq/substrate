# SOW: Host Durable Session Closeout And Inbox Resume Contract

Status: remaining-work draft. This SOW closes the next narrow slice after [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md) and [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md). It is anchored to [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), and [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md).

This is not a redesign slice. The durable session model already exists. Public verbs already exist. Parked sessions already persist. `substrate agent status` already projects the key durable-session fields. Canonical inbox item storage already exists. What remains is to close the operational contract around parked-session recovery and retained work.

## Objective

Finish the durable host-session model by closing the remaining contract ambiguity around:

- whether `reattach` is truly needed as a public verb,
- how `stop` behaves for valid parked sessions,
- what the minimum real inbox contract is today,
- and how parked world-originated work resumes the same orchestration session.

This slice is done only when all of the following are true:

1. The repo chooses one explicit public recovery contract for parked sessions.
2. `stop` works correctly for the same durable session whether it is `active_attached`, `parked_resumable`, or `awaiting_attention`.
3. The already-landed `status` projection is treated as a regression-proofing seam, not an open feature seam.
4. The repo stops implying that persisted inbox scaffolding automatically means a complete runtime resume mechanism.
5. The supported path for parked retained work is explicit in code, tests, and docs.

## Already Landed And Assumed

This SOW assumes the following are already true and are not being redesigned here:

- `substrate agent start`, `turn`, `reattach`, `fork`, and `stop` already exist as public verbs in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:520).
- Public follow-up targeting remains exact `(orchestration_session_id, backend_id)`.
- Durable parked host postures already exist in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:89).
- `substrate agent status --json` already publishes authoritative `posture`, `attached_participant_id`, and `pending_inbox_count` for live-runtime rows, and trace fallback rows keep those fields unset as documented in [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:84).
- Detached authoritative parked sessions already remain status-visible from canonical session truth via [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:66).
- Detached posture classification already comes from persisted session plus participant truth through [classify_public_session_posture(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191) and [valid_detached_host_continuity_posture(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216).
- Session-local inbox persistence already exists through [persist_inbox_item(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039), along with load/list/ack/dismiss support and posture updates.

## What Is Actually Still Open

The remaining gap is not “build status” and it is not “invent a new durability model.” The remaining gap is that the repo still leaves too much room for interpretation in the parked-session control story.

The open questions are:

1. Is `reattach` a real long-term public verb, or should recovery collapse onto `turn`?
2. Is `stop` fully correct for detached but still-valid parked sessions?
3. Is the inbox merely durable scaffolding plus posture input, or is there a supported production resume path for inbox-originated work?
4. What is the exact current-state rule for world-originated work that arrives while the host orchestrator is parked?

## Current Repo Truth

### `status` is mostly closed

This slice must not describe `status` as a primary missing feature.

What is already true:

- `build_status_report(...)` reads live session rows from `state_store.list_status_sessions_for_agent(...)` before trace fallback in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1333).
- parked and awaiting-attention sessions remain visible from authoritative session-root truth in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3695).
- trace fallback does not invent durable-session posture truth in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1389).

The remaining work on `status` is only:

- protect that behavior from regression,
- prove parked sessions stay correctly visible as control and readiness code evolves,
- and keep docs accurate about what those fields do and do not mean.

### Inbox persistence is real, but inbox resume semantics are not fully closed

What is clearly real today:

- canonical inbox item persistence under the session root,
- `pending_inbox_count` as durable state,
- posture normalization from parked to attention-needed when pending work exists,
- and tests/dev-support helpers proving those mechanics exist.

What does not yet look fully closed:

- which production runtime paths create inbox items,
- which inbox item kinds are actually supported today,
- whether inbox work only surfaces retained attention state or can directly resume execution,
- and whether any operator-facing consume/resolve path beyond tests/dev support is already meant to be part of the supported v1 contract.

This slice must close that wording gap. It does not have to build a broad inbox UX. It does have to stop the repo from implying a richer mechanism than the code actually implements.

## Required Product Decision

This slice must explicitly choose one parked-session recovery model and then align code, tests, and docs to it.

### Option A: `reattach` stays public

`reattach` remains a real public operator action with a distinct meaning:

- restore an attached host owner loop without submitting a prompt,
- re-establish attached ownership truth for the same durable session,
- and remain separate from prompt-taking `turn`.

If this option is chosen, `reattach` must have strong success semantics. It cannot report success if the helper briefly wakes and the session immediately reparks or never converges to real `active_attached` truth.

### Option B: public recovery collapses onto `turn`

`turn` becomes the only meaningful public parked-session resume action:

- the parked session remains durable and visible,
- prompt-taking follow-up occurs through `turn`,
- and any attachment restoration is treated as an internal implementation detail rather than a public lifecycle verb.

If this option is chosen, `reattach` must be demoted or removed from the public contract, and detached-world follow-up guidance must stop implying that users need a distinct attach step unless there is still a concrete supported case for it.

This SOW does not assume which option is correct. It requires the repo to choose one and make the rest of the system consistent with that choice.

## In Scope

- choose and freeze the public parked-session recovery contract,
- harden `reattach` if it remains public,
- or demote/remove `reattach` cleanly if recovery intentionally collapses onto `turn`,
- make `stop` fully correct for valid parked sessions,
- freeze the minimum supported inbox contract,
- prove the resume contract for world-originated work against parked sessions,
- and regression-proof the already-landed `status` projection.

## Out Of Scope

This slice does not include:

- fuzzy routing or default-agent routing,
- new root selector forms,
- public world-root `start`,
- a broad new inbox UX,
- durable inbox schema redesign,
- a daemon/router redesign,
- or another durable-session model rewrite.

## Concrete Work Breakdown

### 1. Freeze The Parked-Session Recovery Contract

Primary anchors:

- `run_turn(...)` and `run_reattach(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:492)
- target resolution in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:892)
- intended behavior notes in [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)

Required outcome:

- one exact current-state recovery contract is chosen,
- the contract says whether `reattach` is required, optional, or obsolete,
- and operator guidance matches the implemented behavior instead of a hoped-for future behavior.

### 2. If `reattach` stays, it must restore real attached ownership

Primary anchors:

- [run_reattach(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:565)
- [wait_for_hidden_owner_helper_readiness(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:645)
- detached continuity validation in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216)

Required outcome:

- `reattach` does not report success until the durable session has truly converged back to `active_attached`,
- the same orchestration session remains the winning session before and after reattach,
- and immediate follow-up `status`, `turn`, and `stop` work against that same session without requiring a second recovery step.

Failure discipline:

- if the helper only briefly wakes,
- if ownership truth does not converge,
- or if the session immediately reparks in a way that violates the chosen contract,
- `reattach` must fail rather than claim success.

### 3. Durable `stop` must be first-class for parked sessions

Primary anchors:

- [run_stop(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:667)
- stop closeout handling in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:867)
- public target resolution in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:765)

Required outcome:

- `stop` works for `active_attached`, `parked_resumable`, and `awaiting_attention`,
- the detached parked path does not require a currently live attached-owner PID when durable truth already says the session is valid,
- and the durable parent session closes terminally under the same orchestration session id.

### 4. Freeze The Minimum Real Inbox Contract

Primary anchors:

- canonical inbox persistence in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039)
- posture updates and pending-count normalization in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1726)
- current dev-support seam in [agent_dev_support.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:1)

Required outcome:

- the repo explicitly states which runtime paths create inbox items today,
- the repo explicitly states which inbox item kinds are actually supported today,
- the repo explicitly states whether pending inbox work only produces `awaiting_attention` or can also directly resume execution,
- and the repo explicitly states what consumes or resolves the retained work under the chosen contract.

Non-goal:

- this does not require a polished inbox command surface unless that is the minimal thing needed to make the supported contract real.

### 5. Prove The Parked World-Originated Resume Contract

Primary anchors:

- detached-world guidance in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:535)
- detached continuity validation in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216)
- current intended behavior notes in [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)

Required outcome:

- the repo proves one supported current-state rule for work that arrives while the orchestrator is parked.

At minimum, the rule must answer:

- does parked retained work require explicit `turn`,
- does some class of retained work still require explicit `reattach`,
- or is any automatic resume path actually supported today?

This slice must not leave those answers implied.

### 6. Treat `status` as hardening, not greenfield

Primary anchors:

- [build_status_report(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1270)
- [status_visible_participants(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:66)

Required outcome:

- the current authoritative parked-session fields remain correct,
- parked sessions stay visible from canonical session truth,
- and later control/readiness changes do not reintroduce attached-live-only assumptions.

## Required Test Additions Or Tightening

### Integration coverage in [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

Required scenarios:

- parked `start` -> `status --json` still shows the same orchestration session as parked,
- parked `start` -> `turn` -> parked again stays on the same orchestration session id,
- parked session with pending inbox work -> `status --json` shows `awaiting_attention`,
- if `reattach` remains public: parked session -> `reattach` succeeds only when durable `active_attached` truth is actually restored,
- if `reattach` remains public: a successful `reattach` leaves immediate `turn` and `stop` usable against the same session,
- parked session -> `stop` succeeds without a live attached-owner plane,
- awaiting-attention parked session -> `stop` also succeeds,
- and parked retained work remains visible and recoverable under the chosen contract.

### Store-level coverage in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required scenarios:

- detached host continuity remains a valid public control target only when the persisted resume contract is intact,
- detached parked participants remain status-visible from authoritative session truth when owner PID liveness is absent,
- inbox persistence and resolution keep `pending_inbox_count` and posture synchronized under the supported item kinds,
- and contradictory session/participant linkage still fails closed for control while remaining readable on status when appropriate.

### Runtime lifecycle coverage in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required scenarios:

- detached parking normalization preserves the exact fields later recovery depends on,
- pending inbox escalation keeps the same session in `awaiting_attention` without invalidation,
- and closeout paths do not regress parked durable sessions back into attached-live-only assumptions.

### Inbox realism coverage

Required scenarios:

- at least one supported non-test runtime path proves how a real inbox item is created for a parked session,
- the supported consumption or resolution path for that item kind is explicit,
- and tests do not rely only on dev-support helpers to imply a production resume story if one does not exist.

## Acceptance Criteria

- the repo chooses one explicit public parked-session recovery contract.
- if `reattach` remains public, `reattach` success always corresponds to real durable `active_attached` truth for the same orchestration session.
- if `reattach` is demoted or removed, the replacement recovery contract is implemented and documented consistently across code, tests, and user docs.
- `stop` works for valid parked host sessions without requiring a live attached owner process.
- the already-landed `status` projection remains authoritative and non-regressive for parked sessions.
- the minimum supported inbox contract is explicit and matches what the code actually implements.
- the parked-session world-originated resume contract is explicit and regression-proven rather than implied.

## Validation Expectations

- add or tighten targeted tests in the files above for each acceptance path,
- run the touched shell test coverage and full workspace tests:
  - `cargo test --workspace -- --nocapture`
- manual validation for this slice must explicitly exercise:
  - parked `start`,
  - parked `status`,
  - parked `turn`,
  - parked `stop`,
  - and, if retained, parked `reattach`.

## Docs And Truth Sync

When this slice is closed, the truth docs must stop advertising uncertainty that the code has already resolved and must stop implying inbox behavior that is not actually supported.

At minimum:

- update [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md) to reflect the chosen parked-session recovery contract,
- update [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) so its remaining-gap wording reflects the true post-closeout state,
- and update [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md) so operator guidance matches the chosen semantics for parked recovery, `stop`, and retained parked-session work.

## Done Shape

This slice is complete when the repo no longer hand-waves any of these points:

- whether `reattach` is truly needed,
- how parked retained work actually resumes,
- whether parked durable sessions stop cleanly,
- and what inbox behavior is truly supported today versus only persisted as scaffolding.
