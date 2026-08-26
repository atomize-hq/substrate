**Kind:** slice row
**Stable ID:** `e3-agent-config-projection-and-gateway-adoption`
**Canonical for:** extracted E3 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 222
**Supersedes:** canonical ownership of the extracted `E3 — AgentConfigProjectionService and gateway adoption` row
**Superseded by:** none
**Projection consumers:** [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# E3 — AgentConfigProjectionService and gateway adoption

> **Authority boundary:** This file owns only the extracted E3 Track E row. It preserves the exact row text below, keeps copied-credential compatibility non-promotable, and does not dispatch E3.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E3 — AgentConfigProjectionService and gateway adoption** | Make logical/effective/native config projection first-class per retained worker and point world Codex at the existing in-world gateway secure-FD credential/session boundary. | `02` config projection + envelope + realization rows; `04` envelope projection, existing-carrier adoption note, and `LaunchTimeSecretHandoffV1`; existing gateway auth-handoff internals/verification docs; config projection, Codex mapping, and workspace overlay designs | Envelope; retained manifest; EffectivePolicyResolver; in-world gateway; RuntimeFamilyRealizationAdapter | `execution/agent_inventory.rs`; `crates/codex`; `world-service/member_runtime.rs`; existing in-world gateway endpoint/session integration points; bounded projection module; projection tests | No reimplementation of the landed FD carrier; no ambient runtime file as authority; no secret payload in projected files, manifests, traces, or UAA child FDs; no workspace-sync policy decision. | Sibling workers get isolated non-secret projections; `CODEX_HOME`, `config.toml`, and provider wiring can be rebuilt from Substrate truth plus accepted policy; Codex uses the exact managed gateway whose credentials arrived through the existing one-time FD carrier; copied auth/config runs only as named/logged/non-promotable compatibility; narrowed hints never substitute for enforcement. | `RG-CONFIG-01`, `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`; E3 credential-handoff clause of `RG-OBS-01` |
