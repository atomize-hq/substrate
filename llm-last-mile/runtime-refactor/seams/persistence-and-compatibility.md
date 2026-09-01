**Kind:** seam family
**Stable ID:** `persistence-and-compatibility-family`
**Canonical for:** persistence and compatibility projection seam extraction
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies plus the current A3 closure-status overlay only; no implementation or successor authority
**Source span:** D8 persistence and compatibility family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `StateStore` and `CompatibilityReadModel` rows
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Persistence and compatibility projection seam extraction

> **Authority boundary:** This file owns only the extracted `StateStore` and `CompatibilityReadModel` seam rows and the current A3 closure overlay. It does not move the A0 authority-leak inventory, reopen D5/D6/D7 families, promote sibling seams, or authorize successor implementation work.

## A3 closure-status overlay (2026-09-01; current)

The extracted packet-era rows below remain byte-stable. Under the exact implementation and proof
identities in the [A3 terminal closure](../slices/a3-persistence-and-compatibility-split.md#terminal-closure),
the bounded StateStore/CompatibilityReadModel ownership transition is
`ContractCorrectAndProven`: production StateStore reads enter the separate read model only through
a read-only capability; capability construction, trusted-root identity, locking, atomic
persistence, rollback, publication, and file/directory `fsync` remain in StateStore; and focused
plus bounded differential proof exercises that path. No durable format, schema version, migration,
or public interface changed.

This qualification promotes the A3 read-model boundary only. It does not move semantic authority
back into StateStore, promote unrelated StateStore surfaces or InboxProjection, close the full
`RG-BASE-03` continuity witness, or admit E2 or C2.

## StateStore

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| StateStore | `AgentRuntimeStateStore`, session/participant persistence, atomic JSON writes, active-task records, obligation/inbox persistence in `agent_runtime/state_store.rs` | `UsefulFootholdButWrongBoundary` | no | not applicable | no | A1.1 retains and hardens only persisted physical-home bootstrap classification, operation-bound temp reconciliation, cross-process atomic persistence, immutable object/key storage, immutable greenfield certification, complete pre-A1 collection enumeration/rejection, and revision-CAS. Under A1.1d every in-repository transaction that reads or mutates either pre-A1 authority collection—including flat/canonical snapshots, leases, removals, compatibility/read-repair persistence, and parent-session persistence triggered by another operation—must perform its complete read, decision, write, rename, removal, and final `fsync` through the same retained opened physical-root transaction. The lock protects that opened root only: rename, replacement, rebind, or identity uncertainty fails closed and leaves the replacement tree untouched. A1.1e exposes accepted-home operations through a distinct bound capability with no lexical descendant-path API; replacement, rebind, or identity uncertainty fails before a bound read or write can consume the replacement tree. Stale lifecycle and world-binding rejection is correct and remains fail-closed. StateStore persists the transition selected and validated by HostSessionAuthority and may persist an exact policy/snapshot identity; it must not parse, compose, finalize, choose, reconstruct, or synthesize effective policy or a successor lifecycle transition. For B1/B2.1 it may supply only separately scoped opaque physical capabilities for the receipt registry and supervisor journal; it cannot become their semantic owner or a generic activated-store writer. No converter, dual-write path, unrelated StateStore redesign, or general persistence extraction is added. A1.1d remains incomplete. | HostSessionAuthority; CompatibilityReadModel; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; ObligationLedger |

## CompatibilityReadModel

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| CompatibilityReadModel | torn-root fallback, synthesized session records, legacy inbox projection, read repair, status-visible participant logic in `agent_runtime/state_store.rs` | `UsefulFootholdButWrongBoundary` | no | not applicable | no | A1.1 does not consume compatibility state as authority: any pre-A1 session/participant artifact is `UnsupportedLegacyState`, and unreadable/uncertain collections fail closed. During A1.1d, any compatibility read that participates in a read-decide-write transaction and any compatibility/read-repair persistence into the guarded collections is bound to the same retained opened physical-root transaction; it cannot escape through a path-based helper or treat root replacement as authorization. A1.1d does not extract or promote this seam. A3 still owns the later persistence/compatibility separation and read-only projection/diagnostics; no missing/unreadable projection may imply `ExpectedAbsent` or override newer authority. | StateStore; HostSessionAuthority; InboxProjection |
