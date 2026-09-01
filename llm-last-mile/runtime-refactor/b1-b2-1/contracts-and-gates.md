**Kind:** contracts and gates
**Stable ID:** `B1-B2.1-family`
**Canonical for:** B1/B2.1 family runtime carrier, adapter, retained prerequisites, acceptance, supervisor, and differential contracts
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** composite of the seven preserved root compatibility spans listed in the extraction ledger; mixed future-family regions are excluded
**Supersedes:** canonical ownership of the extracted source bodies; source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# B1/B2.1 family contracts and gates

## 2A. Runtime event identity and ordering carrier

B0 extends the existing runtime stream family; it does not create a parallel transport. The
runtime producer assigns identity before emission. Host decoders, receipt code, supervisors,
messaging code, and the ledger consume those fields unchanged.

```rust
struct RuntimeFrameIdentityV1 {
    schema_version: u32,          // exactly 1
    stream_id: String,            // stable for one accepted runtime stream
    frame_sequence: u64,          // starts at 1; strictly monotonic and gap-free
}

struct RuntimeEventIdentityV1 {
    event_id: String,             // stable for one logical semantic event
    event_sequence: u64,          // starts at 1; strictly monotonic within stream
}

struct RuntimeTerminalIdentityV1 {
    terminal_event_id: String,
    terminal_event_sequence: u64,
}
```

Every `ExecuteStreamFrame` carries `RuntimeFrameIdentityV1`. Semantic `Event` frames additionally
carry `RuntimeEventIdentityV1`; the terminal `Exit` frame carries both a semantic event identity
and `RuntimeTerminalIdentityV1` with equal ID/sequence. `Start`, stdout, stderr, event, and terminal
frames participate in the same frame sequence. `Error` is either a typed semantic terminal frame
with exact terminal identity or an observation/transport error that cannot prove run completion.

B0 rules:

1. `stream_id`, frame sequence, event ID/sequence, and terminal identity are runtime-produced;
   the host never synthesizes them from arrival order, timestamps, payload hashes, span IDs, EOF,
   or process state.
2. Re-emitting the same logical frame/event after retry or reconnect preserves every identity.
   The producer never originates a second meaning at the same sequence position.
3. Frame and event sequences are positive and strictly monotonic. An exact identity with identical
   canonical bytes is valid replay and B2.1 consumes it as a no-op. A gap, reorder, conflicting
   duplicate, or frame after terminal is protocol-invalid and B2.1 rejects it; that consumer-side
   enforcement is not B0 ownership.
4. The terminal frame is the final semantic event and names its exact event ID/sequence. Stream
   exhaustion without it is not completion.
5. B0 owns only the carrier. B1 owns acceptance, B2.1 owns durable observation/dedupe/restart,
   B3.1 owns retained-event semantics, and C1 owns obligation materialization/completeness.

### B2.1-3 exact producer replay transport

World-service may add one bounded process-memory registry and endpoint for exact B0 frame replay.
The registry is a producer transport cache, not durable state and not a supervisor. A replay request
must supply all of:

```rust
struct ExecuteStreamReplayRequestV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    stream_id: String,
    after_frame_sequence: u64,
}
```

The registry binds that request to the exact accepted producer stream. It returns only frames with
the same `stream_id` and `frame_sequence > after_frame_sequence`, in strict sequence, with the
original identity and canonical bytes, and may then continue that exact live stream. It cannot
enumerate streams or accept lookup by session, backend, process, endpoint, partial identity, or any
fuzzy match. The implementation must set explicit maximum retained streams, maximum frames per
stream, and maximum bytes per stream. Overflow, missing or expired retention, cursor-ahead state,
identity mismatch, corruption, reorder, conflict, and unavailable replay fail closed.

The replay registry is lost on world-service restart unless a later separately authorized durable
producer contract proves otherwise. It must not be described as durable across that restart. It
cannot create or modify B1 acceptance, a supervisor claim/cursor/journal, a terminal result,
cancellation, lifecycle state, an obligation or Complete cut, or success. ReceiptRegistry remains
immutable acceptance authority; `RuntimeEventTransport` transports producer facts only.


### B1/B2.1 bounded read-only dispatch-authority adapter

The implementability audit selected **Case B**. The adapter became implementation-authorized after
the independently review-clean docs-only control-pack correction and the review-clean A1.2a,
A1.2a-WB, A1.2a-S, B1/B2.1-R0, B3.2a, B3.2a-WA, B1 receipt-core, and B2.1 supervisor-core
prerequisites were present. Its implementation is part of B1/B2.1-0, not the later joint closeout.
A1.1e alone could read exact current authority but could not create it; A1.2a,
A1.2a-WB, and A1.2a-S now satisfy the bounded ordinary-internal-host creator/adopter portion of
that sequence, B1/B2.1-R0 is independently review-clean, and B3.2a plus B3.2a-WA are independently
review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The recovered B1/B2.1 cores
and B1/B2.1-0 implementation are review-clean through `6436289f`, `c519024b`, `de727091`,
`717579b0`, and `83101dcb`. The later joint production closeout is complete on the frozen
2026-08-03 source without additional product bytes, and B3.1, C1, and the bounded internal A1.2b
packet are complete on the bound Tuesday, August 4, 2026 candidate. The then-current historical
gate was `AUTHORITY_REQUIRED:R3_RESUME`; its former macOS-parity replacement is
superseded as the global schedule by `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY`. The historical
prepared type also combined
B-owned accepted/inspection routing with retained-worker admission data that has no canonical
live-state representation.

The authority/retained branch is **A1.2a → A1.2a-WB → A1.2a-S → B1/B2.1-R0 → B3.2a → B3.2a-WA**. The independently
recovered B1 receipt → B2.1 supervisor branch first joins it at **B1/B2.1-0**, after which the hard
order is **joint B1/B2.1 production closeout → B3.1 → C1 → A1.2b**. R0 consumes no receipt or
supervisor datum; neither B core is therefore a false prerequisite of R0.

`tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1` is a read-only adapter and
owns no durable state. For its active-task branch only, it consumes the existing bound authority
capability or the single authorized trusted open-and-bind conversion and resolves:

1. the exact current HostSessionAuthority;
2. the exact immutable `WorldWorkAcceptanceRecordV1`; and
3. the exact `WorldWorkExecutionSupervisor` claim, durable cursor, interruption/unresolved state,
   and immutable terminal closeout when present.

It accepts only one complete equality join across physical store ID and home, orchestration
session, caller participant and backend, world ID and generation, task/active-run identity,
acceptance record identity, and stream identity where applicable. It returns only a bounded
projection into the existing tool-invocation result and performs no mutation. Absent, stale,
ambiguous, cross-session, backend-mismatched, world-mismatched, task-reused, incomplete, or
conflicting truth fails closed. Unknown task, stale linkage, caller/backend mismatch, world-binding
mismatch, nonterminal active work, exact terminal work, and producer-replay-unavailable or otherwise
unresolved observation remain distinct existing outcomes; they cannot be collapsed into generic
not-found or stale linkage.

The adapter cannot create or mutate authority, acceptance, supervisor state, or any legacy active-
task record; treat receipt existence alone as active, running, or terminal truth; derive lifecycle
from PID, guard, waiter, helper, socket, timeout, EOF, readiness, process state, or caller presence;
reinterpret retained work as an ephemeral active task; or bypass the full dispatcher. Its retained-
worker branch, retained target selection, retained error categories, and retained Inspect/Cancel/
Stop, continue-fork, fork, and Spawn behavior remain unchanged. Its function signature and callers
remain unchanged. The existing shared pre-`match` legacy caller/world resolution may be
mechanically relocated into the retained-worker branch with identical inputs, order, errors, and
results so the active-task branch no longer executes it. That partitioning grants no retained
semantic change and no parallel resolver. The approved GitNexus HIGH impact is exactly 12 direct callers, four affected
execution processes, and 14 impacted symbols for this active-task branch. Any additional production
symbol, signature/caller change, retained-branch change, or new HIGH/CRITICAL process family is a
stop requiring a new control-pack correction.

The minimal prerequisite contract is:

1. **A1.2a — current-authority establishment/read prerequisite:** use the production
   HostSessionAuthority protocol to perform the strict greenfield-only V1-to-V2 root upgrade, then
   verify greenfield absence, issue/claim/apply one Start, create initial authority, and exact-join
   retry. Its read result joins the applied Start descriptor and accepted-home bound capability to
   exact session/caller/lineage/workspace/world/revision/policy truth. It does not implement a
   non-greenfield upgrade, Attach, ResumeOneTurn, startup/post-turn reconciliation, obligation-cut
   consumption, correlation supply to world work, or public consumer adoption.
2. **A1.2a-WB — Host/world-binding validation correction:** preserve exact descriptor/launch-scope
   equality, accept `Host + None`, `Host + Some(exact)`, and `World + Some(exact)`, and reject
   `World + None` or any malformed binding without mutation during Start issuance. Application
   persists the exact accepted binding, and `HostSessionAuthority::resolve_current_exact` enforces
   the identical matrix and returns that binding unchanged. Exact present binding is session
   authority, never host-participant placement. The packet changes no schema, canonical bytes,
   fixtures, migration, compatibility path, or already-persisted object; all other facade behavior
   remains outside scope.
3. **A1.2a-S — bounded internal Start adoption prerequisite:** make
   `prepare_host_orchestrator_runtime_from_resolved` construct only an unpersisted
   `GreenfieldHostStartProposalV1` carrying exact store/home/workspace/descriptor/policy/shell
   observations but no authoritative session/participant/run identity. The proposal is a distinct
   type, never a variant or partially initialized form of `PreparedAgentRuntime`. On the real
   dormant-host launch path, `dispatch_targeted_follow_up_turn` first obtains the exact optional
   world binding, then calls `apply_greenfield_host_start_from_authority`; that adapter derives the
   A1.2a issuer key from the greenfield store identity plus the complete canonical Start plan and,
   before transport or any legacy write, applies/exact-joins A1.2a Start. Only that result supplies
   identities and constructs one fully materialized existing-shape `PreparedAgentRuntime` plus its
   compatibility session/participant view. `PreparedAgentRuntime` itself, its fork/member
   constructors, and its remote-member consumers do not acquire a pending variant or change
   semantics. The new runtime carries the exact bound capability in an immutable shared
   `RuntimeAuthorityContext::Bound` inside `RuntimeOrchestrationContext`; no context exists before
   application. The hidden-owner constructor may initialize only the `Legacy` variant and receives
   no adoption semantics. The authority-managed branch of
   `start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx` performs no
   activated-store legacy session/participant/snapshot write; it leaves startup ownership Pending
   and treats launch, readiness, endpoint, process, prompt, and event state as observations. It does
   not adopt hidden-owner plans, public Start/Attach/Resume, startup-result reconciliation, or
   post-turn behavior.
4. **B1/B2.1-R0 — canonical retained-target protocol prerequisite:** add the smallest
   bounded registration handshake in which RetainedWorkerRuntime creates the immutable descriptor,
   participant-specific resume handle, and retained-worker object graph from a caller-fixed
   participant plan. HostSessionAuthority validates that plan and its exact commitment/scope and
   atomically appends exactly the retained participant to
   authoritative lineage, adds its typed worker ref, advances authority revision, and persists the
   non-transition `RetainedWorkerAuthorityRegistrationV1` proof consumed by `resolve_exact`.
   Exact retry joins; stale or conflicting revision fails closed. Registration cannot change
   active caller, posture, workspace, world, current policy, origin, or any unrelated ref. R0 has
   no production ingress caller and cannot itself count as full-dispatcher proof. It cannot use a
   test-only fixture or process-local map, and it adds no message, accepted-turn observation, active-turn,
   park/cancel/stop/fork, or live-admission semantics.
5. **B3.2a — retained creation/admission bridge prerequisite:** route both real Spawn adapters—the
   direct dispatcher path and the live internal-toolbox retained-runtime path—through the same
   authority-bound `SpawnWorldWorker` preparation before generic compatibility
   preparation, atomically count/reserve a durable participant slot across processes before R0,
   pass that fixed participant plan to R0, exact-join it before transport, commit the final proof
   plus the same fixed RetainedWorkerRuntime admission record, and
   make the remote retained-start path validate the exact
   session/request/participant/backend/protocol/world proof rather than invoke any activated-store
   legacy session/participant writer. The admission record is nonterminal before transport,
   routable only after exact Registered truth, and terminal only after exact B0 terminal truth.
   Every ambiguous interruption remains nonterminal and counted live, so existing spawn admission
   narrows without HSA-ref counting or compatibility liveness. B3.2a has no accepted-turn,
   message, park/cancel/stop/fork, abandonment protocol, or final lifecycle semantics. A queued
   `SlotReserved` record with no head is valid; head acquisition requires complete exact
   re-presentation of the earliest request, while current-head reconciliation/release never
   promotes another slot and persists no request/prompt/payload preimage.
6. **B3.2a-WA — exact bound-world ownership adoption prerequisite:** after authority-managed proof
   validation and before member creation, exact-join the HSA session/world ID/generation/policy,
   participant, project/world-spec identity, and current generic-world metadata. The runtime-family/
   Linux backend alone may durably publish `GenericExactBoundWorld ->
   SharedSessionOwnerExactBoundWorld`, preserving world ID and generation. Exact retry joins without
   rewrite; conflicts, missing/corrupt/ambiguous metadata, or partial publication fail closed with no
   alternate world. This changes no HSA, RetainedWorkerRuntime, transport-claim, routability, or
   terminal truth and proves no launch. Compatibility `None` stays unchanged. It adds no world-api
   field, schema version, side table, shell rebinding, request/prompt persistence, PID/liveness
   authority, recovery protocol, or non-Linux proof.
7. **B1/B2.1-0 — action-scoped dispatch preparation:** accept the A1.2a/A1.2a-WB/A1.2a-S,
   B1/B2.1-R0, B3.2a, and B3.2a-WA typed read results plus B1 receipt and B2.1 supervisor truth through a
   caller-supplied bound capability or one explicitly authorized trusted open-and-bind conversion;
   build a B-owned prepared view only for RunWorldTask, ordinary retained ContinueWorldWorker, and
   ephemeral accepted-task Inspect/Cancel/Wait; and leave `WorkerContinueForkCommand`, retained
   Inspect/Cancel/Stop and fork admission/lifecycle calculation on unchanged compatibility paths.
   It leaves the already-landed B3.2a spawn creation/admission bridge unchanged and does not alter spawn
   policy, steering, outcome, or lifecycle semantics. It neither uses
   nor replaces that legacy count and cannot claim it as authority.
8. After those prerequisites, validate the typed request, require session/caller/world agreement,
   exact-join receipt and supervisor truth for accepted work, and fail closed on any absent,
   incomplete, stale, or conflicting authority/binding/acceptance/claim.

R0's non-transition proof is a distinct HostSessionAuthority record, not an application-journal
entry and not a new transition mode:

```rust
enum RetainedWorkerAuthorityRegistrationRequestStateV1 {
    Reserved,
    Applied {
        authority_revision_after: u64,
        authority_record_commitment_after: AuthorityObjectCommitmentV1,
    },
}

struct RetainedWorkerAuthorityRegistrationRequestV1 {
    schema_version: u32, // exactly 1
    issuer_request_id: String,
    registration_id: String,
    orchestration_session_id: String,
    authority_revision_before: u64,
    authority_record_commitment_before: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    descriptor_ref_id: String,
    descriptor_commitment: AuthorityObjectCommitmentV1,
    resume_handle_ref_id: String,
    resume_handle_commitment: AuthorityObjectCommitmentV1,
    retained_worker_ref_id: String,
    retained_worker_commitment: AuthorityObjectCommitmentV1,
    current_policy_ref: AuthorityObjectRefV1,
    world_binding: WorldBindingV1,
    registered_at: TimestampV1,
    state: RetainedWorkerAuthorityRegistrationRequestStateV1,
}

struct RetainedWorkerAuthorityRegistrationV1 {
    schema_version: u32, // exactly 1
    issuer_request_id: String,
    registration_id: String,
    orchestration_session_id: String,
    authority_revision_before: u64,
    authority_record_commitment_before: AuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    authority_record_commitment_after: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    authoritative_lineage_commitment_after: AuthorityObjectCommitmentV1,
    descriptor_ref: AuthorityObjectRefV1,
    resume_handle_ref: AuthorityObjectRefV1,
    retained_worker_ref: AuthorityObjectRefV1,
    current_policy_ref: AuthorityObjectRefV1,
    world_binding: WorldBindingV1,
    registered_at: TimestampV1,
}
```

The R0 caller must carry the existing validated retained-creation ingress request/idempotency ID.
The API domain-separates it with the closed `retained-worker-registration` operation tag before
using it as `issuer_request_id`; this key cannot be reused as a transition issuer key, and R0 may
not create a replacement after entering the operation. Its caller also supplies one exact
participant plan; the production caller is the B3.2a `SlotReserved` record. Before any object file
is published, one root CAS validates that participant plan, the complete proposed canonical object
bytes, and exact scope, then exact-joins or creates its request-index reservation. Under the root
lock it generates/collision-checks the registration and planned object IDs and fixes all three canonical
commitments, expected authority revision/commitment, current policy, world, and `registered_at`.
An exact retry with the same issuer ID and complete bytes joins; reuse with any changed field fails
before object publication. A crash before reservation publication leaves no object or semantic
change and may retry from the same ingress request. A crash after reservation publication rereads
these fixed identities rather than generating replacements.

Only after that reservation is durable may RetainedWorkerRuntime publish the exact descriptor,
resume-handle, and retained-worker bytes under the three reserved IDs. Their existence grants no
authority. The final root CAS verifies every byte/commitment and the unchanged reservation, adds
the three object-index entries, performs the authority lineage/ref mutation, inserts the immutable
registration journal, and advances the request state from `Reserved` to `Applied` atomically.
`StateRootV2.retained_worker_registration_journal` keys the proof by exact `registration_id`.
That final CAS must verify the pre-revision/commitment, current policy and world,
descriptor backend/protocol/world execution scope, participant-specific resume identity, and the
retained object's session/participant/world/descriptor/resume/policy refs; append the participant
exactly once to authoritative lineage; append the worker ref exactly once; increment authority
revision exactly once; and commit the new authority hash plus registration proof. Active caller,
posture, workspace/store, world, policy, origin, attach ref, internal resume refs, and existing
lineage/worker refs remain byte-for-byte unchanged. A reserved object file present before a
losing/crashed final CAS is verified and reused only through its exact Reserved request; without
that reservation it is an ordinary orphan subject to reconciliation. It never becomes authority by
existence alone.

`resolve_exact` and current-authority proof validation accept the unique highest matching proof
from exactly one of: an initial or post-turn transition application phase, a terminal startup
application phase, or this registration journal. An Accepted startup result does not advance
authority and supplies no authority proof; the already-current initial/R0 proof remains current.
For every proof source, before/after revisions and commitments must form the unique contiguous
history for that session, and the highest committed revision must equal the current authority.
Missing, ambiguous, behind, skipped, substituted, or conflicting proof fails closed. A retry after
the final CAS, including a lost response, joins the `Applied` request index to the identical journal
and current authority and returns the original result. Reuse with different bytes, duplicate
participant/ref under a different issuer or registration ID, or a stale expected revision fails.
This operation creates no live/terminal state and cannot satisfy retained admission
counting. When Start startup ownership is still Pending, each successful registration also becomes
one link in the only permitted application-revision-to-current-revision ancestry. It does not
rewrite the Pending expected revision or constitute startup acceptance; A1.2b must later validate
the complete contiguous chain under the preserved root
[`Lifecycle, retry, and fail-closed rules`](../04-contracts-and-gates.md#lifecycle-retry-and-fail-closed-rules)
for the A1.2 durable host-intent protocol.

B3.2a adds a separate RetainedWorkerRuntime-owned creation/admission record; it is neither
HostSessionAuthority nor accepted-turn Supervisor truth:

```rust
enum RetainedWorkerAdmissionCommitmentAlgorithmV1 {
    HmacSha256,
}

struct RetainedWorkerAdmissionCommitmentV1 {
    schema_version: u32, // exactly 1
    algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1,
    key_id: String,
    digest_hex: String,
}

struct RetainedWorkerAdmissionRegistrationV1 {
    registration_id: String,
    retained_worker_ref: AuthorityObjectRefV1,
}

enum RetainedWorkerAdmissionStateV1 {
    SlotReserved {
        slot_sequence: u64,
        reserved_at: TimestampV1,
    },
    AuthorityRegistrationHead {
        authority_revision_expected: u64,
        authority_record_commitment_expected: AuthorityObjectCommitmentV1,
        head_acquired_at: TimestampV1,
    },
    PreTransportNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
    },
    TransportClaimedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        transport_claim_id: String,
        claimed_at: TimestampV1,
    },
    Routable {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: String,
        registered_frame_sequence: u64,
        registered_event_id: String,
        registered_event_sequence: u64,
        registered_at: TimestampV1,
    },
    InterruptedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: Option<String>,
        last_frame_sequence: Option<u64>,
        interrupted_at: TimestampV1,
    },
    Terminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: String,
        terminal_frame_sequence: u64,
        terminal_event_id: String,
        terminal_event_sequence: u64,
        exit_code: i32,
        terminal_at: TimestampV1,
    },
    RejectedBeforeRegistration {
        reason: String,
        rejected_at: TimestampV1,
    },
}

struct RetainedWorkerAdmissionRecordV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1,
    orchestration_session_id: String,
    admission_authority_revision: u64,
    admission_authority_record_commitment: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    bootstrap_run_id: String,
    backend_id: String,
    protocol: String,
    world_binding: WorldBindingV1,
    current_policy_ref: AuthorityObjectRefV1,
    current_policy_revision: String,
    max_live_retained_workers: u64,
    state: RetainedWorkerAdmissionStateV1,
    record_revision: u64,
}
```

The corrected queued-promotion contract uses this existing V1 schema and the existing
`SlotReserved` state. It adds no schema version, persisted request/preimage object, side table,
secret domain, or lifecycle state merely to encode "waiting for retry." Queued `SlotReserved`
records with no current `AuthorityRegistrationHead` are a valid registry state.

`canonical_spawn_fingerprint` is a RetainedWorkerRuntime-owned, domain-separated keyed HMAC. It
does not use, extend, pin, or participate in the HostSessionAuthority commitment-key registry. Its
exact HMAC input is the following byte sequence, where `len64` is unsigned 64-bit big-endian and
each `canonical_*` member is the exact `CanonicalJsonV1` encoding of the complete strict typed
value named by that member:

```text
"substrate.retained-worker.admission.hmac-input.v1\0"
 || len64("substrate.retained-worker.admission.spawn.v1")
 || "substrate.retained-worker.admission.spawn.v1"
 || len64(authority_store_id) || authority_store_id
 || len64(issuer_request_id) || issuer_request_id
 || len64(canonical_validated_spawn_request) || canonical_validated_spawn_request
 || len64(canonical_exact_current_authority) || canonical_exact_current_authority
 || len64(canonical_descriptor_and_runtime_plan) || canonical_descriptor_and_runtime_plan
 || len64(canonical_policy_and_admission_cap) || canonical_policy_and_admission_cap
 || len64(retained_participant_id) || retained_participant_id
 || len64(bootstrap_run_id) || bootstrap_run_id
```

The first canonical member is the complete validated `SpawnWorldWorker` request, including the
prompt/payload bytes in its typed canonical location. The second contains the exact caller,
session, revision, authority-record commitment, lineage, workspace, world, and current-policy
observation returned by the bound read. The third contains the complete descriptor and runtime
launch plan. The fourth contains the exact policy identity, current policy revision,
`max_live_retained_workers`, and admission inputs. Unknown or omitted fields, alternate JSON,
delimiter concatenation, presentation serialization, and digesting a prompt/payload separately are
forbidden. The stored digest is lowercase-hex `HMAC-SHA-256(admission_key, bytes_above)`.

RetainedWorkerRuntime owns one separate admission commitment-key envelope and registry header per
authority store through its opaque fixed-registry physical capability. The envelope binds exact
schema version, authority-store ID, key ID, creation timestamp, `HmacSha256`, and 32 OS-CSPRNG
bytes. First initialization under the root lock publishes the owner-only/no-follow key file through
a same-directory temp, `fsync`, no-replace rename, and directory `fsync`, then commits and `fsync`s
the registry header naming that key. A crash before the header commit leaves an unregistered key
orphan that locked recovery deletes only after proving the registry is absent; a committed header
with a missing, malformed, wrong-store, wrong-key, or unreadable envelope is corruption and fails
closed. Concurrent initialization exact-joins only the same committed header and key identity.

B3.2a performs no admission-key rotation, retirement, or deletion. Its single committed key remains
verification-capable for every live, interrupted, terminal, rejected, tombstoned, or otherwise
retained admission record that names it, so crash retry and long-lived exact joins cannot lose
verification. A later rotation/retention packet must add a locked complete reachability scan over
the separate admission registry and may retire a key only after zero records reference it. HSA key
rotation and reachability never inspect or control this registry. Raw key bytes, unredacted
canonical input bytes, and prompt/payload bytes are never persisted in the admission record, a side
table, a sealed-preimage object, logs, traces, diagnostics, transport proof, or any other durable
artifact; they exist only in the ordinary in-memory scope of the typed call that received them.
They are never reconstructed from adjacent state or inferred from the stored digest. Diagnostics
may report only key ID and equality/mismatch. Every exact join and every state advance that depends
on the Spawn fingerprint receives the complete typed canonical input, recomputes the framed HMAC
bytes above, and verifies the stored commitment. Digest equality without that complete input is
never promotion or state-advance authority. Issuer reuse
with changed prompt, payload, caller, authority, descriptor, policy, cap, participant, or run ID
conflicts before any additional mutation.

The fixed admission registry is persisted through an opaque physical capability bound to the same
opened root/store identity; RetainedWorkerRuntime alone validates and changes its semantic bytes.
It is keyed by exact `(orchestration_session_id, retained_participant_id)`, and each
`issuer_request_id` may name exactly one record. Each post-R0 registration ID may appear in exactly
one state value. The root lock orders registry publication with R0, but the files are not falsely
described as one cross-file atomic rename.

Before R0 reservation, B3.2a resolves the A1.2a-S bound exact authority and performs one locked registry
transaction that revalidates the current policy identity/cap, joins every post-R0 record to
current HSA registration truth, counts every `SlotReserved`, `AuthorityRegistrationHead`,
`PreTransportNonterminal`, `TransportClaimedNonterminal`, `Routable`, or
`InterruptedNonterminal` record as live, enforces the cap
including all existing slots, allocates/collision-checks participant and bootstrap-run IDs,
computes the complete canonical Spawn fingerprint, assigns the next monotonic session slot
sequence, and persists one immutable `SlotReserved` identity. `Terminal` and
`RejectedBeforeRegistration` do not count. This check and
slot creation are one cross-process admission CAS; the existing process-local bootstrap guard may
remain only as a same-process fast reservation. A missing, extra, stale, cross-session/world,
duplicate, or commitment-mismatched R0/registry join fails closed rather than being omitted from
the count.
`retained_worker_refs.len()`, legacy `authoritative_live`, PID, helper, socket, endpoint, and
process-local maps are never count inputs. Existing action/backend/session/world/policy checks and
the exact `max_live_retained_workers` comparison run against this count before a new reservation.

The registry also owns one durable per-session R0 head. At most one nonterminal record may be
`AuthorityRegistrationHead`, while any number permitted by the cap may remain `SlotReserved` when
no head exists. Head acquisition is request-driven. An exact retry first finds its existing
`SlotReserved` record, re-presents and verifies the complete typed Spawn request, prompt/payload,
admission-time authority observation, descriptor/runtime plan, policy/cap, retained participant,
bootstrap run, and stored HMAC, and proves that this record is the lowest-sequence queued slot. A
root-locked transaction may advance only that record to `AuthorityRegistrationHead` and fix the
current HSA revision/commitment only when no head exists. The retry also validates the complete
admission-to-current ancestry: every contiguous intervening proof must be an exact R0 registration
proof, while active caller, workspace, world, policy, origin, and every unrelated ref remain
unchanged. Any transition proof, gap, ambiguity, changed bound field, changed canonical byte, later
slot, or digest-only presentation conflicts without mutating either the requested or earlier slot.

Current-head reconciliation is a different durable transaction. The same head request must
re-present the complete canonical fingerprint input, exact-join its applied HSA proof, and may then
advance only its own record to `PreTransportNonterminal`, thereby releasing the head. It must not
promote any queued slot in that transaction. A future queued request acquires the released head only
when that earliest request is itself re-presented and passes the complete verification above. A
crash between the HSA CAS and current-record advancement leaves the same head in place; its exact
retry joins the proof and advances only that record. A crash after advancement may reopen with no
head and queued slots, which is valid and remains stable until an exact earliest retry.

A later request cannot overtake an earlier queued slot merely because it reappears first. PID,
caller presence or loss, helper/process liveness, timeout, EOF, socket/endpoint state, transport
state, and observer loss cannot acquire, replace, renew, or steal the head. An abandoned earliest
slot remains conservatively live and blocks every later slot until the same canonical request
retries or a later explicitly owned lifecycle/control protocol resolves it. B3.2a adds no such
abandonment/control protocol.


### B3.2a-WA `ExactBoundWorldOwnershipAdoptionV1`

The first bounded Linux live Spawn proof reached durable admission, R0 registration, the unique transport
claim, and typed launch-proof validation, then failed closed before process creation because the
HSA-bound generic REPL world and the world-service-created shared-owner world had different IDs.
Creating another world is not a valid repair: HSA already owns the exact session binding and neither
world-service nor backend metadata may replace it. The docs-first correction therefore made the
internal operation contract B3.2a-WA, required before B3.2a closeout and B1/B2.1-0; the recorded
result in `05` now satisfies that prerequisite through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`:

```text
ExactBoundWorldOwnershipAdoptionV1

GenericExactBoundWorld
    -> SharedSessionOwnerExactBoundWorld
```

This is an internal service/backend call contract. It is not a new `world-api` field, request
variant, persisted wire-schema version, authority object, admission state, side table, or lifecycle
state. The existing generic/shared world metadata shape remains unchanged.

Ownership remains separated:

1. `HostSessionAuthority` alone owns the exact durable orchestration-session world ID and
   generation and the policy binding accepted with that authority. Adoption consumes this tuple as
   immutable evidence and cannot create, repair, revise, or clear it.
2. `RetainedWorkerRuntime` alone owns admission, R0-registration join, the unique transport claim,
   routability, interruption, and terminal truth. Adoption changes none of those bytes and cannot
   turn an exact joined claim into permission to resend.
3. World-service and the runtime-family/Linux world backend own only physical realization and
   durable physical ownership metadata. They may adopt the exact already-bound generic world; they
   may not choose a replacement world when that exact world exists.
4. `WorldDispatchControl`, the shell episode, helpers, and compatibility projections consume the
   result only. They do not authorize or perform adoption.

The closed transient input/evidence set is:

```rust
struct ExactBoundWorldOwnershipAdoptionV1 {
    orchestration_session_id: String,
    world_id: String,
    world_generation: u64,
    participant_id: String,
    authority_managed_launch_proof: RetainedWorkerLaunchAuthorityProofV1,
    policy_ref_id: String,
    policy_revision: String,
    policy_snapshot_hash: String,
    canonical_project_identity: String,
    canonical_world_spec_identity: String,
    target_shared_session_owner_id: String,
    adoption_correlation_id: String,
}
```

The sketch freezes semantic members, not a Rust/public/wire type requirement. The implementation may
use a narrower private borrowed type composed from existing values. The non-secret adoption
correlation is equality-only call-scope evidence and is not a prompt/request preimage or ownership
source. Raw prompt, request, payload, credentials, or secrets are excluded from the operation,
metadata, errors, logs, traces, and diagnostics.

The operation may run only when all preconditions hold under the backend's trusted shared-owner
lock:

1. The authority-managed carrier is `Some(exact proof)` and its complete existing strict validation
   has succeeded before adoption. Compatibility `None` cannot enter this operation and cannot be
   reinterpreted as a managed request.
2. Exact session, world ID, generation, participant, policy ref/revision/snapshot hash, project, and
   world-spec identity match the validated proof/request and the durable/current backend evidence.
3. The selected physical world is the exact HSA-bound world, has the exact supported generic owner
   shape, and has complete, contiguous, readable metadata. The implementation resolves by exact
   world identity, never by project/spec compatibility alone.
4. Generic metadata has no shared owner fields, foreign owner, conflicting generation, stale policy,
   substituted project/spec, or ambiguous/partially published ownership state.
5. If the exact world is already a shared owner, every owner/session/world/generation/policy/project/
   spec member must match and the operation joins without rewrite. Any mismatch fails closed.
6. Missing, corrupt, forked, discontinuous, substituted, unreadable, unsupported, or multiply
   matching evidence fails closed without metadata mutation or alternate-world creation.
7. Durable adoption completes before member process creation. Adoption does not prove transport
   submission, member creation, Registered, routability, terminal success, or any later lifecycle
   outcome.

First adoption changes only the existing world's ownership metadata from exact generic to exact
active shared-session owner while preserving its world ID, generation, root, project, cgroup,
network, overlay, filesystem mode, policy snapshot, and all other realization semantics. Publication
uses the existing trusted lock and recognized temporary-file, file-`fsync`, atomic rename, and parent
directory-`fsync` conventions. No success may be returned before directory durability. The exact
crash/reopen matrix is:

1. **Before temporary-file creation or persistence:** the durable final remains authoritative
   generic metadata. Reopen observes no adoption; an exact retry may begin first publication.
2. **After a temporary write but before its file `fsync`:** temporary-file presence is never
   semantic ownership. Under the trusted lock, an operation-bound temp with incomplete bytes is
   removed as non-semantic cleanup and the parent directory is re-`fsync`ed; a complete exact temp
   is fully revalidated and file-`fsync`ed before it can proceed. An unrecognized or conflicting
   temp fails closed without deletion or ownership mutation.
3. **After temporary-file `fsync`, including immediately before atomic rename:** the recognized
   candidate remains non-semantic. Reopen revalidates the authoritative generic final and the
   complete exact candidate under the trusted lock, then may atomically rename that candidate and
   parent-directory-`fsync` it. Presence or file durability alone never adopts the world.
4. **After atomic rename but before parent-directory `fsync`:** reopen may observe either the
   original exact generic final (the rename did not survive) or the exact adopted final (the rename
   survived without proven directory durability). Under the trusted lock, the exact generic final
   may restart publication from step 1, including step-2/3 handling of any recognized temp; the exact
   adopted final must be completely revalidated and its file plus parent directory re-`fsync`ed
   before join/success. Missing, conflicting, multiply matching, or otherwise ambiguous final/temp
   evidence fails closed without ownership mutation. Neither observed shape proves adoption before
   parent-directory durability.
5. **After parent-directory `fsync` but before the registry/root response:** the exact final tuple
   is durable adopted ownership. Exact retry joins it byte-for-byte without rewrite and without
   claiming member launch.

Exact retry is exercised after every state above. Recognized temp cleanup or republication never
changes HSA, RetainedWorkerRuntime, or semantic ownership before the exact final rename plus required
directory durability. Conflicting retry leaves the exact durable bytes unchanged.

Conflicting retry leaves the exact durable bytes unchanged. PID, timeout, caller disappearance,
helper state, socket or endpoint state, EOF, process liveness, prompt content, shell-local state, and
compatibility projections cannot acquire, steal, replace, renew, or clear ownership.

Compatibility and scope are closed:

- authority-managed retained Spawn uses adoption only after `Some(exact proof)` validation;
- existing compatibility `None` behavior and ordinary world execution remain unchanged;
- world filesystem, overlay, network, caging, capability, policy, discovery, and write-sync
  behavior remain unchanged;
- Linux proof makes no macOS or Windows claim;
- runtime implementation is authorized only in `crates/world-service/src/service.rs`,
  `crates/world/src/lib.rs`, and `crates/world/src/session.rs`, plus focused colocated or existing
  integration tests needed to prove this contract;
- shell/HSA rebinding, `crates/world-api/src/lib.rs`, schema versions, alternate side tables,
  unrelated world lifecycle refactors, resend/recovery, and abandonment/cancellation remain outside
  scope.

`RG-WORLD-ADOPT-01` was the blocking regression gate and is now satisfied for the bounded Linux
packet through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. Its permanent proof requires exact same-ID/generation adoption,
zero alternate-world creation, byte-stable exact retry, conflict-without-mutation coverage across
owner/session/policy/project/spec/generation, every named publication crash boundary, member creation
only after durable adoption, unchanged compatibility `None`, request/prompt-marker absence, and green
world-service/world-backend tests plus Linux doctor, ordinary world execution, and bounded
authority-managed live Spawn proof. The recorded result in `05` supplies that evidence. Passing
this gate does not promote any seam and makes no native macOS or Windows claim.

The production order is exact:

1. `dispatch_orchestrator_world_request` validates the raw request and routes only
   `SpawnWorldWorker` to the B3.2a authority-bound preparation before the still-legacy generic
   prepared-dispatch path. Concretely it calls `prepare_authority_bound_spawn_world_worker` and
   then `spawn_prepared_world_worker`; every other action remains on its prior path until
   B1/B2.1-0. Existing `spawn_world_worker` remains the legacy prepared-entry wrapper and delegates
   its already-prepared bootstrap to the same `spawn_prepared_world_worker` execution body.
2. Existing steering/session/world checks and the same-process fast concurrency reservation run
   through `WorldDispatchSteeringInput` and `WorldDispatchConcurrencyInput` passed to the same
   named functions used by legacy preparation. The locked admission CAS
   then exact-joins or creates the `SlotReserved` record from the
   domain-separated ingress request, complete canonical Spawn fingerprint, participant ID, and
   bootstrap run ID before R0. Concurrent processes
   therefore cannot both observe the same available slot. Exact retry returns the same slot;
   changed bytes conflict. The transport builder must use this participant ID and cannot allocate a
   retry-local UUID.
3. A request re-presentation for an existing `SlotReserved` may acquire the head only if it is the
   lowest-sequence queued slot, no head exists, the complete canonical fingerprint input and stored
   HMAC match, and the full admission-to-current ancestry is R0-only and exact. Changed bytes or a
   later-slot retry conflicts with zero mutation. Creating a slot, reconciling another head, caller
   presence, PID, helper, socket, endpoint, timeout, EOF, process state, or observer state never
   supplies promotion eligibility.
4. Only the durable per-session `AuthorityRegistrationHead` may call R0. Its exact retry
   re-presents the complete canonical input, validates the participant and bootstrap run rather
   than allocating either, then exact-joins or creates its HSA request reservation, publishes
   objects, and commits the final HSA CAS. A separate registry transaction exact-joins that proof,
   advances only the matching current record to `PreTransportNonterminal`, and releases the head;
   it does not promote another slot. No member transport opens before both durable commits. Crash
   after slot publication retries from that slot; crash after R0 commit but before admission
   advancement exact-joins R0 and advances only the matching slot. An unrelated request cannot
   adopt either half. A typed terminal R0 rejection may advance the slot to
   `RejectedBeforeRegistration` only after locked proof that no R0 request, registration, object
   parent, or authority mutation exists; ambiguous/local errors leave the same
   `AuthorityRegistrationHead` live. A proven rejection releases the head without promoting a
   queued slot.
5. Before either adapter opens transport, one registry CAS advances
   `PreTransportNonterminal` to `TransportClaimedNonterminal` and fixes a unique
   `transport_claim_id`; only that CAS winner may send. A concurrent exact join waits for Routable
   or returns the existing bounded in-progress/interrupted compatibility result and never sends a
   duplicate. Crash, caller loss, timeout, or process observation cannot steal or renew the claim;
   ambiguous claim state stays live/nonroutable for later B3.2 reconciliation.
6. B3.2a-WA is the intervening physical-ownership step. The direct dispatcher builder carries
   `RetainedWorkerLaunchAuthorityProofV1` through
   `MemberDispatchTransportRequest` and transport-api `MemberDispatchRequestV1`.
   `Service::execute_stream` first completes the existing strict authority-managed proof validation,
   then invokes `ExactBoundWorldOwnershipAdoptionV1` for the exact HSA-bound world. Only after durable
   same-ID/generation adoption may it pass the exact typed member-dispatch request to
   `MemberRuntimeManager::launch`, whose existing validation remains in force before creating the
   world member. The managed branch never calls generic `AttachOrCreate` first and never creates an
   alternate shared-owner world. The direct stream
   consumer accepts only the exact matching B0 Registered event, persists Routable, returns the
   unchanged Spawn outcome, and hands the remaining body to the registry observer.
7. The live `handle_internal_toolbox_world_dispatch_request` Spawn branch invokes the same
   authority-bound preparation instead of `prepare_orchestrator_world_dispatch`, and a new
   `prepare_member_runtime_startup_from_authority_registration` constructs `PreparedAgentRuntime`
   from the slot/R0 graph without calling `prepare_member_runtime_startup_for_descriptor` or
   allocating a participant, bootstrap run, lease, or descriptor identity. It preserves the
   retained runtime handle/map behavior. `build_member_dispatch_transport_request` carries the
   same typed proof.
8. `start_remote_member_runtime_with_prepared` resolves and validates the exact R0/admission tuple
   against its received session/request/participant/backend/protocol/world values before emitting
   Registered. On this activated path, it invokes none of its legacy
   `persist_participant`/`persist_runtime_snapshots` session-participant writers; their in-memory
   fields and existing emitted events remain episode observations. Missing or mismatched proof
   fails before Registered with no fallback. The pre-activation compatibility path is unchanged.
9. The host accepts Routable only from the exact matching B0 Registered event and persists its
   stream/frame/event identity before returning the existing Spawn receipt. The remaining response
   body is handed to an observer independent of the initiating caller. Only an exact B0 Exit and
   matching terminal identity may persist `Terminal`. Error, EOF, body drop, timeout, observer
   loss, process death, PID/helper/socket posture, or restart uncertainty remains or becomes
   `InterruptedNonterminal`; it stays counted live and cannot become routable by inference.

B3.2a proof must exercise a valid no-head/queued registry after reopen; exact earliest retry with
one and multiple queued slots; later retry and changed prompt, payload, caller, authority,
descriptor/runtime plan, policy/cap, participant, and bootstrap-run conflicts with zero slot
mutation; digest-only refusal; concurrent identical/conflicting earliest retries with at most one
head; independent heads for different sessions; and cap accounting that includes every queued
slot. Crash proof covers before, during, and after current-head advancement/release and queued-head
publication, with exact retry from each durable state. Security proof scans registry/object files,
logs, traces, diagnostics, and errors for request/prompt/payload bytes. The explicit caller/PID/
helper/socket/endpoint/timeout/EOF/process-liveness/observer-loss matrix proves that none can
acquire or steal admission, head, transport-claim, routability, or terminal authority.

B3.2a-WA proof separately exercises `RG-WORLD-ADOPT-01`: exact generic-to-shared adoption preserves
world ID/generation and creates no second world; exact retry joins without metadata rewrite; and
owner/session/policy/project/spec/generation conflicts fail without mutation. Publication tests
crash/reopen and exact retry (a) before temp creation/persistence, (b) after temp write, (c) after
temp-file `fsync`, (d) immediately before atomic rename, (e) after rename but before parent-directory
`fsync`, and (f) after parent-directory `fsync` but before response. At (e), tests prove both
permitted reopen shapes: an exact original generic final restarts publication, while an exact adopted
final is revalidated and re-`fsync`ed before join. They prove a temp is never semantic ownership by
presence alone, incomplete recognized temps receive only operation-bound cleanup, conflicting,
missing, or ambiguous temp/final evidence fails without mutation, and any exact final join
re-establishes required file and parent-directory durability before success. The member runtime
cannot be created before durable adoption. Compatibility `None` and ordinary world execution remain
unchanged, and prompt/request markers are absent from fixture storage, world metadata, service
storage, logs, traces, errors, and the bounded journal scan. Adoption alone is never accepted as
transport submission, Registered, routability, terminal success, or member launch proof.

An ordinary retained Continue may consume only the exact R0 target joined to a Routable B3.2a
record. `PreTransportNonterminal`, `InterruptedNonterminal`, and `Terminal` return distinct bounded
non-routable failures without changing authority. Remaining observer reconnect/replay, park,
cancel, stop, fork, accepted-turn lifecycle, and final worker semantics stay in the later B3.2/B4
packets.

The B-owned adapter never falls back to legacy active-task authority and cannot issue/apply a
transition, synthesize authority, change lifecycle, consume obligations, or fabricate correlation.
PID, helper, socket, prompt, foreground-guard, process-local-map, and `authoritative_live`
observations are excluded from every B-owned durable decision. A1.2b remains after B3.1/C1 for all
successor and obligation-dependent work.

Retained-worker Inspect/Cancel/Stop is outside B1/B2.1-0 because R0 deliberately supplies no
retained lifecycle, delivery, or terminal-closeout truth. Those historical cases must still enter
the unchanged full dispatcher for differential accounting, but at this closeout they may only
remain `FailToSameFailure` with an identical normalized signature. Replacing their dispatcher
entry with a lower-level resolver or transport call remains `RegressionMasked`.

The canonical source of every value used to construct or interpret
`PreparedOrchestratorWorldDispatch` is exactly:

| Prepared value or decision input | Canonical source classification | Binding rule |
|---|---|---|
| exact accepted bootstrap-home/store binding and derived `BoundAgentRuntimeStateStore` capability | `HostSessionAuthorityTruth` | A1.2a establishes and reads current authority; A1.2a-S adopts that applied Start on the ordinary internal host bootstrap and carries the exact accepted-home bound capability into production. Only the bound capability may access authority, receipt-registry, or supervisor namespaces. |
| legacy `AgentRuntimeStateStore` clone retained by the prepared compatibility shape | `CompatibilityProjectionValidatedAgainstAuthority` | It may perform only compatibility reads after its physical home/store identity validates against the bound capability; it cannot select or access durable authority namespaces. |
| complete validated request, including request/idempotency/action/mode/payload fields | `ValidatedRequestInput` | Validation proves shape only; every authority-bearing request field is checked against its named canonical source below. |
| orchestration session ID and authority revision | `HostSessionAuthorityTruth` | Must equal the exact resolved authority and root observation. |
| current prepared `OrchestrationSessionRecord` shape | `MissingCanonicalRepresentation` | A1 activation rejects legacy writers and A1.1e does not return this compatibility record. B1/B2.1-0 replaces it on its named B-owned actions with a narrower typed dispatch view sourced from A1.2a authority/application truth; it does not synthesize legacy lifecycle state. |
| active caller participant ID and authoritative lineage | `HostSessionAuthorityTruth` | Caller must be the exact active authoritative participant and a member of the exact lineage. |
| current prepared caller backend, role, and participant record shape | `MissingCanonicalRepresentation` | `DurableSessionAuthorityV1` contains IDs/lineage but not backend/role. A1.2a exposes the exact applied descriptor through a typed read result; B1/B2.1-0 narrows the named B-owned paths instead of synthesizing the legacy record shape. |
| workspace binding and authority-store identity | `HostSessionAuthorityTruth` | Use exact canonical workspace root, authority-store root, and store ID. |
| requested world ID and generation | `ValidatedRequestInput` | Must be present in the validated request where required and equal the exact authority world binding. |
| authoritative world binding and generation | `HostSessionAuthorityTruth` | For an action/runtime that requires a world, absence or mismatch fails closed; a host runtime may validly have no binding or may consume the exact session binding. Compatibility state cannot create or repair it. |
| current policy ref and policy revision | `HostSessionAuthorityTruth` | The ref must be a Policy object and both values must equal the exact current authority. |
| effective-policy/snapshot projection used by existing steering behavior | `CompatibilityProjectionValidatedAgainstAuthority` | Its ref/revision and canonical snapshot commitment must validate against exact current-policy identity before use. |
| `live_retained_worker_count` used by `WorkerContinueForkCommand`/spawn/fork steering | `MissingCanonicalRepresentation` | `retained_worker_refs` carry no live/terminal state, while legacy `authoritative_live` is forbidden authority. B3.2a has supplied the exact RetainedWorkerRuntime admission count for production Spawn, and B3.2a-WA has supplied the required exact physical same-world ownership prerequisite; both are review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. B1/B2.1-0 removes the value from only its RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait view. Continue-fork, retained Inspect/Cancel/Stop, and fork semantics remain unchanged and unpromoted for later RetainedWorkerRuntime/B4. |
| target participant ID named by the request | `ValidatedRequestInput` | It is a requested target only until exact authority/accepted-work validation succeeds. |
| current prepared retained target backend, role, and participant record shape | `MissingCanonicalRepresentation` | A1.1e does not expose the retained object/descriptor as this legacy record, and activated stores reject the legacy writer that current fixtures use. R0 replaces the immutable identity shape: RetainedWorkerRuntime creates the descriptor/resume/worker graph; HostSessionAuthority atomically appends its participant to lineage, binds its ref, and proves the new revision as `HostSessionAuthorityTruth`. B3.2a supplies separate routability/admission truth, B3.2a-WA exact-adopts physical ownership of the same HSA-bound world without changing its ID/generation, and B1/B2.1-0 consumes the exact join without fabricating broader lifecycle state. |
| immutable accepted task/active-run identity and acceptance revision | `ReceiptRegistryTruth` | Exact lookup is scoped by store, session, work identity, caller/backend, and world; unknown acceptance fails closed. |
| active claim, journal cursor, interruption state, routability, and immutable terminal closeout | `SupervisorTruth` | Exact claim must match the acceptance record; waiter/caller drop does not delete it and only exact terminal truth closes routability. |
| optional host-transition correlation before A1.2b adoption | `MissingCanonicalRepresentation` | It remains absent through A1.2a and the joint closeout; request ID, task/active-run ID, compatibility state, or the adapter may not synthesize it. |
| PID, helper, socket, prompt, foreground guard, process-local map, and legacy `authoritative_live` state | `EpisodeObservationOnly` | These values may support transport diagnostics only and never supply or override prepared authority, accepted work, routability, or terminal truth. |

The required current-authority creator/read view and the prepared session/caller/live-retained
fields make Case A unrealizable. A1.2a supplies only the prerequisite authority establishment/read
capability; A1.2a-S supplies only its bounded internal production adopter; B1/B2.1-R0 supplies only canonical retained-target registration/read; B3.2a supplies the
production creation/admission bridge and canonical live/routability state; B3.2a-WA supplies only
exact physical ownership adoption of the HSA-bound world; B1/B2.1-0 removes non-B fields from
the B-owned prepared view. A1.2b keeps all
successor/post-turn and obligation-dependent ownership after C1. Full
`dispatch_orchestrator_world_request`/`dispatch_prepared_orchestrator_world_request` entry is
required regression proof; tests that invoke only a lower-level receipt, supervisor, transport, or
resolver API cannot close this gate.

The recorded B1/B2.1-0 gate result is review-clean through `83101dcb`. The B-owned view requires
exact acceptance revision equality and a durable supervisor cursor at or beyond the acceptance
frame before projecting active truth; ordinary retained Continue journals the canonical B0 stream
without invoking legacy obligation or auto-attach writers. The active-task tool adapter preserves
distinct unknown, stale, backend/world mismatch, nonterminal, exact-terminal, and unresolved-replay
outcomes without mutation. Retained compatibility, Spawn, fork, and foreground blocking behavior
remain unchanged. This gate result does not close B1/B2.1 jointly or promote any seam.

Accordingly, the prior claim that `prepare_orchestrator_world_dispatch` itself belongs to A1.2 is
rejected. It remains a B-owned `WorldDispatchControl` consumer after the named prerequisites.
A1.2a alone issues/applies greenfield Start; A1.2a-S only adopts that result and carries its bound
capability; neither packet owns prepared world-work joins, receipts, supervision, or dispatch
lifecycle decisions.


## 2C. `WorldWorkAcceptanceRecordV1`

B1 persists one minimal accepted-work anchor for either lifecycle family without pulling E2 or
B3.2 into the early corridor:

```rust
struct WorldWorkAcceptanceContextV1 {
    schema_version: u32,          // exactly 1
    proposed_acceptance_record_id: String,
    request_id: String,
    message_id: Option<String>,
    caller_backend_id: String,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

enum AcceptedWorldWorkIdentityV1 {
    EphemeralTask {
        task_run_id: String,
    },
    RetainedTurn {
        active_run_id: String,
        message_id: String,
        target_participant_id: String,
    },
}

struct WorldWorkAcceptanceRecordV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    request_id: String,
    authority_store_id: String,
    authority_revision_observed: u64,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    current_policy_snapshot_ref: PolicySnapshotRefV1,
    current_policy_snapshot_hash: String,
    current_policy_revision: String,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    accepted_at: Timestamp,
    record_revision: u64,
}
```

B1 preallocates `proposed_acceptance_record_id` through ReceiptRegistry before submission, but that
candidate is not an accepted record and is not inspectable as accepted work until the runtime
acknowledgement arrives. `WorldWorkAcceptanceContextV1` lives in
`crates/transport-api-types/src/lib.rs` and imports the common correlation type. The existing
`ExecuteRequest` and `MemberTurnSubmitRequestV1` each gain one such typed context; `world-service`
retains the same request-scoped context in its stream/member context. On acknowledgement, B1
persists a record whose `acceptance_record_id`
exactly equals the proposal and whose `runtime_acceptance` joins the acknowledged B0 stream. An
exact submission retry reuses the same proposal; conflicting reuse fails closed. Subsequent B3.1
events copy the retained acceptance ID, request/message causation, caller backend, and optional
correlation into their typed `AgentEvent.worker_event` member; the existing member-turn request
supplies orchestration session, caller/worker participant, worker backend, active run, and world.
This is a typed
request-envelope extension only: it neither creates acceptance before acknowledgement nor changes
`ExecuteStreamFrame`.

The record owns accepted work identity, not final lifecycle, supervisor claim, host-transition
semantics, or model-facing receipt semantics. `host_transition_correlation` is absent for ordinary
world work and remains absent on every production path until A1.2b supplies it from an already
validated HostSessionAuthority transition. B1 defines and stores the optional carrier as part of
its canonical acceptance record, validates only exact equality with the record's
store/session/caller/authority fields, and retains it unchanged; it neither authenticates, issues,
applies, nor interprets the transition and never equates `transition_run_id` with the distinct
accepted task/active-run identity. The landed A1.1e facade is sufficient for the receipt core to
validate an already-existing authority without a new hash domain or verifier; it is not sufficient
to establish that authority or supply the complete joint-closeout prepared-dispatch view. A1.2a
supplies the missing current-authority establishment/read prerequisite but keeps correlation
absent. Production A1.2b later becomes the sole correlation source, first validating its own
intent/revision/payload commitment and then supplying the exact
correlation through the HostSessionAuthority-owned call boundary. B1 validates the exact current
policy identity used for submission but does not invent the E2 retained-worker capability cap. E2
creates the immutable run/cap commitments and the final active receipt references this exact
acceptance record. A conflicting retry cannot create another record for the same runtime work
identity. B2.1 creates its separate claim and journal only through this record.

### B1 receipt-core review versus production completion

B1-3a/B1-3b may become independently review-clean once proposal allocation, exact retry,
acknowledgement validation, activated-store persistence, immutable acceptance inspection, and
legacy-writer rejection are proven. That review point is the only prerequisite that B2.1 consumes;
it is not B1 production completion.

On both the ephemeral-task and retained-turn production paths, the exact acceptance transition is
the handoff boundary. After the immutable record commits, B2.1-1 must durably create or exact-join
the supervisor observation claim before the stream loop reads any subsequent frame. The ephemeral
path must not call `register_active_ephemeral_world_task`, and the retained path must not leave the
foreground loop as the observation owner. Any claim conflict or stale identity fails closed without
ignoring acceptance or falling back to a legacy/generic writer. B1 and B2.1 close only after one
joint production integration gate proves both paths; B3.1 cannot begin from receipt-core review
alone.

### B1 frozen proposal, persistence, and revision semantics

`WorldWorkReceiptRegistry` allocates `proposed_acceptance_record_id` as
`wwa_<lowercase UUIDv7>`. Allocation is a durable proposal reservation, not an acceptance record.
The reservation is stored before transport submission and survives caller drop and host restart so
an exact retry reuses the same proposal. B1 does not expire or garbage-collect reservations; a
later lifecycle packet may add an explicit terminal cleanup rule. Accepted-work inspection never
returns a reservation. Allocation occurs inside the registry transaction and checks the candidate
against every proposal and accepted record across all session states in the single store-wide
registry document; a generated collision is regenerated to a distinct UUIDv7 or fails closed before
publication.

The proposal fingerprint is the exact tuple of:

- authority store ID and observed authority revision;
- orchestration session, caller participant/backend, target backend, world ID/generation;
- request ID, optional retained message ID, and optional unchanged host-transition correlation;
- the complete validated dispatch request, including idempotency key, action, mode, exact typed
  payload, and every optional identity field;
- work family, plus the final typed transport-request commitment and every generated or resolved
  transport identity that must be reused, including the ephemeral task member participant or the
  retained active run/message/target; and
- current policy ref, canonical `PolicySnapshotV3` hash, and policy revision captured before
  submission.

The durable internal proposal and registry schemas are:

```rust
enum ProposedWorldWorkIdentityV1 {
    EphemeralTask,
    RetainedTurn {
        active_run_id: String,
        message_id: String,
        target_participant_id: String,
    },
}

enum WorldWorkSubmissionIdentityV1 {
    EphemeralTask {
        validated_dispatch_request: ValidatedWorldDispatchRequestV1,
        member_dispatch_request: MemberDispatchRequestV1,
        canonical_execute_request_sha256: String,
    },
    RetainedTurn {
        validated_dispatch_request: ValidatedWorldDispatchRequestV1,
        canonical_member_turn_submit_request_sha256: String,
    },
}

struct WorldWorkAcceptanceProposalV1 {
    schema_version: u32,          // exactly 1
    acceptance_context: WorldWorkAcceptanceContextV1,
    authority_store_id: String,
    authority_revision_observed: u64,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    proposed_work: ProposedWorldWorkIdentityV1,
    submission_identity: WorldWorkSubmissionIdentityV1,
    current_policy_snapshot_ref: PolicySnapshotRefV1,
    current_policy_snapshot_hash: String,
    current_policy_revision: String,
    created_at: Timestamp,
}

struct WorldWorkReceiptRegistrySessionStateV1 {
    schema_version: u32,          // exactly 1
    proposals_by_request_id: BTreeMap<String, WorldWorkAcceptanceProposalV1>,
    records_by_acceptance_record_id: BTreeMap<String, WorldWorkAcceptanceRecordV1>,
}

struct WorldWorkReceiptRegistryStateV1 {
    schema_version: u32,          // exactly 1
    sessions_by_id: BTreeMap<String, WorldWorkReceiptRegistrySessionStateV1>,
}
```

Each transport-request digest is lowercase SHA-256 over canonical JSON of a B1-local tagged input
containing `domain = "substrate.b1.world-work-submission.v1"`, the exact
`ValidatedWorldDispatchRequestV1`, and the complete final typed `ExecuteRequest` or
`MemberTurnSubmitRequestV1` including its acceptance context. Canonical JSON recursively sorts
object keys and preserves array order. This registry-local equality commitment is not a
HostSessionAuthority commitment or hash domain, does not authenticate transition meaning, and is
never logged as observability evidence. The raw task environment does not need a second durable
copy in the proposal: the caller reconstructs the complete typed request, and exact retry proceeds
only when its canonical digest matches.

`PolicySnapshotRefV1` is not a new B1 policy-reference model. It is the existing exact
HostSessionAuthority `AuthorityObjectRefV1` current-policy reference, required to have object kind
`Policy`, retained unchanged under the B1-facing field name.

Within one authority store, the durable proposal key is
`(orchestration_session_id, request_id)`. An exact fingerprint match with an unaccepted proposal
returns that stored proposal for transport submission. If the proposal already has its immutable
accepted record, the exact retry returns that record without resubmitting transport or launching
duplicate runtime work. Any mismatch, including a different proposed ID, message, active run,
target, world, policy, or correlation, fails before transport submission or mutation. The current
production paths always supply `host_transition_correlation = None`. A retained proposal allocates one
`wwm_<lowercase UUIDv7>` message ID and reuses it on exact retry. The current V1 retained
`active_run_id` is the exact `MemberTurnSubmitRequestV1.run_id`; request and active-run fields remain
separately validated even when an existing caller supplies equal strings.

For an ephemeral task, the registry allocates the `awm_<lowercase UUIDv7>` member participant and
builds the complete `MemberDispatchRequestV1` and final `ExecuteRequest` before publishing the
proposal. An unaccepted exact retry reuses that stored member-dispatch request, including the same
participant, resolved runtime, prompt, world, run, backend, and lineage fields, reconstructs the
final ExecuteRequest, and requires the same submission digest before transport. For a retained
turn, the final `MemberTurnSubmitRequestV1` is built with the stored proposal/message/context before
publication and its exact digest is required on retry. A changed idempotency key, action, mode,
typed payload or rendered prompt, task member participant, runtime descriptor, command, CWD,
environment, network/filesystem request, retained target, or any other typed submission field is a
conflict before transport; B1 never silently allocates replacement transport identity for the same
proposal.

`WorldWorkReceiptRegistry` persists one store-wide versioned document at exactly
`run/agent-hub/world-work-receipt-registry-v1.json` beneath the already bound activated authority
store. This fixed path is deliberately outside the legacy `sessions` and `participants` collections:
placing a receipt below either collection would correctly make the next HostSessionAuthority
preflight reject the root as unsupported pre-A1 authority state. The document contains one
session-keyed state for each orchestration session; each session state contains proposal
reservations keyed by request ID and immutable acceptance records keyed by acceptance-record ID.
The outer session key must exactly equal every contained proposal/record session ID. Every mutation
loads and validates the complete store-wide document, checks acceptance-ID and retained-message
uniqueness across every session, checks the exact scoped work-identity secondary key, and publishes
one replacement document atomically. No separate index file, per-session file, process-local
registry, multi-file update, dual write, fallback write, or side table may participate in acceptance
truth.

Receipt semantics stay above the physical store. `WorldWorkReceiptRegistry` alone parses and
validates `WorldWorkReceiptRegistryStateV1`, allocates identities, decides exact retry versus
conflict, performs inspection, and supplies canonical JSON bytes. It uses the existing canonical
JSON rules and B1-local submission hashes unchanged; the physical layer treats registry bytes as
opaque and does not add a HostSessionAuthority commitment/hash domain or interpret a receipt.
Malformed canonical bytes, unknown fields/variants, invalid session keys, duplicate identities, or
semantic inconsistency fail before publication.

The HostSessionAuthority store exposes one additive physical capability, conceptually
`WorldWorkReceiptRegistryStorageV1`, bound to the exact `CanonicalDirectoryV1` physical root and
`authority_store_id` observed during exact resolution. Its public operation surface is restricted to
read the fixed registry document, atomically replace that document, and finish/revalidate the
transaction; it has no caller-selected collection/path, removal, authority-root CAS, typed-object,
key, intent, journal, or session-authority operation. `AgentRuntimeStateStore` and
`BoundAgentRuntimeStateStore` do not gain a generic post-activation writer. Root replacement/rebind,
wrong store identity, non-`ValidExisting` activation posture, unsafe/malformed receipt path state, or
an otherwise invalid retained capability fails before receipt mutation and leaves the observed tree
untouched. A later valid HostSessionAuthority root revision does not itself stale this store-bound
capability: the transaction re-reads the current root under lock and requires the same physical root
and store ID. B1 never changes either strict `StateRootV1` or strict `StateRootV2`,
`root_revision`, or `authority_revision`; proposal and
record fields retain the exact authority revision observed before submission. The only B1 accepted
record transition remains absence to immutable `record_revision = 1`; no additional physical or
registry revision counter is introduced.

Lock ordering is fixed. A receipt transaction opens and revalidates the trusted root against the
capability, acquires the existing cross-process `authority-v1/lock/root.lock` first, validates and
reconciles canonical authority-store temps, completes `ValidExisting` semantic preflight, verifies
the locked root's store ID, then opens/reconciles the fixed receipt file and runs the registry
operation. It never takes the process-local StateStore snapshot mutex and never nests another
authority transaction. The same root lock is held through registry validation, temp-file sync,
same-directory atomic replacement, directory/root sync, final root/file revalidation, and release.

Receipt crash reconciliation is namespace-specific and also runs under that root lock. Publication
writes canonical bytes to a same-directory temp named only
`world-work-receipt-registry-v1--<32 lowercase hex>.tmp`, syncs it, revalidates the retained root,
atomically replaces `world-work-receipt-registry-v1.json`, syncs the containing directories, reopens
and byte-validates the final file, and only then releases the lock. On entry, an entry using the
receipt-temp prefix but not the exact grammar, a non-regular exact-name entry, or a no-follow or
retained-identity revalidation failure is unsafe and fails closed without cleanup. Every safely
opened regular file with the exact receipt-temp grammar is non-authoritative interrupted publication
state and is removed without parsing or validating its bytes before the final registry document is
read; unrelated `run/agent-hub` entries remain untouched. A crash therefore leaves either the old
complete file or the new complete file plus at most an exact-name non-authoritative temp. The next
exact semantic retry reloads the survivor and either joins the committed proposal/record or reapplies
the missing mutation; it never infers acceptance from a temp. The existing legacy writer remains
rejected after initialization or activation and is neither called nor weakened by this capability.

The logical accepted-record primary key is
`(authority_store_id, acceptance_record_id)`. The exact work-identity uniqueness key is
`(authority_store_id, orchestration_session_id, AcceptedWorldWorkIdentityV1)`, including the enum
discriminant so task and retained identities cannot alias. An acceptance-record ID may name only
one complete record in the authority store, and one scoped runtime work identity may name only one
acceptance-record ID. Inspection by acceptance-record ID scans the exact bound store and succeeds
only for one record; zero or multiple matches fail closed. Inspection by work identity additionally
requires the exact orchestration-session scope. Proposal-only, missing, ambiguous, cross-store, or
stale-session lookups are not accepted work.

`record_revision` is created at exactly `1`. The B1 acceptance record is immutable: exact retry
returns the stored bytes and revision without rewriting timestamps or incrementing the revision;
conflict performs no write. Thus the only B1 revision transition is absence to revision 1.
Supervisor claims, journals, terminal state, and final active receipts use their own later revision
domains and cannot mutate this acceptance anchor.

The one-file creation transaction validates the proposal and every primary/secondary uniqueness
constraint before adding the record. Concurrent exact creations converge to the same stored record;
concurrent conflicting creations serialize to one winner and one fail-closed result. `accepted_at`
and `runtime_acceptance.observed_at` are fixed by the winning first acknowledgement and are ignored
only when deciding whether a later otherwise byte-identical acknowledgement is an exact retry; the
stored timestamps are always returned unchanged.

The live B1 acknowledgement choice is frozen to the first B0 `Start` frame for both current work
families; `SubmissionAccepted` and `RegisteredFrame` remain reserved enum values and are not B1
production sources:

- `run_world_task`: world-service has successfully created the member runtime control before
  emitting `Start`. The non-empty Start `span_id` is the exact `task_run_id`,
  `runtime_submission_id`, and accepted task identity. The unchanged typed request must satisfy
  `acceptance_context.request_id == MemberDispatchRequestV1.run_id == proposal.request_id`; any
  substituted request context fails before persistence.
- `continue_world_worker`: world-service has validated the exact retained target, reserved the turn
  slot, successfully created runtime run control, registered the submitted turn, and retained the
  unchanged request acceptance context before emitting `Start`. The Start `span_id` is the runtime
  submission ID; `active_run_id` is the request's exact `run_id`, and message ID/retained target come
  only from that retained typed context and typed request.

For either family, Start must be frame sequence 1 on the response stream holding the same typed
context. The record copies `stream_id` and `frame_sequence` from that frame and the proposed ID from
the retained context. Task Start sets only `task_run_id`; retained Start sets only
`active_run_id`, `message_id`, and `retained_participant_id`. A repeated Start, non-Start frame,
Error, EOF, stream exhaustion, terminal frame, local request write, or process-spawn observation
cannot create acceptance. Any mismatch in proposal, store/session/caller/backend/target/world,
policy, work identity, retained message/target, stream, or frame identity fails before persistence.

The policy ref and revision come from the exact pre-submission HostSessionAuthority observation.
The snapshot hash is exactly the existing `ResolvedPolicySnapshot.snapshot_hash`: lowercase
SHA-256 over the serialized canonical `PolicySnapshotV3` actually put on the task request or
resolved for the retained submission before the request is sent. B1 does not define or recompute a
second hash. It carries those three values unchanged through acknowledgement and never re-resolves
policy afterward. These fields describe current submission policy only; they are not the E2
immutable retained-worker cap or final receipt commitment.


## 5. Receipt acceptance source

The B1 acceptance record is an internal durable anchor, not the returnable active receipt described
below. It captures exact current-policy and B0 acknowledgement truth. E2 must create the final
immutable policy/cap commitment and B2.1 must persist its separate
`WorldWorkExecutionClaimV1` before B2.2 may return the linked active receipt.

A foreground call may return an accepted receipt only after all of the following are durable:

1. exact session, caller, backend, world, and task/worker target identity;
2. the accepted immutable `PolicySnapshotV3` ref/hash;
3. exact `task_run_id` or `active_run_id` plus retained `message_id` where applicable;
4. the exact source-owned B2.1 `WorldWorkExecutionClaimV1` and its durable claim identity; and
5. runtime submission acknowledgement that is joined to the same identity.

The persisted acceptance evidence has this minimum shape:

```rust
struct RuntimeAcceptanceEvidenceV1 {
    acknowledgement_kind: RuntimeAcceptanceAcknowledgementKindV1,
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    runtime_submission_id: Option<String>,
    task_run_id: Option<String>,
    active_run_id: Option<String>,
    message_id: Option<String>,
    retained_participant_id: Option<String>,
    observed_at: Timestamp,
}

enum RuntimeAcceptanceAcknowledgementKindV1 {
    SubmissionAccepted,
    StartFrame,
    RegisteredFrame,
}
```

The source-owned B2.1 evidence is the exact canonical `WorldWorkExecutionClaimV1` defined in
`crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs`. Its durable identity
is the bound supervisor store plus the exact
`WorldWorkExecutionSupervisorStateV1.executions_by_acceptance_record_id` key, which equals the B1
acceptance-record ID. The claim carries its own acceptance revision, observer identity/epoch, claim
revision, runtime submission, stream, subject, store/session/caller/backend/world, and optional
HSA-owned transition correlation. Durable frame/event cursors and interruption are separate
supervisor-owned state, not fields of that claim. B2.1 defines no `resumable` field and E2 must not
infer one.

V1 acceptance boundaries:

- `RuntimeAcceptanceEvidenceV1.acceptance_record_id` exactly equals the request's proposed ID, and
  the acknowledgement belongs to the runtime stream holding that same typed acceptance context;
  absence or mismatch fails before record persistence or B3.1 event emission.
- For `run_world_task`, acceptance may use the first non-terminal `Start` or `Registered` frame
  only when the protocol defines that frame as runtime acceptance and its B0 stream/frame identity
  includes or unambiguously joins to `task_run_id`.
- For `continue_world_worker`, acceptance requires an explicit retained-turn submission
  acknowledgement or first non-terminal B0 frame that includes or unambiguously joins to
  `active_run_id`, `message_id`, and the exact retained target.
- A socket write, HTTP request submission, process spawn attempt, or locally allocated ID is not runtime acceptance by itself.
- A terminal-only identity observation cannot be relabeled as pre-terminal acceptance.
- If the current runtime protocol cannot expose accepted identity before terminal exit, extend that protocol before changing the foreground tool to receipt-oriented early return.

B1 persists and inspects both task and retained-turn acceptance records while the existing
foreground call still waits. B2.1-1 then performs the no-gap durable supervisor handoff at the same
accepted production boundary, B2.1-2 makes that foreground call a waiter over journal truth, and
B2.1-3 proves restart reconciliation. Foreground early return is a separate B2.2 gate, and B1 is not
production-complete until the joint B1/B2.1 closeout passes.


## 11. Supervisor idempotency and restart rules

`WorldWorkExecutionSupervisor` alone owns one canonical receipt-scoped claim-and-journal state containing
active observation identity, lease/observer epoch, exact durable cursor, canonical B0 frame/event
entries, restart discovery, interruption/reconciliation state, unresolved producer-replay state,
and immutable terminal closeout. This state is not an
active-task side table and cannot be interpreted by StateStore or HostSessionAuthority. A dedicated
`WorldWorkExecutionSupervisorStorageV1`-shaped capability may provide opaque crash-safe physical
persistence in the activated authority store, but it has no caller-selected path/collection API and
does not create a generic activated-store writer. Legacy activated-store rejection remains
unchanged.

The claim identity binds the exact authority store, B1 acceptance-record ID and revision,
orchestration session, accepted task/active-run identity, B0 stream, world ID/generation, and claim
revision/observer epoch. An exact duplicate claim joins; a conflicting or stale claim fails closed.
Neither the receipt registry nor the physical store may mutate or infer supervisor state.

For future E2 linkage only, B2.1 may expose one read-only, behavior-neutral accessor/projection of
the exact claim identity, canonical claim preimage/hash inputs, and durable claim key. That accessor
does not alter supervisor state, reinterpret lifecycle, create acceptance, infer resumability,
mutate receipts or observations, or expose cursor/interruption references unless a separately
authorized E2 subject requires them. B2.1 remains the sole semantic owner of the returned evidence.

B2.1 is reviewed in three ordered subpackets:

1. **B2.1-1 — durable claim and no-gap handoff:** create or exact-join the claim at exact B1
   acceptance before reading the next frame; replace `register_active_ephemeral_world_task` on the
   accepted ephemeral path and transfer the accepted retained path from foreground observation
   ownership to that same claim.
2. **B2.1-2 — journal and blocking waiter:** persist canonical frame/event bytes and exact identity
   before compatibility delivery; exact replay is a no-op and every gap, reorder, conflict, stale
   observer, or post-terminal write fails closed. Foreground inspect/wait and the minimum cancel
   compatibility consume receipt/supervisor truth without owning it.
3. **B2.1-3 — restart and terminal reconciliation:** the supervisor reopens every nonterminal
   claim at its exact cursor, resumes only through exact producer replay/reconciliation, and keeps
   terminal closeout immutable/idempotent. A host/shell restart may resume when the process-memory
   world-service registry survives. A world-service restart or unavailable producer leaves the
   valid durable claim nonterminal and unresolved. Missing exact terminal truth remains
   nonterminal/interrupted.

The completed joint B1/B2.1 production closeout recorded at the bound 2026-08-03 source snapshot
proves ephemeral acceptance, retained production handoff, legacy-writer exclusion, blocking
compatibility, restart survival, and exact terminal behavior. Actual caller/waiter/guard drop is
proven on the RunWorldTask and ephemeral accepted-task production routes; retained
foreground-waiter drop remains separately proved at the accepted-stream boundary. B3.1 is now
complete on the bound Tuesday, August 4, 2026 candidate, and every later packet still requires
fresh authority and its own review/proof gates.

1. **Accepted anchor first:** B1 persists the acceptance record, exact current-policy identity, B0
   stream identity, and acknowledgement sequence before B2.1-1 creates an observation claim in the
   same production handoff without reading a subsequent frame.
2. **Single logical observer:** supervisors claim a lease with
   `(acceptance_record_id, record_revision, lease_epoch)`. A stale lease cannot write a newer
   revision.
3. **Durable observation journal:** at B1 acceptance, B2.1 installs the only post-acceptance
   observation path with no unjournaled handoff gap. It records canonical B0 frame bytes and the
   receipt-scoped frame/event cursor plus the exact B1 acceptance record ID/revision, accepted-work
   identity, and optional transition correlation before waiter delivery or derived receipt/terminal
   state.
4. **Restart discovery:** the canonical supervisor recovery operation enumerates nonterminal
   supervisor claims/cursors,
   exact-joins each to its immutable B1 acceptance record, and resumes only from that durable
   cursor or through exact runtime replay/reconciliation. A crash after acceptance publication but
   before claim publication leaves an accepted-but-unclaimed handoff, not active or terminal
   inference: the supervisor performs a bounded exact anti-join against immutable accepted records,
   validates the full claim identity, creates an interrupted claim at the acknowledgement cursor,
   and requires exact producer replay/reconciliation before advancing it. Missing claim state alone
   never proves running, completion, cancellation, or failure.
5. **Frame dedupe/order:** each frame is keyed by exact `(acceptance_record_id, stream_id,
   frame_sequence)`. An identical duplicate is a no-op; a gap, reorder, conflicting duplicate, or
   post-terminal frame fails closed.
6. **Event dedupe/order:** durable worker events use exact `(acceptance_record_id, event_id,
   event_sequence)` joined to their frame. Reprocessing cannot duplicate the journal entry. C1,
   not the supervisor, decides whether and how that event materializes an obligation.
   Once B3.1 lands, its semantic validator requires the event's acceptance ID, active run, and
   optional transition correlation to exactly equal the B1 record before generic journal handoff;
   absence, substitution, or mismatch fails closed without inference. B2.1's packet exit itself
   requires only opaque canonical-event commitment plus B0/B1 identity and order.
7. **Monotonic states:** terminal states never revert; stale observers cannot overwrite newer
   state; equal-revision conflicting writes fail closed.
8. **Interrupted observation:** EOF, timeout, caller/observer/process loss, endpoint or replay
   absence, or PID/helper/socket/readiness state without exact terminal event proof records an
   interruption, exact unresolved/replay-unavailable state, and retry metadata. It does not
   fabricate terminal success, cancellation, deletion, cursor advance, or a complete obligation
   cut.
9. **Reconciliation:** only exact producer replay/reconciliation that returns the missing B0 frames
   and exact terminal event may advance the cursor or close the run. A runtime known to have exited
   without that terminal event records an interruption/protocol failure with diagnostics, remains
   incomplete, and cannot produce a C1 Complete cut; ambiguous truth likewise retries/fails closed.
10. **Terminal ordering:** only the exact B0 terminal event ID/sequence closes the supervisor
    observation journal. Supervisor terminal closeout and worker active-turn clearing commit
    atomically or through replay-safe idempotent owner-approved steps; the immutable B1 acceptance
    record does not change, and StateStore supplies persistence only.
11. **Ledger handoff:** the supervisor invokes C1 with durable exact B3.1 events and the terminal
    cut, including exact acceptance-record ID/revision, stream ID, accepted-work identity, and the
    scope-equal owner-supplied transition correlation when a transition-scoped snapshot will be queried. It does not
    classify/materialize obligations. C1 independently commits canonical ledger revisions and
    completeness; coordinated storage never transfers semantic ownership.
12. **Blocking compatibility:** B2.1 may leave the foreground waiting on the durable receipt after
    handoff. Dropping that waiter or any foreground guard cannot delete the acceptance record,
    supervisor claim, journal, or work. Only B2.2 enables model-visible early return.
13. **Cancellation:** one durable cancel request ID is reused across retries; repeated transport
    delivery is safe.
14. **Diagnostics:** non-zero exit, stream error, reconciliation failure, and cancel failure retain
    exact active-run/session/world/policy/stream/event joins.
15. **Startup activation:** the current production `run_async_repl` hook may call exactly one
    canonical supervisor recovery entry point and retain the returned observation tasks. It cannot
    enumerate or interpret claims, duplicate reconciliation, or make lifecycle/terminal decisions.
    Store corruption, invalid claim identity, or impossible durable state is fatal and fails
    startup closed. An individual valid nonterminal claim with unavailable producer replay remains
    durably unresolved; ordinary shell startup cannot discard or terminalize it and need not
    pretend every accepted stream resumed. This bounded hook proves neither full ingress-surface
    neutrality nor any seam promotion.


## 11A. Differential-baseline transition gate

A broad-suite differential is monotonic only when exact test-name set comparison and the same
normalized failure-signature procedure prove every transition. Totals alone are not evidence. This
is the permanent `RG-DIFF-01` regression gate recorded in `05`.

Allowed transitions are limited to:

- historical pass to pass;
- historical failure to the same failure with the same normalized signature;
- historical failure to pass only with the exact causal-resolution proof below; and
- a new test that passes.

The following transitions or explanations are forbidden:

- a historical pass becomes a failure;
- a historical test is removed, hidden, filtered, renamed to evade comparison, or newly ignored;
- a historical failure has an unexplained changed signature;
- a historical failure passes because an assertion was deleted or weakened;
- success bypasses the intended production path through a direct resolver, transport, receipt,
  supervisor, or other lower-level substitute; or
- success disables behavior, swallows an error, adds a permissive fallback, or uses test-only
  branching.

For every historical failure that becomes a pass, the closeout evidence must:

1. preserve the exact historical test name;
2. record its historical failure message and normalized signature;
3. reproduce that historical failure at the exact baseline;
4. prove current success through the intended production path;
5. inspect the test-source changes;
6. show assertions are unchanged or strengthened, or explain and independently prove an
   intentional semantic rewrite;
7. map the transition to an exact slice behavior and changed symbol;
8. prove no unrelated feature or enforcement behavior was removed; and
9. obtain independent review.

For the B1/B2.1 inventory frozen in `05`, preserving a historical name while replacing its real
dispatcher, guard-drop, waiter-drop, or tool-to-dispatch route with a lower-level owner call is
still `RegressionMasked`. Renaming, replacing, newly ignoring, weakening assertions, or substituting
a direct resolver/transport/receipt/supervisor call cannot count as `FailToPass` proof.

The evidence must also publish the complete historical and current inventories, exact transition
matrix, retained-failure names and normalized signatures, failure-to-pass manifest, new-test
manifest, and hashes for those artifacts. Any uncertain transition is
`BaselineRegressionAmbiguous`; it cannot satisfy closeout or contract promotion.
