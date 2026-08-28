**Kind:** seam family
**Stable ID:** `dispatch-and-episode-transport-family`
**Canonical for:** dispatch and episode transport seam extraction
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** D8 dispatch and episode transport family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `InternalToolboxTransport`, `RuntimeToolInvocationAdapter`, and `WorldDispatchControl` rows
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Dispatch and episode transport seam extraction

> **Authority boundary:** This file owns only the extracted `InternalToolboxTransport`, `RuntimeToolInvocationAdapter`, and `WorldDispatchControl` seam rows. It does not promote any seam, move the A0 authority-leak inventory, reopen D5/D6/D7 or B1/B2.1/B3.1/C1 family owners, extract `SteeringPolicyEngine`, `EffectivePolicyResolver`, or `DispatchPolicyNarrowingPatch`, or authorize B2.2/B3.2/B4/E-track implementation work.

## InternalToolboxTransport

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| InternalToolboxTransport | `toolbox_transport_path` and endpoint registration in `agent_runtime/control.rs`; toolbox owner lifecycle in `repl/async_repl.rs`; internal endpoint integration coverage in `repl_world_first_routing_v1.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | Preserve internal session-scoped transport, but bind requests through runtime-injected exact authority and make transport availability independent from durable session/work truth. | RuntimeToolInvocationAdapter; HostSessionAuthority; WorldDispatchControl; SurfaceAdapter |

## RuntimeToolInvocationAdapter

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| RuntimeToolInvocationAdapter | model contract/types and runtime injection in `agent_runtime/tool_invocation_contract.rs`; prompt composition in `execution/prompt_fulfillment.rs`; toolbox env construction in `agent_runtime/control.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | Keep model-visible validation and hidden internal identity; change long-running outputs from terminal-shaped outcomes to accepted receipts; route only through the transport/dispatch authority chain. | InternalToolboxTransport; WorldDispatchControl; WorldWorkReceiptRegistry; SteeringPolicyEngine |

## WorldDispatchControl

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldDispatchControl | typed request/outcome contracts in `agent_runtime/dispatch_contract.rs`; orchestration in `execution/orchestrator_world_dispatch.rs`; exact-target state-store resolvers | `MislandedWrongModel` | no | no | no | Turn the file-level orchestration into a facade over HostSessionAuthority, policy resolution, durable receipts, supervisor, messaging, and retained runtime. Before the joint closeout, A1.2a-S carries production-bound authority into the toolbox and B3.2a routes both direct-dispatch and live internal-toolbox `SpawnWorldWorker` adapters through R0 exact authority, an atomic canonical-fingerprint/cap slot, one serialized per-session registration head acquired only by exact lowest-slot request re-presentation after any prior head is separately reconciled and released, and a typed shell-to-world-service launch proof while preserving existing Spawn policy/outcome. B3.2a-WA then strict-validates the authority-managed `Some(exact proof)` and durably exact-adopts the same HSA-bound world for physical shared-session ownership without changing its ID/generation or proving member launch. B1/B2.1-0 is the first branch join: it accepts current-authority, canonical retained-target, B3.2a admission/routability truth, B3.2a-WA physical ownership truth, receipt, and supervisor truth and partitions prepared state by action/payload. Its B-owned view covers RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait without the legacy live-retained count. `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, and fork remain on unchanged compatibility paths, unpromoted, until the remaining RetainedWorkerRuntime/B4 work supplies canonical lifecycle/closeout truth. B4 owns the exact user/tool-facing inspection and cancellation surface for pending Spawn admission, consuming rather than authoring RetainedWorkerRuntime's durable admission transitions and reporting the frozen distinct outcomes. Neither adapter can issue/apply a transition, synthesize authority, or supply transition correlation. Preserve blocking compatibility; B4 retains final cancel outcomes. | HostSessionAuthority; RuntimeToolInvocationAdapter; SteeringPolicyEngine; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; RetainedWorkerRuntime |
