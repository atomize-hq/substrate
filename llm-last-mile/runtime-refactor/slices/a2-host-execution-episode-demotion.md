**Kind:** slice row
**Stable ID:** `a2-host-execution-episode-demotion`
**Canonical for:** extracted A2 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 131
**Supersedes:** canonical ownership of the extracted `A2 — HostExecutionEpisode demotion` row
**Superseded by:** none
**Projection consumers:** [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# A2 — HostExecutionEpisode demotion

> **Authority boundary:** This file owns only the extracted A2 Track A row. It preserves the exact row text below, does not authorize episode-demotion implementation, and does not dispatch A2.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A2 — HostExecutionEpisode demotion** | Represent REPL/helper/toolbox/recovered processes as episodes whose PID/socket/readiness data are observations only. | `01` invariants 1–2; `02` SurfaceAdapter row; `04` HostExecutionEpisodeV1; `DESIGN-internal-toolbox-transport-and-session-binding.md` | HostSessionAuthority; InternalToolboxTransport; RouterAttachTrigger | `agent_runtime/control.rs`; `execution/agents_cmd.rs`; `repl/async_repl.rs`; episode-focused tests | No deletion of `__owner-helper`; no transport protocol rewrite; no auto-attach redesign. | Killing/orphaning an episode leaves durable session truth intact; stale episode updates are revision-rejected; transport classification is explicit. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01` |
