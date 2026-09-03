**Kind:** slice row
**Stable ID:** `a3-persistence-and-compatibility-split`
**Canonical for:** extracted A3 slice row plus its terminal closure
**Status:** terminally complete
**Authority scope:** exact extracted source table header and row, the terminal closure below, and a projection of the bounded A3-scoped `RG-BASE-03` disposition; no successor admission, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 132
**Supersedes:** canonical ownership of the extracted `A3 — Persistence and compatibility split` row
**Superseded by:** none
**Projection consumers:** [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# A3 — Persistence and compatibility split

> **Authority boundary:** This file owns the extracted A3 Track A row and its terminal closure. The `RG-BASE-03` note below projects the canonical evidence disposition and adds no successor authority here. This file preserves the exact row text; the closure records completed authority and proof without admitting or dispatching E2, C2, or any other successor.

## RG-BASE-03 completion-wall projection

**Projection only:** the
[canonical disposition](../evidence/baseline-behaviors-and-smoke-scenarios.md#a3-scoped-rg-base-03-disposition)
keeps `RG-BASE-03` open, blocking, unwaived, and unchanged as a stable gate; the failed full
targeted-host-turn/UAA continuity witness is not green. A3 consumes only the bounded differential
disposition recorded there:
unchanged canonical/compatibility behavior, usable unprefixed `ls`/`pwd`, the required `cd ../` cage
denial, zero compatibility-read mutation, and no regression relative to the exact baseline. Honest
baseline-equivalent failure at the later lifecycle boundary permits A3 verification to continue,
but any candidate-specific persistence or compatibility defect still blocks A3.

The full later targeted-turn continuity proof remains with its existing downstream owner,
[C2](c2-inbox-and-auto-attach-projections.md#rg-base-03-continuity-proof-owner-projection), after C2's declared
prerequisites. A3 may not weaken the activated-store legacy-writer guard or add HSA,
obligation-ledger, receipt, supervisor, or lifecycle semantics to make that witness pass. This
closure does not admit or dispatch C2 and does not mark the full continuity witness green.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A3 — Persistence and compatibility split** | Reduce StateStore to atomic persistence/schema evolution and isolate diagnostic compatibility reads from new authority writes. | `02` StateStore + CompatibilityReadModel rows; `01` authority map; `04` durable revision rules | HostSessionAuthority; InboxProjection; receipt persistence | `agent_runtime/state_store.rs`; a bounded new facade/module under `agent_runtime/`; persistence/read-model tests | No wholesale database rewrite; no conversion of pre-A1 authority artifacts into A1 authority; no behavior changes outside moved ownership. | New authority writes bypass compatibility projection; torn-root/unsupported-state diagnostics remain read-only; newer revisions always win. | `RG-AUTH-02`, `RG-BASE-01`, `RG-BASE-03` |

## Terminal closure

The completed implementation is bound to:

- commit `627e0febf9e0a5143a8b92ba54f1c9377d3ffae5`, with sole parent
  `cdfd2a9566ad01d9e05011aff56612698aa7491b`;
- tree `2fe1acc6d0abe2a50c2f44dc47f636f51223d390`;
- subject `refactor: split runtime compatibility reads from persistence`;
- reviewed fingerprint
  `sha256:0c97b5f30950114c3a0245040d8615e05719370f55c31e2e2581fc3d4c2824ab`; and
- final independent implementation verdict `CLEAN`.

The production StateStore read entry points now construct `CompatibilityReadModel` through the
bounded `CompatibilityReadSource` capability. That capability exposes reads only: it has no
transition, publication, removal, root/path, transaction, or generic write operation. Its
filesystem and transaction-backed implementations remain inside `state_store.rs`, where the
trusted-root identity checks, existing root lock, atomic write/rename, rollback, publication, and
file/directory `fsync` ownership remain. Compatibility code therefore cannot mutate durable bytes,
advance an HSA revision, or create an absent authority namespace.

The extracted read model preserves canonical-over-flat-over-legacy precedence, participant and
session fallback behavior, and torn-root/unsupported-state diagnostics. Canonical newer authority
continues to win; stale participant/session snapshots and activated-store legacy writers continue
to fail closed. The implementation changes no public interface, durable JSON bytes or format,
schema version, migration, HSA semantics, A1/A2 lifecycle behavior, public Stop, episode,
auto-attach, receipt, supervisor, or platform implementation.

The exact A3 evidence wall passed all 17 focused boundary, namespace-absence, trusted-root,
locking, precedence, torn-root, stale/newer-revision, legacy-writer, temp-reconciliation, and exact-
retry tests. The broader library compatibility wall passed 16 of 18 tests; its two
`compatibility_spawn_*` failures reproduce with the same names and failure boundary against exact
baseline commit `d6c06082c9287256a9d56005991bf9962059660c`, so neither is counted as A3 proof
or an A3 regression. The landed RG-BASE-03 correction additionally records unchanged
canonical/compatibility behavior, usable unprefixed `ls`/`pwd`, required `cd ../` cage denial, and
zero compatibility-read mutation. All candidate-specific review findings were remediated before
the recorded implementation `CLEAN` verdict. The unfiltered package-target invocation does not
reach that wall because unchanged `managed_lifecycle_v1` compilation still cannot resolve
`MAC_PUBLISHER_SERVICE_LABEL_V1`; both the integration target and included lifecycle-control source
are byte-identical to the exact baseline, so this pre-existing compile failure is not counted as
A3 proof, regression, or waiver.

The frozen implementation hashes are `compatibility.rs`
`6206353354fd2467ac8c8043e7cf917500dbcf1de005d53510d27f75e33e6da7`, `state_store.rs`
`2c8c042cf356bc4e757cf4e077437727dd8ed82bcb914e001b4234c540c1f17b`, and `mod.rs`
`1b6bbba0ebaf0640116a90d2d86fade126e6fe2bc587f1ddd0b42f838fea56e3`.

The A3-scoped clause of `RG-AUTH-02` is green, and `RG-BASE-01` remains green through A3.
A3's bounded `RG-BASE-03` differential disposition is satisfied, but the full gate remains open,
blocking, unwaived, and not green under C2; no C2 authority is exercised. A3 is terminally complete
and no longer active. Because Track A contains only A0 through A3, A0's committed inventory
satisfies its diagnostic exit, and A1/A2 are already terminal, Track A is also terminally complete.
E1 subsequently reached terminal closure at
`18f719898ce2a48f65e95b3b23f3b2cfd685c4af` over implementation
`6194788d45267d91b4428a42e24c02dfcaae3c1e`; E2 later reached terminal closure over implementation
`96e102d9f5690e0d63957f6e9db56d632b7cdd17`. The A3 closure itself did not admit, dispatch,
implement, or complete E1, E2, or any later track.
