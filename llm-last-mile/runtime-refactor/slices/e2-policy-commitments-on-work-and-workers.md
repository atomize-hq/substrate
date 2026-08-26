**Kind:** slice row
**Stable ID:** `e2-policy-commitments-on-work-and-workers`
**Canonical for:** extracted E2 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 221
**Supersedes:** canonical ownership of the extracted `E2 — Policy commitments on work and workers` row
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# E2 — Policy commitments on work and workers

> **Authority boundary:** This file owns only the extracted E2 Track E row. It preserves the exact row text below, does not reopen B1/B2.1 or B3.1/C1 families, and does not dispatch E2.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E2 — Policy commitments on work and workers** | Build immutable active-run snapshots and retained-worker caps linked to B1 acceptance records; recompute future turns as parent ∧ cap ∧ turn patch before B2.2/B3.2 expose final receipts. | `04` acceptance/receipt/manifest and immutable snapshot rules; `02` ReceiptRegistry/RetainedRuntime rows | B1 acceptance records; B3.1 event truth; B2.2/B3.2 consumers; E1 resolver; fork lifecycle | receipt/manifest persistence; `orchestrator_world_dispatch.rs`; retained lifecycle code; policy tests | No mutation of accepted snapshots; no automatic worker broadening; no config rendering. | Final receipts/manifests reference their B1 acceptance record and record immutable hash/ref/revision/reason; parent narrowing affects future turns; parent broadening does not widen worker; fork inherits cap. | `RG-POLICY-03`, `RG-RECEIPT-02`, `RG-CANCEL-01`; E2 policy clause of `RG-OBS-01` |
