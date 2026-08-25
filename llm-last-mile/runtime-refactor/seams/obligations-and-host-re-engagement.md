**Kind:** seam family
**Stable ID:** `obligations-and-host-re-engagement-family`
**Canonical for:** obligations and host re-engagement seam extraction plus related owner navigation
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies plus the preserved related-owner link only; no implementation authority
**Source span:** D8 obligations and host re-engagement family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `InboxProjection`, `AutoAttachProjection`, and `RouterAttachTrigger` rows; the related `ObligationLedger` row remains canonical under `b3-1-c1/`
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Obligations and host re-engagement seam family

> **Authority boundary:** This file owns only the extracted `InboxProjection`, `AutoAttachProjection`, and `RouterAttachTrigger` seam rows and the preserved related-owner link for the already-extracted `ObligationLedger` family row. It does not promote any seam, move the A0 authority-leak inventory, reopen `b3-1-c1/`, extract the D-section seam families, or authorize B3.1/C1/B4/D9+ implementation work.

## Related canonical owner

`ObligationLedger` already has its canonical family row under [`../b3-1-c1/crosswalk.md#obligationledger`](../b3-1-c1/crosswalk.md#obligationledger). This D8 landing preserves that D6 owner unchanged.

## InboxProjection

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| InboxProjection | `agent_runtime/host_inbox.rs`; compatibility durable inbox items in `state_store.rs`; `execution/host_inbox_materialization.rs` | `DefensiveScaffoldingOnly` | no | not applicable | no | Make inbox a read/projection model over canonical obligations; isolate compatibility ingress; prevent inbox rows or counts from becoming independent authority. | ObligationLedger; CompatibilityReadModel; AutoAttachProjection |

## AutoAttachProjection

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| AutoAttachProjection | attach fields on obligation records; eligibility/claim helpers in `agent_runtime/auto_attach.rs` | `DefensiveScaffoldingOnly` | no | no | no | Derive eligibility and claim state only from canonical obligations plus effective policy; make claims idempotent and session-coalesced. | ObligationLedger; InboxProjection; RouterAttachTrigger; SteeringPolicyEngine |

## RouterAttachTrigger

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| RouterAttachTrigger | router discovery and helper launch in `agent_runtime/auto_attach.rs`; detached `spawn_blocking` trigger in `orchestrator_world_dispatch.rs` | `DefensiveScaffoldingOnly` | no | no | no | Move to an explicit durable trigger/claim/settle path; restore sanctioned host ownership once; record result; prohibit prompt replay and direct worker actions. | AutoAttachProjection; HostSessionAuthority; SurfaceAdapter; ObligationLedger |
