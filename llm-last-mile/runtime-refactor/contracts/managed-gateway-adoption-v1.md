**Kind:** contract
**Stable ID:** `managed-gateway-adoption-v1`
**Status:** canonical specification; E3 is not admitted, dispatched, or implemented
**Canonical for:** E3 managed-gateway identity, dormant/activation lifecycle, OS access boundary, non-secret intent/launch-input/ACK/reference records, one-time secret-FD reuse, and pre-release revalidation
**Authority baseline:** source commit `138864a26dbc4721366c6cc8934d464d1929a189`, tree `881393e8fc6db7d4422f97cceadfa550d076a14f`
**Supersedes:** E3 planning statements that treat a loopback URL, readiness response, request header, copied auth, or provider success as managed-gateway adoption proof
**Superseded by:** none
**Projection consumers:** [`agent-config-projection-v1.md`](agent-config-projection-v1.md), [`launch-time-secret-handoff-v1.md`](launch-time-secret-handoff-v1.md), [`../slices/e3-agent-config-projection-and-gateway-adoption.md`](../slices/e3-agent-config-projection-and-gateway-adoption.md)

# Managed gateway adoption V1

> **Authority boundary:** E3 reuses the landed `GatewayAuthBundleV1` pipe and gateway one-time
> consumer exactly once. It adds non-secret orchestration identity, an initially denied access
> boundary, durable intent/ACK references, and a launch fence. It does not persist secrets, authorize
> requests from identity headers, redefine gateway provider policy, or let Codex/UAA inherit the FD.

## Landed primitive and remaining gap

The baseline already validates `GatewayAuthBundleV1`, uses `pipe2(O_CLOEXEC)`, exposes only the
numeric pointer `SUBSTRATE_LLM_AUTH_BUNDLE_FD` to the gateway child, removes known raw-secret
environment variables, clears close-on-exec only for that gateway receiver, consumes the bundle once,
removes the pointer environment variable, and disables gateway token persistence. This contract does
not replace that implementation.

The baseline readiness object and runtime manifest are not E3 adoption authority. They identify a
backend/world/port/process imperfectly, do not bind a retained participant or config series, do not
prove an OS access boundary, and do not produce a durable non-secret handoff join. The direct Codex
member path also does not consume them. E3 closes only that gap.

## Exact schemas

Every struct below is strict and denies unknown fields. Every enum uses the exact externally tagged
variant names shown. Optional fields, where referenced by another contract's canonical preimage, are
present as JSON `null`; no secret or secret-derived value is part of any object.
`InWorldGatewayRefV1` and `ManagedGatewayActivationIntentRefV1` are the lower-leaf wire types owned
by `transport-api-types` as fixed by the projection contract. The immutable identity, boundary,
intent, launch-input, handoff-wrapper, ACK, and reference schemas below are owned by
`config-projection`; that crate also owns the strict, ephemeral, nonpersistent
`E3GatewaySecretReadyAttestationV1` wire type shared by the gateway and world-service.
`ManagedGatewayLaunchCapabilityV1`, every `Held*` member it contains, and the
ephemeral readiness probe input/connected/result types are private world-service types; gateway's consumed launch
contract remains private to `gateway`. Neither operational crate type is imported or re-exported by
`config-projection`.

```rust
struct InWorldGatewayRefV1 {
    authority_store_id: String,
    gateway_instance_id: String,
    gateway_identity_hash: String,
}

struct InWorldGatewayIdentityV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    gateway_instance_id: String,
    config_projection_identity_hash: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    gateway_artifact_sha256: String,
    access_boundary_id: String,
    gateway_identity_hash: String,
}

struct GatewayProcessIdentityV1 {
    pid: u32,
    pid_start_time_ticks: u64,
    pidfd_inode: u64,
    executable_device_id: u64,
    executable_inode: u64,
    executable_sha256: String,
    process_cgroup: CanonicalCgroupIdentityV1,
    child_security_attestation_hash: String,
    secret_ready_attestation_hash: String,
}

struct GatewayListenerIdentityV1 {
    transport: String, // exactly "tcp"
    network_namespace_inode: u64,
    address: String, // exactly "127.0.0.1"
    port: u16,
    socket_inode: u64,
    listen_backlog: u32, // exactly 16
    deny_boundary_effective_before_listen: bool, // exactly true
    responses_base_path: String, // exactly "/v1"
}

struct GatewayRuntimeConfigIdentityV1 {
    root: CanonicalDirectoryV1,
    relative_path: String, // exactly "config.toml"
    mode: u32, // exactly 0o600
    byte_length: u64,
    sha256: String,
}

struct GatewayHttpSurfaceV1 {
    inherited_listener_only: bool, // exactly true
    readiness_method: String, // exactly "GET"
    readiness_path: String, // exactly "/health"
    member_method: String, // exactly "POST"
    member_path: String, // exactly "/v1/responses"
    auxiliary_listener_count: u32, // exactly 0
}

struct GatewayAccessBoundaryRefV1 {
    authority_store_id: String,
    access_boundary_id: String,
    revision: u64,
    boundary_hash: String,
}

struct GatewayAccessBoundaryV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    kernel_effect_intent_ref: E3KernelEffectIntentRefV1,
    access_boundary_id: String,
    gateway_instance_id: String,
    config_projection_identity_hash: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    gateway_listener: GatewayListenerIdentityV1,
    readiness_probe_cgroup: CanonicalCgroupIdentityV1,
    allowed_member_cgroup: CanonicalCgroupIdentityV1,
    nftables_chain: NftablesChainIdentityV1,
    nftables_rules: Vec<NftablesRuleIdentityV1>,
    posture: GatewayAccessPostureV1,
    revision: u64,
    predecessor_ref: Option<GatewayAccessBoundaryRefV1>,
    boundary_hash: String,
}

struct CanonicalCgroupIdentityV1 {
    cgroup_v2_mount_device_id: u64,
    cgroup_v2_mount_inode: u64,
    cgroup_directory_inode: u64,
    cgroup_relative_path: String,
}

struct NftablesRuleIdentityV1 {
    role: NftablesRuleRoleV1,
    family: String, // exactly "inet"
    table: String,
    chain: String,
    rule_handle: u64,
    canonical_expression: String,
}

enum NftablesRuleRoleV1 { ReadinessProbeAccept, ExactMemberAccept, RejectRemainder }

struct NftablesChainIdentityV1 {
    family: String, // exactly "inet"
    table: String,
    chain: String,
    chain_handle: u64,
    chain_type: String, // exactly "filter"
    hook: String, // exactly "output"
    priority: i32, // exactly -100
    policy: String, // exactly "accept"
}

enum GatewayAccessPostureV1 { DenyAllDormant, AllowExactMember, Revoked }

struct ManagedGatewayActivationIntentV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    preparation_id: String,
    activation_intent_id: String,
    config_projection_identity_hash: String,
    dormant_record_id: String,
    dormant_revision: u64,
    expected_gateway_artifact: DescriptorPinnedArtifactV1,
    expected_gateway_ref: InWorldGatewayRefV1,
    expected_access_boundary_ref: GatewayAccessBoundaryRefV1,
    secret_handoff_ref: SecretHandoffRefV1,
    fence_id: String,
    readiness_nonce: String,
    created_at: Timestamp,
    intent_hash: String,
}

struct ManagedGatewayActivationIntentRefV1 {
    authority_store_id: String,
    activation_intent_id: String,
    intent_hash: String,
}

struct ManagedGatewayLaunchInputV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    launch_input_id: String,
    activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    dormant_projection_ref: ConfigProjectionRefV1,
    gateway_ref: InWorldGatewayRefV1,
    config_projection_identity_hash: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    listener_identity: GatewayListenerIdentityV1,
    gateway_config: GatewayRuntimeConfigIdentityV1,
    http_surface: GatewayHttpSurfaceV1,
    access_boundary_ref: GatewayAccessBoundaryRefV1,
    secret_handoff_prepared_ref: SecretHandoffRefV1,
    readiness_nonce: String,
    launch_input_hash: String,
}

struct ManagedGatewayLaunchInputRefV1 {
    authority_store_id: String,
    launch_input_id: String,
    launch_input_hash: String,
}

struct SecretHandoffRefV1 {
    authority_store_id: String,
    handoff_id: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    runtime_family: String,
    world_id: String,
    world_generation: u64,
    receiving_gateway_identity_hash: String,
    handoff_state_revision: u64,
    handoff_hash: String,
}

// E3-retained wrapper; the embedded LaunchTimeSecretHandoffV1 schema is unchanged.
struct ConfigProjectionSecretHandoffRevisionV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    handoff: LaunchTimeSecretHandoffV1,
    predecessor_ref: Option<SecretHandoffRefV1>,
    revision_hash: String,
}

// Ephemeral nonsecret post-exec barrier evidence; never durable on its own.
struct E3GatewaySecretReadyAttestationV1 {
    schema_version: u32, // exactly 1
    gateway_instance_id: String,
    launch_input_hash: String,
    gateway_pid: u32,
    gateway_pid_start_time_ticks: u64,
    dumpable: u32, // exactly 0 after PR_SET_DUMPABLE and PR_GET_DUMPABLE
    rlimit_core_soft: u64, // exactly 0
    rlimit_core_hard: u64, // exactly 0
    tracer_pid: u32, // exactly 0
    attestation_hash: String,
}

struct ManagedGatewayActivationAckV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    activation_ack_id: String,
    activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    config_projection_identity_hash: String,
    dormant_projection_ref: ConfigProjectionRefV1,
    gateway_ref: InWorldGatewayRefV1,
    gateway_process_identity: GatewayProcessIdentityV1,
    child_security_attestation: E3ChildSecurityAttestationV1,
    listener_identity: GatewayListenerIdentityV1,
    access_boundary_ref: GatewayAccessBoundaryRefV1,
    secret_handoff_ref: SecretHandoffRefV1,
    secret_handoff_terminal_state: SecretHandoffStateV1, // exactly Consumed
    gateway_ready_revision: u64,
    readiness_nonce: String,
    launch_input_ref: ManagedGatewayLaunchInputRefV1,
    observed_at: Timestamp,
    ack_hash: String,
}

struct ManagedGatewayActivationAckRefV1 {
    authority_store_id: String,
    activation_ack_id: String,
    ack_hash: String,
}

// World-service-private, sealed, non-serializable, non-cloneable process-local capability.
struct ManagedGatewayLaunchCapabilityV1 {
    preparation_id: String,
    launch_input: ManagedGatewayLaunchInputV1,
    privileged_child_exclusion_lease: HeldE3PrivilegedChildExclusionLeaseV1,
    world_entry_wrapper: HeldExecutableDescriptor,
    gateway_executable: HeldExecutableDescriptor,
    inherited_listener: HeldSocketDescriptor,
    nonsecret_launch_input_reader: HeldPipeDescriptor,
    world_fs_enforcement_input_reader: HeldPipeDescriptor,
    setup_ready_writer: HeldPipeDescriptor,
    final_exec_reader: HeldPipeDescriptor,
    secret_handoff_reader: HeldPipeDescriptor,
    gateway_secret_ready_reader: HeldPipeDescriptor,
    gateway_secret_ready_writer: HeldPipeDescriptor,
    process_cgroup: HeldCgroupDescriptor,
    readiness_probe_cgroup: HeldCgroupDescriptor,
    allowed_member_cgroup: HeldCgroupDescriptor,
    boundary: HeldNftablesBoundary,
}
```

IDs are `cgi_`, `gab_`, `gai_`, `gal_`, `gaa_`, and `gsh_` followed by a lowercase UUIDv7 for
gateway instance, access boundary, activation intent, launch input, ACK, and E3 secret handoff
respectively. The E3 authority service allocates every `gsh_` handoff ID once from its CSPRNG when it
accepts the exact preparation; an exact retry reuses that ID and any new attempt or gateway process
allocates another. This E3 allocation domain does not alter the existing strict
`LaunchTimeSecretHandoffV1` schema or constrain a non-E3 producer's existing ID domain.
`readiness_nonce` is a public UUIDv7
challenge generated by the authority service for this intent; it proves freshness/equality only and
is not an authorization secret. `gateway_ready_revision` is exactly 1 for the one process/intent;
readiness is immutable and a restart allocates a new identity and ACK rather than incrementing it.
`preparation_id` is the exact `e3p_` ID accepted by the preparation route. The intent, sealed launch
capability, credential-source ref, preparation response, and in-memory preparation-map key must all
carry that same value; equality under another preparation never substitutes.

`SecretHandoffRefV1.handoff_hash` equals the referenced
`ConfigProjectionSecretHandoffRevisionV1.revision_hash`, lowercase SHA-256 over
`{"domain":"substrate.e3.config-projection-secret-handoff-revision.v1","revision":<the exact
wrapper with revision_hash omitted>}` under the projection canonical-JSON rules. The embedded
`LaunchTimeSecretHandoffV1` is the existing schema unchanged and contains no secret bytes or
secret-derived digest. Prepared has `predecessor_ref = null`; Delivered names the exact Prepared ref;
Consumed/Failed/Expired names the exact immediately prior nonterminal ref. The ref resolves exactly
one retained wrapper revision and validates store, session, participant, runtime family,
world/generation, gateway receiver, state revision, predecessor, and hash; an equal terminal state
from another handoff is not a substitute.
For E3, every embedded handoff additionally requires
`delivery = SecureFd { fd_name: "SUBSTRATE_LLM_AUTH_BUNDLE_FD", one_time: true,
gateway_receiver_only: true, deny_child_inheritance: true, close_after_consume: true }`, byte-equal to
the nonsecret handoff projection. A different V1 logical label or any false Boolean is
`WrongBinding` before intent publication or secret access; no inferred environment name substitutes.

The listener is intentionally loopback for Codex 0.125 `base_url` compatibility. Loopback placement
alone is not isolation. “Reserve” has one exact meaning: world-service creates
`socket(AF_INET, SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK, IPPROTO_TCP)`, binds it to
`127.0.0.1:0`, captures the assigned nonzero port and socket/network-namespace identities, and
requires `SO_ACCEPTCONN=0`; it does not call `listen` yet. No reuse-port/reuse-address option or
alternate address is enabled. While the socket remains bound but non-listening, world-service
publishes the boundary kernel-effect intent, installs and exact-readback-validates
`DenyAllDormant`, and only then calls `listen` once with backlog exactly 16. It immediately requires
`SO_ACCEPTCONN=1`, TCP state `LISTEN`, and a zero-time `ppoll` with no `POLLIN`/error/hangup event;
the readiness and member cgroups are still empty, so the already-effective boundary prevents a new
unauthorized queue entrant after this check. Failure closes the socket, revokes/removes the boundary,
and publishes no Dormant head. A connect attempted before `listen` cannot complete or queue bytes;
any observed pre-gateway accept readiness conflicts and aborts. Only after those proofs may
world-service pass the same held open file description to the descriptor-pinned gateway; the gateway
may not bind, reopen, relisten, or substitute it. Before gateway exec, world-service therefore has a
readback-proven default-deny cgroup/nftables owner rule for the exact accepting listener.
`DenyAllDormant` permits only a descriptor-pinned world-service probe running in
`readiness_probe_cgroup`; `AllowExactMember` permits that probe and connections
solely from `allowed_member_cgroup` and its descendants; `Revoked` permits none. The readiness,
member, and gateway-process cgroups are per-purpose children of the exact world-generation cgroup
and must all differ. A sibling participant,
same UID, forged header, reused
port, or process in the world root cgroup is denied. The cgroup directory is opened and physically
identified before use; the nftables table/chain/rule handles and canonical rule expression are hashed
in `NftablesRuleIdentityV1`. If the platform cannot prove this boundary, credential-requiring E3 is
unsupported rather than degraded to shared loopback. This V1 acceptance scope is Linux only.

E3 in-world mode has exactly one listening socket: the inherited listener in
`GatewayListenerIdentityV1`. It neither calls `TcpListener::bind` for the configured main address nor
starts the baseline auxiliary OAuth callback listener on `127.0.0.1:1455`. Immediately after gateway
exec and again before readiness, world-service inspects the pinned process's network namespace and
descriptor/socket inventory and requires exactly one TCP `LISTEN` socket owned by that process, with
the same open-file-description/socket inode as the retained inherited listener; zero, two, `:1455`, a
Unix listener, a UDP listener, or an unowned/substituted listener aborts activation. Connected
provider sockets are not listeners and do not satisfy or expand this check.

No baseline inet-diag or sock-diag helper is authority for these claims, and E3 does not invent a
standalone observation subsystem. The P1 listener-ordering checks are performed inline by the
fence-named `E3GatewayRuntimeAuthorityV1` on its held listener descriptor using `fstat`,
`getsockname`, `SO_ACCEPTCONN`, `TCP_INFO`, and zero-time `ppoll`; inability to obtain an exact result
is `UnsupportedPlatform` or `WrongBinding` before Dormant publication. Readiness connection evidence
comes from the descriptor-pinned, security-attested probe's closed syscall state machine and
canonical connected attestation. Before releasing that probe to connect and again before releasing
its request, the parent independently revalidates the exact PID/start tuple and readiness cgroup;
the already-effective boundary permits the listener connection only from that cgroup. Neither a
port/tuple claim alone nor an ambient world-service networking observation is authority.

Boundary revision 1 is `DenyAllDormant` with `predecessor_ref = null`. Its only successor is
`AllowExactMember` or `Revoked`; the only successor of `AllowExactMember` is `Revoked`; `Revoked` is
terminal. Revisions are prior revision plus one and exact-retry byte-equal only. A new attempt uses a
new access-boundary ID and initial deny revision—no old boundary or rule set is reopened.

The boundary is in the gateway process's exact recorded network namespace and matches only IPv4
destination `127.0.0.1`, TCP, and the reserved listener port. When the baseline Linux backend uses
its cgroup-scoped host-namespace fallback rather than a distinct netns, the recorded namespace inode
and all three exact world-generation cgroups are mandatory; namespace sharing never widens the rule.
`nftables_chain` records the base chain handle,
filter type, output hook, accept policy, and priority `-100`; activation first verifies every broader
world allow chain for the namespace has a numerically later priority, including the baseline priority
0 world filter, or reports `UnsupportedPlatform` before gateway start.
The table name is exactly `substrate_e3_` followed by the first 24 lowercase hex characters of
SHA-256 over the UTF-8 `access_boundary_id`; the chain name is exactly `gateway_output`. An existing
name is acceptable only for an exact retry whose recorded table/chain handles and complete rule set
match; any hash-prefix collision or unequal object is `Conflict`. Baseline GC's
`substrate_<world-id>` table deletion cannot match this namespace. E3 recovery, not generic GC,
revokes and later removes the exact E3 table after all bound cgroups are empty.
`nftables_rules` is ordered exactly as evaluated and is
installed/replaced as one atomic nftables batch. `DenyAllDormant` contains
`[ReadinessProbeAccept, RejectRemainder]`; `AllowExactMember` contains
`[ReadinessProbeAccept, ExactMemberAccept, RejectRemainder]`; `Revoked` contains only
`[RejectRemainder]`. Every canonical expression is normalized nftables JSON (object keys sorted under
the projection canonical-JSON rules), matches the exact address/port, and for an accept uses the
kernel socket-cgroup-v2 ancestor match at the recorded cgroup depth. The reject tail rejects every
other connection to that listener; unrelated destinations and ports continue to pre-existing world
policy. Each revision stores every exact table, chain, chain/rule handle, role, and expression returned
by the kernel. Delete/recreate, order/handle/expression change, missing reject tail, broader cgroup
match, or chain posture/priority change is wrong-binding or stale authority and prevents release.

## Canonical hashes and reference validation

This contract uses the canonical JSON rules in
[`agent-config-projection-v1.md#canonical-encoding-hashes-ids-and-exact-validation`](agent-config-projection-v1.md#canonical-encoding-hashes-ids-and-exact-validation).
Lowercase SHA-256 values are computed over:

| Value | Canonical preimage |
|---|---|
| `gateway_identity_hash` | `{"domain":"substrate.e3.in-world-gateway-identity.v1","gateway":<gateway identity with gateway_identity_hash omitted>}` |
| `boundary_hash` | `{"boundary":<boundary with boundary_hash omitted>,"domain":"substrate.e3.gateway-access-boundary.v1"}` |
| `intent_hash` | `{"domain":"substrate.e3.managed-gateway-activation-intent.v1","intent":<intent with intent_hash omitted>}` |
| `launch_input_hash` | `{"domain":"substrate.e3.managed-gateway-launch-input.v1","launch_input":<launch input with launch_input_hash omitted>}` |
| gateway secret-ready `attestation_hash` | `{"attestation":<secret-ready attestation with attestation_hash omitted>,"domain":"substrate.e3.gateway-secret-ready-attestation.v1"}` |
| handoff `revision_hash` | `{"domain":"substrate.e3.config-projection-secret-handoff-revision.v1","revision":<handoff revision wrapper with revision_hash omitted>}` |
| `ack_hash` | `{"ack":<ack with ack_hash omitted>,"domain":"substrate.e3.managed-gateway-activation-ack.v1"}` |
| readiness-probe `input_hash` | `{"domain":"substrate.e3.managed-gateway-readiness-probe-input.v1","input":<probe input with input_hash omitted>}` |
| readiness-probe `connected_hash` | `{"connected":<connected attestation with connected_hash omitted>,"domain":"substrate.e3.managed-gateway-readiness-probe-connected.v1"}` |
| readiness-probe `result_hash` | `{"domain":"substrate.e3.managed-gateway-readiness-probe-result.v1","result":<probe result with result_hash omitted>}` |

An `InWorldGatewayRefV1` resolves the immutable `InWorldGatewayIdentityV1` by exact store and instance
ID and requires the recomputed identity hash. Process and listener identity are intentionally absent
before spawn and become immutable ACK evidence; they never rewrite the preallocated gateway identity.
The ACK embeds the exact nonsecret wrapper security attestation validated before gateway exec and
secret delivery; its recomputed hash must equal
`gateway_process_identity.child_security_attestation_hash`. Its process identity also carries the
exact validated post-exec secret-ready attestation hash; that attestation must bind the same gateway
instance, launch input, PID, and start time and prove zero dumpability/core limits before delivery.
A gateway self-report without the pinned pipe/barrier, later `/proc` snapshot, same-PID value, or
equal attestation under another projection cannot replace either proof.
`ManagedGatewayLaunchInputRefV1` resolves exactly one immutable launch input by configured store and
ID and requires its recomputed hash; equal input bytes under another ID do not substitute.
An intent/launch-input/ref/ACK is valid only through the configured accepted-home projection registry. Resolution
uses no caller-selected path. Stable store, projection, session, participant, backend,
world/generation, artifact, gateway, process, listener, fence, nonce, and hash fields are exact
equality joins. Revisioned handoff and access-boundary references obey the unique monotonic lineage
rules below rather than impossible cross-state byte equality. Equal URL, port, PID, bytes, backend
name, terminal state, or posture never substitutes for a different identity or predecessor chain.
Live validation uses the held pidfd, `/proc/<pid>/stat`, `/proc/<pid>/exe`, the
retained duplicate of the inherited listener descriptor plus kernel socket diagnostics, and held
cgroup/nftables rule-set handles; it rejects PID/port/socket/inode reuse.
The readiness result's `response_body_sha256` is ordinary lowercase SHA-256 over the exact decoded
body bytes; the body must itself be the unique canonical JSON encoding of the fixed readiness schema
below before it is compared or admitted.
The three cgroup directories are owner-only control objects unavailable to the gateway/member;
world-service is the sole membership writer and rechecks `/proc/<pid>/cgroup` through the pinned PID
identity at ACK, boundary CAS, and child release.

The exact permitted joins are:

- the intent's `secret_handoff_ref` resolves the `Prepared` revision and its
  `expected_access_boundary_ref` resolves `DenyAllDormant` revision 1;
- the launch input repeats those intent identities, the exact dormant projection ref, listener,
  `GatewayHttpSurfaceV1`, Deny ref, and nonce, and its hash is computed only after all are fixed;
- after descriptor-pinned gateway exec succeeds and its exact process identity is captured, the
  handoff head advances from the intent's Prepared ref to the unique `Delivered` successor;
- the ACK contains the exact persisted `launch_input_ref`, exact gateway process/security identity,
  and repeats the intent and launch-input
  identity, but its `secret_handoff_ref` resolves the unique `Consumed` successor of that Delivered
  revision: same handoff ID and bindings, state revision exactly Prepared plus two, Delivered's
  `predecessor_ref` equals Prepared and Consumed's `predecessor_ref` equals Delivered; the ACK
  access-boundary ref is still byte-equal to the intent's Deny ref;
- the `ReadyClosed` record contains that exact ACK/Consumed ref and exact Deny ref; and
- the `Active` record retains the exact intent, ACK, Consumed handoff, gateway, and listener, while
  its access-boundary ref resolves the unique `AllowExactMember` successor whose predecessor ref is
  exactly the ACK's Deny ref and whose revision is Deny plus one.

For each individual ref, every store/ID/revision/hash field is byte-equal to the object it resolves.
No rule permits Prepared, Delivered, and Consumed to equal one another, or Deny to equal Allow;
authority is the exact unique successor chain just stated. A second successor, skipped revision, predecessor mismatch, or
equal-state object under another ID is `Conflict` or `WrongBinding` and prevents release.

## Sealed launch capability and inherited inputs

Only world-service may construct `ManagedGatewayLaunchCapabilityV1`. After resolving the published
dormant head and exact-validating the intent, listener, Deny boundary, Prepared handoff, gateway
artifact, config bytes, and all held OS objects, it first-writer-publishes the immutable launch input
and exact-readback-resolves its ref. The sealed capability is neither JSON nor a public Rust
construction surface; it is non-cloneable, cannot be reconstructed from any ref alone, and is
consumed by one spawn attempt.

The gateway config root is exactly
`/run/substrate/e3-gateway/<series-id>/<fence-id>`, physically descriptor-pinned, owned by the
bootstrap-authenticated target UID/primary GID, and mode `0700`. The fixed parent is root-owned mode
`0711`; world-service constructs a complete same-parent
`.e3-gateway-realization-tmp.<lowercase-uuidv7>`, file/directory-fsyncs and descriptor-readback-validates
it, then installs it by no-replace rename and fsyncs the parent. `config.toml` is mode `0600`,
UTF-8/LF, ends in one LF, and is exactly the existing
`cli:codex-world` binding rendered with the reserved port:

```toml
[server]
host = "127.0.0.1"
port = <listener-port>
log_level = "info"

[router]
default = "codex"

[[providers]]
name = "openai-codex"
provider_type = "openai"
auth_type = "oauth"
oauth_provider = "openai-codex"
models = ["codex-mini-latest"]
enabled = true

[[models]]
name = "codex"

[[models.mappings]]
priority = 1
provider = "openai-codex"
actual_model = "codex-mini-latest"
```

Recovery accepts that directory temporary only in the fixed parent with a closed exact entry set,
matching config bytes/hash/UID/GID/modes, and no evidence that a child could have owned it. An equal
installed final permits removal plus parent fsync; an absent final permits one complete no-replace
install. Truncation, unknown entry, symlink, unequal contender, post-ownership ambiguity, or changed
descriptor identity fails closed; after possible child ownership, world-service kills the exact
gateway cgroup and revokes the boundary before deleting any rebuildable realization.

The angle-bracketed port is replaced by its shortest decimal form. No credential, environment
reference, extra provider, or caller-selected model is present. `gateway_config.sha256` is ordinary
SHA-256 over these final bytes. E3 does not own or change the existing routed-to-actual model mapping;
any baseline change to it requires fresh authority rather than silent renderer drift.

World-service serializes `ManagedGatewayLaunchInputV1` as the projection canonical JSON onto a fresh
non-secret `pipe2(O_CLOEXEC)` and passes only its read end. It separately serializes the bounded
`E3WorldFsEnforcementInputV1` for role `ManagedGateway`, derived from the projection's already
authenticated immutable E2 launch snapshot with `support_policy_version = 1`; the wrapper derives
the complete fixed support closure in the controlling
projection contract. It also creates a fresh nonsecret `pipe2(O_CLOEXEC)` for the secret-ready
barrier, retains only its read end, and clears close-on-exec only on the writer passed through the
single wrapper/gateway exec chain. No other process receives either end. The gateway launch
environment is built from empty inheritance and contains
exactly `HOME` for the private gateway root, `LANG=C.UTF-8`, `LC_ALL=C.UTF-8`, the fixed E3 `PATH`,
`RUST_LOG=error`, `SUBSTRATE_LLM_GATEWAY_MODE=in_world`, the authority-created nonsecret
`SUBSTRATE_LLM_GATEWAY_CONFIG_PATH`, `SUBSTRATE_LLM_GATEWAY_DISABLE_TOKEN_PERSISTENCE=1`, plus the
gateway-specific decimal-FD pointers:

```text
SUBSTRATE_E3_GATEWAY_LAUNCH_FD
SUBSTRATE_E3_GATEWAY_LISTENER_FD
SUBSTRATE_E3_GATEWAY_SECRET_READY_FD
SUBSTRATE_LLM_AUTH_BUNDLE_FD
```

The immediate executable is not the gateway. World-service uses the same manifest-pinned ELF
`substrate-world-entry` helper and two-stage protocol defined by the projection contract, with
`SUBSTRATE_WORLD_ENTRY_ROLE=managed_gateway`; its five wrapper pointers identify the held gateway
ELF, gateway working directory, enforcement-input reader, setup-ready writer, and final-exec reader.
The launch-input, reserved-listener, secret-ready write end, and empty secret-handoff read end also
survive the wrapper exec;
no cgroup, nftables, registry, installer-store, or other enforcement-control descriptor does. The
parent attaches the gated PID to the exact gateway cgroup before releasing wrapper exec. The wrapper
derives the exact E2 plan and installs the projection contract's derived-support layer followed by
its gateway role-narrowing layer, which carries no E2 workspace/project right, then clears all
ambient/effective/permitted/inheritable/bounding
capabilities, drops to the installed UID/GID with no supplementary groups, installs the exact
no-new-privileges/dumpability/ptrace/seccomp posture, and emits the strict
`E3ChildSecurityAttestationV1`. It cannot read the still-empty secret pipe. World-service
exact-validates the attestation and independent `/proc`/pidfd/cgroup/FD/negative-access probes before
writing the one final-exec byte.

The already unprivileged wrapper then uses the held manifest-matched gateway descriptor with
`execveat(AT_EMPTY_PATH)` and argv exactly
`["substrate-gateway","--config",<gateway-realization-config-path>,"start"]`, where the substituted
path is exactly `/run/substrate/e3-gateway/<series-id>/<fence-id>/config.toml`; it never reopens the executable
path. The config is a rebuildable realization of nonsecret launch-input/projection authority, not
authority itself. It is file- and directory-fsynced, descriptor-readback-matched, and fixes the
inherited listener's address/port and existing backend route. Before privilege descent the wrapper
bind-mounts that exact file onto itself read-only in its private mount namespace; the parent validates
the mount through the pinned PID. Same-UID Codex access is denied by its distinct Landlock rules.
The gateway first removes all four pointer variables, reads the launch input to bounded EOF,
closes that FD, canonical-decodes and hash-validates it, and exact-compares it to its configured
process identity. It then adopts the inherited listener, validates its socket/namespace/address/port
identity, and restores close-on-exec before accepting. Before opening or reading the still-empty auth
pipe, that same gateway PID sets both `RLIMIT_CORE` values to zero, calls
`prctl(PR_SET_DUMPABLE, 0)`, requires `prctl(PR_GET_DUMPABLE) == 0`, reads back both zero core limits
and `TracerPid: 0`, and writes exactly one canonical
`E3GatewaySecretReadyAttestationV1` followed by EOF to the dedicated nonsecret pipe. The object is at
most 4,096 bytes; missing EOF, a second/trailing byte, noncanonical encoding, hash mismatch, or any
nonzero posture aborts and closes the secret FD without reading. The ready writer closes before the
gateway invokes the existing one-time `GatewayAuthBundleV1` reader; that reader then blocks until the
parent writes and closes the bundle. No gateway path may restore dumpability or raise either core
limit during the credential lifetime. An extra inherited descriptor, missing pointer,
nondecimal/duplicate FD, alias among the four FDs, wrong owner/type, repeated read, trailing launch
bytes, or any ambient identity value aborts before readiness.

After `/proc/<pinned-pid>/exe` exact-matches the held no-setid/no-file-capability gateway artifact,
the inherited listener inventory still matches, and independent post-exec checks reproduce the
attested UID/GID, zero capability masks, no-new-privileges, `TracerPid=0`, both zero core limits, the
exact boot ID and irreversible Yama `ptrace_scope=3` denial, seccomp mode,
both Landlock layer hashes and their effective-intersection hash, and cgroup, world-service reads the
secret-ready pipe to bounded EOF and exact-validates its hash, gateway/launch/PID/start binding,
`dumpable=0`, zero core limits, and tracer value. That descriptor-pinned self-attestation plus the
parent's live process readback becomes `GatewayProcessIdentityV1.secret_ready_attestation_hash`.
Only then does world-service write the existing `GatewayAuthBundleV1` bytes into the
pipe, closes and scrubs its writer/transient buffer, and publishes the unique `Delivered` handoff
successor. Thus no privileged child can read secret bytes: the pipe descriptor is inherited through
the trusted wrapper, but it remains empty until the same pinned PID is unprivileged and has execed the
gateway. The gateway consumes the existing schema/one-time reader exactly once and closes the FD.
EOF before the write, an early read, wrapper/gateway crash, process identity drift, or any failed
security or secret-ready attestation publishes only the legal redacted failure path and never writes the secret.
The write additionally requires the live process-wide E3-exclusive lease from the projection
contract and a fresh proof that no non-E3 or unclassified child exists. That lease remains held until
the gateway/member cgroups are empty and the boundary is durably revoked; a strict-V1, ordinary
execute, PTY, or compatibility-gateway request cannot race into this credential-bearing interval.

The launch capability selects a dedicated E3 router, not baseline `build_app`. Its complete
method/path allowlist is exactly `GET /health` and `POST /v1/responses`; all other methods or paths are
unregistered and return the framework's non-handler 404/405 without touching OAuth, tokens,
messages, chat completions, structured events, count-tokens, routing, or provider state. `GET
/health` must pass the exact readiness-nonce validator before it returns the schema below and is
intended only for the descriptor-pinned probe. `POST /v1/responses` must pass the exact three-field E3
identity validator before body parsing, tracing, routing, or provider access. No member-accessible
route omits that validator. Both handlers first repeat the gateway's
`PR_GET_DUMPABLE=0`/soft-and-hard-`RLIMIT_CORE=0` readback; a mismatch terminates the gateway without
parsing a request or returning success. The existing general `build_app`, OAuth routes, token mutation routes,
main bind, and port-1455 callback bind remain unchanged only for non-E3 gateway launches and are
unreachable from the sealed E3 launch capability.

The launch-input pipe is non-secret transport, not durable authority; its accepted-home immutable
launch-input object, the intent, and dormant projection are authoritative, and the ACK records its
exact ref. The gateway readiness probe is not a connection from a world-service Tokio thread.
World-service constructs this bounded, nonsecret input from the already validated launch capability:

```rust
struct ManagedGatewayReadinessProbeInputV1 {
    schema_version: u32, // exactly 1
    launch_input_ref: ManagedGatewayLaunchInputRefV1,
    gateway_ref: InWorldGatewayRefV1,
    listener_identity: GatewayListenerIdentityV1,
    readiness_nonce: String,
    readiness_probe_cgroup: CanonicalCgroupIdentityV1,
    target_uid: u64,
    target_gid: u64,
    enforcement_input: E3WorldFsEnforcementInputV1,
    connect_deadline_ms: u32, // exactly 1000 after the start byte
    request_release_deadline_ms: u32, // exactly 1000 after connected-attestation write
    response_deadline_ms: u32, // exactly 1000 after request-release byte
    maximum_response_bytes: u32, // exactly 65536
    input_hash: String,
}

struct ManagedGatewayReadinessProbeConnectedV1 {
    schema_version: u32, // exactly 1
    probe_pid: u32,
    probe_pid_start_time_ticks: u64,
    input_hash: String,
    socket_fd: u32,
    socket_inode: u64,
    local_address: String, // exactly "127.0.0.1"
    local_port: u16,
    peer_address: String, // exactly "127.0.0.1"
    peer_port: u16,
    readiness_probe_cgroup: CanonicalCgroupIdentityV1,
    connected_hash: String,
}

struct ManagedGatewayReadinessProbeResultV1 {
    schema_version: u32, // exactly 1
    probe_pid: u32,
    probe_pid_start_time_ticks: u64,
    input_hash: String,
    connected_hash: String,
    child_security_attestation: E3ChildSecurityAttestationV1,
    http_status: u16, // exactly 200
    response_body_base64: String,
    response_body_sha256: String,
    result_hash: String,
}

struct ManagedGatewayReadinessResponseV1 {
    schema_version: u32, // exactly 1
    readiness_nonce: String,
    launch_input_hash: String,
    gateway_ref: InWorldGatewayRefV1,
    config_projection_identity_hash: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    listener_identity: GatewayListenerIdentityV1,
    secret_handoff_prepared_ref: SecretHandoffRefV1,
    secret_handoff_consumed: bool, // exactly true
    secret_ready_attestation_hash: String,
}
```

The nested `enforcement_input` is complete and exact: role
`ManagedGatewayReadinessProbe`, the same projection identity/E2 authority/snapshot/cgroup/UID/GID as
the activation, null immutable-config source and private realization, the descriptor-pinned
world-entry wrapper as `executable_artifact`, support policy 1, and a valid recomputed enforcement
hash. It is authenticated through the probe `input_hash` and is independently revalidated exactly as
the other two roles; a mismatch between either outer duplicate and the nested input is malformed or
wrong-binding before setup-ready.
The input, connected-attestation, and result pipes each carry exactly one EOF-terminated canonical
JSON object of at most 65,536 bytes. Empty input, missing pipe EOF, trailing/second object, oversize,
strict-decode failure, noncanonical re-encode, or hash mismatch aborts. The result's
`child_security_attestation` must be byte-equal to the already validated setup-ready attestation;
neither copy substitutes for the parent's live readback.

The parent forks through the same prebuilt syscall-only gate, attaches the child to the held
`readiness_probe_cgroup`, and descriptor-execs the already manifest-pinned `substrate-world-entry`
with role `ManagedGatewayReadinessProbe`. Its argv is exactly `["substrate-world-entry"]`, its working
directory is `/`, and its environment starts empty and contains exactly the authenticated literal
`SUBSTRATE_WORLD_ENTRY_ROLE=managed_gateway_readiness_probe` plus these seven decimal FD pointers:

```text
SUBSTRATE_E3_READINESS_PROBE_INPUT_FD
SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD
SUBSTRATE_E3_READINESS_PROBE_START_FD
SUBSTRATE_E3_READINESS_PROBE_CONNECTED_FD
SUBSTRATE_E3_READINESS_PROBE_REQUEST_RELEASE_FD
SUBSTRATE_E3_READINESS_PROBE_RESULT_FD
SUBSTRATE_WORLD_ENTRY_SELF_ARTIFACT_FD
```

The role parser accepts that lowercase literal only for this ABI and routes it exclusively to
`run_managed_gateway_readiness_probe`; a missing, duplicate, differently cased, aliased, nondecimal,
or extra environment entry aborts before any socket syscall or setup-ready output. The seven values
identify respectively the immutable probe-input reader, setup-ready writer, one-byte probe-start
reader, bounded connected-attestation writer, one-byte request-release
reader, bounded result writer, and a duplicate of the exact held wrapper ELF descriptor. The last FD
is used only to byte/metadata/runtime-support match `enforcement_input.executable_artifact`, is closed
before the start byte, and can never select or execute another artifact. It inherits no listener,
gateway secret, config, workspace, cgroup, nftables, registry, or second/final target-executable
descriptor. The wrapper validates the input, self-artifact, and its cgroup,
applies the exact readiness-probe support closure, drops to the target UID/GID with zero groups and
all capability sets zero, installs the common anti-ptrace/no-new-privileges posture and a probe
seccomp profile. Before installing that filter it allocates and fixes all
request/connected-attestation/result/parser buffers, resolves the exact numeric IPv4 listener, reads
the immutable deadline durations, and performs every pre-socket filesystem, cgroup, identity, hash, and
attestation computation. On x86-64 the filter first requires
`AUDIT_ARCH_X86_64`, defaults to `SECCOMP_RET_KILL_PROCESS`, and allows only `read`, `write`, `close`,
`socket`, `connect`, `ppoll`, `getsockopt`, `getsockname`, `getpeername`, `fstat`, `clock_gettime`,
`prctl`, `rt_sigreturn`, `exit`, and `exit_group`.
It argument-filters `socket` to exactly `AF_INET`, `SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK`, and
`IPPROTO_TCP`; `getsockopt` to `SOL_SOCKET/SO_ERROR`; and `clock_gettime` to `CLOCK_MONOTONIC`.
`prctl` is allowed only for `PR_GET_SECCOMP` immediately after filter installation; every other first
argument is killed. That readback must return filter mode 2 before the prebuilt attestation is
written.
There is no `open*`, `fcntl`, memory-map/allocation, fork/clone, exec, namespace, ptrace, netlink, or
second socket syscall after filter installation. Error paths use the preallocated bounded result or
`_exit` and invoke no destructor-dependent syscall.

Seccomp cannot inspect a `connect` sockaddr or prove call cardinality. The descriptor-pinned trusted
role function therefore has a closed state machine that issues exactly one socket and one connect,
compares the complete `sockaddr_in` bytes to the input's numeric listener before the syscall, handles
only `EINPROGRESS` through deadline-bounded `ppoll` plus one `SO_ERROR` read, and never accepts another
address or FD. `getsockname`, `getpeername`, and `fstat` are called only on that returned socket in
the fixed order and their complete results must match the input listener and kernel socket identity.
The wrapper first emits the setup-ready/security attestation and blocks. After independent parent
validation, one start byte permits that same PID to create and connect the socket. While retaining
the established socket, it writes exactly one canonical
`ManagedGatewayReadinessProbeConnectedV1` to its dedicated pipe and blocks on the request-release
FD. The parent
exact-validates its hash, PID/start tuple, input hash, local/peer tuple, socket FD/inode, and cgroup;
separate PID/cgroup readback proves the child remains in the recorded readiness-probe cgroup, and the
boundary readback still names only that cgroup. The peer port must equal the listener port; the kernel-assigned local port must be
nonzero and unequal to it. Only then does the parent
write one request-release byte. The child sends the fixed request, writes one canonical result whose
`connected_hash` exact-matches the validated attestation, closes the socket, and exits. The parent
revalidates PID/cgroup after result readback. EOF, extra bytes, a deadline expiry, or any mismatch at
either barrier kills the probe and aborts activation; neither barrier byte is replayable. Each of
the three deadlines uses the child's `CLOCK_MONOTONIC` reading taken immediately after consuming the
start byte, completing the connected-attestation write, or consuming the request-release byte,
respectively. Failure to receive the request-release byte within its exact interval closes the
socket and exits, and the parent must not write that stale gate afterward.

The probe is an E3-classified child covered by the already-live process-wide exclusive lease; it does
not acquire a non-E3 lease. Its pipe/result bytes are ephemeral evidence and are never authority.
`gateway_health_ready` and `gateway_health_ready_blocking` remain the compatibility implementation
for non-E3 gateways; the E3 path cannot call them. With `<port>` as the shortest decimal listener
port and `<nonce>` as the lowercase canonical UUIDv7, the complete request bytes are exactly, with
ASCII and CRLF endings:

```text
GET /health HTTP/1.1\r\nHost: 127.0.0.1:<port>\r\nX-Substrate-E3-Readiness-Nonce: <nonce>\r\nAccept: application/json\r\nConnection: close\r\n\r\n
```

No authorization, body, transfer encoding, provider request, or other header is sent. Partial writes
are completed from the one fixed buffer before the response deadline. The E3 gateway returns exactly
`HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: <body-length>\r\nConnection: close\r\n\r\n<body>`, with headers in that order, shortest-decimal body length, and no extra header or trailing
byte. `<body>` is the unique canonical JSON response below. The probe reads through the declared
length and then requires EOF; a malformed status/header/CRLF, duplicate or unknown header,
chunking/compression, early EOF, bytes beyond the declared body, total bytes greater than 65,536,
connect expiry, connected-state request-release expiry, response expiry, or missing EOF by the
applicable monotonic deadline aborts without ACK. Clock adjustment cannot extend any interval.

The canonical JSON response is available solely to the exact probe cgroup and contains exactly schema 1,
`readiness_nonce`, `launch_input_hash`, `gateway_ref`, `config_projection_identity_hash`,
`orchestration_session_id`, `retained_participant_id`, `backend_id`, `world_id`,
`world_generation`, `listener_identity`, the exact `secret_handoff_prepared_ref`, and
`secret_handoff_consumed = true`, and the exact `secret_ready_attestation_hash`. The gateway cannot
report ready until its existing one-time reader has successfully parsed the bundle, installed the
in-memory auth context, and repeated the zero dumpability/core-limit readback. World-service
exact-compares every field, including `secret_ready_attestation_hash` against
`GatewayProcessIdentityV1.secret_ready_attestation_hash`, then publishes the unique `Consumed`
successor of the already durable
`Delivered` revision before ACK
publication. No URL, header, successful provider request, or launch-input pipe alone substitutes for
that ACK.

## Dormant, activation, and two-stage child protocol

The only valid sequence is:

1. After strict acceptance of `POST /v1/e3/config-projection/prepare`, move its integrated auth into
   the process-local credential capability and acquire the E3-exclusive admission. Under the
   accepted-home projection root lock, preallocate the dormant record ID/revision and every gateway,
   cgroup, and boundary ID, then reserve and retain the sole listener. For each gateway,
   readiness-probe, and allowed-member parent cgroup, publish and exact-readback its immutable
   kernel-effect intent before creating the fixed-name empty cgroup, then publish and exact-readback
   its child-cgroup registration. Publish the socket-bound gateway identity, then publish and
   exact-readback the boundary kernel-effect intent before atomically installing and reading back
   the initial `DenyAllDormant` boundary. Only then call `listen(16)` and pass the exact accepting-
   listener/empty-queue observation wall above. Publish the non-secret credential ref and `Prepared`
   handoff revision/head, and exact activation intent
   containing the request's preparation ID and those dormant coordinates; then
   publish that dormant projection record and head with `ZeroLiveClosed` and no ACK, acquire and
   exact-readback the revision-1 Held dispatch-consumer lease, and only then return the preparation
   response. The intent
   contains no record hash, so this is an acyclic first-writer chain. The projection head is the last
   visible publication; a crash before it leaves no activatable epoch and is handled by the specified
   recovery rules. Assert no member child for the series exists before publication. This is the
   launch-inhibiting zero-live publication fence.
2. Revalidate the active head, E2 link, intent, descriptor-pinned gateway, and deny-all boundary.
   This step begins only when world-service accepts a strict `MemberDispatchRequestV2` whose carrier
   names the dormant head, prepared intent/gateway/fence, and pre-acquired dispatch lease and joins
   the same still-live sealed preparation; no ACK or secret is present in or required from that
   carrier. World-service, not the shell, owns all remaining steps within that one launch request.
   Create one fresh empty landed `GatewayAuthBundleV1` pipe for the exact gateway process; retain the
   unwritten secret bytes solely in world-service transient memory. The gateway may not bind a
   replacement socket. Construct the sealed launch capability and descriptor-exec the pinned wrapper
   behind its parent cgroup gate with the nonsecret launch input, listener, enforcement input, and
   empty secret reader. The wrapper applies the exact derived-support and gateway role-narrowing
   filesystem layers, privilege drop,
   capability clearing, anti-ptrace posture, and seccomp filter, emits its security attestation, and
   blocks. Only the readiness probe can connect; Codex and gateway do not yet exist.
3. Exact-validate the wrapper security attestation and independent live process state, then release
   the final wrapper gate. Require that the same pinned unprivileged PID descriptor-execed the exact
   gateway artifact and retained exactly the allowed listener/carrier descriptors. Require its
   canonical secret-ready attestation and live readback to prove `Dumpable=0`, both core limits zero,
   and the exact launch/PID/start binding. Only now write the secret bundle, close/scrub the parent
   side, and publish the unique `Delivered` handoff successor.
   The gateway alone consumes and closes the read end and may expose readiness only after installing
   its in-memory auth context. World-service closes its write end and every transient secret buffer.
   The durable handoff remains `Delivered` until the exact readiness response is validated. Any failed spawn, parse,
   receiver mismatch, expiry, or duplicate read closes both ends, revokes the boundary, records only
   a redacted failure, publishes the legal `Prepared|Delivered -> Failed|Expired` terminal successor,
   and requires a new preparation/handoff/intent attempt.
4. Probe the exact listener using the public readiness nonce, validate the returned launch-input hash,
   nonce, Prepared handoff ref, consumed Boolean, and fixed gateway identity tuple. Require the
   connected-state attestation barrier and parent observation above before releasing the HTTP
   request; after its result, revalidate process
   executable/listener/access boundary; CAS the handoff head from `Delivered` to its unique terminal `Consumed`
   successor; then publish one immutable non-secret ACK whose `dormant_projection_ref` names the
   already committed dormant record and whose handoff ref names that successor. Provider success is
   not readiness evidence.
5. Publish a successor `ReadyClosed` projection revision containing the ACK. This remains acyclic:
   the ACK points to its dormant predecessor, never to the record that contains the ACK. Create the
   Codex child with prebuilt argv/environment/descriptors behind the wrapper-exec gate, attach it to
   the exact member cgroup, and release only that gate. The pinned wrapper creates the private config
   namespace, applies the authenticated E2 filesystem policy, completes the exact UID/GID/capability/
   no-new-privileges/anti-ptrace/seccomp transition, emits the exact setup-ready and child-security
   attestations, and blocks on the independent
   final-exec gate; Codex does not exist and cannot connect yet.
6. Immediately before activation, resolve the configured accepted-home active head and revalidate
   the carrier's dormant predecessor and the current ReadyClosed record, E2 link, intent, zero-live
   fence, ACK, gateway, access boundary, and all three artifact descriptors. CAS the boundary from
   the ACK's exact `DenyAllDormant` ref to its unique
   `AllowExactMember` successor for the exact child cgroup and publish the `Active/Released`
   projection revision with that successor ref.
7. Immediately before final-exec release for the initial or any later sequential resumed turn,
   validate the setup-ready/security attestation and child namespace,
   then repeat the same authority and live-identity validation, including equality with the
   just-published active head and complete ordered rule set. Write exactly one final-exec byte; the
   wrapper revalidates and descriptor-execs Codex.
   Any mismatch kills the unreleased child, revokes the rule, and leaves no live member.

`Active/Released` durably releases the retained projection/gateway capability, not an individual
Codex execution and not evidence by itself that any child ran. The initial and every sequential
resumed turn still require their exact E2 launch/turn authority, process-local security attestation,
and one-shot barrier; the landed E2 durable join owns resumed-turn retry truth. Recovery never blindly
releases a barrier: if an exact pinned turn child is provably live in the allowed cgroup it retains the
active epoch without another release; if it is provably unreleased or terminal it kills any remnant
without treating the record as execution proof; if observation is ambiguous it kills the entire bound
member cgroup and revokes before failing closed. No recovery path launches a concurrent/duplicate
turn child, replays a barrier write, or reuses the ACK.

The gateway may be credential-ready while `Dormant` or `ReadyClosed`, but it is inaccessible to the
member. At runtime `Active` may be treated as live only when the exact boundary is allowed, the
retained capability/lease and gateway process validate, and any current turn child is exact-bound to
that runtime; the record alone is not process-execution evidence. Ordinary completion of one turn
does not revoke a resumable runtime. On retained-runtime termination, launch failure before a valid
session, world-generation change, cap invalidation, or series retirement, world-service first moves
the boundary to `Revoked`, verifies denial, then terminates or retires the gateway. A service restart
uses a new process identity, handoff, intent, ACK, and projection fence/root revision in the same
series when the immutable subject is unchanged; it never replays the launch input, consumed FD, or
ACK. A fresh series is permitted only when an immutable subject field changed. After verified process
death and boundary revocation, recovery may delete the descriptor-revalidated rebuildable `/run`
gateway config/root and logs; durable launch-input/intent/ACK evidence remains retained and no
secure-erasure claim is made.

## Headers and authorization

Codex sends only these E3-added inbound identity headers:

```text
X-Substrate-Orchestration-Session
X-Substrate-Participant
X-Substrate-Projection
```

They are non-secret correlation metadata. The gateway exact-compares them to its active projection
before parsing every `POST /v1/responses` request and rejects missing, duplicate, or mismatched
values; `X-Substrate-Projection` is exactly the stable
`ConfigProjectionIdentityV1.identity_hash`, not a revision-specific record ID or hash. Matching values
do not authorize a connection. The kernel cgroup/nftables access boundary authorizes connectivity
before HTTP parsing. Upstream
`Authorization` and `ChatGPT-Account-ID` are separately derived by the gateway from the consumed
`GatewayAuthBundleV1`; they are not copied from the three identity headers and never return to Codex.

`codex_base_url` is exactly `http://127.0.0.1:<port>/v1` for the bound listener and is non-secret.
Codex's custom provider has no `env_key`, bearer token, command auth, `requires_openai_auth`, or
environment-derived HTTP header. A caller-supplied URL, an endpoint from a different gateway
lifecycle response, or an ambient `OPENAI_BASE_URL` is rejected.

`GET /health` accepts none of those three member headers as a substitute for its exact single
`X-Substrate-E3-Readiness-Nonce` value. The nonce is not placed in the member projection, environment,
argv, or native config. Every other route/method is absent in E3 mode as specified above.

## Secret-FD noninterference

The existing one-time FD carrier remains the only contract-correct secret delivery path. E3 may add
identity/ref/state capture around it but must preserve all of these invariants:

- the handoff's `SecureFd.fd_name` is exactly `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, all four delivery
  Booleans are true, and the gateway environment uses that same exact pointer name;
- secret bytes are never persisted in projection, intent, ACK, native config, manifests, receipts,
  traces, logs, crash output, or gateway runtime config;
- before its E3 UDS can accept a byte, world-service has set/read back soft and hard
  `RLIMIT_CORE=0` and `PR_GET_DUMPABLE=0`; the gateway likewise proves those values behind the
  secret-ready barrier, and neither process restores them during its remaining lifetime;
- the four gateway FD-pointer variables exist only in the wrapper/gateway launch environment, while
  the wrapper-only pointer variables are removed before gateway exec; the gateway removes all four
  nonsecret/secret pointers before parsing their carriers;
- all raw-secret environment variables are removed, and the gateway launch environment is built
  explicitly rather than inherited;
- the empty pipe read end is inheritable only across the pinned wrapper and single descriptor-pinned
  gateway exec; world-service writes secret bytes only after the wrapper's security attestation,
  exact gateway-exec proof, and secret-ready non-dumpable/core-limit barrier, and all other FDs remain
  close-on-exec;
- the parent write end and gateway read end close after the one delivery/consume attempt;
- Codex, the E3 local adapter, UAA compatibility wrappers, MCP children, shell commands, and provider
  child processes receive none of the gateway launch FDs, secret-ready FD, wrapper-control FDs,
  pointer variables, or secret bytes;
  the nonsecret launch input is also not forwarded; and
- retry/restart creates a fresh descriptor and fresh handoff ID; terminal handoffs cannot reopen.

The non-secret ACK proves exact consumer identity, handoff terminal state, readiness challenge, and
access-boundary posture. It deliberately contains no secret-derived digest, token prefix, credential
file locator, reusable authentication value, or claim that provider credentials are valid upstream.

## Compatibility and failure behavior

Copied host auth/config remains a separately policy-granted, named, logged
`CompatibilityCopyBridge` only. It creates no gateway intent/ACK/ref, cannot make a shared loopback
listener acceptable, cannot satisfy an E3 gate, and is never automatic fallback for missing,
malformed, newer, legacy, wrong-bound, hash-invalid, stale, partially published, or conflicting E3
authority. Gateway absence or an unsupported kernel boundary is typed `UnsupportedLegacyState` or
`UnsupportedPlatform`; missing authenticated E2/Landlock confinement, privilege/capability descent,
anti-ptrace posture, or control-path denial is `UnsupportedSecurityPosture`; corruption/substitution
is fail-closed.

This contract adds no implementation evidence and marks no gate green. E3 still requires later fresh
admission and explicit dispatch; D1 later consumes the ACK/projection capability in its V3 carrier,
and D3 later owns final end-to-end integration proof.
