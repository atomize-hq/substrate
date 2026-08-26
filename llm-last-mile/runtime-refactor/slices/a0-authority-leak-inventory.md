**Kind:** slice row
**Stable ID:** `a0-authority-leak-inventory`
**Canonical for:** extracted A0 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 129
**Supersedes:** canonical ownership of the extracted `A0 — Authority leak inventory` row
**Superseded by:** none
**Projection consumers:** [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# A0 — Authority leak inventory

> **Authority boundary:** This file owns only the extracted A0 Track A row. It preserves the exact row text below, does not authorize diagnostics beyond the row scope, and does not dispatch A0.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A0 — Authority leak inventory** | Produce a repo-grounded inventory of every process/socket/helper/owner/cwd/env value currently used in durable decisions. | `00`; `01` invariants 1–3; `02` A0 inventory contract plus HostSessionAuthority, StateStore, and SurfaceAdapter rows; helper walkthrough/debug docs | HostSessionAuthority; HostExecutionEpisode; StateStore; CompatibilityReadModel | `agent_runtime/control.rs`; `agent_runtime/state_store.rs`; `execution/agents_cmd.rs`; `execution/orchestrator_world_dispatch.rs`; `repl/async_repl.rs`; UAA launch/member-runtime paths | No behavior changes except diagnostics/tests. No authority facade. No seventh control-pack file. | A committed inventory table in `02-seam-crosswalk.md` classifies every usage as `SignalOnly`, `FastPathTransport`, `AuthorityDecision`, or `CompatibilityRead`, and records proposed owner seam, first migration target, and proof gate. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01` |
