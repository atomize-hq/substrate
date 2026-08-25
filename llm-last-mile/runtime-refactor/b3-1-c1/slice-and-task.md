**Kind:** slice and task record
**Stable ID:** `B3.1-C1-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** B3.1 and C1 packet/slice rows and recorded results only
**Source span:** composite of the four preserved root compatibility spans listed in the extraction ledger

# B3.1/C1 family slice and task record

## B3.1 packet row

| Packet or slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B3.1 — Bounded retained-turn event/causation prerequisite** | Provide only the typed worker-to-host retained event envelope C1 requires, joined to B0 identity and B1 active-run truth, and normalize provider semantics at the producer before `AgentEvent` construction. | WorldWorkerMessagingProtocol. | `01` authority map/invariants 3–6; `02` MessagingProtocol row; `04` acceptance context + producer semantic normalization + retained event envelope; `05` `RG-MSG-01`/`RG-OBL-01`; messaging design envelope/thread/ordering/attention sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; RetainedWorkerRuntime; ObligationLedger. | Add `NormalizedWorldWorkerEventFacetV1`, the optional top-level `AgentEvent.worker_event: Option<WorldWorkerEventV1>`, and their typed fields in `crates/common/src/agent_events.rs`; consume B1's shared types from `crates/common/src/{authority_commitment.rs,lib.rs}`; consume/validate the existing B1 `WorldWorkAcceptanceContextV1` request extension in `crates/transport-api-types/src/lib.rs` without changing `ExecuteStreamFrame`; move the current provider-payload thread/class/attention/payload interpretation into one explicit fail-closed producer normalizer and shape the final member from that facet plus retained request/runtime context in exactly `crates/world-service/src/member_runtime.rs`; bounded host validation, deletion of C1-path JSON-pointer classification, and generic-journal handoff in `crates/shell/src/execution/orchestrator_world_dispatch.rs`; bounded host-facing semantic adapters in `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`; only mechanical `worker_event: None` compatibility updates in `crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs`, `crates/shell/src/repl/async_repl.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/support/repl_world_service.rs`, and `crates/world-service/src/service.rs`; focused tests colocated in those exact files. | No `ExecuteStreamFrame` variant/field change, no upstream provider-wrapper contract change, no new request-envelope semantics beyond consuming B1's context, no host or ledger identity/semantic inference from emitted `AgentEvent.data`, no producer-chosen eligible subset, no omission/untyped event/permissive downgrade for an unknown or malformed provider shape, no separate transport protocol, no host-to-worker message completion, no foreground early return, no worker park/cancel/stop/fork rewrite, no host-transition issuance/authentication/interpretation, no obligation classification/materialization, and no policy broadening. | After exact B1 acknowledgement, every retained `ExecuteStreamFrame::Event` through the terminal cut is in the closed B3.1 domain. Before `AgentEvent` construction, the producer normalizer maps every existing provider `AgentWrapperEvent` kind/payload into an exact typed facet for thread/class/attention/payload; parsing provider payload is permitted only at that named adapter boundary. Explicit supported non-attention shapes receive a typed non-attention class; missing, ambiguous, deferred, unknown, or malformed shapes fail the stream and C1 cut closed rather than being dropped, left untyped, or becoming progress/no-attention. The final typed `AgentEvent.worker_event` joins that facet with exact sources: acceptance ID, request/message causation and target backend come from the retained B1 context; session, source/target participant, source backend, active run, and world come from typed request/runtime context; B0 supplies event/frame identity. B3.1 validates that B2.1's generic event commitment equals the full canonical typed member and that all B0/B1/context fields match before generic-journal/C1 handoff; any opaque correlation is copied unchanged and compared only for equality. Host-side JSON-pointer classification is not a C1 input. Missing or mismatched identity fails closed; request ID or active-run ID never substitutes for transition intent/run. | Bounded producer/consumer clauses of `RG-MSG-01`; prerequisite clauses of `RG-OBL-01`; B3.1 event clause of `RG-OBS-01`. | B1/B2.1 joint production closeout and landed A1.1e. | Remaining `RG-MSG-01`, upstream provider-wrapper taxonomy cleanup, A1.2b production correlation supply, foreground retained receipt return, host-to-worker messaging, retained lifecycle/park/nonzero-exit behavior (`B3.2`), C1 materialization, and seam promotion. |

## B3.1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, B3.1 completes the retained
worker-to-host event/causation packet without changing `ExecuteStreamFrame`. `world-service` now
normalizes supported retained provider wrapper events fail-closed before `AgentEvent`
construction, joins one `NormalizedWorldWorkerEventFacetV1` with exact B1 accepted context,
typed runtime context, and copied B0 stream/frame/event identity, and emits the typed
`AgentEvent.worker_event` member on every post-acknowledgement retained `Event` frame through the
terminal cut. Shell consumers validate the full typed envelope, reject mismatched scope or
correlation, delete the old C1-path JSON-pointer inference, and hand the canonical commitment to
the existing B2.1 generic journal without interpreting it there.

Focused common, transport, world-service, and shell retained-event tests are green. Both
`make shell-lib-wall` and `make shell-lib-wall-serial` retain
`1324 discovered / 1276 passed / 48 failed / 0 ignored` and failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`. That inventory growth is
exactly two new passing shell tests:
`execution::orchestrator_world_dispatch::tests::accepted_retained_typed_event_validation_precedes_generic_journaling`
and
`execution::orchestrator_world_dispatch::tests::typed_control_ack_projection_uses_top_level_worker_event_class`.
The historical normalized-signature SHA-256
`2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90` first moved to the
independently authorized intermediate candidate SHA-256
`a1753d3d2dcd19f36a9350660ce88d9f2713871dd7734414c005ab8e52e70470`, then to the superseded
serial candidate SHA-256 `401fd5d38e059961de7fd4f773f7a817b9f2c1790ded0500228ed1168f7e811a`,
and finally to the accepted candidate SHA-256
`167807acacbf51c8507ef1c6a20d195b5d1a19e66ea62e5d612de1c2ef340f17`. The accepted final
differential against the superseded serial candidate is limited to 23 FILE:LINE-only movements in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`; test names, file paths, columns, and
normalized panic bodies remain unchanged, and the exact historical-to-final differential remains
recorded in
[`review-control/b3-1-differential-evidence.json`](../review-control/b3-1-differential-evidence.json).
No test name, failure message, assertion, or production behavior is removed, renamed, substituted,
ignored, or weakened. B3.1 is complete, C1 is complete on the same bound Tuesday candidate,
A1.2b is complete on that same bound candidate as the internal durable successor/post-turn
protocol only, and no seam is promoted.

## C1 packet row

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C1 — Event-to-obligation materializer and semantic cut** | Consume durable exact events, idempotently create canonical obligations while the supervisor observes active work, and own the monotonic ledger revision/materialized-event cut plus closed snapshot query consumed by A1.2b. | ObligationLedger. | `01` invariants 4–6; `02` RuntimeEventTransport/Supervisor/Messaging/Obligation rows; `04` runtime carrier, supervisor rules, and `ObligationLedgerSnapshotReadV1`; `05` `RG-OBL-01`/`RG-OBL-02`; obligation-ledger producer/dedupe sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; MessagingProtocol; HostSessionAuthority consume-only client. | `agent_runtime/obligation_ledger.rs`; bounded consumer integration from the receipt-scoped supervisor journal; bounded StateStore persistence only; focused materialization/cut/snapshot tests. | No runtime identity generation, receipt acceptance, observation ownership, retained-envelope or host-transition semantics, producer event eligibility choice, inbox rendering, router launch, direct prompt injection, or transfer of obligation semantics to HostSessionAuthority/StateStore. Stream exhaustion, EOF, timeout, PID/helper/socket state, inbox rows, pending counts, worker flags, and compatibility projections are forbidden completeness inputs. | Each B3.1 attention event creates exactly one canonical obligation before the exact B0 terminal event when applicable; duplicate journal replay creates none. Before classification and snapshot capture, C1 verifies that every post-acknowledgement retained B2.1 `Event` journal ref through the cut commits one full canonical B3.1 target/source/thread/class/attention/request/message/transition/payload envelope. The ledger advances one monotonic session revision and materialized-through event watermark, returns Pending before coverage, then returns a Complete snapshot binding exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, and the complete ordered exhaustive join of all retained `Event` refs through the cut even for `NoUnresolvedAttention`, plus unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run, authority revision, disposition, and sorted canonical-record commitments. Any untyped/omitted/substituted event or stale/mismatched scope, correlation, revision, cut, disposition, or record commitment keeps the cut non-Complete. | `RG-OBL-01`, C1 clauses of `RG-OBL-02`, obligation clauses of `RG-SUP-01`, bounded consumer clauses of `RG-MSG-01`, and C1 clauses of `RG-OBS-01`. | A1.1e, B0, B1, B2.1, and B3.1. | A1.2b production correlation supply and consumption/adoption, C2 projections, C3 router behavior, B2.2 foreground early return, and every seam promotion. |

## C1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, C1 is complete. The accepted retained path
now feeds exact B1 acceptance plus B2.1 durable retained `Event` refs and validated B3.1 envelopes
into `ObligationLedger` as each typed `Event` or terminal `Exit` is durably accepted, removes the
old terminal-coupled production writer from that C1-owned accepted path, and advances the
monotonic ledger revision/materialized-through watermark without reopening B0/B1/B2.1/B3.1
ownership. Broad proof remains monotonic through both `make shell-lib-wall` and
`make shell-lib-wall-serial` at `1330 discovered / 1282 passed / 48 failed / 0 ignored`, with
failure-name SHA-256 `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and
normalized-signature SHA-256
`e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40`. That inventory growth is
exactly six new passing shell tests, and the retained 48-failure differential is limited to 23
FILE:LINE-only movements in `crates/shell/src/execution/orchestrator_world_dispatch.rs` recorded in
[`review-control/c1-differential-evidence.json`](../review-control/c1-differential-evidence.json).
A1.2b is complete on that same bound Tuesday candidate as the internal durable successor/post-turn
protocol only, the then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`, and no seam
was promoted. The later reentry gate is closed by the selected
[A1.3 Linux-first packet](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), later
narrowed by the held
[A1.3-P0 Linux-first preparatory packet](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and finally corrected to the active
[A1.3-P1 Linux-first atomic public-adoption packet](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
