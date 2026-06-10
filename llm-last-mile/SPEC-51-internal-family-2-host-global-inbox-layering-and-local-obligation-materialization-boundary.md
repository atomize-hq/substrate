# Spec: Internal Family-2 Host-Global Inbox Layering And Local Obligation Materialization Boundary

Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Prior slice:
- [SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md](./SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md)
- [PLAN-50.md](./PLAN-50.md)
- [TASKS-50.md](./TASKS-50.md)
Related design stack:
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Phase: `SPECIFY`  
Status: landed runtime truth reviewed and Packet 4 closeout verified on `2026-06-08`
Validation note: Packet 4's validation wall is green. Final validation required one narrow in-scope stabilization follow-up in [`crates/shell/src/execution/host_inbox_materialization.rs`](../crates/shell/src/execution/host_inbox_materialization.rs): the pre-existing `load_host_inbox_record` fallback was simplified to `unwrap_or_default()` in two helper reads without changing Slice `51` behavior or reopening broader Family-2 scope.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `50` closed the local obligation-envelope question, so Slice `51` should be the first `SUBSTRATE_HOME/host_inbox/` layering slice rather than more local-ledger widening.
2. This slice should land only the bounded host-global inbox and local materialization boundary:
   - host-level ingress artifact under `SUBSTRATE_HOME/host_inbox/`
   - exact local materialization into the already-landed canonical obligation ledger
   - no broader cross-host sync, lease coordination, or federation routing productization
3. The host-global inbox must not become the canonical owner of local orchestration-session state; once materialized, the local obligation remains the authoritative deferred-work record.
4. The router must continue to act on local obligations only. It may run after materialization, but it must not consume `host_inbox` records directly as if they were obligations.
5. This slice should stay internal-only in v1: no new public CLI surface, no public daemon UX, and no new user-facing policy key should be required.
6. The slice should materialize only records whose exact local boundary truth is valid. If a host-inbox record targets another host or fails exact validation, the runtime should fail closed before creating a local obligation.
7. Existing Slice `48`/`49`/`50` review, auto-attach, wrong-host, and ingress/causation semantics should be reused rather than redesigned.

If any of these are wrong, correct them before implementation.

## Objective

Land the next Family-2 slice by introducing a bounded host-global inbox namespace under `SUBSTRATE_HOME/host_inbox/` and an exact local materialization path that turns eligible host-global ingress records into canonical local obligations, without widening into remote sync protocols, cross-host delivery productization, or a new public review surface.

Primary runtime story:

1. a host-level ingress record exists under `SUBSTRATE_HOME/host_inbox/`,
2. the record carries enough exact identity, target, and causation truth to describe one local deferred-work materialization candidate,
3. a sanctioned host-side materializer validates exact local boundary truth,
4. if the record is valid for this host, the materializer creates or reuses one canonical local obligation under the target orchestration session,
5. if the record is invalid for this host or lacks required exact truth, it fails closed without creating a local obligation,
6. once a local obligation exists, existing inbox/review and router-owned auto-attach behavior continue to consume only the obligation ledger,
7. no cross-host delivery engine, receive cursor sync protocol, or public host-global inbox UX lands in this slice.

## Observed Repo Floor

The current repo already has the local Family-2 floor this slice should reuse:

1. the canonical local obligation artifact exists as [`OrchestrationObligationRecord`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs),
2. local obligation persistence and reload behavior already live in [`state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs),
3. Slice `48` already landed the internal router-owned local auto-attach execution boundary,
4. Slice `49` already landed bounded host-targeting and wrong-host fail-closed posture for local obligations,
5. Slice `50` already landed ingress-ready identity and causation fields on the local obligation envelope.

The current repo now has the bounded host-global inbox layer above that local floor:

1. `SUBSTRATE_HOME/host_inbox/` persistence and read/write ownership live behind the state-store seam,
2. [`host_inbox.rs`](../crates/shell/src/execution/agent_runtime/host_inbox.rs) defines the internal host-inbox artifact plus exact `pending`, `materialized`, and `failed_closed` materialization-state truth,
3. [`host_inbox_materialization.rs`](../crates/shell/src/execution/host_inbox_materialization.rs) provides the sanctioned host-side local materializer that converts eligible host-global ingress records into canonical local obligations,
4. [`orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) runs that materialization pre-pass before router-owned auto-attach discovery while keeping the router consuming obligations only,
5. persisted host-inbox outcomes now prove whether a record materialized, failed closed, or remains pending for retry.

Slice `51` therefore lands as the bounded host-global inbox layering and exact local materialization slice without reopening local obligation semantics.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Expected runtime files:
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - one new bounded host-inbox module under `crates/shell/src/execution/agent_runtime/`
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) or one new bounded host-side execution file under `crates/shell/src/execution/` if the materialization entrypoint belongs outside the dispatch file
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs) only if a bounded helper is needed to keep materialized obligation construction exact and explanation-ready
- Expected docs:
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
  - [`docs/TRACE.md`](../docs/TRACE.md) remains unchanged for this slice because the current implementation does not add a new canonical trace record family for host-inbox materialization outcomes

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
cargo test -p shell obligation -- --nocapture
```

Expanded targeted validation if the host-side execution entrypoint changes:

```bash
cargo test -p shell orchestrator_world_dispatch -- --nocapture
cargo test -p shell auto_attach -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

The current repo structure relevant to this slice is:

- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative read/write seam for canonical local sessions and obligations
  - Slice `51` should extend this store with host-inbox path helpers and persistence helpers rather than inventing a second runtime-owned filesystem writer
- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - canonical local obligation schema and validation boundary
  - Slice `51` should reuse the landed host-targeting and ingress/causation envelope rather than widening it again
- `crates/shell/src/execution/agent_runtime/`
  - best place for one new bounded `host_inbox` module that defines the host-global ingress artifact and local materialization-state contract
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - current internal host-side router-owned entrypoint area
  - Slice `51` may add a bounded host-side materialization pass here if that is the narrowest place to sequence `host_inbox -> obligation -> existing router`
- `docs/`
  - docs must stay honest that Slice `51` lands host-global inbox layering and local materialization only, not remote sync or federation delivery
- `llm-last-mile/`
  - repo-local planning authority for this Family-2 follow-on slice

## Code Style

Follow the existing runtime style: exact identity, fail-closed boundaries, explanation-ready outcomes, and one authoritative durable seam per layer.

Preferred style:

```rust
if record.target_host_id.as_deref() != Some(local_host_id) {
    anyhow::bail!(
        "wrong_target_host: host inbox record {} targets host {} not local host {}",
        record.record_id,
        record.target_host_id.as_deref().unwrap_or("<missing>"),
        local_host_id,
    );
}

let obligation = materialize_host_inbox_record_as_local_obligation(&record)?;
store.persist_obligation(&obligation)?;
```

Conventions:

1. keep `host_inbox` records and local obligations as different layers with different ownership semantics,
2. materialize local obligations only from exact host-inbox truth, never from partial guesses,
3. make materialization idempotent and explanation-ready,
4. keep the router consuming obligations only, not host-inbox records directly,
5. do not widen into public CLI, public policy, or remote sync protocol work in this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- shell runtime regression suites

Test levels for this slice:

1. host-inbox artifact and persistence tests:
   - host-inbox records round-trip under `SUBSTRATE_HOME/host_inbox/`,
   - required exact identity fields validate,
   - invalid or blank target/ingress fields fail validation,
   - materialization state persists without becoming the canonical local obligation owner
2. local materialization tests:
   - a valid local host-inbox record materializes exactly one local obligation,
   - the materialized obligation preserves target/ingress/causation envelope truth from the host-inbox record,
   - repeated materialization attempts are idempotent and do not create duplicate obligations
3. fail-closed boundary tests:
   - wrong-host host-inbox records do not create local obligations,
   - records missing required exact local boundary truth do not create local obligations,
   - explanation-ready materialization failure state is persisted
4. coexistence tests:
   - once a local obligation is materialized, existing inbox projection and router-owned auto-attach continue to act on the obligation ledger only,
   - no path consumes `host_inbox` directly as router work

## Boundaries

- Always:
  - keep the local obligation ledger as the canonical local deferred-work artifact
  - keep `host_inbox` as a host-level ingress/materialization layer above local session truth
  - materialize only when exact local boundary truth is valid
  - keep router-owned attach consuming local obligations only
- Ask first:
  - adding a new public CLI or public daemon lifecycle surface for `host_inbox`
  - adding new cross-host sync, lease, or federation protocol behavior
  - widening the slice into new public policy keys or operator-facing workflow surfaces
- Never:
  - let `host_inbox` replace the local obligation ledger as the canonical session-local owner
  - let the router consume `host_inbox` records directly
  - materialize a local obligation from wrong-host or non-exact ingress truth
  - widen this slice into broader cross-host delivery or federation routing productization

## Success Criteria

This slice is complete only when all of the following are true:

1. the runtime has a bounded host-global inbox namespace under `SUBSTRATE_HOME/host_inbox/`,
2. the runtime has one internal host-inbox artifact model with exact identity, target, and materialization-state semantics suitable for local obligation materialization,
3. a sanctioned host-side materializer can convert a valid host-inbox record into exactly one canonical local obligation without duplicating obligations on re-run,
4. materialized obligations reuse the landed Slice `49`/`50` host-targeting and ingress/causation envelope truth rather than inventing a new local schema,
5. invalid local-boundary cases fail closed without creating local obligations,
6. router-owned auto-attach and inbox/review projection still consume only the local obligation ledger once materialization occurs,
7. no remote sync protocol, cross-host delivery productization, or new public host-global inbox UX lands.

## Open Questions

1. Should the first host-inbox slice require `target_host_id` on every host-inbox record, or permit a narrow local-only untargeted form for migration/bootstrap cases? Default assumption for this spec: require exact target-host truth for host-global records so the materialization boundary stays fail-closed and boring.
2. What is the narrowest durable materialization-state vocabulary for host-inbox records: `pending`, `materialized`, and `failed_closed`, or is an explicit `superseded` state also needed in Slice `51`? Default assumption for this spec: keep the first slice to `pending`, `materialized`, and `failed_closed`.
3. Resolved at implementation: the first host-side materializer runs as a bounded pre-pass near the existing router-owned host-side entrypoint, ahead of router auto-attach discovery, while still preserving the rule that only canonical local obligations become router work.
