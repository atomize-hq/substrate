**Kind:** slice row
**Stable ID:** `a3-persistence-and-compatibility-split`
**Canonical for:** extracted A3 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 132
**Supersedes:** canonical ownership of the extracted `A3 — Persistence and compatibility split` row
**Superseded by:** none
**Projection consumers:** [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# A3 — Persistence and compatibility split

> **Authority boundary:** This file owns only the extracted A3 Track A row. It preserves the exact row text below, does not authorize persistence/compatibility implementation, and does not dispatch A3.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A3 — Persistence and compatibility split** | Reduce StateStore to atomic persistence/schema evolution and isolate diagnostic compatibility reads from new authority writes. | `02` StateStore + CompatibilityReadModel rows; `01` authority map; `04` durable revision rules | HostSessionAuthority; InboxProjection; receipt persistence | `agent_runtime/state_store.rs`; a bounded new facade/module under `agent_runtime/`; persistence/read-model tests | No wholesale database rewrite; no conversion of pre-A1 authority artifacts into A1 authority; no behavior changes outside moved ownership. | New authority writes bypass compatibility projection; torn-root/unsupported-state diagnostics remain read-only; newer revisions always win. | `RG-AUTH-02`, `RG-BASE-01`, `RG-BASE-03` |
