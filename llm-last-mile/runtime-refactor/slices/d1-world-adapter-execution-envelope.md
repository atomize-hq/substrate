**Kind:** slice row
**Stable ID:** `d1-world-adapter-execution-envelope`
**Canonical for:** extracted D1 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 212
**Supersedes:** canonical ownership of the extracted `D1 — World adapter execution envelope` row
**Superseded by:** none
**Projection consumers:** [`track-d-uaa-execution-envelope-and-side-effect-mediation.md`](track-d-uaa-execution-envelope-and-side-effect-mediation.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# D1 — World adapter execution envelope

> **Authority boundary:** This file owns only the extracted D1 Track D row. It preserves the exact row text below, keeps the fail-closed staging rule outside the row text, and does not dispatch D1.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **D1 — World adapter execution envelope** | Materialize and persist `WorldRuntimeAdapterExecutionEnvelopeV1` with guest entrypoint, projection identity, world binding, policy snapshot, broker requirement, credential posture, secret-handoff ref, and explicit mediation/compatibility posture. | `02` envelope + realization rows; `04` envelope and `LaunchTimeSecretHandoffV1` contracts; existing gateway auth-handoff internals/verification docs; Codex home/auth mapping design; config projection design | AgentConfigProjectionService; EffectivePolicyResolver; RetainedWorkerManifest; in-world Substrate gateway | `agent_runtime/{validator,dispatch_contract}.rs`; `execution/agent_inventory.rs`; `world-service/member_runtime.rs`; existing managed gateway launch/handoff types; `config/agents/*`; envelope tests | No per-operation broker yet; no host runtime behavior change; no reimplementation of the existing FD carrier; no unnamed credential fallback; no compatibility evidence counted as contract proof. | Host/world envelopes are distinct; world Codex cannot use host path; sibling workers cannot alias envelope/projection identity. Every world envelope records `SecureGatewayHandoff`, `CompatibilityCopyBridge`, or `NoCredentialsRequired`; secure posture joins the existing carrier to the exact gateway receiver and non-secret evidence; Codex receives that gateway's endpoint/session contract; compatibility copy is named/logged/non-promotable; credential-requiring mediated mode fails closed without exact adoption. | `RG-UAA-01`, `RG-UAA-02`, `RG-CONFIG-01`, `RG-CONFIG-03`, `RG-CONFIG-04` |
