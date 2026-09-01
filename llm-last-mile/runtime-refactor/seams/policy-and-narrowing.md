**Kind:** seam family
**Stable ID:** `policy-and-narrowing-family`
**Canonical for:** policy and narrowing seam extraction plus the current E1/E2 disposition
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies plus the documentation-only E1/E2 disposition; no implementation authority
**Source span:** D8 policy and narrowing family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `SteeringPolicyEngine`, `EffectivePolicyResolver`, and `DispatchPolicyNarrowingPatch` rows
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Policy and narrowing seam extraction

> **Authority boundary:** This file owns only the extracted `SteeringPolicyEngine`, `EffectivePolicyResolver`, and `DispatchPolicyNarrowingPatch` seam rows. It does not promote any seam, move the A0 authority-leak inventory, reopen D5/D6/D7 or already-extracted D8 family owners, rewrite `b1-b2-1/` or `b3-1-c1/`, or authorize E-track/B3.2/B4 implementation work.

## Current E1/E2 disposition

The extracted rows below preserve their pre-E1 chronology. E1's strict authenticated patch,
broker composition, canonical `PolicySnapshotV3`, and containment/enforcement scope is terminally
closed at `18f719898ce2a48f65e95b3b23f3b2cfd685c4af` over implementation
`6194788d45267d91b4428a42e24c02dfcaae3c1e`. E2 remains undispatched and owns the separate
immutable [`DispatchPolicyCommitmentV1`](../contracts/dispatch-policy-commitment-v1.md), not
receipt/full-manifest construction. EffectivePolicyResolver composes; the bounded
DispatchPolicyCommitmentRegistry persists and links; later receipt/manifest owners consume the
exact ref. For retained Continue/Fork, the resolver consumes independently authenticated current
parent and immutable worker/source-worker cap; launch-policy equality with current parent is not a
prerequisite. Parent narrowing may further restrict future work and broadening cannot widen the
cap.

E2 policy-snapshot bytes are exactly the landed E1
`serde_json::to_vec(PolicySnapshotV3)` bytes, and E2 uses the exact E1/B1 SHA-256 over that sequence.
It decodes and equality-checks the supplied bytes against the expected E1 snapshot and requires
reserialization to be byte-identical. E2 record/index/link hashes remain separately
domain-separated deterministic preimages; their recursive key sorting does not redefine the policy
snapshot hash. This correction changes no `PolicySnapshotV3`, E1 serialization, schema 3, B1 stored
hash, broker composition, or enforcement behavior.

## SteeringPolicyEngine

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| SteeringPolicyEngine | `agents.world_dispatch.*` fields in `crates/broker`; `enforce_world_dispatch_steering_policy` and event/payload checks in `orchestrator_world_dispatch.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | Consolidate deny-by-default action/mode/backend/session/world/event/autonomy decisions into one explanation-ready boundary used by every ingress path. | WorldDispatchControl; EffectivePolicyResolver; ObligationLedger; WorldWorkerMessagingProtocol |

## EffectivePolicyResolver

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| EffectivePolicyResolver | `crates/broker/src/effective_policy.rs`; `execution/policy_model.rs`; `execution/policy_snapshot.rs`; current policy composition in dispatch code | `UsefulFootholdButWrongBoundary` | no | no | no | The broker remains the sole owner of effective-policy parsing, default/global/workspace layering, validation, finalization, and explanation provenance. A1.1e adds one bounded broker entry point that accepts an explicit global-policy source derived by shell code from the already-accepted bootstrap home and reuses the canonical broker pipeline; shell code only supplies that input and consumes the result. Differential parity is focused-proof clean, but no real dispatch path or enforcement point adopts this entry point yet. Later E-track parent/cap/turn composition, dispatch narrowing, immutable acceptance, and enforcement remain unresolved: add the dispatch acceptance API there without treating the A1.1e entry point as closure or promoting this seam. | SteeringPolicyEngine; DispatchPolicyNarrowingPatch; WorldWorkReceiptRegistry; WorldCommandExecutionBroker |

## DispatchPolicyNarrowingPatch

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| DispatchPolicyNarrowingPatch | `allow_capability_narrowing` policy flag; boolean capability overrides in `dispatch_contract.rs`; inventory-time `policy_overlay.world_fs` validation in `agent_inventory.rs` | `MissingSeam` | no | no | no | Add request-scoped restricted `PolicyPatch.world_fs`; implement path-containment monotonicity; persist narrowing reason and resulting snapshot on task/turn/worker records. | EffectivePolicyResolver; WorldWorkReceiptRegistry; RetainedWorkerRuntime; AgentConfigProjectionService |
