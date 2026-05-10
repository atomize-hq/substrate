# SOW: Host Orchestrator Durable Session And Parked-Resumable Ownership

Status: implementation-oriented follow-on draft. This SOW turns [ADR-0047 — Host Orchestrator Durable Session and Parked-Resumable Ownership](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) into an execution slice after the already-landed public caller surfaces in [20-public-non-interactive-agent-caller-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md) and the public turn hardening in [22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md).

The public verbs are already the right verbs: `substrate agent start`, `substrate agent turn`, `substrate agent reattach`, and `substrate agent stop`. The remaining gap is lifecycle truth. The runtime still leans too hard on an attached helper/client process as if attached control retention were the durable orchestration authority itself. This slice corrects that model: the durable unit is the Substrate-owned orchestration session, while a Codex/backend process is an attachable execution client that may exit cleanly without invalidating the session.

## Objective

Make host orchestration durable across clean prompt-driven backend exits by:

- preserving the existing public `start|turn|reattach|stop` contract,
- introducing explicit valid postures for attached, parked, and attention-needed host sessions,
- ensuring world-originated approvals/completions are retained by Substrate even when no host client is attached,
- and tightening the prompt bridge so any request that emits `Accepted` always terminates with an explicit `Completed` or `Failed` envelope.

This slice is not about new public routing syntax. It is about making the already-public caller surface bind to durable orchestration state instead of transient attachment.

## Why This Is Needed

The repo already has the public surface and most of the selector discipline, but the runtime truth is still too attached-process-centric.

- Public prompt-taking is already wired through `run_start(...)`, `run_turn(...)`, `run_reattach(...)`, and `run_stop(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301).
- Exact public follow-up targeting is already enforced by `resolve_public_turn_target(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711), which resolves one authoritative retained slot from exact `(orchestration_session_id, backend_id)`.
- The public prompt bridge already exposes `Accepted`, `Completed`, and `Failed` envelopes in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:266) and drives prompt submission through `run_public_prompt_command(...)` in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063).
- `OrchestrationSessionRecord` still has only `Allocating | Active | Invalidated | Stopping | Stopped | Failed` in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:7), which is not rich enough to express “valid but detached and resumable.”
- Participant liveness is still modeled around attached control retention flags such as `control_owner_retained`, `event_stream_active`, `completion_observer_retained`, and `terminal_observed_at` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81).
- The existing REPL test `start_host_orchestrator_runtime_invalidates_when_attached_control_exits()` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335) documents the current behavior bias directly: attached-control exit still tends to collapse into invalidation rather than a valid parked session.
- World follow-up already depends on resumption plumbing such as `build_session_resume_extension(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:35), but there is not yet a durable Substrate-owned inbox/task posture that cleanly separates “no client attached right now” from “session is dead.”

The result is a mismatch between public contract and runtime lifecycle:

- `start` and `turn` already look like durable orchestration commands,
- but the runtime still risks treating clean host-client exit as owner loss,
- and world-originated follow-up pressure still lacks a first-class durable parked/attention model.

## Relationship To Existing Slices

This SOW consumes and narrows follow-on work after existing `llm-last-mile` slices.

- [19-public-agent-control-surfaces.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/19-public-agent-control-surfaces.md) made the public lifecycle namespace explicit.
- [20-public-non-interactive-agent-caller-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md) defined public prompt-taking via `start`, `turn`, and `reattach`.
- [22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md) hardened the exact public turn path and world-member follow-up routing.

This slice must not reopen those selector or transport decisions. It clarifies what those commands bind to and what durable lifecycle semantics they require.

## Scope

In scope:

- define a durable host orchestration session model that survives clean attached-client exit,
- add explicit attached, parked-resumable, and attention-needed host posture semantics,
- preserve exact public `start`, `turn`, `reattach`, and `stop` command shapes,
- retain world-originated approvals/completions/updates in a Substrate-owned durable inbox, queue, or equivalent task ledger,
- make host `turn` and `reattach` capable of resuming a valid parked host session,
- tighten the post-`Accepted` prompt bridge contract so terminal delivery is explicit on every path,
- add tests and operator-facing docs needed to prove the durable lifecycle behavior.

## Out Of Scope

This slice does not include:

- changing the public selector pair for follow-up turns,
- introducing fuzzy session routing, default backend routing, or latest-session fallback,
- redesigning Linux world-member follow-up away from `MemberTurnSubmitRequestV1`,
- making detached-world follow-up self-sustaining without a valid host orchestration session,
- redesigning REPL grammar,
- designing the full Substrate-native prompt/context builder,
- or defining a remote multi-tenant orchestration control plane.

## Required Runtime Contract

### 1. Public verbs stay stable

The public contract remains:

- `substrate agent start --backend <backend_id> --prompt ...`
- `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ...`
- `substrate agent reattach --session <orchestration_session_id>`
- `substrate agent stop --session <orchestration_session_id>`

This slice must not rename or broaden those verbs.

### 2. The durable authority is the orchestration session, not the attached client

After a session is created successfully, clean exit of the prompt-driven host client must not automatically mean orchestration loss. The durable authority is the Substrate-owned session record plus its retained routing and pending-work state.

### 3. Host posture must be explicit

The runtime must expose semantics equivalent to these postures:

- `active_attached`
  - a host execution client is attached and can receive prompt traffic immediately
- `parked_resumable`
  - the orchestration session is still valid, but no host execution client is currently attached
- `awaiting_attention`
  - the orchestration session is still valid and has pending approvals, completion messages, or other retained work that requires host-side review/resume
- `terminal`
  - the orchestration session is no longer routable

The implementation may choose whether `awaiting_attention` is persisted directly or derived from `parked_resumable + pending durable inbox items`, but the operator-visible behavior must exist.

### 4. Detached-host and detached-world rules stay distinct

- A parked host session is valid and resumable.
- Detached-world follow-up remains fail-closed.
- World follow-up must continue to route through a valid host orchestration session owned by Substrate.

This slice must not use parked-host durability as a reason to loosen detached-world fail-closed posture.

### 5. World-originated delivery must be durable without an attached client

Approvals, completion notices, and similar orchestration-relevant messages from world-side work must land in a Substrate-owned durable inbox, queue, or equivalent persisted task ledger. Live attached clients may consume those events immediately, but absence of an attached client must not silently drop them or invalidate the session.

### 6. Accepted implies explicit terminal delivery

Once a public prompt request emits `Accepted`, the prompt bridge must always terminate with an explicit `Completed` or `Failed` envelope. Stream disappearance, EOF, or helper exit without a terminal envelope is a runtime bug.

## Concrete Work Breakdown

### 1. Introduce durable host session posture separate from raw orchestration state

The current session/orchestration records in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81) and [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) need an explicit way to represent:

- valid attached host ownership,
- valid parked resumable ownership,
- and valid attention-needed ownership,

without overloading terminal state or invalidation state.

This does not require a specific enum layout, but it does require durable state authority that distinguishes “not attached right now” from “session is dead.”

### 2. Stop treating clean attached-client exit as automatic invalidation

The attached host runtime and REPL-owned lifecycle handling in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2572) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335) must be adjusted so that:

- clean prompt-driven host client exit transitions a valid host session into parked-resumable posture,
- explicit stop or true fatal invalidation still transitions to terminal,
- and retained world/member linkage is not rewritten as though the orchestration session itself vanished.

### 3. Add a Substrate-owned durable inbox or task ledger for detached host periods

This slice needs one persisted delivery surface for world-originated work that outlives any attached Codex/backend process. The exact storage shape may be minimal, but it must support:

- durable retention of approvals/completions/updates,
- detection that a parked session now requires attention,
- and later consumption through `turn`, `reattach`, status, or an equivalent sanctioned host resume path.

This is the architectural seam that lets “parked but valid” remain truthful under real world-side activity.

### 4. Make `turn` and `reattach` resume valid parked host sessions

The current public control and prompt path across [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:324), [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711), [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:35) must be updated so that:

- `turn` can resume prompt-taking against a valid parked host session,
- `reattach` can explicitly restore an attached host execution client,
- and exact `(orchestration_session_id, backend_id)` routing remains authoritative.

This slice must not introduce any fuzzy recovery heuristics.

### 5. Harden the public prompt bridge terminal contract

`run_public_prompt_command(...)` in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063) already models `Accepted`, `Completed`, and `Failed`, and it already treats prompt-stream EOF before terminal as `owner_unreachable` in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1418).

This slice must make that a deliberate runtime invariant:

- every path after `Accepted` ends in `Completed` or `Failed`,
- parked-host transitions do not masquerade as owner disappearance,
- and helper/client exit is surfaced through explicit terminal delivery rather than silent stream loss.

### 6. Preserve world fail-closed posture while broadening host durability

Nothing in this slice should weaken:

- host-only root `start` in v1,
- exact public `turn` targeting,
- detached-world follow-up rejection,
- or the existing Linux-first world-member contract.

The only broadened continuity is valid parked host continuity.

### 7. Update tests and operator-facing docs to reflect the corrected lifecycle model

At minimum this slice should update:

- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1083)
- the relevant unit coverage in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1916)
- the relevant unit coverage in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2456)
- the runtime/session lifecycle coverage in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335)

If operator-facing behavior changes materially, update only the narrow docs required to tell the truth about parked/resumable host ownership and accepted-to-terminal guarantees.

## Acceptance Criteria

This slice is done when all of the following are true:

1. `substrate agent start`, `turn`, `reattach`, and `stop` keep the same public verb/selector contract.
2. A successfully established host orchestration session can survive clean attached-client exit without being invalidated automatically.
3. The runtime exposes an explicit valid parked-resumable host posture instead of forcing detached host into terminal/invalidation semantics.
4. World-originated approvals/completions/updates are retained durably when no host client is attached.
5. A parked host session can resume through `substrate agent turn --session ... --backend ... --prompt ...`.
6. A parked host session can resume attached ownership through `substrate agent reattach --session ...`.
7. Detached-world follow-up remains fail-closed until a sanctioned host path re-establishes routable ownership.
8. Exact `(orchestration_session_id, backend_id)` targeting remains authoritative for public turns.
9. Once a public prompt request emits `Accepted`, it always ends with explicit `Completed` or `Failed`.
10. Prompt-stream EOF or silent helper exit after `Accepted` is treated as a bug/failure path, not an operator-facing steady state.
11. Host-only root `start` remains fail-closed for world-only backends.
12. Existing public world-member follow-up coverage from the `PLAN-22` slice remains green as a non-regression constraint.

## Testing Expectations

Primary coverage areas:

- orchestration/session posture transitions in [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:7) and [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:81)
- public caller-surface behavior in [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1083)
- prompt bridge terminal-delivery invariants in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1063)
- REPL/runtime lifecycle handling in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:7335)
- exact public turn resolution in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:711)

Required assertions:

- `active_attached -> parked_resumable` is covered explicitly,
- `parked_resumable -> active_attached` via `reattach` is covered explicitly,
- `parked_resumable -> awaiting_attention` is covered explicitly or proven through durable pending-work derivation,
- legal transitions to terminal are covered explicitly,
- host `start` creates a valid orchestration session and clean host client exit does not invalidate it,
- host `turn` succeeds against a valid parked session,
- `reattach` succeeds against a valid parked session,
- world-originated retained work survives detached host periods,
- detached-world follow-up still fails closed,
- and every accepted prompt request ends in `Completed` or `Failed`.

Recommended commands:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell async_repl -- --nocapture
```

## Explicit Non-Goals For Review

- Do not broaden detached-world public continuity.
- Do not add fuzzy recovery or default backend/session resolution.
- Do not redesign `MemberTurnSubmitRequestV1`.
- Do not repurpose `substrate -c`.
- Do not promise a full daemonized orchestration control plane in this slice.
