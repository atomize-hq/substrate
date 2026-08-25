**Kind:** seam family
**Stable ID:** `uaa-provider-realization-and-side-effect-mediation-family`
**Canonical for:** UAA/provider realization and side-effect mediation seam extraction
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** D8 UAA/provider realization and side-effect mediation family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `WorldCommandExecutionBroker` and `RuntimeFamilyRealizationAdapter` rows
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# UAA/provider realization and side-effect mediation seam family

> **Authority boundary:** This file owns only the extracted `WorldCommandExecutionBroker` and `RuntimeFamilyRealizationAdapter` seam rows. It does not promote any seam, move the A0 authority-leak inventory, reopen D5/D6/D7 or already-extracted D8 family owners, absorb the separate configuration/gateway family, or authorize D9+ implementation work.

## WorldCommandExecutionBroker

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldCommandExecutionBroker | existing world-service execution, guard, overlay/full-isolation/Landlock/network primitives; no UAA per-operation broker; Codex gateway currently enables external-sandbox bypass | `MissingSeam` | no | no | no | Interpose on every world-UAA shell/edit/write/MCP/tool/process/network side effect; execute under the envelope's `PolicySnapshotV3`; disable or reject unbrokerable channels. | WorldRuntimeAdapterExecutionEnvelope; EffectivePolicyResolver; RuntimeFamilyRealizationAdapter; world-service enforcement |

## RuntimeFamilyRealizationAdapter

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| RuntimeFamilyRealizationAdapter | `crates/gateway/src/adapter_runtime.rs`; UAA/client construction in shell and world-service; provider-specific session/output handling; managed gateway already accepts one-time FD auth independently of the direct member path | `UsefulFootholdButWrongBoundary` | no | not applicable | no | Keep only provider mechanics; consume Substrate-owned envelope/config/policy/receipt plus the existing in-world gateway endpoint/session inputs. Point Codex/provider traffic at that gateway; never receive raw host credentials or read the secure handoff FD. Remove lifecycle, binding, sandbox-authority, and credential-authority decisions from family adapters; copied credentials remain named/logged compatibility only. | WorldRuntimeAdapterExecutionEnvelope; AgentConfigProjectionService; WorldCommandExecutionBroker; RetainedWorkerRuntime |
