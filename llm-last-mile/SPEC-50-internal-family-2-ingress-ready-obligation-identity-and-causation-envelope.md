# Spec: Internal Family-2 Ingress-Ready Obligation Identity And Causation Envelope

Source remaining-scope note: [REMAINING-family-2-scope-2026-05-30.md](./REMAINING-family-2-scope-2026-05-30.md)  
Prior slice:
- [SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md](./SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md)
- [PLAN-49.md](./PLAN-49.md)
- [TASKS-49.md](./TASKS-49.md)
Related design stack:
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)  
Phase: `SPECIFY`  
Status: implemented and validation-aligned on `2026-06-08`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `49` closed the bounded host-targeting and wrong-host fail-closed boundary, so the next honest Family-2 seam is the remaining ingress-ready identity envelope, not `host_inbox` materialization.
2. Slice `50` should land only the still-deferred optional identity fields called out after Slice `49`:
   - `ingress_source_kind`
   - `ingress_source_id`
   - `ingress_received_at`
   - `causation_event_id`
   - `causation_message_id`
   - `causation_request_id`
3. This slice should canonicalize exact local truth only. If the current runtime does not surface an exact event or message identifier, the corresponding `causation_*` field should remain absent rather than being guessed from adjacent metadata.
4. `thread_id` is not the same contract as `causation_message_id`; Slice `50` should not silently reinterpret thread identity as message identity.
5. The current local retained-worker obligation producer already has exact request/run identity and receipt timing it can preserve, even if exact worker event/message identifiers are not always available on the current live seam.
6. Slice `50` should not add `SUBSTRATE_HOME/host_inbox/`, remote ingress materialization, cross-host delivery, new policy keys, or broader federation logic.
7. Router-owned auto-attach semantics from Slices `48` and `49` should remain behaviorally unchanged; this slice is about bounded identity-envelope widening, not new routing behavior.

If any of these are wrong, correct them before implementation.

## Objective

Land the next Family-2 envelope slice by widening the local orchestration obligation record just enough to preserve ingress-ready identity and exact local causation truth needed for future `host_inbox` or remote-ingress layering, without yet landing any host-global inbox, remote materialization, or cross-host execution behavior.

Primary runtime story:

1. a local unresolved obligation is created from the already-landed local retained-worker or runtime seam,
2. the canonical obligation record preserves explicit ingress classification and receipt identity when the local producer knows it exactly,
3. the canonical obligation record preserves exact local request causation when the local producer knows it exactly,
4. exact worker event or message identifiers are preserved only when the current producer seam surfaces them directly,
5. fields with no exact truth remain absent rather than guessed,
6. router-owned attach, review-state semantics, and wrong-host behavior preserve the landed Slice `49` posture,
7. no `host_inbox`, remote ingress materialization, or federation protocol lands in this slice.

## Observed Repo Floor

The current repo already has most of the local Family-2 floor this slice should reuse:

1. the canonical local obligation artifact exists as [`OrchestrationObligationRecord`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs),
2. local obligation persistence and backward-compatible reload behavior already live in [`state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs),
3. the current local retained-worker obligation producer already persists exact session, backend, world, request, and payload truth through [`orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
4. Slice `49` already landed bounded `origin_host_id` and `target_host_id` support plus wrong-host fail-closed behavior,
5. the router-owned auto-attach path already consumes only local obligations and does not currently need the broader ingress-ready envelope to preserve its existing behavior.

What this slice now lands in the live tree is the rest of the ingress-ready envelope:

1. the obligation record now carries `ingress_source_kind`, `ingress_source_id`, `ingress_received_at`, `causation_event_id`, `causation_message_id`, and `causation_request_id`,
2. locally produced retained-worker obligations now stamp exact `local_runtime` ingress classification plus exact `request.run_id` receipt/request-causation truth in canonical fields,
3. exact worker event/message identifiers are preserved only when surfaced directly from worker payload metadata,
4. `thread_id` remains payload-only thread identity and is not promoted to canonical message causation truth when exact message identity is absent.

That keeps Slice `50` honest: the local-only boundary is widened just enough for ingress-ready identity and exact local causation truth, but no `host_inbox`, remote ingress, or broader federation machinery is implied.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Expected runtime files:
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) only if exact local event/message identity widening proves necessary for this slice
- Expected docs:
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
  - [`docs/TRACE.md`](../docs/TRACE.md) only if a real canonical trace-facing ingress/causation shape lands

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
cargo test -p shell obligation -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell orchestrator_world_dispatch -- --nocapture
```

Expanded targeted validation if the event contract widens:

```bash
cargo test -p shell dispatch_contract -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

The current repo structure relevant to this slice is:

- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - canonical obligation schema and validation boundary
  - Slice `50` should widen only for the six deferred ingress-ready identity and causation fields
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - durable obligation persistence, reload, and session projections
  - Slice `50` should keep this the authoritative read/write boundary for the widened envelope
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - current local retained-worker obligation producer and router-trigger entrypoint
  - Slice `50` should stamp exact local ingress and request-causation truth here when available
- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - current typed retained-worker event contract
  - Slice `50` should only widen this seam if exact event/message identity can be surfaced narrowly without reopening broader messaging design
- `docs/`
  - docs must stay honest that Slice `50` lands ingress-ready local identity-envelope widening only, not `host_inbox` or remote delivery
- `llm-last-mile/`
  - repo-local planning authority for this Family-2 follow-on slice

## Code Style

Follow the existing runtime style: exact identity, fail-closed boundaries, explanation-ready semantics, and no synthetic truth for fields that the current runtime cannot state exactly.

Preferred style:

```rust
obligation.ingress_source_kind = Some("local_runtime".to_string());
obligation.ingress_source_id = Some(request.run_id.clone());
obligation.ingress_received_at = Some(obligation.created_at);
obligation.causation_request_id = Some(request.run_id.clone());

if let Some(event_id) = surfaced_exact_event_id(event) {
    obligation.causation_event_id = Some(event_id);
}
```

Conventions:

1. preserve exact canonical identity in first-class fields when the runtime already knows it,
2. leave optional causation fields unset when the runtime lacks exact truth,
3. do not reinterpret `thread_id` as a message identifier,
4. keep host-targeting, router, and review-state semantics unchanged unless exact identity preservation requires a bounded adjacent update,
5. do not widen into public config, CLI, or host-global ingress surfaces in this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- shell runtime regression suites

Test levels for this slice:

1. obligation-schema and persistence tests:
   - the six new optional fields round-trip when present,
   - missing new fields remain backward compatible for existing persisted obligations,
   - blank or whitespace-only identity fields fail validation if the runtime chooses to validate them strictly
2. local producer tests:
   - local obligations preserve exact ingress-source classification and local receipt identity when created from the retained-worker path,
   - `causation_request_id` preserves exact local request/run identity when known,
   - existing payload-side metadata that still lacks a canonical field remains intact
3. exactness/no-synthesis tests:
   - absent exact event/message identifiers remain absent,
   - `thread_id` is not promoted to `causation_message_id`,
   - no synthetic placeholder ids are persisted
4. dispatch-contract tests only if needed:
   - if the retained-worker event contract widens, the new identity fields remain optional, exact, and backward compatible

## Boundaries

- Always:
  - keep the obligation ledger as the canonical local durable truth
  - preserve exact local ingress and causation identity only when the runtime already knows it
  - keep missing exact event/message ids absent rather than guessed
  - preserve landed Slice `49` host-targeting and wrong-host behavior unchanged
- Ask first:
  - adding a new public policy key, CLI surface, or `SUBSTRATE_HOME/host_inbox/` path
  - widening the retained-worker message protocol beyond the minimum needed to surface exact event/message identity
  - introducing remote ingress materialization, cross-host delivery, or federation semantics
- Never:
  - add `SUBSTRATE_HOME/host_inbox/` in this slice
  - let the router consume remote/global ingress records directly
  - treat `thread_id` as `causation_message_id`
  - fabricate `causation_event_id` or `causation_message_id` from non-authoritative adjacent metadata

## Success Criteria

This slice is complete only when all of the following are true:

1. the canonical obligation record preserves room for `ingress_source_kind`, `ingress_source_id`, `ingress_received_at`, `causation_event_id`, `causation_message_id`, and `causation_request_id`,
2. persistence and reload behavior preserve those fields when present without breaking already-persisted obligations that do not carry them,
3. local obligation producers stamp exact local ingress and request-causation truth into canonical obligation fields when the current runtime knows it exactly,
4. exact worker event/message identifiers are persisted only when the current producer seam surfaces them directly,
5. absent exact event/message identifiers stay absent rather than synthetic,
6. router-owned attach behavior, review-state semantics, and wrong-host fail-closed posture preserve the landed Slice `49` behavior,
7. no `host_inbox`, remote ingress materialization, or broader federation workflow lands.

## Open Questions

1. Should local retained-worker obligations classify `ingress_source_kind` as a single exact internal label such as `local_runtime`, or should the slice preserve a small internal label family such as `local_runtime` versus `local_router` even before host-global ingress exists? Default assumption for this spec: use the narrowest exact label needed by the current producer path and keep the contract internal-only.
2. If the current retained-worker event seam does not surface an exact event or message identifier, should Slice `50` stop at preserving the optional fields and exact local request causation only, or should it widen the producer seam just enough to surface one exact worker event identifier? Default assumption for this spec: widen only if exact truth is already available in raw surfaced events without reopening broader retained-worker messaging design.
