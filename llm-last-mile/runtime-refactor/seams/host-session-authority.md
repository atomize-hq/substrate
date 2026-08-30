**Kind:** seam family
**Stable ID:** `host-session-authority-family`
**Canonical for:** host/session authority family seam extraction and related owner navigation
**Status:** canonical current seam-family record with preserved packet-era row
**Authority scope:** exact extracted family-local source body, the preserved related-owner link, and the A1.3-P1 closure-status overlay only; no implementation or successor authority
**Source span:** D8 host/session authority family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `SurfaceAdapter / HostExecutionEpisode` row; the related `HostSessionAuthority` row remains canonical under `a1-2-earlier-histories/`
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Host/session authority seam family

> **Authority boundary:** This file owns only the extracted `SurfaceAdapter / HostExecutionEpisode` seam row and the preserved related-owner link for the already-extracted `HostSessionAuthority` family row. It does not promote either seam, move the A0 authority-leak inventory, reopen `a1-2-earlier-histories/`, or authorize A1/A2/A3 implementation work.

## A1.3-P1 closure-status overlay (2026-08-29; current)

The extracted seam row below remains byte-stable; its reference to the active A1.3-P1 packet is
packet-era scope wording, not current scheduling status. A1.3-P1 is terminally complete, while the
row's `MissingSeam` classification remains unchanged and unpromoted. A1.4 is only the named
successor awaiting fresh admission and explicit dispatch and receives no authority here.

## Related canonical owner

`HostSessionAuthority` already has its canonical family row under [`../a1-2-earlier-histories/crosswalk.md#hostsessionauthority-family-row`](../a1-2-earlier-histories/crosswalk.md#hostsessionauthority-family-row). This D8 landing preserves that D6 owner unchanged.

## SurfaceAdapter / HostExecutionEpisode

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| SurfaceAdapter / HostExecutionEpisode | `crates/shell/src/execution/agents_cmd.rs`; hidden-helper launch/transport code in `agent_runtime/control.rs`; live runtime ownership in `repl/async_repl.rs` | `MissingSeam` | no | not applicable | no | Introduce a generic episode identity/status boundary; make REPL, CLI helper, toolbox, and recovered episodes report PID, helper-process, active-handle, readiness, and prompt-stream observations through it. Episode construction and transport state cannot create successor authority or reset a parked session to `Allocating`; launch follows durable `Attach`/`ResumeOneTurn` application. For B2.1-3 only, current `run_async_repl` may invoke one canonical `WorldWorkExecutionSupervisor` recovery entry point and retain its returned observation tasks; it cannot inspect records, reproduce restart logic, or interpret unresolved or terminal state. That bounded activation hook is not complete ingress-surface neutrality and does not promote this row or the supervisor row. the active A1.3-P1 packet's bounded adapter may transport only the exact actor-bound startup protocol event; A1.2b's HostSessionAuthority CAS alone constructs `HostStartupOwnershipEvidenceV1` and commits the result. Timeout, EOF, helper/PID/socket/handle/readiness loss remains ambiguous and cannot construct terminal evidence. | HostSessionAuthority; StateStore; InternalToolboxTransport; RouterAttachTrigger; WorldWorkExecutionSupervisor |
