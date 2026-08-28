**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted deferred retained-spawn admission recovery contract including the recovery preconditions, paired `RetainedWorkerAdmissionCommitmentCarrierV1` / `RetainedWorkerLaunchAuthorityProofV1` schemas, compatibility boundaries, route exclusions, allowlists, and recorded-result limits
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract`](../04-contracts-and-gates.md#deferred-retained-spawn-admission-recovery-contract), baseline lines 71–213; the exact 9896-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `d903299bdd345316fd0943d28abc300a1cef001ce0744746f72f53607eb64c12`

<!-- exact-extracted-body:start -->
### Deferred retained-spawn admission recovery contract

B3.2a deliberately stops at conservative exact-retry behavior. It adds no cancellation,
abandonment, expiry, or liveness-based resolution protocol and adds no runtime enum, schema field,
or persisted request preimage for future recovery. The remaining B3.2 packet owns every durable
admission-state transition used to resolve an abandoned admission; `WorldDispatchControl` in B4
owns the user/tool-facing exact inspect/cancel verb and consumes the RetainedWorkerRuntime result
without writing admission state itself.

Any later resolution request must exact-join all of the following durable preconditions before a
state advance: issuer request identity, admission-record identity and revision, orchestration
session, current exact authority and the relevant admission-to-current ancestry, current policy
identity and authorization, retained participant, and the exact admission state being resolved.
The complete canonical Spawn request remains required wherever fingerprint verification or exact
retry semantics depend on it. PID, timeout, caller presence or disappearance, helper state, socket
state, endpoint state, EOF, observer loss, and process liveness supply no resolution authority.

Resolution is forward-only against durable protocol truth. It must not delete or roll back an
already-committed R0 lineage/ref/registration, classify R0 rejection as cancellation, or treat
stopping an already-created worker as cancelling a pending admission. Partial or already-applied
R0 registration is reconciled to its exact admission record. A transport claim whose launch or
registration result is ambiguous remains live and cannot free capacity until exact transport and
runtime truth is reconciled. Crashes before and after resolution converge on retry; repeated
resolution exact-joins the same terminal result; and the live admission count decreases only after
that terminal result is durably published. Only then may the next eligible queued request acquire
the registration head under the existing earliest-slot rule.

B4 may freeze and return these semantic outcome categories without adding them to the B3.2a
runtime schema: cancelled before registration; registered before transport; transport ambiguous
or cancellation pending; already routable or terminal; invalid target; ambiguous target; and
policy denied. Remaining B3.2 must first provide the durable, restart-safe resolution/reconciliation
primitive those outcomes consume.

The host-to-world launch carrier is typed and substitution-resistant:

```rust
struct RetainedWorkerAdmissionCommitmentCarrierV1 {
    schema_version: u32, // exactly 1
    algorithm: String,   // exactly "hmac-sha-256"
    key_id: String,
    digest_hex: String,
}

struct RetainedWorkerLaunchAuthorityProofV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
    registration_id: String,
    registration_commitment: AuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    authority_record_commitment_after: AuthorityObjectCommitmentV1,
    orchestration_session_id: String,
    caller_participant_id: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    transport_claim_id: String,
    backend_id: String,
    protocol: String,
    world_binding: WorldBindingV1,
    current_policy_ref_id: String,
    current_policy_revision: String,
    retained_worker_ref_id: String,
    retained_worker_commitment: AuthorityObjectCommitmentV1,
}
```

Before either transport builder serializes it, the host exact-joins every field to the current HSA
registration proof, admission record/fingerprint, bound caller, descriptor, policy, and world. The
carrier is the exact field-for-field, transport-neutral equality projection of the internal
RetainedWorkerAdmissionCommitmentV1; it contains no secret key and grants no HSA or admission
mutation authority. The receiving world compares the closed carrier and dispatch fields only; it
does not verify or reinterpret the host-only HMAC input. The
optional carrier field on the V1 compatibility request may remain absent only on the explicitly
pre-activation legacy path; both A1.2a-S authority-managed Spawn producers require it with no
fallback. `transport-api-types` validates its closed shape. Production member Spawn enters
`Service::execute_stream`. For authority-managed `Some(exact proof)`, that member branch completes
the existing strict proof/dispatch equality validation, durably adopts the exact HSA-bound physical
world through B3.2a-WA, and only then calls `MemberRuntimeManager::launch`, which retains its own
validation before process creation. Compatibility `None` retains the existing direct path.
`world-api`, `Service::execute`, and `convert_member_dispatch_request` are not part of this route.
World-service does not mint or advance HSA or admission truth; the bound host verification supplies
authority authenticity, B3.2a-WA supplies physical ownership only, and the launch boundary supplies
exact carrier/request equality.

`#[serde(default)]` on the optional `MemberDispatchRequestV1` proof field preserves wire
compatibility by supplying `Default::default()` only when that field is absent during
deserialization. It does not initialize Rust struct literals: each literal must name the field (or
use explicit Rust struct update syntax, which is not authorized for these fixtures). Therefore the
B3.2a allowlist additionally permits edits only in
`crates/world-service/tests/member_runtime_world_placement_v1.rs`,
`crates/world-service/tests/streamed_execute_cancel_v1.rs`, and
`crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`, solely to set the field to
`None` in existing explicitly pre-activation or legacy literals and prove unchanged compatibility.
Every other fixture input, test name, assertion, expected outcome, and expected error remains
unchanged. Any existing test that claims the authority-managed B3.2a route must carry an exact
valid proof through the canonical production path instead; `None` is not valid there. These test
files may not introduce a production path, helper default, fixture-only authority, alternate
proof, alternate transport, side table, or weakened assertion. Authority-managed Spawn still
requires the exact proof. This mechanical authorization did not itself complete B3.2a; the
recorded B3.2a/B3.2a-WA result in `05` now does.

That Rust-literal requirement also authorizes exactly three production compatibility
initializers and nothing else: `build_run_world_task_transport_request` and
`build_fork_world_worker_transport_request` in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`, plus `convert_member_dispatch` in
`crates/world-mac-lima/src/lib.rs`. Each may only name the new optional proof field as explicit
`None`; every pre-existing backend, protocol, run, session, participant, lineage, world, prompt,
runtime, route-selection, policy, placement, lifecycle, error, and outcome field remains
byte-for-byte and behaviorally unchanged. `None` grants no retained-worker launch authority to
RunWorldTask, ForkWorldWorker, or the macOS world-api conversion. This is not macOS
authority-managed Spawn adoption and authorizes no edit to `crates/world-api/src/lib.rs`,
`Service::execute`, or `convert_member_dispatch_request`. No default constructor, side table,
environment carrier, alternate transport route, or hidden proof synthesis may replace these
explicit initializers. Both authority-managed B3.2a Spawn producers still require
`Some(exact RetainedWorkerLaunchAuthorityProofV1)` exact-joined to admission and R0 truth before
serialization, and missing, malformed, or mismatched proof fails before process creation. This
mechanical authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05`
is review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

The same compiler-required widening authorizes only seven additional production-symbol edits. In
`crates/shell/src/execution/orchestrator_world_dispatch.rs`, `fork_world_worker` and
`continue_world_worker_fork_command_bootstrap_after_delivery` may only pass explicit `None` for the
optional retained-worker authority context to the widened stream helper. In
`crates/shell/src/repl/async_repl.rs`, `apply_greenfield_host_start_from_authority`,
`prepare_hidden_owner_helper_runtime`,
`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
`prepare_fork_child_runtime_startup_for_descriptor`, and
`prepare_member_runtime_startup_for_descriptor` may only initialize, destructure, or preserve the
new optional retained-worker launch-authority proof/admission fields as `None`. Those values grant
no retained-worker launch authority: fork and fork continuation remain compatibility paths; Host
Start retains only its A1.2a-S host-session authority; hidden-owner-helper behavior is unchanged;
and `prepare_member_runtime_startup_for_descriptor` remains legacy pre-activation preparation,
distinct from the B3.2a-only `prepare_member_runtime_startup_from_authority_registration` path.
Only that authority-registration preparer may construct the paired
`Some(exact RetainedWorkerLaunchAuthorityProofV1)` and exact admission context. An
authority-managed retained Spawn with either value missing fails closed and cannot fall back to the
legacy preparer or reinterpret generic `None` as compatibility. The seven exceptions change no
packet ownership, policy, lifecycle, transport, identity, lineage, error, outcome, host-runtime,
helper, legacy-member, or fork semantics and authorize no other structural change. This mechanical
authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05` is
review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

<!-- exact-extracted-body:end -->
