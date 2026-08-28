**Kind:** slice row
**Stable ID:** `b4-receipt-targeted-cancel-inspect-stop`
**Canonical for:** extracted B4 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 180
**Supersedes:** canonical ownership of the extracted `B4 — Receipt-targeted cancel/inspect/stop` row
**Superseded by:** none
**Projection consumers:** [`track-b-world-dispatch-receipts-supervision-and-cancel.md`](track-b-world-dispatch-receipts-supervision-and-cancel.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# B4 — Receipt-targeted cancel/inspect/stop

> **Authority boundary:** This file owns only the extracted B4 Track B row. It preserves the exact row text below, does not widen C-track or unrelated lifecycle authority, and does not dispatch B4.

| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B4 — Receipt-targeted cancel/inspect/stop** | Resolve control against active receipts and worker manifests with distinct durable outcomes, and expose exact inspection and cancellation of pending Spawn admission through WorldDispatchControl without taking ownership of RetainedWorkerRuntime state transitions. | WorldDispatchControl owns the user/tool-facing verb and resolves exact targets; ReceiptRegistry owns immutable accepted identity; Supervisor owns active observation and terminal truth; RetainedWorkerRuntime owns worker stop lifecycle and durable admission-state resolution. | `04` cancel categories, deferred admission-recovery contract, and supervisor cancellation rules; `02` WorldDispatchControl/ReceiptRegistry/RetainedRuntime rows; `05` cancel/closeout rows and `RG-ADMISSION-01`. | HostSessionAuthority; Supervisor; private transport episode status. | `orchestrator_world_dispatch.rs`; `agent_runtime/{state_store,dispatch_contract,control}.rs`; world-service cancel seam; control tests. | No weakening exact identity, no treating no active run as stale linkage, no liveness-based admission resolution, and no stop/cancel/admission-abandonment conflation. | Same-turn continue→cancel targets active run. Pending Spawn inspection/cancellation exact-joins the admission identity and returns distinct outcomes for cancelled-before-registration, registered-before-transport, transport-ambiguous/cancel-pending, already routable/terminal, invalid target, ambiguity, and policy denial. Repeated terminal resolution is idempotent; no-active, terminal, unreachable, invalid, mismatch, ambiguity, and policy denial for active work remain distinct. | `RG-CANCEL-01`, `RG-CANCEL-02`, `RG-CLOSE-01`, `RG-BASE-04`, and the user/tool-facing clauses of `RG-ADMISSION-01`; B4 cancel clause of `RG-OBS-01`. | B2.2 and B3.2. | C2/C3 and unrelated lifecycle/policy work. |
