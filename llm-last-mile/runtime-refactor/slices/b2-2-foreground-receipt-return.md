**Kind:** slice row
**Stable ID:** `b2-2-foreground-receipt-return`
**Canonical for:** extracted B2.2 slice row and the current `E2-RM` admission correction
**Status:** canonical slice row record; B2.2 blocked on specified-but-unadmitted `E2-RM`
**Authority scope:** exact extracted source table header and row plus the documentation-only admission correction below; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 177
**Supersedes:** canonical ownership of the extracted `B2.2 — Foreground receipt return` row
**Superseded by:** none
**Projection consumers:** [`track-b-world-dispatch-receipts-supervision-and-cancel.md`](track-b-world-dispatch-receipts-supervision-and-cancel.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# B2.2 — Foreground receipt return

> **Authority boundary:** This file owns only the extracted B2.2 Track B row and its current
> admission correction. It preserves the exact row text below, does not reopen the completed
> B1/B2.1 or E2 implementation, and does not admit or dispatch `E2-RM` or B2.2.

## Current admission disposition

B2.2 is not admissible from the completed E2 closure alone. The existing authenticated E2 lookup
requires an already-known `DispatchPolicyCommitmentRefV1` and does not expose the complete stored
material needed to reproduce an original foreground receipt after response loss, restart, B2.1
observer/claim revision advancement, or parent-policy drift.

The new prerequisite is [`E2-RM — authenticated accepted-work receipt-material
projection`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite).
It is an E2-owned, read-only, behavior-neutral lookup/projection and is not B2.2 behavior. This
documentation correction specifies but does not admit, dispatch, implement, or complete it.

B1 continues to own acceptance and runtime-acknowledgement truth; B2.1 continues to own observation,
journal, replay, and terminal truth; E2 continues to own immutable policy commitments and retained
caps. B2.2 alone later owns foreground receipt construction and return from the authenticated
`E2-RM` result. B3.2 retains all remaining receipt, messaging, manifest, and lifecycle work; B4
retains receipt-targeted cancel/inspect/stop; and D1/E3 retain their later execution-envelope and
config-projection fields. C2/C3 and all other authority remain outside this correction.

B2.2 requires a fresh admission and explicit dispatch only after `E2-RM` has its own fresh
admission, bounded implementation, Linux acceptance evidence, independent final `CLEAN` review,
commit, and exact commit/tree identity. The B2.2 re-admission must consume
`AuthenticatedAcceptedWorkReceiptMaterialV1` without mutation or reinterpretation, prove that the
same original material is used after response loss/restart/B2.1 advancement/parent drift, and
re-establish B2.2's own file and behavior fence against the then-live baseline. Landing `E2-RM`
does not itself admit B2.2.

## Preserved pre-correction row (chronology only)

The current disposition above supersedes only this row's admission and prerequisite interpretation;
the extracted header and row remain byte-identical chronology.

| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
