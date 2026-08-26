**Kind:** slice row
**Stable ID:** `b2-2-foreground-receipt-return`
**Canonical for:** extracted B2.2 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row only; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 177
**Supersedes:** canonical ownership of the extracted `B2.2 — Foreground receipt return` row
**Superseded by:** none
**Projection consumers:** [`track-b-world-dispatch-receipts-supervision-and-cancel.md`](track-b-world-dispatch-receipts-supervision-and-cancel.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# B2.2 — Foreground receipt return

> **Authority boundary:** This file owns only the extracted B2.2 Track B row. It preserves the exact row text below, does not reopen the completed B1/B2.1 family, and does not dispatch B2.2.

| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
