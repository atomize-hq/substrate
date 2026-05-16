# SOW: Host Durable Session Closeout And QA Hardening

Status: truth-aligned remaining-work draft. This SOW closes the next narrow slice after [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md) and [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md). It is anchored to [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), and [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md).

This is not a redesign slice. The durable session model already exists. The public verbs already exist. Parked sessions already persist. `substrate agent status --json` already projects the key durable-session fields. Canonical inbox item storage already exists. The remaining work is to regression-proof the already chosen contract and keep docs, tests, and operator guidance aligned to it.

## Frozen Public Contract

This slice assumes and preserves the following contract:

1. `substrate agent start --backend <backend_id> --prompt ...` creates or binds the durable host orchestration session and uses the user prompt as the true initial backend prompt.
2. `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...` is prompt-taking follow-up on that same durable session.
3. `substrate agent reattach --session <orchestration_session_id>` is attached-owner recovery only for that same durable session.
4. `substrate agent stop --session <orchestration_session_id>` is the canonical closeout path for attached and parked durable host sessions.
5. `substrate agent status --json` is the authoritative parked-session read surface for live-runtime `posture`, `attached_participant_id`, and `pending_inbox_count`.
6. Detached-world follow-up stays fail-closed until `reattach` restores an active host owner.
7. Durable inbox behavior stays narrow: persistence exists, posture normalization into `awaiting_attention` exists, internal ack/dismiss support exists, and dev-support/test ingress exists, but no public inbox command surface or automatic resume-from-inbox workflow is shipped.

## Objective

Finish the durable host-session closeout slice by making the frozen contract above regression-proof in code, tests, and docs.

This slice is done only when all of the following are true:

1. The repo stops describing parked-session recovery semantics as open design.
2. `stop` keeps working correctly for the same durable session whether it is `active_attached`, `parked_resumable`, or `awaiting_attention`.
3. The already-landed `status` projection is treated as a hardening seam, not an open feature seam.
4. Detached-world follow-up continues to fail closed until `reattach` restores an active host owner.
5. The repo stops implying that persisted inbox scaffolding means a public inbox workflow or automatic runtime resume mechanism.

## Already Landed And Assumed

This SOW assumes the following are already true and are not being redesigned here:

- `substrate agent start`, `turn`, `reattach`, `fork`, and `stop` already exist as public verbs in [cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:520).
- Public follow-up targeting remains exact `(orchestration_session_id, backend_id)`.
- Durable parked host postures already exist in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:89).
- `substrate agent status --json` already publishes authoritative `posture`, `attached_participant_id`, and `pending_inbox_count` for live-runtime rows, and trace fallback rows keep those fields unset as documented in [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:84).
- Detached authoritative parked sessions already remain status-visible from canonical session truth via [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:66).
- Detached posture classification already comes from persisted session plus participant truth through [classify_public_session_posture(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2191) and [valid_detached_host_continuity_posture(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216).
- Session-local inbox persistence already exists through [persist_inbox_item(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039), along with load/list/ack/dismiss support and posture updates.

## Current Repo Truth

### `status` is closed as a product contract

This slice must not describe `status` as a primary missing feature.

What is already true:

- `build_status_report(...)` reads live session rows from `state_store.list_status_sessions_for_agent(...)` before trace fallback in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1333).
- parked and awaiting-attention sessions remain visible from authoritative session-root truth in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:3695).
- trace fallback does not invent durable-session posture truth in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1389).

The remaining work on `status` is only:

- protect that behavior from regression,
- prove parked sessions stay correctly visible as control and readiness code evolves,
- and keep docs accurate about what those fields do and do not mean.

### Inbox persistence is real and intentionally narrow

What is clearly real today:

- canonical inbox item persistence under the session root,
- `pending_inbox_count` as durable state,
- posture normalization from parked to `awaiting_attention` when pending work exists,
- internal ack/dismiss support for retained items,
- and dev-support/test helpers proving those mechanics exist.

What this slice must keep explicit:

- no public inbox command surface is shipped,
- no public inbox-driven automatic resume path is shipped,
- and docs must not imply broader productization than the code currently implements.

### Detached-world follow-up remains fail-closed

The current supported rule is narrow:

- detached host recovery uses `reattach`,
- host prompt-taking follow-up uses `turn`,
- and detached-world follow-up remains fail-closed until `reattach` restores an active host owner.

This slice must preserve that fail-closed rule rather than soften it.

## In Scope

- keep the `turn` versus `reattach` contract frozen and documented truthfully,
- harden `reattach` so success means real durable attached truth for the named session,
- keep `stop` first-class for valid parked sessions,
- regression-proof the already-landed `status` projection,
- keep detached-world follow-up fail-closed until `reattach`,
- and freeze the minimum supported inbox contract to persistence, posture normalization, internal ack/dismiss, and dev-support/test ingress only.

## Out Of Scope

This slice does not include:

- fuzzy routing or default-agent routing,
- new root selector forms,
- public world-root `start`,
- a broad new inbox UX,
- a public inbox command surface,
- automatic resume from inbox items,
- detached-world follow-up without `reattach`,
- durable inbox schema redesign,
- a daemon/router redesign,
- or another durable-session model rewrite.

## Concrete Work Breakdown

### 1. Keep the public parked-session contract frozen

Primary anchors:

- `run_turn(...)` and `run_reattach(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:492)
- target resolution in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:892)
- intended behavior notes in [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)

Required outcome:

- docs and operator guidance consistently say `turn` is prompt-taking follow-up on the same durable session,
- `reattach` remains attached-owner recovery only,
- and no packet or truth doc reopens that contract as a product-design choice.

### 2. `reattach` success must mean attached truth is actually restored

Primary anchors:

- [run_reattach(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:565)
- [wait_for_hidden_owner_helper_readiness(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:645)
- detached continuity validation in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216)

Required outcome:

- `reattach` does not report success until the durable session has truly converged back to `active_attached`,
- the same orchestration session remains the winning session before and after reattach,
- and immediate follow-up `status`, `turn`, and `stop` work against that same session without requiring a second recovery step.

### 3. Durable `stop` must remain first-class for parked sessions

Primary anchors:

- [run_stop(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:667)
- stop closeout handling in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:867)
- public target resolution in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:765)

Required outcome:

- `stop` works for `active_attached`, `parked_resumable`, and `awaiting_attention`,
- the detached parked path does not require a currently live attached-owner PID when durable truth already says the session is valid,
- and the durable parent session closes terminally under the same orchestration session id.

### 4. Keep the inbox contract narrow

Primary anchors:

- canonical inbox persistence in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1039)
- posture updates and pending-count normalization in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1726)
- current dev-support seam in [agent_dev_support.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs:1)

Required outcome:

- the repo explicitly states that shipped inbox behavior is limited to persistence, posture normalization, internal ack/dismiss support, and dev-support/test ingress,
- no doc implies a public inbox command surface,
- and no doc implies automatic resume or public workflow productization from inbox items.

### 5. Keep detached-world follow-up fail-closed until `reattach`

Primary anchors:

- detached-world guidance in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:535)
- detached continuity validation in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2216)
- current intended behavior notes in [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)

Required outcome:

- the repo keeps one explicit current-state rule for detached-world follow-up,
- that rule stays fail-closed until `reattach` restores an active host owner,
- and no doc softens that rule into implicit resume or auto-reattach semantics.

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
- parked session -> `reattach` succeeds only when durable `active_attached` truth is actually restored,
- a successful `reattach` leaves immediate `turn` and `stop` usable against the same session,
- parked session -> `stop` succeeds without a live attached-owner plane,
- awaiting-attention parked session -> `stop` also succeeds,
- detached-world follow-up stays fail-closed until `reattach`,
- and parked retained inbox state does not imply a public inbox workflow or automatic resume path.

### Store-level coverage in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required scenarios:

- detached host continuity remains a valid public control target only when the persisted resume contract is intact,
- detached parked participants remain status-visible from authoritative session truth when owner PID liveness is absent,
- inbox persistence and resolution keep `pending_inbox_count` and posture synchronized under the supported item kinds,
- and contradictory session/participant linkage still fails closed for control while remaining readable on status when appropriate.

### Runtime lifecycle coverage in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required scenarios:

- detached parking normalization preserves the exact fields later `turn`, `reattach`, `stop`, and `status` depend on,
- pending inbox escalation keeps the same session in `awaiting_attention` without invalidation,
- and closeout paths do not regress parked durable sessions back into attached-live-only assumptions.

### Inbox realism coverage

Required scenarios:

- at least one supported non-test runtime path proves how a real inbox item is created for a parked session,
- that proof stays limited to persistence and posture normalization unless broader public behavior is intentionally shipped in a later slice,
- and tests do not use dev-support helpers to imply a public inbox workflow or automatic runtime resume story that does not exist.

## Acceptance Criteria

- the repo keeps one explicit public parked-session recovery contract: `turn` for prompt-taking follow-up and `reattach` for attached-owner recovery.
- `reattach` success always corresponds to real durable `active_attached` truth for the same orchestration session.
- `stop` works for valid parked host sessions without requiring a live attached owner process.
- the already-landed `status` projection remains authoritative and non-regressive for parked sessions.
- detached-world follow-up stays fail-closed until `reattach` restores an active host owner.
- the minimum supported inbox contract is explicit and limited to persistence, posture normalization, internal ack/dismiss support, and dev-support/test ingress.
- docs and tests stop implying a public inbox workflow, automatic inbox-driven resume, or any detached-world follow-up path that bypasses `reattach`.

## Validation Expectations

- add or tighten targeted tests in the files above for each acceptance path,
- run the touched shell test coverage and full workspace tests:
  - `cargo test --workspace -- --nocapture`
- manual validation for this slice must explicitly exercise:
  - parked `start`,
  - parked `status`,
  - parked `turn`,
  - parked `reattach`,
  - parked `stop`,
  - and detached-world fail-closed follow-up before and after `reattach`.

## Docs And Truth Sync

When this slice is closed, the truth docs must stop advertising uncertainty that the code has already resolved and must stop implying inbox behavior that is not actually supported.

At minimum:

- update [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md) to reflect the frozen parked-session recovery contract,
- update [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) so its remaining-gap wording reflects the true post-closeout state,
- and update [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md) so operator guidance matches the frozen semantics for parked recovery, `stop`, detached-world fail-closed behavior, and narrow inbox state.

## Done Shape

This slice is complete when the repo no longer hand-waves any of these points:

- `turn` is prompt-taking follow-up on the same durable session,
- `reattach` is attached-owner recovery only,
- `stop` is the canonical closeout path for attached and parked durable host sessions,
- `status --json` is the authoritative parked-session read surface,
- detached-world follow-up stays fail-closed until `reattach`,
- and inbox behavior is documented as narrow retained state rather than as a public inbox product or automatic resume mechanism.
