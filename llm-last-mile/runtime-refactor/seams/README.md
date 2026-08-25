**Kind:** seam index
**Stable ID:** `shared-seam-crosswalk`
**Canonical for:** shared seam-crosswalk reading rules, shared table-column meanings, classification consequences, and extracted seam-family navigation
**Status:** canonical shared seam-crosswalk record
**Authority scope:** exact extracted shared seam-crosswalk source bodies plus navigation for extracted seam families only; no implementation authority
**Source span:** D8 shared seam extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted root `Reading rule` and `Classification consequences` spans; repeated seam-table headers are projections of this file's shared column definitions
**Superseded by:** none
**Projection consumers:** [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md), [`../index/README.md`](../index/README.md), [`host-session-authority.md`](host-session-authority.md), [`persistence-and-compatibility.md`](persistence-and-compatibility.md), [`dispatch-and-episode-transport.md`](dispatch-and-episode-transport.md), [`policy-and-narrowing.md`](policy-and-narrowing.md), [`runtime-event-receipt-supervision-and-retained-runtime.md`](runtime-event-receipt-supervision-and-retained-runtime.md), [`obligations-and-host-re-engagement.md`](obligations-and-host-re-engagement.md), [`configuration-and-gateway-adoption.md`](configuration-and-gateway-adoption.md), [`uaa-provider-realization-and-side-effect-mediation.md`](uaa-provider-realization-and-side-effect-mediation.md)

# Runtime seam crosswalk index

> **Authority boundary:** This file owns only the extracted shared seam-crosswalk reading rules, repeated table-column meanings, classification consequences, and navigation for extracted seam families. It does not authorize implementation, promote any seam, alter packet/gate status, reopen D5/D6/D7 families, or move the A0 authority-leak inventory out of `02-seam-crosswalk.md`.

## Reading rule

This is a semantic assessment of the target seam, not an inventory-completeness score. `proven by smoke/e2e = no` means the full target seam lacks production-path proof even when component or integration tests exist.

Promotion rule:

```text
correct owner
+ real call path
+ intended enforcement
+ smoke/e2e/regression proof
= ContractCorrectAndProven
```

No required seam currently meets all four conditions.

Supporting-primitive correction: the managed `world-service` -> `substrate-gateway` `GatewayAuthBundleV1` secure-FD carrier exists and has focused launcher/consumer integration proof. Its presence does not promote the wider config/envelope/realization seams because direct world Codex still bypasses that consumer path and uses copied seed-home auth/config. Preserve the carrier; close Codex adoption and Substrate-owned projection separately.

## Shared crosswalk table columns

The repeated seam tables in root compatibility projections and extracted seam-family files preserve these shared column meanings. Any repeated header is a projection of this column set, not a separate authority record.

| Column | Meaning |
|---|---|
| `Seam` | assessed seam name |
| `Current code artifacts` | currently observed implementation surface |
| `Current semantic status` | current seam-wide semantic status label |
| `Authority boundary correct?` | whether the present authority owner is already correct |
| `Enforcement point correct?` | whether the present enforcement point is already correct |
| `Proven by smoke/e2e?` | whether the full seam currently has production-path proof |
| `Refactor action` | preserved target-owner transition rule plus explicit non-promotions and exclusions |
| `Sibling seams that must stay in context` | adjacent seams that remain required context when interpreting the row |

## Classification consequences

- `UsefulFootholdButWrongBoundary` means preserve reusable code only after its authority placement is corrected.
- `DefensiveScaffoldingOnly` means retain it during transition if useful, but do not design later slices as though the target seam exists.
- `MislandedWrongModel` requires replacing the semantic center, not patching symptoms around it.
- `MissingSeam` may still have strong neighboring primitives. Those primitives are inputs to the seam, not proof of it.

## Extracted seam families

| Family | Canonical owner | Current extracted scope |
|---|---|---|
| Host/session authority | [`host-session-authority.md`](host-session-authority.md) | The extracted `SurfaceAdapter / HostExecutionEpisode` row plus a preserved link to the already-canonical `HostSessionAuthority` family row. |
| Persistence and compatibility projection | [`persistence-and-compatibility.md`](persistence-and-compatibility.md) | The extracted `StateStore` and `CompatibilityReadModel` rows. |
| Dispatch and episode transport | [`dispatch-and-episode-transport.md`](dispatch-and-episode-transport.md) | The extracted `InternalToolboxTransport`, `RuntimeToolInvocationAdapter`, and `WorldDispatchControl` rows. |
| Policy and narrowing | [`policy-and-narrowing.md`](policy-and-narrowing.md) | The extracted `SteeringPolicyEngine`, `EffectivePolicyResolver`, and `DispatchPolicyNarrowingPatch` rows. |
| Runtime event, receipt, supervision, and retained runtime | [`runtime-event-receipt-supervision-and-retained-runtime.md`](runtime-event-receipt-supervision-and-retained-runtime.md) | The extracted `RuntimeEventTransport`, `WorldWorkReceiptRegistry`, `WorldWorkExecutionSupervisor`, and `RetainedWorkerRuntime` rows while leaving the existing D6 `WorldWorkerMessagingProtocol` owner unchanged. |
| Obligations and host re-engagement | [`obligations-and-host-re-engagement.md`](obligations-and-host-re-engagement.md) | The extracted `InboxProjection`, `AutoAttachProjection`, and `RouterAttachTrigger` rows plus preserved navigation to the existing D6 `ObligationLedger` owner. |
| Configuration and gateway adoption | [`configuration-and-gateway-adoption.md`](configuration-and-gateway-adoption.md) | The extracted `AgentConfigProjectionService` and `WorldRuntimeAdapterExecutionEnvelope` rows. |
| UAA/provider realization and side-effect mediation | [`uaa-provider-realization-and-side-effect-mediation.md`](uaa-provider-realization-and-side-effect-mediation.md) | The extracted `WorldCommandExecutionBroker` and `RuntimeFamilyRealizationAdapter` rows. |
