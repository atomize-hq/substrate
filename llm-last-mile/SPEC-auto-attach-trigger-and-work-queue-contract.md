# Spec: Auto-Attach Trigger And Work-Queue Contract

Source handoff: [2026-05-28-195040-host-world-dispatch-design-stack.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.codex/handoffs/2026-05-28-195040-host-world-dispatch-design-stack.md)  
Related design stack:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-world-worker-lifecycle-model.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-notification-inbox-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-host-to-world-steering-policy-matrix.md)  
Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:
1. This first step is still design-only. The immediate deliverable is a frozen contract doc, not runtime code or a public CLI.
2. Auto-trigger attach from pending work is required for this track; manual-only lazy attach has already been rejected by the user decision captured in the handoff.
3. The durable notification inbox remains a passive record of orchestration events, while the new work-queue contract is a separate active-routing layer that determines whether host attach work should be attempted.
4. Auto-attach work is always rooted in a durable host orchestration session and must never become direct world-to-world continuation that bypasses host-session authority.
5. Exact identity remains mandatory across this slice: `orchestration_session_id`, source notification or source worker identity, exact `backend_id`, and authoritative `world_id` / `world_generation` where world-bound context matters.
6. The first shipping slice should stay conservative: worker-originated fork suggestions may produce notifications, but the auto-attach trigger contract should not silently authorize autonomous worker forking.
7. The contract should be Linux-first and consistent with the supported macOS/Lima posture; Windows/WSL parity is not required to freeze the design.
8. `awaiting_attention` remains a host-session posture derived from durable state; the work queue may influence when attach is attempted, but it must not redefine host posture semantics that already belong to the inbox/notification contract.

If any of these are wrong, correct them before planning.

## Observed Repo Floor

The current repo and design stack already freeze several facts this spec should treat as input rather than reopen:

1. Durable host-session authority already exists and remains authoritative even when no host client is attached.
2. Durable inbox artifacts already exist under the session root and already normalize unresolved attention-driving work into host-session `awaiting_attention`.
3. Public `reattach` exists as an exact-session recovery path, but there is no shipped automatic attach workflow yet.
4. The new design stack already separated:
   - dispatch/allocation,
   - retained worker messaging/steering,
   - worker lifecycle,
   - durable notification/inbox semantics,
   - steering policy.
5. The missing gap for this step is not “how do notifications persist?” but “which durable conditions become active attach work, how that work is deduped, and how it stays distinct from passive inbox state.”

## Objective

Define the contract for turning durable orchestration events into exact, deduplicated, policy-gated auto-attach work without collapsing:

1. passive notification durability,
2. host-session posture,
3. router-owned attach requests,
4. worker lifecycle,
5. and later attach execution mechanics.

The contract must answer:

1. which notification kinds or runtime events are eligible to create auto-attach work,
2. what the canonical queued work artifact looks like,
3. how idempotency and dedupe work,
4. how queue state differs from inbox state,
5. what exact identities and causation fields are mandatory,
6. and what must fail closed.

Primary operator/runtime story:

1. A retained world worker emits a durable event such as `follow_up_required`, `approval_required`, `blocked`, or `fork_request`.
2. The notification persists under the orchestration session regardless of whether a host client is attached.
3. Policy-gated trigger evaluation may convert that durable event into one exact auto-attach work item for the same orchestration session.
4. That work item records why attach work should be attempted, what durable state caused it, and what dedupe key prevents duplicate queue fan-out.
5. Later router/daemon logic may claim and process the queued work item through sanctioned attach paths, but queue creation itself must not imply that attach already happened.

## Non-Goals

This spec does not:

1. define the router/daemon worker that watches and executes queued items,
2. decide continuity attach versus fresh attach execution logic in detail,
3. define a public inbox or queue CLI surface,
4. introduce worker-to-worker direct continuation,
5. authorize autonomous fork execution,
6. or reopen existing notification or host-posture contracts except where this spec must reference them.

## Architectural Direction

### Passive inbox versus active work queue

The repo now needs two distinct durable layers:

1. `inbox` / notifications:
   - authoritative record that something happened,
   - source of host `awaiting_attention` posture,
   - may exist without any automatic attach attempt.
2. `work_queue` / auto-attach requests:
   - explicit request ledger for router-owned attach evaluation,
   - created only for eligible event classes,
   - idempotent and deduplicated,
   - separate state machine from notification status.

The queue must not be modeled as a synonym for unread inbox items.

### Canonical artifact

The design doc produced from this spec should freeze a conceptual artifact shaped like:

```text
AutoAttachWorkItemV1
- work_item_id
- orchestration_session_id
- kind
- source_notification_id?
- source_participant_id?
- source_backend_id?
- target_backend_id?
- world_id?
- world_generation?
- idempotency_key
- dedupe_key
- status
- attempt_count
- last_attempt_at?
- created_at
- updated_at
- causation_request_id?
- causation_event_id?
- payload?
```

The queue artifact is not a prompt and is not a session-resume handle.

### Eligible trigger classes

The design doc should freeze which persisted events may create auto-attach work. Initial default direction:

1. eligible by default:
   - `follow_up_required`
   - `approval_required`
   - `blocked`
   - `fork_request`
2. explicitly not auto-attach by default:
   - `task_completed`
   - `result_available`
   - `fork_recommendation`
3. open but likely conservative:
   - `task_failed`
   - `runtime_alert`
   - `escalation_recommended`

The first contract should prefer under-triggering to over-triggering.

### Queue state versus notification state

Notification lifecycle and queue lifecycle must stay separate.

Illustrative queue statuses:

1. `queued`
2. `claimed`
3. `completed`
4. `cancelled`
5. `dead_letter`

Resolving a notification does not automatically mean a queue item never existed; it means the queue item must become non-actionable under explicit rules.

### Fail-closed rules

The contract must fail closed when:

1. the source orchestration session is missing or terminal,
2. the source notification is missing, malformed, or already non-actionable,
3. exact world-binding truth is required but absent or mismatched,
4. the persisted host attach contract is unavailable for later attach execution,
5. the same dedupe class already has an active queued or claimed work item,
6. policy denies automatic attach evaluation for the event class or session posture.

## Commands

Current review commands:

```bash
sed -n '1,260p' llm-last-mile/SPEC-auto-attach-trigger-and-work-queue-contract.md
sed -n '1,260p' llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md
sed -n '1,260p' llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md
```

Repo validation floor once implementation exists:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test --workspace -- --nocapture
```

Focused future operator validation targets:

```bash
substrate agent status --json
substrate agent reattach --session <orchestration_session_id> --backend <backend_id> --json
substrate agent toolbox status --session <orchestration_session_id> --json
substrate agent doctor --json
```

## Project Structure

Relevant design and runtime seams for this slice:

```text
llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md
  Existing durable notification semantics and host attention posture.

llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md
  Host-to-world dispatch verbs and exact identity rules.

llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md
  Existing lazy-attach slice that will need rewriting after this contract and the router doc land.

AGENT_ORCHESTRATION_GAP_MATRIX.md
  Current repo truth for durable inbox, public reattach, and remaining automatic attach gap.

archive/LLM_AI_CAPABILITY_ENABLEMENT_PLANNING_ORDER.md
  Earlier router-daemon and durable request-queue sequencing constraints.

crates/shell/src/execution/agent_runtime/orchestration_session.rs
  Host-session posture and authoritative orchestration session state.

crates/shell/src/execution/agent_runtime/state_store.rs
  Durable session-root storage, inbox persistence, and live state selection.

crates/shell/src/execution/agent_runtime/control.rs
  Existing control-plane orchestration/runtime helpers that future attach processing will likely compose with.

crates/shell/tests/
  Future regression coverage for status, reattach, queue gating, and fail-closed behavior.
```

## Code Style

Prefer explicit typed queue contracts over overloaded inbox or prompt semantics.

Preferred style:

```rust
enum AutoAttachWorkKind {
    ReviewNotification,
    ApprovalFollowUp,
    BlockedWorkerResume,
    ForkRequestReview,
}

struct AutoAttachWorkItemV1 {
    orchestration_session_id: String,
    source_notification_id: Option<String>,
    source_participant_id: Option<String>,
    dedupe_key: String,
    status: AutoAttachWorkStatus,
}
```

Contract conventions:

1. exact ids and typed enums instead of heuristic matching,
2. additive enum growth only,
3. no synthetic prompt text as queue payload,
4. no hidden state transitions that bypass durable artifacts,
5. no implicit broadening from notification presence to attach execution.

## Testing Strategy

Primary framework and locations:

1. Rust unit tests beside queue and state-store helpers once implementation exists.
2. Shell integration tests under [`crates/shell/tests/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests) for session posture, queue creation, dedupe, and fail-closed behavior.
3. Doc/manual validation for the design stack before code begins.

Coverage expectations:

1. one eligible notification creates exactly one actionable work item,
2. repeated trigger ingestion with the same dedupe class does not create duplicate active items,
3. resolving or dismissing the source notification produces explicit queue-state effects rather than silent deletion,
4. missing session, missing notification, missing attach truth, or mismatched world binding all fail closed,
5. `awaiting_attention` remains driven by notifications rather than by queue status,
6. auto-attach queue creation does not consume inbox work or mutate worker lifecycle directly.

## Boundaries

- Always:
  - preserve exact session/backend/world identity,
  - keep inbox state and queue state as separate durable artifacts,
  - record causation and dedupe fields explicitly,
  - fail closed on missing authoritative state.
- Ask first:
  - changing host posture taxonomy,
  - adding a public inbox/queue CLI surface,
  - introducing new daemon/service ownership beyond the planned router track,
  - enabling autonomous fork execution in v1.
- Never:
  - inject synthetic prompts to “simulate” attach work,
  - use fuzzy target selection,
  - treat unread notifications as equivalent to queued attach work,
  - let queue artifacts bypass host-session durable authority.

## Success Criteria

This spec is complete for Phase 1 when all of the following are true:

1. the repo has a saved spec file for this slice under `llm-last-mile/`,
2. assumptions and non-goals are explicit enough to prevent the queue from collapsing into the inbox model,
3. the spec freezes the requirement for a separate durable auto-attach work artifact,
4. the spec defines the minimum identity, causation, and dedupe properties the later design doc must carry,
5. the spec narrows the initial eligible trigger classes and fail-closed rules,
6. the spec leaves router execution mechanics and attach-mode selection for the next planning/design step rather than muddling them into this contract.

## Open Questions

1. Should `task_failed` and `runtime_alert` ever auto-enqueue by default in v1, or should they remain notification-only unless policy explicitly opts in?
2. Is one active queue item per `(orchestration_session_id, source_notification_id, kind)` sufficient, or does dedupe need a broader class such as worker-plus-obligation?
3. Should queue artifacts live under a dedicated `work_queue/` session-root directory, or does the design need an abstraction that avoids freezing path shape yet?
4. What is the exact queue effect when the source notification is resolved while a work item is already `claimed`?
5. Which queue statuses are required in v1 versus deferred for later retry/backoff sophistication?
