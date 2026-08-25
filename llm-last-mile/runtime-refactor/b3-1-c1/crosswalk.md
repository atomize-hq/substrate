**Kind:** crosswalk
**Stable ID:** `B3.1-C1-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** B3.1 messaging and C1 obligation family rows plus the recorded B3.1 crosswalk result
**Source span:** composite of the three preserved root compatibility spans listed in the extraction ledger

# B3.1/C1 family seam crosswalk

## WorldWorkerMessagingProtocol

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldWorkerMessagingProtocol | current `ContinueWorldWorkerEventV1`/event-class adapters in `dispatch_contract.rs`; JSON-pointer thread/class/attention/payload classification and optional-event drop in `orchestrator_world_dispatch.rs`; current shared `AgentEvent` plus `ExecuteStreamFrame::Event`; provider `AgentWrapperEvent` to `AgentEvent` construction in `world-service/member_runtime.rs` | `UsefulFootholdButWrongBoundary` | no | no | no | B3.1 adds the producer-construction `NormalizedWorldWorkerEventFacetV1` and optional typed top-level retained worker-to-host member carried by the existing `ExecuteStreamFrame::Event { event: AgentEvent }`; B1 has already added the shared equality-only correlation/commitment types and typed acceptance context. After B1 acknowledgement and before `AgentEvent` construction, one fail-closed world-service normalizer maps every retained provider wrapper event into exact thread/class/attention/payload semantics and request/message causation from the retained `WorldWorkAcceptanceContextV1`; no upstream wrapper redesign is required. World-service then joins that facet with exact accepted record, source/target participant and backend, active run, world, and B0 identity. Every post-ack retained `Event` frame must carry the typed member; an unknown, ambiguous, or malformed provider shape fails the stream and C1 cut closed instead of being dropped, silently becoming progress/no-attention, or remaining untyped. B2.1 already commits generic canonical `AgentEvent` bytes without interpreting them; B3.1 verifies that commitment equals the full typed member, copies any B1 opaque host-transition correlation unchanged, and validates equality without authenticating, verifying, or interpreting it. Request ID and active-run ID cannot be inferred as transition intent/run, and after emission host/C1 code cannot use `AgentEvent.data` to supply missing identity or semantics. B3.2 later completes remaining messaging/lifecycle behavior. | RuntimeEventTransport; WorldWorkExecutionSupervisor; RetainedWorkerRuntime; SteeringPolicyEngine; ObligationLedger |

## Recorded B3.1 result

Recorded B3.1 result: on the bound Tuesday, August 4, 2026 source candidate, the retained
worker-to-host envelope is complete on the real retained production path. `world-service` now
normalizes supported provider wrapper events fail-closed before `AgentEvent` construction, shell
validation consumes the typed member instead of C1-path JSON-pointer repair, and the packet leaves
no seam promoted. C1 and the bounded internal A1.2b packet are complete on that same bound
Tuesday candidate; the then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`. Its
former macOS-parity replacement is now superseded as the global schedule by
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md).

## ObligationLedger

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| ObligationLedger | `agent_runtime/obligation_ledger.rs`; obligation persistence/projection helpers in `state_store.rs`; post-stream materialization in `orchestrator_world_dispatch.rs` | `DefensiveScaffoldingOnly` | no | no | no | Consume B2.1's durable exact event journal through its generic typed journal-event reference; after B3.1 validates that each canonical transport commitment is the complete shared retained envelope, reverify it, then idempotently classify and materialize obligations before terminal exit when applicable. C1 alone owns canonical obligation revisions/records, the monotonic per-session ledger revision, the materialized-through event watermark, and the closed Complete/Pending snapshot query scoped to exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, and an exhaustive ordered join to every post-ack retained B2.1 `Event` ref with committed B3.1 source/target/thread/class/attention/causation/payload semantics through the cut even when no attention obligation exists, plus unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run and authority revision. An untyped, omitted, or substituted retained event keeps the cut non-Complete. A1.2 consumes that semantic result unchanged and cannot invent the event or transition correlation; a shared opened-root lock supplies physical serialization, never semantic authority. Stream exhaustion, EOF, timeout, inbox rows/counts, worker flags, and compatibility projections are excluded. Until C1 supplies a Complete cut, resumable A1.2 post-turn remains Pending and this seam is not promoted. | RuntimeEventTransport; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; WorldWorkerMessagingProtocol; InboxProjection; AutoAttachProjection; HostSessionAuthority |
