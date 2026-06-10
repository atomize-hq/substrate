# PLAN-51: Internal Family-2 Host-Global Inbox Layering And Local Obligation Materialization Boundary

Source spec: [SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md)  
Source prior slice: [PLAN-50.md](./PLAN-50.md)  
Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Plan type: first post-`50` host-global inbox layering slice  
Status: landed runtime truth reviewed and Packet 4 closeout verified on `2026-06-08`
Validation note: Packet 4's validation wall is green. Final validation required one narrow in-scope stabilization follow-up in [`crates/shell/src/execution/host_inbox_materialization.rs`](../crates/shell/src/execution/host_inbox_materialization.rs): two helper reads now use `unwrap_or_default()` for the existing unreadable-record fallback, which kept the closeout behavior-neutral and did not widen Slice `51`.

## Objective

Land the next Family-2 slice by introducing the first bounded host-global inbox layer under `SUBSTRATE_HOME/host_inbox/` and wiring one exact local materialization path into the already-landed local obligation ledger, without widening into remote sync protocols, cross-host delivery productization, or a new public operator surface.

This slice is complete only when all of the following are true:

1. the runtime has one bounded host-global inbox artifact family under `SUBSTRATE_HOME/host_inbox/`,
2. host-inbox records can persist exact identity, target, ingress, and materialization outcome truth without becoming the canonical local deferred-work owner,
3. a sanctioned host-side materializer can convert valid records into exactly one local obligation,
4. repeated materialization attempts are idempotent,
5. invalid local-boundary cases fail closed before local obligation creation,
6. router-owned auto-attach and review/inbox projection still consume only local obligations after materialization,
7. remote sync, lease/lock coordination, and broader federation delivery remain deferred.

## Packet 4 Closeout Note

Implementation status on the current tree:

1. Packets `1` through `3` are already landed in runtime code before this session.
2. Packet `4` aligned docs to the landed `host_inbox -> local obligation -> router` boundary, reran the full validation wall, and carried one explicit bounded runtime follow-up discovered during validation: a behavior-neutral helper simplification in `host_inbox_materialization.rs`.
3. `docs/TRACE.md` stays untouched because the implementation logs explanation-ready materialization outcomes through the existing dispatch logging surface rather than adding a new canonical trace record family.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md) has been reviewed and is the source of truth for scope before implementation planning advances.

## Major Components And Dependencies

1. one new bounded host-inbox module under `crates/shell/src/execution/agent_runtime/`
   - owns the canonical host-global inbox artifact, validation rules, and materialization-state vocabulary for this slice
   - must stay host-level and must not become a second local obligation ledger
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - owns durable runtime storage and write ownership for local session artifacts
   - should extend to host-inbox path helpers and host-inbox persistence/materialization helpers rather than allowing ad hoc filesystem writes elsewhere
3. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
   - already owns the canonical local obligation schema
   - should be reused as-is unless a bounded helper is needed for exact materialization mapping
4. host-side execution wiring under `crates/shell/src/execution/`
   - needs one internal-only materialization entrypoint
   - must keep the router consuming obligations only, never host-inbox records directly
5. `docs/CONFIGURATION.md` and `docs/TRACE.md` if needed
   - must stay honest about what Slice `51` lands and what it still defers

## Plan Summary

After Slice `50`, the remaining Family-2 work narrowed again:

1. the local obligation ledger already has the needed host-targeting and ingress-ready envelope,
2. the router already consumes local obligations and fails closed on wrong-host local materialization,
3. the missing seam is now above the local ledger: a host-global ingress layer that can feed it without replacing it.

The design stack is already clear enough to freeze that seam now:

1. the host-global inbox is a host-level ingress/materialization layer under `SUBSTRATE_HOME`,
2. the local obligation ledger remains the canonical local deferred-work artifact,
3. remote/global ingress must materialize local obligations before the router acts,
4. exact local-boundary failures must stop before local obligation creation,
5. broader cross-host sync and federation delivery remain later work.

The narrowest honest Slice `51` is therefore:

1. freeze the host-inbox artifact and materialization-state contract first,
2. land host-inbox persistence and idempotent local-obligation materialization second,
3. add one bounded host-side execution entrypoint and coexistence checks third,
4. finish with docs and validation last.

## Locked Decisions

### What changes

1. Slice `51` introduces the first bounded `SUBSTRATE_HOME/host_inbox/` runtime namespace.
2. Slice `51` defines one internal host-inbox artifact contract for local materialization.
3. Slice `51` materializes local obligations from valid host-inbox records instead of routing on host-inbox records directly.
4. Slice `51` records materialization outcomes durably and explanation-readily.
5. Slice `51` reuses the landed local obligation envelope from Slices `49` and `50`.

### What does not change

1. no public host-inbox CLI or daemon UX,
2. no remote sync protocol, lease coordination, or federation routing productization,
3. no new public policy surface,
4. no router consumption of host-global records directly,
5. no replacement of the local obligation ledger as canonical session-local truth.

## Implementation Order

### Packet 1: Host-Inbox Contract Freeze

Goal:

1. freeze the bounded host-global inbox artifact shape,
2. freeze the minimum materialization-state vocabulary,
3. preserve the hard boundary that host-inbox is not the canonical local obligation owner.

Primary touch surface:

1. one new bounded host-inbox module under `crates/shell/src/execution/agent_runtime/`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. targeted host-inbox contract and persistence tests

Why first:

1. the host-global layer needs one frozen artifact before persistence or execution logic can depend on it,
2. this slice must not drift into a second obligation ledger,
3. idempotent materialization depends on a clear materialization-state contract.

Verification checkpoint:

1. the host-inbox artifact preserves exact identity, target, ingress, and materialization-state truth,
2. no local obligation schema widening is required beyond the landed Slice `50` floor,
3. host-inbox ownership is explicitly separate from local obligation ownership.

### Packet 2: Host-Inbox Persistence And Local Materialization

Goal:

1. persist host-inbox records under `SUBSTRATE_HOME/host_inbox/`,
2. materialize a valid host-inbox record into exactly one local obligation,
3. make materialization idempotent and fail closed on invalid local-boundary truth.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. the new host-inbox module
3. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs` only if a bounded materialization helper is needed
4. targeted state-store and obligation tests

Why second:

1. the host-global layer is only useful if it can create canonical local obligations,
2. local obligation materialization is the semantic center of this slice,
3. idempotence and fail-closed behavior should be proven before adding any host-side execution wiring.

Verification checkpoint:

1. valid records materialize one local obligation exactly once,
2. repeated materialization attempts do not create duplicates,
3. wrong-host or invalid records do not create local obligations,
4. materialized obligations preserve the landed host-targeting and ingress/causation envelope truth.

### Packet 3: Host-Side Execution Boundary And Router Coexistence

Goal:

1. introduce one internal-only host-side materialization entrypoint,
2. prove that existing router-owned attach remains downstream of local obligation materialization rather than consuming host-inbox records directly,
3. keep the slice explanation-ready without widening into workflow-engine or federation work.

Primary touch surface:

1. host-side execution wiring under `crates/shell/src/execution/`
2. the new host-inbox module
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. `crates/shell/src/execution/agent_runtime/auto_attach.rs` or [`orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) only if narrow coexistence tests or bounded sequencing require it

Why third:

1. execution wiring should consume a frozen persistence/materialization contract,
2. the main semantic risk after Packet 2 is accidentally letting the router act on host-inbox records directly,
3. coexistence proof should stay bounded before docs are updated.

Verification checkpoint:

1. a non-test internal host-side materialization entrypoint exists,
2. router-owned auto-attach still consumes obligations only,
3. host-inbox materialization outcomes are explanation-ready,
4. no public daemon lifecycle or federation control surface is introduced.

### Packet 4: Docs Alignment And Validation

Goal:

1. align docs with the landed host-global inbox layering boundary,
2. keep broader cross-host delivery and federation work explicitly deferred,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `docs/TRACE.md` only if a new canonical trace family lands
3. `llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md`
4. `llm-last-mile/PLAN-51.md`
5. `llm-last-mile/TASKS-51.md`

What this packet must enforce:

1. docs describe Slice `51` as host-global inbox layering plus local materialization only,
2. docs keep the local obligation ledger as canonical local truth,
3. docs do not imply remote sync, lease coordination, cross-host delivery, or public host-inbox UX have landed.

Verification checkpoint:

1. docs and runtime truth are honest,
2. validation is green,
3. the next Family-2 follow-on can stay focused on broader delivery/sync work instead of reopening local materialization semantics.

## Risks And Mitigations

1. Risk: Slice `51` accidentally creates a second canonical local deferred-work ledger.
   Mitigation: keep host-inbox artifact semantics strictly host-level ingress/materialization only; the local obligation remains authoritative after materialization.
2. Risk: the slice drifts into cross-host sync or federation productization.
   Mitigation: keep the first host-global slice local-host materialization only and defer sync/lease/delivery protocol work.
3. Risk: repeated materialization creates duplicate local obligations.
   Mitigation: freeze one deterministic idempotence rule and persist materialization outcomes durably.
4. Risk: router-owned auto-attach begins consuming host-inbox records directly.
   Mitigation: keep one explicit `host_inbox -> local obligation -> existing router` ordering and test it.
5. Risk: wrong-host or non-exact records create unsound local obligations.
   Mitigation: validate target/ingress boundary truth first and fail closed before obligation creation.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because persistence and materialization need one frozen host-inbox contract.
2. Packet 2 before Packet 3 because the host-side execution path should consume the landed idempotent materialization boundary, not a provisional one.
3. Packet 3 before Packet 4 because docs should describe the real execution boundary honestly.

### Can be deferred

1. remote sync and receive-cursor protocols,
2. lease or lock coordination for cross-host delivery,
3. public host-inbox review or management UX,
4. broader cross-host delivery and federation routing/productization.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell host_inbox -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell obligation -- --nocapture
cargo test -p shell orchestrator_world_dispatch -- --nocapture
cargo test -p shell auto_attach -- --nocapture
cargo test --workspace -- --nocapture
```

If Packet 3 lands without changing router coexistence or dispatch sequencing, the targeted `orchestrator_world_dispatch` or `auto_attach` rerun may be skipped, but the final closeout must say explicitly why it was unnecessary.

## Expected Follow-On Order After Slice `51`

If Slice `51` lands cleanly as the host-global inbox layering and local materialization slice, the next Family-2 follow-on should remain:

1. broader host-global ingress lifecycle work such as receive-cursor or sync-state coordination if still needed,
2. only after that, any broader cross-host delivery, remote ingress materialization, lease/lock coordination, or federation routing/productization.

Slice `51` is therefore a boundary slice: it lands the first host-global ingress layer while preserving the design rule that local obligations remain the canonical local deferred-work truth.
