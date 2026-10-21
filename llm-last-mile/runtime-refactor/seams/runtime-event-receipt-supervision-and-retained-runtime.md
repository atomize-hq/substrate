**Kind:** seam family
**Stable ID:** `runtime-event-receipt-supervision-and-retained-runtime-family`
**Canonical for:** runtime event, receipt, supervision, and retained-worker runtime seam extraction plus the current E2 and second `E2-RM` ownership corrections and proposed B01/B03 composition boundary
**Status:** canonical current seam-family record
**Authority scope:** exact extracted family-local source bodies plus the documentation-only E2 and `E2-RM` ownership corrections and proposed B01/B03 composition boundary; no implementation authority
**Source span:** D8 runtime-event, receipt, supervision, and retained-runtime family extraction from `02-seam-crosswalk.md`
**Supersedes:** canonical ownership of the extracted `RuntimeEventTransport`, `WorldWorkReceiptRegistry`, `WorldWorkExecutionSupervisor`, and `RetainedWorkerRuntime` rows
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../02-seam-crosswalk.md`](../02-seam-crosswalk.md)

# Runtime event, receipt, supervision, and retained runtime seam extraction

> **Authority boundary:** This file owns only the extracted `RuntimeEventTransport`, `WorldWorkReceiptRegistry`, `WorldWorkExecutionSupervisor`, and `RetainedWorkerRuntime` seam rows. It does not promote any seam, move the A0 authority-leak inventory, reopen D5/D6/D7 or already-extracted D8 family owners, rewrite `b1-b2-1/` or `b3-1-c1/`, extract the remaining C/D seam families, or authorize B0/B1/B2.1/B3.1/B3.2/B4/C implementation work.

## Proposed B01/B03 composition boundary — 2026-10-21

The [joint consumer contract](../../../docs/contracts/draft/collider-b01-b05-handoff.md#32-immutable-request-identity-and-the-admission-boundary) and [B03 protocol](../../../docs/contracts/draft/lane-world-binding.md#47-associationadmission-consistency-and-recovery) propose an authenticated consumer ingress, original-request/material correlation, exact association reservation and runtime consume/fence extension. These are target responsibilities, not present exported APIs, implementation admission, component-owner signoff or a change to the historical seam classifications below.

B1 remains sole proposal/immutable ACK-acceptance owner. A consumer correlation reverse-joins its one existing session/request proposal; runtime `Consumed` is only effect admission and cannot create B0 Start or B1 truth. B2.1 alone claims/journals/replays exact accepted streams and owns terminality; E2 retains original admitted policy material; B2.2 constructs/returns the receipt; Workspace association authority authenticates the reserved historical binding. S1 retains original B04 material and separately joins actual authorized delivery. Complete consumer acceptance requires all original B1/B2.1/E2/B2.2/B03 joins, but proves neither full B04 delivery nor installed readiness.

Dispatch's proposed recovery coordinator can complete original missing owner publications after exact evidence; read-only reconciliation cannot invoke it, repair indexes or execute again. Preparing, possible-send unresolved, accepted-material-incomplete, complete acceptance, conclusive no-send refusal and read/integrity failure remain distinct. Original acceptance survives foreground/publication loss. Runtime gate/epoch closure precedes authoritative effect-facing revocation/currentness commit; all applicable invalidators must participate or the profile is unsupported. Emergency invalidation/cancellation and a later current-use/delivery guard remain independent.

B03 §§4.7–4.9 supplies concrete source/module/symbol placement, per-owner locking/forward-join recovery, positive fence/close/release evidence and every crash boundary. B01 §§3.3–3.4 supplies non-reconciling authenticated reads, privacy/retention and real-profile proof obligations. No duplicate acceptance registry, scheduler, global authority lock across runtime I/O or frozen permissions is introduced. Existing replay remains bounded process-memory producer transport and cannot prove no-send after service restart. This amendment neither promotes seams nor reopens retained-worker/E3/I1/I2 work.

[`B01-B03-P1`](../slices/tasks/b01-b03-authenticated-preflight.md) binds proposed initial native Workspace open/load, separately approved absent-only initialization and distinct no-send preflight. The human clarification supersedes the setup-issuer prerequisite: no enrollment/pairing/new approval for configured open. Admission contract bound is separate from source dispatch (not authorized) and runtime proof (unperformed). Normal open/preflight has no identity/correlation/proposal/reservation, reconciliation, runtime gate or execute effects; nonreconciling exact HSA/custody/config/policy/association reads and current disclosure checks must meet the packet. Initialization is a separate declared mutation. Eligibility grants no future send/B1 acceptance or promotion of the later full-send composition above.

**Current E3 status correction — 2026-10-21:** the [canonical dated E3-F correction](../slices/e3-agent-config-projection-and-gateway-adoption.md#current-e3-f-status-correction--2026-10-21) supersedes inherited current-status projections below: E3-F was dispatched and is paused; W8 remains unaccepted/unclosed, `c381` published-unintegrated; no continuation or closure follows. The
[I1/I2 design binding](../contracts/agent-config-projection-v1.md#e3-f-i1i2-ownership-and-accounting-reconciliation)
now specifies required tagged retained ownership and sealed adapter-to-gateway cleanup accounting.
Per-turn readback while gateway lives is process-local custody evidence only; whole-attempt durable
terminal cleanup requires every exact registered process/cgroup and all original ordering/retry
invariants. Neither kernel cleanup evidence, a reaped process nor gateway liveness establishes B0
terminal truth, B1 acceptance or B2.1 completion. Those existing owners remain unchanged; no new
supervisor/registry/schema or seam promotion follows. Implementation/proof are outstanding. The
[canonical scheduling checkpoint](../slices/e3-agent-config-projection-and-gateway-adoption.md#i1i2-design-binding-and-runtime-return-sequence--2026-10-21)
keeps I1/I2 first, separately authorized paused-W8 E3-F continuation/proof next, then separate P1
selection at the bounded proof return. This is priority, not an E3-F prerequisite for read-only opening.

## Current E2 ownership correction

E2 consumes but does not mutate B1 acceptance, the exact source-owned B2.1
`WorldWorkExecutionClaimV1`, or fresh-Spawn B3.2a admission truth and persists the separate immutable
[`DispatchPolicyCommitmentV1`](../contracts/dispatch-policy-commitment-v1.md). B2.1 may expose one
read-only, behavior-neutral projection of the exact claim identity, canonical preimage/hash inputs,
and durable `executions_by_acceptance_record_id` key. It exposes no invented
`SupervisorObservationClaimV1`/`resumable` field, does not change state or lifecycle meaning, and
does not mutate receipts/observations. Cursor/interruption references remain separate B2.1-owned
facts and are absent unless an E2 subject actually requires them.

For Fresh Spawn, E2 first-writer CAS-stores the immutable complete
patch/snapshot/cap/subject/binding reservation and ref, an E2-keyed complete validated-request
commitment with no prompt/payload preimage, and stable worker/bootstrap identity,
completes publication and `fsync`, privately recomputes/equality-checks the request commitment, then
supplies only an opaque authenticated reservation capability/ref plus the identities through the
bounded B3.2a reservation input. B3.2a consumes the preallocated identities in its
unchanged fingerprint. Only the existing `allow_capability_narrowing == true` rejection may admit an
authenticated E2 narrowing attestation so a nonempty `RestrictedWorldFs` E1 patch can pass;
`UnchangedParent` and empty patches cannot obtain that attestation. The opaque proof is threaded
through all seven validator call sites and only their minimum wrappers, including
`mark_admission_routable`, `mark_admission_terminal`, and `mark_admission_interrupted`, without entering B3.2a
plan/fingerprint/record bytes. After the first
registration-bearing state, E2 retains the reservation ref,
CAS-links all stable source-record fields plus exact registration, and publishes the commitment.
Mutable B3.2a state/revision is excluded from that stable link. The reservation grants no admission,
lifecycle, or routability and cannot change B3.2a's fingerprint, schema, other validation, registry,
cap accounting, ordering, registration, transport, or lifecycle semantics. Crash/retry joins reservation, B3.2a admission, or
commitment at their exact durable boundary; changed material conflicts before B3.2a.

RetainedWorkerRuntime keeps all target, admission, identity, routability, and launch-policy/cap
authentication and returns the immutable cap separately from current parent. It must return typed
unsupported-legacy state before Continue/Fork when canonical cap bytes/ref are missing or
hash-invalid; it may not reconstruct a cap from a newer parent. Launch-policy/current-parent
equality is not required, so later parent narrowing restricts future work and broadening remains
capped. Fork uses an E2-owned strict request-to-cap link and gains no B3.2a admission authority.
ReceiptRegistry and Supervisor do not become policy owners. B2.2/B3.2 later consume the exact ref;
the full manifest also waits for D1/E3 identities. No routing, supervisor, cancellation, receipt,
manifest, or lifecycle behavior is authorized by this correction.

The historical B2.2 admission finding adds no ReceiptRegistry or Supervisor authority to E2. It identified
the now implemented/review-clean/landed
[`E2-RM` projection](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite):
an E2-owned read-only lookup from exact request/subject identity plus expected B1 acceptance to the
already-stored immutable E2 record, preserved historical B2.1 claim preimage/hash, E1 snapshot
bytes/ref/hash/revision/reason, and retained cap bytes/ref/hash when applicable. The projection
never reads current B2.1 observer/journal/terminal state, never reconstructs from current parent
policy, and cannot mutate, reconcile, or reinterpret any source owner.

`E2-RM` preserves the seam split: B1 owns acceptance, B2.1 owns observation/journal/replay/terminal
truth, E2 owns commitment/cap material, and B2.2 later owns foreground receipt construction and
return. B3.2, B4, C2/C3, D1, and E3 retain their existing scopes. [B2.2](../slices/b2-2-foreground-receipt-return.md#current-admission-disposition) records its separate
implemented/review-clean/landed status. Separate canonical terminal closure was not located for
E2-RM or B2.2; the earlier admission correction remains historical. This status correction admits
neither packet and promotes no whole seam.

### Second `E2-RM` seam correction (2026-09-03; controlling)

**Integration status qualification (2026-09-17):** The second correction below remains
controlling for read semantics, ownership, and the bounded implementation/proof fence. Its
“future”, “unadmitted”, “unimplemented”, and B2.2-blocked statements record 2026-09-03
scheduling and proof status, not current blockers. E2-RM and B2.2 are now
implemented/review-clean/landed as recorded above; separate canonical terminal closures
were not located. E3-A–D landed; [E3-E is closed](../slices/e3-agent-config-projection-and-gateway-adoption.md#e3-e-terminal-closure-2026-09-17);
enclosing E3 remains incomplete, and E3-F remains unadmitted and undispatched. This
qualification creates no proof result, closure, admission, or new implementation authority.

The read boundary spans existing physical owners without transferring their semantics. HSA owns a
new future non-reconciling read transaction over the already-existing accepted-home layout and
root lock. E2 owns only its physical immutable-registry/key snapshot and final receipt-material
projection. B1 owns only canonical full-registry validation and the opaque durable acceptance
witness. `store.rs` may provide only the aggregate
root/store/state-root-bytes/root-revision/E2/B1 snapshot wiring. These layers capture E2 and B1
under the same root lock; no later lookup may supplement the snapshot.

The existing HSA, E2, and B1 transaction paths remain unchanged because they may reconcile,
create, publish, or `fsync`. The new read path instead validates root/layout/lock identity,
owner/mode/type/device/inode/link count, safe namespace manifests, stable file bytes and metadata,
and clean absence. Recognized root/E2/B1 temporaries fail closed and remain byte-identical. A
missing complete E2 directory is clean absence, while an existing partial E2 layout is an error.
No read path may clean, repair, migrate, backfill, initialize/rotate keys, or publish authority.

Caller B1 data is an equality expectation and lookup selector only; authenticated B1 fields come
only from the witness keyed by `(authority_store_id, acceptance_record_id)`, returned directly as
the sole success shape. Registry absence, no matching record, violated uniqueness, store mismatch,
and unsafe/partial/corrupt material have only typed fail-closed error paths. Every independently
persisted B1 field has a typed expected-value mismatch classification, including distinct
`AuthorityRevisionObserved` and `AcceptedAt` results. E2's preserved B2.1 claim and immutable
snapshot/cap records remain the exclusive historical sources for their result fields. Current B2.1
observation, journal, terminal, retained-worker, and parent-policy state remain outside the
projection. Exact HSA state-root bytes and the store-wide HSA root revision are private
transaction-stability provenance only, not receipt material. No per-session current authority
revision is selected or consulted, and historic B1 `authority_revision_observed` is never compared
to current HSA authority. `accepted_at` and runtime observation time remain independent. Missing B1
cannot become a success, E2 legacy compatibility, synthesized evidence, or a reason for current
B1/B2.1 lookup. B2.2 receives no read, construction, return, or lifecycle authority here.

The exact future fence is limited to the six existing files in the controlling E2-RM contract and
their colocated tests. B1 persistence/schema/publication and B2.1 semantics do not change. E2 stays
terminally complete; E2-RM remains specified but unadmitted, undispatched, and unimplemented;
B2.2 remains blocked and unadmitted; and E3 remains separately specified but unadmitted,
undispatched, and unimplemented.

## RuntimeEventTransport

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| RuntimeEventTransport | Canonical `RuntimeFrameIdentityV1`, `RuntimeEventIdentityV1`, and `RuntimeTerminalIdentityV1` in `crates/common/src/agent_events.rs`, re-exported without aliases by `crates/transport-api-types/src/lib.rs`; identified `ExecuteStreamFrame`; `RuntimeEventStreamProducer` plus ordinary task-stream sinks in `crates/world-service/src/service.rs`; retained launch/turn producers in `crates/world-service/src/member_runtime.rs`; carrier-only typed decoders in the named shell consumers; bounded process-memory replay in `crates/world-service/src/runtime_replay.rs` plus its exact handler/route and transport client types | `MissingSeam` **with B0 producer carrier and B2.1 replay transport recovered; full seam unpromoted** | producer/replay transport yes; full seam no | producer and recovered supervisor consumer core yes; accepted-family production path yes | B0 and B2.1 component/restart proof yes; accepted-family production proof yes; full seam no | B0 landed one non-empty UUIDv7 `stream_id` per accepted producer stream, `schema_version = 1`, one frame counter shared by Start/stdout/stderr/Event/Exit/Error, one semantic counter for Event/Exit, stable UUIDv7 event IDs, exact Exit/terminal identity equality, producer closure after terminal/error, and canonical NDJSON clone/re-emission byte stability. The stdout/stderr sink holds the producer lock through enqueue, so assignment order is emission order. Ordinary task, retained launch, and retained submitted-turn paths all construct identity before host observation. Shell compatibility decodes and forwards the carrier unchanged; missing/malformed identity fails, and Error/EOF/stream exhaustion cannot create terminal truth or `AlreadyTerminal`. Standalone legacy `AgentEvent` decoding may omit event identity, but an in-repository runtime `Event` may not. B2.1-3 adds only bounded process-memory producer retention and exact replay transport keyed by supplied acceptance-record ID, stream ID, and frame cursor; no enumeration or fuzzy/session-only lookup is allowed. Replay preserves exact B0 identity, order, and bytes and creates no acceptance, claim, lifecycle, terminal, obligation, or success truth. The recovered B2.1 supervisor owns duplicate/gap/reorder/conflict handling, durable consumer journal/replay acceptance, and restart observation; the producer registry remains process-memory world-service transport state, so this row is not promoted to `ContractCorrectAndProven`. B0 added no receipt, retained-message, obligation, lifecycle, or foreground-return semantics. The B1 receipt and B2.1 supervisor/replay cores plus their joint production integration closeout are review-clean at the bound 2026-08-03 source snapshot without additional product/test edits, and B3.1 is unblocked without promoting this seam. | WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; WorldWorkerMessagingProtocol; ObligationLedger |

## WorldWorkReceiptRegistry

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldWorkReceiptRegistry | recovered `world_work_receipt_registry.rs` immutable acceptance core and bounded HSA physical receipt capability; typed `WorldWorkAcceptanceContextV1`; remaining terminal-shaped `HostToolRunWorldTaskReceiptV1` compatibility surface | `MislandedWrongModel` | recovered core yes; full seam no | recovered core yes; joint production path open | component proof yes; full seam no | Recovered B1-3a/B1-3b provide durable accepted task and retained-turn acceptance records instead of temporary/terminal identity on the B-owned paths. ReceiptRegistry preallocates a proposed acceptance-record ID and sends it with exact request/message/caller-backend identity in `WorldWorkAcceptanceContextV1` on the existing typed task/turn request; world-service retains that context, but the proposal is not accepted truth until the exact B0 acknowledgement durably creates the matching immutable record. The record includes exact caller participant/backend, task/active-run ID, B0 stream identity/acceptance sequence, current policy identity, and optional intent/revision/opaque-payload-commitment/transition-run/authority correlation using the common equality-only types. Production correlation stays absent until A1.2b supplies it through the HostSessionAuthority-owned boundary; the registry retains and scope-checks it without authenticating, verifying, issuing, applying, or interpreting the transition. The receipt core is review-clean through `6436289f` without claiming B1 complete. B1 production completion waits for the joint B1/B2.1 closeout proving every accepted path enters the supervisor and never the legacy active-task writer. | RuntimeEventTransport; WorldDispatchControl; WorldWorkExecutionSupervisor; RetainedWorkerRuntime; EffectivePolicyResolver |

## WorldWorkExecutionSupervisor

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| WorldWorkExecutionSupervisor | recovered `execution/world_work_execution_supervisor.rs` durable claim/journal/recovery core plus bounded HSA physical capability; canonical recovery activation in `run_async_repl`; exact replay client; remaining blocking stream/waiter compatibility in `orchestrator_world_dispatch.rs` | `MissingSeam` **with B2.1 core recovered; accepted-family production closeout complete** | recovered core yes; full seam no | recovered core yes; joint production path yes | component/restart proof yes; accepted-family production proof yes; full seam no | Recovered B2.1-1 durably claims both task and retained-turn streams at exact B1 acceptance before any subsequent frame can bypass the supervisor and replaces legacy active-task registration on the accepted production path. Recovered B2.1-2 journals exact B0 identity and canonical bytes before waiter delivery, makes exact replay a no-op, rejects gaps/reorder/conflicts/stale observers/post-terminal frames, and preserves blocking compatibility over supervisor truth. Recovered B2.1-3 alone discovers every nonterminal claim and exact cursor, interprets durable state, and reconciles only through exact producer replay. A surviving world-service producer registry permits exact replay and live resumption after host/shell restart. A world-service restart or unavailable producer leaves the claim durably nonterminal and unresolved; it cannot fabricate terminal state, delete truth, advance the cursor, or complete an obligation cut. Caller/waiter drop does not delete claims or work; EOF, timeout, PID/helper/socket/process/readiness/caller/endpoint state, stream exhaustion, or runtime exit without the exact B0 terminal event cannot close a run or complete the cut. The recovered supervisor/replay cores are review-clean through `c519024b` and `de727091`, and the later joint B1/B2.1 production closeout is recorded at the bound 2026-08-03 source snapshot without additional product/test edits. B3.1 is now the next packet. | RuntimeEventTransport; WorldWorkReceiptRegistry; WorldDispatchControl; SurfaceAdapter / HostExecutionEpisode; WorldWorkerMessagingProtocol; ObligationLedger; RetainedWorkerRuntime |

## RetainedWorkerRuntime

Table-column projection: the repeated seam-table header below is a projection of the canonical shared column definitions in [`README.md#shared-crosswalk-table-columns`](README.md#shared-crosswalk-table-columns).

| Seam | Current code artifacts | Current semantic status | Authority boundary correct? | Enforcement point correct? | Proven by smoke/e2e? | Refactor action | Sibling seams that must stay in context |
|---|---|---|---|---|---|---|---|
| RetainedWorkerRuntime | R0 object construction/exact target resolution plus B3.2a admission key, registry, fingerprint, slot, registration-head, transport-claim, routability, and terminal truth in `agent_runtime/retained_worker_runtime.rs`; HSA-owned R0 reservation/application maps and bounded admission-storage capability in `agent_runtime/host_session_authority`; typed proof production/transport/validation in `orchestrator_world_dispatch.rs`, `async_repl.rs`, `transport-api-types`, and `world-service`; participant/session manifests in `agent_runtime/session.rs`; remaining compatibility lifecycle mutations across `async_repl.rs`, `state_store.rs`, and `orchestrator_world_dispatch.rs` | `MislandedWrongModel` | no | no | no | R0 provides only immutable descriptor/resume/worker object construction and exact target read, consuming the HSA-owned atomic lineage/ref registration proof without owning authority mutation or live state. It is review-clean component proof with no production ingress and does not promote this seam. B3.2a has now added only the fixed durable creation/admission record and both production Spawn bridges needed before the joint closeout: a RetainedWorkerRuntime-owned keyed commitment over the complete canonical request/authority/runtime/policy plan, crash-stable admission-key retention, atomic cap slot, stable participant/bootstrap run, and one serialized per-session registration head prevent over-admission, changed-byte retry, and stale-revision races without extending HSA commitment domains. The existing `SlotReserved` state may remain queued with no head; current-head reconciliation never promotes another slot, and only exact complete re-presentation of the earliest request can acquire the released head. No request/prompt/payload preimage is persisted. A transport-neutral equality proof crosses the shell transport-api request; the real `Service::execute_stream` member branch first completes strict `Some(exact proof)` validation, durably exact-adopts the HSA-bound world through B3.2a-WA, and only then passes the unchanged typed request into launch-boundary revalidation. Admission is nonterminal before transport, routable only on exact Registered truth, and terminal only on exact B0 terminal truth, with ambiguous interruption retained and counted live. Both activated paths avoid legacy session/participant writers and supply canonical live-count/routability reads. B3.2a plus B3.2a-WA are review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda` without promoting this seam. The remaining B3.2 packet later owns durable abandoned-admission resolution and restart reconciliation, including exact terminal handling of abandoned `SlotReserved`/`AuthorityRegistrationHead` state and reconciliation of post-R0 or transport-ambiguous state before cap release; it also owns full worker manifests, accepted-turn lifecycle, park, cancel, stop, fork, inspect, invalidation, and broader reconciliation. B4 owns only the exact user/tool-facing pending-admission inspect/cancel verb and outcomes. Accepted task/turn identity stays in WorldWorkReceiptRegistry and host posture stays separate. | HostSessionAuthority; WorldWorkReceiptRegistry; WorldWorkExecutionSupervisor; WorldWorkerMessagingProtocol |
