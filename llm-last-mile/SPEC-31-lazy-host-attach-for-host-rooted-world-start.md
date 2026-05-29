# Spec: Lazy Host Attach For Host-Rooted World Start

Source SOW: [31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Source lifecycle design: [DESIGN-world-worker-lifecycle-model.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md)  
Adjacent landed slices: [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md), [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Phase: `SPECIFY`  
Status: draft for review on 2026-05-29

## Assumptions

These are the assumptions I am making so this spec stays concrete. Correct them before implementation if any are wrong.

1. Slice 30 remains frozen: the normal public world-backed `agent start` happy path is already host-attached at successful return, and slice 31 only owns specialized born-unattached or later-detached host-rooted sessions.
2. The persisted `HostAttachContract` is still the only durable attach-truth baseline; slice 31 may reuse and narrow it, but must not invent a second attach-truth dialect.
3. The repo's existing durable inbox artifacts in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) are current compatibility surface only; canonical pending-work truth for this slice should become obligation-ledger based, with inbox/review and auto-attach treated as projections.
4. Linux remains the only supported implementation target for the specialized automatic attach path in this slice unless a later slice explicitly expands parity.
5. Exact operator-visible posture names may still change during review, but the distinction between never-attached-yet, previously attached and resumable, and unresolved-attention detached sessions is frozen and may not be collapsed.

## Observed Repo Floor

The current repo already provides the floor this slice must build on:

1. Durable host attach truth already exists in [`HostAttachContract`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:79), including continuity selectors, capability gates, and launch knobs.
2. Host-session posture already distinguishes `active_attached`, `born_unattached`, `parked_resumable`, and `awaiting_attention` in [`OrchestrationSessionPosture`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:68).
3. Later world follow-up already fails closed for detached or born-unattached host-rooted sessions until sanctioned host attach is restored, via checks in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs).
4. A durable inbox path already exists in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:383), and detached posture derivation already uses `pending_inbox_count`; slice 31 must not let that legacy storage shape become the long-term canonical source of truth.
5. World participant identity, lineage, and invalidation are already explicit in [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs), which matches the lifecycle design's requirement for exact identity, exact world binding, and fail-closed invalidation.

## Objective

Build the specialized host-rooted session path that may be born without an attached host execution client, persists unresolved host-side work durably while unattached, and restores sanctioned host ownership through obligation-driven attach.

Primary operator/runtime story:

1. A host-rooted orchestration session may exist without an attached host execution client in this specialized path.
2. Persisted attach truth is still written at session birth and remains authoritative while no client is attached.
3. Unresolved host-side work persists durably as obligations, not as synthetic prompts and not as a second canonical notification queue.
4. Router-owned automatic attach may trigger from eligible unresolved obligations, coalesced per orchestration session.
5. Manual `substrate agent reattach --session <id>` remains canonical and interoperates cleanly with automatic attach.
6. World-side follow-up remains fail-closed until sanctioned host ownership is actually restored.

## Frozen Direction

This spec freezes the following product and runtime direction:

1. Durable authority remains the host-rooted orchestration session whether or not a host client is currently attached.
2. The attach baseline is the persisted `HostAttachContract` written at birth or copied forward through sanctioned successor paths.
3. Unresolved work while unattached is modeled as durable obligations under the orchestration session.
4. Inbox/review and automatic attach are projections over those obligations rather than independent ledgers.
5. Automatic attach is required in this slice; it is not an optional follow-on.
6. Automatic attach is continuity-first when a valid continuity selector still matches persisted attach truth, with fresh attach as the fallback when continuity is unavailable but attach truth remains valid.
7. Detached or born-unattached public world follow-up remains fail-closed until host ownership is actually restored.
8. Worker lifecycle state remains distinct from host-session posture.

## Required Runtime Taxonomy

The implementation must keep these detached host-session distinctions separate:

1. `active_attached`
   - the host execution client is attached and authoritative.
2. never-attached-yet posture
   - the host-rooted session is valid and attach-capable, but no host execution client has ever attached yet.
3. `parked_resumable`
   - the host client is currently absent, no unresolved attention-driving obligations exist, and later attach is valid.
4. `awaiting_attention`
   - the host client is currently absent and unresolved attention-driving obligations exist.
5. `terminal`
   - the orchestration session is closed and not routable.

The implementation may choose final naming for item 2, but it must not collapse items 2-4 into one detached bucket.

## Worker Lifecycle Relationship

This slice inherits the world-worker lifecycle model and must preserve the authority boundary:

1. `awaiting_attention` remains host-session posture only.
2. Retained workers may enter `attention_pending`, but that worker state must only influence host posture through durable obligation creation.
3. `ephemeral` work that cannot finish as one-shot work must escalate explicitly through `needs_retained_followup`; it must not silently become retained or silently create synthetic host prompts.
4. Retained worker invalidation remains fail-closed on exact world-binding or world-generation mismatch.

## Tech Stack

- Language: Rust 2021, MSRV 1.89+
- Public control surface: [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- Durable host posture and attach truth: [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- Durable state persistence and status projection: [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- Host/world runtime control plumbing: [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- Participant identity, lineage, and invalidation: [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- Shared persisted attach resolution: [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- Likely new slice-31 modules:
  - `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - `crates/shell/src/execution/agent_runtime/auto_attach.rs`

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

Targeted shell suites:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

Focused runtime tests likely needed during implementation:

```bash
cargo test -p shell orchestration_session -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell control -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

Manual operator validation targets:

```bash
substrate agent start --backend <backend_id> --scope world --prompt "hello" --json
substrate agent status --json
substrate agent reattach --session <orchestration_session_id> --json
substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt "next" --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent doctor --json
substrate agent toolbox status --json
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agents_cmd.rs`
  - public `reattach`, `turn`, `status`, and operator-visible failure guidance
- `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
  - detached posture taxonomy, attach-truth invariants, and host-session normalization rules
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - durable obligation persistence, status projection, review projection, and fail-closed routing checks
- `crates/shell/src/execution/agent_runtime/control.rs`
  - host attach execution, continuity/fresh attach selection, and follow-up control behavior
- `crates/shell/src/execution/agent_runtime/session.rs`
  - retained worker lineage and invalidation invariants
- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - persisted attach resolution and narrowing-only overlay enforcement
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - new canonical obligation record definitions and persistence helpers
- `crates/shell/src/execution/agent_runtime/auto_attach.rs`
  - new router-owned attach-trigger evaluation and session coalescing helpers
- `crates/shell/tests/agent_public_control_surface_v1.rs`
  - public CLI attach, follow-up, and fail-closed regression coverage
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - status, toolbox, doctor, and lineage regression coverage
- `llm-last-mile/`
  - slice planning, plan review, and closeout artifacts

## Code Style

Follow the existing `shell` crate style: explicit fail-closed errors, narrow helpers, and stable contract wording that tests can assert exactly.

Example style:

```rust
fn detached_follow_up_classifier(
    posture: OrchestrationSessionPosture,
) -> &'static str {
    match posture {
        OrchestrationSessionPosture::BornUnattached => "unsupported_platform_or_posture",
        OrchestrationSessionPosture::AwaitingAttention => "owner_unreachable",
        OrchestrationSessionPosture::ParkedResumable => "owner_unreachable",
        OrchestrationSessionPosture::ActiveAttached => "ok",
        OrchestrationSessionPosture::Terminal => "terminal_session",
    }
}
```

Conventions:

1. Use `Result<T, anyhow::Error>` and attach context at durable-state boundaries.
2. Keep policy and routing denials explicit and stable; do not hide detached failure under generic transport errors.
3. Preserve exact identity and lineage fields rather than introducing fuzzy target selection.
4. Reuse persisted attach-resolution helpers instead of re-deriving attach truth from stale participant snapshots.
5. When adding new durable records, validate them at the persistence boundary and keep compatibility projection code separate from canonical state.

## Testing Strategy

Frameworks:

- Rust unit tests and integration tests via `cargo test`
- Existing shell public-control and successor-contract suites

Test levels for this feature:

1. Unit tests for posture invariants, obligation-record validation, attach-mode selection, and session-coalescing rules.
2. State-store tests for obligation persistence, review projection, attach projection, and pending-attention derivation.
3. Integration tests for manual `reattach`, automatic attach, follow-up fail-closed behavior, and continuity-versus-fresh attach fallback.
4. Regression tests proving no hidden bootstrap prompt, no duplicate attach launches per session, and no detached world follow-up before host ownership returns.
5. Manual Linux validation proving slice-30 happy path still stays attached while the specialized slice-31 path behaves truthfully and fail-closed.

Coverage expectations:

1. Every new detached posture transition must be pinned by tests.
2. Every eligible obligation class used for auto-attach must have both a positive and negative test.
3. Both continuity-first and fresh-fallback attach paths must have explicit coverage.
4. A stale or invalid persisted attach contract must fail closed before launching backend work.

## Boundaries

- Always:
  - keep slice 30's normal host-attached world-backed start path unchanged
  - preserve `HostAttachContract` as the single attach-truth baseline
  - keep worker lifecycle state separate from host-session posture
  - fail closed when authoritative world binding, lineage, or attach truth is missing or invalid
- Ask first:
  - changing public CLI syntax beyond current `reattach` and existing control verbs
  - introducing a new on-disk durable artifact format that replaces current inbox compatibility files in one step
  - broadening automatic attach beyond Linux or beyond the router-owned path described here
- Never:
  - inject hidden bootstrap prompts or synthetic follow-up prompts
  - derive attach truth from stale participant snapshots when persisted attach truth is missing
  - allow detached or born-unattached world follow-up before sanctioned host attach succeeds
  - collapse never-attached, parked-resumable, and awaiting-attention into one detached status

## Success Criteria

This slice is done only when all of the following are true:

1. Specialized host-rooted sessions may exist without an attached host client while still carrying authoritative persisted attach truth.
2. Operator-visible status distinguishes never-attached-yet, detached-resumable, and detached-awaiting-attention postures without misrepresenting slice 30's default happy path.
3. Unresolved host-side work persists as canonical obligations under the orchestration session.
4. Inbox/review state and automatic attach eligibility derive from those obligations rather than acting as separate canonical ledgers.
5. Eligible unresolved obligations trigger router-owned attach evaluation that coalesces to at most one attach episode per orchestration session at a time.
6. Automatic attach prefers continuity when valid and falls back to fresh attach only when continuity is unavailable but persisted attach truth remains valid.
7. Manual `reattach` can satisfy or supersede a pending automatic attach without duplicate launches or duplicate durable work.
8. Detached or born-unattached world follow-up remains fail-closed until host ownership is restored through sanctioned attach.
9. No synthetic prompt reconstruction, hidden bootstrap prompt, or world-first continuation path is introduced.
10. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, the two shell suites, and `cargo test --workspace -- --nocapture` are green, with Linux manual smoke evidence recorded.

## Packet 4 Implementation Notes

Implementation notes captured on 2026-05-29:

1. manual `reattach` and router-owned automatic attach now share one hidden-owner-helper launch path in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), which keeps launch-time normalization and attach restoration checks centralized.
2. automatic attach planning in [`auto_attach.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs) resolves the persisted host attach contract directly from orchestration-session truth and avoids broadening the requested attach mode:
   - persisted `continuity_required` remains continuity-required
   - persisted `continuity_preferred` and `fresh_allowed` narrow to continuity-preferred so fresh attach remains fallback-only
3. attach startup extension generation now distinguishes continuity-backed attach from fresh attach preparation, so missing continuity no longer forces resume-only startup metadata when the persisted contract still permits fresh recovery.
4. Packet 4 validation adds planner/bootstrap seam coverage for:
   - continuity-first automatic attach planning
   - fresh-fallback planning when continuity is unavailable but persisted truth still allows attach
   - fail-closed planning when persisted truth requires continuity that no longer exists
5. The current Codex backend still rejects a prompt-free fresh control attach at the wrapper boundary with an empty-prompt validation error. The landed slice therefore proves the specialized fresh-fallback planner/bootstrap seam and the fail-closed interop contract, but it does not yet claim a fully verified end-to-end prompt-free Codex fresh attach runtime path.

## Open Questions

1. What should the final operator-visible label be for the never-attached-yet specialized posture: keep `born_unattached` for v1, or rename it while preserving legacy status projection?
2. Should the obligation ledger land as a new durable artifact family alongside the existing inbox compatibility surface, or should the existing inbox storage layout be migrated in place behind a compatibility reader?
3. What is the concrete execution boundary for router-owned automatic attach in this repo: synchronous evaluation in the current runtime, or a dedicated background daemon/work-loop artifact that persists claimed attach episodes?
