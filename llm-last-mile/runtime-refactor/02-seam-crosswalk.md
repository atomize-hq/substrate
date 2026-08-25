# Seam Crosswalk

## Reading rule

Canonical content: [`seams/README.md#reading-rule`](seams/README.md#reading-rule).

The repeated seam-table headers below are projections of the canonical shared column definitions in [`seams/README.md#shared-crosswalk-table-columns`](seams/README.md#shared-crosswalk-table-columns).

## A1.1d-5 private-home prerequisite crosswalk

Canonical content: [`a1-2-earlier-histories/crosswalk.md#a11d-5-private-home-prerequisite-crosswalk`](a1-2-earlier-histories/crosswalk.md#a11d-5-private-home-prerequisite-crosswalk).

## A1.1d-5R2 canonical prefix-propagation inventory

Canonical content: [`a1-2-earlier-histories/crosswalk.md#a11d-5r2-canonical-prefix-propagation-inventory`](a1-2-earlier-histories/crosswalk.md#a11d-5r2-canonical-prefix-propagation-inventory).

### F0 test-proof prerequisite crosswalk

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#f0-test-proof-prerequisite-crosswalk`](a1.1d-5r2-2f/crosswalk.md#f0-test-proof-prerequisite-crosswalk).

### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f0a--substrate_home-test-isolation`](a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f0a--substrate_home-test-isolation).

### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f0b--deterministic-renderer-output-test-isolation`](a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f0b--deterministic-renderer-output-test-isolation).

## A0 authority-leak inventory contract

Canonical content: [`a1-2-earlier-histories/crosswalk.md#a0-authority-leak-inventory-contract`](a1-2-earlier-histories/crosswalk.md#a0-authority-leak-inventory-contract).

## A. Host authority and ingress

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| [SurfaceAdapter / HostExecutionEpisode](seams/host-session-authority.md#surfaceadapter--hostexecutionepisode) | `crates/shell/src/execution/agents_cmd.rs`; hidden-helper launch/transport code in `agent_runtime/control.rs`; live runtime ownership in `repl/async_repl.rs` | `MissingSeam` | no | not applicable | no | Introduce a generic episode identity/status boundary; make REPL, CLI helper, toolbox, and recovered episodes report PID, helper-process, active-handle, readiness, and prompt-stream observations through it. Episode construction and transport state cannot create successor authority or reset a parked session to `Allocating`; launch follows durable `Attach`/`ResumeOneTurn` application. For B2.1-3 only, current `run_async_repl` may invoke one canonical `WorldWorkExecutionSupervisor` recovery entry point and retain its returned observation tasks; it cannot inspect records, reproduce restart logic, or interpret unresolved or terminal state. That bounded activation hook is not complete ingress-surface neutrality and does not promote this row or the supervisor row. the active A1.3-P1 packet's bounded adapter may transport only the exact actor-bound startup protocol event; A1.2b's HostSessionAuthority CAS alone constructs `HostStartupOwnershipEvidenceV1` and commits the result. Timeout, EOF, helper/PID/socket/handle/readiness loss remains ambiguous and cannot construct terminal evidence. | HostSessionAuthority; StateStore; InternalToolboxTransport; RouterAttachTrigger; WorldWorkExecutionSupervisor |
| HostSessionAuthority | [Canonical A1.2/earlier-histories family row](a1-2-earlier-histories/crosswalk.md#hostsessionauthority-family-row) | canonical family row moved | — | — | — | See canonical family row. | SurfaceAdapter; StateStore; CompatibilityReadModel; WorldDispatchControl; ObligationLedger |
| StateStore | `AgentRuntimeStateStore`, session/participant persistence, atomic JSON writes, active-task records, obligation/inbox persistence in `agent_runtime/state_store.rs` | `UsefulFootholdButWrongBoundary` | no | not applicable | no | A1.1 retains and hardens only persisted physical-home bootstrap classification, operation-bound temp reconciliation, cross-process atomic persistence, immutable object/key storage, immutable greenfield certification, complete pre-A1 collection enumeration/rejection, and revision-CAS. Under A1.1d every in-repository transaction that reads or mutates either pre-A1 authority collection—including flat/canonical snapshots, leases, removals, compatibility/read-repair persistence, and parent-session persistence triggered by another operation—must perform its complete read, decision, write, rename, removal, and final `fsync` through the same retained opened physical-root transaction. The lock protects that opened root only: rename, replacement, rebind, or identity uncertainty fails closed and leaves the replacement tree untouched. A1.1e exposes accepted-home operations through a distinct bound capability with no lexical descendant-path API; replacement, rebind, or identity uncertainty fails before a bound read or write can consume the replacement tree. Stale lifecycle and world-binding rejection is correct and remains fail-closed. StateStore persists the transition selected and validated by HostSessionAuthority and may persist an exact policy/snapshot identity; it must not parse, compose, finalize, choose, reconstruct, or synthesize effective policy or a successor lifecycle transition. For B1/B2.1 it may supply only separately scoped opaque physical capabilities for the receipt registry and supervisor journal; it cannot become their semantic owner or a generic activated-store writer. No converter, dual-write path, unrelated StateStore redesign, or general persistence extraction is added. A1.1d remains incomplete. | HostSessionAuthority; CompatibilityReadModel; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; ObligationLedger |
| CompatibilityReadModel | torn-root fallback, synthesized session records, legacy inbox projection, read repair, status-visible participant logic in `agent_runtime/state_store.rs` | `UsefulFootholdButWrongBoundary` | no | not applicable | no | A1.1 does not consume compatibility state as authority: any pre-A1 session/participant artifact is `UnsupportedLegacyState`, and unreadable/uncertain collections fail closed. During A1.1d, any compatibility read that participates in a read-decide-write transaction and any compatibility/read-repair persistence into the guarded collections is bound to the same retained opened physical-root transaction; it cannot escape through a path-based helper or treat root replacement as authorization. A1.1d does not extract or promote this seam. A3 still owns the later persistence/compatibility separation and read-only projection/diagnostics; no missing/unreadable projection may imply `ExpectedAbsent` or override newer authority. | StateStore; HostSessionAuthority; InboxProjection |
| InternalToolboxTransport | `toolbox_transport_path` and endpoint registration in `agent_runtime/control.rs`; toolbox owner lifecycle in `repl/async_repl.rs`; internal endpoint integration coverage in `repl_world_first_routing_v1.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | Preserve internal session-scoped transport, but bind requests through runtime-injected exact authority and make transport availability independent from durable session/work truth. | RuntimeToolInvocationAdapter; HostSessionAuthority; WorldDispatchControl; SurfaceAdapter |
| RuntimeToolInvocationAdapter | model contract/types and runtime injection in `agent_runtime/tool_invocation_contract.rs`; prompt composition in `execution/prompt_fulfillment.rs`; toolbox env construction in `agent_runtime/control.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | Keep model-visible validation and hidden internal identity; change long-running outputs from terminal-shaped outcomes to accepted receipts; route only through the transport/dispatch authority chain. | InternalToolboxTransport; WorldDispatchControl; WorldWorkReceiptRegistry; SteeringPolicyEngine |

## B. Dispatch, policy, receipts, and retained runtime

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldDispatchControl | typed request/outcome contracts in `agent_runtime/dispatch_contract.rs`; orchestration in `execution/orchestrator_world_dispatch.rs`; exact-target state-store resolvers | `MislandedWrongModel` | no | no | no | Turn the file-level orchestration into a facade over HostSessionAuthority, policy resolution, durable receipts, supervisor, messaging, and retained runtime. Before the joint closeout, A1.2a-S carries production-bound authority into the toolbox and B3.2a routes both direct-dispatch and live internal-toolbox `SpawnWorldWorker` adapters through R0 exact authority, an atomic canonical-fingerprint/cap slot, one serialized per-session registration head acquired only by exact lowest-slot request re-presentation after any prior head is separately reconciled and released, and a typed shell-to-world-service launch proof while preserving existing Spawn policy/outcome. B3.2a-WA then strict-validates the authority-managed `Some(exact proof)` and durably exact-adopts the same HSA-bound world for physical shared-session ownership without changing its ID/generation or proving member launch. B1/B2.1-0 is the first branch join: it accepts current-authority, canonical retained-target, B3.2a admission/routability truth, B3.2a-WA physical ownership truth, receipt, and supervisor truth and partitions prepared state by action/payload. Its B-owned view covers RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait without the legacy live-retained count. `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, and fork remain on unchanged compatibility paths, unpromoted, until the remaining RetainedWorkerRuntime/B4 work supplies canonical lifecycle/closeout truth. B4 owns the exact user/tool-facing inspection and cancellation surface for pending Spawn admission, consuming rather than authoring RetainedWorkerRuntime's durable admission transitions and reporting the frozen distinct outcomes. Neither adapter can issue/apply a transition, synthesize authority, or supply transition correlation. Preserve blocking compatibility; B4 retains final cancel outcomes. | HostSessionAuthority; RuntimeToolInvocationAdapter; SteeringPolicyEngine; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; RetainedWorkerRuntime |
| SteeringPolicyEngine | `agents.world_dispatch.*` fields in `crates/broker`; `enforce_world_dispatch_steering_policy` and event/payload checks in `orchestrator_world_dispatch.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | Consolidate deny-by-default action/mode/backend/session/world/event/autonomy decisions into one explanation-ready boundary used by every ingress path. | WorldDispatchControl; EffectivePolicyResolver; ObligationLedger; WorldWorkerMessagingProtocol |
| EffectivePolicyResolver | `crates/broker/src/effective_policy.rs`; `execution/policy_model.rs`; `execution/policy_snapshot.rs`; current policy composition in dispatch code | `UsefulFootholdButWrongBoundary` | no | no | no | The broker remains the sole owner of effective-policy parsing, default/global/workspace layering, validation, finalization, and explanation provenance. A1.1e adds one bounded broker entry point that accepts an explicit global-policy source derived by shell code from the already-accepted bootstrap home and reuses the canonical broker pipeline; shell code only supplies that input and consumes the result. Differential parity is focused-proof clean, but no real dispatch path or enforcement point adopts this entry point yet. Later E-track parent/cap/turn composition, dispatch narrowing, immutable acceptance, and enforcement remain unresolved: add the dispatch acceptance API there without treating the A1.1e entry point as closure or promoting this seam. | SteeringPolicyEngine; DispatchPolicyNarrowingPatch; WorldWorkReceiptRegistry; WorldCommandExecutionBroker |
| DispatchPolicyNarrowingPatch | `allow_capability_narrowing` policy flag; boolean capability overrides in `dispatch_contract.rs`; inventory-time `policy_overlay.world_fs` validation in `agent_inventory.rs` | `MissingSeam` | no | no | no | Add request-scoped restricted `PolicyPatch.world_fs`; implement path-containment monotonicity; persist narrowing reason and resulting snapshot on task/turn/worker records. | EffectivePolicyResolver; WorldWorkReceiptRegistry; RetainedWorkerRuntime; AgentConfigProjectionService |
| RuntimeEventTransport | Canonical `RuntimeFrameIdentityV1`, `RuntimeEventIdentityV1`, and `RuntimeTerminalIdentityV1` in `crates/common/src/agent_events.rs`, re-exported without aliases by `crates/transport-api-types/src/lib.rs`; identified `ExecuteStreamFrame`; `RuntimeEventStreamProducer` plus ordinary task-stream sinks in `crates/world-service/src/service.rs`; retained launch/turn producers in `crates/world-service/src/member_runtime.rs`; carrier-only typed decoders in the named shell consumers; bounded process-memory replay in `crates/world-service/src/runtime_replay.rs` plus its exact handler/route and transport client types | `MissingSeam` **with B0 producer carrier and B2.1 replay transport recovered; full seam unpromoted** | producer/replay transport yes; full seam no | producer and recovered supervisor consumer core yes; accepted-family production path yes | B0 and B2.1 component/restart proof yes; accepted-family production proof yes; full seam no | B0 landed one non-empty UUIDv7 `stream_id` per accepted producer stream, `schema_version = 1`, one frame counter shared by Start/stdout/stderr/Event/Exit/Error, one semantic counter for Event/Exit, stable UUIDv7 event IDs, exact Exit/terminal identity equality, producer closure after terminal/error, and canonical NDJSON clone/re-emission byte stability. The stdout/stderr sink holds the producer lock through enqueue, so assignment order is emission order. Ordinary task, retained launch, and retained submitted-turn paths all construct identity before host observation. Shell compatibility decodes and forwards the carrier unchanged; missing/malformed identity fails, and Error/EOF/stream exhaustion cannot create terminal truth or `AlreadyTerminal`. Standalone legacy `AgentEvent` decoding may omit event identity, but an in-repository runtime `Event` may not. B2.1-3 adds only bounded process-memory producer retention and exact replay transport keyed by supplied acceptance-record ID, stream ID, and frame cursor; no enumeration or fuzzy/session-only lookup is allowed. Replay preserves exact B0 identity, order, and bytes and creates no acceptance, claim, lifecycle, terminal, obligation, or success truth. The recovered B2.1 supervisor owns duplicate/gap/reorder/conflict handling, durable consumer journal/replay acceptance, and restart observation; the producer registry remains process-memory world-service transport state, so this row is not promoted to `ContractCorrectAndProven`. B0 added no receipt, retained-message, obligation, lifecycle, or foreground-return semantics. The B1 receipt and B2.1 supervisor/replay cores plus their joint production integration closeout are review-clean at the bound 2026-08-03 source snapshot without additional product/test edits, and B3.1 is unblocked without promoting this seam. | WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; WorldWorkerMessagingProtocol; ObligationLedger |
| WorldWorkReceiptRegistry | recovered `world_work_receipt_registry.rs` immutable acceptance core and bounded HSA physical receipt capability; typed `WorldWorkAcceptanceContextV1`; remaining terminal-shaped `HostToolRunWorldTaskReceiptV1` compatibility surface | `MislandedWrongModel` | recovered core yes; full seam no | recovered core yes; joint production path open | component proof yes; full seam no | Recovered B1-3a/B1-3b provide durable accepted task and retained-turn acceptance records instead of temporary/terminal identity on the B-owned paths. ReceiptRegistry preallocates a proposed acceptance-record ID and sends it with exact request/message/caller-backend identity in `WorldWorkAcceptanceContextV1` on the existing typed task/turn request; world-service retains that context, but the proposal is not accepted truth until the exact B0 acknowledgement durably creates the matching immutable record. The record includes exact caller participant/backend, task/active-run ID, B0 stream identity/acceptance sequence, current policy identity, and optional intent/revision/opaque-payload-commitment/transition-run/authority correlation using the common equality-only types. Production correlation stays absent until A1.2b supplies it through the HostSessionAuthority-owned boundary; the registry retains and scope-checks it without authenticating, verifying, issuing, applying, or interpreting the transition. The receipt core is review-clean through `6436289f` without claiming B1 complete. B1 production completion waits for the joint B1/B2.1 closeout proving every accepted path enters the supervisor and never the legacy active-task writer. | RuntimeEventTransport; WorldDispatchControl; WorldWorkExecutionSupervisor; RetainedWorkerRuntime; EffectivePolicyResolver |
| WorldWorkExecutionSupervisor | recovered `execution/world_work_execution_supervisor.rs` durable claim/journal/recovery core plus bounded HSA physical capability; canonical recovery activation in `run_async_repl`; exact replay client; remaining blocking stream/waiter compatibility in `orchestrator_world_dispatch.rs` | `MissingSeam` **with B2.1 core recovered; accepted-family production closeout complete** | recovered core yes; full seam no | recovered core yes; joint production path yes | component/restart proof yes; accepted-family production proof yes; full seam no | Recovered B2.1-1 durably claims both task and retained-turn streams at exact B1 acceptance before any subsequent frame can bypass the supervisor and replaces legacy active-task registration on the accepted production path. Recovered B2.1-2 journals exact B0 identity and canonical bytes before waiter delivery, makes exact replay a no-op, rejects gaps/reorder/conflicts/stale observers/post-terminal frames, and preserves blocking compatibility over supervisor truth. Recovered B2.1-3 alone discovers every nonterminal claim and exact cursor, interprets durable state, and reconciles only through exact producer replay. A surviving world-service producer registry permits exact replay and live resumption after host/shell restart. A world-service restart or unavailable producer leaves the claim durably nonterminal and unresolved; it cannot fabricate terminal state, delete truth, advance the cursor, or complete an obligation cut. Caller/waiter drop does not delete claims or work; EOF, timeout, PID/helper/socket/process/readiness/caller/endpoint state, stream exhaustion, or runtime exit without the exact B0 terminal event cannot close a run or complete the cut. The recovered supervisor/replay cores are review-clean through `c519024b` and `de727091`, and the later joint B1/B2.1 production closeout is recorded at the bound 2026-08-03 source snapshot without additional product/test edits. B3.1 is now the next packet. | RuntimeEventTransport; WorldWorkReceiptRegistry; WorldDispatchControl; SurfaceAdapter / HostExecutionEpisode; WorldWorkerMessagingProtocol; ObligationLedger; RetainedWorkerRuntime |
| WorldWorkerMessagingProtocol | [Canonical B3.1/C1 family row](b3-1-c1/crosswalk.md#worldworkermessagingprotocol) | canonical family row moved | — | — | — | See canonical family row. | RuntimeEventTransport; WorldWorkExecutionSupervisor; RetainedWorkerRuntime; SteeringPolicyEngine; ObligationLedger |
| RetainedWorkerRuntime | R0 object construction/exact target resolution plus B3.2a admission key, registry, fingerprint, slot, registration-head, transport-claim, routability, and terminal truth in `agent_runtime/retained_worker_runtime.rs`; HSA-owned R0 reservation/application maps and bounded admission-storage capability in `agent_runtime/host_session_authority`; typed proof production/transport/validation in `orchestrator_world_dispatch.rs`, `async_repl.rs`, `transport-api-types`, and `world-service`; participant/session manifests in `agent_runtime/session.rs`; remaining compatibility lifecycle mutations across `async_repl.rs`, `state_store.rs`, and `orchestrator_world_dispatch.rs` | `MislandedWrongModel` | no | no | no | R0 provides only immutable descriptor/resume/worker object construction and exact target read, consuming the HSA-owned atomic lineage/ref registration proof without owning authority mutation or live state. It is review-clean component proof with no production ingress and does not promote this seam. B3.2a has now added only the fixed durable creation/admission record and both production Spawn bridges needed before the joint closeout: a RetainedWorkerRuntime-owned keyed commitment over the complete canonical request/authority/runtime/policy plan, crash-stable admission-key retention, atomic cap slot, stable participant/bootstrap run, and one serialized per-session registration head prevent over-admission, changed-byte retry, and stale-revision races without extending HSA commitment domains. The existing `SlotReserved` state may remain queued with no head; current-head reconciliation never promotes another slot, and only exact complete re-presentation of the earliest request can acquire the released head. No request/prompt/payload preimage is persisted. A transport-neutral equality proof crosses the shell transport-api request; the real `Service::execute_stream` member branch first completes strict `Some(exact proof)` validation, durably exact-adopts the HSA-bound world through B3.2a-WA, and only then passes the unchanged typed request into launch-boundary revalidation. Admission is nonterminal before transport, routable only on exact Registered truth, and terminal only on exact B0 terminal truth, with ambiguous interruption retained and counted live. Both activated paths avoid legacy session/participant writers and supply canonical live-count/routability reads. B3.2a plus B3.2a-WA are review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda` without promoting this seam. The remaining B3.2 packet later owns durable abandoned-admission resolution and restart reconciliation, including exact terminal handling of abandoned `SlotReserved`/`AuthorityRegistrationHead` state and reconciliation of post-R0 or transport-ambiguous state before cap release; it also owns full worker manifests, accepted-turn lifecycle, park, cancel, stop, fork, inspect, invalidation, and broader reconciliation. B4 owns only the exact user/tool-facing pending-admission inspect/cancel verb and outcomes. Accepted task/turn identity stays in WorldWorkReceiptRegistry and host posture stays separate. | HostSessionAuthority; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; WorldWorkerMessagingProtocol |

Recorded B3.1 result: canonical content moved to [`b3-1-c1/crosswalk.md#recorded-b31-result`](b3-1-c1/crosswalk.md#recorded-b31-result).

## C. Obligations and host re-engagement

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| ObligationLedger | [Canonical B3.1/C1 family row](b3-1-c1/crosswalk.md#obligationledger) | canonical family row moved | — | — | — | See canonical family row. | RuntimeEventTransport; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; WorldWorkerMessagingProtocol; InboxProjection; AutoAttachProjection; HostSessionAuthority |
| InboxProjection | `agent_runtime/host_inbox.rs`; compatibility durable inbox items in `state_store.rs`; `execution/host_inbox_materialization.rs` | `DefensiveScaffoldingOnly` | no | not applicable | no | Make inbox a read/projection model over canonical obligations; isolate compatibility ingress; prevent inbox rows or counts from becoming independent authority. | ObligationLedger; CompatibilityReadModel; AutoAttachProjection |
| AutoAttachProjection | attach fields on obligation records; eligibility/claim helpers in `agent_runtime/auto_attach.rs` | `DefensiveScaffoldingOnly` | no | no | no | Derive eligibility and claim state only from canonical obligations plus effective policy; make claims idempotent and session-coalesced. | ObligationLedger; InboxProjection; RouterAttachTrigger; SteeringPolicyEngine |
| RouterAttachTrigger | router discovery and helper launch in `agent_runtime/auto_attach.rs`; detached `spawn_blocking` trigger in `orchestrator_world_dispatch.rs` | `DefensiveScaffoldingOnly` | no | no | no | Move to an explicit durable trigger/claim/settle path; restore sanctioned host ownership once; record result; prohibit prompt replay and direct worker actions. | AutoAttachProjection; HostSessionAuthority; SurfaceAdapter; ObligationLedger |

## D. UAA realization, projection, and side-effect mediation

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| AgentConfigProjectionService | placement-aware inventory in `execution/agent_inventory.rs`; host seed-home auth copy plus bounded `config.toml` subset in `world-service/member_runtime.rs`; Codex home helpers in `crates/codex`; separate landed managed-gateway carrier in `crates/common/src/gateway_auth_bundle.rs`, `crates/world-service/src/gateway_runtime.rs`, and `crates/gateway/src/server/mod.rs` | `DefensiveScaffoldingOnly` | no | no | no | Point direct world Codex at the already-landed in-world gateway carrier and replace copied host auth/config authority with per-worker Substrate-owned projection. Keep `CODEX_HOME`, `.codex`, `config.toml`, `.mcp.json`, provider endpoint, and runtime-native config as rebuildable non-secret projections from logical config plus accepted policy. Any copied credential path remains an explicitly named/logged compatibility mode with retirement criteria and is barred from `ContractCorrectAndProven`. | WorldRuntimeAdapterExecutionEnvelope; RuntimeFamilyRealizationAdapter; EffectivePolicyResolver; RetainedWorkerRuntime; WorldCommandExecutionBroker |
| WorldRuntimeAdapterExecutionEnvelope | world Codex guest-entrypoint validation in `agent_runtime/validator.rs`; placement config in `config/agents/codex.yaml`; launcher/env/cwd plus copied seed-home setup in `world-service/member_runtime.rs`; reusable managed-gateway FD carrier and one-time consumer | `DefensiveScaffoldingOnly` | no | no | no | Persist one envelope bound to world generation, retained worker, Substrate-owned config projection, immutable policy snapshot, runtime deps, command-broker posture, and credential posture. Reuse the existing secure-FD carrier, join its non-secret evidence to the exact gateway receiver, and give Codex the gateway endpoint/session contract instead of copied credentials. Fail closed or use named non-promotable compatibility mode when exact adoption is unavailable. | AgentConfigProjectionService; RuntimeFamilyRealizationAdapter; WorldCommandExecutionBroker; RetainedWorkerRuntime |
| WorldCommandExecutionBroker | existing world-service execution, guard, overlay/full-isolation/Landlock/network primitives; no UAA per-operation broker; Codex gateway currently enables external-sandbox bypass | `MissingSeam` | no | no | no | Interpose on every world-UAA shell/edit/write/MCP/tool/process/network side effect; execute under the envelope's `PolicySnapshotV3`; disable or reject unbrokerable channels. | WorldRuntimeAdapterExecutionEnvelope; EffectivePolicyResolver; RuntimeFamilyRealizationAdapter; world-service enforcement |
| RuntimeFamilyRealizationAdapter | `crates/gateway/src/adapter_runtime.rs`; UAA/client construction in shell and world-service; provider-specific session/output handling; managed gateway already accepts one-time FD auth independently of the direct member path | `UsefulFootholdButWrongBoundary` | no | not applicable | no | Keep only provider mechanics; consume Substrate-owned envelope/config/policy/receipt plus the existing in-world gateway endpoint/session inputs. Point Codex/provider traffic at that gateway; never receive raw host credentials or read the secure handoff FD. Remove lifecycle, binding, sandbox-authority, and credential-authority decisions from family adapters; copied credentials remain named/logged compatibility only. | WorldRuntimeAdapterExecutionEnvelope; AgentConfigProjectionService; WorldCommandExecutionBroker; RetainedWorkerRuntime |

### B3.2a-WA bounded seam-ownership addendum

Canonical content: [`b1-b2-1/crosswalk.md#b32a-wa-bounded-seam-ownership-addendum`](b1-b2-1/crosswalk.md#b32a-wa-bounded-seam-ownership-addendum).

### B1/B2.1 activated-store ownership and production-handoff correction

Canonical content: [`b1-b2-1/crosswalk.md#b1b21-activated-store-ownership-and-production-handoff-correction`](b1-b2-1/crosswalk.md#b1b21-activated-store-ownership-and-production-handoff-correction).

### B2.1-3 producer replay and startup activation correction

Canonical content: [`b1-b2-1/crosswalk.md#b21-3-producer-replay-and-startup-activation-correction`](b1-b2-1/crosswalk.md#b21-3-producer-replay-and-startup-activation-correction).

### B1/B2.1 dispatch-authority ownership audit

Canonical content: [`b1-b2-1/crosswalk.md#b1b21-dispatch-authority-ownership-audit`](b1-b2-1/crosswalk.md#b1b21-dispatch-authority-ownership-audit).

## Classification consequences

Canonical content: [`seams/README.md#classification-consequences`](seams/README.md#classification-consequences).

## A1.1d-5R2-2F0-HC corrected consolidated harness crosswalk

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f0-hc-corrected-consolidated-harness-crosswalk`](a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f0-hc-corrected-consolidated-harness-crosswalk).

### Corrected environment source closure and ownership

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#corrected-environment-source-closure-and-ownership`](a1.1d-5r2-2f/crosswalk.md#corrected-environment-source-closure-and-ownership).

### Binding additive source and test manifest

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#binding-additive-source-and-test-manifest`](a1.1d-5r2-2f/crosswalk.md#binding-additive-source-and-test-manifest).

## A1.1d historical differential authority crosswalk

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#a11d-historical-differential-authority-crosswalk`](a1.1d-5r2-2f/crosswalk.md#a11d-historical-differential-authority-crosswalk).

## A1.1d-5R2-2F readiness source closure

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f-readiness-source-closure`](a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f-readiness-source-closure).

### Readiness semantics closed over the owner

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#readiness-semantics-closed-over-the-owner`](a1.1d-5r2-2f/crosswalk.md#readiness-semantics-closed-over-the-owner).

## Historical F5-PD nested diagnostic seam crosswalk

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#historical-f5-pd-nested-diagnostic-seam-crosswalk`](a1.1d-5r2-2f/crosswalk.md#historical-f5-pd-nested-diagnostic-seam-crosswalk).

### Source closure

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#source-closure`](a1.1d-5r2-2f/crosswalk.md#source-closure).

### Impact and ownership closure

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#impact-and-ownership-closure`](a1.1d-5r2-2f/crosswalk.md#impact-and-ownership-closure).

## A1.1d-5R2-2F completed seam crosswalk

Canonical content: [`a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f-completed-seam-crosswalk`](a1.1d-5r2-2f/crosswalk.md#a11d-5r2-2f-completed-seam-crosswalk).
## Broad-wall invocation provenance crosswalk

Canonical content: [`a1.1d-5r2-2-renewed-closeout/crosswalk.md#broad-wall-invocation-provenance-crosswalk`](a1.1d-5r2-2-renewed-closeout/crosswalk.md#broad-wall-invocation-provenance-crosswalk).

## Renewed R2-2 publication crosswalk

Canonical content: [`a1.1d-5r2-2-renewed-closeout/crosswalk.md#renewed-r2-2-publication-crosswalk`](a1.1d-5r2-2-renewed-closeout/crosswalk.md#renewed-r2-2-publication-crosswalk).

## Closeout-remediation crosswalk

Canonical content: [`a1.1d-5r2-2-renewed-closeout/crosswalk.md#closeout-remediation-crosswalk`](a1.1d-5r2-2-renewed-closeout/crosswalk.md#closeout-remediation-crosswalk).

### P1 historical runner note and current authority

Canonical content: [`a1.1d-5r2-2-renewed-closeout/crosswalk.md#p1-historical-runner-note-and-current-authority`](a1.1d-5r2-2-renewed-closeout/crosswalk.md#p1-historical-runner-note-and-current-authority).

### R1 direct-caller inventory

Canonical content: [`a1.1d-5r2-2-renewed-closeout/crosswalk.md#r1-direct-caller-inventory`](a1.1d-5r2-2-renewed-closeout/crosswalk.md#r1-direct-caller-inventory).

### P1 source-closure decision

Canonical content: [`a1.1d-5r2-2-renewed-closeout/crosswalk.md#p1-source-closure-decision`](a1.1d-5r2-2-renewed-closeout/crosswalk.md#p1-source-closure-decision).
## R2-4 terminal crosswalk disposition

Canonical content: [`a1.1d-5r2-4/seam-crosswalk-disposition.md#r2-4-terminal-crosswalk-disposition`](a1.1d-5r2-4/seam-crosswalk-disposition.md#r2-4-terminal-crosswalk-disposition).

## A1.1d-5R3 canonical ownership and source-closure crosswalk

Canonical content: [`a1.1d-5r3/crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk`](a1.1d-5r3/crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk).

### Historical `R3CleanupOnly` rows (superseded for active scheduling)

Canonical content: [`a1.1d-5r3/crosswalk.md#historical-r3cleanuponly-rows-superseded-for-active-scheduling`](a1.1d-5r3/crosswalk.md#historical-r3cleanuponly-rows-superseded-for-active-scheduling).

### R3 action fences embedded in broader rows

Canonical content: [`a1.1d-5r3/crosswalk.md#r3-action-fences-embedded-in-broader-rows`](a1.1d-5r3/crosswalk.md#r3-action-fences-embedded-in-broader-rows).

### Finding and gate ownership

Canonical content: [`a1.1d-5r3/crosswalk.md#finding-and-gate-ownership`](a1.1d-5r3/crosswalk.md#finding-and-gate-ownership).

### Reverse ownership and frozen boundaries

Canonical content: [`a1.1d-5r3/crosswalk.md#reverse-ownership-and-frozen-boundaries`](a1.1d-5r3/crosswalk.md#reverse-ownership-and-frozen-boundaries).

### Source-closure and delivery refinements

Canonical content: [`a1.1d-5r3/crosswalk.md#source-closure-and-delivery-refinements`](a1.1d-5r3/crosswalk.md#source-closure-and-delivery-refinements).

### `A1.1d-5R3-MAC` implementation status

Canonical content: [`a1.1d-5r3/crosswalk.md#a11d-5r3-mac-implementation-status`](a1.1d-5r3/crosswalk.md#a11d-5r3-mac-implementation-status).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/crosswalk-status.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/crosswalk-status.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN crosswalk status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/crosswalk-status.md#aux-r3-mac-evidence-recovery-plan-crosswalk-status-2026-08-07`](r3-mac-evidence-recovery/crosswalk-status.md#aux-r3-mac-evidence-recovery-plan-crosswalk-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN authority supersession (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/crosswalk-status.md#aux-r3-mac-evidence-recovery-plan-authority-supersession-2026-08-07`](r3-mac-evidence-recovery/crosswalk-status.md#aux-r3-mac-evidence-recovery-plan-authority-supersession-2026-08-07).
## Current cross-lane scheduling and ownership (2026-08-20; controlling)

Canonical content: [`macos-dev-parity/cross-lane-scheduling-and-ownership.md#current-cross-lane-scheduling-and-ownership-2026-08-20-controlling`](macos-dev-parity/cross-lane-scheduling-and-ownership.md#current-cross-lane-scheduling-and-ownership-2026-08-20-controlling).
