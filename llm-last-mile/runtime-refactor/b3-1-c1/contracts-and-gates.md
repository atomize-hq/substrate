**Kind:** contracts and gates
**Stable ID:** `B3.1-C1-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** bounded B3.1 retained worker event envelope required by C1
**Source span:** [`04-contracts-and-gates.md#2b-bounded-retained-worker-event-envelope`](../04-contracts-and-gates.md#2b-bounded-retained-worker-event-envelope), baseline lines 3253–3379

# B3.1/C1 family contracts and gates

## 2B. Bounded retained worker event envelope

B3.1 supplies the minimum semantic envelope C1 needs without moving B3.2 early. It extends the
existing shared `AgentEvent` in `crates/common/src/agent_events.rs` with one optional, explicitly
typed top-level `worker_event` field. The same module defines the producer-construction-only
`NormalizedWorldWorkerEventFacetV1`. In `world-service/member_runtime.rs`, one named fail-closed
normalizer parses the existing provider `AgentWrapperEvent` kind/payload before `AgentEvent`
construction and produces this facet; it is the sole permitted provider-payload interpretation for
the C1 path. `ExecuteStreamFrame::Event { event: AgentEvent }` remains unchanged, so
`world-service` can produce the envelope and shell consumers can validate it without host-side
identity or semantic inference from `AgentEvent.data` or a second transport protocol.
Both the equality-only commitment representation and its host-transition correlation carrier live
in `crates/common/src/authority_commitment.rs` and are re-exported by
`crates/common/src/lib.rs`. They contain no verifier or semantic authority:

```rust
enum OpaqueAuthorityCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

struct HostTransitionWorkCorrelationV1 {
    schema_version: u32,          // exactly 1
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    transition_intent_id: String,
    transition_intent_revision_observed: u64,
    transition_run_id: String,
    transition_payload_commitment: OpaqueAuthorityCommitmentV1,
    authority_revision_observed: u64,
}

enum WorldWorkerEventClassV1 {
    Reply,
    ProgressUpdate,
    FollowUpQuestion,
    ApprovalRequest,
    Blocked,
    AttentionRequired,
    ForkRequest,
    ForkRecommendation,
    Result,
    Failure,
}

// Producer-construction type, not an independent wire envelope.
struct NormalizedWorldWorkerEventFacetV1 {
    schema_version: u32,          // exactly 1
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    causation_message_id: String,
    causation_request_id: String,
    payload: serde_json::Value,
}

struct WorldWorkerEventV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    event_id: String,
    event_sequence: u64,
    request_id: String,
    active_run_id: String,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    causation_message_id: String,
    causation_request_id: String,
    orchestration_session_id: String,
    source_participant_id: String,
    target_participant_id: String,
    source_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    payload: serde_json::Value,
    emitted_at: Timestamp,
}

struct AgentEvent {
    // All existing legacy fields remain unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worker_event: Option<WorldWorkerEventV1>,
}
```

The exact B0 event identity is copied unchanged. `acceptance_record_id` and `active_run_id` join the
B1 accepted retained turn. When B1 carries `host_transition_correlation`, B3.1 copies it unchanged;
B3.1 rejects absence or mismatch rather than inferring intent/run/authority scope from request ID,
active-run ID, foreground state, or payload. Before `AgentEvent` construction, the producer
normalizer maps every provider wrapper event into exact thread/class/attention/payload semantics
and joins request/message causation from the retained B1 context. Explicit supported non-attention
wrapper shapes receive a typed non-attention class. Unknown, ambiguous, deferred, or malformed
shapes fail the retained stream and C1 cut closed; they cannot be dropped, left untyped, or
downgraded to progress or no-attention. The final source/target, world, run, thread, class,
attention, and causation fields are explicit and validated before ledger delivery; the resulting
B2.1 generic journal ref commits the exact canonical shared-envelope bytes without the supervisor
interpreting them.

After emission, neither host code nor C1 may infer a missing field from free text,
`AgentEvent.data`, optional nested runtime payloads, the most recent worker, or foreground request
state. Raw provider payload may remain as compatibility/debug data, but it is not semantic truth
for the C1 path. B3.1 does not require an upstream provider-wrapper taxonomy change and does not
complete host-to-worker messaging, retained lifecycle, park, cancel, stop, fork, or model-facing
early return; those remain B3.2/B4.

`worker_event` may be absent only for streams outside the B3.1-adopted accepted-retained scope. In
that scope, every post-acknowledgement `ExecuteStreamFrame::Event` must carry it through the exact
terminal cut; absence or invalidity is a protocol failure and makes a Complete C1 snapshot
unavailable. Every identity/correlation/semantic field comes from this typed member rather than
host parsing of `AgentEvent.data`. The normalizer may read provider `AgentWrapperEvent.data` only
before emission, under its explicit exhaustive provider-shape tests, to construct
`NormalizedWorldWorkerEventFacetV1`.
`OpaqueAuthorityCommitmentV1` preserves the exact variant and fields of the HostSessionAuthority
commitment supplied later by A1.2b, but B1/B3.1/C1 may only retain and compare it; they cannot mint,
verify, reinterpret, log as observability evidence, or use it as obligation semantics.
