**Kind:** seam family
**Stable ID:** `configuration-and-gateway-adoption-family`
**Canonical for:** configuration and gateway adoption seam extraction
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** D8 configuration and gateway adoption family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `AgentConfigProjectionService` and `WorldRuntimeAdapterExecutionEnvelope` rows
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Configuration and gateway adoption seam family

> **Authority boundary:** This file owns only the extracted `AgentConfigProjectionService` and `WorldRuntimeAdapterExecutionEnvelope` seam rows. It does not promote any seam, move the A0 authority-leak inventory, reopen D5/D6/D7 or already-extracted D8 family owners, absorb the separate UAA/provider realization and side-effect mediation family, or authorize D9+ implementation work.

## AgentConfigProjectionService

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| AgentConfigProjectionService | placement-aware inventory in `execution/agent_inventory.rs`; host seed-home auth copy plus bounded `config.toml` subset in `world-service/member_runtime.rs`; Codex home helpers in `crates/codex`; separate landed managed-gateway carrier in `crates/common/src/gateway_auth_bundle.rs`, `crates/world-service/src/gateway_runtime.rs`, and `crates/gateway/src/server/mod.rs` | `DefensiveScaffoldingOnly` | no | no | no | Point direct world Codex at the already-landed in-world gateway carrier and replace copied host auth/config authority with per-worker Substrate-owned projection. Keep `CODEX_HOME`, `.codex`, `config.toml`, `.mcp.json`, provider endpoint, and runtime-native config as rebuildable non-secret projections from logical config plus accepted policy. Any copied credential path remains an explicitly named/logged compatibility mode with retirement criteria and is barred from `ContractCorrectAndProven`. | WorldRuntimeAdapterExecutionEnvelope; RuntimeFamilyRealizationAdapter; EffectivePolicyResolver; RetainedWorkerRuntime; WorldCommandExecutionBroker |

## WorldRuntimeAdapterExecutionEnvelope

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldRuntimeAdapterExecutionEnvelope | world Codex guest-entrypoint validation in `agent_runtime/validator.rs`; placement config in `config/agents/codex.yaml`; launcher/env/cwd plus copied seed-home setup in `world-service/member_runtime.rs`; reusable managed-gateway FD carrier and one-time consumer | `DefensiveScaffoldingOnly` | no | no | no | Persist one envelope bound to world generation, retained worker, Substrate-owned config projection, immutable policy snapshot, runtime deps, command-broker posture, and credential posture. Reuse the existing secure-FD carrier, join its non-secret evidence to the exact gateway receiver, and give Codex the gateway endpoint/session contract instead of copied credentials. Fail closed or use named non-promotable compatibility mode when exact adoption is unavailable. | AgentConfigProjectionService; RuntimeFamilyRealizationAdapter; WorldCommandExecutionBroker; RetainedWorkerRuntime |
