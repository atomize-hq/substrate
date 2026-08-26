**Kind:** slice row
**Stable ID:** `c2-inbox-and-auto-attach-projections`
**Canonical for:** extracted C2 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 199
**Supersedes:** canonical ownership of the extracted `C2 — Inbox and auto-attach projections` row
**Superseded by:** none
**Projection consumers:** [`track-c-obligations-inbox-auto-attach-and-router-attach.md`](track-c-obligations-inbox-auto-attach-and-router-attach.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# C2 — Inbox and auto-attach projections

> **Authority boundary:** This file owns only the extracted C2 Track C row. It preserves the exact row text below, does not reopen C1, and does not dispatch C2.

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C2 — Inbox and auto-attach projections** | Make inbox and attach eligibility pure projections over obligation state and effective policy. | InboxProjection and AutoAttachProjection. | `02` projection rows; notification inbox design; auto-attach trigger design. | ObligationLedger; CompatibilityReadModel; SteeringPolicyEngine. | `agent_runtime/{host_inbox,obligation_ledger,auto_attach,state_store}.rs`; `host_inbox_materialization.rs`; projection tests. | No host process launch, no worker action, and no removal of compatibility ingress without migration proof. | Deleting/rebuilding projections does not lose obligation truth; session-coalesced claim is deterministic; wrong-host and policy denial fail closed. | Projection clauses of `RG-OBL-02`, `RG-ATTACH-01`, `RG-BASE-03`. | A1, A2/A3, C1, and B4; B4 transitively supplies B2.2/B3.2/E2. | C3 router launch and unrelated worker controls. |
