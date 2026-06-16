# Spec: Internal Family-2 Host-Global Ingress Receive-Cursor And Sync-State Coordination

Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior Family-2 slice:
- [SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md)
- [PLAN-51.md](./PLAN-51.md)
- [TASKS-51.md](./TASKS-51.md)
Related design stack:
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
Phase: `SPECIFY`  
Status: proposed next slice on `2026-06-12`

## Post-Slice-60 Refresh Gate (Unmissable)

If Slice `58`, Slice `59`, and Slice `60` land before Slice `57` implementation starts, this Slice `57` doc set must be refreshed against live repo truth **before implementation**.

That refresh is required even though Slice `57` remains a different seam.

Minimum required refresh after Slice `60` lands:

1. update sequencing language that still frames Slice `57` as the immediate next slice after Slice `56`,
2. re-read the live `orchestrator_world_dispatch.rs` seam because Slice `59` may have changed world-dispatch/runtime-truth behavior there,
3. re-run the coexistence check that Slice `57` still only layers bounded ingress coordination above `host_inbox` and does not accidentally inherit runtime/selector assumptions from Slice `58`/`59`/`60`,
4. then explicitly confirm that Slice `57` still needs no scope change beyond wording/coexistence refresh.

Do **not** treat the existing June 2026 sequencing language in this doc as implementation-ready truth if Slice `60` has already landed.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `56` is checkpoint-green, so the next honest execution-bearing roadmap seam is no longer another Family-1 caller/status/toolbox slice.
2. The remaining-scope note and gap matrix are aligned that the next likely Family-2 seam, if the repo still needs more than the landed local boundary, is host-global ingress lifecycle coordination rather than reopening local materialization.
3. The smallest worthwhile coordination slice is not broader federation or cross-host delivery. It is the source-scoped receive-cursor and sync-state floor above the already-landed `host_inbox -> local obligation -> router` path.
4. The current repo truth already has:
   - bounded host-global ingress artifacts under `SUBSTRATE_HOME/host_inbox/`,
   - exact local materialization into canonical obligations,
   - router-owned auto-attach consuming local obligations only.
5. This slice should introduce only the missing host-global ingress coordination state needed to ingest upstream work idempotently and durably into `host_inbox`.
6. This slice should not decide or ship broader cross-host delivery, lease/lock coordination, public inbox UX, or a new public operator daemon surface.
7. A separate stabilization handoff exists in [STABILIZATION_NEXT_SLICE_2026-06-12.md](../STABILIZATION_NEXT_SLICE_2026-06-12.md). This spec is the next roadmap slice from the remaining-scope docs, not a claim that stabilization cannot be prioritized first if the team chooses runtime-risk work over roadmap sequencing.

If any of these are wrong, correct them before implementation.

## Objective

Land the next bounded Family-2 slice by adding source-scoped host-global ingress coordination above the landed `host_inbox` layer, so Substrate can durably track:

1. what upstream ingress source is being advanced,
2. which receive cursor or sync position has been durably applied locally,
3. whether a given upstream ingress record has already been persisted into canonical local `host_inbox` state,
4. and when replay or retry should remain idempotent rather than duplicating local host-inbox records.

Primary runtime story:

1. an internal ingress source submits one or more exact host-global inbox candidates for the local host,
2. the runtime validates source identity and monotonic receive-cursor truth for that source,
3. the runtime durably persists the eligible host-inbox record before advancing the source coordination state,
4. repeated application of the same ingress cursor or record is idempotent and does not create duplicate host-inbox records,
5. once the record is durably present in `SUBSTRATE_HOME/host_inbox/`, the already-landed materialization pass may later convert it into a canonical local obligation,
6. local materialization, inbox projection, and router-owned auto-attach remain unchanged consumers of local obligations only,
7. no remote delivery engine, no lease/lock coordination, and no public operator surface land in this slice.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Expected runtime files:
  - [`crates/shell/src/execution/agent_runtime/host_inbox.rs`](../crates/shell/src/execution/agent_runtime/host_inbox.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - one new bounded host-inbox coordination module under `crates/shell/src/execution/agent_runtime/`
  - one new bounded host-side ingress apply helper under `crates/shell/src/execution/` only if coordination entrypoint logic does not fit cleanly inside the store/module seam
- Existing adjacent runtime seams that should remain consumers rather than be redesigned:
  - [`crates/shell/src/execution/host_inbox_materialization.rs`](../crates/shell/src/execution/host_inbox_materialization.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted validation floor:

```bash
cargo test -p shell host_inbox -- --nocapture
cargo test -p shell state_store -- --nocapture
```

Expanded targeted validation if the host-side ingress apply entrypoint changes:

```bash
cargo test -p shell host_inbox_materialization -- --nocapture
cargo test -p shell orchestrator_world_dispatch -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

The current repo structure relevant to this slice is:

- `crates/shell/src/execution/agent_runtime/host_inbox.rs`
  - canonical host-global inbox artifact and materialization-state boundary from Slice `51`
  - this slice should extend the host-global ingress model only enough to describe source-scoped coordination truth above the record layer
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative persistence seam for local sessions, obligations, and host-inbox artifacts
  - this slice should keep coordination-state persistence store-owned rather than adding ad hoc writers elsewhere
- `crates/shell/src/execution/host_inbox_materialization.rs`
  - already-landed local `host_inbox -> obligation` materialization pass
  - this slice should treat that pass as a downstream consumer, not the place where ingress cursors are invented
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - current host-side router/materialization orchestration entrypoint
  - this slice may only gain a bounded coordination-aware ingress apply hook if needed; it must not become a general federation engine
- `llm-last-mile/`
  - planning authority for this Family-2 follow-on slice

## Code Style

Keep one authoritative durable layer per responsibility:

1. source-scoped ingress coordination state above `host_inbox`,
2. canonical `host_inbox` records as the local host-global ingress artifact,
3. canonical local obligations as the only router/input artifact after materialization.

Preferred style:

```rust
if !cursor_state.can_apply(&candidate_cursor) {
    anyhow::bail!(
        "non_monotonic_host_inbox_cursor: source {} cannot move from {} to {}",
        source_key,
        cursor_state.last_applied_cursor,
        candidate_cursor,
    );
}

store.persist_host_inbox_record(&record)?;
store.advance_host_inbox_cursor(&source_key, &candidate_cursor, &record.record_id)?;
```

Conventions:

1. advance source coordination state only after the corresponding host-inbox record is durably persisted,
2. make duplicate cursor or duplicate record replay idempotent rather than silently duplicative,
3. keep cursor/sync-state truth explanation-ready and exact,
4. keep local materialization and router attachment consuming their existing lower layer only,
5. do not smuggle federation, lease ownership, or public UX into this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests

Test levels for this slice:

1. coordination-state artifact tests:
   - source-scoped coordination records round-trip durably,
   - invalid cursor/sync-state shapes fail validation,
   - coordination state stays distinct from host-inbox record truth
2. ingress apply tests:
   - applying a valid new cursor persists exactly one host-inbox record and advances sync state once,
   - replaying the same cursor/record is idempotent,
   - non-monotonic or mismatched replay fails closed without advancing coordination state
3. boundary coexistence tests:
   - advancing receive cursor does not imply local obligation materialization,
   - the existing materialization pass still acts only on canonical host-inbox records,
   - router-owned auto-attach still acts only on local obligations
4. failure-mode tests:
   - malformed coordination artifacts fail closed,
   - cursor advancement never occurs before durable host-inbox persistence,
   - missing local session truth may leave a host-inbox record pending for later materialization without rolling back the already-durable ingress receipt

## Boundaries

- Always:
  - keep source-scoped ingress coordination above the canonical `host_inbox` record layer
  - keep `host_inbox` as the only canonical local host-global ingress artifact
  - advance receive cursor or sync state only after durable local receipt is safely persisted
  - preserve idempotent replay for duplicate ingress batches or records
- Ask first:
  - adding broader cross-host delivery semantics
  - adding lease/lock ownership for distributed delivery
  - widening this slice into public review/manage UX or new operator commands
  - changing the already-landed local obligation schema instead of layering above it
- Never:
  - let source coordination state replace canonical host-inbox record truth
  - let cursor advancement depend on later router attach success or obligation resolution
  - treat remote delivery, federation, or public inbox UX as part of this slice
  - bypass the existing exact local materialization boundary

## Success Criteria

This slice is complete only when all of the following are true:

1. the runtime has one bounded durable coordination artifact for source-scoped host-global ingress receive-cursor or sync-state truth,
2. the runtime can apply a valid ingress candidate by durably persisting a canonical host-inbox record before advancing that source’s coordination state,
3. replaying the same upstream cursor or record is idempotent and does not create duplicate local host-inbox artifacts,
4. non-monotonic, mismatched, or otherwise invalid ingress replay fails closed without silently advancing coordination state,
5. the already-landed `host_inbox -> local obligation -> router` path remains intact and continues to consume only the lower canonical layer appropriate to each step,
6. no cross-host delivery engine, no distributed lease/lock model, and no public operator inbox surface land in this slice.

## Open Questions

1. What is the narrowest durable source key for coordination state:
   - exact upstream stream id,
   - `(origin_host_id, ingress_source_kind, source_channel_id)`,
   - or another exact source-scoped key?
   Default assumption for this spec: the slice should freeze one exact source key and not overload per-record `ingress_source_id` if that field is still semantically “request id” rather than “stream id”.
2. Should the first cursor floor allow only monotonic advance, or also an explicit idempotent repeat of the last applied cursor?
   Default assumption for this spec: allow exact repeat of the last applied cursor only when the associated record identity matches, and otherwise fail closed.
3. Where should the first ingress apply helper live?
   Default assumption for this spec: keep persistence and validation store-owned, and add one bounded host-side helper only if it materially improves sequencing clarity.
