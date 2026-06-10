# PLAN-50: Internal Family-2 Ingress-Ready Obligation Identity And Causation Envelope

Source spec: [SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md](./SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md)  
Source prior slice: [PLAN-49.md](./PLAN-49.md)  
Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Plan type: first post-`49` ingress-ready identity-envelope slice  
Status: implemented and validation-aligned on `2026-06-08`

## Objective

Land the next Family-2 envelope slice by widening the local obligation ledger only enough to preserve ingress-ready identity and exact local causation truth for future `host_inbox` or remote-ingress layering, without widening into host-global inbox materialization, remote delivery, or broader federation machinery.

This slice is complete only when all of the following are true:

1. the obligation record carries the six remaining deferred envelope fields:
   - `ingress_source_kind`
   - `ingress_source_id`
   - `ingress_received_at`
   - `causation_event_id`
   - `causation_message_id`
   - `causation_request_id`
2. local persistence and reload preserve those fields when present,
3. local producers stamp exact ingress and request-causation truth into canonical fields when that truth is already available,
4. exact event or message identifiers are canonicalized only when the current producer seam surfaces them directly,
5. missing exact event/message identifiers remain absent rather than synthetic,
6. Slice `49` host-targeting, wrong-host fail-closed posture, and router behavior remain unchanged,
7. `host_inbox`, remote ingress materialization, and cross-host delivery remain deferred.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md](./SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md) has been reviewed and is the source of truth for scope before implementation planning advances.

## Major Components And Dependencies

1. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
   - owns the canonical obligation schema and validation boundary
   - must widen only for the six deferred ingress-ready identity and causation fields in this slice
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - owns durable obligation persistence, reload, and projection boundaries
   - must remain the authoritative write/read seam for the widened envelope
3. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - owns the current local retained-worker obligation producer
   - should stamp exact local ingress and request-causation truth when known
4. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - owns the typed retained-worker event surface
   - should widen only if exact local event/message identity can be surfaced narrowly without reopening broader messaging design
5. `docs/CONFIGURATION.md` and `docs/TRACE.md` if needed
   - must stay honest about what this slice does and does not land

## Plan Summary

After Slice `49`, the next remaining Family-2 seam is narrower than `host_inbox`:

1. the local obligation ledger still lacks the rest of the ingress-ready identity envelope,
2. some joinable identity currently lives only in payload-side metadata rather than canonical obligation fields,
3. the current local producer seam already knows exact request/run identity and local receipt timing,
4. the current local producer seam may or may not expose exact worker event/message identifiers without a bounded adjacent widening.

The design stack is already clear enough to freeze that seam now:

1. local obligations remain the authoritative record for local deferred work,
2. future `host_inbox` or remote ingress layers must adapt into that local ledger rather than replace it,
3. exact optional fields should be added now even if some remain absent for current local producers,
4. absent exact event/message truth must stay absent rather than being guessed from `thread_id` or other nearby metadata.

The narrowest honest Slice `50` is therefore:

1. freeze the six-field envelope contract first,
2. widen persistence and backward-compatible reload second,
3. canonicalize exact local ingress and request-causation truth in producers third,
4. add bounded event/message-causation threading only if the current producer seam already surfaces exact ids,
5. finish with docs and validation last.

## Locked Decisions

### What changes

1. Slice `50` widens the local obligation envelope for ingress-ready identity and exact local causation truth.
2. Slice `50` canonicalizes exact local ingress metadata in first-class obligation fields instead of leaving it payload-only when the runtime already knows it exactly.
3. Slice `50` preserves exact local request causation in first-class obligation fields when the runtime already knows it exactly.
4. Slice `50` may widen the retained-worker producer seam only if that is the narrowest way to surface exact event/message identity without reopening broader messaging design.

### What does not change

1. no `SUBSTRATE_HOME/host_inbox/`,
2. no remote ingress materialization or remote federation protocol,
3. no new public CLI or policy surface for ingress/causation metadata,
4. no reinterpretation of `thread_id` as `causation_message_id`,
5. no broader source-role or distributed lineage project beyond the six deferred fields,
6. no router lifecycle or cross-host delivery productization.

## Implementation Order

### Packet 1: Ingress-Ready Envelope Contract Freeze

Goal:

1. freeze the exact six-field envelope widening,
2. preserve backward compatibility for obligations that do not carry the new fields,
3. define the no-synthesis rule for exact event/message causation.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. targeted obligation round-trip tests

Why first:

1. producer logic needs one frozen canonical record,
2. this slice must not balloon into `host_inbox` or remote-ingress behavior,
3. exactness rules should be explicit before any producer-side wiring begins.

Verification checkpoint:

1. only the six deferred fields are added,
2. persistence remains backward compatible for obligations with no new metadata,
3. the spec-aligned no-synthesis rule is explicit in tests and/or helper behavior.

### Packet 2: Local Producer And Persistence Widening

Goal:

1. stamp exact local ingress truth into canonical obligation fields where the current local producer already knows it,
2. stamp exact local request causation into canonical obligation fields where the current local producer already knows it,
3. keep legacy/local obligations compatible when the new metadata is absent.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
4. targeted producer and persistence tests

Why second:

1. the new fields are only valuable if current local obligations can actually carry exact local truth,
2. this producer widening is still local-only and does not require `host_inbox` or router behavior changes,
3. persistence must stay authoritative before any optional event/message identity widening is considered.

Verification checkpoint:

1. locally produced obligations round-trip with exact ingress and request-causation truth when known,
2. current local-only paths without the new metadata keep working,
3. no `host_inbox`, remote ingress, or cross-host state path is introduced.

### Packet 3: Exact Event/Message Causation Threading Or Explicit Absence Freeze

Goal:

1. preserve exact `causation_event_id` and `causation_message_id` only when the current producer seam surfaces them directly,
2. keep those fields absent when no exact truth exists,
3. prevent adjacent metadata such as `thread_id` from becoming synthetic canonical causation truth.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs` only if narrowly required
3. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
4. targeted producer/contract tests

Why third:

1. exact event/message identity is the main semantic risk in this slice,
2. producer-side exactness should consume the frozen Packet 1 contract and the landed Packet 2 persistence path,
3. this packet should resolve whether exact ids can be surfaced now without reopening broader retained-worker messaging design.

Verification checkpoint:

1. exact event/message ids are preserved only when surfaced directly,
2. missing exact event/message ids remain `None`,
3. `thread_id` remains thread identity only and is not promoted to canonical message identity.

### Packet 4: Docs Alignment And Validation

Goal:

1. align docs with the landed ingress-ready identity-envelope behavior,
2. keep `host_inbox` and remote-ingress materialization explicitly deferred,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `docs/TRACE.md` if touched
3. `llm-last-mile/SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md`
4. `llm-last-mile/PLAN-50.md`
5. `llm-last-mile/TASKS-50.md`

What this packet must enforce:

1. docs describe Slice `50` as ingress-ready local identity-envelope widening only,
2. docs do not imply `host_inbox`, remote ingress, or cross-host delivery have landed,
3. docs preserve the follow-on seam for host-global inbox layering and later distributed delivery.

Verification checkpoint:

1. docs and runtime truth are honest,
2. validation is green,
3. the next Family-2 follow-on can stay focused on `host_inbox` layering instead of reopening ingress-ready envelope questions.

## Risks And Mitigations

1. Risk: Slice `50` balloons into `host_inbox` or remote ingress materialization.
   Mitigation: keep all writes local-session scoped and forbid host-global inbox or remote delivery work in this slice.
2. Risk: the slice invents synthetic event/message identities from `thread_id` or payload shape guesses.
   Mitigation: freeze the no-synthesis rule; only persist `causation_event_id` or `causation_message_id` when exact surfaced truth exists.
3. Risk: widening the retained-worker event contract reopens broader messaging design.
   Mitigation: treat dispatch-contract changes as optional and bounded; if exact ids are not already available narrowly, preserve explicit absence instead of redesigning the protocol.
4. Risk: backward compatibility breaks for already-persisted obligations with no ingress-ready metadata.
   Mitigation: preserve compatibility for missing fields and treat them as current local-only behavior.
5. Risk: router or review behavior changes accidentally because new metadata exists.
   Mitigation: keep router, attach, and review semantics unchanged unless a bounded adjacent fix is required to preserve existing behavior.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because producers need one frozen envelope contract.
2. Packet 2 before Packet 3 because exact event/message causation should consume the landed persistence boundary rather than inventing a parallel path.
3. Packet 3 before Packet 4 because docs should describe the actual landed event/message causation posture honestly.

### Can be deferred

1. `SUBSTRATE_HOME/host_inbox/`,
2. remote ingress materialization,
3. cross-host delivery, sync, lease, or federation protocols,
4. broader source-role or distributed lineage widening beyond the six deferred fields,
5. public router lifecycle or workflow-engine productization.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell obligation -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell orchestrator_world_dispatch -- --nocapture
cargo test -p shell dispatch_contract -- --nocapture
cargo test --workspace -- --nocapture
```

If Packet 3 lands without any `dispatch_contract.rs` change, the targeted `dispatch_contract` rerun may be skipped, but the final closeout must say explicitly why it was unnecessary.

## Expected Follow-On Order After Slice `50`

If Slice `50` lands cleanly as the ingress-ready local identity-envelope slice, the next Family-2 follow-on should remain:

1. host-global inbox layering under `SUBSTRATE_HOME/host_inbox/`,
2. only after that, any broader cross-host delivery, remote ingress materialization, or federation routing/productization.

Slice `50` is therefore transitional infrastructure: it prepares the canonical local obligation record for later host-global ingress layering without claiming that such a layer already exists.
