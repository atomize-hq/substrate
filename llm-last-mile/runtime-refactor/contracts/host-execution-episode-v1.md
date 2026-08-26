**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted `HostExecutionEpisodeV1` schema, episode-kind literals, transport-status literals, and rules 1–6
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#2-hostexecutionepisodev1`](../04-contracts-and-gates.md#2-hostexecutionepisodev1), baseline lines 51–100; the exact 1645-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `28e86e2e7e29b458280875f921ca327a469621ac96bd7e592a0a176027aa0117`

<!-- exact-extracted-body:start -->
## 2. `HostExecutionEpisodeV1`

```rust
struct HostExecutionEpisodeV1 {
    schema_version: u32,                 // exactly 1
    episode_id: String,
    kind: HostExecutionEpisodeKindV1,
    orchestration_session_id: String,
    observed_authority_revision: u64,
    backend_id: Option<String>,
    process_ref: Option<ProcessRefV1>,
    transport_status: HostExecutionEpisodeTransportStatusV1,
    started_at: Timestamp,
    last_heartbeat_at: Option<Timestamp>,
    ended_at: Option<Timestamp>,
    exit_observation: Option<EpisodeExitObservationV1>,
}
```

Episode kinds:

```text
ReplAttachedEpisode
HiddenOwnerHelperStartEpisode
HiddenOwnerHelperAttachEpisode
HiddenOwnerHelperResumeOneTurnEpisode
RuntimeToolboxEpisode
SyntheticOrRecoveredEpisode
```

Transport status:

```text
Available
UnavailableButDurableAuthorityExists
UnavailableAndNoAuthoritativeRoute
StaleOrOrphaned
```

Rules:

1. Episodes submit observations and requested transitions to `HostSessionAuthority`; they never write durable posture directly.
2. An observation whose `observed_authority_revision` is stale cannot mutate authority.
3. Episode exit does not delete session, worker, binding, receipt, or obligation truth.
4. Private transport success may accelerate delivery; it does not define durable success.
5. PID, process/helper presence, active handles, readiness, and prompt-stream state are episode or
   transport observations only; their absence does not erase `ParkedResumable` authority.
6. Episode construction and launch follow durable transition application and cannot reset a parked
   session to `Allocating` or authorize a successor participant.

<!-- exact-extracted-body:end -->
