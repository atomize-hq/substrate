**Kind:** slice row
**Stable ID:** `d2-world-command-execution-broker`
**Canonical for:** extracted D2 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 213
**Supersedes:** canonical ownership of the extracted `D2 — WorldCommandExecutionBroker` row
**Superseded by:** none
**Projection consumers:** [`track-d-uaa-execution-envelope-and-side-effect-mediation.md`](track-d-uaa-execution-envelope-and-side-effect-mediation.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# D2 — WorldCommandExecutionBroker

> **Authority boundary:** This file owns only the extracted D2 Track D row. It preserves the exact row text below, does not broaden mediation claims beyond the row scope, and does not dispatch D2.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **D2 — WorldCommandExecutionBroker** | Mediate every world-UAA side-effect intent through the accepted `PolicySnapshotV3` and existing world-service enforcement path. | `02` broker row; `04` envelope and snapshot acceptance; `01` invariants 8–11 | EffectivePolicyResolver; runtime-family adapters; world-service enforcement | `crates/gateway`; relevant `agent-api-*`; `world-service` execute/guard/fs/network paths; bounded shell adapter glue | No parallel sandbox; no env-only enforcement; no shell-only claim while edit/MCP/write channels bypass. | Shell, apply/edit, direct write, write-capable tool/MCP, process, and network channels are brokered or disabled; snapshot mismatch/unbrokered intent fails closed. | `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-02`; D2 broker clause of `RG-OBS-01` |
